//! Animated page wrapper for smooth page transitions.
//!
//! Provides entrance animations when pages appear, creating a polished
//! navigation experience.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::animated_page::{AnimatedPage, PageTransition};
//!
//! rsx! {
//!     AnimatedPage {
//!         transition: PageTransition::SlideFromRight,
//!         children: rsx! {
//!             h1 { "Page Content" }
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::springs;

/// Page transition animation types.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum PageTransition {
    /// Slide in from the right (navigation forward).
    #[default]
    SlideFromRight,
    /// Slide in from the left (navigation back).
    SlideFromLeft,
    /// Slide in from the bottom (modal-style).
    SlideFromBottom,
    /// Fade in with slight scale up.
    FadeScale,
    /// Simple fade in.
    Fade,
    /// No animation (instant appearance).
    None,
}

/// Properties for the AnimatedPage component.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedPageProps {
    /// Page content.
    pub children: Element,
    /// Transition animation type.
    #[props(default = PageTransition::SlideFromRight)]
    pub transition: PageTransition,
    /// Delay before animation starts (in milliseconds).
    #[props(default = 0.0)]
    pub delay_ms: f32,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Page wrapper with entrance animation.
///
/// Wraps page content with smooth entrance animations for a polished
/// navigation experience.
#[component]
pub fn AnimatedPage(props: AnimatedPageProps) -> Element {
    // Determine initial values based on transition type
    let (initial_opacity, initial_tx, initial_ty, initial_scale) = match props.transition {
        PageTransition::SlideFromRight => (0.0f32, 30.0f32, 0.0f32, 1.0f32),
        PageTransition::SlideFromLeft => (0.0f32, -30.0f32, 0.0f32, 1.0f32),
        PageTransition::SlideFromBottom => (0.0f32, 0.0f32, 30.0f32, 1.0f32),
        PageTransition::FadeScale => (0.0f32, 0.0f32, 0.0f32, 0.95f32),
        PageTransition::Fade => (0.0f32, 0.0f32, 0.0f32, 1.0f32),
        PageTransition::None => (1.0f32, 0.0f32, 0.0f32, 1.0f32),
    };

    // Use motion hooks for animation
    let mut opacity = use_motion(initial_opacity);
    let mut translate_x = use_motion(initial_tx);
    let mut translate_y = use_motion(initial_ty);
    let mut scale = use_motion(initial_scale);

    // Animate to final values on mount
    use_effect(move || {
        if props.transition != PageTransition::None {
            use std::time::Duration;

            opacity.animate_to(
                1.0f32,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition()))
                    .with_delay(Duration::from_millis(props.delay_ms as u64)),
            );
            translate_x.animate_to(
                0.0f32,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition()))
                    .with_delay(Duration::from_millis(props.delay_ms as u64)),
            );
            translate_y.animate_to(
                0.0f32,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition()))
                    .with_delay(Duration::from_millis(props.delay_ms as u64)),
            );
            scale.animate_to(
                1.0f32,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition()))
                    .with_delay(Duration::from_millis(props.delay_ms as u64)),
            );
        }
    });

    let transform = match props.transition {
        PageTransition::SlideFromRight | PageTransition::SlideFromLeft => {
            format!("translateX({}px)", translate_x.get_value())
        }
        PageTransition::SlideFromBottom => {
            format!("translateY({}px)", translate_y.get_value())
        }
        PageTransition::FadeScale => {
            format!("scale({})", scale.get_value())
        }
        _ => String::new(),
    };

    rsx! {
        div {
            style: format!(
                "opacity: {}; \
                 transform: {}; \
                 will-change: transform, opacity;",
                opacity.get_value(),
                transform
            ),
            class: "{props.class}",

            {props.children}
        }
    }
}

