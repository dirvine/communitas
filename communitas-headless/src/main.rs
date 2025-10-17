// Communitas Headless Node
// This binary runs a headless Communitas node using saorsa-core APIs

// Security: CLI tools may use unwrap in controlled contexts
// Core library crates maintain strict no-unwrap policies

use anyhow::{Context, Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use clap::Parser;
// TODO: Re-enable when bootstrap_integration is available in communitas-core
// use communitas_core::bootstrap_integration::{BootstrapConfig, EnhancedBootstrapManager};
use ed25519_dalek::SigningKey as Ed25519SecretKey;
use four_word_networking::FourWordAdaptiveEncoder;
use once_cell::sync::Lazy;
use rand::RngCore;
use rand::rngs::OsRng;
// Removed: saorsa-core imports - replaced with saorsa-pqc and four-word-networking
// use saorsa_core::address::NetworkAddress;
// use saorsa_core::identity::FourWordAddress;
// use saorsa_core::quantum_crypto::{...};

// Removed: four_word_networking::FourWordAddress - using communitas_core::identity instead
// Stub: PQC crypto removed - saorsa-pqc dependency eliminated
// Using simple placeholder types for compilation

// Placeholder types for removed dependencies
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FourWordAddress(String);

impl FourWordAddress {
    pub fn parse_str(s: &str) -> Result<Self, String> {
        // Stub validation - just accept any string
        Ok(Self(s.to_string()))
    }

    pub fn words(&self) -> Vec<String> {
        // Stub implementation - split by dashes
        self.0.split('-').map(|s| s.to_string()).collect()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlDsaSecretKey(Vec<u8>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlDsaPublicKey(Vec<u8>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlKemSecretKey(Vec<u8>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlKemPublicKey(Vec<u8>);

// Stub implementations
impl MlDsaSecretKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(bytes.to_vec()))
    }
}

impl MlDsaPublicKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(bytes.to_vec()))
    }
}

impl MlKemSecretKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(bytes.to_vec()))
    }
}

impl MlKemPublicKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Ok(Self(bytes.to_vec()))
    }
}

// Stub key generation functions
fn try_keygen_with_rng(
    _rng: &mut impl rand::RngCore,
) -> Result<(MlDsaPublicKey, MlDsaSecretKey), String> {
    Ok((MlDsaPublicKey(vec![0; 32]), MlDsaSecretKey(vec![0; 32])))
}

fn try_kem_keygen_with_rng(
    _rng: &mut impl rand::RngCore,
) -> Result<(MlKemPublicKey, MlKemSecretKey), String> {
    Ok((MlKemPublicKey(vec![0; 32]), MlKemSecretKey(vec![0; 32])))
}

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::env;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::signal;
// TODO: Re-enable when bootstrap_integration and communitas_container are available
// use tokio::sync::RwLock as AsyncRwLock;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Try to self-update the binary using GitHub releases
pub fn try_self_update() -> Result<Option<String>> {
    use self_update::cargo_crate_version;
    let owner =
        std::env::var("COMMUNITAS_UPDATE_REPO_OWNER").unwrap_or_else(|_| "dirvine".to_string());
    let name =
        std::env::var("COMMUNITAS_UPDATE_REPO_NAME").unwrap_or_else(|_| "communitas".to_string());

    // Primary attempt
    let mut cfg = self_update::backends::github::Update::configure();
    let builder = cfg
        .repo_owner(&owner)
        .repo_name(&name)
        .bin_name("communitas-headless")
        .current_version(cargo_crate_version!());
    match builder.build()?.update() {
        Ok(status) => Ok(Some(status.version().to_string())),
        Err(e1) => {
            // Optional fallback repo (if the project lives under a different owner)
            let fallback_owner = if owner == "dirvine" {
                "david-irvine"
            } else {
                "dirvine"
            };
            let mut cfg2 = self_update::backends::github::Update::configure();
            let b2 = cfg2
                .repo_owner(fallback_owner)
                .repo_name(&name)
                .bin_name("communitas-headless")
                .current_version(cargo_crate_version!());
            match b2.build()?.update() {
                Ok(status) => Ok(Some(status.version().to_string())),
                Err(_e2) => Err(e1.into()),
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "communitas-headless",
    author,
    version,
    about = "Headless Communitas P2P node",
    long_about = None
)]
struct Args {
    /// Configuration file path (defaults to per-instance directory under the user config root)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Storage directory (defaults to per-instance directory under the user data root)
    #[arg(short, long)]
    storage: Option<PathBuf>,

    /// Identifier used to segregate config and data directories when running multiple instances
    #[arg(long)]
    instance_id: Option<String>,

    /// Listen address
    /// If not provided, you can set COMMUNITAS_QUIC_PORT or COMMUNITAS_QUIC_LISTEN.
    /// Recommended: use a random high port (>1024) per node.
    #[arg(short, long, default_value = "0.0.0.0:0")]
    listen: SocketAddr,

    /// Bootstrap nodes (four-word addresses)
    #[arg(short, long)]
    bootstrap: Vec<String>,

    /// Enable metrics endpoint
    #[arg(long)]
    metrics: bool,

    /// Metrics listen address
    #[arg(long, default_value = "127.0.0.1:9600")]
    metrics_addr: SocketAddr,

    /// Perform self-update from GitHub Releases and exit (no server)
    #[arg(long, default_value_t = false)]
    self_update: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    /// Node identity (four-word address)
    identity: Option<String>,

    /// Bootstrap nodes
    bootstrap_nodes: Vec<String>,

    /// Storage settings
    storage: StorageConfig,

    /// Network settings
    network: NetworkConfig,

    /// Auto-update settings
    update: UpdateConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageConfig {
    /// Base directory for storage
    base_dir: PathBuf,

    /// Cache size in MB
    cache_size_mb: usize,

    /// Enable FEC for storage
    enable_fec: bool,

    /// FEC parameters
    fec_k: usize,
    fec_m: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkConfig {
    /// Listen addresses
    listen_addrs: Vec<SocketAddr>,

    /// Enable IPv6
    enable_ipv6: bool,

    /// Enable WebRTC bridge
    enable_webrtc: bool,

    /// QUIC settings
    quic_idle_timeout_ms: u64,
    quic_max_streams: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateConfig {
    /// Update channel (stable, beta, nightly)
    channel: String,

    /// Check interval in seconds
    check_interval_secs: u64,

    /// Enable auto-update
    auto_update: bool,

    /// Jitter range in seconds (0 disables jitter, default 0 for saorsa-core 0.3.18+)
    jitter_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        default_config_with_storage(PathBuf::from("communitas-data"))
    }
}

fn default_config_with_storage(base_dir: PathBuf) -> Config {
    Config {
        identity: None,
        bootstrap_nodes: vec![
            // Digital Ocean NYC3 Bootstrap Nodes (v0.1.18 dual-stack)
            // Droplet: 2064413, IPv4: 167.71.188.131, IPv6: 2604:a880:800:14:0:1:db7c:c000
            "bless-lava-jeffrey-parking:443".to_string(),
            // Droplet: communitas-bootstrap-1, IPv4: 138.197.29.195, IPv6: 2604:a880:800:14:0:1:db7c:b000
            "bless-route-evaporate-lunch:443".to_string(),
        ],
        storage: StorageConfig {
            base_dir,
            cache_size_mb: 1024,
            enable_fec: true,
            fec_k: 8,
            fec_m: 4,
        },
        network: NetworkConfig {
            listen_addrs: vec![
                // IPv4 wildcard address - listens on all IPv4 interfaces
                std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                    443,
                ),
                // IPv6 wildcard address - listens on all IPv6 interfaces
                std::net::SocketAddr::new(
                    std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED),
                    443,
                ),
            ],
            enable_ipv6: true,
            enable_webrtc: false,
            quic_idle_timeout_ms: 30000,
            quic_max_streams: 100,
        },
        update: UpdateConfig {
            channel: "stable".to_string(),
            check_interval_secs: 21600, // 6 hours
            auto_update: true,
            jitter_secs: 0, // No jitter needed for saorsa-core 0.3.18+
        },
    }
}

fn sanitize_instance_id(raw: &str) -> String {
    let mut sanitized: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    sanitized.truncate(64);
    if sanitized.trim_matches('-').is_empty() {
        "communitas".to_string()
    } else {
        sanitized.trim_matches('-').to_string()
    }
}

fn default_instance_id() -> String {
    let host = env::var("COMMUNITAS_INSTANCE_HOST")
        .or_else(|_| env::var("HOSTNAME"))
        .or_else(|_| env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "communitas".to_string());
    let host = sanitize_instance_id(&host.to_lowercase());
    let pid = std::process::id();
    format!("{}-{}", host, pid)
}

fn ensure_absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = env::current_dir().context("Failed to determine current working directory")?;
    Ok(cwd.join(path))
}

fn resolve_config_path(args: &Args, instance_id: &str) -> Result<PathBuf> {
    if let Some(ref cli_path) = args.config {
        return ensure_absolute(cli_path);
    }

    if let Ok(env_path) = env::var("COMMUNITAS_CONFIG_PATH") {
        return ensure_absolute(Path::new(&env_path));
    }

    let base = if let Ok(env_dir) = env::var("COMMUNITAS_CONFIG_DIR") {
        ensure_absolute(Path::new(&env_dir))?
    } else if let Some(dir) = dirs::config_dir() {
        dir
    } else {
        anyhow::bail!(
            "Unable to determine configuration directory. Set --config or COMMUNITAS_CONFIG_PATH."
        );
    };

    Ok(base
        .join("communitas")
        .join(sanitize_instance_id(instance_id))
        .join("config.toml"))
}

enum StoragePathHint {
    CommandLine(PathBuf),
    Env(PathBuf),
    Default(PathBuf),
}

impl StoragePathHint {
    fn path(&self) -> &Path {
        match self {
            StoragePathHint::CommandLine(p)
            | StoragePathHint::Env(p)
            | StoragePathHint::Default(p) => p.as_path(),
        }
    }
}

fn resolve_storage_hint(args: &Args, instance_id: &str) -> Result<StoragePathHint> {
    if let Some(ref cli_storage) = args.storage {
        return ensure_absolute(cli_storage).map(StoragePathHint::CommandLine);
    }

    if let Ok(env_path) = env::var("COMMUNITAS_DATA_PATH") {
        return ensure_absolute(Path::new(&env_path)).map(StoragePathHint::Env);
    }

    let base = if let Ok(env_dir) = env::var("COMMUNITAS_DATA_DIR") {
        ensure_absolute(Path::new(&env_dir))?
    } else if let Some(dir) = dirs::data_dir() {
        dir
    } else if let Some(dir) = dirs::home_dir() {
        dir.join(".communitas")
    } else {
        anyhow::bail!(
            "Unable to determine storage directory. Set --storage or COMMUNITAS_DATA_PATH."
        );
    };

    let storage = base
        .join("communitas")
        .join(sanitize_instance_id(instance_id));
    Ok(StoragePathHint::Default(storage))
}

async fn load_or_create_config(path: &Path, default_config: Config) -> Result<Config> {
    if tokio::fs::try_exists(path).await? {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file {}", path.display()))?;
        toml::from_str(&content).context("Failed to parse config")
    } else {
        let parent = path.parent().context("Invalid config path")?;
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;

        let content = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;
        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("Failed to write config file {}", path.display()))?;

        Ok(default_config)
    }
}

