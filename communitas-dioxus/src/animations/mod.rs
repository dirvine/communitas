//! Animation system for Communitas Dioxus.
//!
//! Provides spring physics, tween animations, and micro-interactions
//! for a Telegram-quality user experience.
#![allow(dead_code)]
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::animations::{standard_spring, gentle_spring};
//! use dioxus_motion::prelude::*;
//!
//! // Animate with spring physics
//! let mut scale = use_motion(1.0f32);
//! scale.animate_to(1.1, standard_spring());
//! ```

pub mod micro_interactions;
pub mod springs;
pub mod transitions;

use dioxus_motion::prelude::*;

/// Initialize the animation system with high-performance settings.
///
/// Call this once at application startup for optimal animation performance.
pub fn init_animation_system() {
    // Resource pools are automatically initialized on first use
    // This function serves as a hook for future optimization
}

/// Standard spring for UI elements - bouncy but quick.
///
/// Good for general UI feedback like button presses and hover effects.
///
/// # Spring Parameters
/// - stiffness: 400.0 - Moderate resistance
/// - damping: 30.0 - Slight overshoot for bouncy feel
/// - mass: 1.0 - Standard weight
pub fn standard_spring() -> Spring {
    Spring {
        stiffness: 400.0,
        damping: 30.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Gentle spring for subtle movements.
///
/// Use for ambient animations and non-intrusive motion.
///
/// # Spring Parameters
/// - stiffness: 200.0 - Soft resistance
/// - damping: 25.0 - Minimal overshoot
/// - mass: 1.0 - Standard weight
pub fn gentle_spring() -> Spring {
    Spring {
        stiffness: 200.0,
        damping: 25.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Snappy spring for quick feedback.
///
/// Ideal for immediate response interactions like toggles and switches.
///
/// # Spring Parameters
/// - stiffness: 600.0 - High resistance for quick response
/// - damping: 35.0 - Controlled overshoot
/// - mass: 0.8 - Lighter for faster movement
pub fn snappy_spring() -> Spring {
    Spring {
        stiffness: 600.0,
        damping: 35.0,
        mass: 0.8,
        velocity: 0.0,
    }
}

/// Bouncy spring for playful interactions.
///
/// Use sparingly for celebratory moments or onboarding.
///
/// # Spring Parameters
/// - stiffness: 300.0 - Moderate resistance
/// - damping: 15.0 - High overshoot for bouncy effect
/// - mass: 1.0 - Standard weight
pub fn bouncy_spring() -> Spring {
    Spring {
        stiffness: 300.0,
        damping: 15.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Smooth spring for page transitions.
///
/// Provides elegant, flowing motion between states.
///
/// # Spring Parameters
/// - stiffness: 250.0 - Soft resistance
/// - damping: 28.0 - Balanced overshoot
/// - mass: 1.2 - Slightly heavier for smoother feel
pub fn smooth_spring() -> Spring {
    Spring {
        stiffness: 250.0,
        damping: 28.0,
        mass: 1.2,
        velocity: 0.0,
    }
}

/// Quick spring for micro-interactions.
///
/// Very fast response for subtle feedback.
///
/// # Spring Parameters
/// - stiffness: 800.0 - Very high resistance
/// - damping: 40.0 - Tight control
/// - mass: 0.5 - Light for speed
pub fn quick_spring() -> Spring {
    Spring {
        stiffness: 800.0,
        damping: 40.0,
        mass: 0.5,
        velocity: 0.0,
    }
}

/// Convert a Spring to an AnimationConfig for use with dioxus_motion.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::{standard_spring, spring_config};
///
/// let config = spring_config(standard_spring());
/// ```
pub fn spring_config(spring: Spring) -> AnimationConfig {
    AnimationConfig::new(AnimationMode::Spring(spring))
}

/// Animation duration constants in milliseconds.
pub mod duration {
    /// Instant - for immediate state changes.
    pub const INSTANT: f32 = 0.0;
    /// Fast - for micro-interactions.
    pub const FAST: f32 = 100.0;
    /// Normal - for standard transitions.
    pub const NORMAL: f32 = 200.0;
    /// Slow - for emphasis.
    pub const SLOW: f32 = 300.0;
    /// Slower - for dramatic effect.
    pub const SLOWER: f32 = 500.0;
    /// Page transition duration.
    pub const PAGE: f32 = 350.0;
    /// Stagger delay between list items.
    pub const STAGGER: f32 = 50.0;
}

/// Easing functions for tween animations.
pub mod easing {
    /// Linear interpolation.
    pub const LINEAR: &str = "linear";
    /// Default ease - smooth acceleration and deceleration.
    pub const EASE: &str = "ease";
    /// Ease in - accelerate from zero.
    pub const EASE_IN: &str = "ease-in";
    /// Ease out - decelerate to zero.
    pub const EASE_OUT: &str = "ease-out";
    /// Ease in-out - smooth both ways.
    pub const EASE_IN_OUT: &str = "ease-in-out";
    /// Bounce effect.
    pub const BOUNCE: &str = "cubic-bezier(0.34, 1.56, 0.64, 1)";
    /// Smooth deceleration.
    pub const SMOOTH: &str = "cubic-bezier(0.25, 0.1, 0.25, 1)";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_spring_has_correct_values() {
        let spring = standard_spring();
        assert_eq!(spring.stiffness, 400.0);
        assert_eq!(spring.damping, 30.0);
        assert_eq!(spring.mass, 1.0);
    }

    #[test]
    fn snappy_spring_is_lighter() {
        let spring = snappy_spring();
        assert!(spring.mass < 1.0);
        assert!(spring.stiffness > 500.0);
    }

    #[test]
    fn gentle_spring_is_softer() {
        let spring = gentle_spring();
        assert!(spring.stiffness < 300.0);
        assert!(spring.damping < 30.0);
    }

    #[test]
    fn spring_config_creates_animation_config() {
        let spring = standard_spring();
        let config = spring_config(spring);
        // Config should be created successfully
        // The actual type is opaque, so we just verify it compiles
        let _ = config;
    }
}
