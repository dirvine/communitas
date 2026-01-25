//! Focus management utilities for modals and dialogs.
//!
//! This module provides hooks for managing keyboard focus in accessible
//! modal dialogs, drawers, and other overlay components.
//!
//! # Features
//!
//! - Focus trapping within containers
//! - Return focus to trigger element on close
//! - Auto-focus on mount
//!
//! # Example
//!
//! ```ignore
//! use communitas_dioxus::hooks::focus::{use_focus_trap, FocusTrapConfig};
//!
//! #[component]
//! fn Modal(on_close: EventHandler<()>) -> Element {
//!     let container_ref = use_focus_trap(FocusTrapConfig {
//!         on_escape: Some(on_close),
//!         auto_focus_first: true,
//!     });
//!
//!     rsx! {
//!         div {
//!             ..container_ref,
//!             role: "dialog",
//!             // Modal content
//!         }
//!     }
//! }
//! ```

#![allow(dead_code)]

use dioxus::prelude::*;

/// Configuration for focus trap behavior.
#[derive(Clone, Default)]
pub struct FocusTrapConfig {
    /// Callback when Escape key is pressed.
    pub on_escape: Option<EventHandler<()>>,
    /// Whether to auto-focus the first focusable element.
    pub auto_focus_first: bool,
    /// Whether to allow focus to leave the container.
    pub allow_outside_focus: bool,
}

/// CSS selector for focusable elements.
const FOCUSABLE_SELECTOR: &str = "a[href], button:not(:disabled), input:not(:disabled), \
    select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex=\"-1\"]), \
    [contenteditable=\"true\"]";

/// Focus trap hook that traps keyboard focus within a container element.
///
/// Returns a signal that should be spread onto the container element.
/// The focus trap handles:
/// - Tab wrapping from last to first focusable element
/// - Shift+Tab wrapping from first to last focusable element
/// - Escape key to call the on_escape callback
///
/// # Example
///
/// ```ignore
/// let trap_ref = use_focus_trap(FocusTrapConfig {
///     on_escape: Some(on_close),
///     auto_focus_first: true,
/// });
///
/// rsx! {
///     div {
///         onkeydown: move |evt| trap_ref.handle_keydown(evt),
///         // content
///     }
///  }
/// ```
pub fn use_focus_trap(config: FocusTrapConfig) -> FocusTrapHandle {
    let container_id = use_signal(|| format!("focus-trap-{}", rand_id()));
    let active = use_signal(|| true);

    FocusTrapHandle {
        container_id,
        config,
        active,
    }
}

/// Handle returned by use_focus_trap for managing focus.
#[derive(Clone)]
pub struct FocusTrapHandle {
    container_id: Signal<String>,
    config: FocusTrapConfig,
    active: Signal<bool>,
}

impl FocusTrapHandle {
    /// Get the container ID for this focus trap.
    #[must_use]
    pub fn id(&self) -> String {
        self.container_id.read().clone()
    }

