//! Transition presets for common animation patterns.
//!
//! Provides pre-configured AnimationConfig instances for consistent
//! motion across the application.
#![allow(dead_code)]

use super::{gentle_spring, quick_spring, smooth_spring, snappy_spring, springs, standard_spring};
use dioxus_motion::prelude::*;

/// Duration values in milliseconds for tween animations.
pub mod duration {
    /// Instant state changes.
    pub const INSTANT: f32 = 0.0;
    /// Micro-interactions (100ms).
    pub const FAST: f32 = 100.0;
    /// Standard transitions (200ms).
    pub const NORMAL: f32 = 200.0;
    /// Emphasis animations (300ms).
    pub const SLOW: f32 = 300.0;
    /// Dramatic effects (500ms).
    pub const SLOWER: f32 = 500.0;
    /// Page transitions (350ms).
    pub const PAGE: f32 = 350.0;
}

/// Page transition: Slide from right (navigation forward).
///
/// Use when navigating to a new page deeper in the hierarchy.
pub fn page_slide_in_right() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::page_transition()))
}

/// Page transition: Slide to right (navigation back).
///
/// Use when returning to a previous page.
pub fn page_slide_out_right() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(snappy_spring()))
}

/// Page transition: Fade in.
///
/// Subtle entrance for pages without directional context.
pub fn page_fade_in() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(smooth_spring()))
}

/// Overlay fade in for modals and dialogs.
///
/// Backdrop appearance animation.
pub fn overlay_fade_in() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(gentle_spring()))
}

/// Overlay fade out.
///
/// Backdrop dismissal animation.
pub fn overlay_fade_out() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(quick_spring()))
}

/// Modal content appearance.
///
/// Scale and fade in for modal dialogs.
pub fn modal_appear() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::modal_appear()))
}

/// Modal content dismissal.
///
/// Scale and fade out for modal dialogs.
pub fn modal_dismiss() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::modal_dismiss()))
}

/// Popover scale animation.
///
/// Entrance for dropdowns and popovers.
pub fn popover_appear() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::dropdown_appear()))
}

/// Popover dismissal.
pub fn popover_dismiss() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(quick_spring()))
}

/// Toast notification entrance.
pub fn toast_enter() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::toast_enter()))
}

/// Toast notification exit.
pub fn toast_exit() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::toast_exit()))
}

/// List item stagger entrance.
///
/// Use with delay based on item index.
pub fn list_item_enter() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
}

/// Button press feedback.
pub fn button_press() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::button_press()))
}

/// Button release.
pub fn button_release() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::button_release()))
}

/// Card hover elevation.
pub fn card_hover() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::card_hover()))
}

/// Card press feedback.
pub fn card_press() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::card_press()))
}

/// Input focus state.
pub fn input_focus() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::input_focus()))
}

/// Input blur state.
pub fn input_blur() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(gentle_spring()))
}

/// Sidebar expand/collapse.
pub fn sidebar_toggle() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::sidebar_toggle()))
}

/// Shimmer effect for skeletons.
pub fn shimmer() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(Spring {
        stiffness: 100.0,
        damping: 20.0,
        mass: 2.0,
        velocity: 0.0,
    }))
}

/// Error shake animation.
pub fn error_shake() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::error_shake()))
}

/// Success bounce animation.
pub fn success_bounce() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::success_bounce()))
}

/// Tooltip appearance.
pub fn tooltip_appear() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::tooltip_appear()))
}

/// Tooltip dismissal.
pub fn tooltip_dismiss() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(quick_spring()))
}

/// Switch toggle animation.
pub fn switch_toggle() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::switch_toggle()))
}

/// Quick fade for subtle state changes.
pub fn quick_fade() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(quick_spring()))
}

/// Standard fade for most transitions.
pub fn standard_fade() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(standard_spring()))
}

/// Slow fade for emphasis.
pub fn slow_fade() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(smooth_spring()))
}

/// Scale up animation.
pub fn scale_up() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::success_bounce()))
}

/// Scale down animation.
pub fn scale_down() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(gentle_spring()))
}

/// Slide up entrance.
pub fn slide_up() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
}

/// Slide down entrance.
pub fn slide_down() -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
}

/// Create a delayed animation config.
///
/// Adds a delay before the animation starts.
///
/// # Arguments
///
/// * `config` - Base animation config
/// * `delay_ms` - Delay in milliseconds
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::transitions::{list_item_enter, with_delay};
///
/// let config = with_delay(list_item_enter(), 100.0);
/// ```
use std::time::Duration;

pub fn with_delay(config: AnimationConfig, delay_ms: f32) -> AnimationConfig {
    config.with_delay(Duration::from_millis(delay_ms as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_are_positive() {
        // Verify duration constants are positive values
        // These are compile-time constants, so we verify their values directly
        const _: () = assert!(duration::FAST > 0.0);
        const _: () = assert!(duration::NORMAL > 0.0);
        const _: () = assert!(duration::SLOW > 0.0);
    }

    #[test]
    fn transition_configs_are_created() {
        let _ = page_fade_in();
        let _ = overlay_fade_in();
        let _ = modal_appear();
        let _ = toast_enter();
    }
}