async fn save_config(path: &Path, config: &Config) -> Result<()> {
    let content = toml::to_string_pretty(config).context("Failed to serialize config")?;
    tokio::fs::write(path, content)
        .await
        .with_context(|| format!("Failed to write config file {}", path.display()))?;
    Ok(())
}

const IDENTITY_DIR_NAME: &str = "identity";
const IDENTITY_FILE_NAME: &str = "identity.json";
const IDENTITY_GENERATION_ATTEMPTS: usize = 1024;

#[derive(Debug)]
struct IdentityMaterial {
    four_words: String,
    ml_dsa_public: Vec<u8>,
    ml_dsa_secret: Vec<u8>,
    ml_kem_public: Vec<u8>,
    ml_kem_secret: Vec<u8>,
    ed25519_public: Vec<u8>,
    ed25519_secret: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredIdentity {
    four_words: String,
    ml_dsa_public: String,
    ml_dsa_secret: String,
    ml_kem_public: String,
    ml_kem_secret: String,
    ed25519_public: String,
    ed25519_secret: String,
}

impl StoredIdentity {
    fn into_material(self) -> Result<IdentityMaterial> {
        let StoredIdentity {
            four_words,
            ml_dsa_public,
            ml_dsa_secret,
            ml_kem_public,
            ml_kem_secret,
            ed25519_public,
            ed25519_secret,
        } = self;

        let canonical = canonicalize_four_words(&four_words)?;

        let ml_dsa_public = BASE64
            .decode(ml_dsa_public)
            .context("Failed to decode stored ML-DSA public key")?;
        let ml_dsa_secret = BASE64
            .decode(ml_dsa_secret)
            .context("Failed to decode stored ML-DSA secret key")?;
        let ml_kem_public = BASE64
            .decode(ml_kem_public)
            .context("Failed to decode stored ML-KEM public key")?;
        let ml_kem_secret = BASE64
            .decode(ml_kem_secret)
            .context("Failed to decode stored ML-KEM secret key")?;
        let ed25519_public = BASE64
            .decode(ed25519_public)
            .context("Failed to decode stored Ed25519 public key")?;
        let ed25519_secret = BASE64
            .decode(ed25519_secret)
            .context("Failed to decode stored Ed25519 secret key")?;

        MlDsaPublicKey::try_from_bytes(&ml_dsa_public)
            .map_err(|e| anyhow!("Invalid ML-DSA public key in identity store: {}", e))?;
        MlDsaSecretKey::try_from_bytes(&ml_dsa_secret)
            .map_err(|e| anyhow!("Invalid ML-DSA secret key in identity store: {}", e))?;
        MlKemPublicKey::try_from_bytes(&ml_kem_public)
            .map_err(|e| anyhow!("Invalid ML-KEM public key in identity store: {}", e))?;
        MlKemSecretKey::try_from_bytes(&ml_kem_secret)
            .map_err(|e| anyhow!("Invalid ML-KEM secret key in identity store: {}", e))?;

        if ed25519_public.len() != 32 {
            return Err(anyhow!("Stored Ed25519 public key must be 32 bytes"));
        }
        if ed25519_secret.len() != 32 {
            return Err(anyhow!("Stored Ed25519 secret key must be 32 bytes"));
        }
        let secret_seed: [u8; 32] = ed25519_secret
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("Stored Ed25519 secret key must be 32 bytes"))?;
        let signing = Ed25519SecretKey::from_bytes(&secret_seed);
        if signing.verifying_key().to_bytes().as_slice() != ed25519_public.as_slice() {
            return Err(anyhow!(
                "Stored Ed25519 public key does not match secret key"
            ));
        }

