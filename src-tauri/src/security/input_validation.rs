//! Comprehensive input validation and sanitization service
//!
//! This module provides:
//! - Input validation using the validator crate
//! - Sanitization against injection attacks
//! - Path traversal protection
//! - Content-type validation

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    #[error("Input validation failed: {0}")]
    InvalidInput(String),
    #[error("Input too long: {current} > {max}")]
    TooLong { current: usize, max: usize },
    #[error("Input contains malicious content")]
    MaliciousContent,
    #[error("Invalid format")]
    InvalidFormat,
}

/// Maximum allowed input sizes to prevent DoS attacks
pub const MAX_MESSAGE_LENGTH: usize = 100_000; // 100KB max message
pub const MAX_USERNAME_LENGTH: usize = 64;
pub const MAX_PATH_LENGTH: usize = 260; // Windows MAX_PATH compatible
pub const MAX_FOUR_WORDS_LENGTH: usize = 100;

/// Input validation service providing secure input processing
#[derive(Debug, Clone)]
pub struct InputValidator {
    /// Regex for validating four-word addresses
    four_words_pattern: Regex,
    /// Regex for validating usernames
    username_pattern: Regex,
    /// Regex for detecting potential SQL injection
    sql_injection_pattern: Regex,
    /// Regex for detecting script injection
    script_injection_pattern: Regex,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl InputValidator {
    /// Create a new input validator with compiled regex patterns
    pub fn new() -> Self {
        Self {
            four_words_pattern: Regex::new(r"^[a-z]+-[a-z]+-[a-z]+-[a-z]+$")
                .expect("Four words pattern should compile"),
            username_pattern: Regex::new(r"^[a-zA-Z0-9_-]{3,64}$")
                .expect("Username pattern should compile"),
            sql_injection_pattern: Regex::new(r"(?i)(select|insert|update|delete|drop|create|alter|exec|union|script|javascript|vbscript)")
                .expect("SQL injection pattern should compile"),
            script_injection_pattern: Regex::new(r"(?i)(<script|javascript:|vbscript:|on\w+\s*=)")
                .expect("Script injection pattern should compile"),
        }
    }

    /// Validate and sanitize a four-word network identity
    pub fn validate_four_words(&self, input: &str) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Four-word address cannot be empty".to_string(),
            ));
        }

        if input.len() > MAX_FOUR_WORDS_LENGTH {
            return Err(ValidationError::TooLong {
                current: input.len(),
                max: MAX_FOUR_WORDS_LENGTH,
            });
        }

        // Check for potential injection attempts
        if self.contains_malicious_content(input) {
            return Err(ValidationError::MaliciousContent);
        }

        let sanitized = input.trim().to_lowercase();

        if !self.four_words_pattern.is_match(&sanitized) {
            return Err(ValidationError::InvalidFormat);
        }

        Ok(sanitized)
    }

    /// Validate four-word address format
    pub fn validate_four_word_address(&self, input: &str) -> Result<String, ValidationError> {
        self.validate_four_words(input)
    }

    /// Validate title field
    pub fn validate_title(&self, input: &str) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Title cannot be empty".to_string(),
            ));
        }
        if input.len() > 200 {
            return Err(ValidationError::TooLong {
                current: input.len(),
                max: 200,
            });
        }
        if self.contains_malicious_content(input) {
            return Err(ValidationError::MaliciousContent);
        }
        Ok(input.trim().to_string())
    }

    /// Validate description field
    pub fn validate_description(&self, input: &str) -> Result<String, ValidationError> {
        if input.len() > 1000 {
            return Err(ValidationError::TooLong {
                current: input.len(),
                max: 1000,
            });
        }
        if self.contains_malicious_content(input) {
            return Err(ValidationError::MaliciousContent);
        }
        Ok(input.trim().to_string())
    }

    /// Validate content field
    pub fn validate_content(&self, input: &str) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Content cannot be empty".to_string(),
            ));
        }
        if input.len() > 10 * 1024 * 1024 {
            return Err(ValidationError::TooLong {
                current: input.len(),
                max: 10 * 1024 * 1024,
            });
        }
        if self.script_injection_pattern.is_match(input) {
            return Err(ValidationError::MaliciousContent);
        }
        Ok(input.to_string())
    }

    /// Validate tags array
    pub fn validate_tags(&self, tags: &[String]) -> Result<Vec<String>, ValidationError> {
        if tags.len() > 10 {
            return Err(ValidationError::TooLong {
                current: tags.len(),
                max: 10,
            });
        }

        let mut validated_tags = Vec::new();
        for tag in tags {
            if tag.is_empty() {
                continue; // Skip empty tags
            }
            if tag.len() > 50 {
                return Err(ValidationError::TooLong {
                    current: tag.len(),
                    max: 50,
                });
            }
            if self.contains_malicious_content(tag) {
                return Err(ValidationError::MaliciousContent);
            }
            validated_tags.push(tag.trim().to_lowercase());
        }
        Ok(validated_tags)
    }

    /// Validate file name
    pub fn validate_file_name(&self, input: &str) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::InvalidInput(
                "File name cannot be empty".to_string(),
            ));
        }
        if input.len() > 255 {
            return Err(ValidationError::TooLong {
                current: input.len(),
                max: 255,
            });
        }
        if input.contains('/') || input.contains('\\') || input.contains('\0') {
            return Err(ValidationError::InvalidFormat);
        }
        if self.contains_malicious_content(input) {
            return Err(ValidationError::MaliciousContent);
        }
        Ok(input.trim().to_string())
    }

    /// Validate content type
    pub fn validate_content_type(&self, input: &str) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Content type cannot be empty".to_string(),
            ));
        }
        if input.len() > 100 {
            return Err(ValidationError::TooLong {
                current: input.len(),
                max: 100,
            });
        }
        if self.contains_malicious_content(input) {
            return Err(ValidationError::MaliciousContent);
        }
        Ok(input.trim().to_lowercase())
    }

    /// Validate UUID format
    pub fn validate_uuid(&self, input: &str) -> Result<String, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::InvalidInput(
                "UUID cannot be empty".to_string(),
            ));
        }
        // Basic UUID format check
        let uuid_pattern = Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
        )
        .map_err(|_| ValidationError::InvalidFormat)?;

        if !uuid_pattern.is_match(input) {
            return Err(ValidationError::InvalidFormat);
        }
        Ok(input.to_lowercase())
    }

    /// Validate and sanitize a username
    pub fn validate_username(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }

        if input.len() > MAX_USERNAME_LENGTH {
            return Err(anyhow::anyhow!(
                "Username too long: {} > {}",
                input.len(),
                MAX_USERNAME_LENGTH
            ));
        }

        // Check for potential injection attempts
        if self.contains_malicious_content(input) {
            return Err(anyhow::anyhow!(
                "Username contains potentially malicious content"
            ));
        }

        let sanitized = input.trim();

        if !self.username_pattern.is_match(sanitized) {
            return Err(anyhow::anyhow!(
                "Invalid username format. Only alphanumeric characters, hyphens, and underscores allowed"
            ));
        }

        Ok(sanitized.to_string())
    }

    /// Validate and sanitize a message content
    pub fn validate_message_content(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Message content cannot be empty"));
        }

        if input.len() > MAX_MESSAGE_LENGTH {
            return Err(anyhow::anyhow!(
                "Message too long: {} > {}",
                input.len(),
                MAX_MESSAGE_LENGTH
            ));
        }

        // Check for script injection attempts
        if self.script_injection_pattern.is_match(input) {
            return Err(anyhow::anyhow!(
                "Message content contains potentially malicious scripts"
            ));
        }

        // Sanitize the message by removing null bytes and control characters
        let sanitized = input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect::<String>();

        Ok(sanitized)
    }

    /// Validate and secure a file path against directory traversal attacks
    pub fn validate_file_path(&self, input: &str) -> Result<PathBuf> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("File path cannot be empty"));
        }

        if input.len() > MAX_PATH_LENGTH {
            return Err(anyhow::anyhow!(
                "File path too long: {} > {}",
                input.len(),
                MAX_PATH_LENGTH
            ));
        }

        // Check for directory traversal patterns
        if input.contains("..")
            || input.contains("./")
            || input.contains("\\..")
            || input.contains(".\\")
        {
            return Err(anyhow::anyhow!(
                "Path contains directory traversal patterns"
            ));
        }

        // Check for absolute paths (we only allow relative paths)
        if Path::new(input).is_absolute() {
            return Err(anyhow::anyhow!("Absolute paths not allowed"));
        }

        // Check for null bytes and other problematic characters
        if input.contains('\0') || input.contains('\x01') {
            return Err(anyhow::anyhow!("Path contains invalid characters"));
        }

        let path = PathBuf::from(input);

        // Verify the path doesn't escape when canonicalized
        match path.canonicalize() {
            Ok(canonical) => {
                // This is a simplistic check - in production you'd check against allowed directories
                if canonical.to_string_lossy().contains("..") {
                    return Err(anyhow::anyhow!(
                        "Canonicalized path contains traversal patterns"
                    ));
                }
            }
            Err(_) => {
                // Path doesn't exist yet, which is fine for new files
                // But we still validate the structure
            }
        }

        Ok(path)
    }

    /// Check if input contains potentially malicious content
    fn contains_malicious_content(&self, input: &str) -> bool {
        // Check for SQL injection patterns
        if self.sql_injection_pattern.is_match(input) {
            return true;
        }

        // Check for script injection patterns
        if self.script_injection_pattern.is_match(input) {
            return true;
        }

        // Check for null bytes and control characters (except common whitespace)
        if input.contains('\0')
            || input
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            return true;
        }

        false
    }

    /// Validate JSON input structure and size
    pub fn validate_json_input<T>(&self, input: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if input.is_empty() {
            return Err(anyhow::anyhow!("JSON input cannot be empty"));
        }

        if input.len() > MAX_MESSAGE_LENGTH {
            return Err(anyhow::anyhow!(
                "JSON input too large: {} > {}",
                input.len(),
                MAX_MESSAGE_LENGTH
            ));
        }

        // Parse JSON safely
        serde_json::from_str(input).with_context(|| "Failed to parse JSON input")
    }

    /// Sanitize and validate a generic string input
    pub fn sanitize_string(&self, input: &str, max_length: usize) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Input cannot be empty"));
        }

        if input.len() > max_length {
            return Err(anyhow::anyhow!(
                "Input too long: {} > {}",
                input.len(),
                max_length
            ));
        }

        if self.contains_malicious_content(input) {
            return Err(anyhow::anyhow!(
                "Input contains potentially malicious content"
            ));
        }

        // Remove control characters except common whitespace
        let sanitized = input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect::<String>()
            .trim()
            .to_string();

        if sanitized.is_empty() {
            return Err(anyhow::anyhow!("Input becomes empty after sanitization"));
        }

        Ok(sanitized)
    }
}

