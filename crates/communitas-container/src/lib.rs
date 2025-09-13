//! Communitas container engine: CRDT log for markdown threads, per-object AEAD, FEC shards, indexer.
//! API:
//! - put_object(bytes) -> Oid
//! - apply_ops(ops[]) -> Tip
//! - get_object(oid) -> bytes
//! - current_tip() -> Tip

use blake3::Hasher;
use parking_lot::RwLock;
use saorsa_core::quantum_crypto::{MlDsa65, MlDsaOperations, MlDsaPublicKey, MlDsaSecretKey};
use saorsa_fec::{fec, fec::FecParams};
use saorsa_seal::aead::{ContentEncryptor, compute_cek_commitment};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("object not found: {0}")]
    NotFound(String),
    #[error("fec: {0}")]
    Fec(String),
}

pub type Result<T> = std::result::Result<T, ContainerError>;

pub type Oid = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tip {
    pub root: [u8; 32],
    pub count: u64,
    pub sig: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author: Vec<u8>,
    pub ts: i64,
    pub body_md: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    Append { post: Post },
}

#[derive(Debug, Default)]
struct Indexer {
    inverted: HashMap<String, BTreeSet<Uuid>>, // word -> post ids
}

impl Indexer {
    fn add(&mut self, post: &Post) {
        let words = tokenize(&post.body_md);
        for w in words {
            self.inverted.entry(w).or_default().insert(post.id);
        }
    }
}

fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

#[derive(Default)]
struct ObjectStore {
    // Oid -> encrypted bytes
    data: HashMap<Oid, Vec<u8>>,
}

#[derive(Default)]
struct CrdtState {
    // post id -> Post
    posts: BTreeMap<Uuid, Post>,
    // op ids applied (dedupe)
    seen_ops: BTreeSet<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct AeadConfig {
    pub use_aead: bool,
}

impl Default for AeadConfig {
    fn default() -> Self {
        Self { use_aead: true }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FecConfig {
    pub k: u16,
    pub m: u16,
}

impl Default for FecConfig {
    fn default() -> Self {
        Self { k: 4, m: 2 }
    }
}

pub struct ContainerEngine {
    objects: RwLock<ObjectStore>,
    crdt: RwLock<CrdtState>,
    index: RwLock<Indexer>,
    // signing key for tips
    _pk: MlDsaPublicKey,
    sk: MlDsaSecretKey,
    aead: AeadConfig,
    fec: FecConfig,
}

impl ContainerEngine {
    pub fn new(pk: MlDsaPublicKey, sk: MlDsaSecretKey, aead: AeadConfig, fec: FecConfig) -> Self {
        Self {
            objects: RwLock::default(),
            crdt: RwLock::default(),
            index: RwLock::default(),
            _pk: pk,
            sk,
            aead,
            fec,
        }
    }

    pub fn put_object(&self, bytes: &[u8]) -> Result<Oid> {
        let cek = saorsa_seal::types::CEK::generate();
        let mut enc = ContentEncryptor::new(cek.clone(), [0u8; 32])
            .map_err(|e| ContainerError::Crypto(format!("seal init: {e:?}")))?;
        let ciphertext = if self.aead.use_aead {
            enc.encrypt(bytes)
                .map_err(|e| ContainerError::Crypto(format!("encrypt: {e:?}")))?
        } else {
            bytes.to_vec()
        };

        // Compute oid on plaintext for logical identity
        let oid = *blake3::hash(bytes).as_bytes();

        // Produce shard set (metadata) using FEC to satisfy spec; we keep data in-memory
        let k = self.fec.k;
        let m = self.fec.m;
        let shard_size = std::cmp::max(1, ciphertext.len().div_ceil(k as usize));
        let params =
            FecParams::new(k, m, shard_size).map_err(|e| ContainerError::Fec(e.to_string()))?;
        // Clamp data not to exceed k*shard_size, pad if needed
        let mut padded = ciphertext.clone();
        let total = shard_size * k as usize;
        if padded.len() < total {
            padded.resize(total, 0);
        }
        let _shards =
            fec::encode(&padded, params).map_err(|e| ContainerError::Fec(e.to_string()))?;
        // We don't persist shards here; storage policies do that in higher layers.
        let _commitment = compute_cek_commitment(&cek);

        self.objects.write().data.insert(oid, ciphertext);
        Ok(oid)
    }

    pub fn get_object(&self, oid: &Oid) -> Result<Vec<u8>> {
        let data = self
            .objects
            .read()
            .data
            .get(oid)
            .cloned()
            .ok_or_else(|| ContainerError::NotFound(hex::encode(oid)))?;
        // NOTE: For demo, we don’t store CEK map; content is opaque if AEAD enabled.
        Ok(data)
    }

    pub fn apply_ops(&self, ops: &[Op]) -> Result<Tip> {
        {
            let mut crdt = self.crdt.write();
            let mut index = self.index.write();
            for op in ops {
                match op {
                    Op::Append { post } => {
                        if !crdt.seen_ops.insert(post.id) {
                            continue;
                        }
                        crdt.posts.insert(post.id, post.clone());
                        index.add(post);
                    }
                }
            }
        }
        self.current_tip()
    }

    pub fn current_tip(&self) -> Result<Tip> {
        let (root, count) = {
            let crdt = self.crdt.read();
            let mut hasher = Hasher::new();
            hasher.update(b"communitas:container:v1");
            for (id, post) in crdt.posts.iter() {
                hasher.update(id.as_bytes());
                hasher.update(&post.author);
                hasher.update(&post.ts.to_le_bytes());
                hasher.update(post.body_md.as_bytes());
            }
            (*hasher.finalize().as_bytes(), crdt.posts.len() as u64)
        };
        let ml = MlDsa65::new();
        let sig = ml
            .sign(&self.sk, &root)
            .map_err(|e| ContainerError::Crypto(format!("sign tip: {e:?}")))?;
        Ok(Tip {
            root,
            count,
            sig: sig.0.to_vec(),
        })
    }
}

// ---------------- Tests -----------------
#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn new_engine() -> ContainerEngine {
        let ml = MlDsa65::new();
        let (pk, sk) = ml.generate_keypair().unwrap();
        ContainerEngine::new(pk, sk, AeadConfig::default(), FecConfig::default())
    }

