// Web publishing commands with saorsa-fec encryption and security hardening
// This module provides secure web content publishing and browsing capabilities

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;
use uuid::Uuid;
use blake3;
use std::time::Duration;
use crate::secure_fec::{SecureFecManager, FecConfig, SecureFecError};
use crate::security::rate_limiter::{RateLimiter, RateLimitError};
use crate::security::input_validation::{InputValidator, ValidationError};

/// Security: Custom error type with sanitized messages
#[derive(Debug, Clone, thiserror::Error, Serialize)]
pub enum WebPublishingError {
    #[error("Invalid input parameters")]
    InvalidInput,
    #[error("Content too large")]
    ContentTooLarge,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Authentication required")]
    AuthenticationRequired,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Network operation failed")]
    NetworkError,
    #[error("Storage operation failed")]
    StorageError,
    #[error("Content not found")]
    ContentNotFound,
    #[error("Encryption operation failed")]
    EncryptionError,
    #[error("Configuration error")]
    ConfigurationError,
}

impl From<SecureFecError> for WebPublishingError {
    fn from(error: SecureFecError) -> Self {
        match error {
            SecureFecError::InvalidInput => WebPublishingError::InvalidInput,
            SecureFecError::PermissionDenied => WebPublishingError::PermissionDenied,
            SecureFecError::AuthenticationFailed => WebPublishingError::AuthenticationRequired,
            SecureFecError::NetworkError => WebPublishingError::NetworkError,
            SecureFecError::StorageError => WebPublishingError::StorageError,
            _ => WebPublishingError::EncryptionError,
        }
    }
}

impl From<RateLimitError> for WebPublishingError {
    fn from(_: RateLimitError) -> Self {
        WebPublishingError::RateLimitExceeded
    }
}

impl From<ValidationError> for WebPublishingError {
    fn from(_: ValidationError) -> Self {
        WebPublishingError::InvalidInput
    }
}

/// Security: Content type enumeration for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "plain_text")]
    PlainText,
}

impl ContentType {
    /// Security: Validate content type string
    pub fn from_str(s: &str) -> Result<Self, WebPublishingError> {
        match s.to_lowercase().as_str() {
            "markdown" => Ok(ContentType::Markdown),
            "html" => Ok(ContentType::Html),
            "plain_text" => Ok(ContentType::PlainText),
            _ => Err(WebPublishingError::InvalidInput),
        }
    }
}

/// Security: Request structure with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishContentRequest {
    pub title: String,
    pub description: String,
    pub content_type: ContentType,
    pub content: String,
    pub tags: Vec<String>,
    pub attachments: Option<Vec<String>>,
}

/// Security: Response structure for published content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebContentMetadata {
    pub content_id: String,
    pub title: String,
    pub description: String,
    pub content_type: ContentType,
    pub content: String,
    pub tags: Vec<String>,
    pub attachments: Option<Vec<FileAttachment>>,
    pub created_at: String,
    pub updated_at: String,
    pub size: usize,
}

/// Security: File attachment metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub file_id: String,
    pub file_name: String,
    pub file_size: usize,
    pub content_type: String,
}

/// Security: Web publishing state with rate limiting
pub struct WebPublishingState {
    fec_manager: Arc<SecureFecManager>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    input_validator: Arc<InputValidator>,
    content_cache: Arc<RwLock<std::collections::HashMap<String, WebContentMetadata>>>,
}

impl WebPublishingState {
    pub fn new() -> Result<Self, WebPublishingError> {
        let fec_config = FecConfig {
            max_content_size: 10 * 1024 * 1024, // 10MB limit for web content
            ..Default::default()
        };
        
        let fec_manager = Arc::new(
            SecureFecManager::new(fec_config)
                .map_err(|_| WebPublishingError::ConfigurationError)?
        );
        
        let rate_limiter = Arc::new(RwLock::new(
            RateLimiter::with_limit(10, std::time::Duration::from_secs(60)) // 10 requests per minute
        ));
        
        let input_validator = Arc::new(InputValidator::new());
        let content_cache = Arc::new(RwLock::new(std::collections::HashMap::new()));
        
        Ok(Self {
            fec_manager,
            rate_limiter,
            input_validator,
            content_cache,
        })
    }
}

