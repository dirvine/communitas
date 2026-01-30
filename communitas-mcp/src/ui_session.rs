// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! UI session token management for MCP Apps widgets.

use rand::RngCore;
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

const DEFAULT_UI_SESSION_TTL: Duration = Duration::from_secs(10 * 60);

/// Issued UI session token with expiry.
#[derive(Debug, Clone)]
pub(crate) struct UiSession {
    pub(crate) token: String,
    pub(crate) expires_at: SystemTime,
}

impl UiSession {
    pub(crate) fn expires_in(&self, now: SystemTime) -> u64 {
        match self.expires_at.duration_since(now) {
            Ok(duration) => duration.as_secs(),
            Err(_) => 0,
        }
    }
}

/// In-memory UI session token store.
#[derive(Debug, Default)]
pub(crate) struct UiSessionStore {
    ttl: Duration,
    sessions: HashMap<String, SystemTime>,
}

impl UiSessionStore {
    pub(crate) fn with_default_ttl() -> Self {
        Self::new(DEFAULT_UI_SESSION_TTL)
    }

    pub(crate) fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            sessions: HashMap::new(),
        }
    }

    pub(crate) fn issue(&mut self) -> UiSession {
        self.purge_expired();
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        let now = SystemTime::now();
        let expires_at = match now.checked_add(self.ttl) {
            Some(value) => value,
            None => now,
        };
        self.sessions.insert(token.clone(), expires_at);
        UiSession { token, expires_at }
    }

    pub(crate) fn validate(&mut self, token: &str) -> bool {
        self.purge_expired();
        self.sessions.contains_key(token)
    }

    pub(crate) fn purge_expired(&mut self) {
        let now = SystemTime::now();
        self.sessions
            .retain(|_, expires_at| expires_at.duration_since(now).is_ok());
    }
}