/// Validated input types for common use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedFourWords {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedUsername {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedMessage {
    pub content: String,
    pub message_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPath {
    pub path: String,
}

// Regex constants for validator derive macro
lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{3,64}$")
        .expect("Failed to compile username regex pattern - this is a development error");
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Trait for types that can be validated
pub trait ValidatedInput: Sized {
    /// Validate the input using the validator
    fn validate_with(validator: &InputValidator, input: &str) -> Result<Self>;
}

impl ValidatedInput for ValidatedFourWords {
    fn validate_with(validator: &InputValidator, input: &str) -> Result<Self> {
        let validated_value = validator.validate_four_words(input)?;
        let instance = Self {
            value: validated_value,
        };
        Ok(instance)
    }
}

impl ValidatedInput for ValidatedUsername {
    fn validate_with(validator: &InputValidator, input: &str) -> Result<Self> {
        let validated_value = validator.validate_username(input)?;
        let instance = Self {
            value: validated_value,
        };
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_words_validation() {
        let validator = InputValidator::new();

        // Valid four-words addresses
        assert!(
            validator
                .validate_four_words("hello-world-test-network")
                .is_ok()
        );
        assert!(validator.validate_four_words("a-b-c-d").is_ok());

        // Invalid formats
        assert!(validator.validate_four_words("hello-world-test").is_err()); // Only 3 words
        assert!(
            validator
                .validate_four_words("hello_world_test_network")
                .is_err()
        ); // Underscores
        assert!(
            validator
                .validate_four_words("hello world test network")
                .is_err()
        ); // Spaces
        assert!(validator.validate_four_words("").is_err()); // Empty
        assert!(
            validator
                .validate_four_words("Hello-World-Test-Network")
                .is_ok()
        ); // Uppercase (gets lowercased)
    }

    #[test]
    fn test_username_validation() {
        let validator = InputValidator::new();

        // Valid usernames
        assert!(validator.validate_username("user123").is_ok());
        assert!(validator.validate_username("test-user").is_ok());
        assert!(validator.validate_username("user_name").is_ok());

        // Invalid usernames
        assert!(validator.validate_username("ab").is_err()); // Too short
        assert!(validator.validate_username("user@example.com").is_err()); // Invalid characters
        assert!(validator.validate_username("").is_err()); // Empty
        assert!(validator.validate_username(&"a".repeat(65)).is_err()); // Too long
    }

    #[test]
    fn test_path_validation() {
        let validator = InputValidator::new();

        // Valid paths
        assert!(validator.validate_file_path("documents/test.md").is_ok());
        assert!(validator.validate_file_path("file.txt").is_ok());

        // Invalid paths (directory traversal)
        assert!(validator.validate_file_path("../etc/passwd").is_err());
        assert!(
            validator
                .validate_file_path("documents/../../../secret")
                .is_err()
        );
        assert!(validator.validate_file_path("/absolute/path").is_err());
        assert!(validator.validate_file_path("").is_err());
    }

    #[test]
    fn test_malicious_content_detection() {
        let validator = InputValidator::new();

        // SQL injection attempts
        assert!(
            validator
                .validate_username("admin'; DROP TABLE users;--")
                .is_err()
        );
        assert!(
            validator
                .validate_message_content("SELECT * FROM secrets")
                .is_err()
        );

        // Script injection attempts
        assert!(
            validator
                .validate_message_content("<script>alert('xss')</script>")
                .is_err()
        );
        assert!(
            validator
                .validate_message_content("javascript:alert('xss')")
                .is_err()
        );

        // Clean content should pass
        assert!(
            validator
                .validate_message_content("This is a normal message")
                .is_ok()
        );
    }

    #[test]
    fn test_message_length_limits() {
        let validator = InputValidator::new();

        let long_message = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        assert!(validator.validate_message_content(&long_message).is_err());

        let max_message = "a".repeat(MAX_MESSAGE_LENGTH);
        assert!(validator.validate_message_content(&max_message).is_ok());
    }

    #[test]
    fn test_json_validation() {
        let validator = InputValidator::new();

        // Valid JSON
        let valid_json = r#"{"name": "test", "value": 123}"#;
        let result: Result<serde_json::Value, _> = validator.validate_json_input(valid_json);
        assert!(result.is_ok());

        // Invalid JSON
        let invalid_json = r#"{"name": "test", "value": }"#;
        let result: Result<serde_json::Value, _> = validator.validate_json_input(invalid_json);
        assert!(result.is_err());

        // Empty input
        let empty_json = "";
        let result: Result<serde_json::Value, _> = validator.validate_json_input(empty_json);
        assert!(result.is_err());

        // Oversized JSON
        let oversized_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(MAX_MESSAGE_LENGTH));
        let result: Result<serde_json::Value, _> = validator.validate_json_input(&oversized_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_string() {
        let validator = InputValidator::new();

        // Normal string
        let result = validator.sanitize_string("Hello World", 100);
        assert_eq!(result.unwrap(), "Hello World");

        // String with control characters
        let result = validator.sanitize_string("Hello\n\r\tWorld\x00", 100);
        assert_eq!(result.unwrap(), "Hello\n\r\tWorld");

        // Empty string
        let result = validator.sanitize_string("", 100);
        assert!(result.is_err());

        // Oversized string
        let long_string = "a".repeat(101);
        let result = validator.sanitize_string(&long_string, 100);
        assert!(result.is_err());

        // String that becomes empty after sanitization
        let result = validator.sanitize_string("\x01\x02\x03", 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_title_validation() {
        let validator = InputValidator::new();

        // Valid title
        let result = validator.validate_title("My Project Title");
        assert_eq!(result.unwrap(), "My Project Title");

        // Empty title
        let result = validator.validate_title("");
        assert!(result.is_err());

        // Oversized title
        let long_title = "a".repeat(201);
        let result = validator.validate_title(&long_title);
        assert!(result.is_err());

        // Title with malicious content
        let result = validator.validate_title("Project <script>alert('xss')</script>");
        assert!(result.is_err());
    }

    #[test]
    fn test_description_validation() {
        let validator = InputValidator::new();

        // Valid description
        let result = validator.validate_description("This is a project description");
        assert_eq!(result.unwrap(), "This is a project description");

        // Empty description (should be OK)
        let result = validator.validate_description("");
        assert_eq!(result.unwrap(), "");

        // Oversized description
        let long_desc = "a".repeat(1001);
        let result = validator.validate_description(&long_desc);
        assert!(result.is_err());

        // Description with malicious content
        let result =
            validator.validate_description("Description with <script>alert('xss')</script> code");
        assert!(result.is_err());
    }

    #[test]
    fn test_content_validation() {
        let validator = InputValidator::new();

        // Valid content
        let result = validator.validate_content("This is valid content");
        assert_eq!(result.unwrap(), "This is valid content");

        // Empty content
        let result = validator.validate_content("");
        assert!(result.is_err());

        // Oversized content
        let long_content = "a".repeat(10 * 1024 * 1024 + 1);
        let result = validator.validate_content(&long_content);
        assert!(result.is_err());

        // Content with script tags
        let result =
            validator.validate_content("Content with <script>alert('xss')</script> scripts");
        assert!(result.is_err());
    }

    #[test]
    fn test_tags_validation() {
        let validator = InputValidator::new();

        // Valid tags
        let tags = vec![
            "rust".to_string(),
            "security".to_string(),
            "p2p".to_string(),
        ];
        let result = validator.validate_tags(&tags);
        assert_eq!(result.unwrap().len(), 3);

        // Too many tags
        let many_tags = (0..11).map(|i| format!("tag{}", i)).collect::<Vec<_>>();
        let result = validator.validate_tags(&many_tags);
        assert!(result.is_err());

        // Oversized tag
        let oversized_tag = "a".repeat(51);
        let tags = vec![oversized_tag];
        let result = validator.validate_tags(&tags);
        assert!(result.is_err());

        // Tag with malicious content
        let tags = vec![
            "normal".to_string(),
            "<script>alert('xss')</script>".to_string(),
        ];
        let result = validator.validate_tags(&tags);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_name_validation() {
        let validator = InputValidator::new();

        // Valid file names
        assert!(validator.validate_file_name("document.pdf").is_ok());
        assert!(validator.validate_file_name("my-file_v2.txt").is_ok());

        // Empty file name
        assert!(validator.validate_file_name("").is_err());

        // Oversized file name
        let long_name = "a".repeat(256);
        assert!(validator.validate_file_name(&long_name).is_err());

        // File name with path traversal
        assert!(validator.validate_file_name("../secret.txt").is_err());
        assert!(validator.validate_file_name("../../../etc/passwd").is_err());

        // File name with null bytes
        assert!(validator.validate_file_name("file.txt\x00").is_err());

        // File name with malicious content
        assert!(validator.validate_file_name("file<script>.txt").is_err());
    }

    #[test]
    fn test_content_type_validation() {
        let validator = InputValidator::new();

        // Valid content types
        assert!(validator.validate_content_type("application/pdf").is_ok());
        assert!(validator.validate_content_type("text/plain").is_ok());
        assert!(validator.validate_content_type("image/jpeg").is_ok());

        // Empty content type
        assert!(validator.validate_content_type("").is_err());

        // Oversized content type
        let long_type = "a".repeat(101);
        assert!(validator.validate_content_type(&long_type).is_err());

        // Content type with malicious content
        assert!(
            validator
                .validate_content_type("text/html<script>")
                .is_err()
        );
    }

    #[test]
    fn test_uuid_validation() {
        let validator = InputValidator::new();

        // Valid UUID
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result = validator.validate_uuid(valid_uuid);
        assert_eq!(result.unwrap(), valid_uuid.to_lowercase());

        // Empty UUID
        assert!(validator.validate_uuid("").is_err());

        // Invalid UUID format
        assert!(validator.validate_uuid("not-a-uuid").is_err());
        assert!(validator.validate_uuid("550e8400-e29b-41d4-a716").is_err()); // Too short
        assert!(
            validator
                .validate_uuid("550e8400-e29b-41d4-a716-446655440000-extra")
                .is_err()
        ); // Too long
    }
}
