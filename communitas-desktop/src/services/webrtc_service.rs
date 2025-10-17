use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Call session state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CallState {
    Idle,
    Initiating,
    Ringing,
    Connecting,
    Connected,
    Disconnecting,
    Ended,
    Failed(String),
}

/// Call type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CallType {
    /// One-to-one audio call
    AudioPeer,
    /// One-to-one video call
    VideoPeer,
    /// Group audio call
    AudioGroup,
    /// Group video call
    VideoGroup,
}

/// Call session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSession {
    pub id: String,
    pub call_type: CallType,
    pub initiator_id: String,
    pub participants: Vec<String>, // Member or Group IDs
    pub state: CallState,
    pub created_at: i64,
    pub connected_at: Option<i64>,
    pub ended_at: Option<i64>,
}

/// WebRTC signaling message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalingMessage {
    /// SDP offer from initiator
    Offer {
        session_id: String,
        sdp: String,
        from: String,
    },
    /// SDP answer from receiver
    Answer {
        session_id: String,
        sdp: String,
        from: String,
    },
    /// ICE candidate exchange
    IceCandidate {
        session_id: String,
        candidate: String,
        from: String,
    },
    /// Call termination
    Hangup {
        session_id: String,
        from: String,
    },
}

/// WebRTC service for managing real-time communication
pub struct WebRtcService {
    crdt: Arc<CrdtManager>,
    /// Active call sessions
    active_sessions: Arc<RwLock<HashMap<String, CallSession>>>,
}

impl WebRtcService {
    /// Create a new WebRtcService
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self {
            crdt,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize a peer-to-peer call
    pub async fn initiate_peer_call(
        &self,
        initiator_id: &str,
        peer_id: &str,
        call_type: CallType,
    ) -> Result<CallSession> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        // Validate call type is peer-to-peer
        match call_type {
            CallType::AudioPeer | CallType::VideoPeer => {}
            _ => anyhow::bail!("Call type must be AudioPeer or VideoPeer for peer calls"),
        }

        let session = CallSession {
            id: session_id.clone(),
            call_type: call_type.clone(),
            initiator_id: initiator_id.to_string(),
            participants: vec![initiator_id.to_string(), peer_id.to_string()],
            state: CallState::Initiating,
            created_at: now,
            connected_at: None,
            ended_at: None,
        };

        // Store session in active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }

        // Persist to database
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO call_sessions (id, call_type, initiator_id, participants, state, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                session_id,
                serde_json::to_string(&call_type)?,
                initiator_id,
                serde_json::to_string(&session.participants)?,
                serde_json::to_string(&session.state)?,
                now
            ],
        )
        .await
        .context("Failed to create call session")?;

