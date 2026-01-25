//! Authentication components.
//!
//! Provides UI components for authentication flows including:
//! - Passkey/biometric authentication prompts
//! - Password authentication forms
//! - Recovery warnings

mod passkey_prompt;
mod recovery_warning;

pub use passkey_prompt::*;
pub use recovery_warning::*;