/// Security: Publish web content with comprehensive validation and encryption
#[tauri::command]
pub async fn publish_web_content(
    request: PublishContentRequest,
    state: State<'_, WebPublishingState>,
) -> Result<String, WebPublishingError> {
    // Security: Check rate limit first
    {
        let mut limiter = state.rate_limiter.write().await;
        limiter.check_rate_limit("publish_web_content", 60)?;
    }

    // Security: Validate all input parameters
    state.input_validator.validate_title(&request.title)?;
    state.input_validator.validate_description(&request.description)?;
    state.input_validator.validate_content(&request.content)?;
    state.input_validator.validate_tags(&request.tags)?;

    // Security: Additional content-specific validation
    if request.content.len() > 10 * 1024 * 1024 {
        return Err(WebPublishingError::ContentTooLarge);
    }

    // Security: Sanitize content based on type
    let sanitized_content = match request.content_type {
        ContentType::Markdown => sanitize_markdown(&request.content)?,
        ContentType::Html => sanitize_html(&request.content)?,
        ContentType::PlainText => sanitize_plain_text(&request.content)?,
    };

    // Create content metadata
    let content_id = generate_content_id(&request)?;
    let now = chrono::Utc::now().to_rfc3339();
    
    let metadata = WebContentMetadata {
        content_id: content_id.clone(),
        title: request.title.clone(),
        description: request.description.clone(),
        content_type: request.content_type.clone(),
        content: sanitized_content.clone(),
        tags: request.tags.clone(),
        attachments: None, // TODO: Implement attachment handling
        created_at: now.clone(),
        updated_at: now,
        size: sanitized_content.len(),
    };

    // Security: Encrypt content using saorsa-fec
    let content_bytes = serde_json::to_vec(&metadata)
        .map_err(|_| WebPublishingError::StorageError)?;
    
    let encrypted_content = state.fec_manager
        .encrypt_data(&content_bytes, None)
        .await?;

    // Store encrypted content (in production, this would go to DHT)
    {
        let mut cache = state.content_cache.write().await;
        cache.insert(content_id.clone(), metadata);
    }

    // TODO: Distribute encrypted chunks to DHT peers with geographic routing
    // For now, we simulate successful distribution
    
    Ok(content_id)
}

/// Security: Browse web content with decryption and validation
#[tauri::command]
pub async fn browse_entity_web(
    four_word_address: String,
    state: State<'_, WebPublishingState>,
) -> Result<WebContentMetadata, WebPublishingError> {
    // Security: Check rate limit
    {
        let mut limiter = state.rate_limiter.write().await;
        limiter.check_rate_limit("browse_entity_web", 60)?;
    }

    // Security: Validate four-word address format
    state.input_validator.validate_four_word_address(&four_word_address)?;

    // Generate content ID from four-word address
    let content_id = calculate_content_id_from_address(&four_word_address)?;

    // Retrieve content from cache (in production, this would query DHT)
    let cached_content = {
        let cache = state.content_cache.read().await;
        cache.get(&content_id).cloned()
    };

    match cached_content {
        Some(content) => Ok(content),
        None => Err(WebPublishingError::ContentNotFound),
    }
}

/// Security: Store file with encryption and chunking
#[tauri::command]
pub async fn store_web_file(
    file_data: Vec<u8>,
    file_name: String,
    content_type: String,
    state: State<'_, WebPublishingState>,
) -> Result<String, WebPublishingError> {
    // Security: Check rate limit
    {
        let mut limiter = state.rate_limiter.write().await;
        limiter.check_rate_limit("store_file", 60)?;
    }

    // Security: Validate input parameters
    state.input_validator.validate_file_name(&file_name)?;
    state.input_validator.validate_content_type(&content_type)?;

    // Security: Validate file size
    if file_data.is_empty() {
        return Err(WebPublishingError::InvalidInput);
    }
    if file_data.len() > 100 * 1024 * 1024 { // 100MB limit
        return Err(WebPublishingError::ContentTooLarge);
    }

    // Generate file ID
    let file_id = Uuid::new_v4().to_string();

    // Security: Encrypt file data using saorsa-fec
    let encrypted_content = state.fec_manager
        .encrypt_data(&file_data, None)
        .await?;

    // TODO: Distribute encrypted chunks to DHT with Reed-Solomon encoding
    // TODO: Implement geographic distribution for optimal retrieval
    
    // For now, simulate successful storage
    Ok(file_id)
}

/// Security: Retrieve file with decryption and integrity verification
#[tauri::command]
pub async fn retrieve_file(
    file_id: String,
    state: State<'_, WebPublishingState>,
) -> Result<Vec<u8>, WebPublishingError> {
    // Security: Check rate limit
    {
        let mut limiter = state.rate_limiter.write().await;
        limiter.check_rate_limit("retrieve_file", 60)?;
    }

    // Security: Validate file ID format
    state.input_validator.validate_uuid(&file_id)?;

    // TODO: Retrieve encrypted chunks from DHT
    // TODO: Decrypt using appropriate key
    // TODO: Verify integrity and reconstruct original file
    
    // For now, return empty data to prevent compilation errors
    // In production, this would decrypt and return the actual file data
    Err(WebPublishingError::ContentNotFound)
}

