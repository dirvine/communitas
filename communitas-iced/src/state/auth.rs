// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Authentication state for login and vault management.

/// Information about a stored vault.
#[derive(Debug, Clone)]
pub struct VaultInfo {
    /// Four-word identity for this vault.
    pub four_words: String,
    /// Display name associated with the identity.
    pub display_name: String,
    /// Path to the vault storage.
    pub path: String,
    /// Whether biometric auth is available.
    pub biometric_available: bool,
}

/// Authentication state.
#[derive(Debug, Clone, Default)]
pub struct AuthState {
    /// Available vaults on this device.
    pub vaults: Vec<VaultInfo>,
    /// Currently selected vault (by four-word identity).
    pub selected_vault: Option<String>,
    /// Password input field.
    pub password: String,
    /// Whether authentication is in progress.
    pub is_loading: bool,
    /// Error message from last attempt.
    pub error: Option<String>,
    /// Whether biometric authentication is available.
    pub biometric_available: bool,
    /// Whether we're in create identity mode.
    pub creating_identity: bool,
    /// New identity display name input.
    pub new_display_name: String,
    /// New identity password input.
    pub new_password: String,
    /// New identity password confirmation.
    pub new_password_confirm: String,
}

impl AuthState {
    /// Create a new authentication state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a vault is selected.
    #[must_use]
    pub fn has_selected_vault(&self) -> bool {
        self.selected_vault.is_some()
    }

    /// Get the selected vault info.
    #[must_use]
    pub fn selected_vault_info(&self) -> Option<&VaultInfo> {
        self.selected_vault
            .as_ref()
            .and_then(|fw| self.vaults.iter().find(|v| &v.four_words == fw))
    }

    /// Check if passwords match for new identity creation.
    #[must_use]
    pub fn passwords_match(&self) -> bool {
        !self.new_password.is_empty() && self.new_password == self.new_password_confirm
    }

    /// Validate new identity form.
    #[must_use]
    pub fn can_create_identity(&self) -> bool {
        !self.new_display_name.is_empty() && self.new_password.len() >= 8 && self.passwords_match()
    }
}
