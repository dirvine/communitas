//! Comprehensive input validation and sanitization service
//! 
//! This module provides:
//! - Input validation using the validator crate
//! - Sanitization against injection attacks
//! - Path traversal protection
//! - Content-type validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use validator::{Validate, ValidationError, ValidationErrors};
use secrecy::{ExposeSecret, Secret};
use regex::Regex;

/// Maximum allowed input sizes to prevent DoS attacks
pub const MAX_MESSAGE_LENGTH: usize = 100_000;  // 100KB max message
pub const MAX_USERNAME_LENGTH: usize = 64;
pub const MAX_PATH_LENGTH: usize = 260;  // Windows MAX_PATH compatible
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
    pub fn validate_four_words(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Four-word address cannot be empty"));
        }

        if input.len() > MAX_FOUR_WORDS_LENGTH {
            return Err(anyhow::anyhow!("Four-word address too long: {} > {}", input.len(), MAX_FOUR_WORDS_LENGTH));
        }

        // Check for potential injection attempts
        if self.contains_malicious_content(input)? {
            return Err(anyhow::anyhow!("Four-word address contains potentially malicious content"));
        }

        let sanitized = input.trim().to_lowercase();

        if !self.four_words_pattern.is_match(&sanitized) {
            return Err(anyhow::anyhow!("Invalid four-word address format. Expected: word-word-word-word"));
        }

        Ok(sanitized)
    }

    /// Validate and sanitize a username
    pub fn validate_username(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }

        if input.len() > MAX_USERNAME_LENGTH {
            return Err(anyhow::anyhow!("Username too long: {} > {}", input.len(), MAX_USERNAME_LENGTH));
        }

        // Check for potential injection attempts
        if self.contains_malicious_content(input)? {
            return Err(anyhow::anyhow!("Username contains potentially malicious content"));
        }

        let sanitized = input.trim();

        if !self.username_pattern.is_match(sanitized) {
            return Err(anyhow::anyhow!("Invalid username format. Only alphanumeric characters, hyphens, and underscores allowed"));
        }

        Ok(sanitized.to_string())
    }

    /// Validate and sanitize a message content
    pub fn validate_message_content(&self, input: &str) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Message content cannot be empty"));
        }

        if input.len() > MAX_MESSAGE_LENGTH {
            return Err(anyhow::anyhow!("Message too long: {} > {}", input.len(), MAX_MESSAGE_LENGTH));
        }

        // Check for script injection attempts
        if self.script_injection_pattern.is_match(input) {
            return Err(anyhow::anyhow!("Message content contains potentially malicious scripts"));
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
            return Err(anyhow::anyhow!("File path too long: {} > {}", input.len(), MAX_PATH_LENGTH));
        }

        // Check for directory traversal patterns
        if input.contains("..") || input.contains("./") || input.contains("\\..") || input.contains(".\\") {
            return Err(anyhow::anyhow!("Path contains directory traversal patterns"));
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
                    return Err(anyhow::anyhow!("Canonicalized path contains traversal patterns"));
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
    fn contains_malicious_content(&self, input: &str) -> Result<bool> {
        // Check for SQL injection patterns
        if self.sql_injection_pattern.is_match(input) {
            return Ok(true);
        }

        // Check for script injection patterns
        if self.script_injection_pattern.is_match(input) {
            return Ok(true);
        }

        // Check for null bytes and control characters (except common whitespace)
        if input.contains('\0') || input.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            return Ok(true);
        }

        Ok(false)
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
            return Err(anyhow::anyhow!("JSON input too large: {} > {}", input.len(), MAX_MESSAGE_LENGTH));
        }

        // Parse JSON safely
        serde_json::from_str(input)
            .with_context(|| "Failed to parse JSON input")
    }

    /// Sanitize and validate a generic string input
    pub fn sanitize_string(&self, input: &str, max_length: usize) -> Result<String> {
        if input.is_empty() {
            return Err(anyhow::anyhow!("Input cannot be empty"));
        }

        if input.len() > max_length {
            return Err(anyhow::anyhow!("Input too long: {} > {}", input.len(), max_length));
        }

        if self.contains_malicious_content(input)? {
            return Err(anyhow::anyhow!("Input contains potentially malicious content"));
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedFourWords {
    #[validate(length(min = 7, max = 100))] // minimum: "a-b-c-d"
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedUsername {
    #[validate(length(min = 3, max = 64))]
    #[validate(regex = "USERNAME_REGEX")]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedMessage {
    #[validate(length(min = 1, max = 100000))]
    pub content: String,
    #[validate(length(max = 64))]
    pub message_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedPath {
    #[validate(length(min = 1, max = 260))]
    pub path: String,
}

// Regex constants for validator derive macro
lazy_static::lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]{3,64}$").unwrap();
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationErrors>;

/// Trait for types that can be validated
pub trait ValidatedInput: Sized {
    /// Validate the input using the validator
    fn validate_with(validator: &InputValidator, input: &str) -> Result<Self>;
}

impl ValidatedInput for ValidatedFourWords {
    fn validate_with(validator: &InputValidator, input: &str) -> Result<Self> {
        let validated_value = validator.validate_four_words(input)?;
        let instance = Self { value: validated_value };
        instance.validate().map_err(|e| anyhow::anyhow!("Validation failed: {:?}", e))?;
        Ok(instance)
    }
}

impl ValidatedInput for ValidatedUsername {
    fn validate_with(validator: &InputValidator, input: &str) -> Result<Self> {
        let validated_value = validator.validate_username(input)?;
        let instance = Self { value: validated_value };
        instance.validate().map_err(|e| anyhow::anyhow!("Validation failed: {:?}", e))?;
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
        assert!(validator.validate_four_words("hello-world-test-network").is_ok());
        assert!(validator.validate_four_words("a-b-c-d").is_ok());
        
        // Invalid formats
        assert!(validator.validate_four_words("hello-world-test").is_err()); // Only 3 words
        assert!(validator.validate_four_words("hello_world_test_network").is_err()); // Underscores
        assert!(validator.validate_four_words("hello world test network").is_err()); // Spaces
        assert!(validator.validate_four_words("").is_err()); // Empty
        assert!(validator.validate_four_words("Hello-World-Test-Network").is_ok()); // Uppercase (gets lowercased)
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
        assert!(validator.validate_file_path("documents/../../../secret").is_err());
        assert!(validator.validate_file_path("/absolute/path").is_err());
        assert!(validator.validate_file_path("").is_err());
    }

    #[test]
    fn test_malicious_content_detection() {
        let validator = InputValidator::new();
        
        // SQL injection attempts
        assert!(validator.validate_username("admin'; DROP TABLE users;--").is_err());
        assert!(validator.validate_message_content("SELECT * FROM secrets").is_err());
        
        // Script injection attempts
        assert!(validator.validate_message_content("<script>alert('xss')</script>").is_err());
        assert!(validator.validate_message_content("javascript:alert('xss')").is_err());
        
        // Clean content should pass
        assert!(validator.validate_message_content("This is a normal message").is_ok());
    }

    #[test]
    fn test_message_length_limits() {
        let validator = InputValidator::new();
        
        let long_message = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        assert!(validator.validate_message_content(&long_message).is_err());
        
        let max_message = "a".repeat(MAX_MESSAGE_LENGTH);
        assert!(validator.validate_message_content(&max_message).is_ok());
    }
}