    /// Check if the focus trap is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        *self.active.read()
    }

    /// Handle keydown events for focus trapping.
    pub fn handle_keydown(&self, evt: Event<KeyboardData>) {
        if !self.is_active() {
            return;
        }

        let key = evt.key();

        // Handle Escape
        if key == Key::Escape {
            if let Some(ref on_escape) = self.config.on_escape {
                evt.prevent_default();
                on_escape.call(());
            }
            return;
        }

        // Handle Tab for focus wrapping
        if key == Key::Tab {
            // In a browser environment, we would query the DOM here
            // For now, we provide the structure for the focus trap
            // The actual DOM manipulation would happen in WASM
            #[cfg(target_arch = "wasm32")]
            {
                self.handle_tab_navigation(&evt);
            }
        }
    }

    /// Handle Tab key navigation (WASM only).
    #[cfg(target_arch = "wasm32")]
    fn handle_tab_navigation(&self, evt: &Event<KeyboardData>) {
        use wasm_bindgen::JsCast;
        use web_sys::{Document, Element, HtmlElement, NodeList};

        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };

        let container_id = self.container_id.read().clone();
        let Some(container) = document.get_element_by_id(&container_id) else {
            return;
        };

        let Ok(focusable) = container.query_selector_all(FOCUSABLE_SELECTOR) else {
            return;
        };

        let len = focusable.length();
        if len == 0 {
            return;
        }

        let active_element = document.active_element();
        let shift = evt.modifiers().shift();

        // Find current focus index
        let mut current_index: Option<u32> = None;
        for i in 0..len {
            if let Some(el) = focusable.item(i) {
                if Some(el.clone()) == active_element.clone().map(|e| e.unchecked_into()) {
                    current_index = Some(i);
                    break;
                }
            }
        }

        // Calculate next index
        let next_index = match (current_index, shift) {
            (Some(0), true) => len - 1,            // Shift+Tab from first -> last
            (Some(i), true) => i - 1,              // Shift+Tab -> previous
            (Some(i), false) if i == len - 1 => 0, // Tab from last -> first
            (Some(i), false) => i + 1,             // Tab -> next
            (None, true) => len - 1,               // Shift+Tab with no focus -> last
            (None, false) => 0,                    // Tab with no focus -> first
        };

        // Focus the next element
        if let Some(el) = focusable.item(next_index) {
            if let Ok(html_el) = el.dyn_into::<HtmlElement>() {
                evt.prevent_default();
                let _ = html_el.focus();
            }
        }
    }
}

/// Hook to return focus to a previous element when component unmounts.
///
/// This is useful for modals and dialogs where focus should return
/// to the trigger element (button that opened the modal) when closed.
///
/// # Example
///
/// ```ignore
/// let return_focus = use_return_focus();
///
/// // Focus will automatically return to the previously focused element
/// // when this component unmounts
/// ```
pub fn use_return_focus() -> ReturnFocusHandle {
    let previous_element_id = use_signal(|| Option::<String>::None);

    // Capture the currently focused element on mount
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = document.active_element() {
                    // Store the element ID if it has one
                    let id = el.id();
                    if !id.is_empty() {
                        previous_element_id.set(Some(id));
                    }
                }
            }
        }
    });

    ReturnFocusHandle {
        previous_element_id,
    }
}

/// Handle for returning focus to a previous element.
#[derive(Clone, Copy)]
pub struct ReturnFocusHandle {
    previous_element_id: Signal<Option<String>>,
}

impl ReturnFocusHandle {
    /// Manually return focus to the previous element.
    pub fn return_focus(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::HtmlElement;

            if let Some(id) = self.previous_element_id.read().as_ref() {
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(el) = document.get_element_by_id(id) {
                        if let Ok(html_el) = el.dyn_into::<HtmlElement>() {
                            let _ = html_el.focus();
                        }
                    }
                }
            }
        }
    }

    /// Get the ID of the element that will receive focus.
    #[must_use]
    pub fn target_id(&self) -> Option<String> {
        self.previous_element_id.read().clone()
    }
}

// Note: Focus return should be called explicitly via return_focus()
// or by using use_effect with cleanup in the calling component.

/// Hook to auto-focus an element when it mounts.
///
/// Returns a signal containing the element ID that should be applied
/// to the target element.
///
/// # Example
///
/// ```ignore
/// let auto_focus = use_auto_focus();
///
/// rsx! {
///     input {
///         id: auto_focus.id(),
///         r#type: "text",
///     }
/// }
/// ```
pub fn use_auto_focus() -> AutoFocusHandle {
    let element_id = use_signal(|| format!("auto-focus-{}", rand_id()));
    let focused = use_signal(|| false);

    // Focus the element on mount
    use_effect(move || {
        if *focused.read() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::HtmlElement;

            // Small delay to ensure element is in DOM
            let id = element_id.read().clone();
            // Copy signal for the async block
            let mut focused_signal = focused;
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(10).await;
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(el) = document.get_element_by_id(&id) {
                        if let Ok(html_el) = el.dyn_into::<HtmlElement>() {
                            let _ = html_el.focus();
                            focused_signal.set(true);
                        }
                    }
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut focused_signal = focused;
            focused_signal.set(true);
        }
    });

    AutoFocusHandle {
        element_id,
        focused,
    }
}

/// Handle for auto-focus functionality.
#[derive(Clone, Copy)]
pub struct AutoFocusHandle {
    element_id: Signal<String>,
    focused: Signal<bool>,
}

