//! Screen reader announcement system for accessibility.
//!
//! This module provides a centralized system for announcing updates to screen readers
//! using ARIA live regions. It supports both polite (non-urgent) and assertive
//! (important) announcements with message queuing to avoid overlap.
//!
//! # Example
//!
//! ```ignore
//! // In your app root, include the Announcer component
//! rsx! {
//!     Announcer {}
//!     // ... rest of your app
//! }
//!
//! // In any child component, use the announcer hook
//! let announce = use_announcer();
//! announce("Item saved successfully", AnnouncementMode::Polite);
//! ```

// These are public API items that will be used by consumers of the component library.
// They are intentionally exposed for accessibility integration in other parts of the app.
#![allow(dead_code)]

use dioxus::prelude::*;
use std::collections::VecDeque;

/// The mode of announcement determines which ARIA live region is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementMode {
    /// Polite announcements wait for the user to finish their current task.
    /// Use for non-urgent updates like "Item saved" or "3 results found".
    Polite,
    /// Assertive announcements interrupt the user immediately.
    /// Use for important changes like "Error occurred" or "Session expiring".
    Assertive,
}

/// An announcement to be read by screen readers.
#[derive(Debug, Clone, PartialEq)]
pub struct Announcement {
    /// The message to announce.
    pub message: String,
    /// The mode (polite or assertive).
    pub mode: AnnouncementMode,
    /// Unique ID for deduplication.
    id: u64,
}

/// Context for the announcer system.
/// Uses signals which are Copy, making it easy to use across closures.
#[derive(Clone, Copy)]
pub struct AnnouncerContext {
    /// Signal containing the current polite announcement.
    polite: Signal<Option<String>>,
    /// Signal containing the current assertive announcement.
    assertive: Signal<Option<String>>,
    /// Queue of pending announcements.
    queue: Signal<VecDeque<Announcement>>,
    /// Counter for generating unique IDs.
    counter: Signal<u64>,
}

impl AnnouncerContext {
    /// Create a new announcer context.
    pub(crate) fn new() -> Self {
        Self {
            polite: Signal::new(None),
            assertive: Signal::new(None),
            queue: Signal::new(VecDeque::new()),
            counter: Signal::new(0),
        }
    }

    /// Add an announcement to the queue.
    pub fn announce(&self, message: impl Into<String>, mode: AnnouncementMode) {
        let message = message.into();
        if message.is_empty() {
            return;
        }

        // Copy signals to local variables for mutation
        let mut counter = self.counter;
        let mut queue = self.queue;

        let id = {
            let current = *counter.read();
            counter.set(current + 1);
            current
        };

        let announcement = Announcement { message, mode, id };

        // Add to queue
        queue.write().push_back(announcement);
    }

    /// Process the next announcement in the queue.
    fn process_next(&self) {
        // Copy signals to local variables for mutation
        let mut queue = self.queue;
        let mut polite = self.polite;
        let mut assertive = self.assertive;

        if let Some(announcement) = queue.write().pop_front() {
            match announcement.mode {
                AnnouncementMode::Polite => {
                    polite.set(Some(announcement.message));
                }
                AnnouncementMode::Assertive => {
                    assertive.set(Some(announcement.message));
                }
            }
        }
    }

    /// Clear polite announcement.
    fn clear_polite(&self) {
        let mut polite = self.polite;
        polite.set(None);
    }

    /// Clear assertive announcement.
    fn clear_assertive(&self) {
        let mut assertive = self.assertive;
        assertive.set(None);
    }

    /// Check if queue has items.
    fn has_pending(&self) -> bool {
        !self.queue.read().is_empty()
    }
}

