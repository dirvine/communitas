//! Onboarding tour types and data structures.
//!
//! Defines the data model for the new-user onboarding experience,
//! including tour steps, state tracking, and the default tour content.

/// Represents the position where a tooltip should be displayed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TooltipPosition {
    /// Display tooltip above the target element.
    Top,
    /// Display tooltip below the target element (default).
    #[default]
    Bottom,
    /// Display tooltip to the left of the target element.
    Left,
    /// Display tooltip to the right of the target element.
    Right,
    /// Display tooltip centered on screen (no specific target).
    Center,
}

/// A single step in the onboarding tour.
///
/// Each step represents one screen/overlay in the guided tour,
/// with content to display and an optional target element to highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TourStep {
    /// Unique identifier for this step (e.g., "welcome", "messaging").
    pub id: &'static str,
    /// Display title for this step.
    pub title: &'static str,
    /// Description text explaining this feature.
    pub description: &'static str,
    /// Optional CSS selector for the target element to highlight.
    /// When `None`, the step is shown as a centered modal.
    pub target_selector: Option<&'static str>,
    /// Position where the tooltip should appear relative to the target.
    pub position: TooltipPosition,
}

/// The current state of the onboarding tour.
///
/// Tracks where the user is in the tour and whether they've completed or skipped it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TourState {
    /// Index of the currently active step (0-based).
    pub current_step: usize,
    /// Total number of steps in the tour.
    pub total_steps: usize,
    /// Whether the tour is currently active/visible.
    pub active: bool,
    /// Whether the user has skipped the tour.
    pub skipped: bool,
}

impl TourState {
    /// Create a new tour state for the given steps.
    #[must_use]
    pub fn new(total_steps: usize) -> Self {
        Self {
            current_step: 0,
            total_steps,
            active: false,
            skipped: false,
        }
    }

    /// Start the tour from the beginning.
    pub fn start(&mut self) {
        self.current_step = 0;
        self.active = true;
        self.skipped = false;
    }

    /// Advance to the next step.
    ///
    /// Returns `true` if there are more steps, `false` if the tour is complete.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.current_step + 1 < self.total_steps {
            self.current_step += 1;
            true
        } else {
            self.active = false;
            false
        }
    }

    /// Go back to the previous step.
    ///
    /// Returns `true` if successful, `false` if already at the first step.
    pub fn previous(&mut self) -> bool {
        if self.current_step > 0 {
            self.current_step -= 1;
            true
        } else {
            false
        }
    }

    /// Skip the tour entirely.
    pub fn skip(&mut self) {
        self.active = false;
        self.skipped = true;
    }

    /// Complete the tour.
    pub fn complete(&mut self) {
        self.active = false;
        self.skipped = false;
    }

    /// Check if the tour is on the last step.
    #[must_use]
    pub fn is_last_step(&self) -> bool {
        self.current_step + 1 >= self.total_steps
    }

    /// Check if the tour is on the first step.
    #[must_use]
    pub fn is_first_step(&self) -> bool {
        self.current_step == 0
    }

    /// Get the progress as a percentage (0-100).
    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        if self.total_steps == 0 {
            return 100;
        }
        let progress = (self.current_step as f64 / self.total_steps as f64) * 100.0;
        progress.round() as u8
    }
}

