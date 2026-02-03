//! Loading overlay component with fade and scale animations.
//!
//! Provides a polished loading state with animated backdrop and content.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::loading_overlay::LoadingOverlay;
//!
//! rsx! {
//!     LoadingOverlay {
//!         visible: true,
//!         message: Some("Loading...".to_string()),
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::transitions;
use crate::animations::{snappy_spring, springs};
use crate::design_tokens::{palette, radius, semantic, shadow, spacing, typography};

/// Properties for the LoadingOverlay component.
#[derive(Props, Clone, PartialEq)]
pub struct LoadingOverlayProps {
    /// Whether the overlay is visible.
    #[props(default = false)]
    pub visible: bool,
    /// Loading message to display.
    #[props(default = None)]
    pub message: Option<String>,
    /// Spinner size.
    #[props(default = SpinnerSize::Medium)]
    pub spinner_size: SpinnerSize,
    /// Whether to show the backdrop blur.
    #[props(default = true)]
    pub blur_backdrop: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Optional sub-message for additional context.
    #[props(default = None)]
    pub sub_message: Option<String>,
}

/// Spinner size variants.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum SpinnerSize {
    /// Small spinner.
    Small,
    /// Medium spinner (default).
    #[default]
    Medium,
    /// Large spinner.
    Large,
}

/// Loading overlay with fade and scale animations.
///
/// Features:
/// - Fade in/out backdrop
/// - Scale animation for content
/// - Configurable spinner sizes
/// - Optional blur backdrop
#[component]
pub fn LoadingOverlay(props: LoadingOverlayProps) -> Element {
    let mut backdrop_opacity = use_motion(0.0f32);
    let mut content_opacity = use_motion(0.0f32);
    let mut content_scale = use_motion(0.9f32);

    // Animate based on visibility
    use_effect(move || {
        if props.visible {
            backdrop_opacity.animate_to(1.0, transitions::overlay_fade_in());
            content_opacity.animate_to(1.0, transitions::overlay_fade_in());
            content_scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::modal_appear())),
            );
        } else {
            backdrop_opacity.animate_to(0.0, transitions::overlay_fade_out());
            content_opacity.animate_to(0.0, transitions::overlay_fade_out());
            content_scale.animate_to(0.9, transitions::overlay_fade_out());
        }
    });

    let pointer_events = if props.visible { "auto" } else { "none" };
    let display = if backdrop_opacity.get_value() < 0.01 && !props.visible {
        "none"
    } else {
        "flex"
    };

    let backdrop_filter = if props.blur_backdrop {
        "blur(8px)".to_string()
    } else {
        String::new()
    };

    rsx! {
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 display: {}; \
                 align-items: center; \
                 justify-content: center; \
                 background: rgba(0, 0, 0, {}); \
                 backdrop-filter: {}; \
                 opacity: {}; \
                 pointer-events: {}; \
                 z-index: 1000;",
                display,
                backdrop_opacity.get_value() * 0.7,
                backdrop_filter,
                backdrop_opacity.get_value(),
                pointer_events
            ),
            class: "{props.class}",
            role: "dialog",
            aria_busy: "true",
            aria_label: "Loading",

            // Content container
            div {
                style: format!(
                    "display: flex; \
                     flex-direction: column; \
                     align-items: center; \
                     gap: {}; \
                     padding: {}; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     opacity: {}; \
                     transform: scale({}); \
                     box-shadow: {};",
                    spacing::BASE,
                    spacing::XL,
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    content_opacity.get_value(),
                    content_scale.get_value(),
                    shadow::LG
                ),

                // Spinner
                AnimatedSpinner { size: props.spinner_size }

                // Message
                if let Some(message) = props.message {
                    span {
                        style: format!(
                            "color: {}; \
                             font-family: {}; \
                             font-size: {}; \
                             font-weight: {};",
                            semantic::TEXT_PRIMARY,
                            typography::FONT_BODY,
                            typography::SIZE_BASE,
                            typography::WEIGHT_MEDIUM
                        ),
                        "{message}"
                    }
                }

                // Sub-message
                if let Some(sub_message) = props.sub_message {
                    span {
                        style: format!(
                            "color: {}; \
                             font-family: {}; \
                             font-size: {}; \
                             text-align: center; \
                             max-width: 250px;",
                            semantic::TEXT_MUTED,
                            typography::FONT_BODY,
                            typography::SIZE_SM
                        ),
                        "{sub_message}"
                    }
                }
            }
        }
    }
}

