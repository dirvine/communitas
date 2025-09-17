// Communitas Headless Node
// This binary runs a headless Communitas node using saorsa-core APIs

mod peer_cache;

use anyhow::{Context, Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use clap::Parser;
use ed25519_dalek::SigningKey as Ed25519SecretKey;
use once_cell::sync::Lazy;
use rand::RngCore;
use rand::rngs::OsRng;
use saorsa_core::address::NetworkAddress;
use saorsa_core::identity::FourWordAddress;
use saorsa_core::quantum_crypto::{
    MlDsaPublicKey, MlDsaSecretKey, MlKemPublicKey, MlKemSecretKey, generate_ml_dsa_keypair,
    generate_ml_kem_keypair,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::signal;
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Import peer cache types
use crate::peer_cache::{PeerCache, PeerInfo};

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
    /// Configuration file path
    #[arg(short, long, default_value = "/etc/communitas/config.toml")]
    config: PathBuf,

    /// Storage directory
    #[arg(short, long, default_value = "/var/lib/communitas")]
    storage: PathBuf,

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
        Self {
            identity: None,
            bootstrap_nodes: vec![
                "ocean-forest-moon-star:443".to_string(),
                "river-mountain-sun-cloud:443".to_string(),
            ],
            storage: StorageConfig {
                base_dir: PathBuf::from("/var/lib/communitas"),
                cache_size_mb: 1024,
                enable_fec: true,
                fec_k: 8,
                fec_m: 4,
            },
            network: NetworkConfig {
                listen_addrs: vec![std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                    443,
                )],
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
}

async fn load_or_create_config(path: &PathBuf) -> Result<Config> {
    if path.exists() {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config")
    } else {
        // Create default config
        let config = Config::default();
        let parent = path.parent().context("Invalid config path")?;
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create config directory")?;

        let content = toml::to_string_pretty(&config).context("Failed to serialize config")?;
        tokio::fs::write(path, content)
            .await
            .context("Failed to write config")?;

        Ok(config)
    }
}

async fn save_config(path: &PathBuf, config: &Config) -> Result<()> {
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

        MlDsaPublicKey::from_bytes(&ml_dsa_public)
            .map_err(|e| anyhow!("Invalid ML-DSA public key in identity store: {}", e))?;
        MlDsaSecretKey::from_bytes(&ml_dsa_secret)
            .map_err(|e| anyhow!("Invalid ML-DSA secret key in identity store: {}", e))?;
        MlKemPublicKey::from_bytes(&ml_kem_public)
            .map_err(|e| anyhow!("Invalid ML-KEM public key in identity store: {}", e))?;
        MlKemSecretKey::from_bytes(&ml_kem_secret)
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
    if !saorsa_core::fwid::fw_check(words.clone()) {
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
        let candidate = NetworkAddress::from_ipv4(ipv4, port);
        if let Some(words) = candidate.four_words() {
            if let Ok(canonical) = canonicalize_four_words(words) {
                return Ok(canonical);
            }
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

            let (ml_dsa_public, ml_dsa_secret) = generate_ml_dsa_keypair()
                .map_err(|e| anyhow!("Failed to generate ML-DSA keypair: {}", e))?;
            let (ml_kem_public, ml_kem_secret) = generate_ml_kem_keypair()
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
    _dht_client: Arc<saorsa_core::messaging::DhtClient>,
) -> Result<()> {
    use warp::Filter;

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

    let routes = health.or(metrics);

    tokio::spawn(async move {
        warp::serve(routes).run(addr).await;
    });

    Ok(())
}

/// Connect to a peer using QUIC
async fn connect_to_peer(addr_str: String) -> Result<()> {
    use std::net::ToSocketAddrs;

    // Parse the address (format: "host:port" or "four-words:port")
    let socket_addr = addr_str
        .to_socket_addrs()
        .map_err(|e| anyhow::anyhow!("Failed to resolve {}: {}", addr_str, e))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No addresses resolved for {}", addr_str))?;

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

                    // Update peer cache with successful connection
                    {
                        let cache_guard = PEER_CACHE.read().await;
                        if let Some(cache) = cache_guard.as_ref() {
                            let peer_id = socket_addr.to_string();
                            let peer_info = PeerInfo {
                                identity: None, // Will be updated when we exchange identities
                                address: socket_addr,
                                public_key: None, // Will be updated from handshake
                                last_connected: chrono::Utc::now().timestamp(),
                                connection_count: 1,
                                quality_score: 80, // Start with good score
                            };

                            let cache_clone = cache.clone();
                            tokio::spawn(async move {
                                if let Err(e) = cache_clone.add_peer(peer_id, peer_info).await {
                                    warn!("Failed to update peer cache: {}", e);
                                }
                            });
                        }
                    }

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
        match try_self_update() {
            Ok(Some(ver)) => {
                println!("updated-to={}", ver);
            }
            Ok(None) => println!("no-update"),
            Err(e) => {
                eprintln!("self-update error: {:#}", e);
            }
        }
        return Ok(());
    }
    // Load or create config
    let mut config = load_or_create_config(&args.config).await?;
    let mut config_dirty = false;
    info!("Loaded configuration from {:?}", args.config);

    // Merge command-line bootstrap nodes with config
    if !args.bootstrap.is_empty() {
        info!(
            "Adding {} bootstrap nodes from command line",
            args.bootstrap.len()
        );
        for bootstrap in &args.bootstrap {
            if !config.bootstrap_nodes.contains(bootstrap) {
                config.bootstrap_nodes.push(bootstrap.clone());
                config_dirty = true;
            }
        }
    }

    // Try self-update if enabled
    if config.update.auto_update {
        info!("Checking for updates...");
        match try_self_update() {
            Ok(Some(new_version)) => {
                info!("Successfully updated to version {}", new_version);
                info!("Please restart the application to use the new version");
                // In production, you might want to restart automatically
                // or notify the user through other means
            }
            Ok(None) => {
                info!("No updates available");
            }
            Err(e) => {
                warn!("Failed to check for updates: {:#}", e);
            }
        }
    }

    // Setup storage
    tokio::fs::create_dir_all(&config.storage.base_dir)
        .await
        .context("Failed to create storage directory")?;

    // Initialize peer cache
    let peer_cache = PeerCache::new(config.storage.base_dir.clone(), 100)?;
    {
        let mut cache_guard = PEER_CACHE.write().await;
        *cache_guard = Some(Arc::new(peer_cache));
    }
    info!("Initialized peer cache");

    // Setup identity
    let identity_material = setup_identity(&config).await?;
    let identity = identity_material.four_words.clone();
    if config.identity.as_deref() != Some(identity.as_str()) {
        config.identity = Some(identity.clone());
        config_dirty = true;
    }
    if config_dirty {
        save_config(&args.config, &config).await?;
    }
    info!("Node identity: {}", identity);

    // Initialize DHT using saorsa_core
    info!(
        "Initializing DHT with {} bootstrap nodes",
        config.bootstrap_nodes.len()
    );
    for bootstrap in &config.bootstrap_nodes {
        info!("  Bootstrap node: {}", bootstrap);
    }

    // Create DHT client
    use saorsa_core::messaging::DhtClient;
    let dht_client =
        Arc::new(DhtClient::new().map_err(|e| anyhow::anyhow!("DHT init failed: {}", e))?);
    info!("DHT client initialized");

    // Load cached peers and connect to them first
    let mut all_bootstrap_nodes = config.bootstrap_nodes.clone();

    {
        let cache_guard = PEER_CACHE.read().await;
        if let Some(cache) = cache_guard.as_ref() {
            let cached_peers = cache.get_bootstrap_candidates(5).await;
            info!("Found {} cached peers to try", cached_peers.len());

            for peer_info in cached_peers {
                let addr_str = peer_info.address.to_string();
                if !all_bootstrap_nodes.contains(&addr_str) {
                    all_bootstrap_nodes.push(addr_str);
                }
            }
        }
    }

    // Connect to bootstrap nodes (including cached peers)
    if !all_bootstrap_nodes.is_empty() {
        info!(
            "Connecting to {} bootstrap/cached nodes...",
            all_bootstrap_nodes.len()
        );
        for bootstrap_addr in &all_bootstrap_nodes {
            match connect_to_peer(bootstrap_addr.clone()).await {
                Ok(_) => info!("Connected to node: {}", bootstrap_addr),
                Err(e) => warn!("Failed to connect to {}: {}", bootstrap_addr, e),
            }
        }
    }

    // Start health/metrics endpoint if enabled
    if args.metrics {
        start_health_endpoint(args.metrics_addr, dht_client.clone()).await?;
        info!("Metrics endpoint started on {}", args.metrics_addr);
    }

    // Optionally start a background delta generator (very simple demo)
    if std::env::var("COMMUNITAS_GENERATE_DELTAS").is_ok() {
        tokio::spawn(async move {
            use communitas_container as cc;
            use uuid::Uuid;
            loop {
                let ts = chrono::Utc::now().timestamp();
                let post = cc::Post {
                    id: Uuid::new_v4(),
                    author: b"server".to_vec(),
                    ts,
                    body_md: format!("# Server note\nGenerated at {}", ts),
                };
                let op = cc::Op::Append { post };
                {
                    let mut w = OP_LOG.write().await;
                    w.push(op);
                }
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        });
        info!("Delta generator enabled via COMMUNITAS_GENERATE_DELTAS");
    }

    // Resolve listen address from args or env overrides
    let mut listen_addr = args.listen;
    if let Ok(s) = std::env::var("COMMUNITAS_QUIC_LISTEN") {
        if let Ok(sa) = s.parse::<SocketAddr>() {
            listen_addr = sa;
        }
    } else if let Ok(v) = std::env::var("COMMUNITAS_QUIC_PORT")
        && let Ok(p) = v.parse::<u16>()
    {
        listen_addr.set_port(p);
        listen_addr.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
    }

    // Start QUIC delta server (raw public key, RFC 7250 style)
    let storage_dir = config.storage.base_dir.clone();
    tokio::spawn(async move {
        if let Err(e) = start_quic_delta_server(listen_addr, storage_dir).await {
            warn!("QUIC delta server exited: {e}");
        }
    });

    // Main event loop
    info!("Communitas node started successfully");
    info!("Press Ctrl+C to shutdown");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    // Graceful shutdown
    info!("Performing graceful shutdown...");

    // Save peer cache before shutdown
    {
        let cache_guard = PEER_CACHE.read().await;
        if let Some(cache) = cache_guard.as_ref() {
            if let Err(e) = cache.save().await {
                warn!("Failed to save peer cache: {}", e);
            } else {
                info!("Peer cache saved successfully");
            }
        }
    }

    // Close all active connections
    if let Ok(mut conns) = ACTIVE_CONNECTIONS.try_write() {
        info!("Closing {} active connections", conns.len());
        conns.clear();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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
use communitas_container as cc;

#[derive(Serialize, Deserialize)]
struct DeltaRequest<'a> {
    from_root_hex: Option<&'a str>,
    want_since_count: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct DeltaResponse {
    ops: Vec<cc::Op>,
}

// Very small in-memory op log for demo/testing. Not persisted.
static OP_LOG: Lazy<AsyncRwLock<Vec<cc::Op>>> = Lazy::new(|| AsyncRwLock::new(Vec::new()));

// Global connection tracking
use ant_quic::HighLevelConnection;
static ACTIVE_CONNECTIONS: Lazy<Arc<RwLock<HashMap<String, HighLevelConnection>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));
static PEER_CACHE: Lazy<Arc<AsyncRwLock<Option<Arc<PeerCache>>>>> =
    Lazy::new(|| Arc::new(AsyncRwLock::new(None)));

async fn ops_since(count: u64) -> Vec<cc::Op> {
    let r = OP_LOG.read().await;
    if (count as usize) >= r.len() {
        return Vec::new();
    }
    r[count as usize..].to_vec()
}

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
    let rustls_srv = RawPublicKeyConfigBuilder::new()
        .with_server_key(sk)
        .enable_certificate_type_extensions()
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
                            // Accept a single bi-directional stream for request/response
                            match conn.accept_bi().await {
                                Ok((mut send, mut recv)) => {
                                    let mut buf = Vec::new();
                                    if let Ok(bytes) = recv.read_to_end(1024 * 1024).await {
                                        buf = bytes;
                                    }
                                    let text = String::from_utf8_lossy(&buf);
                                    let req: Result<DeltaRequest, _> =
                                        serde_json::from_str(text.trim_end());
                                    let since =
                                        req.ok().and_then(|r| r.want_since_count).unwrap_or(0);
                                    let ops = ops_since(since).await;
                                    let resp = DeltaResponse { ops };
                                    match serde_json::to_string(&resp) {
                                        Ok(mut s) => {
                                            s.push('\n');
                                            let _ = send.write_all(s.as_bytes()).await;
                                        }
                                        Err(e) => {
                                            warn!("serialize response failed: {e}");
                                        }
                                    }
                                    let _ = send.finish();
                                }
                                Err(e) => warn!("accept_bi failed: {e}"),
                            }
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

        // First call should create default config
        let config1 = load_or_create_config(&config_path).await.unwrap();
        assert!(config_path.exists());

        // Second call should load existing config
        let config2 = load_or_create_config(&config_path).await.unwrap();
        assert_eq!(config1.update.channel, config2.update.channel);
    }
}
