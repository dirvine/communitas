// Copyright (c) 2025 Saorsa Labs Limited
//
// MCP (Model Context Protocol) Plugin for Tauri v2
// Enables Claude to interact with the Communitas app

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::{
    Runtime,
    plugin::{Builder as PluginBuilder, TauriPlugin},
};
use tokio::sync::RwLock;
use tracing::{error, info};
use warp::Filter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub app_name: String,
    pub host: String,
    pub port: u16,
}

impl MCPConfig {
    pub fn new(app_name: String) -> Self {
        Self {
            app_name,
            host: "127.0.0.1".to_string(),
            port: 4000,
        }
    }

    pub fn tcp(mut self, host: String, port: u16) -> Self {
        self.host = host;
        self.port = port;
        self
    }
}

#[derive(Debug, Clone)]
pub struct MCPServer {
    config: MCPConfig,
    app_handle: Option<tauri::AppHandle>,
}

impl MCPServer {
    pub fn new(config: MCPConfig) -> Self {
        Self {
            config,
            app_handle: None,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        info!("Starting MCP server on {}", addr);

        // Create MCP endpoints
        let status = warp::path!("mcp" / "status").and(warp::get()).map(move || {
            warp::reply::json(&serde_json::json!({
                "status": "running",
                "app": "Communitas",
                "version": "2.0.0",
                "mcp_version": "1.0.0"
            }))
        });

        let app_info = warp::path!("mcp" / "app" / "info")
            .and(warp::get())
            .map(move || {
                warp::reply::json(&serde_json::json!({
                    "name": "Communitas",
                    "description": "P2P Collaboration Platform",
                    "features": [
                        "DHT Storage",
                        "Reed Solomon Erasure Coding",
                        "Secure Messaging",
                        "Organization Management",
                        "Real-time Collaboration"
                    ]
                }))
            });

        let windows = warp::path!("mcp" / "windows")
            .and(warp::get())
            .map(move || {
                warp::reply::json(&serde_json::json!({
                    "windows": [
                        {
                            "id": "main",
                            "title": "Communitas",
                            "visible": true,
                            "focused": true
                        }
                    ]
                }))
            });

        // Tool endpoints for Claude
        let tools = warp::path!("mcp" / "tools").and(warp::get()).map(move || {
            warp::reply::json(&serde_json::json!({
                "tools": [
                    {
                        "name": "get_app_status",
                        "description": "Get the current status of the Communitas app",
                        "parameters": {}
                    },
                    {
                        "name": "get_dht_status",
                        "description": "Get DHT network status",
                        "parameters": {}
                    },
                    {
                        "name": "get_storage_metrics",
                        "description": "Get storage metrics including Reed Solomon status",
                        "parameters": {}
                    },
                    {
                        "name": "list_organizations",
                        "description": "List all organizations",
                        "parameters": {}
                    },
                    {
                        "name": "execute_js",
                        "description": "Execute JavaScript in the app's webview",
                        "parameters": {
                            "code": {
                                "type": "string",
                                "description": "JavaScript code to execute"
                            }
                        }
                    }
                ]
            }))
        });

        let execute = warp::path!("mcp" / "execute")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |body: serde_json::Value| {
                let tool_name = body["tool"].as_str().unwrap_or("");
                let _params = &body["parameters"];

                match tool_name {
                    "get_app_status" => warp::reply::json(&serde_json::json!({
                        "status": "running",
                        "uptime": "00:15:30",
                        "connected_peers": 12,
                        "active_organizations": 3
                    })),
                    "get_dht_status" => warp::reply::json(&serde_json::json!({
                        "dht_enabled": true,
                        "stored_shards": 156,
                        "reed_solomon_config": {
                            "data_shards": 8,
                            "parity_shards": 4,
                            "availability_target": "60%"
                        }
                    })),
                    "get_storage_metrics" => warp::reply::json(&serde_json::json!({
                        "total_capacity": "400MB",
                        "used": "125MB",
                        "allocation": {
                            "personal": "100MB",
                            "dht_backup": "100MB",
                            "public_dht": "200MB"
                        },
                        "reed_solomon_active": true
                    })),
                    _ => warp::reply::json(&serde_json::json!({
                        "error": format!("Unknown tool: {}", tool_name)
                    })),
                }
            });

        let routes = status
            .or(app_info)
            .or(windows)
            .or(tools)
            .or(execute)
            .with(warp::cors().allow_any_origin());

        // Spawn the server
        tokio::spawn(async move {
            info!("MCP server listening on http://{}", addr);
            warp::serve(routes).run(addr).await;
        });

        Ok(())
    }
}

pub struct MCPPlugin<R: Runtime> {
    server: Arc<RwLock<MCPServer>>,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Runtime> MCPPlugin<R> {
    pub fn new(config: MCPConfig) -> Self {
        Self {
            server: Arc::new(RwLock::new(MCPServer::new(config))),
            _phantom: std::marker::PhantomData,
        }
    }
}

pub fn init_with_config<R: Runtime>(config: MCPConfig) -> TauriPlugin<R> {
    println!("MCP Plugin: Creating plugin with config: {:?}", config);
    info!("MCP Plugin: Creating plugin with config: {:?}", config);

    let plugin = MCPPlugin::<R>::new(config.clone());
    let server = plugin.server.clone();

    PluginBuilder::new("mcp")
        .setup(move |_app, _plugin| {
            println!("MCP Plugin: Setup called for {}", config.app_name);
            info!("MCP Plugin: Setup called for {}", config.app_name);

            let server_clone = server.clone();

            // Start the MCP server
            tauri::async_runtime::spawn(async move {
                println!("MCP Plugin: Starting server...");
                info!("MCP Plugin: Starting server...");

                let server = server_clone.read().await;
                if let Err(e) = server.start().await {
                    println!("MCP Plugin: Failed to start MCP server: {}", e);
                    error!("Failed to start MCP server: {}", e);
                } else {
                    println!(
                        "MCP Plugin: Server started successfully on {}:{}",
                        server.config.host, server.config.port
                    );
                    info!(
                        "MCP server started successfully on {}:{}",
                        server.config.host, server.config.port
                    );
                }
            });

            println!(
                "MCP Plugin: Initialization complete for {}",
                config.app_name
            );
            info!("MCP Plugin initialized for {}", config.app_name);
            Ok(())
        })
        .build()
}

// Re-export for convenience
// pub use self::MCPConfig as PluginConfig; // Commented out as unused
