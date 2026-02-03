// Central application error type for backend commands and services

use thiserror::Error;

#[allow(dead_code)]
pub type AppResult<T> = Result<T, AppError>;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Mutex poisoned")]
    MutexPoisoned,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Other: {0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(e: toml::de::Error) -> Self {
        AppError::Other(format!("TOML error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let io_error = AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        assert!(format!("{}", io_error).contains("I/O error"));

        let validation_error = AppError::Validation("invalid input".to_string());
        assert_eq!(
            format!("{}", validation_error),
            "Validation error: invalid input"
        );

        let not_found_error = AppError::NotFound("file.txt".to_string());
        assert_eq!(format!("{}", not_found_error), "Not found: file.txt");

        let network_error = AppError::Network("connection refused".to_string());
        assert_eq!(
            format!("{}", network_error),
            "Network error: connection refused"
        );
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err: AppError = anyhow::anyhow!("custom error").into();
        assert!(format!("{}", anyhow_err).contains("custom error"));
    }

    #[test]
    fn test_error_from_toml() {
        let toml_err: AppError = toml::from_str::<toml::Value>("=").unwrap_err().into();
        assert!(format!("{}", toml_err).contains("TOML error"));
    }

    #[test]
    fn test_app_result_type() {
        let success: AppResult<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);

        let failure: AppResult<i32> = Err(AppError::NotFound("missing".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_mutex_poisoned_error() {
        let error = AppError::MutexPoisoned;
        assert_eq!(format!("{}", error), "Mutex poisoned");
    }

    #[test]
    fn test_hex_error_conversion() {
        let hex_err: AppError = hex::FromHexError::InvalidHexCharacter { c: 'x', index: 0 }.into();
        assert!(format!("{}", hex_err).contains("Hex decode error"));
    }
}
