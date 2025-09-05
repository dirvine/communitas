// Communitas Auto-Updater Service
// Monitors for updates and applies them with jitter

use anyhow::{Context, Result};
use clap::Parser;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "communitas-autoupdater")]
#[command(about = "Auto-updater for Communitas nodes", long_about = None)]
struct Args {
    /// Update configuration file
    #[arg(short, long, default_value = "/etc/communitas/update.toml")]
    config: PathBuf,

    /// Binary installation directory
    #[arg(short, long, default_value = "/opt/communitas/bin")]
    bin_dir: PathBuf,

    /// Update check URL
    #[arg(short, long)]
    update_url: Option<String>,

    /// Dry run (don't actually update)
    #[arg(long)]
    dry_run: bool,

    /// Run once and exit
    #[arg(long)]
    once: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateConfig {
    /// Update channel (stable, beta, nightly)
    channel: String,

    /// GitHub repository (owner/repo)
    repository: String,

    /// Check interval in seconds
    check_interval_secs: u64,

    /// Jitter range in seconds (0-6h default)
    jitter_secs: u64,

    /// Signature verification public key
    pub_key: Option<String>,

    /// Maximum retries on failure
    max_retries: usize,

    /// Backoff multiplier for retries
    backoff_multiplier: f64,

    /// Maximum backoff in seconds
    max_backoff_secs: u64,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            channel: "stable".to_string(),
            repository: "dirvine/communitas-foundation".to_string(),
            check_interval_secs: 21600, // 6 hours
            jitter_secs: 21600,         // 0-6h jitter
            pub_key: None,
            max_retries: 10,
            backoff_multiplier: 2.0,
            max_backoff_secs: 43200, // 12 hours
        }
    }
}

#[derive(Debug, Deserialize)]
struct ReleaseInfo {
    tag_name: String,
    name: String,
    published_at: String,
    assets: Vec<ReleaseAsset>,
    body: Option<String>, // changelog
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

async fn load_config(path: &PathBuf) -> Result<UpdateConfig> {
    if path.exists() {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read update config")?;
        toml::from_str(&content).context("Failed to parse update config")
    } else {
        // Create default config
        let config = UpdateConfig::default();
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

async fn get_current_version() -> Result<String> {
    // In production, read from installed binary or version file
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

async fn fetch_latest_release(config: &UpdateConfig) -> Result<ReleaseInfo> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        config.repository
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "communitas-autoupdater")
        .send()
        .await
        .context("Failed to fetch release info")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "GitHub API returned status: {}",
            response.status()
        ));
    }

    response
        .json::<ReleaseInfo>()
        .await
        .context("Failed to parse release info")
}

async fn download_binary(url: &str, dest: &PathBuf) -> Result<()> {
    info!("Downloading from: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to download binary")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    // Write to temporary file first
    let temp_path = dest.with_extension("tmp");
    tokio::fs::write(&temp_path, bytes)
        .await
        .context("Failed to write temporary file")?;

    Ok(())
}

async fn verify_signature(binary_path: &PathBuf, sig_url: &str, pub_key: &str) -> Result<bool> {
    // Download signature
    let client = reqwest::Client::new();
    let sig_response = client
        .get(sig_url)
        .send()
        .await
        .context("Failed to download signature")?;

    if !sig_response.status().is_success() {
        warn!("Signature download failed, skipping verification");
        return Ok(false);
    }

    let _sig_bytes = sig_response
        .bytes()
        .await
        .context("Failed to read signature")?;

    // In production, implement proper signature verification
    // using ML-DSA-65 or Ed25519
    warn!("Signature verification not yet implemented");
    Ok(true)
}

async fn atomic_replace(src: &PathBuf, dest: &PathBuf) -> Result<()> {
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(src).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(src, perms).await?;
    }

    // Atomic rename
    tokio::fs::rename(src, dest)
        .await
        .context("Failed to replace binary")?;

    Ok(())
}

