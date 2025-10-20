// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Port Manager for Dynamic UDP Port Allocation
//!
//! Manages random high UDP port allocation (49152-65535) for QUIC transport.
//! Follows IANA ephemeral port recommendations and checks availability before binding.

use anyhow::Result;
use rand::Rng;
use std::net::{SocketAddr, UdpSocket};
use tracing::{debug, info, warn};

/// Port range for dynamic allocation (IANA ephemeral ports)
const PORT_RANGE_START: u16 = 49152;
const PORT_RANGE_END: u16 = 65535;

/// Maximum retries for finding an available port
const MAX_PORT_RETRIES: usize = 10;

/// Port Manager for dynamic port allocation
#[derive(Debug, Clone)]
pub struct PortManager {
    /// Preferred port (if previously used successfully)
    preferred_port: Option<u16>,
}

impl PortManager {
    /// Create a new PortManager
    pub fn new() -> Self {
        Self {
            preferred_port: None,
        }
    }

    /// Create a PortManager with a preferred port
    pub fn with_preferred_port(port: u16) -> Self {
        Self {
            preferred_port: Some(port),
        }
    }

    /// Allocate a random high UDP port
    ///
    /// Strategy:
    /// 1. Try preferred port if available
    /// 2. Generate random ports in ephemeral range (49152-65535)
    /// 3. Test if port is available by attempting to bind
    /// 4. Retry up to MAX_PORT_RETRIES times
    ///
    /// # Returns
    /// An available UDP port number
    pub fn allocate_port(&mut self) -> Result<u16> {
        // Try preferred port first
        if let Some(port) = self.preferred_port {
            debug!("Trying preferred port: {}", port);
            if self.is_port_available(port) {
                info!("✅ Using preferred port: {}", port);
                return Ok(port);
            }
            warn!("Preferred port {} not available", port);
        }

        // Generate random port in ephemeral range
        let mut rng = rand::thread_rng();
        for attempt in 1..=MAX_PORT_RETRIES {
            let port = rng.gen_range(PORT_RANGE_START..=PORT_RANGE_END);
            debug!("Attempt {}: Trying random port {}", attempt, port);

            if self.is_port_available(port) {
                info!("✅ Allocated random port: {}", port);
                self.preferred_port = Some(port); // Remember for next time
                return Ok(port);
            }
        }

        Err(anyhow::anyhow!(
            "Failed to allocate port after {} attempts",
            MAX_PORT_RETRIES
        ))
    }

    /// Check if a port is available by attempting to bind
    ///
    /// # Arguments
    /// * `port` - Port number to test
    ///
    /// # Returns
    /// true if port can be bound, false otherwise
    fn is_port_available(&self, port: u16) -> bool {
        // Try binding to IPv4 and IPv6
        // Construct SocketAddr directly to avoid parsing and potential panics
        let ipv4_addr =
            SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port);
        let ipv6_addr =
            SocketAddr::new(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), port);

        // Try IPv4 first
        match UdpSocket::bind(ipv4_addr) {
            Ok(_socket) => {
                debug!("Port {} available on IPv4", port);
                // Socket automatically closes when dropped
                true
            }
            Err(e) => {
                debug!("Port {} unavailable on IPv4: {}", port, e);
                // Try IPv6 as fallback
                match UdpSocket::bind(ipv6_addr) {
                    Ok(_socket) => {
                        debug!("Port {} available on IPv6", port);
                        true
                    }
                    Err(e2) => {
                        debug!("Port {} unavailable on IPv6: {}", port, e2);
                        false
                    }
                }
            }
        }
    }

    /// Get the preferred port if set
    pub fn get_preferred_port(&self) -> Option<u16> {
        self.preferred_port
    }

    /// Set a new preferred port
    pub fn set_preferred_port(&mut self, port: u16) {
        self.preferred_port = Some(port);
    }

    /// Clear the preferred port
    pub fn clear_preferred_port(&mut self) {
        self.preferred_port = None;
    }
}

impl Default for PortManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_manager_new() {
        let pm = PortManager::new();
        assert!(pm.get_preferred_port().is_none());
    }

    #[test]
    fn test_port_manager_with_preferred() {
        let pm = PortManager::with_preferred_port(50000);
        assert_eq!(pm.get_preferred_port(), Some(50000));
    }

    #[test]
    fn test_allocate_port_success() {
        let mut pm = PortManager::new();
        let port = pm.allocate_port().expect("should allocate port");

        // Verify port is in ephemeral range
        assert!((PORT_RANGE_START..=PORT_RANGE_END).contains(&port));

        // Port should now be remembered as preferred
        assert_eq!(pm.get_preferred_port(), Some(port));
    }

    #[test]
    fn test_allocate_port_preferred() {
        // Use a likely available port
        let mut pm = PortManager::with_preferred_port(63000);

        let port = pm.allocate_port().expect("should allocate port");

        // Should get preferred port if available
        // (May fail if port is in use, but test demonstrates the logic)
        assert!((PORT_RANGE_START..=PORT_RANGE_END).contains(&port));
    }

    #[test]
    fn test_is_port_available_invalid() {
        let pm = PortManager::new();

        // Port 80 is likely privileged/unavailable
        // This test may vary by system
        let _available = pm.is_port_available(80);
        // Just verify the method runs without panic
    }

    #[test]
    fn test_set_clear_preferred() {
        let mut pm = PortManager::new();

        pm.set_preferred_port(55555);
        assert_eq!(pm.get_preferred_port(), Some(55555));

        pm.clear_preferred_port();
        assert!(pm.get_preferred_port().is_none());
    }

    #[test]
    fn test_default() {
        let pm = PortManager::default();
        assert!(pm.get_preferred_port().is_none());
    }

    #[test]
    fn test_multiple_allocations() {
        let mut pm1 = PortManager::new();
        let mut pm2 = PortManager::new();

        let port1 = pm1.allocate_port().expect("pm1 should allocate");
        let port2 = pm2.allocate_port().expect("pm2 should allocate");

        // Ports should be in valid range
        assert!((PORT_RANGE_START..=PORT_RANGE_END).contains(&port1));
        assert!((PORT_RANGE_START..=PORT_RANGE_END).contains(&port2));
    }
}