/// The default onboarding tour steps for new users.
///
/// This constant defines the complete tour experience, introducing
/// users to each major feature of Communitas.
pub const TOUR_STEPS: &[TourStep] = &[
    TourStep {
        id: "welcome",
        title: "Welcome to Communitas",
        description: "A decentralized collaboration platform that puts you in control. \
                      Your data stays on your devices, encrypted and private. \
                      Let's take a quick tour of the main features.",
        target_selector: None,
        position: TooltipPosition::Center,
    },
    TourStep {
        id: "identity",
        title: "Your Connection Address",
        description: "Share your four-word connection address to let others find you on the network. \
                      This tells others WHERE to connect, not WHO you are. \
                      Your actual identity is your cryptographic public key.",
        target_selector: Some("[data-tour='identity']"),
        position: TooltipPosition::Bottom,
    },
    TourStep {
        id: "messaging",
        title: "Messaging",
        description: "End-to-end encrypted messaging with groups and channels. \
                      All your conversations are private and sync across your devices. \
                      Create channels for teams or direct message individuals.",
        target_selector: Some("[data-tour='messaging']"),
        position: TooltipPosition::Right,
    },
    TourStep {
        id: "drive",
        title: "Your Drive",
        description: "Virtual disks for organizing your files. \
                      Keep files private, share with specific people, or publish publicly. \
                      Everything is encrypted and stored locally first.",
        target_selector: Some("[data-tour='drive']"),
        position: TooltipPosition::Right,
    },
    TourStep {
        id: "canvas",
        title: "Canvas Collaboration",
        description: "Real-time collaborative editing and drawing. \
                      Perfect for brainstorming, diagramming, and creating together. \
                      Changes sync instantly with collaborators.",
        target_selector: Some("[data-tour='canvas']"),
        position: TooltipPosition::Right,
    },
    TourStep {
        id: "kanban",
        title: "Project Management",
        description: "Organize your work with Kanban boards. \
                      Track progress, assign tasks, and collaborate with your team. \
                      Boards sync in real-time across all participants.",
        target_selector: Some("[data-tour='kanban']"),
        position: TooltipPosition::Right,
    },
    TourStep {
        id: "calls",
        title: "Voice & Video Calls",
        description: "Connect with others via secure voice and video calls. \
                      All communications are encrypted end-to-end. \
                      Share your screen for presentations and collaboration.",
        target_selector: Some("[data-tour='calls']"),
        position: TooltipPosition::Right,
    },
    TourStep {
        id: "settings",
        title: "Settings & Help",
        description: "Customize your experience and manage your account. \
                      Access security settings, notification preferences, and more. \
                      Visit our help center if you have questions.",
        target_selector: Some("[data-tour='settings']"),
        position: TooltipPosition::Left,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tour_steps_count() {
        assert_eq!(TOUR_STEPS.len(), 8);
    }

    #[test]
    fn test_tour_steps_unique_ids() {
        let mut ids: Vec<&str> = TOUR_STEPS.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), TOUR_STEPS.len(), "All step IDs should be unique");
    }

    #[test]
    fn test_first_step_is_welcome() {
        assert_eq!(TOUR_STEPS[0].id, "welcome");
        assert!(TOUR_STEPS[0].target_selector.is_none());
        assert_eq!(TOUR_STEPS[0].position, TooltipPosition::Center);
    }

    #[test]
    fn test_tour_state_new() {
        let state = TourState::new(8);
        assert_eq!(state.current_step, 0);
        assert_eq!(state.total_steps, 8);
        assert!(!state.active);
        assert!(!state.skipped);
    }

    #[test]
    fn test_tour_state_start() {
        let mut state = TourState::new(8);
        state.start();
        assert!(state.active);
        assert_eq!(state.current_step, 0);
    }

    #[test]
    fn test_tour_state_next() {
        let mut state = TourState::new(3);
        state.start();

        assert!(state.next());
        assert_eq!(state.current_step, 1);

        assert!(state.next());
        assert_eq!(state.current_step, 2);

        assert!(!state.next());
        assert!(!state.active);
    }

    #[test]
    fn test_tour_state_previous() {
        let mut state = TourState::new(3);
        state.start();
        state.current_step = 2;

        assert!(state.previous());
        assert_eq!(state.current_step, 1);

        assert!(state.previous());
        assert_eq!(state.current_step, 0);

        assert!(!state.previous());
        assert_eq!(state.current_step, 0);
    }

    #[test]
    fn test_tour_state_skip() {
        let mut state = TourState::new(8);
        state.start();
        state.skip();

        assert!(!state.active);
        assert!(state.skipped);
    }

    #[test]
    fn test_tour_state_complete() {
        let mut state = TourState::new(8);
        state.start();
        state.complete();

        assert!(!state.active);
        assert!(!state.skipped);
    }

    #[test]
    fn test_tour_state_is_last_step() {
        let mut state = TourState::new(3);
        assert!(!state.is_last_step());

        state.current_step = 2;
        assert!(state.is_last_step());
    }

    #[test]
    fn test_tour_state_is_first_step() {
        let mut state = TourState::new(3);
        assert!(state.is_first_step());

        state.current_step = 1;
        assert!(!state.is_first_step());
    }

    #[test]
    fn test_tour_state_progress_percent() {
        let mut state = TourState::new(4);
        assert_eq!(state.progress_percent(), 0);

        state.current_step = 1;
        assert_eq!(state.progress_percent(), 25);

        state.current_step = 2;
        assert_eq!(state.progress_percent(), 50);

        state.current_step = 3;
        assert_eq!(state.progress_percent(), 75);
    }

    #[test]
    fn test_tour_state_progress_percent_empty() {
        let state = TourState::new(0);
        assert_eq!(state.progress_percent(), 100);
    }

    #[test]
    fn test_tooltip_position_default() {
        let pos = TooltipPosition::default();
        assert_eq!(pos, TooltipPosition::Bottom);
    }
}
