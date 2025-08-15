// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! Standalone Bootstrap Node Binary for Communitas
//!
//! This binary runs the bootstrap node in production environments.

use anyhow::Result;
use clap::{Arg, Command};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info};

use communitas_tauri::bootstrap::{run_bootstrap_node, BootstrapConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("communitas-bootstrap")
        .version("2.0.0")
        .about("Communitas P2P Bootstrap Node")
        .arg(
            Arg::new("listen")
                .long("listen")
                .short('l')
                .help("Listen address (e.g., 0.0.0.0:8888)")
                .default_value("0.0.0.0:8888"),
        )
        .arg(
            Arg::new("public-address")
                .long("public-address")
                .short('p')
                .help("Public address for other nodes")
                .default_value("bootstrap.communitas.app:8888"),
        )
        .arg(
            Arg::new("data-dir")
                .long("data-dir")
                .short('d')
                .help("Data storage directory")
                .default_value("/var/lib/saorsa"),
        )
        .arg(
            Arg::new("max-connections")
                .long("max-connections")
                .help("Maximum concurrent connections")
                .default_value("1000"),
        )
        .arg(
            Arg::new("health-port")
                .long("health-port")
                .help("Health check server port")
                .default_value("8888"),
        )
        .get_matches();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,communitas=debug")
        .init();

    info!("Starting Communitas Bootstrap Node v2.0.0");

    // Build configuration from CLI arguments
    let listen_address: SocketAddr = matches.get_one::<String>("listen").unwrap().parse()?;

    let public_address = matches
        .get_one::<String>("public-address")
        .unwrap()
        .to_string();

    let data_dir = PathBuf::from(matches.get_one::<String>("data-dir").unwrap());

    let max_connections: usize = matches
        .get_one::<String>("max-connections")
        .unwrap()
        .parse()?;

    let health_check_port: u16 = matches.get_one::<String>("health-port").unwrap().parse()?;

    let config = BootstrapConfig {
        listen_address,
        public_address,
        max_connections,
        data_dir: data_dir.clone(),
        dht_storage_path: data_dir.join("dht"),
        enable_health_check: true,
        health_check_port,
        connection_timeout: 30,
    };

    info!("Configuration loaded:");
    info!("  Listen address: {}", config.listen_address);
    info!("  Public address: {}", config.public_address);
    info!("  Data directory: {}", config.data_dir.display());
    info!("  Max connections: {}", config.max_connections);
    info!("  Health check port: {}", config.health_check_port);

    // Run the bootstrap node
    if let Err(e) = run_bootstrap_node(config).await {
        error!("Bootstrap node failed: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
