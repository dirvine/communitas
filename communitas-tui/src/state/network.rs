#![allow(dead_code)]

use std::time::SystemTime;

/// Network connection state
#[derive(Debug, Clone)]
pub struct NetworkState {
    /// Connection status
    pub status: ConnectionStatus,
    /// Last update timestamp
    pub last_update: SystemTime,
    /// Number of connected peers
    pub peer_count: usize,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            status: ConnectionStatus::Disconnected,
            last_update: SystemTime::now(),
            peer_count: 0,
            bootstrap_nodes: Vec::new(),
        }
    }

    /// Update connection status
    pub fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;
        self.last_update = SystemTime::now();
    }

    /// Update peer count
    pub fn set_peer_count(&mut self, count: usize) {
        self.peer_count = count;
        self.last_update = SystemTime::now();
    }

    /// Get status indicator symbol
    pub fn status_symbol(&self) -> &'static str {
        match self.status {
            ConnectionStatus::Connected => "●",
            ConnectionStatus::Connecting => "◐",
            ConnectionStatus::Disconnected => "○",
            ConnectionStatus::Error(_) => "✗",
        }
    }

    /// Get status color name
    pub fn status_color(&self) -> &'static str {
        match self.status {
            ConnectionStatus::Connected => "green",
            ConnectionStatus::Connecting => "yellow",
            ConnectionStatus::Disconnected => "gray",
            ConnectionStatus::Error(_) => "red",
        }
    }
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
}
