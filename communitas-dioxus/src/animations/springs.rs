//! Spring configuration presets for different animation contexts.
//!
//! Springs provide natural, physics-based motion that feels more organic
//! than linear tweens. Each preset is tuned for specific use cases.
#![allow(dead_code)]

use dioxus_motion::prelude::*;

/// Spring for button press feedback.
///
/// Quick response with slight overshoot for tactile feel.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::springs::button_press;
///
/// let spring = button_press();
/// ```
pub fn button_press() -> Spring {
    Spring {
        stiffness: 600.0,
        damping: 35.0,
        mass: 0.8,
        velocity: 0.0,
    }
}

/// Spring for button release.
///
/// Slightly bouncy return to rest state.
pub fn button_release() -> Spring {
    Spring {
        stiffness: 400.0,
        damping: 25.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Spring for card hover elevation.
///
/// Smooth, subtle lift effect.
pub fn card_hover() -> Spring {
    Spring {
        stiffness: 300.0,
        damping: 28.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Spring for card press.
///
/// Quick scale down for tactile feedback.
pub fn card_press() -> Spring {
    Spring {
        stiffness: 700.0,
        damping: 38.0,
        mass: 0.7,
        velocity: 0.0,
    }
}

/// Spring for page transitions.
///
/// Smooth, flowing motion between pages.
pub fn page_transition() -> Spring {
    Spring {
        stiffness: 250.0,
        damping: 28.0,
        mass: 1.2,
        velocity: 0.0,
    }
}

/// Spring for modal appearance.
///
/// Slight bounce for attention-grabbing entrance.
pub fn modal_appear() -> Spring {
    Spring {
        stiffness: 350.0,
        damping: 22.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Spring for modal dismissal.
///
/// Quick exit with minimal overshoot.
pub fn modal_dismiss() -> Spring {
    Spring {
        stiffness: 500.0,
        damping: 35.0,
        mass: 0.9,
        velocity: 0.0,
    }
}

/// Spring for list item entrance.
///
/// Gentle fade-in with slight upward motion.
pub fn list_item_enter() -> Spring {
    Spring {
        stiffness: 300.0,
        damping: 26.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Spring for toast notifications.
///
/// Playful entrance with quick settle.
pub fn toast_enter() -> Spring {
    Spring {
        stiffness: 400.0,
        damping: 20.0,
        mass: 0.9,
        velocity: 0.0,
    }
}

/// Spring for toast exit.
///
/// Quick slide out.
pub fn toast_exit() -> Spring {
    Spring {
        stiffness: 600.0,
        damping: 35.0,
        mass: 0.8,
        velocity: 0.0,
    }
}

/// Spring for sidebar collapse/expand.
///
/// Smooth width transition.
pub fn sidebar_toggle() -> Spring {
    Spring {
        stiffness: 280.0,
        damping: 30.0,
        mass: 1.1,
        velocity: 0.0,
    }
}

/// Spring for input focus.
///
/// Quick glow and border color transition.
pub fn input_focus() -> Spring {
    Spring {
        stiffness: 500.0,
        damping: 32.0,
        mass: 0.8,
        velocity: 0.0,
    }
}

/// Spring for shimmer effect.
///
/// Continuous smooth motion.
pub fn shimmer() -> Spring {
    Spring {
        stiffness: 100.0,
        damping: 20.0,
        mass: 2.0,
        velocity: 0.0,
    }
}

/// Spring for error shake.
///
/// Quick oscillation for error feedback.
pub fn error_shake() -> Spring {
    Spring {
        stiffness: 800.0,
        damping: 15.0,
        mass: 0.5,
        velocity: 0.0,
    }
}

/// Spring for success bounce.
///
/// Celebratory bounce for successful actions.
pub fn success_bounce() -> Spring {
    Spring {
        stiffness: 350.0,
        damping: 12.0,
        mass: 1.0,
        velocity: 0.0,
    }
}

/// Spring for switch toggle.
///
/// Snappy on/off transition.
pub fn switch_toggle() -> Spring {
    Spring {
        stiffness: 700.0,
        damping: 40.0,
        mass: 0.6,
        velocity: 0.0,
    }
}

/// Spring for dropdown appearance.
///
/// Quick scale and fade.
pub fn dropdown_appear() -> Spring {
    Spring {
        stiffness: 450.0,
        damping: 28.0,
        mass: 0.9,
        velocity: 0.0,
    }
}

/// Spring for tooltip appearance.
///
/// Subtle fade with slight scale.
pub fn tooltip_appear() -> Spring {
    Spring {
        stiffness: 400.0,
        damping: 30.0,
        mass: 0.8,
        velocity: 0.0,
    }
}

/// Create a custom spring with specified parameters.
///
/// # Arguments
///
/// * `stiffness` - Spring stiffness (higher = faster, more rigid)
/// * `damping` - Damping coefficient (higher = less overshoot)
/// * `mass` - Mass of the object (higher = slower movement)
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::animations::springs::custom;
///
/// let spring = custom(500.0, 25.0, 1.0);
/// ```
pub fn custom(stiffness: f32, damping: f32, mass: f32) -> Spring {
    Spring {
        stiffness,
        damping,
        mass,
        velocity: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_press_is_snappy() {
        let spring = button_press();
        assert!(spring.stiffness > 500.0);
        assert!(spring.mass < 1.0);
    }

    #[test]
    fn page_transition_is_smooth() {
        let spring = page_transition();
        assert!(spring.stiffness < 300.0);
        assert!(spring.mass > 1.0);
    }

    #[test]
    fn custom_spring_uses_parameters() {
        let spring = custom(100.0, 20.0, 0.5);
        assert_eq!(spring.stiffness, 100.0);
        assert_eq!(spring.damping, 20.0);
        assert_eq!(spring.mass, 0.5);
    }
}
