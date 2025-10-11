use communitas_core::crdt::EntityType;
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Current identity response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub four_words: String,
    pub display_name: String,
    pub is_logged_in: bool,
}

/// Network status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatusResponse {
    pub connected: bool,
    pub offline: bool,
}

/// Send message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub text: String,
}

/// Send message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
}

/// Create entity request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityRequest {
    pub name: String,
    pub entity_type: EntityType,
    pub members: Vec<String>, // Four-word addresses
}

/// Entity response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub members: Vec<String>,
}

/// Message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub author: String,
    pub text: String,
    pub timestamp: u64,
    pub reply_to_id: Option<String>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ========================================================================
// Authentication Types
// ========================================================================

/// Create vault request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVaultRequest {
    pub four_words: Option<String>, // If None, generate new identity
    pub password: String,
    pub display_name: String,
}

/// Create vault response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVaultResponse {
    pub four_words: String,
    pub display_name: String,
    pub session_token: Option<String>,
}

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub four_words: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub four_words: String,
    pub display_name: String,
    pub session_token: Option<String>,
}

/// Logout response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}
