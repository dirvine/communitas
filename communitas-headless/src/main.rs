// Communitas Headless Node
// This binary runs a headless Communitas node using saorsa-gossip-based APIs

// Security: CLI tools may use unwrap in controlled contexts
// Core library crates maintain strict no-unwrap policies

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use anyhow::{Context, Result, anyhow};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use clap::Parser;
// Cryptography module with real ML-DSA-87 implementation
mod crypto;
use four_word_networking::FourWordAdaptiveEncoder;
use once_cell::sync::Lazy;
// use rand::RngCore; // Unused
// Removed: saorsa-core imports - replaced with saorsa-pqc and four-word-networking
// use saorsa_core::address::NetworkAddress;
// use saorsa_core::identity::FourWordAddress;
// use saorsa_core::quantum_crypto::{...};

// Removed: four_word_networking::FourWordAddress - using communitas_core::identity instead
// PQC crypto implementation provided by crypto module

use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::env;
use std::io::{ErrorKind, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tempfile::TempDir;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use self_update::update::{Release, ReleaseAsset, ReleaseUpdate};

fn decode_update_public_keys(keys: &[String]) -> Result<Vec<[u8; 32]>> {
    let mut parsed = Vec::new();
    for key in keys {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            continue;
        }
        let bytes = BASE64
            .decode(trimmed.as_bytes())
            .map_err(|e| anyhow!("Invalid update public key (base64 decode failed): {e}"))?;
        if bytes.len() != 32 {
            return Err(anyhow!(
                "Invalid update public key length: expected 32 bytes, got {}",
                bytes.len()
            ));
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        parsed.push(key_bytes);
    }
    Ok(parsed)
}

fn default_true() -> bool {
    true
}

fn build_download_headers(update: &dyn ReleaseUpdate) -> Result<HeaderMap> {
    let mut headers = update.api_headers(&update.auth_token())?;
    headers.insert(ACCEPT, HeaderValue::from_static("application/octet-stream"));
    Ok(headers)
}

fn download_asset_bytes(update: &dyn ReleaseUpdate, asset: &ReleaseAsset) -> Result<Vec<u8>> {
    let mut download = self_update::Download::from_url(&asset.download_url);
    download.set_headers(build_download_headers(update)?);
    let mut buffer = Vec::new();
    download
        .download_to(&mut buffer)
        .with_context(|| format!("Failed to download {}", asset.name))?;
    Ok(buffer)
}

fn download_asset_to_file(
    update: &dyn ReleaseUpdate,
    asset: &ReleaseAsset,
    dest: &Path,
) -> Result<()> {
    let mut file = std::fs::File::create(dest)
        .with_context(|| format!("Failed to create download destination {}", dest.display()))?;
    let mut download = self_update::Download::from_url(&asset.download_url);
    download.set_headers(build_download_headers(update)?);
    download.show_progress(update.show_download_progress());
    download.set_progress_style(update.progress_template(), update.progress_chars());
    download
        .download_to(&mut file)
        .with_context(|| format!("Failed to download {}", asset.name))?;
    Ok(())
}

fn is_hex_sha256(value: &str) -> bool {
    if value.len() != 64 {
        return false;
    }
    value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn matches_asset_name(token: &str, asset_name: &str) -> bool {
    let trimmed = token
        .trim()
        .trim_start_matches('*')
        .trim_matches('"')
        .trim_matches('\'');
    trimmed.ends_with(asset_name)
}

fn parse_checksum_for_asset(contents: &str, asset_name: &str) -> Result<String> {
    let mut hash_only_candidate = None;
    let mut meaningful_lines = 0;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        meaningful_lines += 1;

        if let Some(eq_idx) = trimmed.rfind('=') {
            let (left, right) = trimmed.split_at(eq_idx);
            if left.contains(asset_name) {
                let hash = right.trim_start_matches('=').trim();
                if is_hex_sha256(hash) {
                    return Ok(hash.to_ascii_lowercase());
                }
            }
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        if parts.len() == 1 {
            if is_hex_sha256(parts[0]) {
                hash_only_candidate = Some(parts[0].to_ascii_lowercase());
            }
            continue;
        }

        if is_hex_sha256(parts[0])
            && parts
                .iter()
                .skip(1)
                .any(|p| matches_asset_name(p, asset_name))
        {
            return Ok(parts[0].to_ascii_lowercase());
        }

        let last = parts[parts.len() - 1];
        if is_hex_sha256(last)
            && parts
                .iter()
                .take(parts.len() - 1)
                .any(|p| matches_asset_name(p, asset_name))
        {
            return Ok(last.to_ascii_lowercase());
        }
    }

    if meaningful_lines == 1
        && let Some(hash) = hash_only_candidate
    {
        return Ok(hash);
    }

    Err(anyhow!(
        "No SHA256 checksum entry found for asset {}",
        asset_name
    ))
}

fn find_checksum_asset(release: &Release, asset_name: &str) -> Option<ReleaseAsset> {
    let direct_candidates = [
        format!("{}.sha256", asset_name),
        format!("{}.sha256.txt", asset_name),
        format!("{}.sha256sum", asset_name),
        format!("{}.sha256sums", asset_name),
    ];
    for candidate in direct_candidates {
        if let Some(asset) = release.assets.iter().find(|a| a.name == candidate) {
            return Some(asset.clone());
        }
    }

    let general_candidates = [
        "SHA256SUMS",
        "SHA256SUMS.txt",
        "sha256sums",
        "sha256sums.txt",
        "sha256.txt",
    ];
    for candidate in general_candidates {
        if let Some(asset) = release
            .assets
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(candidate))
        {
            return Some(asset.clone());
        }
    }

    release
        .assets
        .iter()
        .find(|asset| {
            let lower = asset.name.to_ascii_lowercase();
            lower.contains("sha256") && lower.contains("sum")
        })
        .cloned()
}

fn sha256_hex_for_file(path: &Path) -> Result<String> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn replace_var(mut input: String, var: &str, value: &str) -> String {
    let patterns = [
        format!("{{{{{}}}}}", var),
        format!("{{{{ {}}}}}", var),
        format!("{{{{{} }}}}", var),
        format!("{{{{ {} }}}}", var),
    ];
    for pattern in patterns {
        input = input.replace(&pattern, value);
    }
    input
}

fn resolve_bin_path_in_archive(
    template: &str,
    bin_name: &str,
    target: &str,
    version: &str,
) -> String {
    let mut path = template.to_string();
    path = replace_var(path, "bin", bin_name);
    path = replace_var(path, "target", target);
    path = replace_var(path, "version", version);
    path
}

fn extract_and_replace(
    update: &dyn ReleaseUpdate,
    release: &Release,
    archive_path: &Path,
    tmp_dir: &Path,
) -> Result<()> {
    let bin_path_template = update.bin_path_in_archive();
    let bin_path = resolve_bin_path_in_archive(
        &bin_path_template,
        &update.bin_name(),
        &update.target(),
        &release.version,
    );
    self_update::Extract::from_source(archive_path)
        .extract_file(tmp_dir, &bin_path)
        .with_context(|| format!("Failed to extract {}", bin_path))?;
    let new_exe = tmp_dir.join(&bin_path);
    self_update::self_replace::self_replace(new_exe).context("Failed to replace binary")?;
    Ok(())
}

fn try_self_update_with_checksum(
    owner: &str,
    name: &str,
    require_checksum: bool,
) -> Result<Option<String>> {
    use self_update::cargo_crate_version;

    let mut cfg = self_update::backends::github::Update::configure();
    cfg.repo_owner(owner)
        .repo_name(name)
        .bin_name("communitas-headless")
        .current_version(cargo_crate_version!())
        .no_confirm(true);

    let update = cfg.build()?;
    let release = match update.target_version() {
        None => {
            let latest = update.get_latest_release()?;
            if !self_update::version::bump_is_greater(&update.current_version(), &latest.version)? {
                return Ok(None);
            }
            latest
        }
        Some(ver) => update.get_release_version(&ver)?,
    };

    let target = update.target();
    let target_asset = release
        .asset_for(&target, update.identifier().as_deref())
        .ok_or_else(|| anyhow!("No asset found for target: {}", target))?;

    let checksum_asset = find_checksum_asset(&release, &target_asset.name);
    let expected_checksum = if let Some(asset) = checksum_asset {
        let checksum_bytes = download_asset_bytes(update.as_ref(), &asset)?;
        let checksum_text =
            String::from_utf8(checksum_bytes).context("Checksum asset is not valid UTF-8")?;
        Some(parse_checksum_for_asset(
            &checksum_text,
            &target_asset.name,
        )?)
    } else {
        if require_checksum {
            return Err(anyhow!(
                "No SHA256 checksum asset found for {}",
                target_asset.name
            ));
        }
        warn!(
            "No checksum asset found for {}; proceeding without SHA256 verification",
            target_asset.name
        );
        None
    };

    let tmp_dir = TempDir::new().context("Failed to create temporary update directory")?;
    let archive_path = tmp_dir.path().join(&target_asset.name);
    download_asset_to_file(update.as_ref(), &target_asset, &archive_path)?;

    if let Some(expected) = expected_checksum {
        let actual = sha256_hex_for_file(&archive_path)?;
        if actual != expected {
            return Err(anyhow!(
                "SHA256 mismatch for {} (expected {}, got {})",
                target_asset.name,
                expected,
                actual
            ));
        }
    }

    extract_and_replace(update.as_ref(), &release, &archive_path, tmp_dir.path())?;
    Ok(Some(release.version))
}

/// Try to self-update the binary using GitHub releases
pub fn try_self_update(
    verifying_keys: Vec<[u8; 32]>,
    require_checksum: bool,
) -> Result<Option<String>> {
    use self_update::cargo_crate_version;
    let owner =
        std::env::var("COMMUNITAS_UPDATE_REPO_OWNER").unwrap_or_else(|_| "dirvine".to_string());
    let name =
        std::env::var("COMMUNITAS_UPDATE_REPO_NAME").unwrap_or_else(|_| "communitas".to_string());

    // Primary attempt
    let update_attempt = |repo_owner: &str| -> Result<Option<String>> {
        if verifying_keys.is_empty() {
            warn!("Self-update signature verification disabled (no public keys configured)");
            return try_self_update_with_checksum(repo_owner, &name, require_checksum);
        }

        let mut cfg = self_update::backends::github::Update::configure();
        cfg.repo_owner(repo_owner)
            .repo_name(&name)
            .bin_name("communitas-headless")
            .current_version(cargo_crate_version!())
            .no_confirm(true)
            .verifying_keys(verifying_keys.clone());
        let status = cfg.build()?.update()?;
        if status.updated() {
            Ok(Some(status.version().to_string()))
        } else {
            Ok(None)
        }
    };

    match update_attempt(&owner) {
        Ok(result) => Ok(result),
        Err(e1) => {
            let fallback_owner = if owner == "dirvine" {
                "david-irvine"
            } else {
                "dirvine"
            };
            update_attempt(fallback_owner).map_err(|_| e1)
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

    /// Jitter range in seconds (0 disables jitter)
    jitter_secs: u64,

    /// Base64-encoded public keys for signed update verification
    #[serde(default)]
    public_keys_base64: Vec<String>,

    /// Require SHA256 checksum verification when signatures are unavailable
    #[serde(default = "default_true")]
    require_checksum: bool,
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
            // Saorsa Network Bootstrap Nodes
            // See docs/infrastructure/INFRASTRUCTURE.md for full node list
            "142.93.199.50:11000".to_string(), // saorsa-2: DigitalOcean NYC1 bootstrap
            "147.182.234.192:11000".to_string(), // saorsa-3: DigitalOcean SFO3 bootstrap
            "206.189.7.117:11000".to_string(), // saorsa-4: DigitalOcean AMS3 test node
            "144.126.230.161:11000".to_string(), // saorsa-5: DigitalOcean LON1 test node
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
                // IPv4 wildcard address with random port - listens on all IPv4 interfaces
                // Port 0 tells OS to assign a random available port >1024 (no admin needed)
                std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0),
                // IPv6 wildcard address with random port - listens on all IPv6 interfaces
                std::net::SocketAddr::new(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0),
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
            jitter_secs: 0,
            public_keys_base64: vec![],
            require_checksum: true,
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
#[allow(dead_code)]
const IDENTITY_GENERATION_ATTEMPTS: usize = 1024;

#[derive(Debug)]
struct IdentityMaterial {
    four_words: String,
    // Real ML-DSA-87 keys
    mldsa87_public: Vec<u8>,
    mldsa87_secret: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredIdentity {
    four_words: String,
    // Real ML-DSA-87 keys (base64 encoded)
    mldsa87_public: String,
    mldsa87_secret: String,
}

impl StoredIdentity {
    fn into_material(self) -> Result<IdentityMaterial> {
        let StoredIdentity {
            four_words,
            mldsa87_public,
            mldsa87_secret,
        } = self;

        let canonical = canonicalize_four_words(&four_words)?;

        let mldsa87_public = BASE64
            .decode(&mldsa87_public)
            .context("Failed to decode ML-DSA-87 public key from base64")?;
        let mldsa87_secret = BASE64
            .decode(&mldsa87_secret)
            .context("Failed to decode ML-DSA-87 private key from base64")?;

        if mldsa87_public.len() != 2592 {
            return Err(anyhow::anyhow!(
                "Stored ML-DSA-87 public key must be 2592 bytes, got {}",
                mldsa87_public.len()
            ));
        }
        if mldsa87_secret.len() != 4896 {
            return Err(anyhow::anyhow!(
                "Stored ML-DSA-87 secret key must be 4896 bytes, got {}",
                mldsa87_secret.len()
            ));
        }

        Ok(IdentityMaterial {
            four_words: canonical,
            mldsa87_public,
            mldsa87_secret,
        })
    }
}

fn canonicalize_four_words(input: &str) -> Result<String> {
    let trimmed = input.trim();
    let words: Vec<&str> = trimmed.split('-').collect();
    if words.len() != 4 {
        return Err(anyhow!(
            "Four-word identity must contain exactly 4 words, found {}",
            words.len()
        ));
    }
    // We assume communitas_core::identity is available as per existing code
    if !communitas_core::identity::validate_id_words(trimmed) {
        return Err(anyhow!(
            "Four-word identity contains words outside the allowed dictionary"
        ));
    }
    Ok(trimmed.to_string())
}

fn generate_random_four_words() -> Result<String> {
    communitas_core::identity::generate_id_words()
        .map_err(|e| anyhow!("Failed to generate identity: {}", e))
}

async fn persist_identity_to_disk(path: &Path, material: &IdentityMaterial) -> Result<()> {
    let stored = StoredIdentity {
        four_words: material.four_words.clone(),
        mldsa87_public: BASE64.encode(&material.mldsa87_public),
        mldsa87_secret: BASE64.encode(&material.mldsa87_secret),
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
            // If parsing fails (e.g. old format), we regenerate
            let stored: StoredIdentity = match serde_json::from_slice(&bytes) {
                Ok(s) => s,
                Err(_) => {
                    warn!("Failed to parse existing identity (likely old format), regenerating...");
                    return generate_new_identity(config, &identity_path).await;
                }
            };
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
            generate_new_identity(config, &identity_path).await
        }
        Err(err) => Err(anyhow!(
            "Failed to read identity file {}: {}",
            identity_path.display(),
            err
        )),
    }
}

async fn generate_new_identity(config: &Config, identity_path: &Path) -> Result<IdentityMaterial> {
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

    // Try to load keys from keystore first (if they exist for this identity)
    // If not, generate new ones
    let (mldsa87_public, mldsa87_secret) = match crypto::load_keys_from_keystore(&four_words) {
        Ok(keys) => {
            info!("Loaded keys from system keystore for {}", four_words);
            keys
        }
        Err(_) => {
            info!("Generating new ML-DSA-87 keys for {}", four_words);
            let (pk, sk) = crypto::generate_mldsa87_keypair()
                .context("Failed to generate ML-DSA-87 transport keys")?;
            // Save to keystore
            if let Err(e) = crypto::save_keys_to_keystore(&four_words, &pk, &sk) {
                warn!(
                    "Failed to save keys to keystore (will proceed with file-only persistence for now): {}",
                    e
                );
            }
            (pk, sk)
        }
    };

    let material = IdentityMaterial {
        four_words,
        mldsa87_public,
        mldsa87_secret,
    };

    persist_identity_to_disk(identity_path, &material).await?;
    info!(
        "Persisted node identity {} to {}",
        material.four_words,
        identity_path.display()
    );

    Ok(material)
}

async fn start_health_endpoint(
    addr: SocketAddr,
    // Removed: _dht_client - saorsa-core removed
    // _dht_client: Arc<saorsa_core::messaging::DhtClient>,
    gossip: Option<Arc<communitas_core::GossipContext>>,
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

    // Clone gossip for each endpoint before moving into closures
    let gossip_for_metrics = gossip.clone();
    let metrics = warp::path("metrics").and(warp::get()).then(move || {
        let gossip = gossip_for_metrics.clone();
        async move {
            let (transport_peers, membership_peers) = if let Some(g) = gossip {
                let transport_count = g.transport.connected_peers().await.len();
                let membership_count = g.membership.read().await.active_view().len();
                (transport_count, membership_count)
            } else {
                (0, 0)
            };

            let response = format!(
                "# HELP communitas_peers_connected Number of connected peers (transport layer)\n\
                 # TYPE communitas_peers_connected gauge\n\
                 communitas_peers_connected {}\n\
                 # HELP communitas_membership_peers Number of peers in membership active view\n\
                 # TYPE communitas_membership_peers gauge\n\
                 communitas_membership_peers {}\n",
                transport_peers, membership_peers
            );

            warp::reply::html(response)
        }
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

    // Clone gossip context for API endpoints
    let gossip_for_peers = gossip.clone();
    let api_peers = warp::path!("api" / "peers").and(warp::get()).then(move || {
        let gossip = gossip_for_peers.clone();
        async move {
            let peers = if let Some(g) = gossip {
                let connected = g.transport.connected_peers().await;
                connected
                    .into_iter()
                    .map(|(peer_id, addr)| {
                        serde_json::json!({
                            "peer_id": format!("{:?}", peer_id),
                            "address": addr.to_string()
                        })
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };
            warp::reply::json(&serde_json::json!({
                "connected_peers": peers,
                "count": peers.len()
            }))
        }
    });

    let gossip_for_info = gossip.clone();
    let api_node_info = warp::path!("api" / "node-info")
        .and(warp::get())
        .then(move || {
            let gossip = gossip_for_info.clone();
            async move {
                let info = if let Some(g) = gossip {
                    let peers = g.transport.connected_peers().await;
                    serde_json::json!({
                        "four_words": g.four_words,
                        "display_name": g.display_name,
                        "device_name": g.device_name,
                        "connected_peers": peers.len(),
                        "status": "active"
                    })
                } else {
                    serde_json::json!({
                        "status": "no_gossip_context"
                    })
                };
                warp::reply::json(&info)
            }
        });

    let routes = health
        .or(metrics)
        .or(authenticate)
        .or(get_identity)
        .or(api_peers)
        .or(api_node_info);

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
#[allow(dead_code)]
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

        let quic_tls: saorsa_gossip_transport::quic::crypto::rustls::QuicClientConfig =
            StdArc::new(rustls_cfg)
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

                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Connection failed: {}", e)),
            }
        }
        Err(e) => Err(anyhow::anyhow!("Failed to initiate connection: {}", e)),
    }
}

use communitas_core::{CoreContext, types::DeviceType};

async fn run_node(args: Args) -> Result<()> {
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
    let update_keys = decode_update_public_keys(&config.update.public_keys_base64)?;

    // Self-update mode: do not start services
    if args.self_update {
        // Run the blocking self-update in a spawn_blocking task
        let update_keys = update_keys.clone();
        let require_checksum = config.update.require_checksum;
        match tokio::task::spawn_blocking(move || try_self_update(update_keys, require_checksum))
            .await
        {
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
        let update_keys = update_keys.clone();
        let require_checksum = config.update.require_checksum;
        match tokio::task::spawn_blocking(move || try_self_update(update_keys, require_checksum))
            .await
        {
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

    // Initialize CoreContext (Full Gossip Node)
    let mut context = CoreContext::initialize(
        identity.clone(),
        config
            .identity
            .clone()
            .unwrap_or_else(|| "Headless Node".to_string()),
        sanitize_instance_id(&instance_id),
        DeviceType::Headless,
        config.storage.base_dir.clone(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize CoreContext: {}", e))?;

    // Start networking
    // Use CLI --listen port first, then config listen_addrs, then random
    // Note: CoreContext handles port allocation logic
    let preferred_port = if args.listen.port() > 0 {
        // CLI --listen argument takes precedence
        Some(args.listen.port())
    } else {
        // Fall back to config file listen addresses
        config
            .network
            .listen_addrs
            .first()
            .map(|a| a.port())
            .filter(|&p| p > 0)
    };
    let connection_identity = context
        .start_networking(preferred_port)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start networking: {}", e))?;

    info!(
        "Gossip networking started. Connection identity: {}",
        connection_identity
    );

    // Connect to bootstrap nodes
    for bootstrap in &config.bootstrap_nodes {
        if let Err(e) = context.connect_to_peer(bootstrap).await {
            warn!("Failed to add bootstrap node {}: {}", bootstrap, e);
        } else {
            info!("Added bootstrap node: {}", bootstrap);
        }
    }

    // Start coordinator mode (this node acts as a bootstrap/relay coordinator)
    // Get external address from listen address
    let listen_port = args.listen.port();
    let external_addrs = if listen_port > 0 {
        // Try to get public IP
        let public_ip = local_ip_address::local_ip()
            .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
        vec![std::net::SocketAddr::new(public_ip, listen_port)]
    } else {
        // Use config listen addresses if available
        config.network.listen_addrs.clone()
    };

    if !external_addrs.is_empty() {
        if let Some(gossip_ctx) = &context.gossip {
            match gossip_ctx
                .start_coordinator_mode(external_addrs.clone(), Some(60))
                .await
            {
                Ok(_handle) => {
                    info!(
                        "Coordinator mode started with addresses: {:?}",
                        external_addrs
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to start coordinator mode: {}. Node will operate as regular peer.",
                        e
                    );
                }
            }
        } else {
            warn!("Gossip context not initialized - coordinator mode disabled");
        }
    } else {
        warn!("No external addresses configured - coordinator mode disabled");
    }

    // Start health/metrics endpoint if enabled
    if args.metrics {
        let ctx_clone = context.gossip.clone();
        start_health_endpoint(args.metrics_addr, ctx_clone).await?;
        info!("Metrics endpoint started on {}", args.metrics_addr);
    }

    // Main event loop
    info!("Communitas headless node started successfully");
    info!("Press Ctrl+C to shutdown");

    // Keep context alive
    let _context_guard = context;

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    // Graceful shutdown handled by Drop of CoreContext and other structs
    info!("Performing graceful shutdown...");

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

use saorsa_gossip_transport::TransportAdapter;
use saorsa_gossip_transport::quic::config::{
    ClientConfig as QuicClientConfig, ServerConfig as QuicServerConfig,
};
use saorsa_gossip_transport::quic::crypto::pqc::rustls_provider::{
    with_pqc_support, with_pqc_support_server,
};
use saorsa_gossip_transport::quic::crypto::raw_public_keys::RawPublicKeyConfigBuilder;
use saorsa_gossip_transport::quic::crypto::raw_public_keys::key_utils::{
    MlDsa65PublicKey, MlDsa65SecretKey, generate_keypair as generate_mldsa65_keypair,
};
use saorsa_gossip_transport::quic::high_level::Endpoint as QuicEndpoint;
use std::sync::Arc as StdArc;
// ant-quic send streams provide write_all via their API; no extra trait import needed
// Global connection tracking
use saorsa_gossip_transport::quic::HighLevelConnection;
static ACTIVE_CONNECTIONS: Lazy<Arc<RwLock<HashMap<String, HighLevelConnection>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

#[allow(dead_code)]
async fn start_quic_delta_server(
    listen: std::net::SocketAddr,
    base_dir: std::path::PathBuf,
    // ACCEPT ML-DSA-87 SECRET KEY
    _mldsa87_secret: &[u8],
) -> Result<std::net::SocketAddr> {
    // Persist or generate ML-DSA-65 transport keypair for raw public keys
    let public_key_path = base_dir.join("transport_mldsa65.pub");
    let secret_key_path = base_dir.join("transport_mldsa65.key");
    let (pk, sk): (MlDsa65PublicKey, MlDsa65SecretKey) = if public_key_path.exists()
        && secret_key_path.exists()
    {
        let pk_bytes = tokio::fs::read(&public_key_path)
            .await
            .context("read transport public key")?;
        let sk_bytes = tokio::fs::read(&secret_key_path)
            .await
            .context("read transport secret key")?;
        let pk = MlDsa65PublicKey::from_bytes(&pk_bytes).context("invalid transport public key")?;
        let sk = MlDsa65SecretKey::from_bytes(&sk_bytes).context("invalid transport secret key")?;
        (pk, sk)
    } else {
        let (pk, sk) = generate_mldsa65_keypair()
            .map_err(|e| anyhow::anyhow!("generate ML-DSA-65 keypair: {e}"))?;
        if let Some(parent) = secret_key_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("create key directory")?;
        }
        tokio::fs::write(&public_key_path, pk.as_bytes())
            .await
            .context("write transport public key")?;
        tokio::fs::write(&secret_key_path, sk.as_bytes())
            .await
            .context("write transport secret key")?;
        #[cfg(unix)]
        {
            std::fs::set_permissions(&secret_key_path, std::fs::Permissions::from_mode(0o600))
                .context("set secret key permissions")?;
        }
        (pk, sk)
    };
    info!("QUIC server raw key (hex): {}", hex::encode(pk.as_bytes()));

    // Build rustls server config with raw public key resolver
    // IMPORTANT: For bootstrap nodes, accept connections from any client (trust-on-first-use)
    // This allows bootstrap nodes to be publicly accessible without pre-shared keys
    let rustls_srv = RawPublicKeyConfigBuilder::new()
        .with_server_key(pk, sk)
        .enable_certificate_type_extensions()
        .allow_any_key() // Accept connections from any client
        .build_server_config()
        .map_err(|e| anyhow::anyhow!("raw pk server config: {e}"))?;

    // Convert to ant-quic server crypto config
    let quic_tls: saorsa_gossip_transport::quic::crypto::rustls::QuicServerConfig =
        StdArc::new(rustls_srv)
            .try_into()
            .map_err(|e| anyhow::anyhow!("convert tls server cfg: {e}"))?;
    let server_cfg = with_pqc_support_server(QuicServerConfig::with_crypto(StdArc::new(quic_tls)))
        .map_err(|e| anyhow::anyhow!("enable PQC on server: {e:?}"))?;

    // Bind endpoint
    let endpoint = QuicEndpoint::server(server_cfg, listen)
        .map_err(|e| anyhow::anyhow!("endpoint server bind: {e}"))?;

    // Get the actual bound address (important when binding to port 0 for random assignment)
    let actual_addr = endpoint
        .local_addr()
        .map_err(|e| anyhow::anyhow!("get local addr: {e}"))?;
    info!(
        "QUIC delta server listening on {} (actual bound address: {})",
        listen, actual_addr
    );

    // Convert to four-word address for easy sharing
    match four_word_networking::FourWordAdaptiveEncoder::new() {
        Ok(encoder) => match encoder.encode(&actual_addr.to_string()) {
            Ok(four_words) => info!("Four-word address: {}", four_words),
            Err(e) => warn!("Failed to encode four-word address: {}", e),
        },
        Err(e) => warn!("Failed to create encoder: {}", e),
    }

    // Spawn accept loop in background so we don't block
    tokio::spawn(async move {
        loop {
            match endpoint.accept().await {
                Some(incoming) => {
                    tokio::spawn(async move {
                        match incoming.await {
                            Ok(conn) => {
                                let remote_addr = conn.remote_address();
                                info!("Accepted QUIC connection from {}", remote_addr);

                                // Store the connection in active connections
                                let conn_id =
                                    format!("{}_{}", remote_addr, chrono::Utc::now().timestamp());
                                if let Ok(mut conns) = ACTIVE_CONNECTIONS.try_write() {
                                    // Store a clone of the connection handle to keep it in the map
                                    // Assuming HighLevelConnection is clonable (it usually is, wrapping an Arc)
                                    // If not, we rely on the fact that we don't use 'conn' below except for commented code.
                                    // But to be safe and consistent with connect_to_peer, we should insert it.
                                    conns.insert(conn_id.clone(), conn.clone());
                                    info!(
                                        "Stored incoming connection {} (total: {})",
                                        conn_id,
                                        conns.len()
                                    );
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
    });

    Ok(actual_addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ==================== decode_update_public_keys tests ====================

    #[test]
    fn test_decode_update_public_keys_valid() {
        // 32 bytes = AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA= in base64
        let valid_key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
        let result = decode_update_public_keys(&[valid_key]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], [0u8; 32]);
    }

    #[test]
    fn test_decode_update_public_keys_multiple() {
        let key1 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
        let key2 = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=".to_string();
        let result = decode_update_public_keys(&[key1, key2]).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], [0u8; 32]);
        assert_eq!(result[1], [1u8; 32]);
    }

    #[test]
    fn test_decode_update_public_keys_empty_string_skipped() {
        let valid_key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
        let result =
            decode_update_public_keys(&[valid_key, "".to_string(), "  ".to_string()]).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_decode_update_public_keys_invalid_base64() {
        let invalid = "not-valid-base64!!!".to_string();
        let result = decode_update_public_keys(&[invalid]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("base64 decode"));
    }

    #[test]
    fn test_decode_update_public_keys_wrong_length() {
        // 16 bytes instead of 32
        let short_key = "AAAAAAAAAAAAAAAAAAAAAA==".to_string();
        let result = decode_update_public_keys(&[short_key]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("expected 32 bytes")
        );
    }

    #[test]
    fn test_decode_update_public_keys_whitespace_trimmed() {
        let key_with_whitespace = "  AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=  ".to_string();
        let result = decode_update_public_keys(&[key_with_whitespace]).unwrap();
        assert_eq!(result.len(), 1);
    }

    // ==================== is_hex_sha256 tests ====================

    #[test]
    fn test_is_hex_sha256_valid() {
        let valid = "a".repeat(64);
        assert!(is_hex_sha256(&valid));
    }

    #[test]
    fn test_is_hex_sha256_valid_mixed_case() {
        let valid = "aAbBcCdDeEfF0123456789".to_string() + &"0".repeat(42);
        assert!(is_hex_sha256(&valid));
    }

    #[test]
    fn test_is_hex_sha256_too_short() {
        let short = "a".repeat(63);
        assert!(!is_hex_sha256(&short));
    }

    #[test]
    fn test_is_hex_sha256_too_long() {
        let long = "a".repeat(65);
        assert!(!is_hex_sha256(&long));
    }

    #[test]
    fn test_is_hex_sha256_non_hex_chars() {
        let invalid = "g".repeat(64); // 'g' is not a hex digit
        assert!(!is_hex_sha256(&invalid));
    }

    #[test]
    fn test_is_hex_sha256_empty() {
        assert!(!is_hex_sha256(""));
    }

    // ==================== matches_asset_name tests ====================

    #[test]
    fn test_matches_asset_name_exact() {
        assert!(matches_asset_name("binary.tar.gz", "binary.tar.gz"));
    }

    #[test]
    fn test_matches_asset_name_with_leading_asterisk() {
        assert!(matches_asset_name("*binary.tar.gz", "binary.tar.gz"));
    }

    #[test]
    fn test_matches_asset_name_with_quotes() {
        assert!(matches_asset_name("\"binary.tar.gz\"", "binary.tar.gz"));
        assert!(matches_asset_name("'binary.tar.gz'", "binary.tar.gz"));
    }

    #[test]
    fn test_matches_asset_name_with_whitespace() {
        assert!(matches_asset_name("  binary.tar.gz  ", "binary.tar.gz"));
    }

    #[test]
    fn test_matches_asset_name_with_path() {
        assert!(matches_asset_name(
            "./path/to/binary.tar.gz",
            "binary.tar.gz"
        ));
    }

    #[test]
    fn test_matches_asset_name_no_match() {
        assert!(!matches_asset_name("other.tar.gz", "binary.tar.gz"));
    }

    // ==================== parse_checksum_for_asset tests ====================

    #[test]
    fn test_parse_checksum_bsd_style() {
        // BSD style: SHA256 (filename) = hash
        let hash = "a".repeat(64);
        let contents = format!("SHA256 (binary.tar.gz) = {}", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_standard_format() {
        // Standard format: hash  filename
        let hash = "b".repeat(64);
        let contents = format!("{}  binary.tar.gz", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_hash_then_filename() {
        // hash filename (single space)
        let hash = "c".repeat(64);
        let contents = format!("{} binary.tar.gz", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_filename_then_hash() {
        // Some formats put filename first: binary.tar.gz hash
        let hash = "d".repeat(64);
        let contents = format!("binary.tar.gz {}", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_hash_only_single_line() {
        // Single line with just hash (for per-asset checksum files)
        let hash = "e".repeat(64);
        let result = parse_checksum_for_asset(&hash, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_ignores_comments() {
        let hash = "f".repeat(64);
        let contents = format!(
            "# This is a comment\n\
             # Another comment\n\
             {}  binary.tar.gz",
            hash
        );
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_ignores_empty_lines() {
        let hash = "0".repeat(64);
        let contents = format!("\n\n{}  binary.tar.gz\n\n", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    #[test]
    fn test_parse_checksum_multiple_entries() {
        let hash1 = "1".repeat(64);
        let hash2 = "2".repeat(64);
        let contents = format!(
            "{}  other.tar.gz\n\
             {}  binary.tar.gz",
            hash1, hash2
        );
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash2);
    }

    #[test]
    fn test_parse_checksum_not_found() {
        let hash = "3".repeat(64);
        let contents = format!("{}  other.tar.gz", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No SHA256 checksum")
        );
    }

    #[test]
    fn test_parse_checksum_lowercase_output() {
        // Input with uppercase hex should be lowercased
        let hash = "ABCDEF".to_string() + &"0".repeat(58);
        let contents = format!("{}  binary.tar.gz", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash.to_ascii_lowercase());
    }

    #[test]
    fn test_parse_checksum_with_asterisk_prefix() {
        // Some tools use *filename for binary mode
        let hash = "4".repeat(64);
        let contents = format!("{} *binary.tar.gz", hash);
        let result = parse_checksum_for_asset(&contents, "binary.tar.gz").unwrap();
        assert_eq!(result, hash);
    }

    // ==================== sha256_hex_for_file tests ====================

    #[test]
    fn test_sha256_hex_for_file_empty() {
        let file = NamedTempFile::new().unwrap();
        let result = sha256_hex_for_file(file.path()).unwrap();
        // SHA256 of empty file is well-known
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hex_for_file_known_content() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"hello world").unwrap();
        file.flush().unwrap();
        let result = sha256_hex_for_file(file.path()).unwrap();
        // SHA256 of "hello world" is well-known
        assert_eq!(
            result,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha256_hex_for_file_large_content() {
        let mut file = NamedTempFile::new().unwrap();
        // Write more than one buffer (8192 bytes) to test chunked reading
        let data = vec![0xABu8; 20000];
        file.write_all(&data).unwrap();
        file.flush().unwrap();
        let result = sha256_hex_for_file(file.path()).unwrap();
        // Just verify it returns a valid 64-char hex string
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex_for_file_not_found() {
        let result = sha256_hex_for_file(Path::new("/nonexistent/path/to/file"));
        assert!(result.is_err());
    }

    // ==================== Integration tests ====================

    #[test]
    fn test_checksum_verification_flow() {
        // Simulate the full checksum verification flow
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test binary content").unwrap();
        file.flush().unwrap();

        // Compute actual hash
        let actual_hash = sha256_hex_for_file(file.path()).unwrap();

        // Create checksum file content
        let checksum_content = format!("{}  binary.tar.gz", actual_hash);

        // Parse and verify
        let expected_hash = parse_checksum_for_asset(&checksum_content, "binary.tar.gz").unwrap();
        assert_eq!(actual_hash, expected_hash);
    }

    // ==================== ML-DSA-65 Transport Key Persistence Tests ====================

    use saorsa_gossip_transport::quic::crypto::raw_public_keys::key_utils::{
        MlDsa65PublicKey, MlDsa65SecretKey, generate_keypair as generate_mldsa65_keypair,
    };

    #[test]
    fn test_mldsa65_keypair_generation() {
        let result = generate_mldsa65_keypair();
        assert!(
            result.is_ok(),
            "ML-DSA-65 keypair generation should succeed"
        );

        let (pk, sk) = result.unwrap();
        // ML-DSA-65 public key is 1952 bytes
        assert_eq!(
            pk.as_bytes().len(),
            1952,
            "ML-DSA-65 public key should be 1952 bytes"
        );
        // ML-DSA-65 secret key is 4032 bytes
        assert_eq!(
            sk.as_bytes().len(),
            4032,
            "ML-DSA-65 secret key should be 4032 bytes"
        );
    }

    #[test]
    fn test_mldsa65_keypair_uniqueness() {
        let (pk1, sk1) = generate_mldsa65_keypair().unwrap();
        let (pk2, sk2) = generate_mldsa65_keypair().unwrap();

        assert_ne!(
            pk1.as_bytes(),
            pk2.as_bytes(),
            "Generated public keys should be unique"
        );
        assert_ne!(
            sk1.as_bytes(),
            sk2.as_bytes(),
            "Generated secret keys should be unique"
        );
    }

    #[test]
    fn test_mldsa65_public_key_roundtrip() {
        let (pk, _sk) = generate_mldsa65_keypair().unwrap();
        let bytes = pk.as_bytes().to_vec();

        let restored = MlDsa65PublicKey::from_bytes(&bytes);
        assert!(
            restored.is_ok(),
            "Public key deserialization should succeed"
        );

        let restored_pk = restored.unwrap();
        assert_eq!(
            pk.as_bytes(),
            restored_pk.as_bytes(),
            "Restored public key should match original"
        );
    }

    #[test]
    fn test_mldsa65_secret_key_roundtrip() {
        let (_pk, sk) = generate_mldsa65_keypair().unwrap();
        let bytes = sk.as_bytes().to_vec();

        let restored = MlDsa65SecretKey::from_bytes(&bytes);
        assert!(
            restored.is_ok(),
            "Secret key deserialization should succeed"
        );

        let restored_sk = restored.unwrap();
        assert_eq!(
            sk.as_bytes(),
            restored_sk.as_bytes(),
            "Restored secret key should match original"
        );
    }

    #[test]
    fn test_mldsa65_public_key_invalid_length() {
        // Too short
        let short_bytes = vec![0u8; 100];
        let result = MlDsa65PublicKey::from_bytes(&short_bytes);
        assert!(result.is_err(), "Should reject short public key");

        // Too long
        let long_bytes = vec![0u8; 2000];
        let result = MlDsa65PublicKey::from_bytes(&long_bytes);
        assert!(result.is_err(), "Should reject long public key");
    }

    #[test]
    fn test_mldsa65_secret_key_invalid_length() {
        // Too short
        let short_bytes = vec![0u8; 100];
        let result = MlDsa65SecretKey::from_bytes(&short_bytes);
        assert!(result.is_err(), "Should reject short secret key");

        // Too long
        let long_bytes = vec![0u8; 5000];
        let result = MlDsa65SecretKey::from_bytes(&long_bytes);
        assert!(result.is_err(), "Should reject long secret key");
    }

    #[test]
    fn test_mldsa65_key_file_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let public_key_path = temp_dir.path().join("transport_mldsa65.pub");
        let secret_key_path = temp_dir.path().join("transport_mldsa65.key");

        // Generate keypair
        let (pk, sk) = generate_mldsa65_keypair().unwrap();

        // Write to files
        std::fs::write(&public_key_path, pk.as_bytes()).unwrap();
        std::fs::write(&secret_key_path, sk.as_bytes()).unwrap();

        // Read back
        let pk_bytes = std::fs::read(&public_key_path).unwrap();
        let sk_bytes = std::fs::read(&secret_key_path).unwrap();

        // Verify sizes
        assert_eq!(pk_bytes.len(), 1952);
        assert_eq!(sk_bytes.len(), 4032);

        // Restore and verify
        let restored_pk = MlDsa65PublicKey::from_bytes(&pk_bytes).unwrap();
        let restored_sk = MlDsa65SecretKey::from_bytes(&sk_bytes).unwrap();

        assert_eq!(pk.as_bytes(), restored_pk.as_bytes());
        assert_eq!(sk.as_bytes(), restored_sk.as_bytes());
    }

    #[cfg(unix)]
    #[test]
    fn test_mldsa65_secret_key_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let secret_key_path = temp_dir.path().join("transport_mldsa65.key");

        let (_pk, sk) = generate_mldsa65_keypair().unwrap();
        std::fs::write(&secret_key_path, sk.as_bytes()).unwrap();

        // Set restrictive permissions (0o600 = owner read/write only)
        std::fs::set_permissions(&secret_key_path, std::fs::Permissions::from_mode(0o600)).unwrap();

        let metadata = std::fs::metadata(&secret_key_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "Secret key should have restrictive permissions"
        );
    }

    #[test]
    fn test_mldsa65_key_persistence_roundtrip_integrity() {
        // Simulate the full persistence flow from start_quic_delta_server
        let temp_dir = tempfile::tempdir().unwrap();
        let public_key_path = temp_dir.path().join("transport_mldsa65.pub");
        let secret_key_path = temp_dir.path().join("transport_mldsa65.key");

        // First run: generate and persist
        let (original_pk, original_sk) = generate_mldsa65_keypair().unwrap();
        std::fs::write(&public_key_path, original_pk.as_bytes()).unwrap();
        std::fs::write(&secret_key_path, original_sk.as_bytes()).unwrap();

        // Second run: load from disk
        let pk_bytes = std::fs::read(&public_key_path).unwrap();
        let sk_bytes = std::fs::read(&secret_key_path).unwrap();
        let loaded_pk = MlDsa65PublicKey::from_bytes(&pk_bytes).unwrap();
        let loaded_sk = MlDsa65SecretKey::from_bytes(&sk_bytes).unwrap();

        // Verify loaded keys match originals
        assert_eq!(
            original_pk.as_bytes(),
            loaded_pk.as_bytes(),
            "Loaded public key must match original"
        );
        assert_eq!(
            original_sk.as_bytes(),
            loaded_sk.as_bytes(),
            "Loaded secret key must match original"
        );
    }

    #[test]
    fn test_mldsa65_nonexistent_key_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let public_key_path = temp_dir.path().join("nonexistent.pub");
        let secret_key_path = temp_dir.path().join("nonexistent.key");

        // Verify files don't exist
        assert!(!public_key_path.exists());
        assert!(!secret_key_path.exists());

        // Reading should fail
        assert!(std::fs::read(&public_key_path).is_err());
        assert!(std::fs::read(&secret_key_path).is_err());
    }

    #[test]
    fn test_mldsa65_keys_not_all_zeros() {
        let (pk, sk) = generate_mldsa65_keypair().unwrap();

        // Keys should not be all zeros
        assert!(
            pk.as_bytes().iter().any(|&b| b != 0),
            "Public key should not be all zeros"
        );
        assert!(
            sk.as_bytes().iter().any(|&b| b != 0),
            "Secret key should not be all zeros"
        );
    }
}