/// Animated loading spinner.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedSpinnerProps {
    /// Spinner size.
    #[props(default = SpinnerSize::Medium)]
    pub size: SpinnerSize,
    /// Custom color (defaults to primary).
    #[props(default = None)]
    pub color: Option<String>,
}

/// Animated spinner with rotation.
#[component]
pub fn AnimatedSpinner(props: AnimatedSpinnerProps) -> Element {
    let (size_px, stroke_width) = match props.size {
        SpinnerSize::Small => (24, 2),
        SpinnerSize::Medium => (40, 3),
        SpinnerSize::Large => (64, 4),
    };

    let color = props.color.unwrap_or_else(|| palette::JADE_500.to_string());

    rsx! {
        div {
            style: format!(
                "width: {}px; \
                 height: {}px; \
                 position: relative;",
                size_px,
                size_px
            ),

            // Outer spinning ring
            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     border: {}px solid rgba(255,255,255,0.1); \
                     border-top-color: {}; \
                     border-radius: 50%; \
                     animation: spin 0.8s linear infinite;",
                    stroke_width,
                    color
                ),
            }

            // Inner counter-spinning ring (for visual interest)
            div {
                style: format!(
                    "position: absolute; \
                     inset: {}px; \
                     border: {}px solid rgba(255,255,255,0.05); \
                     border-bottom-color: {}; \
                     border-radius: 50%; \
                     animation: spin 1.2s linear infinite reverse;",
                    stroke_width * 2,
                    stroke_width / 2,
                    color
                ),
            }
        }
    }
}

/// Inline loading indicator for use within content.
#[derive(Props, Clone, PartialEq)]
pub struct InlineLoadingProps {
    /// Loading message.
    #[props(default = None)]
    pub message: Option<String>,
    /// Spinner size.
    #[props(default = SpinnerSize::Small)]
    pub spinner_size: SpinnerSize,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Inline loading indicator.
#[component]
pub fn InlineLoading(props: InlineLoadingProps) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 0.5rem;",
            class: "{props.class}",
            role: "status",
            aria_busy: "true",

            AnimatedSpinner { size: props.spinner_size }

            if let Some(message) = props.message {
                span {
                    style: format!(
                        "color: {}; \
                         font-family: {}; \
                         font-size: {};",
                        semantic::TEXT_SECONDARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM
                    ),
                    "{message}"
                }
            }
        }
    }
}

