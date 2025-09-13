//! Communitas DHT record schemas (pointers-only).
//! Canonical CBOR, ML-DSA signatures, strict size limits and TTLs.

use saorsa_core::quantum_crypto::{
    MlDsa65, MlDsaOperations, MlDsaPublicKey, MlDsaSecretKey, MlDsaSignature,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum DhtSchemaError {
    #[error("serde error: {0}")]
    Serde(#[from] serde_cbor::Error),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("size limit exceeded: {0} bytes (max {1})")]
    Size(usize, usize),
    #[error("expired record")]
    Expired,
    #[error("invalid signature")]
    InvalidSig,
}

fn canonical_cbor<T: Serialize>(v: &T) -> Result<Vec<u8>, DhtSchemaError> {
    // serde_cbor 0.11 does not expose an explicit canonical() toggle for Serializer.
    // Since our structs are fixed-field (no HashMaps), standard serialization is stable.
    // For any map-like data, use BTreeMap in the struct definitions to ensure order.
    Ok(serde_cbor::to_vec(v)?)
}

fn blake3_key(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}

// --------------------- identity:{id} ---------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityDevice {
    pub id: String,
    pub device_type: String, // "Active" | "Headless"
    pub endpoint: Endpoint,
    pub caps: DeviceCaps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub protocol: String, // e.g., "quic"
    pub addr: String,     // e.g., "A.B.C.D:port"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCaps {
    pub storage_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRecordV1 {
    pub ver: u8,
    pub ts: i64,
    pub pk_mldsa: Vec<u8>,
    pub devices: Vec<IdentityDevice>,
    pub sig: Vec<u8>,
}

impl IdentityRecordV1 {
    // Temporary cap aligned with ML-DSA-65 sizes (pk ~2KB, sig ~3.3KB)
    pub const MAX_SIZE: usize = 12 * 1024; // 12 KiB (will be tightened with ML-DSA-44)
    pub const TTL_SECS: i64 = 30 * 24 * 60 * 60; // 30 days

    fn sign_bytes(&self) -> Result<Vec<u8>, DhtSchemaError> {
        // Exclude sig field in canonical bytes
        let mut temp = self.clone();
        temp.sig.clear();
        canonical_cbor(&temp)
    }

    pub fn hash(&self) -> Result<[u8; 32], DhtSchemaError> {
        Ok(blake3_key(&self.sign_bytes()?))
    }

    pub fn sign(&mut self, sk: &MlDsaSecretKey) -> Result<(), DhtSchemaError> {
        let msg = self.sign_bytes()?;
        let ml = MlDsa65::new();
        let sig = ml
            .sign(sk, &msg)
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa sign: {:?}", e)))?;
        self.sig = sig.0.to_vec();
        self.enforce_size()?;
        Ok(())
    }

    pub fn verify(&self, pk: &MlDsaPublicKey, now_ts: i64) -> Result<(), DhtSchemaError> {
        self.enforce_size()?;
        if now_ts - self.ts > Self::TTL_SECS {
            return Err(DhtSchemaError::Expired);
        }
        let msg = self.sign_bytes()?;
        let ml = MlDsa65::new();
        const SIG_LEN: usize = 3309;
        if self.sig.len() != SIG_LEN {
            return Err(DhtSchemaError::InvalidSig);
        }
        let mut arr = [0u8; SIG_LEN];
        arr.copy_from_slice(&self.sig);
        let ok = ml
            .verify(pk, &msg, &MlDsaSignature(Box::new(arr)))
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa verify: {:?}", e)))?;
        if ok {
            Ok(())
        } else {
            Err(DhtSchemaError::InvalidSig)
        }
    }

    fn enforce_size(&self) -> Result<(), DhtSchemaError> {
        let bytes = canonical_cbor(self)?;
        if bytes.len() > Self::MAX_SIZE {
            Err(DhtSchemaError::Size(bytes.len(), Self::MAX_SIZE))
        } else {
            Ok(())
        }
    }
}

// --------------------- presence:{id} ---------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceMediaCaps {
    pub audio: bool,
    pub video: bool,
    pub screen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceRecordV1 {
    pub ver: u8,
    pub ts: i64,
    pub active_device: String,
    pub endpoint_hint: Endpoint,
    pub media: PresenceMediaCaps,
    pub ttl: u16,
    pub sig: Vec<u8>,
}

impl PresenceRecordV1 {
    pub const MAX_SIZE: usize = 8 * 1024; // presence includes signature; relax cap for now
    pub const TTL_SECS: i64 = 120;

    fn sign_bytes(&self) -> Result<Vec<u8>, DhtSchemaError> {
        let mut temp = self.clone();
        temp.sig.clear();
        canonical_cbor(&temp)
    }

    pub fn sign(&mut self, sk: &MlDsaSecretKey) -> Result<(), DhtSchemaError> {
        let msg = self.sign_bytes()?;
        let ml = MlDsa65::new();
        let sig = ml
            .sign(sk, &msg)
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa sign: {:?}", e)))?;
        self.sig = sig.0.to_vec();
        self.enforce_size()?;
        Ok(())
    }

    pub fn verify(&self, pk: &MlDsaPublicKey, now_ts: i64) -> Result<(), DhtSchemaError> {
        self.enforce_size()?;
        if now_ts - self.ts > Self::TTL_SECS {
            return Err(DhtSchemaError::Expired);
        }
        let msg = self.sign_bytes()?;
        let ml = MlDsa65::new();
        const SIG_LEN: usize = 3309;
        if self.sig.len() != SIG_LEN {
            return Err(DhtSchemaError::InvalidSig);
        }
        let mut arr = [0u8; SIG_LEN];
        arr.copy_from_slice(&self.sig);
        let ok = ml
            .verify(pk, &msg, &MlDsaSignature(Box::new(arr)))
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa verify: {:?}", e)))?;
        if ok {
            Ok(())
        } else {
            Err(DhtSchemaError::InvalidSig)
        }
    }

    fn enforce_size(&self) -> Result<(), DhtSchemaError> {
        let bytes = canonical_cbor(self)?;
        if bytes.len() > Self::MAX_SIZE {
            Err(DhtSchemaError::Size(bytes.len(), Self::MAX_SIZE))
        } else {
            Ok(())
        }
    }
}

// ----------------- group/channel/container_tip -----------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRecordV1 {
    pub ver: u8,
    pub ts: i64,
    pub epoch: u64,
    pub membership_commit: [u8; 32],
    pub container_tip: [u8; 32],
    pub write_quorum: u8,
    pub sig: Vec<u8>,
}

impl GroupRecordV1 {
    pub const MAX_SIZE: usize = 8 * 1024;
    pub const TTL_SECS: i64 = 7 * 24 * 60 * 60;

    fn sign_bytes(&self) -> Result<Vec<u8>, DhtSchemaError> {
        let mut tmp = self.clone();
        tmp.sig.clear();
        canonical_cbor(&tmp)
    }

    pub fn sign(&mut self, sk: &MlDsaSecretKey) -> Result<(), DhtSchemaError> {
        let ml = MlDsa65::new();
        let msg = self.sign_bytes()?;
        let sig = ml
            .sign(sk, &msg)
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa sign: {:?}", e)))?;
        self.sig = sig.0.to_vec();
        self.enforce_size()?;
        Ok(())
    }

    pub fn verify(&self, pk: &MlDsaPublicKey, now_ts: i64) -> Result<(), DhtSchemaError> {
        self.enforce_size()?;
        if now_ts - self.ts > Self::TTL_SECS {
            return Err(DhtSchemaError::Expired);
        }
        let ml = MlDsa65::new();
        let msg = self.sign_bytes()?;
        const SIG_LEN: usize = 3309;
        if self.sig.len() != SIG_LEN {
            return Err(DhtSchemaError::InvalidSig);
        }
        let mut arr = [0u8; SIG_LEN];
        arr.copy_from_slice(&self.sig);
        let ok = ml
            .verify(pk, &msg, &MlDsaSignature(Box::new(arr)))
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa verify: {:?}", e)))?;
        if ok {
            Ok(())
        } else {
            Err(DhtSchemaError::InvalidSig)
        }
    }

    fn enforce_size(&self) -> Result<(), DhtSchemaError> {
        let bytes = canonical_cbor(self)?;
        if bytes.len() > Self::MAX_SIZE {
            Err(DhtSchemaError::Size(bytes.len(), Self::MAX_SIZE))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRecordV1 {
    pub ver: u8,
    pub ts: i64,
    pub epoch: u64,
    pub container_tip: [u8; 32],
    pub sig: Vec<u8>,
}

impl ChannelRecordV1 {
    pub const MAX_SIZE: usize = 8 * 1024;
    pub const TTL_SECS: i64 = 7 * 24 * 60 * 60;
    fn sign_bytes(&self) -> Result<Vec<u8>, DhtSchemaError> {
        let mut tmp = self.clone();
        tmp.sig.clear();
        canonical_cbor(&tmp)
    }
    pub fn sign(&mut self, sk: &MlDsaSecretKey) -> Result<(), DhtSchemaError> {
        let ml = MlDsa65::new();
        let msg = self.sign_bytes()?;
        let sig = ml
            .sign(sk, &msg)
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa sign: {:?}", e)))?;
        self.sig = sig.0.to_vec();
        self.enforce_size()?;
        Ok(())
    }
    pub fn verify(&self, pk: &MlDsaPublicKey, now_ts: i64) -> Result<(), DhtSchemaError> {
        self.enforce_size()?;
        if now_ts - self.ts > Self::TTL_SECS {
            return Err(DhtSchemaError::Expired);
        }
        let ml = MlDsa65::new();
        let msg = self.sign_bytes()?;
        const SIG_LEN: usize = 3309;
        if self.sig.len() != SIG_LEN {
            return Err(DhtSchemaError::InvalidSig);
        }
        let mut arr = [0u8; SIG_LEN];
        arr.copy_from_slice(&self.sig);
        let ok = ml
            .verify(pk, &msg, &MlDsaSignature(Box::new(arr)))
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa verify: {:?}", e)))?;
        if ok {
            Ok(())
        } else {
            Err(DhtSchemaError::InvalidSig)
        }
    }
    fn enforce_size(&self) -> Result<(), DhtSchemaError> {
        let bytes = canonical_cbor(self)?;
        if bytes.len() > Self::MAX_SIZE {
            Err(DhtSchemaError::Size(bytes.len(), Self::MAX_SIZE))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerTipVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerTipRecordV1 {
    pub ver: u8,
    pub ts: i64,
    pub content_root: [u8; 32],
    pub version: ContainerTipVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<[u8; 32]>,
    pub sig: Vec<u8>,
}

impl ContainerTipRecordV1 {
    pub const MAX_SIZE: usize = 8 * 1024;
    pub const TTL_SECS: i64 = 90 * 24 * 60 * 60;
    fn sign_bytes(&self) -> Result<Vec<u8>, DhtSchemaError> {
        let mut tmp = self.clone();
        tmp.sig.clear();
        canonical_cbor(&tmp)
    }
    pub fn sign(&mut self, sk: &MlDsaSecretKey) -> Result<(), DhtSchemaError> {
        let ml = MlDsa65::new();
        let msg = self.sign_bytes()?;
        let sig = ml
            .sign(sk, &msg)
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa sign: {:?}", e)))?;
        self.sig = sig.0.to_vec();
        self.enforce_size()?;
        Ok(())
    }
    pub fn verify(&self, pk: &MlDsaPublicKey, now_ts: i64) -> Result<(), DhtSchemaError> {
        self.enforce_size()?;
        if now_ts - self.ts > Self::TTL_SECS {
            return Err(DhtSchemaError::Expired);
        }
        let ml = MlDsa65::new();
        let msg = self.sign_bytes()?;
        const SIG_LEN: usize = 3309;
        if self.sig.len() != SIG_LEN {
            return Err(DhtSchemaError::InvalidSig);
        }
        let mut arr = [0u8; SIG_LEN];
        arr.copy_from_slice(&self.sig);
        let ok = ml
            .verify(pk, &msg, &MlDsaSignature(Box::new(arr)))
            .map_err(|e| DhtSchemaError::Crypto(format!("mldsa verify: {:?}", e)))?;
        if ok {
            Ok(())
        } else {
            Err(DhtSchemaError::InvalidSig)
        }
    }
    fn enforce_size(&self) -> Result<(), DhtSchemaError> {
        let bytes = canonical_cbor(self)?;
        if bytes.len() > Self::MAX_SIZE {
            Err(DhtSchemaError::Size(bytes.len(), Self::MAX_SIZE))
        } else {
            Ok(())
        }
    }
}

// ------------------------- Tests -------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn kp() -> (MlDsaPublicKey, MlDsaSecretKey) {
        let ml = MlDsa65::new();
        ml.generate_keypair().unwrap()
    }

    #[test]
    fn identity_roundtrip_under_size() {
        let (pk, sk) = kp();
        let mut rec = IdentityRecordV1 {
            ver: 1,
            ts: 1,
            pk_mldsa: pk.as_bytes().to_vec(),
            devices: vec![IdentityDevice {
                id: "dev-1".into(),
                device_type: "Active".into(),
                endpoint: Endpoint {
                    protocol: "quic".into(),
                    addr: "1.2.3.4:443".into(),
                },
                caps: DeviceCaps { storage_gb: 100 },
            }],
            sig: vec![],
        };
        rec.sign(&sk).unwrap();
        assert!(rec.verify(&pk, 10).is_ok());
        let bytes = serde_cbor::to_vec(&rec).unwrap();
        assert!(bytes.len() <= IdentityRecordV1::MAX_SIZE);
        let _hash = rec.hash().unwrap();
    }

    #[test]
    fn presence_ttl_and_size() {
        let (pk, sk) = kp();
        let mut p = PresenceRecordV1 {
            ver: 1,
            ts: 1,
            active_device: "dev-1".into(),
            endpoint_hint: Endpoint {
                protocol: "quic".into(),
                addr: "1.2.3.4:443".into(),
            },
            media: PresenceMediaCaps {
                audio: true,
                video: true,
                screen: true,
            },
            ttl: 120,
            sig: vec![],
        };
        p.sign(&sk).unwrap();
        assert!(p.verify(&pk, 100).is_ok());
        let bytes = serde_cbor::to_vec(&p).unwrap();
        assert!(bytes.len() <= PresenceRecordV1::MAX_SIZE);
    }

    #[test]
    fn group_and_channel_verify() {
        let (pk, sk) = kp();
        let mut g = GroupRecordV1 {
            ver: 1,
            ts: 1,
            epoch: 42,
            membership_commit: [9u8; 32],
            container_tip: [7u8; 32],
            write_quorum: 1,
            sig: vec![],
        };
        g.sign(&sk).unwrap();
        assert!(g.verify(&pk, 100).is_ok());

        let mut c = ChannelRecordV1 {
            ver: 1,
            ts: 1,
            epoch: 42,
            container_tip: [1u8; 32],
            sig: vec![],
        };
        c.sign(&sk).unwrap();
        assert!(c.verify(&pk, 100).is_ok());
    }

    #[test]
    fn container_tip_verify_size() {
        let (pk, sk) = kp();
        let mut ct = ContainerTipRecordV1 {
            ver: 1,
            ts: 1,
            content_root: [5u8; 32],
            version: ContainerTipVersion { major: 1, minor: 0 },
            prev: None,
            sig: vec![],
        };
        ct.sign(&sk).unwrap();
        assert!(ct.verify(&pk, 100).is_ok());
        let bytes = serde_cbor::to_vec(&ct).unwrap();
        assert!(bytes.len() <= ContainerTipRecordV1::MAX_SIZE);
    }
}