        Ok(session)
    }

    /// Initialize a group call
    pub async fn initiate_group_call(
        &self,
        initiator_id: &str,
        group_id: &str,
        call_type: CallType,
        participant_ids: Vec<String>,
    ) -> Result<CallSession> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        // Validate call type is group call
        match call_type {
            CallType::AudioGroup | CallType::VideoGroup => {}
            _ => anyhow::bail!("Call type must be AudioGroup or VideoGroup for group calls"),
        }

        // Include initiator in participants
        let mut participants = participant_ids;
        if !participants.contains(&initiator_id.to_string()) {
            participants.insert(0, initiator_id.to_string());
        }

        let session = CallSession {
            id: session_id.clone(),
            call_type: call_type.clone(),
            initiator_id: initiator_id.to_string(),
            participants: participants.clone(),
            state: CallState::Initiating,
            created_at: now,
            connected_at: None,
            ended_at: None,
        };

        // Store session in active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }

        // Persist to database
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO call_sessions (id, call_type, initiator_id, participants, state, created_at, group_id)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                session_id,
                serde_json::to_string(&call_type)?,
                initiator_id,
                serde_json::to_string(&participants)?,
                serde_json::to_string(&session.state)?,
                now,
                group_id
            ],
        )
        .await
        .context("Failed to create group call session")?;

        Ok(session)
    }

    /// Accept an incoming call
    pub async fn accept_call(&self, session_id: &str) -> Result<()> {
        // Update session state
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = CallState::Connecting;

            // Update database
            let db = self.crdt.connection()?;
            db.execute(
                "UPDATE call_sessions SET state = ? WHERE id = ?",
                params![serde_json::to_string(&session.state)?, session_id],
            )
            .await
            .context("Failed to update call state")?;

            Ok(())
        } else {
            anyhow::bail!("Call session not found: {}", session_id)
        }
    }

    /// Mark call as connected
    pub async fn mark_connected(&self, session_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        let mut sessions = self.active_sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.state = CallState::Connected;
            session.connected_at = Some(now);

            // Update database
            let db = self.crdt.connection()?;
            db.execute(
                "UPDATE call_sessions SET state = ?, connected_at = ? WHERE id = ?",
                params![serde_json::to_string(&session.state)?, now, session_id],
            )
            .await
            .context("Failed to update call state")?;

            Ok(())
        } else {
            anyhow::bail!("Call session not found: {}", session_id)
        }
    }

    /// End a call
    pub async fn end_call(&self, session_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        let mut sessions = self.active_sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.state = CallState::Ended;
            session.ended_at = Some(now);

            // Update database
            let db = self.crdt.connection()?;
            db.execute(
                "UPDATE call_sessions SET state = ?, ended_at = ? WHERE id = ?",
                params![serde_json::to_string(&session.state)?, now, session_id],
            )
            .await
            .context("Failed to update call state")?;

            // Remove from active sessions
            sessions.remove(session_id);

            Ok(())
        } else {
            anyhow::bail!("Call session not found: {}", session_id)
        }
    }

    /// Reject an incoming call
    pub async fn reject_call(&self, session_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        let mut sessions = self.active_sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            session.state = CallState::Ended;
            session.ended_at = Some(now);

            // Update database
            let db = self.crdt.connection()?;
            db.execute(
                "UPDATE call_sessions SET state = ?, ended_at = ? WHERE id = ?",
                params![serde_json::to_string(&session.state)?, now, session_id],
            )
            .await
            .context("Failed to update call state")?;

            // Remove from active sessions
            sessions.remove(session_id);

            Ok(())
        } else {
            anyhow::bail!("Call session not found: {}", session_id)
        }
    }

    /// Get call session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<CallSession>> {
        let sessions = self.active_sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// Get all active calls for a member
    pub async fn get_member_active_calls(&self, member_id: &str) -> Result<Vec<CallSession>> {
        let sessions = self.active_sessions.read().await;
        let member_calls: Vec<CallSession> = sessions
            .values()
            .filter(|session| {
                session.participants.contains(&member_id.to_string())
                    && matches!(session.state, CallState::Initiating | CallState::Ringing | CallState::Connecting | CallState::Connected)
            })
            .cloned()
            .collect();
        Ok(member_calls)
    }

    /// Get all active calls for a group
    pub async fn get_group_active_calls(&self, group_id: &str) -> Result<Vec<CallSession>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, call_type, initiator_id, participants, state, created_at, connected_at, ended_at
                 FROM call_sessions
                 WHERE group_id = ? AND state IN (?, ?, ?, ?)
                 ORDER BY created_at DESC",
                params![
                    group_id,
                    serde_json::to_string(&CallState::Initiating)?,
                    serde_json::to_string(&CallState::Ringing)?,
                    serde_json::to_string(&CallState::Connecting)?,
                    serde_json::to_string(&CallState::Connected)?
                ],
            )
            .await?;

        let mut calls = Vec::new();
        while let Some(row) = rows.next().await? {
            calls.push(CallSession {
                id: row.get(0)?,
                call_type: serde_json::from_str(&row.get::<String>(1)?)?,
                initiator_id: row.get(2)?,
                participants: serde_json::from_str(&row.get::<String>(3)?)?,
                state: serde_json::from_str(&row.get::<String>(4)?)?,
                created_at: row.get(5)?,
                connected_at: row.get(6)?,
                ended_at: row.get(7)?,
            });
        }

        Ok(calls)
    }

    /// Process signaling message (SDP offer/answer, ICE candidates)
    pub async fn handle_signaling(&self, message: SignalingMessage) -> Result<()> {
        match message {
            SignalingMessage::Offer { session_id, sdp: _, from: _ } => {
                // Update session state to ringing
                let mut sessions = self.active_sessions.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.state = CallState::Ringing;

                    let db = self.crdt.connection()?;
                    db.execute(
                        "UPDATE call_sessions SET state = ? WHERE id = ?",
                        params![serde_json::to_string(&session.state)?, session_id],
                    )
                    .await
                    .context("Failed to update call state")?;
                }
                Ok(())
            }
            SignalingMessage::Answer { session_id, sdp: _, from: _ } => {
                // Update session state to connecting
                let mut sessions = self.active_sessions.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.state = CallState::Connecting;

                    let db = self.crdt.connection()?;
                    db.execute(
                        "UPDATE call_sessions SET state = ? WHERE id = ?",
                        params![serde_json::to_string(&session.state)?, session_id],
                    )
                    .await
                    .context("Failed to update call state")?;
                }
                Ok(())
            }
            SignalingMessage::IceCandidate { session_id: _, candidate: _, from: _ } => {
                // ICE candidates are handled by WebRTC implementation
                // This is just for logging/tracking
                Ok(())
            }
            SignalingMessage::Hangup { session_id, from: _ } => {
                self.end_call(&session_id).await
            }
        }
    }

    /// Get call history for a member
    pub async fn get_call_history(&self, member_id: &str, limit: i64) -> Result<Vec<CallSession>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, call_type, initiator_id, participants, state, created_at, connected_at, ended_at
                 FROM call_sessions
                 WHERE participants LIKE ?
                 ORDER BY created_at DESC
                 LIMIT ?",
                params![format!("%{}%", member_id), limit],
            )
            .await?;

        let mut calls = Vec::new();
        while let Some(row) = rows.next().await? {
            calls.push(CallSession {
                id: row.get(0)?,
                call_type: serde_json::from_str(&row.get::<String>(1)?)?,
                initiator_id: row.get(2)?,
                participants: serde_json::from_str(&row.get::<String>(3)?)?,
                state: serde_json::from_str(&row.get::<String>(4)?)?,
                created_at: row.get(5)?,
                connected_at: row.get(6)?,
                ended_at: row.get(7)?,
            });
        }

        Ok(calls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt_manager::CrdtManager;
    use tempfile::tempdir;

    async fn create_test_service() -> Result<WebRtcService> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let crdt = Arc::new(CrdtManager::new(db_path.to_str().unwrap()).await?);

        // Create call_sessions table
        let db = crdt.connection()?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS call_sessions (
                id TEXT PRIMARY KEY,
                call_type TEXT NOT NULL,
                initiator_id TEXT NOT NULL,
                participants TEXT NOT NULL,
                state TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                connected_at INTEGER,
                ended_at INTEGER,
                group_id TEXT
            )",
            (),
        )
        .await?;

        Ok(WebRtcService::new(crdt))
    }

    #[tokio::test]
    async fn test_initiate_peer_call() {
        let service = create_test_service().await.unwrap();
        let session = service
            .initiate_peer_call("alice", "bob", CallType::AudioPeer)
            .await
            .unwrap();

        assert_eq!(session.call_type, CallType::AudioPeer);
        assert_eq!(session.initiator_id, "alice");
        assert_eq!(session.participants.len(), 2);
        assert!(session.participants.contains(&"alice".to_string()));
        assert!(session.participants.contains(&"bob".to_string()));
        assert_eq!(session.state, CallState::Initiating);
    }

    #[tokio::test]
    async fn test_initiate_group_call() {
        let service = create_test_service().await.unwrap();
        let participants = vec!["bob".to_string(), "charlie".to_string()];
        let session = service
            .initiate_group_call("alice", "group-1", CallType::AudioGroup, participants)
            .await
            .unwrap();

        assert_eq!(session.call_type, CallType::AudioGroup);
        assert_eq!(session.initiator_id, "alice");
        assert_eq!(session.participants.len(), 3);
        assert!(session.participants.contains(&"alice".to_string()));
        assert_eq!(session.state, CallState::Initiating);
    }

    #[tokio::test]
    async fn test_accept_call() {
        let service = create_test_service().await.unwrap();
        let session = service
            .initiate_peer_call("alice", "bob", CallType::VideoPeer)
            .await
            .unwrap();

        service.accept_call(&session.id).await.unwrap();

        let updated = service.get_session(&session.id).await.unwrap().unwrap();
        assert_eq!(updated.state, CallState::Connecting);
    }

    #[tokio::test]
    async fn test_end_call() {
        let service = create_test_service().await.unwrap();
        let session = service
            .initiate_peer_call("alice", "bob", CallType::AudioPeer)
            .await
            .unwrap();

        service.end_call(&session.id).await.unwrap();

        let ended = service.get_session(&session.id).await.unwrap();
        assert!(ended.is_none()); // Should be removed from active sessions
    }

    #[tokio::test]
    async fn test_get_member_active_calls() {
        let service = create_test_service().await.unwrap();

        // Alice initiates calls with Bob and Charlie
        service
            .initiate_peer_call("alice", "bob", CallType::AudioPeer)
            .await
            .unwrap();
        service
            .initiate_peer_call("alice", "charlie", CallType::VideoPeer)
            .await
            .unwrap();

        let alice_calls = service.get_member_active_calls("alice").await.unwrap();
        assert_eq!(alice_calls.len(), 2);

        let bob_calls = service.get_member_active_calls("bob").await.unwrap();
        assert_eq!(bob_calls.len(), 1);
    }
}