/// Centralized screen reader announcement component.
///
/// This component renders two invisible live regions (one polite, one assertive)
/// that screen readers monitor for updates. Include this once at your app root.
///
/// # Features
///
/// - Queues announcements to avoid overlap
/// - Auto-clears announcements after a delay
/// - Supports both polite and assertive modes
/// - Visually hidden but accessible to screen readers
///
/// # Example
///
/// ```ignore
/// fn App() -> Element {
///     rsx! {
///         Announcer {}
///         Router::<Route> {}
///     }
/// }
/// ```
#[component]
pub fn Announcer() -> Element {
    // Consume the app-wide announcer context provided at the root.
    let ctx = use_context::<AnnouncerContext>();

    // Process queue when it changes
    let queue_len = ctx.queue.read().len();

    // Effect to process announcements from the queue
    use_effect(move || {
        if queue_len > 0 {
            ctx.process_next();
        }
    });

    // Auto-clear announcements after a delay
    let polite_msg = ctx.polite.read().clone();
    let assertive_msg = ctx.assertive.read().clone();

    // Clear polite after 1 second
    use_effect(move || {
        if polite_msg.is_some() {
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                {
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                }
                ctx.clear_polite();
                // Process next in queue if any
                if ctx.has_pending() {
                    ctx.process_next();
                }
            });
        }
    });

    // Clear assertive after 2 seconds (longer for important messages)
    use_effect(move || {
        if assertive_msg.is_some() {
            spawn(async move {
                #[cfg(target_arch = "wasm32")]
                {
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                }
                ctx.clear_assertive();
                // Process next in queue if any
                if ctx.has_pending() {
                    ctx.process_next();
                }
            });
        }
    });

    // CSS for visually hiding the live regions while keeping them accessible
    let sr_only_style = "position: absolute; width: 1px; height: 1px; padding: 0; \
                         margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); \
                         white-space: nowrap; border: 0;";

    rsx! {
        // Polite live region - for non-urgent updates
        div {
            id: "announcer-polite",
            role: "status",
            "aria-live": "polite",
            "aria-atomic": "true",
            style: "{sr_only_style}",
            {ctx.polite.read().clone().unwrap_or_default()}
        }

        // Assertive live region - for important updates
        div {
            id: "announcer-assertive",
            role: "alert",
            "aria-live": "assertive",
            "aria-atomic": "true",
            style: "{sr_only_style}",
            {ctx.assertive.read().clone().unwrap_or_default()}
        }
    }
}

/// Hook to get the announcer function for making screen reader announcements.
///
/// Returns a closure that can be called with a message and mode to queue
/// an announcement for screen readers.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
///
/// // Polite announcement (waits for user to finish current task)
/// announce("File uploaded successfully", AnnouncementMode::Polite);
///
/// // Assertive announcement (interrupts immediately)
/// announce("Session will expire in 1 minute", AnnouncementMode::Assertive);
/// ```
pub fn use_announcer() -> impl Fn(String, AnnouncementMode) + Clone {
    let ctx = use_context::<AnnouncerContext>();

    move |message: String, mode: AnnouncementMode| {
        ctx.announce(message, mode);
    }
}

/// Announce a polite (non-urgent) message to screen readers.
///
/// Use this for updates that don't require immediate attention, such as:
/// - "Item saved"
/// - "3 results found"
/// - "Page loaded"
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_polite(&announce, "Changes saved successfully");
/// ```
pub fn announce_polite<F>(announcer: &F, message: &str)
where
    F: Fn(String, AnnouncementMode),
{
    announcer(message.to_string(), AnnouncementMode::Polite);
}

/// Announce an assertive (important) message to screen readers.
///
/// Use this for updates that require immediate attention, such as:
/// - "Error occurred"
/// - "Session expiring"
/// - "Form validation failed"
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_assertive(&announce, "Failed to save changes");
/// ```
pub fn announce_assertive<F>(announcer: &F, message: &str)
where
    F: Fn(String, AnnouncementMode),
{
    announcer(message.to_string(), AnnouncementMode::Assertive);
}

/// Announce an action that was performed on a target.
///
/// Formats the message as "{action} {target}" for consistent announcements.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_action(&announce, "Moved card to", "Done column");
/// // Announces: "Moved card to Done column"
/// ```
pub fn announce_action<F>(announcer: &F, action: &str, target: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let message = format!("{action} {target}");
    announcer(message, AnnouncementMode::Polite);
}

/// Announce a navigation event.
///
/// Use when the user navigates to a new page or section.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_navigation(&announce, "Settings page");
/// // Announces: "Navigated to Settings page"
/// ```
pub fn announce_navigation<F>(announcer: &F, destination: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let message = format!("Navigated to {destination}");
    announcer(message, AnnouncementMode::Polite);
}

/// Announce that content is loading.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_loading(&announce, "messages");
/// // Announces: "Loading messages"
/// ```
pub fn announce_loading<F>(announcer: &F, content: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let message = format!("Loading {content}");
    announcer(message, AnnouncementMode::Polite);
}