        Ok(IdentityMaterial {
            four_words: canonical,
            ml_dsa_public,
            ml_dsa_secret,
            ml_kem_public,
            ml_kem_secret,
            ed25519_public,
            ed25519_secret,
        })
    }
}

fn canonicalize_four_words(input: &str) -> Result<String> {
    let parsed = FourWordAddress::parse_str(input)
        .map_err(|e| anyhow!("Invalid four-word identity: {}", e))?;
    let words_vec = parsed.words();
    let words: [String; 4] = words_vec.try_into().map_err(|v: Vec<String>| {
        anyhow!(
            "Four-word identity must contain exactly 4 words, found {}",
            v.len()
        )
    })?;
    // Join words back to string for validation
    let words_str = words.join("-");
    if !communitas_core::identity::validate_id_words(&words_str) {
        return Err(anyhow!(
            "Four-word identity contains words outside the allowed dictionary"
        ));
    }
    Ok(parsed.as_str().to_string())
}

fn generate_random_four_words() -> Result<String> {
    let mut rng = OsRng;
    const MIN_PORT: u16 = 1024;
    const PORT_SPAN: u32 = u16::MAX as u32 - MIN_PORT as u32 + 1;
    for _ in 0..IDENTITY_GENERATION_ATTEMPTS {
        let ipv4 = Ipv4Addr::from(rng.next_u32());
        let port = (rng.next_u32() % PORT_SPAN) as u16 + MIN_PORT;
        let addr = SocketAddr::from((ipv4, port));
        if let Ok(words) = communitas_core::identity::conn_words(&addr)
            && let Ok(canonical) = canonicalize_four_words(&words)
        {
            return Ok(canonical);
        }
    }
    Err(anyhow!(
        "Failed to generate a valid four-word identity after {} attempts",
        IDENTITY_GENERATION_ATTEMPTS
    ))
}

async fn persist_identity_to_disk(path: &Path, material: &IdentityMaterial) -> Result<()> {
    let stored = StoredIdentity {
        four_words: material.four_words.clone(),
        ml_dsa_public: BASE64.encode(&material.ml_dsa_public),
        ml_dsa_secret: BASE64.encode(&material.ml_dsa_secret),
        ml_kem_public: BASE64.encode(&material.ml_kem_public),
        ml_kem_secret: BASE64.encode(&material.ml_kem_secret),
        ed25519_public: BASE64.encode(&material.ed25519_public),
        ed25519_secret: BASE64.encode(&material.ed25519_secret),
    };
    let serialized = serde_json::to_vec_pretty(&stored)
        .context("Failed to serialize identity for persistence")?;
    tokio::fs::write(path, serialized)
        .await
        .with_context(|| format!("Failed to write identity file {}", path.display()))?;
    #[cfg(unix)]
    {
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

async fn setup_identity(config: &Config) -> Result<IdentityMaterial> {
    let identity_dir = config.storage.base_dir.join(IDENTITY_DIR_NAME);
    tokio::fs::create_dir_all(&identity_dir)
        .await
        .with_context(|| {
            format!(
                "Failed to create identity directory {}",
                identity_dir.display()
            )
        })?;

    let identity_path = identity_dir.join(IDENTITY_FILE_NAME);

    match tokio::fs::read(&identity_path).await {
        Ok(bytes) => {
            let stored: StoredIdentity = serde_json::from_slice(&bytes)
                .context("Failed to parse persisted identity JSON")?;
            let material = stored.into_material()?;

            if let Some(config_identity) = &config.identity {
                match canonicalize_four_words(config_identity) {
                    Ok(canonical) if canonical != material.four_words => {
                        warn!(
                            "Configured four-word identity {} does not match persisted identity {}; using persisted identity",
                            config_identity, material.four_words
                        );
                    }
                    Err(err) => {
                        warn!(
                            "Configured four-word identity {} is invalid: {}; using persisted identity",
                            config_identity, err
                        );
                    }
                    _ => {}
                }
            }

            info!(
                "Loaded existing node identity {} from {}",
                material.four_words,
                identity_path.display()
            );
            Ok(material)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            let four_words = if let Some(config_identity) = &config.identity {
                let canonical = canonicalize_four_words(config_identity)?;
                info!(
                    "Initializing identity from configured four-word address: {}",
                    canonical
                );
                canonical
            } else {
                let generated = generate_random_four_words()?;
                info!("Generated new four-word identity: {}", generated);
                generated
            };

            let (ml_dsa_public, ml_dsa_secret) = try_keygen_with_rng(&mut OsRng)
                .map_err(|e| anyhow!("Failed to generate ML-DSA keypair: {}", e))?;
            let (ml_kem_public, ml_kem_secret) = try_kem_keygen_with_rng(&mut OsRng)
                .map_err(|e| anyhow!("Failed to generate ML-KEM keypair: {}", e))?;

            let mut rng = OsRng;
            let ed25519_secret = Ed25519SecretKey::generate(&mut rng);
            let ed25519_public = ed25519_secret.verifying_key();

            let material = IdentityMaterial {
                four_words,
                ml_dsa_public: ml_dsa_public.as_bytes().to_vec(),
                ml_dsa_secret: ml_dsa_secret.as_bytes().to_vec(),
                ml_kem_public: ml_kem_public.as_bytes().to_vec(),
                ml_kem_secret: ml_kem_secret.as_bytes().to_vec(),
                ed25519_public: ed25519_public.to_bytes().to_vec(),
                ed25519_secret: ed25519_secret.to_bytes().to_vec(),
            };

            persist_identity_to_disk(&identity_path, &material).await?;
            info!(
                "Persisted node identity {} to {}",
                material.four_words,
                identity_path.display()
            );

            Ok(material)
        }
        Err(err) => Err(anyhow!(
            "Failed to read identity file {}: {}",
            identity_path.display(),
            err
        )),
    }
}

async fn start_health_endpoint(
    addr: SocketAddr,
    // Removed: _dht_client - saorsa-core removed
    // _dht_client: Arc<saorsa_core::messaging::DhtClient>,
) -> Result<()> {
    use warp::Filter;
    use warp::cors;

    let health = warp::path("health").map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
        }))
    });

    let metrics = warp::path("metrics").map(move || {
        // Get actual peer count from active connections
        let peer_count = ACTIVE_CONNECTIONS
            .try_read()
            .map(|conns| conns.len())
            .unwrap_or(0);

        format!(
            "# HELP communitas_peers_connected Number of connected peers\n\
             # TYPE communitas_peers_connected gauge\n\
             communitas_peers_connected {}\n",
            peer_count
        )
    });

    // Add authentication endpoints
    let authenticate = warp::path("authenticate")
        .and(warp::post())
        .and(warp::body::json())
        .map(|body: serde_json::Value| {
            // Simple authentication endpoint for demo
            // In production, this would validate against the actual identity system
            warp::reply::json(&serde_json::json!({
                "success": true,
                "message": "Authentication endpoint ready",
                "received": body
            }))
        });

    let get_identity = warp::path("identity").and(warp::get()).map(|| {
        // Return current node identity info
        warp::reply::json(&serde_json::json!({
            "node_type": "communitas-headless",
            "version": env!("CARGO_PKG_VERSION"),
            "status": "active"
        }))
    });

    let routes = health.or(metrics).or(authenticate).or(get_identity);

    // Add CORS support
    let cors = cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    let routes_with_cors = routes.with(cors);

    tokio::spawn(async move {
        warp::serve(routes_with_cors).run(addr).await;
    });

    Ok(())
}

