use super::types::*;
use crate::backend::Backend;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared application state
pub type AppState = Arc<Mutex<Backend>>;

/// Health check endpoint
pub async fn health() -> impl IntoResponse {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    Json(response)
}

/// Get current identity
pub async fn get_identity(State(backend): State<AppState>) -> impl IntoResponse {
    let backend = backend.lock().await;

    if let Some(session) = backend.get_current_session() {
        let response = IdentityResponse {
            four_words: session.four_words,
            display_name: session.display_name,
            is_logged_in: true,
        };
        (StatusCode::OK, Json(response))
    } else {
        let response = IdentityResponse {
            four_words: String::new(),
            display_name: String::new(),
            is_logged_in: false,
        };
        (StatusCode::OK, Json(response))
    }
}

/// Get network status
pub async fn get_network_status(State(backend): State<AppState>) -> impl IntoResponse {
    let backend = backend.lock().await;

    let connected = backend.check_dht_connection().await.unwrap_or(false);
    let response = NetworkStatusResponse {
        connected,
        offline: !connected,
    };

    Json(response)
}

/// Send message to entity
pub async fn send_message(
    State(backend): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut backend = backend.lock().await;

    match backend
        .send_message(req.entity_id, req.entity_type, req.text)
        .await
    {
        Ok(message_id) => {
            let response = SendMessageResponse { message_id };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to send message: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)))
        }
    }
}

/// Create entity (contact, group, channel, etc.)
pub async fn create_entity(
    State(backend): State<AppState>,
    Json(req): Json<CreateEntityRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut backend = backend.lock().await;

    match backend
        .create_entity(req.name, req.entity_type, req.members)
        .await
    {
        Ok(entity) => {
            let response = EntityResponse {
                id: entity.id,
                name: entity.name,
                entity_type: entity.entity_type,
                members: entity.members,
            };
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to create entity: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)))
        }
    }
}

/// List all entities
pub async fn list_entities(
    State(backend): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let backend = backend.lock().await;

    match backend.get_entities().await {
        Ok(entities) => {
            let response: Vec<EntityResponse> = entities
                .into_iter()
                .map(|e| EntityResponse {
                    id: e.id,
                    name: e.name,
                    entity_type: e.entity_type,
                    members: e.members,
                })
                .collect();
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to list entities: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)))
        }
    }
}

/// Get messages for entity
pub async fn get_entity_messages(
    State(backend): State<AppState>,
    axum::extract::Path(entity_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut backend = backend.lock().await;

    match backend.get_entity_messages(entity_id).await {
        Ok(messages) => {
            let response: Vec<MessageResponse> = messages
                .into_iter()
                .map(|m| MessageResponse {
                    id: m.metadata.id,
                    author: m.content.author,
                    text: m.content.text,
                    timestamp: m.metadata.timestamp,
                    reply_to_id: m.metadata.reply_to_id,
                })
                .collect();
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("Failed to get messages: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)))
        }
    }
}

// ========================================================================
// Authentication Endpoints
// ========================================================================

/// Create new vault and login
pub async fn create_vault(
    State(backend): State<AppState>,
    Json(req): Json<CreateVaultRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut backend = backend.lock().await;

    // Generate four-word identity if not provided
    let four_words = req.four_words.unwrap_or_else(Backend::generate_four_words);

    tracing::info!("Creating vault for: {}", four_words);

    match backend
        .create_vault(&four_words, &req.password, &req.display_name)
        .await
    {
        Ok(session_info) => {
            // Initialize CoreContext after successful vault creation
            if let Err(e) = backend.initialize_core_context().await {
                tracing::error!("Failed to initialize CoreContext: {}", e);
                let error = ErrorResponse {
                    error: format!("Vault created but CoreContext initialization failed: {}", e),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)));
            }

            let response = CreateVaultResponse {
                four_words: session_info.four_words,
                display_name: session_info.display_name,
                session_token: None, // Add session token support later if needed
            };
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => {
            tracing::error!("Vault creation failed: {}", e);
            let error = ErrorResponse {
                error: format!("Failed to create vault: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)))
        }
    }
}

/// Login with existing vault
pub async fn login(
    State(backend): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut backend = backend.lock().await;

    tracing::info!("Login attempt for: {}", req.four_words);

    match backend.login(&req.four_words, &req.password).await {
        Ok(session_info) => {
            // Initialize CoreContext after successful login
            if let Err(e) = backend.initialize_core_context().await {
                tracing::error!("Failed to initialize CoreContext: {}", e);
                let error = ErrorResponse {
                    error: format!(
                        "Login successful but CoreContext initialization failed: {}",
                        e
                    ),
                };
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)));
            }

            let response = LoginResponse {
                four_words: session_info.four_words,
                display_name: session_info.display_name,
                session_token: None, // Add session token support later if needed
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            tracing::error!("Login failed for {}: {}", req.four_words, e);
            let error = ErrorResponse {
                error: format!("Login failed: {}", e),
            };
            Err((StatusCode::UNAUTHORIZED, Json(error)))
        }
    }
}

/// Logout current session
pub async fn logout(
    State(backend): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut backend = backend.lock().await;

    tracing::info!("Logout requested");

    match backend.logout().await {
        Ok(()) => {
            let response = LogoutResponse {
                success: true,
                message: "Logged out successfully".to_string(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            tracing::error!("Logout failed: {}", e);
            let error = ErrorResponse {
                error: format!("Logout failed: {}", e),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)))
        }
    }
}
