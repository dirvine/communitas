//! Focus management system for keyboard navigation
//!
//! This module provides centralized focus management for the Communitas TUI,
//! enabling intuitive keyboard navigation across all interactive components.
//!
//! # Features
//!
//! - **Component Registration**: Register components in desired focus order
//! - **Keyboard Navigation**: Tab/Shift+Tab with automatic wrap-around
//! - **Selective Focus**: Enable/disable components dynamically
//! - **Modal Support**: Focus stack for proper modal layer management
//! - **Event System**: Subscribe to focus change notifications
//!
//! # Example
//!
//! ```rust
//! use communitas_tui::backend::FocusManager;
//! use communitas_tui::messages::ComponentId;
//!
//! // Create focus manager
//! let mut focus = FocusManager::new();
//!
//! // Register components in desired order
//! focus.register(ComponentId::MessageInput, true);
//! focus.register(ComponentId::MessageList, true);
//! focus.register(ComponentId::OrganizationList, true);
//!
//! // Navigate with Tab
//! focus.focus_next(); // MessageInput -> MessageList
//! focus.focus_next(); // MessageList -> OrganizationList
//! focus.focus_next(); // OrganizationList -> MessageInput (wraps)
//!
//! // Navigate backwards with Shift+Tab
//! focus.focus_previous(); // MessageInput -> OrganizationList
//!
//! // Modal workflow
//! focus.push_focus(); // Save current focus
//! focus.focus(ComponentId::ModalButton);
//! // ... modal closes ...
//! focus.pop_focus(); // Restore previous focus
//!
//! // Subscribe to changes
//! focus.subscribe(|change| {
//!     println!("Focus changed from {:?} to {:?}",
//!              change.previous, change.current);
//! });
//! ```
//!
//! # Architecture
//!
//! The `FocusManager` maintains:
//! - A circular queue of focusable components (VecDeque)
//! - Enabled/disabled state per component (HashMap)
//! - Focus stack for modal layer management (Vec)
//! - Subscriber callbacks for focus change events

use crate::messages::ComponentId;
use std::collections::{HashMap, VecDeque};

/// Focus change event emitted when focus transitions between components
///
/// This event is sent to all subscribers when the focused component changes,
/// allowing UI updates, state tracking, and accessibility features.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusChange {
    /// Previously focused component (None if no component was focused)
    pub previous: Option<ComponentId>,
    /// Newly focused component (None if focus was cleared)
    pub current: Option<ComponentId>,
}

/// Central focus manager for keyboard navigation
///
/// Manages focus state across all interactive components in the TUI,
/// providing Tab/Shift+Tab cycling, modal layer support, and focus
/// change notifications.
///
/// # Focus Order
///
/// Components are focused in the order they are registered via [`register`](Self::register).
/// The focus order forms a circular queue, so pressing Tab on the last component
/// wraps to the first component.
///
/// # Modal Support
///
/// Use [`push_focus`](Self::push_focus) when opening a modal to save the current
/// focus state, then [`pop_focus`](Self::pop_focus) when closing to restore it.
/// This ensures users return to exactly where they were before the modal opened.
///
/// # Event Subscription
///
/// Subscribe to focus changes via [`subscribe`](Self::subscribe) to update
/// UI state, trigger accessibility announcements, or log user navigation patterns.
pub struct FocusManager {
    /// Currently focused component
    current_focus: Option<ComponentId>,
    /// Ordered list of focusable components
    focusable: VecDeque<ComponentId>,
    /// Whether each component is enabled for focusing
    enabled: HashMap<ComponentId, bool>,
    /// Focus stack for modal dialogs (stores previous focus when modal opens)
    focus_stack: Vec<Option<ComponentId>>,
    /// Subscribers to focus change events
    subscribers: Vec<Box<dyn Fn(&FocusChange) + Send>>,
}