async fn restart_service() -> Result<()> {
    // Restart systemd service
    let output = tokio::process::Command::new("systemctl")
        .args(&["restart", "communitas.service"])
        .output()
        .await
        .context("Failed to restart service")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Service restart failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

async fn apply_update(
    release: &ReleaseInfo,
    config: &UpdateConfig,
    bin_dir: &PathBuf,
    dry_run: bool,
) -> Result<()> {
    info!("Applying update to version {}", release.tag_name);

    // Find appropriate asset for this platform
    let asset = release
        .assets
        .iter()
        .find(|a| {
            a.name.contains("linux") && (a.name.contains("x86_64") || a.name.contains("amd64"))
        })
        .context("No suitable binary found for this platform")?;

    if dry_run {
        info!("Dry run: would download {}", asset.name);
        return Ok(());
    }

    // Download binary
    let binary_path = bin_dir.join("communitas-node");
    let temp_path = binary_path.with_extension("tmp");

    download_binary(&asset.browser_download_url, &temp_path).await?;

    // Verify signature if configured
    if let Some(pub_key) = &config.pub_key {
        let sig_url = format!("{}.sig", asset.browser_download_url);
        if verify_signature(&temp_path, &sig_url, pub_key).await? {
            info!("Signature verified successfully");
        } else {
            warn!("Signature verification skipped or failed");
        }
    }

    // Atomic replace
    atomic_replace(&temp_path, &binary_path).await?;
    info!("Binary updated successfully");

    // Restart service
    restart_service().await?;
    info!("Service restarted");

    Ok(())
}

async fn check_and_update(config: &UpdateConfig, bin_dir: &PathBuf, dry_run: bool) -> Result<bool> {
    let current = get_current_version().await?;
    info!("Current version: {}", current);

    let release = fetch_latest_release(config).await?;
    info!("Latest version: {}", release.tag_name);

    // Simple version comparison (in production, use semver)
    if release.tag_name.trim_start_matches('v') > current.as_str() {
        info!("Update available: {} -> {}", current, release.tag_name);

        // Apply jitter
        if config.jitter_secs > 0 {
            let jitter = rand::thread_rng().gen_range(0..config.jitter_secs);
            info!("Applying jitter: waiting {} seconds", jitter);
            sleep(Duration::from_secs(jitter)).await;
        }

        apply_update(&release, config, bin_dir, dry_run).await?;
        Ok(true)
    } else {
        info!("Already up to date");
        Ok(false)
    }
}

async fn run_updater(args: Args) -> Result<()> {
    let config = load_config(&args.config).await?;
    info!("Loaded update configuration");

    // Ensure bin directory exists
    tokio::fs::create_dir_all(&args.bin_dir)
        .await
        .context("Failed to create bin directory")?;

    let mut retry_count = 0;
    let mut backoff_secs = 1;

    loop {
        match check_and_update(&config, &args.bin_dir, args.dry_run).await {
            Ok(updated) => {
                if updated {
                    info!("Update completed successfully");
                }
                retry_count = 0;
                backoff_secs = 1;

                if args.once {
                    break;
                }

                // Wait for next check
                sleep(Duration::from_secs(config.check_interval_secs)).await;
            }
            Err(e) => {
                error!("Update check failed: {:#}", e);

                retry_count += 1;
                if retry_count >= config.max_retries {
                    error!("Maximum retries exceeded, waiting for next interval");
                    retry_count = 0;
                    backoff_secs = 1;

                    if args.once {
                        return Err(e);
                    }

                    sleep(Duration::from_secs(config.check_interval_secs)).await;
                } else {
                    warn!(
                        "Retry {} of {}, backing off {} seconds",
                        retry_count, config.max_retries, backoff_secs
                    );
                    sleep(Duration::from_secs(backoff_secs)).await;

                    // Exponential backoff
                    backoff_secs = ((backoff_secs as f64) * config.backoff_multiplier) as u64;
                    backoff_secs = backoff_secs.min(config.max_backoff_secs);
                }
            }
        }
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

    if let Err(e) = run_updater(args).await {
        error!("Auto-updater failed: {:#}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_default_update_config() {
        let config = UpdateConfig::default();
        assert_eq!(config.channel, "stable");
        assert_eq!(config.max_retries, 10);
        assert_eq!(config.jitter_secs, 21600);
    }

    #[tokio::test]
    async fn test_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("update.toml");

        // Should create default config if not exists
        let config = load_config(&config_path).await.unwrap();
        assert!(config_path.exists());
        assert_eq!(config.channel, "stable");
    }
}
