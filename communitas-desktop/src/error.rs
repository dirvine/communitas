// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.

//! Error handling for Tauri commands

use serde::{Deserialize, Serialize};

/// JavaScript-safe error type for Tauri IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl JsError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "[{}] {}", code, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

// Convert from AppError
impl From<communitas_core::AppError> for JsError {
    fn from(err: communitas_core::AppError) -> Self {
        Self::new(err.to_string())
    }
}

// Convert from CrdtError
impl From<communitas_core::CrdtError> for JsError {
    fn from(err: communitas_core::CrdtError) -> Self {
        Self::new(err.to_string())
    }
}

// Convert from anyhow::Error
impl From<anyhow::Error> for JsError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(err.to_string())
    }
}

// Convert from String
impl From<String> for JsError {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// Convert from &str
impl From<&str> for JsError {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_error_creation() {
        let err = JsError::new("test error");
        assert_eq!(err.message, "test error");
        assert!(err.code.is_none());

        let err = JsError::with_code("test error", "TEST_ERROR");
        assert_eq!(err.message, "test error");
        assert_eq!(err.code.as_ref().unwrap(), "TEST_ERROR");
    }

    #[test]
    fn test_js_error_display() {
        let err = JsError::new("test error");
        assert_eq!(err.to_string(), "test error");

        let err = JsError::with_code("test error", "TEST_ERROR");
        assert_eq!(err.to_string(), "[TEST_ERROR] test error");
    }
}