impl std::fmt::Debug for FocusManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusManager")
            .field("current_focus", &self.current_focus)
            .field("focusable", &self.focusable)
            .field("enabled", &self.enabled)
            .field("focus_stack", &self.focus_stack)
            .field("subscribers_count", &self.subscribers.len())
            .finish()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusManager {
    /// Create a new focus manager
    pub fn new() -> Self {
        Self {
            current_focus: None,
            focusable: VecDeque::new(),
            enabled: HashMap::new(),
            focus_stack: Vec::new(),
            subscribers: Vec::new(),
        }
    }

    /// Register a component as focusable
    ///
    /// # Arguments
    /// * `component_id` - The component to register
    /// * `enabled` - Whether the component is initially enabled for focusing
    pub fn register(&mut self, component_id: ComponentId, enabled: bool) {
        if !self.focusable.contains(&component_id) {
            self.focusable.push_back(component_id.clone());
        }
        self.enabled.insert(component_id, enabled);
    }

    /// Unregister a component from focus management
    pub fn unregister(&mut self, component_id: &ComponentId) {
        self.focusable.retain(|id| id != component_id);
        self.enabled.remove(component_id);

        // Clear focus if the removed component was focused
        if self.current_focus.as_ref() == Some(component_id) {
            self.clear_focus();
        }
    }

    /// Enable or disable a component for focusing
    pub fn set_enabled(&mut self, component_id: &ComponentId, enabled: bool) {
        if let Some(state) = self.enabled.get_mut(component_id) {
            *state = enabled;

            // If currently focused component is being disabled, clear focus
            if !enabled && self.current_focus.as_ref() == Some(component_id) {
                self.clear_focus();
            }
        }
    }

    /// Check if a component is enabled for focusing
    pub fn is_enabled(&self, component_id: &ComponentId) -> bool {
        self.enabled.get(component_id).copied().unwrap_or(false)
    }

    /// Focus a specific component
    pub fn focus(&mut self, component_id: ComponentId) -> bool {
        // Check if component is registered and enabled
        if !self.enabled.get(&component_id).copied().unwrap_or(false) {
            return false;
        }

        let previous = self.current_focus.clone();
        self.current_focus = Some(component_id.clone());

        // Notify subscribers
        self.notify_focus_change(FocusChange {
            previous,
            current: Some(component_id),
        });

        true
    }

    /// Clear focus (no component focused)
    pub fn clear_focus(&mut self) {
        let previous = self.current_focus.take();

        if previous.is_some() {
            self.notify_focus_change(FocusChange {
                previous,
                current: None,
            });
        }
    }

    /// Get currently focused component
    pub fn current_focus(&self) -> Option<&ComponentId> {
        self.current_focus.as_ref()
    }

    /// Focus next component in the focus order
    pub fn focus_next(&mut self) -> bool {
        if self.focusable.is_empty() {
            return false;
        }

        // Find current position
        let current_pos = self.current_focus.as_ref()
            .and_then(|id| self.focusable.iter().position(|fid| fid == id));

        // Find next enabled component
        let start_pos = current_pos.map(|p| p + 1).unwrap_or(0);

        for i in 0..self.focusable.len() {
            let idx = (start_pos + i) % self.focusable.len();
            let component_id = &self.focusable[idx];

            if self.is_enabled(component_id) {
                return self.focus(component_id.clone());
            }
        }

        false
    }

    /// Focus previous component in the focus order
    pub fn focus_previous(&mut self) -> bool {
        if self.focusable.is_empty() {
            return false;
        }

        // Find current position
        let current_pos = self.current_focus.as_ref()
            .and_then(|id| self.focusable.iter().position(|fid| fid == id));

        // Find previous enabled component (search backwards)
        // Start from one position before current, or from end if no current
        let len = self.focusable.len();
        let start_pos = match current_pos {
            Some(pos) => (pos + len - 1) % len,  // Go back one with wrap
            None => len - 1,  // Start from end
        };

        for i in 0..len {
            let idx = (start_pos + len - i) % len;
            let component_id = &self.focusable[idx];

            if self.is_enabled(component_id) {
                return self.focus(component_id.clone());
            }
        }

        false
    }

    /// Push current focus to stack (for modals)
    pub fn push_focus(&mut self) {
        self.focus_stack.push(self.current_focus.clone());
    }

    /// Pop focus from stack (when modal closes)
    pub fn pop_focus(&mut self) -> bool {
        if let Some(previous_focus) = self.focus_stack.pop() {
            if let Some(component_id) = previous_focus {
                return self.focus(component_id);
            } else {
                self.clear_focus();
                return true;
            }
        }
        false
    }

    /// Get focus stack depth
    pub fn focus_stack_depth(&self) -> usize {
        self.focus_stack.len()
    }

    /// Subscribe to focus change events
    pub fn subscribe<F>(&mut self, callback: F)
    where
        F: Fn(&FocusChange) + Send + 'static,
    {
        self.subscribers.push(Box::new(callback));
    }

    /// Get count of registered focusable components
    pub fn focusable_count(&self) -> usize {
        self.focusable.len()
    }

    /// Get count of enabled focusable components
    pub fn enabled_count(&self) -> usize {
        self.focusable.iter()
            .filter(|id| self.is_enabled(id))
            .count()
    }

    /// Notify all subscribers of a focus change
    fn notify_focus_change(&self, change: FocusChange) {
        for subscriber in &self.subscribers {
            subscriber(&change);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // === Creation Tests ===

    #[test]
    fn test_focus_manager_creation() {
        let manager = FocusManager::new();
        assert_eq!(manager.current_focus(), None);
        assert_eq!(manager.focusable_count(), 0);
        assert_eq!(manager.enabled_count(), 0);
        assert_eq!(manager.focus_stack_depth(), 0);
    }

    #[test]
    fn test_focus_manager_default() {
        let manager = FocusManager::default();
        // Verify default state matches new()
        assert_eq!(manager.current_focus(), None);
        assert_eq!(manager.focusable_count(), 0);
        assert_eq!(manager.enabled_count(), 0);
        assert_eq!(manager.focus_stack_depth(), 0);
    }

    // === Registration Tests ===

    #[test]
    fn test_register_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        assert_eq!(manager.focusable_count(), 1);
        assert_eq!(manager.enabled_count(), 1);
        assert!(manager.is_enabled(&ComponentId::MessageInput));
    }

    #[test]
    fn test_register_disabled_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, false);

        assert_eq!(manager.focusable_count(), 1);
        assert_eq!(manager.enabled_count(), 0);
        assert!(!manager.is_enabled(&ComponentId::MessageInput));
    }

    #[test]
    fn test_register_duplicate_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageInput, true);

        assert_eq!(manager.focusable_count(), 1); // Should not duplicate
    }

    #[test]
    fn test_register_multiple_components() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);
        manager.register(ComponentId::OrganizationList, false);

        assert_eq!(manager.focusable_count(), 3);
        assert_eq!(manager.enabled_count(), 2);
    }

    // === Unregister Tests ===

    #[test]
    fn test_unregister_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.unregister(&ComponentId::MessageInput);

        assert_eq!(manager.focusable_count(), 0);
        assert!(!manager.is_enabled(&ComponentId::MessageInput));
    }

    #[test]
    fn test_unregister_clears_focus() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.focus(ComponentId::MessageInput);

        assert!(manager.current_focus().is_some());

        manager.unregister(&ComponentId::MessageInput);
        assert!(manager.current_focus().is_none());
    }

    // === Enable/Disable Tests ===

    #[test]
    fn test_set_enabled() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, false);

        manager.set_enabled(&ComponentId::MessageInput, true);
        assert!(manager.is_enabled(&ComponentId::MessageInput));
        assert_eq!(manager.enabled_count(), 1);
    }

    #[test]
    fn test_set_disabled() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        manager.set_enabled(&ComponentId::MessageInput, false);
        assert!(!manager.is_enabled(&ComponentId::MessageInput));
        assert_eq!(manager.enabled_count(), 0);
    }

    #[test]
    fn test_disable_clears_focus() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.focus(ComponentId::MessageInput);

        manager.set_enabled(&ComponentId::MessageInput, false);
        assert!(manager.current_focus().is_none());
    }

    // === Focus Tests ===

    #[test]
    fn test_focus_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        let success = manager.focus(ComponentId::MessageInput);
        assert!(success);
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
    }

    #[test]
    fn test_focus_disabled_component_fails() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, false);

        let success = manager.focus(ComponentId::MessageInput);
        assert!(!success);
        assert!(manager.current_focus().is_none());
    }

    #[test]
    fn test_focus_unregistered_component_fails() {
        let mut manager = FocusManager::new();

        let success = manager.focus(ComponentId::MessageInput);
        assert!(!success);
        assert!(manager.current_focus().is_none());
    }

    #[test]
    fn test_clear_focus() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.focus(ComponentId::MessageInput);

        manager.clear_focus();
        assert!(manager.current_focus().is_none());
    }

    // === Focus Navigation Tests ===

    #[test]
    fn test_focus_next_single_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        let success = manager.focus_next();
        assert!(success);
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
    }

    #[test]
    fn test_focus_next_multiple_components() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);
        manager.register(ComponentId::OrganizationList, true);

        manager.focus_next();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));

        manager.focus_next();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageList));

        manager.focus_next();
        assert_eq!(manager.current_focus(), Some(&ComponentId::OrganizationList));
    }

    #[test]
    fn test_focus_next_wraps_around() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);

        manager.focus_next(); // MessageInput
        manager.focus_next(); // MessageList
        manager.focus_next(); // Wraps to MessageInput

        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
    }

    #[test]
    fn test_focus_next_skips_disabled() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, false);
        manager.register(ComponentId::OrganizationList, true);

        manager.focus_next();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));

        manager.focus_next();
        assert_eq!(manager.current_focus(), Some(&ComponentId::OrganizationList));
    }

    #[test]
    fn test_focus_previous_single_component() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        let success = manager.focus_previous();
        assert!(success);
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
    }

    #[test]
    fn test_focus_previous_multiple_components() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);
        manager.register(ComponentId::OrganizationList, true);

        manager.focus(ComponentId::OrganizationList);

        manager.focus_previous();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageList));

        manager.focus_previous();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
    }

    #[test]
    fn test_focus_previous_wraps_around() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);

        manager.focus(ComponentId::MessageInput);
        manager.focus_previous(); // Wraps to MessageList

        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageList));
    }

    // === Focus Stack Tests ===

    #[test]
    fn test_push_focus() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.focus(ComponentId::MessageInput);

        manager.push_focus();
        assert_eq!(manager.focus_stack_depth(), 1);
    }

    #[test]
    fn test_pop_focus() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);

        manager.focus(ComponentId::MessageInput);
        manager.push_focus();
        manager.focus(ComponentId::MessageList);

        let success = manager.pop_focus();
        assert!(success);
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
        assert_eq!(manager.focus_stack_depth(), 0);
    }

    #[test]
    fn test_pop_focus_empty_stack() {
        let mut manager = FocusManager::new();

        let success = manager.pop_focus();
        assert!(!success);
    }

    #[test]
    fn test_push_pop_multiple_times() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);
        manager.register(ComponentId::MessageList, true);
        manager.register(ComponentId::OrganizationList, true);

        manager.focus(ComponentId::MessageInput);
        manager.push_focus();

        manager.focus(ComponentId::MessageList);
        manager.push_focus();

        manager.focus(ComponentId::OrganizationList);

        assert_eq!(manager.focus_stack_depth(), 2);

        manager.pop_focus();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageList));

        manager.pop_focus();
        assert_eq!(manager.current_focus(), Some(&ComponentId::MessageInput));
    }

    // === Subscription Tests ===

    #[test]
    fn test_subscribe_to_focus_changes() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        let changes = Arc::new(Mutex::new(Vec::new()));
        let changes_clone = changes.clone();

        manager.subscribe(move |change| {
            changes_clone.lock().unwrap().push(change.clone());
        });

        manager.focus(ComponentId::MessageInput);

        let recorded = changes.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].previous, None);
        assert_eq!(recorded[0].current, Some(ComponentId::MessageInput));
    }

    #[test]
    fn test_multiple_subscribers() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, true);

        let count1 = Arc::new(Mutex::new(0));
        let count2 = Arc::new(Mutex::new(0));

        let c1 = count1.clone();
        let c2 = count2.clone();

        manager.subscribe(move |_| {
            *c1.lock().unwrap() += 1;
        });

        manager.subscribe(move |_| {
            *c2.lock().unwrap() += 1;
        });

        manager.focus(ComponentId::MessageInput);

        assert_eq!(*count1.lock().unwrap(), 1);
        assert_eq!(*count2.lock().unwrap(), 1);
    }

    // === Edge Cases ===

    #[test]
    fn test_focus_next_empty_list() {
        let mut manager = FocusManager::new();

        let success = manager.focus_next();
        assert!(!success);
    }

    #[test]
    fn test_focus_next_all_disabled() {
        let mut manager = FocusManager::new();
        manager.register(ComponentId::MessageInput, false);
        manager.register(ComponentId::MessageList, false);

        let success = manager.focus_next();
        assert!(!success);
    }

    #[test]
    fn test_is_enabled_unregistered_component() {
        let manager = FocusManager::new();
        assert!(!manager.is_enabled(&ComponentId::MessageInput));
    }
}