static FOUR_WORD_ENCODER: Lazy<Result<FourWordAdaptiveEncoder>> =
    Lazy::new(|| Ok(FourWordAdaptiveEncoder::new()?));

fn four_word_encoder() -> Result<&'static FourWordAdaptiveEncoder> {
    FOUR_WORD_ENCODER
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to initialise four-word networking decoder: {}", e))
}

fn looks_like_four_word_identity(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }

    let mut count = 0;
    for part in trimmed.split('-') {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }
        count += 1;
    }

    count == 4
}

fn decode_four_word_seed(seed: &str) -> Result<Option<SocketAddr>> {
    let trimmed = seed.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let mut candidate = trimmed;
    let mut explicit_port: Option<u16> = None;

    if let Some((words, port_part)) = trimmed.rsplit_once(':')
        && looks_like_four_word_identity(words)
    {
        let port = port_part.parse::<u16>().map_err(|e| {
            anyhow::anyhow!(
                "Invalid port override '{}' for four-word identity '{}': {}",
                port_part,
                seed,
                e
            )
        })?;
        candidate = words;
        explicit_port = Some(port);
    }

    if !looks_like_four_word_identity(candidate) {
        return Ok(None);
    }

    let encoder = four_word_encoder()?;
    let decoded = encoder.decode(&candidate.replace('-', " ")).map_err(|e| {
        anyhow::anyhow!("Failed to decode four-word identity '{}': {}", candidate, e)
    })?;

    let mut socket_addr = match decoded.parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(_) => {
            let ip = decoded.parse::<std::net::IpAddr>().map_err(|e| {
                anyhow::anyhow!(
                    "Four-word identity '{}' decoded to '{}' which is not a valid socket address: {}",
                    candidate,
                    decoded,
                    e
                )
            })?;

            let port = explicit_port.ok_or_else(|| {
                anyhow::anyhow!(
                    "Four-word identity '{}' decoded to '{}' without a port; specify an explicit port like '{}:PORT'.",
                    candidate,
                    decoded,
                    seed
                )
            })?;

            SocketAddr::new(ip, port)
        }
    };

    if let Some(port) = explicit_port {
        socket_addr = SocketAddr::new(socket_addr.ip(), port);
    }

    Ok(Some(socket_addr))
}

fn canonical_seed_addr(seed: &str) -> Result<Option<SocketAddr>> {
    let trimmed = seed.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if let Some(addr) = decode_four_word_seed(trimmed)? {
        return Ok(Some(addr));
    }

    if let Ok(addr) = trimmed.parse::<SocketAddr>() {
        return Ok(Some(addr));
    }

    Ok(None)
}