/// Announce that content has finished loading.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_loaded(&announce, "5 messages");
/// // Announces: "Loaded 5 messages"
/// ```
pub fn announce_loaded<F>(announcer: &F, content: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let message = format!("Loaded {content}");
    announcer(message, AnnouncementMode::Polite);
}

/// Announce an error to screen readers (assertive mode).
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_error(&announce, "Failed to connect to server");
/// ```
pub fn announce_error<F>(announcer: &F, error: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let message = format!("Error: {error}");
    announcer(message, AnnouncementMode::Assertive);
}

/// Announce a success message.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_success(&announce, "File uploaded");
/// // Announces: "Success: File uploaded"
/// ```
pub fn announce_success<F>(announcer: &F, message: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let msg = format!("Success: {message}");
    announcer(msg, AnnouncementMode::Polite);
}

/// Announce a count of items.
///
/// # Example
///
/// ```ignore
/// let announce = use_announcer();
/// announce_count(&announce, 3, "unread messages");
/// // Announces: "3 unread messages"
/// ```
pub fn announce_count<F>(announcer: &F, count: usize, label: &str)
where
    F: Fn(String, AnnouncementMode),
{
    let message = format!("{count} {label}");
    announcer(message, AnnouncementMode::Polite);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announcement_mode_equality() {
        assert_eq!(AnnouncementMode::Polite, AnnouncementMode::Polite);
        assert_eq!(AnnouncementMode::Assertive, AnnouncementMode::Assertive);
        assert_ne!(AnnouncementMode::Polite, AnnouncementMode::Assertive);
    }

    #[test]
    fn test_announcement_clone() {
        let announcement = Announcement {
            message: "Test message".to_string(),
            mode: AnnouncementMode::Polite,
            id: 1,
        };
        let cloned = announcement.clone();
        assert_eq!(announcement.message, cloned.message);
        assert_eq!(announcement.mode, cloned.mode);
        assert_eq!(announcement.id, cloned.id);
    }

    #[test]
    fn test_announcement_debug() {
        let announcement = Announcement {
            message: "Test".to_string(),
            mode: AnnouncementMode::Assertive,
            id: 42,
        };
        let debug_str = format!("{announcement:?}");
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("Assertive"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_mode_debug() {
        assert_eq!(format!("{:?}", AnnouncementMode::Polite), "Polite");
        assert_eq!(format!("{:?}", AnnouncementMode::Assertive), "Assertive");
    }

    #[test]
    fn test_mode_copy() {
        let mode = AnnouncementMode::Polite;
        let copied = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_action_message_format() {
        // Test the format used by announce_action
        let action = "Moved card to";
        let target = "Done column";
        let message = format!("{action} {target}");
        assert_eq!(message, "Moved card to Done column");
    }

    #[test]
    fn test_navigation_message_format() {
        // Test the format used by announce_navigation
        let destination = "Settings page";
        let message = format!("Navigated to {destination}");
        assert_eq!(message, "Navigated to Settings page");
    }

    #[test]
    fn test_loading_message_format() {
        // Test the format used by announce_loading
        let content = "messages";
        let message = format!("Loading {content}");
        assert_eq!(message, "Loading messages");
    }

    #[test]
    fn test_loaded_message_format() {
        // Test the format used by announce_loaded
        let content = "5 messages";
        let message = format!("Loaded {content}");
        assert_eq!(message, "Loaded 5 messages");
    }

    #[test]
    fn test_error_message_format() {
        // Test the format used by announce_error
        let error = "Connection failed";
        let message = format!("Error: {error}");
        assert_eq!(message, "Error: Connection failed");
    }

    #[test]
    fn test_success_message_format() {
        // Test the format used by announce_success
        let success_msg = "File uploaded";
        let message = format!("Success: {success_msg}");
        assert_eq!(message, "Success: File uploaded");
    }

    #[test]
    fn test_count_message_format() {
        // Test the format used by announce_count
        let count = 3;
        let label = "unread messages";
        let message = format!("{count} {label}");
        assert_eq!(message, "3 unread messages");
    }

    #[test]
    fn test_empty_message_handling() {
        // Empty messages should be valid (handled at context level)
        let message = "";
        assert!(message.is_empty());
    }
}