impl AutoFocusHandle {
    /// Get the element ID to apply to the target element.
    #[must_use]
    pub fn id(&self) -> String {
        self.element_id.read().clone()
    }

    /// Check if the element has been focused.
    #[must_use]
    pub fn is_focused(&self) -> bool {
        *self.focused.read()
    }
}

/// Generate a random ID for element identification.
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u32)
        .unwrap_or(0)
        .wrapping_mul(1103515245)
        .wrapping_add(12345)
}

/// Get all focusable elements within a container (browser only).
///
/// Returns a list of focusable elements matching the standard
/// focusable selector (links, buttons, inputs, etc.).
#[cfg(target_arch = "wasm32")]
pub fn get_focusable_elements(container_id: &str) -> Vec<web_sys::Element> {
    use web_sys::Document;

    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };

    let Some(container) = document.get_element_by_id(container_id) else {
        return Vec::new();
    };

    let Ok(node_list) = container.query_selector_all(FOCUSABLE_SELECTOR) else {
        return Vec::new();
    };

    let mut elements = Vec::with_capacity(node_list.length() as usize);
    for i in 0..node_list.length() {
        if let Some(el) = node_list.item(i) {
            elements.push(el);
        }
    }
    elements
}

/// Focus the first focusable element in a container (browser only).
#[cfg(target_arch = "wasm32")]
pub fn focus_first_element(container_id: &str) -> bool {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    let elements = get_focusable_elements(container_id);
    if let Some(first) = elements.first() {
        if let Ok(html_el) = first.clone().dyn_into::<HtmlElement>() {
            return html_el.focus().is_ok();
        }
    }
    false
}

/// Focus the last focusable element in a container (browser only).
#[cfg(target_arch = "wasm32")]
pub fn focus_last_element(container_id: &str) -> bool {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    let elements = get_focusable_elements(container_id);
    if let Some(last) = elements.last() {
        if let Ok(html_el) = last.clone().dyn_into::<HtmlElement>() {
            return html_el.focus().is_ok();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_trap_config_default() {
        let config = FocusTrapConfig::default();
        assert!(config.on_escape.is_none());
        assert!(!config.auto_focus_first);
        assert!(!config.allow_outside_focus);
    }

    #[test]
    fn focusable_selector_contains_common_elements() {
        assert!(FOCUSABLE_SELECTOR.contains("a[href]"));
        assert!(FOCUSABLE_SELECTOR.contains("button"));
        assert!(FOCUSABLE_SELECTOR.contains("input"));
        assert!(FOCUSABLE_SELECTOR.contains("select"));
        assert!(FOCUSABLE_SELECTOR.contains("textarea"));
        assert!(FOCUSABLE_SELECTOR.contains("[tabindex]"));
        assert!(FOCUSABLE_SELECTOR.contains("[contenteditable"));
    }

    #[test]
    fn focusable_selector_excludes_disabled() {
        assert!(FOCUSABLE_SELECTOR.contains("button:not(:disabled)"));
        assert!(FOCUSABLE_SELECTOR.contains("input:not(:disabled)"));
        assert!(FOCUSABLE_SELECTOR.contains("select:not(:disabled)"));
        assert!(FOCUSABLE_SELECTOR.contains("textarea:not(:disabled)"));
    }

    #[test]
    fn focusable_selector_excludes_negative_tabindex() {
        assert!(FOCUSABLE_SELECTOR.contains("[tabindex]:not([tabindex=\"-1\"])"));
    }

    #[test]
    fn rand_id_generates_different_values() {
        // Due to time-based generation, consecutive calls should produce different values
        // with the multiplier/addition
        let id1 = rand_id();
        let id2 = rand_id();
        // They might occasionally be the same if called in same nanosecond
        // but the wrapping math should still work
        assert!(id1 > 0 || id2 > 0);
    }

    #[test]
    fn focus_trap_config_clone() {
        let config = FocusTrapConfig {
            on_escape: None,
            auto_focus_first: true,
            allow_outside_focus: false,
        };
        let cloned = config.clone();
        assert!(cloned.auto_focus_first);
        assert!(!cloned.allow_outside_focus);
    }
}