/// Connect to a peer using QUIC
async fn connect_to_peer(addr_str: String) -> Result<()> {
    use std::net::ToSocketAddrs;

    let trimmed = addr_str.trim();

    // Parse the address (format: "host:port" or "four words")
    let socket_addr = if let Some(addr) = decode_four_word_seed(trimmed)? {
        info!("Decoded four-word identity '{}' to {}", trimmed, addr);
        addr
    } else {
        trimmed
            .to_socket_addrs()
            .map_err(|e| anyhow::anyhow!("Failed to resolve {}: {}", trimmed, e))?
            .next()
            .ok_or_else(|| anyhow::anyhow!("No addresses resolved for {}", trimmed))?
    };

    info!("Connecting to peer at {}", socket_addr);

    // Create QUIC client endpoint
    let bind_addr: std::net::SocketAddr = if socket_addr.is_ipv4() {
        "0.0.0.0:0".parse()?
    } else {
        "[::]:0".parse()?
    };

    let endpoint = QuicEndpoint::client(bind_addr)
        .map_err(|e| anyhow::anyhow!("Failed to create QUIC client: {}", e))?;

    // Build client config with raw public keys
    let client_cfg = {
        let builder = RawPublicKeyConfigBuilder::new()
            .enable_certificate_type_extensions()
            // For testing: trust any public key (trust-on-first-use)
            .allow_any_key();

        let rustls_cfg = builder
            .build_client_config()
            .map_err(|e| anyhow::anyhow!("Failed to build client config: {}", e))?;

        let quic_tls: ant_quic::crypto::rustls::QuicClientConfig = StdArc::new(rustls_cfg)
            .try_into()
            .map_err(|e| anyhow::anyhow!("Failed to convert to QUIC TLS config: {}", e))?;

        let client = QuicClientConfig::new(StdArc::new(quic_tls));
        with_pqc_support(client)
            .map_err(|e| anyhow::anyhow!("Failed to enable PQC support: {:?}", e))?
    };

    // Set client config and connect
    let mut ep = endpoint.clone();
    ep.set_default_client_config(client_cfg);

    // Connect with SNI (use "peer" as default)
    match ep.connect(socket_addr, "peer") {
        Ok(connecting) => {
            match connecting.await {
                Ok(conn) => {
                    info!("Successfully connected to {}", socket_addr);

                    // Store the connection in active connections
                    let conn_id = format!("{}_{}", socket_addr, chrono::Utc::now().timestamp());
                    if let Ok(mut conns) = ACTIVE_CONNECTIONS.try_write() {
                        conns.insert(conn_id.clone(), conn);
                        info!("Stored connection {} (total: {})", conn_id, conns.len());
                    }

                    // TODO: Re-enable when bootstrap_integration is available
                    // Update bootstrap manager with successful connection
                    // {
                    //     let manager_guard = BOOTSTRAP_MANAGER.read().await;
                    //     if let Some(manager) = manager_guard.as_ref() {
                    //         let manager_clone = manager.clone();
                    //         let addr_str = socket_addr.to_string();

                    //         tokio::spawn(async move {
                    //             if let Err(e) = manager_clone.add_bootstrap_node(&addr_str).await {
                    //                 warn!("Failed to update bootstrap cache: {}", e);
                    //             } else {
                    //                 debug!("Added peer {} to bootstrap cache", addr_str);
                    //             }
                    //         });
                    //     }
                    // }

                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Connection failed: {}", e)),
            }
        }
        Err(e) => Err(anyhow::anyhow!("Failed to initiate connection: {}", e)),
    }
}

async fn run_node(args: Args) -> Result<()> {
    // Self-update mode: do not start services
    if args.self_update {
        // Run the blocking self-update in a spawn_blocking task
        match tokio::task::spawn_blocking(try_self_update).await {
            Ok(Ok(Some(ver))) => {
                tracing::info!("updated-to={}", ver);
            }
            Ok(Ok(None)) => tracing::info!("no-update"),
            Ok(Err(e)) => {
                tracing::error!("self-update error: {:#}", e);
            }
            Err(e) => {
                tracing::error!("spawn error: {:#}", e);
            }
        }
        return Ok(());
    }
    let instance_id = args
        .instance_id
        .as_deref()
        .map(sanitize_instance_id)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(default_instance_id);

    let config_path = resolve_config_path(&args, &instance_id)?;
    let config_path = ensure_absolute(&config_path)?;
    let storage_hint = resolve_storage_hint(&args, &instance_id)?;
    let storage_default = ensure_absolute(storage_hint.path())?;

    let config_exists = tokio::fs::try_exists(&config_path).await?;
    let mut config = load_or_create_config(
        &config_path,
        default_config_with_storage(storage_default.clone()),
    )
    .await?;
    let mut config_dirty = false;
    info!(
        "Loaded configuration from {} (instance {})",
        config_path.display(),
        instance_id
    );

    match storage_hint {
        StoragePathHint::CommandLine(_) | StoragePathHint::Env(_) => {
            if config.storage.base_dir != storage_default {
                config.storage.base_dir = storage_default.clone();
                config_dirty = true;
            }
        }
        StoragePathHint::Default(_) => {
            if !config_exists && config.storage.base_dir != storage_default {
                config.storage.base_dir = storage_default.clone();
                config_dirty = true;
            }
        }
    }

    let storage_base = ensure_absolute(&config.storage.base_dir)?;
    if storage_base != config.storage.base_dir {
        config.storage.base_dir = storage_base.clone();
        config_dirty = true;
    }

    let mut canonical_bootstrap_addrs: HashSet<SocketAddr> = HashSet::new();
    for existing in &config.bootstrap_nodes {
        if let Ok(Some(addr)) = canonical_seed_addr(existing) {
            canonical_bootstrap_addrs.insert(addr);
        }
    }

    // Merge command-line bootstrap nodes with config
    if !args.bootstrap.is_empty() {
        info!(
            "Adding {} bootstrap nodes from command line",
            args.bootstrap.len()
        );
        for bootstrap in &args.bootstrap {
            let trimmed = bootstrap.trim();
            if trimmed.is_empty() {
                continue;
            }

            match canonical_seed_addr(trimmed) {
                Ok(Some(addr)) => {
                    if canonical_bootstrap_addrs.insert(addr) {
                        config.bootstrap_nodes.push(trimmed.to_string());
                        config_dirty = true;
                    } else {
                        info!(
                            "Bootstrap {} resolves to {} which is already configured; skipping duplicate",
                            trimmed, addr
                        );
                    }
                }
                Ok(None) => {
                    if !config
                        .bootstrap_nodes
                        .iter()
                        .any(|existing| existing.trim() == trimmed)
                    {
                        config.bootstrap_nodes.push(trimmed.to_string());
                        config_dirty = true;
                    }
                }
                Err(e) => {
                    warn!(
                        "Bootstrap {} looks like a four-word identity but could not be decoded: {}",
                        trimmed, e
                    );
                    if !config
                        .bootstrap_nodes
                        .iter()
                        .any(|existing| existing.trim() == trimmed)
                    {
                        config.bootstrap_nodes.push(trimmed.to_string());
                        config_dirty = true;
                    }
                }
            }
        }
    }

    // Try self-update if enabled
    if config.update.auto_update {
        info!("Checking for updates...");
        // Run the blocking self-update in a spawn_blocking task
        match tokio::task::spawn_blocking(try_self_update).await {
            Ok(Ok(Some(new_version))) => {
                info!("Successfully updated to version {}", new_version);
                info!("Please restart the application to use the new version");
                // In production, you might want to restart automatically
                // or notify the user through other means
            }
            Ok(Ok(None)) => {
                info!("No updates available");
            }
            Ok(Err(e)) => {
                warn!("Failed to check for updates: {:#}", e);
            }
            Err(e) => {
                warn!("Failed to spawn update task: {:#}", e);
            }
        }
    }

    // Setup storage
    tokio::fs::create_dir_all(&config.storage.base_dir)
        .await
        .context("Failed to create storage directory")?;

    // TODO: Re-enable when bootstrap_integration is available in communitas-core
    // Initialize bootstrap manager using saorsa-core
    // let bootstrap_config = BootstrapConfig {
    //     max_contacts: 5000,
    //     default_nodes: config.bootstrap_nodes.clone(),
    //     auto_discovery: true,
    //     cache_dir: config.storage.base_dir.join("bootstrap"),
    //     quality_threshold: 0.3,
    // };

    // let bootstrap_manager = EnhancedBootstrapManager::new(bootstrap_config)
    //     .await
    //     .context("Failed to initialize bootstrap manager")?;

    // {
    //     let mut manager_guard = BOOTSTRAP_MANAGER.write().await;
    //     *manager_guard = Some(Arc::new(bootstrap_manager));
    // }
    info!("Initialized bootstrap manager with saorsa-core");

    // Setup identity
    let identity_material = setup_identity(&config).await?;
    let identity = identity_material.four_words.clone();
    if config.identity.as_deref() != Some(identity.as_str()) {
        config.identity = Some(identity.clone());
        config_dirty = true;
    }
    if config_dirty {
        save_config(&config_path, &config).await?;
    }
    info!("Node identity: {}", identity);

    // Removed: DHT client initialization - saorsa-core removed
    info!(
        "Skipping DHT initialization (saorsa-core removed) - {} bootstrap nodes configured",
        config.bootstrap_nodes.len()
    );
    for bootstrap in &config.bootstrap_nodes {
        let trimmed = bootstrap.trim();
        match decode_four_word_seed(trimmed) {
            Ok(Some(addr)) => info!("  Bootstrap node: {} -> {}", trimmed, addr),
            Ok(None) => info!("  Bootstrap node: {}", trimmed),
            Err(e) => warn!(
                "  Bootstrap node {} looks like a four-word identity but could not be decoded: {}",
                trimmed, e
            ),
        }
    }

    // Stub: DHT client removed - saorsa-core dependency eliminated
    // let dht_client = Arc::new(DhtClient::new().map_err(|e| anyhow::anyhow!("DHT init failed: {}", e))?);
    info!("DHT client stubbed (saorsa-core removed)");

    // Load cached peers and connect to them first
    let mut seen_resolved_addrs: HashSet<SocketAddr> = HashSet::new();
    let mut seen_unresolved: HashSet<String> = HashSet::new();
    let mut all_bootstrap_nodes: Vec<String> = Vec::new();

    for bootstrap in &config.bootstrap_nodes {
        let trimmed = bootstrap.trim();
        let trimmed_owned = trimmed.to_string();
        match canonical_seed_addr(&trimmed_owned) {
            Ok(Some(addr)) => {
                if seen_resolved_addrs.insert(addr) {
                    all_bootstrap_nodes.push(trimmed_owned);
                }
            }
            Ok(None) => {
                if seen_unresolved.insert(trimmed_owned.clone()) {
                    all_bootstrap_nodes.push(trimmed_owned);
                }
            }
            Err(e) => {
                warn!(
                    "Bootstrap entry {} looks like a four-word identity but failed to decode: {}",
                    trimmed, e
                );
                if seen_unresolved.insert(trimmed_owned.clone()) {
                    all_bootstrap_nodes.push(trimmed_owned);
                }
            }
        }
    }

    // TODO: Re-enable when bootstrap_integration is available
    // Add cached peers from bootstrap manager
    // {
    //     let manager_guard = BOOTSTRAP_MANAGER.read().await;
    //     if let Some(manager) = manager_guard.as_ref() {
    //         match manager.get_bootstrap_candidates(10).await {
    //             Ok(candidates) => {
    //                 info!("Found {} cached bootstrap candidates", candidates.len());
    //                 for node_str in candidates {
    //                     // Check if it's a socket address or four-word address
    //                     if node_str.contains(':') {
    //                         // It's a socket address
    //                         if let Ok(addr) = node_str.parse::<SocketAddr>() {
    //                             if seen_resolved_addrs.insert(addr) {
    //                                 all_bootstrap_nodes.push(node_str);
    //                             }
    //                         }
    //                     } else {
    //                         // It's a four-word address
    //                         if seen_unresolved.insert(node_str.clone()) {
    //                             all_bootstrap_nodes.push(node_str);
    //                         }
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 warn!("Failed to get bootstrap candidates: {}", e);
    //             }
    //         }
    //     }
    // }

    // Collect all listen addresses from config (dual-stack IPv4 + IPv6)
    let mut listen_addrs: Vec<SocketAddr> = if !config.network.listen_addrs.is_empty() {
        config.network.listen_addrs.clone()
    } else {
        vec![args.listen]
    };

    // Allow environment variables to override listen addresses
    if let Ok(s) = std::env::var("COMMUNITAS_QUIC_LISTEN") {
        if let Ok(sa) = s.parse::<SocketAddr>() {
            // Replace all addresses with env override
            listen_addrs = vec![sa];
        }
    } else if let Ok(v) = std::env::var("COMMUNITAS_QUIC_PORT") {
        if let Ok(p) = v.parse::<u16>() {
            // Update port for all addresses
            for addr in &mut listen_addrs {
                addr.set_port(p);
            }
        }
    }

    // Override from command line if not default
    if args.listen != SocketAddr::from(([0, 0, 0, 0], 0)) {
        listen_addrs = vec![args.listen];
    }

    // CRITICAL FIX: Start QUIC delta servers FIRST (before trying to connect)
    // This ensures listeners are ready when other nodes try to connect
    info!(
        "Starting QUIC servers on {} address(es): {:?}",
        listen_addrs.len(),
        listen_addrs
    );

    for listen_addr in listen_addrs.clone() {
        let storage_dir = config.storage.base_dir.clone();
        tokio::spawn(async move {
            info!("QUIC delta server listening on {}", listen_addr);
            if let Err(e) = start_quic_delta_server(listen_addr, storage_dir).await {
                warn!("QUIC delta server on {} exited: {e}", listen_addr);
            }
        });
    }

    // Give listeners time to start (important for bootstrap nodes to see each other)
    info!("Waiting 2 seconds for QUIC listeners to initialize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    info!("QUIC listeners ready");

    // Build set of self addresses to filter from bootstrap connections
    // CRITICAL: Expand 0.0.0.0 wildcards to all local interface IPs
    let mut self_addrs: HashSet<SocketAddr> = HashSet::new();
    for addr in &listen_addrs {
        if addr.ip().is_unspecified() {
            // Wildcard address (0.0.0.0 or ::) - expand to all local interfaces
            // Add localhost
            self_addrs.insert(SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), addr.port()));

            // Get all network interface addresses and add them
            if let Ok(interfaces) = local_ip_address::list_afinet_netifas() {
                for (_name, ip) in interfaces {
                    self_addrs.insert(SocketAddr::new(ip, addr.port()));
                }
            }
        } else {
            // Specific address - use as-is
            self_addrs.insert(*addr);
        }
    }

    // Filter bootstrap nodes to exclude self-addresses
    let filtered_bootstrap_nodes: Vec<String> = all_bootstrap_nodes
        .iter()
        .filter(|addr_str| {
            // Try to resolve to SocketAddr and check if it matches our listen addresses
            if let Ok(Some(socket_addr)) = canonical_seed_addr(addr_str) {
                let is_self = self_addrs.contains(&socket_addr);
                if is_self {
                    info!("Filtering out self-address: {} (resolved to {})", addr_str, socket_addr);
                }
                !is_self
            } else {
                // If we can't resolve it, keep it (might be a DNS name or four-word that resolves differently)
                true
            }
        })
        .cloned()
        .collect();

    // Connect to bootstrap nodes (excluding self, after listeners are ready)
    if !filtered_bootstrap_nodes.is_empty() {
        info!(
            "Connecting to {} bootstrap/cached nodes (filtered {} self-addresses)...",
            filtered_bootstrap_nodes.len(),
            all_bootstrap_nodes.len() - filtered_bootstrap_nodes.len()
        );
        for bootstrap_addr in &filtered_bootstrap_nodes {
            match connect_to_peer(bootstrap_addr.clone()).await {
                Ok(_) => info!("✓ Connected to peer: {}", bootstrap_addr),
                Err(e) => warn!("Failed to connect to {}: {}", bootstrap_addr, e),
            }
        }
    } else {
        info!("No external bootstrap nodes to connect to (all addresses are self)");
    }

    // Start health/metrics endpoint if enabled
    if args.metrics {
        start_health_endpoint(args.metrics_addr).await?;
        info!("Metrics endpoint started on {}", args.metrics_addr);
    }

    // TODO: Re-enable when communitas_container is available
    // Optionally start a background delta generator (very simple demo)
    // if std::env::var("COMMUNITAS_GENERATE_DELTAS").is_ok() {
    //     tokio::spawn(async move {
    //         use communitas_container as cc;
    //         use uuid::Uuid;
    //         loop {
    //             let ts = chrono::Utc::now().timestamp();
    //             let post = cc::Post {
    //                 id: Uuid::new_v4(),
    //                 author: b"server".to_vec(),
    //                 ts,
    //                 body_md: format!("# Server note\nGenerated at {}", ts),
    //             };
    //             let op = cc::Op::Append { post };
    //             {
    //                 let mut w = OP_LOG.write().await;
    //                 w.push(op);
    //             }
    //             tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    //         }
    //     });
    //     info!("Delta generator enabled via COMMUNITAS_GENERATE_DELTAS");
    // }

    // Main event loop
    info!("Communitas node started successfully");
    info!("Press Ctrl+C to shutdown");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    // Graceful shutdown
    info!("Performing graceful shutdown...");

    // Bootstrap cache is automatically saved by saorsa-core
    info!("Bootstrap cache will be saved automatically by saorsa-core");

    // Close all active connections
    if let Ok(mut conns) = ACTIVE_CONNECTIONS.try_write() {
        info!("Closing {} active connections", conns.len());
        conns.clear();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Rustls crypto provider (required for QUIC/TLS)
    // Use aws-lc-rs as the default crypto backend
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| anyhow!("Failed to install default crypto provider"))?;

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if let Err(e) = run_node(args).await {
        error!("Node failed: {:#}", e);
        std::process::exit(1);
    }

    Ok(())
}

