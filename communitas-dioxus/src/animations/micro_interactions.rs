//! Micro-interaction utilities for delightful UI feedback.
//!
//! Provides reusable animation patterns for common interactions like
//! button presses, hover effects, and state changes.
#![allow(dead_code)]

use dioxus::prelude::*;
use dioxus_motion::prelude::*;
use std::time::Duration;

use super::transitions;
use crate::animations::gentle_spring;

async fn async_delay(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::future::TimeoutFuture;
        let millis = duration.as_millis().min(u128::from(u32::MAX)) as u32;
        TimeoutFuture::new(millis).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(duration).await;
    }
}

/// Animation state for a pressable element.
#[derive(Clone, Copy, PartialEq)]
pub enum PressState {
    /// Default resting state.
    Idle,
    /// Mouse/touch is pressing down.
    Pressing,
    /// Hovering without pressing.
    Hovering,
}

/// Use press animation for interactive elements.
///
/// Returns a signal with the current scale value and handlers for
/// press events.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_press_animation;
///
/// let (scale, press_handlers) = use_press_animation();
/// ```
pub fn use_press_animation() -> (Signal<f32>, PressHandlers) {
    let mut scale = use_motion(1.0f32);

    let on_press_start = Callback::new(move |_| {
        scale.animate_to(0.95, transitions::button_press());
    });

    let on_press_end = Callback::new(move |_| {
        scale.animate_to(1.0, transitions::button_release());
    });

    let on_hover_start = Callback::new(move |_| {
        scale.animate_to(1.02, transitions::card_hover());
    });

    let on_hover_end = Callback::new(move |_| {
        scale.animate_to(1.0, transitions::card_hover());
    });

    (
        use_signal(move || scale.get_value()),
        PressHandlers {
            on_press_start,
            on_press_end,
            on_hover_start,
            on_hover_end,
        },
    )
}

/// Handlers for press animations.
pub struct PressHandlers {
    /// Call when press starts (mousedown/touchstart).
    pub on_press_start: Callback<(), ()>,
    /// Call when press ends (mouseup/touchend).
    pub on_press_end: Callback<(), ()>,
    /// Call when hover starts.
    pub on_hover_start: Callback<(), ()>,
    /// Call when hover ends.
    pub on_hover_end: Callback<(), ()>,
}

/// Use elevation animation for cards and elevated surfaces.
///
/// Returns a signal with the current elevation value (in pixels).
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_elevation;
///
/// let (elevation, elevation_handlers) = use_elevation(0.0, 8.0);
/// ```
pub fn use_elevation(default: f32, hover: f32) -> (Signal<f32>, ElevationHandlers) {
    let mut elevation = use_motion(default);

    let on_hover_start = Callback::new(move |_| {
        elevation.animate_to(hover, transitions::card_hover());
    });

    let on_hover_end = Callback::new(move |_| {
        elevation.animate_to(default, transitions::card_hover());
    });

    let on_press = Callback::new(move |_| {
        elevation.animate_to(default, transitions::card_press());
    });

    (
        use_signal(move || elevation.get_value()),
        ElevationHandlers {
            on_hover_start,
            on_hover_end,
            on_press,
        },
    )
}

/// Handlers for elevation animations.
pub struct ElevationHandlers {
    /// Call when hover starts.
    pub on_hover_start: Callback<(), ()>,
    /// Call when hover ends.
    pub on_hover_end: Callback<(), ()>,
    /// Call when pressed.
    pub on_press: Callback<(), ()>,
}

/// Use focus animation for input fields.
///
/// Returns signals for border opacity and glow intensity.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_focus_animation;
///
/// let (border_opacity, glow_opacity, focus_handlers) = use_focus_animation();
/// ```
pub fn use_focus_animation() -> (Signal<f32>, Signal<f32>, FocusHandlers) {
    let mut border_opacity = use_motion(0.0f32);
    let mut glow_opacity = use_motion(0.0f32);

    let on_focus = Callback::new(move |_| {
        border_opacity.animate_to(1.0, transitions::input_focus());
        glow_opacity.animate_to(1.0, transitions::input_focus());
    });

    let on_blur = Callback::new(move |_| {
        border_opacity.animate_to(0.0, transitions::input_blur());
        glow_opacity.animate_to(0.0, transitions::input_blur());
    });

    (
        use_signal(move || border_opacity.get_value()),
        use_signal(move || glow_opacity.get_value()),
        FocusHandlers { on_focus, on_blur },
    )
}

/// Handlers for focus animations.
pub struct FocusHandlers {
    /// Call when element receives focus.
    pub on_focus: Callback<(), ()>,
    /// Call when element loses focus.
    pub on_blur: Callback<(), ()>,
}