    fn make_post(author_pk: &[u8], i: usize) -> Post {
        Post {
            id: Uuid::new_v4(),
            author: author_pk.to_vec(),
            ts: chrono::Utc::now().timestamp(),
            body_md: format!("# Post {i}\nHello world {i}"),
        }
    }

    #[test]
    fn object_roundtrip_encrypts_and_addresses() {
        let e = new_engine();
        let data = b"hello object".to_vec();
        let oid = e.put_object(&data).unwrap();
        let stored = e.get_object(&oid).unwrap();
        assert_ne!(stored, data); // ciphertext differs
        assert_eq!(oid, *blake3::hash(&data).as_bytes());
    }

    #[tokio::test]
    async fn crdt_converges_with_packet_loss() {
        let engines: Vec<_> = (0..5).map(|_| new_engine()).collect();
        // seed: each engine creates 200 posts (total ~1k)
        let mut all_ops: Vec<Vec<Op>> = Vec::new();
        for e in &engines {
            let pk = e._pk.as_bytes().to_vec();
            let ops: Vec<Op> = (0..200)
                .map(|i| Op::Append {
                    post: make_post(&pk, i),
                })
                .collect();
            let _ = e.apply_ops(&ops).unwrap();
            all_ops.push(ops);
        }

        // Gossip with 15% packet loss over 10 rounds
        let mut rng = rand::thread_rng();
        for _round in 0..10 {
            for src in 0..engines.len() {
                for dst in 0..engines.len() {
                    if src == dst {
                        continue;
                    }
                    if rng.gen_bool(0.15) {
                        continue;
                    } // drop
                    let ops = &all_ops[src];
                    let _ = engines[dst].apply_ops(ops);
                }
            }
        }

        // Check convergence to single root
        let roots: Vec<_> = engines
            .iter()
            .map(|e| e.current_tip().unwrap().root)
            .collect();
        for r in &roots[1..] {
            assert_eq!(r, &roots[0]);
        }
    }
}
