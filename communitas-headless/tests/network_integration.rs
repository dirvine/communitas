use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn build_binary() -> std::path::PathBuf {
    let status = Command::new("cargo")
        .args(&["build", "-p", "communitas-headless"])
        .status()
        .expect("Failed to build binary");
    assert!(status.success());

    let cwd = std::env::current_dir().unwrap();
    // println!("Current directory: {:?}", cwd);

    let candidates = vec![
        cwd.join("target/debug/communitas-headless"),
        cwd.join("../target/debug/communitas-headless"),
        std::path::PathBuf::from("target/debug/communitas-headless"), // Relative
    ];

    for path in candidates {
        if path.exists() {
            // println!("Found binary at: {:?}", path);
            return path.canonicalize().unwrap();
        }
    }

    panic!("Could not find communitas-headless binary in target/debug");
}

// Helper to capture logs in background and search for patterns
struct LogMonitor {
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogMonitor {
    fn new(child_stdout: std::process::ChildStdout, prefix: &'static str) -> Self {
        let logs = Arc::new(Mutex::new(Vec::new()));
        let logs_clone = logs.clone();

        thread::spawn(move || {
            let reader = BufReader::new(child_stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("{}: {}", prefix, l); // Echo to test stdout
                    logs_clone.lock().unwrap().push(l);
                }
            }
        });

        Self { logs }
    }

    fn wait_for(&self, pattern: &str, timeout: Duration) -> Option<String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            {
                let guard = self.logs.lock().unwrap();
                for line in guard.iter() {
                    if line.contains(pattern) {
                        return Some(line.clone());
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        None
    }
}

#[test]
fn test_two_nodes_connection() {
    // Kill any zombie processes from previous runs
    let _ = Command::new("pkill").arg("communitas-headless").status();
    thread::sleep(Duration::from_secs(1)); // Wait for kill

    let binary = build_binary();

    // Create temp dirs for nodes
    let temp_dir = std::env::temp_dir().join("communitas-integration-test");
    let _ = std::fs::remove_dir_all(&temp_dir); // Clean up previous
    let dir_a = temp_dir.join("node_a");
    let dir_b = temp_dir.join("node_b");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();

    // Create config for Node A
    let config_a_path = dir_a.join("config.toml");
    let config_a_content = r#"
identity = "ocean-forest-moon-star"
bootstrap_nodes = []

[storage]
base_dir = "data"
cache_size_mb = 1024
enable_fec = true
fec_k = 8
fec_m = 4

[network]
listen_addrs = ["127.0.0.1:0"]
enable_ipv6 = false
enable_webrtc = false
quic_idle_timeout_ms = 30000
quic_max_streams = 100

[update]
channel = "stable"
check_interval_secs = 21600
auto_update = false
jitter_secs = 0
"#;
    std::fs::write(&config_a_path, config_a_content).unwrap();

    // Start Node A
    let mut node_a = Command::new(&binary)
        .arg("--config")
        .arg(&config_a_path)
        .arg("--instance-id")
        .arg("node-a")
        .arg("--storage")
        .arg(dir_a.join("data"))
        .arg("--listen")
        .arg("127.0.0.1:0") // CLI overrides config
        .arg("--metrics")
        .arg("--metrics-addr")
        .arg("127.0.0.1:9601")
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Node A");

    let stdout_a = node_a.stdout.take().unwrap();
    let monitor_a = LogMonitor::new(stdout_a, "Node A");

    // Wait for Node A to start and get its address
    // Look for "Gossip networking started on IP:PORT"
    let log_line = monitor_a
        .wait_for("Gossip networking started on", Duration::from_secs(10))
        .expect("Timed out waiting for Node A to start");

    // Parse address
    // Format: ... Gossip networking started on 127.0.0.1:54321 (identity words)
    let mut address_a = String::new();
    if let Some(idx) = log_line.find("on ") {
        let rest = &log_line[idx + 3..]; // Skip "on "
        if let Some(end) = rest.find(' ') {
            address_a = rest[..end].to_string();
        } else {
            address_a = rest.to_string();
        }
        // Clean up any trailing punctuation if any
        address_a = address_a.trim_matches(|c| c == ')' || c == ',').to_string();
    }

    println!("Node A started at {}", address_a);
    assert!(!address_a.is_empty());

    // Create config for Node B
    let config_b_path = dir_b.join("config.toml");
    let config_b_content = r#"
bootstrap_nodes = []

[storage]
base_dir = "data"
cache_size_mb = 1024
enable_fec = true
fec_k = 8
fec_m = 4

[network]
listen_addrs = ["127.0.0.1:0"]
enable_ipv6 = false
enable_webrtc = false
quic_idle_timeout_ms = 30000
quic_max_streams = 100

[update]
channel = "stable"
check_interval_secs = 21600
auto_update = false
jitter_secs = 0
"#;
    std::fs::write(&config_b_path, config_b_content).unwrap();

    // Start Node B, bootstrapping from Node A
    let mut node_b = Command::new(&binary)
        .arg("--config")
        .arg(&config_b_path)
        .arg("--instance-id")
        .arg("node-b")
        .arg("--storage")
        .arg(dir_b.join("data"))
        .arg("--listen")
        .arg("127.0.0.1:0")
        .arg("--metrics")
        .arg("--metrics-addr")
        .arg("127.0.0.1:9602")
        .arg("--bootstrap")
        .arg(&address_a)
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start Node B");

    let stdout_b = node_b.stdout.take().unwrap();
    let monitor_b = LogMonitor::new(stdout_b, "Node B");

    // Wait for Node B to connect to Node A
    monitor_b
        .wait_for("Connected to peer", Duration::from_secs(10))
        .expect("Node B failed to connect to Node A");

    // Give some time for connection to stabilize and metrics to update
    thread::sleep(Duration::from_secs(2));

    // Verify metrics
    // Check Node A metrics
    let status = Command::new("curl")
        .arg("-s")
        .arg("http://127.0.0.1:9601/metrics")
        .output()
        .expect("Failed to curl Node A metrics");
    let output_a = String::from_utf8_lossy(&status.stdout);
    println!("Node A Metrics:\n{}", output_a);
    assert!(output_a.contains("communitas_peers_connected 1"));

    // Check Node B metrics
    // Retry curl if port 9602 conflict occurred (though unlikely with killing previous)
    let status = Command::new("curl")
        .arg("-s")
        .arg("http://127.0.0.1:9602/metrics")
        .output()
        .expect("Failed to curl Node B metrics");
    let output_b = String::from_utf8_lossy(&status.stdout);
    println!("Node B Metrics:\n{}", output_b);
    assert!(output_b.contains("communitas_peers_connected 1"));

    // Cleanup
    let _ = node_a.kill();
    let _ = node_b.kill();
}