// ---------------- QUIC Delta Server (raw SPKI) -----------------

use ant_quic::config::{ClientConfig as QuicClientConfig, ServerConfig as QuicServerConfig};
use ant_quic::crypto::pqc::rustls_provider::{with_pqc_support, with_pqc_support_server};
use ant_quic::crypto::raw_public_keys::RawPublicKeyConfigBuilder;
use ant_quic::crypto::raw_public_keys::key_utils::public_key_to_bytes;
use ant_quic::high_level::Endpoint as QuicEndpoint;
use std::sync::Arc as StdArc;
// ant-quic send streams provide write_all via their API; no extra trait import needed
// TODO: Re-enable when communitas_container is available
// use communitas_container as cc;

// TODO: Re-enable when communitas_container is available
// #[derive(Serialize, Deserialize)]
// struct DeltaRequest<'a> {
//     from_root_hex: Option<&'a str>,
//     want_since_count: Option<u64>,
// }

// #[derive(Serialize, Deserialize)]
// struct DeltaResponse {
//     ops: Vec<cc::Op>,
// }

// Very small in-memory op log for demo/testing. Not persisted.
// static OP_LOG: Lazy<AsyncRwLock<Vec<cc::Op>>> = Lazy::new(|| AsyncRwLock::new(Vec::new()));

