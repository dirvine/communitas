//! Onboarding experience module for new users.
//!
//! Provides the data model and tour content for guiding new users
//! through Communitas features on first launch.

pub mod steps;
pub mod tour_overlay;
pub mod types;

pub use steps::{
    CallsStep, CanvasStep, DriveStep, HelpStep, KanbanStep, MessagingStep, SettingsStep,
    StepContent, WelcomeStep,
};
pub use tour_overlay::{
    SpotlightCutout, SpotlightCutoutProps, TourOverlay, TourOverlayProps, TourTooltip,
    TourTooltipProps, finish_tour, next_step, previous_step, skip_tour, use_tour_state,
};
pub use types::{TOUR_STEPS, TooltipPosition, TourState, TourStep};
