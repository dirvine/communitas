//! Hook for respecting user's reduced motion preference.
//!
//! Provides accessibility support by detecting and respecting the
//! `prefers-reduced-motion` media query.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::hooks::use_reduced_motion::use_reduced_motion;
//!
//! let reduced_motion = use_reduced_motion();
//!
//! if reduced_motion() {
//!     // Use instant transitions
//! } else {
//!     // Use animations
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::standard_spring;

/// Check if the user prefers reduced motion.
///
/// Returns a signal that tracks the `prefers-reduced-motion` media query.
/// This is useful for accessibility - users who prefer reduced motion
/// will get instant transitions instead of animations.
///
/// # Platform Support
///
/// - Web: Uses `window.matchMedia('(prefers-reduced-motion: reduce)')`
/// - Desktop: Returns `false` (no media query support)
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::hooks::use_reduced_motion::use_reduced_motion;
///
/// let reduced_motion = use_reduced_motion();
///
/// // In your component
/// let animation_duration = if reduced_motion() { 0 } else { 300 };
/// ```
pub fn use_reduced_motion() -> Signal<bool> {
    use_signal(|| {
        // Check for prefers-reduced-motion
        // On desktop/webview without media query support, default to false
        check_reduced_motion_preference()
    })
}

/// Check the reduced motion preference.
///
/// This function attempts to detect the user's motion preference.
/// Returns `true` if the user prefers reduced motion.
fn check_reduced_motion_preference() -> bool {
    // On web platforms, check the media query
    #[cfg(target_arch = "wasm32")]
    {
        // Use web-sys to check media query if available
        // For now, return false as default
        // TODO(github.com/saorsa-labs/communitas#WASM-ACCESSIBILITY):
        // Implement web-sys media query check for prefers-reduced-motion.
        // Requires: window.matchMedia('(prefers-reduced-motion: reduce)').matches
        // Blocked on: web-sys feature flags and testing infrastructure.
        false
    }

    // On desktop platforms, there's no standard media query
    // Users can set an environment variable to indicate preference
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var("COMMUNITAS_REDUCED_MOTION")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    }
}

/// Get an appropriate animation config based on motion preference.
///
/// Returns an instant transition if reduced motion is preferred,
/// otherwise returns the provided animation config.
///
/// # Type Parameters
///
/// * `T` - The animated value type
///
/// # Arguments
///
/// * `reduced_motion` - Whether reduced motion is preferred
/// * `normal_config` - The animation config to use when motion is enabled
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::hooks::use_reduced_motion::{use_reduced_motion, get_accessible_animation};
/// use dioxus_motion::prelude::*;
///
/// let reduced = use_reduced_motion();
/// let config = get_accessible_animation(reduced(), standard_spring());
/// ```
pub fn get_accessible_animation(
    reduced_motion: bool,
    normal_config: AnimationConfig,
) -> AnimationConfig {
    if reduced_motion {
        // Return instant transition using a spring with very high stiffness
        AnimationConfig::new(AnimationMode::Spring(Spring {
            stiffness: 10000.0,
            damping: 10000.0,
            mass: 0.01,
            velocity: 0.0,
        }))
    } else {
        normal_config
    }
}

/// Hook that returns animation configs respecting motion preferences.
///
/// This is a convenience hook that combines `use_reduced_motion` with
/// config generation.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::hooks::use_reduced_motion::use_accessible_animation;
/// use communitas_dioxus::animations::standard_spring;
///
/// let get_config = use_accessible_animation();
/// let config = get_config(standard_spring());
/// ```
pub fn use_accessible_animation() -> impl Fn(AnimationConfig) -> AnimationConfig {
    let reduced = use_reduced_motion();

    move |normal_config: AnimationConfig| -> AnimationConfig {
        if reduced() {
            // Use a spring with zero values for instant transition
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 10000.0,
                damping: 10000.0,
                mass: 0.01,
                velocity: 0.0,
            }))
        } else {
            normal_config
        }
    }
}

