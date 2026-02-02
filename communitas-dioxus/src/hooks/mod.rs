//! Reusable hooks for Communitas Dioxus application.
//!
//! This module provides custom hooks for common UI patterns.
//!
//! # Available Hooks
//!
//! - [`focus`] - Focus management utilities for modals and dialogs
//! - [`use_categorized_entities`] - Efficient entity categorization from directory snapshots
//! - [`use_reduced_motion()`] - Accessibility hook for respecting motion preferences

pub mod focus;
pub mod use_categorized_entities;
pub mod use_reduced_motion;

// Re-export focus utilities
pub use focus::{FocusTrapConfig, use_auto_focus, use_focus_trap, use_return_focus};

// Re-export entity categorization
pub use use_categorized_entities::CategorizedEntities;

// Re-export reduced motion utilities
pub use use_reduced_motion::{
    AccessibleAnimation, AccessibleAnimationWrapper, use_accessible_animation,
    use_accessible_preset, use_reduced_motion,
};
