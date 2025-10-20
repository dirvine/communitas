// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Input validation service for Communitas
//!
//! Centralizes all input validation logic that was previously scattered
//! across UI components. This ensures consistent validation rules and
//! keeps business logic out of the presentation layer.
use std::collections::HashSet;

/// Input type enumeration for validation rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    /// Entity names (channels, groups, projects, etc.)
    EntityName,
    /// Display names for users/identities
    DisplayName,
    /// Message content
    Message,
    /// Thread reply content
    ThreadReply,
    /// Password input
    Password,
}

/// Validation result type
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validation error with user-friendly messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: ValidationErrorCode,
}

impl std::error::Error for ValidationError {}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validation error codes for programmatic handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorCode {
    Required,
    TooShort,
    TooLong,
    InvalidFormat,
    ReservedWord,
    ContainsInvalidChars,
    Empty,
    OnlyWhitespace,
}

/// Main validation service
pub struct ValidationService {
    reserved_words: HashSet<String>,
}

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationService {
    /// Create a new validation service with default reserved words
    pub fn new() -> Self {
        let mut reserved_words = HashSet::new();
        // Add common reserved words that shouldn't be used as entity names
        for word in [
            "admin",
            "system",
            "root",
            "null",
            "undefined",
            "none",
            "default",
            "public",
            "private",
            "internal",
            "external",
            "test",
            "testing",
            "example",
            "sample",
            "demo",
            "temp",
            "temporary",
        ] {
            reserved_words.insert(word.to_string());
        }

        Self { reserved_words }
    }

    /// Validate input based on type
    pub fn validate(&self, input: &str, input_type: InputType) -> ValidationResult<()> {
        match input_type {
            InputType::EntityName => self.validate_entity_name(input),
            InputType::DisplayName => self.validate_display_name(input),
            InputType::Message => self.validate_message(input),
            InputType::ThreadReply => self.validate_thread_reply(input),
            InputType::Password => self.validate_password(input),
        }
    }

    /// Validate entity name (channel, group, project names)
    pub fn validate_entity_name(&self, name: &str) -> ValidationResult<()> {
        // Check for empty or whitespace-only
        if name.trim().is_empty() {
            return Err(ValidationError {
                field: "entity_name".to_string(),
                message: "Entity name cannot be empty".to_string(),
                code: ValidationErrorCode::Required,
            });
        }

        // Check minimum length (after trimming)
        let trimmed = name.trim();
        if trimmed.len() < 2 {
            return Err(ValidationError {
                field: "entity_name".to_string(),
                message: "Entity name must be at least 2 characters long".to_string(),
                code: ValidationErrorCode::TooShort,
            });
        }

        // Check maximum length
        if trimmed.len() > 50 {
            return Err(ValidationError {
                field: "entity_name".to_string(),
                message: "Entity name cannot exceed 50 characters".to_string(),
                code: ValidationErrorCode::TooLong,
            });
        }

        // Check for reserved words (case-insensitive)
        let lower_trimmed = trimmed.to_lowercase();
        if self.reserved_words.contains(&lower_trimmed) {
            return Err(ValidationError {
                field: "entity_name".to_string(),
                message: format!("'{}' is a reserved word and cannot be used", trimmed),
                code: ValidationErrorCode::ReservedWord,
            });
        }

        // Check for invalid characters (only allow alphanumeric, spaces, hyphens, underscores)
        if !trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
        {
            return Err(ValidationError {
                field: "entity_name".to_string(),
                message: "Entity name can only contain letters, numbers, spaces, hyphens, and underscores".to_string(),
                code: ValidationErrorCode::ContainsInvalidChars,
            });
        }

        // Check for consecutive spaces or special characters
        if trimmed.contains("  ") || trimmed.contains("--") || trimmed.contains("__") {
            return Err(ValidationError {
                field: "entity_name".to_string(),
                message: "Entity name cannot contain consecutive spaces or special characters"
                    .to_string(),
                code: ValidationErrorCode::InvalidFormat,
            });
        }

        Ok(())
    }

    /// Validate display name
    pub fn validate_display_name(&self, name: &str) -> ValidationResult<()> {
        // Check for empty or whitespace-only
        if name.trim().is_empty() {
            return Err(ValidationError {
                field: "display_name".to_string(),
                message: "Display name cannot be empty".to_string(),
                code: ValidationErrorCode::Required,
            });
        }

        let trimmed = name.trim();

        // Check minimum length
        if trimmed.is_empty() {
            return Err(ValidationError {
                field: "display_name".to_string(),
                message: "Display name cannot be empty".to_string(),
                code: ValidationErrorCode::TooShort,
            });
        }

        // Check maximum length
        if trimmed.len() > 100 {
            return Err(ValidationError {
                field: "display_name".to_string(),
                message: "Display name cannot exceed 100 characters".to_string(),
                code: ValidationErrorCode::TooLong,
            });
        }

        // Allow more characters for display names (including emojis, special chars)
        // Just check for control characters
        if trimmed.chars().any(|c| c.is_control()) {
            return Err(ValidationError {
                field: "display_name".to_string(),
                message: "Display name cannot contain control characters".to_string(),
                code: ValidationErrorCode::ContainsInvalidChars,
            });
        }

        Ok(())
    }

    /// Validate message content
    pub fn validate_message(&self, message: &str) -> ValidationResult<()> {
        // Allow empty messages for drafts, but not whitespace-only if non-empty
        let trimmed = message.trim();

        if !message.is_empty() && trimmed.is_empty() {
            return Err(ValidationError {
                field: "message".to_string(),
                message: "Message cannot be only whitespace".to_string(),
                code: ValidationErrorCode::OnlyWhitespace,
            });
        }

        // Check maximum length (reasonable limit for messages)
        if message.len() > 10000 {
            return Err(ValidationError {
                field: "message".to_string(),
                message: "Message cannot exceed 10,000 characters".to_string(),
                code: ValidationErrorCode::TooLong,
            });
        }

        // Allow all characters in messages (including newlines, emojis, etc.)
        // Just check for null bytes or other problematic characters
        if message.chars().any(|c| c == '\0') {
            return Err(ValidationError {
                field: "message".to_string(),
                message: "Message cannot contain null characters".to_string(),
                code: ValidationErrorCode::ContainsInvalidChars,
            });
        }

        Ok(())
    }

    /// Validate thread reply content (same rules as messages)
    pub fn validate_thread_reply(&self, reply: &str) -> ValidationResult<()> {
        self.validate_message(reply)
    }

    /// Validate password
    pub fn validate_password(&self, password: &str) -> ValidationResult<()> {
        // Check minimum length
        if password.len() < 8 {
            return Err(ValidationError {
                field: "password".to_string(),
                message: "Password must be at least 8 characters long".to_string(),
                code: ValidationErrorCode::TooShort,
            });
        }

        // Check maximum length (reasonable limit for security)
        if password.len() > 128 {
            return Err(ValidationError {
                field: "password".to_string(),
                message: "Password cannot exceed 128 characters".to_string(),
                code: ValidationErrorCode::TooLong,
            });
        }

        // Check for control characters
        if password.chars().any(|c| c.is_control()) {
            return Err(ValidationError {
                field: "password".to_string(),
                message: "Password cannot contain control characters".to_string(),
                code: ValidationErrorCode::ContainsInvalidChars,
            });
        }

        Ok(())
    }

    /// Sanitize input by trimming whitespace
    pub fn sanitize(&self, input: &str, input_type: InputType) -> String {
        match input_type {
            InputType::EntityName | InputType::DisplayName => input.trim().to_string(),
            InputType::Message | InputType::ThreadReply | InputType::Password => input.to_string(),
        }
    }

    /// Validate and sanitize input in one step
    pub fn validate_and_sanitize(
        &self,
        input: &str,
        input_type: InputType,
    ) -> ValidationResult<String> {
        let sanitized = self.sanitize(input, input_type);
        self.validate(&sanitized, input_type)?;
        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_validator() -> ValidationService {
        ValidationService::new()
    }

    mod entity_name_validation {
        use super::*;

        #[test]
        fn test_valid_entity_names() {
            let validator = create_validator();

            // Valid names should pass
            assert!(validator.validate_entity_name("My Channel").is_ok());
            assert!(validator.validate_entity_name("project-alpha").is_ok());
            assert!(validator.validate_entity_name("Group_123").is_ok());
            assert!(validator.validate_entity_name("AB").is_ok()); // Minimum 2 chars after trim
            assert!(
                validator
                    .validate_entity_name("Valid Name With Spaces")
                    .is_ok()
            );
        }

        #[test]
        fn test_empty_entity_names() {
            let validator = create_validator();

            let result = validator.validate_entity_name("");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::Required);

            let result = validator.validate_entity_name("   ");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::Required);
        }

        #[test]
        fn test_too_short_entity_names() {
            let validator = create_validator();

            let result = validator.validate_entity_name("A");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::TooShort);
        }

        #[test]
        fn test_too_long_entity_names() {
            let validator = create_validator();

            let long_name = "A".repeat(51);
            let result = validator.validate_entity_name(&long_name);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::TooLong);
        }

        #[test]
        fn test_reserved_words() {
            let validator = create_validator();

            let result = validator.validate_entity_name("admin");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::ReservedWord);

            let result = validator.validate_entity_name("ADMIN"); // case insensitive
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::ReservedWord);
        }

        #[test]
        fn test_invalid_characters() {
            let validator = create_validator();

            let result = validator.validate_entity_name("Channel@Name");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::ContainsInvalidChars
            );

            let result = validator.validate_entity_name("Channel.Name");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::ContainsInvalidChars
            );
        }

        #[test]
        fn test_consecutive_special_chars() {
            let validator = create_validator();

            let result = validator.validate_entity_name("Channel--Name");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::InvalidFormat);

            let result = validator.validate_entity_name("Channel  Name");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::InvalidFormat);
        }
    }

    mod display_name_validation {
        use super::*;

        #[test]
        fn test_valid_display_names() {
            let validator = create_validator();

            assert!(validator.validate_display_name("John Doe").is_ok());
            assert!(validator.validate_display_name("Alice").is_ok());
            assert!(validator.validate_display_name("用户123").is_ok()); // Unicode
            assert!(
                validator
                    .validate_display_name("Name with émojis 🎉")
                    .is_ok()
            );
        }

        #[test]
        fn test_empty_display_names() {
            let validator = create_validator();

            let result = validator.validate_display_name("");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::Required);
        }

        #[test]
        fn test_too_long_display_names() {
            let validator = create_validator();

            let long_name = "A".repeat(101);
            let result = validator.validate_display_name(&long_name);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::TooLong);
        }

        #[test]
        fn test_control_characters_in_display_names() {
            let validator = create_validator();

            let result = validator.validate_display_name("Name\nwith\nnewlines");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::ContainsInvalidChars
            );
        }
    }

    mod message_validation {
        use super::*;

        #[test]
        fn test_valid_messages() {
            let validator = create_validator();

            assert!(validator.validate_message("").is_ok()); // Empty messages allowed
            assert!(validator.validate_message("Hello world!").is_ok());
            assert!(validator.validate_message("Multi\nline\nmessage").is_ok());
            assert!(validator.validate_message("Message with émojis 🎉").is_ok());
        }

        #[test]
        fn test_whitespace_only_messages() {
            let validator = create_validator();

            let result = validator.validate_message("   ");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::OnlyWhitespace
            );

            let result = validator.validate_message("\t\n  \t");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::OnlyWhitespace
            );
        }

        #[test]
        fn test_too_long_messages() {
            let validator = create_validator();

            let long_message = "A".repeat(10001);
            let result = validator.validate_message(&long_message);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::TooLong);
        }

        #[test]
        fn test_null_characters_in_messages() {
            let validator = create_validator();

            let result = validator.validate_message("Message with \0 null");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::ContainsInvalidChars
            );
        }
    }

    mod password_validation {
        use super::*;

        #[test]
        fn test_valid_passwords() {
            let validator = create_validator();

            assert!(validator.validate_password("password123").is_ok());
            assert!(validator.validate_password("A".repeat(8).as_str()).is_ok());
            assert!(validator.validate_password("Complex!@#$%^&*()").is_ok());
        }

        #[test]
        fn test_too_short_passwords() {
            let validator = create_validator();

            let result = validator.validate_password("short");
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::TooShort);
        }

        #[test]
        fn test_too_long_passwords() {
            let validator = create_validator();

            let long_password = "A".repeat(129);
            let result = validator.validate_password(&long_password);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().code, ValidationErrorCode::TooLong);
        }

        #[test]
        fn test_control_characters_in_passwords() {
            let validator = create_validator();

            let result = validator.validate_password("password\n123");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().code,
                ValidationErrorCode::ContainsInvalidChars
            );
        }
    }

    mod sanitization {
        use super::*;

        #[test]
        fn test_entity_name_sanitization() {
            let validator = create_validator();

            assert_eq!(
                validator.sanitize("  My Channel  ", InputType::EntityName),
                "My Channel"
            );
            assert_eq!(
                validator.sanitize("project", InputType::EntityName),
                "project"
            );
        }

        #[test]
        fn test_message_sanitization() {
            let validator = create_validator();

            // Messages preserve whitespace
            assert_eq!(
                validator.sanitize("  Hello  ", InputType::Message),
                "  Hello  "
            );
            assert_eq!(
                validator.sanitize("Multi\nline", InputType::Message),
                "Multi\nline"
            );
        }

        #[test]
        fn test_validate_and_sanitize() {
            let validator = create_validator();

            let result = validator.validate_and_sanitize("  My Channel  ", InputType::EntityName);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "My Channel");

            let result = validator.validate_and_sanitize("", InputType::EntityName);
            assert!(result.is_err());
        }
    }
}