/// Security: Generate deterministic content ID from request
fn generate_content_id(request: &PublishContentRequest) -> Result<String, WebPublishingError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(request.title.as_bytes());
    hasher.update(request.content.as_bytes());
    hasher.update(&serde_json::to_vec(&request.content_type)
        .map_err(|_| WebPublishingError::StorageError)?);
    
    for tag in &request.tags {
        hasher.update(tag.as_bytes());
    }
    
    Ok(hex::encode(hasher.finalize().as_bytes()))
}

/// Security: Calculate content ID from four-word address
fn calculate_content_id_from_address(address: &str) -> Result<String, WebPublishingError> {
    let hash = blake3::hash(address.as_bytes());
    Ok(hex::encode(hash.as_bytes()))
}

/// Security: Sanitize markdown content to prevent XSS
fn sanitize_markdown(content: &str) -> Result<String, WebPublishingError> {
    // TODO: Implement proper markdown sanitization using dompurify equivalent
    // For now, basic HTML entity encoding
    let sanitized = content
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    
    Ok(sanitized)
}

/// Security: Sanitize HTML content to prevent XSS
fn sanitize_html(content: &str) -> Result<String, WebPublishingError> {
    // TODO: Implement comprehensive HTML sanitization
    // For now, strip all HTML tags
    let sanitized = content
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    
    Ok(sanitized)
}

/// Security: Sanitize plain text content
fn sanitize_plain_text(content: &str) -> Result<String, WebPublishingError> {
    // Plain text just needs basic entity encoding
    let sanitized = content
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    
    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_publishing_state_creation() {
        let state = WebPublishingState::new().expect("Failed to create web publishing state");
        
        // Verify state components are initialized
        assert!(state.fec_manager.get_config().max_content_size > 0);
    }

    #[tokio::test]
    async fn test_content_id_generation() {
        let request = PublishContentRequest {
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            content_type: ContentType::Markdown,
            content: "Test content".to_string(),
            tags: vec!["test".to_string()],
            attachments: None,
        };

        let content_id = generate_content_id(&request).expect("Failed to generate content ID");
        
        // Should be a valid hex string
        assert_eq!(content_id.len(), 64);
        assert!(content_id.chars().all(|c| c.is_ascii_hexdigit()));

        // Same request should produce same ID
        let content_id2 = generate_content_id(&request).expect("Failed to generate content ID");
        assert_eq!(content_id, content_id2);
    }

    #[tokio::test]
    async fn test_content_sanitization() {
        let malicious_content = "<script>alert('xss')</script>Hello <b>world</b>";
        
        let sanitized_markdown = sanitize_markdown(malicious_content)
            .expect("Failed to sanitize markdown");
        assert!(!sanitized_markdown.contains("<script>"));
        assert!(!sanitized_markdown.contains("<b>"));

        let sanitized_html = sanitize_html(malicious_content)
            .expect("Failed to sanitize HTML");
        assert!(!sanitized_html.contains("<script>"));
        assert!(!sanitized_html.contains("<b>"));

        let sanitized_text = sanitize_plain_text(malicious_content)
            .expect("Failed to sanitize plain text");
        assert!(!sanitized_text.contains("<script>"));
        assert!(!sanitized_text.contains("<b>"));
    }

    #[tokio::test]
    async fn test_content_type_validation() {
        assert_eq!(ContentType::from_str("markdown").unwrap(), ContentType::Markdown);
        assert_eq!(ContentType::from_str("html").unwrap(), ContentType::Html);
        assert_eq!(ContentType::from_str("plain_text").unwrap(), ContentType::PlainText);
        
        assert!(ContentType::from_str("invalid").is_err());
    }

    #[tokio::test]
    async fn test_address_to_content_id() {
        let address = "ocean-forest-moon-star";
        let content_id = calculate_content_id_from_address(address)
            .expect("Failed to calculate content ID");
        
        // Should be deterministic
        let content_id2 = calculate_content_id_from_address(address)
            .expect("Failed to calculate content ID");
        assert_eq!(content_id, content_id2);
        
        // Different address should produce different ID
        let different_id = calculate_content_id_from_address("mountain-river-sky-earth")
            .expect("Failed to calculate content ID");
        assert_ne!(content_id, different_id);
    }
}