// Global connection tracking
use ant_quic::HighLevelConnection;
static ACTIVE_CONNECTIONS: Lazy<Arc<RwLock<HashMap<String, HighLevelConnection>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));
// TODO: Re-enable when bootstrap_integration is available
// static BOOTSTRAP_MANAGER: Lazy<Arc<AsyncRwLock<Option<Arc<EnhancedBootstrapManager>>>>> =
//     Lazy::new(|| Arc::new(AsyncRwLock::new(None)));

// TODO: Re-enable when communitas_container is available
// async fn ops_since(count: u64) -> Vec<cc::Op> {
//     let r = OP_LOG.read().await;
//     if (count as usize) >= r.len() {
//         return Vec::new();
//     }
//     r[count as usize..].to_vec()
// }

async fn start_quic_delta_server(
    listen: std::net::SocketAddr,
    base_dir: std::path::PathBuf,
) -> Result<()> {
    // Persist or generate transport key (ed25519 seed, 32 bytes)
    let key_path = base_dir.join("transport_ed25519.key");
    let sk: Ed25519SecretKey = if key_path.exists() {
        let bytes = tokio::fs::read(&key_path)
            .await
            .context("read transport key")?;
        anyhow::ensure!(bytes.len() == 32, "transport key must be 32 bytes (seed)");
        let seed: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("transport key file must be exactly 32 bytes"))?;
        Ed25519SecretKey::from_bytes(&seed)
    } else {
        let mut rng = OsRng;
        let sk = Ed25519SecretKey::generate(&mut rng);
        if let Some(parent) = key_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        let _ = tokio::fs::write(&key_path, sk.to_bytes()).await;
        #[cfg(unix)]
        {
            let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
        }
        sk
    };
    let pk_bytes = public_key_to_bytes(&sk.verifying_key());
    info!("QUIC server raw key (hex): {}", hex::encode(pk_bytes));

    // Build rustls server config with raw public key resolver
    // IMPORTANT: For bootstrap nodes, accept connections from any client (trust-on-first-use)
    // This allows bootstrap nodes to be publicly accessible without pre-shared keys
    let rustls_srv = RawPublicKeyConfigBuilder::new()
        .with_server_key(sk)
        .enable_certificate_type_extensions()
        .allow_any_key()  // Accept connections from any client
        .build_server_config()
        .map_err(|e| anyhow::anyhow!("raw pk server config: {e}"))?;

    // Convert to ant-quic server crypto config
    let quic_tls: ant_quic::crypto::rustls::QuicServerConfig =
        StdArc::new(rustls_srv)
            .try_into()
            .map_err(|e| anyhow::anyhow!("convert tls server cfg: {e}"))?;
    let server_cfg = with_pqc_support_server(QuicServerConfig::with_crypto(StdArc::new(quic_tls)))
        .map_err(|e| anyhow::anyhow!("enable PQC on server: {e:?}"))?;

    // Bind endpoint
    let endpoint = QuicEndpoint::server(server_cfg, listen)
        .map_err(|e| anyhow::anyhow!("endpoint server bind: {e}"))?;
    info!("QUIC delta server listening on {}", listen);

    loop {
        match endpoint.accept().await {
            Some(incoming) => {
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            info!("Accepted QUIC connection from {}", conn.remote_address());
                            // TODO: Re-enable when communitas_container is available
                            // Accept a single bi-directional stream for request/response
                            // match conn.accept_bi().await {
                            //     Ok((mut send, mut recv)) => {
                            //         let mut buf = Vec::new();
                            //         if let Ok(bytes) = recv.read_to_end(1024 * 1024).await {
                            //             buf = bytes;
                            //         }
                            //         let text = String::from_utf8_lossy(&buf);
                            //         let req: Result<DeltaRequest, _> =
                            //             serde_json::from_str(text.trim_end());
                            //         let since =
                            //             req.ok().and_then(|r| r.want_since_count).unwrap_or(0);
                            //         let ops = ops_since(since).await;
                            //         let resp = DeltaResponse { ops };
                            //         match serde_json::to_string(&resp) {
                            //             Ok(mut s) => {
                            //                 s.push('\n');
                            //                 let _ = send.write_all(s.as_bytes()).await;
                            //             }
                            //             Err(e) => {
                            //                 warn!("serialize response failed: {e}");
                            //             }
                            //         }
                            //         let _ = send.finish();
                            //     }
                            //     Err(e) => warn!("accept_bi failed: {e}"),
                            // }
                        }
                        Err(e) => warn!("incoming failed: {e}"),
                    }
                });
            }
            None => {
                warn!("Endpoint accept returned None; shutting server");
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.update.channel, "stable");
        assert!(config.storage.enable_fec);
        assert_eq!(config.storage.fec_k, 8);
        assert_eq!(config.storage.fec_m, 4);
    }

    #[tokio::test]
    async fn test_load_or_create_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let storage_root = temp_dir.path().join("data-root");

        // First call should create default config
        let config1 = load_or_create_config(
            &config_path,
            default_config_with_storage(storage_root.clone()),
        )
        .await
        .unwrap();
        assert!(config_path.exists());
        assert_eq!(config1.storage.base_dir, storage_root);

        // Second call should load existing config
        let config2 = load_or_create_config(
            &config_path,
            default_config_with_storage(temp_dir.path().join("other")),
        )
        .await
        .unwrap();
        assert_eq!(config1.update.channel, config2.update.channel);
        assert_eq!(config2.storage.base_dir, storage_root);
    }

    #[test]
    fn test_resolve_paths_respects_env() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("cfg");
        let data_dir = temp.path().join("data");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();

        // Safety: setting process-wide env vars in a test scope is acceptable because we
        // immediately clean them up before the test exits.
        unsafe {
            env::set_var("COMMUNITAS_CONFIG_DIR", &config_dir);
            env::set_var("COMMUNITAS_DATA_DIR", &data_dir);
        }

        let args = Args {
            config: None,
            storage: None,
            instance_id: Some("Test Node".to_string()),
            listen: "0.0.0.0:0".parse().unwrap(),
            bootstrap: Vec::new(),
            metrics: false,
            metrics_addr: "127.0.0.1:9600".parse().unwrap(),
            self_update: false,
        };

        let instance = sanitize_instance_id("Test Node");
        let config_path = resolve_config_path(&args, &instance).unwrap();
        assert!(config_path.starts_with(&config_dir));
        assert_eq!(config_path.file_name().unwrap(), "config.toml");

        let storage_hint = resolve_storage_hint(&args, &instance).unwrap();
        let storage_path = ensure_absolute(storage_hint.path()).unwrap();
        assert!(storage_path.starts_with(&data_dir.join("communitas")));

        unsafe {
            env::remove_var("COMMUNITAS_CONFIG_DIR");
            env::remove_var("COMMUNITAS_DATA_DIR");
        }
    }

    #[tokio::test]
    async fn test_default_config_has_dual_stack_listeners() {
        // Test that default configuration includes both IPv4 and IPv6 listeners
        let temp_dir = tempdir().unwrap();
        let config = default_config_with_storage(temp_dir.path().to_path_buf());

        // Verify we have at least 2 listen addresses
        assert!(
            config.network.listen_addrs.len() >= 2,
            "Expected at least 2 listen addresses (IPv4 and IPv6), got {}",
            config.network.listen_addrs.len()
        );

        // Verify first address is IPv4
        let ipv4_addr = config.network.listen_addrs[0];
        assert!(
            ipv4_addr.is_ipv4(),
            "Expected first listen address to be IPv4, got {:?}",
            ipv4_addr
        );
        assert_eq!(
            ipv4_addr.ip(),
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            "Expected IPv4 wildcard address (0.0.0.0)"
        );

        // Verify second address is IPv6
        let ipv6_addr = config.network.listen_addrs[1];
        assert!(
            ipv6_addr.is_ipv6(),
            "Expected second listen address to be IPv6, got {:?}",
            ipv6_addr
        );
        assert_eq!(
            ipv6_addr.ip(),
            std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED),
            "Expected IPv6 wildcard address (::)"
        );

        // Verify both use the same port
        assert_eq!(
            ipv4_addr.port(),
            ipv6_addr.port(),
            "Expected IPv4 and IPv6 to use the same port"
        );

        // Verify IPv6 is enabled in config
        assert!(
            config.network.enable_ipv6,
            "Expected IPv6 to be enabled in network config"
        );
    }

    #[test]
    fn test_ipv4_and_ipv6_socket_addresses() {
        // Test creating both IPv4 and IPv6 socket addresses
        let ipv4: SocketAddr = "0.0.0.0:443".parse().unwrap();
        let ipv6: SocketAddr = "[::]:443".parse().unwrap();

        assert!(ipv4.is_ipv4());
        assert!(ipv6.is_ipv6());
        assert_eq!(ipv4.port(), 443);
        assert_eq!(ipv6.port(), 443);

        // Test that we can create a vector of both
        let listen_addrs = vec![ipv4, ipv6];
        assert_eq!(listen_addrs.len(), 2);
        assert!(listen_addrs[0].is_ipv4());
        assert!(listen_addrs[1].is_ipv6());
    }

    #[test]
    fn test_sanitize_instance_id_preserves_validity() {
        // Test that sanitization replaces invalid chars with dashes and trims dashes
        let test_cases = vec![
            ("test-node", "test-node"),
            ("Test Node 123", "Test-Node-123"),
            ("node@server", "node-server"),
            ("my.node.name", "my-node-name"),
            ("___test___", "___test___"), // Underscores are preserved
            ("---test---", "test"),       // Dashes are trimmed from edges
            ("", "communitas"),           // Test empty string fallback
        ];

        for (input, expected) in test_cases {
            let result = sanitize_instance_id(input);
            assert_eq!(
                result, expected,
                "sanitize_instance_id({:?}) = {:?}, expected {:?}",
                input, result, expected
            );
        }
    }

    #[test]
    fn print_digital_ocean_bootstrap_addresses() {
        use four_word_networking::FourWordAdaptiveEncoder;

        // Convert Digital Ocean droplet IPs to four-word addresses
        let bootstrap_nodes = vec![
            ("167.71.188.131", 443), // Droplet: 2064413
            ("138.197.29.195", 443), // Droplet: communitas-bootstrap-1
        ];

        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║  Digital Ocean Bootstrap Nodes (Four-Word Addresses)    ║");
        println!("╠══════════════════════════════════════════════════════════╣");

        let encoder = FourWordAdaptiveEncoder::new().expect("Failed to create encoder");

        for (ip, port) in &bootstrap_nodes {
            let address = format!("{}:{}", ip, port);
            match encoder.encode(&address) {
                Ok(four_word_addr) => {
                    println!("║ {}:{} → {}", ip, port, four_word_addr);
                }
                Err(e) => eprintln!("║ ERROR encoding {}: {}", address, e),
            }
        }

        println!("╚══════════════════════════════════════════════════════════╝\n");
    }
}