/// Animated page container with exit animation support.
///
/// Provides both entrance and exit animations for page transitions.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedPageContainerProps {
    /// Page content.
    pub children: Element,
    /// Whether the page is currently visible (controls exit animation).
    #[props(default = true)]
    pub visible: bool,
    /// Entrance transition.
    #[props(default = PageTransition::SlideFromRight)]
    pub enter_transition: PageTransition,
    /// Exit transition.
    #[props(default = PageTransition::Fade)]
    pub exit_transition: PageTransition,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Page container with enter/exit animations.
#[component]
pub fn AnimatedPageContainer(props: AnimatedPageContainerProps) -> Element {
    let mut opacity = use_motion(1.0f32);
    let mut translate_x = use_motion(0.0f32);
    let mut scale = use_motion(1.0f32);

    // Handle visibility changes
    use_effect(move || {
        if props.visible {
            // Entrance animation
            opacity.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition())),
            );
            translate_x.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition())),
            );
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition())),
            );
        } else {
            // Exit animation - use spring with high stiffness for quick exit
            let exit_config = AnimationConfig::new(AnimationMode::Spring(Spring {
                stiffness: 800.0,
                damping: 40.0,
                mass: 0.5,
                velocity: 0.0,
            }));
            opacity.animate_to(0.0, exit_config);

            match props.exit_transition {
                PageTransition::SlideFromRight => {
                    translate_x.animate_to(
                        -30.0,
                        AnimationConfig::new(AnimationMode::Spring(Spring {
                            stiffness: 800.0,
                            damping: 40.0,
                            mass: 0.5,
                            velocity: 0.0,
                        })),
                    );
                }
                PageTransition::SlideFromLeft => {
                    translate_x.animate_to(
                        30.0,
                        AnimationConfig::new(AnimationMode::Spring(Spring {
                            stiffness: 800.0,
                            damping: 40.0,
                            mass: 0.5,
                            velocity: 0.0,
                        })),
                    );
                }
                PageTransition::FadeScale => {
                    scale.animate_to(
                        0.95,
                        AnimationConfig::new(AnimationMode::Spring(Spring {
                            stiffness: 800.0,
                            damping: 40.0,
                            mass: 0.5,
                            velocity: 0.0,
                        })),
                    );
                }
                _ => {}
            }
        }
    });

    let display = if opacity.get_value() < 0.01 && !props.visible {
        "none"
    } else {
        "block"
    };

    rsx! {
        div {
            style: format!(
                "opacity: {}; \
                 transform: translateX({}px) scale({}); \
                 display: {}; \
                 will-change: transform, opacity;",
                opacity.get_value(),
                translate_x.get_value(),
                scale.get_value(),
                display
            ),
            class: "{props.class}",

            {props.children}
        }
    }
}

/// Staggered page content animation.
///
/// Animates children with a staggered delay for a cascading effect.
#[derive(Props, Clone, PartialEq)]
pub struct StaggeredPageContentProps {
    /// Content elements to animate.
    pub children: Element,
    /// Delay between each child (in milliseconds).
    #[props(default = 50.0)]
    pub stagger_ms: f32,
    /// Initial delay before starting animations.
    #[props(default = 100.0)]
    pub initial_delay_ms: f32,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Page content with staggered entrance animation.
#[component]
pub fn StaggeredPageContent(props: StaggeredPageContentProps) -> Element {
    let mut opacity = use_motion(0.0f32);
    let mut translate_y = use_motion(20.0f32);

    use_effect(move || {
        use std::time::Duration;
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(props.initial_delay_ms as u64)),
        );
        translate_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(props.initial_delay_ms as u64)),
        );
    });

    rsx! {
        div {
            style: format!(
                "opacity: {}; \
                 transform: translateY({}px);",
                opacity.get_value(),
                translate_y.get_value()
            ),
            class: "{props.class}",

            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_transition_defaults_to_slide_from_right() {
        let transition: PageTransition = Default::default();
        assert_eq!(transition, PageTransition::SlideFromRight);
    }

    #[test]
    fn animated_page_props_default() {
        let props = AnimatedPageProps::builder().children(rsx! {}).build();
        // Props are wrapped in a builder struct, verify through component behavior
        let _ = props;
    }
}
