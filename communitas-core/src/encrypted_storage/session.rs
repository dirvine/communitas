//! Session Management for Multi-Account Support
//!
//! Handles authenticated sessions with automatic expiration,
//! allowing seamless switching between multiple accounts.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents an authenticated session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub expires_at: u64,
    pub auth_method: AuthMethod,
}

/// Authentication method used for the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password,
    PasswordOnly,  // Familiar device login
    Passkey,
    Combined,      // Password + Passkey
}

impl Session {
    /// Create a new session
    pub fn new(four_words: String, display_name: String, timeout_seconds: u64) -> Self {
        let now = current_timestamp();
        Self {
            id: generate_session_id(),
            four_words,
            display_name,
            created_at: now,
            last_activity: now,
            expires_at: now + timeout_seconds,
            auth_method: AuthMethod::Password,
        }
    }

    /// Create a password-only session (familiar device)
    pub fn new_password_only(
        four_words: String,
        display_name: String,
        timeout_seconds: u64,
    ) -> Self {
        let mut session = Self::new(four_words, display_name, timeout_seconds);
        session.auth_method = AuthMethod::PasswordOnly;
        session
    }

    /// Check if the session has expired
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Update the last activity time
    pub fn touch(&mut self) {
        self.last_activity = current_timestamp();
    }

    /// Extend the session expiration
    pub fn extend(&mut self, additional_seconds: u64) {
        self.expires_at = current_timestamp() + additional_seconds;
        self.touch();
    }

    /// Get remaining time until expiration
    pub fn time_remaining(&self) -> Duration {
        let now = current_timestamp();
        if now >= self.expires_at {
            Duration::from_secs(0)
        } else {
            Duration::from_secs(self.expires_at - now)
        }
    }
}

/// Session manager for handling multiple active sessions
pub struct SessionManager {
    sessions: std::sync::Arc<tokio::sync::RwLock<Vec<Session>>>,
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            max_sessions,
        }
    }

    /// Add a new session
    pub async fn add_session(&self, session: Session) -> Result<String, anyhow::Error> {
        let mut sessions = self.sessions.write().await;

        // Check for existing session with same four_words
        if let Some(existing) = sessions
            .iter_mut()
            .find(|s| s.four_words == session.four_words)
        {
            // Update existing session
            existing.extend(session.expires_at - session.created_at);
            return Ok(existing.id.clone());
        }

        // Enforce maximum sessions limit
        if sessions.len() >= self.max_sessions {
            // Remove oldest expired session or oldest by creation time
            sessions.sort_by_key(|s| s.created_at);
            sessions.remove(0);
        }

        let session_id = session.id.clone();
        sessions.push(session);

        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .find(|s| s.id == session_id && !s.is_expired())
            .cloned()
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .filter(|s| !s.is_expired())
            .cloned()
            .collect()
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(pos) = sessions.iter().position(|s| s.id == session_id) {
            sessions.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let original_len = sessions.len();
        sessions.retain(|s| !s.is_expired());
        original_len - sessions.len()
    }

    /// Switch to a different account session
    pub async fn switch_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
            if !session.is_expired() {
                session.touch();
                return Some(session.clone());
            }
        }
        None
    }
}

/// Session storage for persistence across app restarts
pub struct SessionStorage {
    storage_path: std::path::PathBuf,
}

impl SessionStorage {
    pub fn new(base_path: &std::path::Path) -> Self {
        let storage_path = base_path.join("sessions.enc");
        Self { storage_path }
    }

    /// Save sessions to encrypted storage
    pub async fn save_sessions(&self, sessions: &[Session]) -> anyhow::Result<()> {
        // Filter out expired sessions
        let active_sessions: Vec<_> = sessions
            .iter()
            .filter(|s| !s.is_expired())
            .cloned()
            .collect();

        let json = serde_json::to_vec(&active_sessions)?;

        // In production, this should be encrypted with a device key
        tokio::fs::write(&self.storage_path, json).await?;

        Ok(())
    }

    /// Load sessions from encrypted storage
    pub async fn load_sessions(&self) -> anyhow::Result<Vec<Session>> {
        if !self.storage_path.exists() {
            return Ok(Vec::new());
        }

        let data = tokio::fs::read(&self.storage_path).await?;

        // In production, decrypt with device key
        let sessions: Vec<Session> = serde_json::from_slice(&data)?;

        // Filter out expired sessions
        Ok(sessions.into_iter().filter(|s| !s.is_expired()).collect())
    }

    /// Clear all stored sessions
    pub async fn clear(&self) -> anyhow::Result<()> {
        if self.storage_path.exists() {
            tokio::fs::remove_file(&self.storage_path).await?;
        }
        Ok(())
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn generate_session_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..16).map(|_| rng.r#gen::<u8>()).collect();
    hex::encode(random_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "test-four-words".to_string(),
            "Test User".to_string(),
            300, // 5 minutes
        );

        assert!(!session.is_expired());
        assert_eq!(session.four_words, "test-four-words");
        assert_eq!(session.display_name, "Test User");
    }

    #[test]
    fn test_session_expiration() {
        let mut session = Session::new(
            "test-four-words".to_string(),
            "Test User".to_string(),
            1, // 1 second
        );

        assert!(!session.is_expired());

        // Manually set expiration to past
        session.expires_at = current_timestamp() - 10;
        assert!(session.is_expired());
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new(3);

        // Add sessions
        let session1 = Session::new("user1".to_string(), "User 1".to_string(), 300);
        let session2 = Session::new("user2".to_string(), "User 2".to_string(), 300);
        let session3 = Session::new("user3".to_string(), "User 3".to_string(), 300);

        let id1 = manager.add_session(session1).await.unwrap();
        let id2 = manager.add_session(session2).await.unwrap();
        let id3 = manager.add_session(session3).await.unwrap();

        // Get active sessions
        let active = manager.get_active_sessions().await;
        assert_eq!(active.len(), 3);

        // Switch session
        let switched = manager.switch_session(&id2).await;
        assert!(switched.is_some());
        assert_eq!(switched.unwrap().four_words, "user2");

        // Remove session
        assert!(manager.remove_session(&id1).await);
        let active_after = manager.get_active_sessions().await;
        assert_eq!(active_after.len(), 2);
    }

    #[tokio::test]
    async fn test_session_storage() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path());

        let sessions = vec![
            Session::new("user1".to_string(), "User 1".to_string(), 300),
            Session::new("user2".to_string(), "User 2".to_string(), 300),
        ];

        // Save sessions
        storage.save_sessions(&sessions).await.unwrap();

        // Load sessions
        let loaded = storage.load_sessions().await.unwrap();
        assert_eq!(loaded.len(), 2);

        // Clear sessions
        storage.clear().await.unwrap();
        let after_clear = storage.load_sessions().await.unwrap();
        assert_eq!(after_clear.len(), 0);
    }
}