/// Skeleton loading overlay that shows shimmer effect.
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonOverlayProps {
    /// Whether the overlay is visible.
    #[props(default = false)]
    pub visible: bool,
    /// Number of skeleton items to show.
    #[props(default = 3)]
    pub item_count: usize,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Overlay with shimmer skeleton content.
#[component]
pub fn SkeletonOverlay(props: SkeletonOverlayProps) -> Element {
    let mut opacity = use_motion(0.0f32);

    use_effect(move || {
        if props.visible {
            opacity.animate_to(1.0, transitions::overlay_fade_in());
        } else {
            opacity.animate_to(0.0, transitions::overlay_fade_out());
        }
    });

    let pointer_events = if props.visible { "auto" } else { "none" };
    let display = if opacity.get_value() < 0.01 && !props.visible {
        "none"
    } else {
        "block"
    };

    rsx! {
        div {
            style: format!(
                "position: absolute; \
                 inset: 0; \
                 background: {}; \
                 opacity: {}; \
                 pointer-events: {}; \
                 display: {}; \
                 padding: {};",
                semantic::BG_PRIMARY,
                opacity.get_value(),
                pointer_events,
                display,
                spacing::BASE
            ),
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "Loading content",

            // Skeleton items
            for i in 0..props.item_count {
                div {
                    key: "{i}",
                    style: format!(
                        "height: 60px; \
                         background: {}; \
                         border-radius: {}; \
                         margin-bottom: {};",
                        semantic::BG_SECONDARY,
                        radius::MD,
                        spacing::SM
                    ),
                }
            }
        }
    }
}

/// Progress loading overlay with percentage.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressOverlayProps {
    /// Whether the overlay is visible.
    #[props(default = false)]
    pub visible: bool,
    /// Current progress (0-100).
    pub progress: f32,
    /// Progress message.
    #[props(default = None)]
    pub message: Option<String>,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Loading overlay with progress bar.
#[component]
pub fn ProgressOverlay(props: ProgressOverlayProps) -> Element {
    let mut backdrop_opacity = use_motion(0.0f32);
    let mut content_opacity = use_motion(0.0f32);
    let mut progress_width = use_motion(0.0f32);

    use_effect(move || {
        if props.visible {
            backdrop_opacity.animate_to(1.0, transitions::overlay_fade_in());
            content_opacity.animate_to(1.0, transitions::overlay_fade_in());
        } else {
            backdrop_opacity.animate_to(0.0, transitions::overlay_fade_out());
            content_opacity.animate_to(0.0, transitions::overlay_fade_out());
        }
    });

    use_effect(move || {
        progress_width.animate_to(
            props.progress,
            AnimationConfig::new(AnimationMode::Spring(snappy_spring())),
        );
    });

    let pointer_events = if props.visible { "auto" } else { "none" };
    let display = if backdrop_opacity.get_value() < 0.01 && !props.visible {
        "none"
    } else {
        "flex"
    };

    rsx! {
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 display: {}; \
                 align-items: center; \
                 justify-content: center; \
                 background: rgba(0, 0, 0, {}); \
                 backdrop-filter: blur(8px); \
                 opacity: {}; \
                 pointer-events: {}; \
                 z-index: 1000;",
                display,
                backdrop_opacity.get_value() * 0.7,
                backdrop_opacity.get_value(),
                pointer_events
            ),
            class: "{props.class}",

            div {
                style: format!(
                    "width: 300px; \
                     padding: {}; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     opacity: {};",
                    spacing::XL,
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    content_opacity.get_value()
                ),

                // Progress message
                if let Some(message) = props.message {
                    div {
                        style: format!(
                            "margin-bottom: {}; \
                             color: {}; \
                             font-family: {}; \
                             font-size: {};",
                            spacing::BASE,
                            semantic::TEXT_PRIMARY,
                            typography::FONT_BODY,
                            typography::SIZE_BASE
                        ),
                        "{message}"
                    }
                }

                // Progress bar container
                div {
                    style: format!(
                        "height: 8px; \
                         background: {}; \
                         border-radius: {}; \
                         overflow: hidden;",
                        semantic::BG_TERTIARY,
                        radius::FULL
                    ),

                    // Progress bar fill
                    div {
                        style: format!(
                            "width: {}%; \
                             height: 100%; \
                             background: linear-gradient(90deg, {} 0%, {} 100%); \
                             border-radius: {};",
                            progress_width.get_value(),
                            palette::JADE_500,
                            palette::JADE_400,
                            radius::FULL
                        ),
                    }
                }

                // Progress percentage
                div {
                    style: format!(
                        "margin-top: {}; \
                         text-align: center; \
                         color: {}; \
                         font-family: {}; \
                         font-size: {}; \
                         font-weight: {};",
                        spacing::SM,
                        semantic::TEXT_SECONDARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM
                    ),
                    "{progress_width.get_value() as i32}%"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_size_defaults_to_medium() {
        let size: SpinnerSize = Default::default();
        assert_eq!(size, SpinnerSize::Medium);
    }

    #[test]
    fn loading_overlay_default_props() {
        let props = LoadingOverlayProps::builder().build();
        // Props are wrapped in a builder struct, verify through component behavior
        let _ = props;
    }
}
