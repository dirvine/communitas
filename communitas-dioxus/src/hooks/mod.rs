//! Reusable hooks for Communitas Dioxus application.
//!
//! This module provides custom hooks for common UI patterns.
//!
//! # Available Hooks
//!
//! - [`focus`] - Focus management utilities for modals and dialogs

pub mod focus;

// Re-export focus utilities
pub use focus::{FocusTrapConfig, use_auto_focus, use_focus_trap, use_return_focus};