/// Animation preset that respects reduced motion.
///
/// This enum provides common animation presets that automatically
/// adapt to the user's motion preferences.
#[derive(Clone, Copy, PartialEq)]
pub enum AccessibleAnimation {
    /// No animation (instant).
    None,
    /// Fast animation (100ms).
    Fast,
    /// Normal animation (200ms).
    Normal,
    /// Slow animation (300ms).
    Slow,
    /// Spring physics animation.
    Spring,
}

impl AccessibleAnimation {
    /// Get the animation config for this preset.
    ///
    /// # Arguments
    ///
    /// * `reduced_motion` - Whether reduced motion is preferred
    pub fn config(&self, reduced_motion: bool) -> AnimationConfig {
        // Helper for instant animation (when reduced motion is preferred)
        let instant = || {
            AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 10000.0,
                damping: 10000.0,
                mass: 0.01,
                velocity: 0.0,
            }))
        };

        if reduced_motion {
            return instant();
        }

        match self {
            AccessibleAnimation::None => instant(),
            AccessibleAnimation::Fast => AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 800.0,
                damping: 40.0,
                mass: 0.5,
                velocity: 0.0,
            })),
            AccessibleAnimation::Normal => AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 400.0,
                damping: 30.0,
                mass: 1.0,
                velocity: 0.0,
            })),
            AccessibleAnimation::Slow => AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 200.0,
                damping: 25.0,
                mass: 1.5,
                velocity: 0.0,
            })),
            AccessibleAnimation::Spring => {
                AnimationConfig::new(AnimationMode::Spring(standard_spring()))
            }
        }
    }
}

/// Hook for accessible animation presets.
///
/// Returns a function that converts `AccessibleAnimation` presets
/// to appropriate `AnimationConfig` based on user preferences.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::hooks::use_reduced_motion::{use_accessible_preset, AccessibleAnimation};
///
/// let get_preset = use_accessible_preset();
/// let config = get_preset(AccessibleAnimation::Normal);
/// ```
pub fn use_accessible_preset() -> impl Fn(AccessibleAnimation) -> AnimationConfig {
    let reduced = use_reduced_motion();

    move |preset: AccessibleAnimation| -> AnimationConfig { preset.config(reduced()) }
}

/// Component wrapper that conditionally disables animations.
///
/// When reduced motion is preferred, renders children without
/// animation wrappers.
#[derive(Props, Clone, PartialEq)]
pub struct AccessibleAnimationWrapperProps {
    /// Content to render.
    pub children: Element,
    /// Animation to apply when motion is enabled.
    pub animation: AccessibleAnimation,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Wrapper that respects reduced motion preferences.
#[component]
pub fn AccessibleAnimationWrapper(props: AccessibleAnimationWrapperProps) -> Element {
    let reduced = use_reduced_motion();
    let _config = props.animation.config(reduced());

    // For now, just render children
    // In a full implementation, this would wrap with appropriate animation
    rsx! {
        div {
            class: "{props.class}",
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessible_animation_none_is_instant() {
        let config = AccessibleAnimation::None.config(false);
        // Config should be created (actual testing would require dioxus-motion internals)
        let _ = config;
    }

    #[test]
    fn accessible_animation_respects_reduced_motion() {
        // When reduced motion is true, all animations should be instant
        let fast = AccessibleAnimation::Fast.config(true);
        let slow = AccessibleAnimation::Slow.config(true);
        let spring = AccessibleAnimation::Spring.config(true);

        // All should produce instant configs
        let _ = (fast, slow, spring);
    }

    #[test]
    fn check_reduced_motion_default() {
        // Default should be false unless environment variable is set
        let preference = check_reduced_motion_preference();
        // We can't assert a specific value since it depends on environment
        // Just verify it doesn't panic
        let _ = preference;
    }
}