/// Use toggle animation for switches and checkboxes.
///
/// Returns a signal with the current position (0.0 to 1.0).
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_toggle;
///
/// let (position, toggle) = use_toggle(false);
/// ```
pub fn use_toggle(initial: bool) -> (Signal<f32>, impl FnMut()) {
    let mut position = use_motion(if initial { 1.0f32 } else { 0.0f32 });
    let mut is_on = use_signal(|| initial);

    let toggle = move || {
        let new_state = !is_on();
        is_on.set(new_state);
        let target = if new_state { 1.0f32 } else { 0.0f32 };
        position.animate_to(target, transitions::switch_toggle());
    };

    (use_signal(move || position.get_value()), toggle)
}

/// Use shake animation for error feedback.
///
/// Returns a signal with the current x offset and a trigger function.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_shake;
///
/// let (offset, shake) = use_shake();
///
/// // Trigger shake on error
/// shake();
/// ```
pub fn use_shake() -> (Signal<f32>, impl FnMut()) {
    let mut offset = use_motion(0.0f32);

    let shake = move || {
        // Perform a shake by animating back and forth
        spawn(async move {
            offset.animate_to(-10.0, transitions::error_shake());
            offset.animate_to(10.0, transitions::error_shake());
            offset.animate_to(-10.0, transitions::error_shake());
            offset.animate_to(0.0, transitions::error_shake());
        });
    };

    (use_signal(move || offset.get_value()), shake)
}

/// Use pulse animation for attention-grabbing elements.
///
/// Returns a signal with the current scale value.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_pulse;
///
/// let scale = use_pulse(1.0, 1.1, 1000);
/// ```
pub fn use_pulse(min: f32, max: f32, duration_ms: u64) -> Signal<f32> {
    let scale = use_motion(min);
    let duration_ms = duration_ms.max(200);
    let half_cycle = Duration::from_millis((duration_ms / 2).max(16));

    use_future(move || {
        let mut scale = scale;
        async move {
            loop {
                scale.animate_to(
                    max,
                    AnimationConfig::new(AnimationMode::Spring(gentle_spring())),
                );
                async_delay(half_cycle).await;
                scale.animate_to(
                    min,
                    AnimationConfig::new(AnimationMode::Spring(gentle_spring())),
                );
                async_delay(half_cycle).await;
            }
        }
    });

    use_signal(move || scale.get_value())
}

/// Use entrance animation for elements appearing on screen.
///
/// Automatically animates from initial values to target values on mount.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_entrance;
///
/// let (opacity, translate_y) = use_entrance(0.0, 20.0, 1.0, 0.0);
/// ```
pub fn use_entrance(
    initial_opacity: f32,
    initial_translate_y: f32,
    target_opacity: f32,
    target_translate_y: f32,
) -> (Signal<f32>, Signal<f32>) {
    let mut opacity = use_motion(initial_opacity);
    let mut translate_y = use_motion(initial_translate_y);

    use_effect(move || {
        opacity.animate_to(target_opacity, transitions::page_fade_in());
        translate_y.animate_to(target_translate_y, transitions::slide_up());
    });

    (
        use_signal(move || opacity.get_value()),
        use_signal(move || translate_y.get_value()),
    )
}

/// Use staggered entrance for list items.
///
/// Returns signals for opacity and translate_y with built-in delay.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::micro_interactions::use_staggered_entrance;
///
/// let (opacity, translate_y) = use_staggered_entrance(index, 50.0);
/// ```
pub fn use_staggered_entrance(index: usize, stagger_ms: f32) -> (Signal<f32>, Signal<f32>) {
    let opacity = use_motion(0.0f32);
    let translate_y = use_motion(20.0f32);

    use_future(move || {
        let mut opacity = opacity;
        let mut translate_y = translate_y;
        async move {
            let delay_ms = (index as f32 * stagger_ms).max(0.0) as u64;
            async_delay(Duration::from_millis(delay_ms)).await;
            opacity.animate_to(1.0, transitions::list_item_enter());
            translate_y.animate_to(0.0, transitions::list_item_enter());
        }
    });

    (
        use_signal(move || opacity.get_value()),
        use_signal(move || translate_y.get_value()),
    )
}

/// Calculate spring duration estimate for sequencing animations.
///
/// Returns approximate milliseconds for the spring to settle.
pub fn spring_duration_estimate(spring: Spring, threshold: f32) -> f32 {
    // Simplified estimation based on spring physics
    let damping_ratio = spring.damping / (2.0 * (spring.stiffness * spring.mass).sqrt());
    let natural_frequency = (spring.stiffness / spring.mass).sqrt();

    if damping_ratio >= 1.0 {
        // Overdamped - slower
        1000.0 / natural_frequency * 3.0
    } else {
        // Underdamped - oscillates
        let settle_time = -threshold.ln() / (damping_ratio * natural_frequency);
        settle_time * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animations::standard_spring;

    #[test]
    fn spring_duration_is_positive() {
        let spring = standard_spring();
        let duration = spring_duration_estimate(spring, 0.01);
        assert!(duration > 0.0);
    }
}
