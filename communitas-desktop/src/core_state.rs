// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.

//! CoreContext lifecycle management

use crate::error::JsError;
use communitas_core::{AppError, CoreContext};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Managed state for CoreContext with initialization tracking
pub struct CoreState {
    inner: RwLock<Option<CoreContext>>,
}

impl CoreState {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(None),
        }
    }

    /// Get the CoreContext, returning an error if not initialized
    pub async fn get(&self) -> Result<CoreContext, JsError> {
        self.inner
            .read()
            .await
            .clone()
            .ok_or_else(|| JsError::with_code(
                "Core not initialized. Call core_initialize first.",
                "CORE_NOT_INITIALIZED"
            ))
    }

    /// Set the CoreContext (called by core_initialize)
    pub async fn set(&self, ctx: CoreContext) {
        *self.inner.write().await = Some(ctx);
    }

    /// Check if initialized
    pub async fn is_initialized(&self) -> bool {
        self.inner.read().await.is_some()
    }

    /// Clear the CoreContext (called on shutdown)
    pub async fn clear(&self) {
        *self.inner.write().await = None;
    }
}

impl Default for CoreState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_core_state_lifecycle() {
        let state = CoreState::new();
        
        // Initially not initialized
        assert!(!state.is_initialized().await);
        assert!(state.get().await.is_err());
        
        // After initialization
        // Note: We can't easily create a CoreContext in tests without full setup,
        // so we'll just test the structure is correct
    }
}
