//! Animated modal component with backdrop and content animations.
//!
//! Provides a polished modal experience with fade, scale, and blur animations.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::animated_modal::AnimatedModal;
//!
//! rsx! {
//!     AnimatedModal {
//!         open: true,
//!         on_close: move |_| println!("Modal closed"),
//!         children: rsx! {
//!             h2 { "Modal Title" }
//!             p { "Modal content goes here" }
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::springs;
use crate::animations::transitions;
use crate::design_tokens::{palette, radius, semantic, shadow, spacing, typography};

/// Properties for the AnimatedModal component.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedModalProps {
    /// Whether the modal is open.
    #[props(default = false)]
    pub open: bool,
    /// Close handler.
    pub on_close: EventHandler<()>,
    /// Modal content.
    pub children: Element,
    /// Modal title (optional).
    #[props(default = None)]
    pub title: Option<String>,
    /// Modal size.
    #[props(default = ModalSize::Medium)]
    pub size: ModalSize,
    /// Whether to close on backdrop click.
    #[props(default = true)]
    pub close_on_backdrop: bool,
    /// Whether to close on escape key.
    #[props(default = true)]
    pub close_on_escape: bool,
    /// Additional CSS classes for the content.
    #[props(default = String::new())]
    pub class: String,
    /// Whether to show a close button.
    #[props(default = true)]
    pub show_close_button: bool,
}

/// Modal size variants.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ModalSize {
    /// Small modal (400px).
    Small,
    /// Medium modal (500px, default).
    #[default]
    Medium,
    /// Large modal (700px).
    Large,
    /// Extra large modal (900px).
    ExtraLarge,
    /// Full screen modal.
    FullScreen,
}

impl ModalSize {
    /// Get the max-width for this modal size.
    fn max_width(&self) -> &'static str {
        match self {
            ModalSize::Small => "400px",
            ModalSize::Medium => "500px",
            ModalSize::Large => "700px",
            ModalSize::ExtraLarge => "900px",
            ModalSize::FullScreen => "100vw",
        }
    }

    /// Get the max-height for this modal size.
    fn max_height(&self) -> &'static str {
        match self {
            ModalSize::FullScreen => "100vh",
            _ => "90vh",
        }
    }
}

/// Animated modal with backdrop and content animations.
///
/// Features:
/// - Fade in/out backdrop with blur
/// - Scale and fade content animation
/// - Close on backdrop click (optional)
/// - Close on escape key (optional)
/// - Accessible focus management
#[component]
pub fn AnimatedModal(props: AnimatedModalProps) -> Element {
    let mut backdrop_opacity = use_motion(0.0f32);
    let mut content_opacity = use_motion(0.0f32);
    let mut content_scale = use_motion(0.95f32);
    let mut content_translate_y = use_motion(20.0f32);

    // Animate based on open state
    use_effect(move || {
        if props.open {
            backdrop_opacity.animate_to(1.0, transitions::overlay_fade_in());
            content_opacity.animate_to(1.0, transitions::overlay_fade_in());
            content_scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::modal_appear())),
            );
            content_translate_y.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::modal_appear())),
            );
        } else {
            backdrop_opacity.animate_to(0.0, transitions::overlay_fade_out());
            content_opacity.animate_to(0.0, transitions::overlay_fade_out());
            content_scale.animate_to(0.95, transitions::overlay_fade_out());
            content_translate_y.animate_to(20.0, transitions::overlay_fade_out());
        }
    });

    let handle_backdrop_click = move |_| {
        if props.close_on_backdrop {
            props.on_close.call(());
        }
    };

    let handle_content_click = move |e: Event<MouseData>| {
        e.stop_propagation();
    };

    let handle_keydown = move |evt: KeyboardEvent| {
        if props.close_on_escape && evt.key() == Key::Escape {
            props.on_close.call(());
        }
    };

    let pointer_events = if props.open { "auto" } else { "none" };
    let display = if backdrop_opacity.get_value() < 0.01 && !props.open {
        "none"
    } else {
        "flex"
    };

    let max_width = props.size.max_width();
    let max_height = props.size.max_height();

    rsx! {
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 display: {}; \
                 align-items: center; \
                 justify-content: center; \
                 z-index: 500;",
                display
            ),
            tabindex: "0",
            autofocus: props.open,
            onkeydown: handle_keydown,

            // Backdrop
            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     background: rgba(0, 0, 0, {}); \
                     backdrop-filter: blur({}px); \
                     opacity: {}; \
                     pointer-events: {};",
                    backdrop_opacity.get_value() * 0.7,
                    backdrop_opacity.get_value() * 8.0,
                    backdrop_opacity.get_value(),
                    pointer_events
                ),
                onclick: handle_backdrop_click,
            }

            // Content
            div {
                style: format!(
                    "position: relative; \
                     width: 90%; \
                     max-width: {}; \
                     max-height: {}; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     box-shadow: {}; \
                     opacity: {}; \
                     transform: scale({}) translateY({}px); \
                     pointer-events: {}; \
                     display: flex; \
                     flex-direction: column; \
                     overflow: hidden;",
                    max_width,
                    max_height,
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    shadow::XL,
                    content_opacity.get_value(),
                    content_scale.get_value(),
                    content_translate_y.get_value(),
                    pointer_events
                ),
                class: "{props.class}",
                onclick: handle_content_click,
                role: "dialog",
                aria_modal: "true",

                // Header (if title provided)
                if let Some(ref title) = props.title {
                    div {
                        style: format!(
                            "display: flex; \
                             align-items: center; \
                             justify-content: space-between; \
                             padding: {} {}; \
                             border-bottom: 1px solid {};",
                            spacing::BASE,
                            spacing::LG,
                            semantic::BORDER_DEFAULT
                        ),

                        h2 {
                            style: format!(
                                "margin: 0; \
                                 font-family: {}; \
                                 font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::FONT_DISPLAY,
                                typography::SIZE_LG,
                                typography::WEIGHT_SEMIBOLD,
                                semantic::TEXT_PRIMARY
                            ),
                            "{title}"
                        }

                        // Close button
                        if props.show_close_button {
                            button {
                                style: format!(
                                    "display: flex; \
                                     align-items: center; \
                                     justify-content: center; \
                                     width: 32px; \
                                     height: 32px; \
                                     border: none; \
                                     background: transparent; \
                                     color: {}; \
                                     cursor: pointer; \
                                     border-radius: {}; \
                                     font-size: 18px;",
                                    semantic::TEXT_MUTED,
                                    radius::MD
                                ),
                                onclick: move |_| props.on_close.call(()),
                                aria_label: "Close modal",
                                "✕"
                            }
                        }
                    }
                }

                // Body
                div {
                    style: format!(
                        "padding: {}; \
                         overflow-y: auto; \
                         flex: 1;",
                        if props.title.is_some() {
                            format!("{} {}", spacing::LG, spacing::LG)
                        } else {
                            format!("{} {}", spacing::BASE, spacing::LG)
                        }
                    ),

                    {props.children}
                }
            }
        }
    }
}

/// Modal footer for action buttons.
#[derive(Props, Clone, PartialEq)]
pub struct ModalFooterProps {
    /// Footer content (typically buttons).
    pub children: Element,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Modal footer with consistent styling.
#[component]
pub fn ModalFooter(props: ModalFooterProps) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 justify-content: flex-end; \
                 gap: {}; \
                 padding: {} {}; \
                 border-top: 1px solid {};",
                spacing::SM,
                spacing::BASE,
                spacing::LG,
                semantic::BORDER_DEFAULT
            ),
            class: "{props.class}",

            {props.children}
        }
    }
}

/// Confirmation modal with pre-built layout.
#[derive(Props, Clone, PartialEq)]
pub struct ConfirmModalProps {
    /// Whether the modal is open.
    #[props(default = false)]
    pub open: bool,
    /// Close handler.
    pub on_close: EventHandler<()>,
    /// Confirm handler.
    pub on_confirm: EventHandler<()>,
    /// Modal title.
    pub title: String,
    /// Confirmation message.
    pub message: String,
    /// Confirm button text.
    #[props(default = "Confirm".to_string())]
    pub confirm_text: String,
    /// Cancel button text.
    #[props(default = "Cancel".to_string())]
    pub cancel_text: String,
    /// Whether the confirm action is destructive.
    #[props(default = false)]
    pub destructive: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Pre-built confirmation modal.
#[component]
pub fn ConfirmModal(props: ConfirmModalProps) -> Element {
    let handle_confirm = move |_| {
        props.on_confirm.call(());
        props.on_close.call(());
    };

    let confirm_button_bg = if props.destructive {
        palette::ROSE_500
    } else {
        palette::JADE_500
    };

    rsx! {
        AnimatedModal {
            open: props.open,
            on_close: props.on_close,
            title: props.title,
            size: ModalSize::Small,
            class: props.class,

            p {
                style: format!(
                    "margin: 0; \
                     font-family: {}; \
                     font-size: {}; \
                     color: {}; \
                     line-height: 1.5;",
                    typography::FONT_BODY,
                    typography::SIZE_BASE,
                    semantic::TEXT_SECONDARY
                ),
                "{props.message}"
            }

            ModalFooter {
                button {
                    style: format!(
                        "padding: {} {}; \
                         border: 1px solid {}; \
                         background: transparent; \
                         color: {}; \
                         font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         border-radius: {}; \
                         cursor: pointer;",
                        spacing::SM,
                        spacing::BASE,
                        semantic::BORDER_DEFAULT,
                        semantic::TEXT_SECONDARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        radius::MD
                    ),
                    onclick: move |_| props.on_close.call(()),
                    "{props.cancel_text}"
                }

                button {
                    style: format!(
                        "padding: {} {}; \
                         border: none; \
                         background: {}; \
                         color: white; \
                         font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         border-radius: {}; \
                         cursor: pointer;",
                        spacing::SM,
                        spacing::BASE,
                        confirm_button_bg,
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        radius::MD
                    ),
                    onclick: handle_confirm,
                    "{props.confirm_text}"
                }
            }
        }
    }
}

/// Drawer component that slides in from the side.
#[derive(Props, Clone, PartialEq)]
pub struct DrawerProps {
    /// Whether the drawer is open.
    #[props(default = false)]
    pub open: bool,
    /// Close handler.
    pub on_close: EventHandler<()>,
    /// Drawer content.
    pub children: Element,
    /// Drawer title.
    #[props(default = None)]
    pub title: Option<String>,
    /// Drawer position.
    #[props(default = DrawerPosition::Right)]
    pub position: DrawerPosition,
    /// Drawer width.
    #[props(default = "400px".to_string())]
    pub width: String,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Drawer position.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum DrawerPosition {
    /// Slide in from the left.
    Left,
    /// Slide in from the right (default).
    #[default]
    Right,
}

/// Slide-in drawer component.
#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let mut backdrop_opacity = use_motion(0.0f32);
    let mut content_translate_x = use_motion(if props.position == DrawerPosition::Right {
        100.0f32
    } else {
        -100.0f32
    });

    use_effect(move || {
        if props.open {
            backdrop_opacity.animate_to(1.0, transitions::overlay_fade_in());
            content_translate_x.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition())),
            );
        } else {
            backdrop_opacity.animate_to(0.0, transitions::overlay_fade_out());
            content_translate_x.animate_to(
                if props.position == DrawerPosition::Right {
                    100.0f32
                } else {
                    -100.0f32
                },
                AnimationConfig::new(AnimationMode::Spring(springs::page_transition())),
            );
        }
    });

    let pointer_events = if props.open { "auto" } else { "none" };
    let display = if backdrop_opacity.get_value() < 0.01 && !props.open {
        "none"
    } else {
        "block"
    };

    let position_style = match props.position {
        DrawerPosition::Left => format!(
            "left: 0; transform: translateX({}%)",
            content_translate_x.get_value()
        ),
        DrawerPosition::Right => format!(
            "right: 0; transform: translateX({}%)",
            content_translate_x.get_value()
        ),
    };

    rsx! {
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 display: {}; \
                 z-index: 500;",
                display
            ),

            // Backdrop
            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     background: rgba(0, 0, 0, {}); \
                     opacity: {}; \
                     pointer-events: {};",
                    backdrop_opacity.get_value() * 0.5,
                    backdrop_opacity.get_value(),
                    pointer_events
                ),
                onclick: move |_| props.on_close.call(()),
            }

            // Drawer content
            div {
                style: format!(
                    "position: absolute; \
                     top: 0; \
                     bottom: 0; \
                     width: {}; \
                     max-width: 90vw; \
                     background: {}; \
                     border-{}: 1px solid {}; \
                     box-shadow: {}; \
                     {}; \
                     display: flex; \
                     flex-direction: column;",
                    props.width,
                    semantic::BG_SECONDARY,
                    if props.position == DrawerPosition::Left { "right" } else { "left" },
                    semantic::BORDER_DEFAULT,
                    shadow::XL,
                    position_style
                ),
                class: "{props.class}",
                role: "dialog",
                aria_modal: "true",

                // Header
                if let Some(title) = props.title {
                    div {
                        style: format!(
                            "display: flex; \
                             align-items: center; \
                             justify-content: space-between; \
                             padding: {} {}; \
                             border-bottom: 1px solid {};",
                            spacing::BASE,
                            spacing::LG,
                            semantic::BORDER_DEFAULT
                        ),

                        h2 {
                            style: format!(
                                "margin: 0; \
                                 font-family: {}; \
                                 font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::FONT_DISPLAY,
                                typography::SIZE_LG,
                                typography::WEIGHT_SEMIBOLD,
                                semantic::TEXT_PRIMARY
                            ),
                            "{title}"
                        }

                        button {
                            style: format!(
                                "display: flex; \
                                 align-items: center; \
                                 justify-content: center; \
                                 width: 32px; \
                                 height: 32px; \
                                 border: none; \
                                 background: transparent; \
                                 color: {}; \
                                 cursor: pointer; \
                                 border-radius: {}; \
                                 font-size: 18px;",
                                semantic::TEXT_MUTED,
                                radius::MD
                            ),
                            onclick: move |_| props.on_close.call(()),
                            aria_label: "Close drawer",
                            "✕"
                        }
                    }
                }

                // Body
                div {
                    style: format!(
                        "padding: {}; \
                         overflow-y: auto; \
                         flex: 1;",
                        spacing::LG
                    ),

                    {props.children}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_size_defaults_to_medium() {
        let size: ModalSize = Default::default();
        assert_eq!(size, ModalSize::Medium);
    }

    #[test]
    fn modal_size_max_widths() {
        assert_eq!(ModalSize::Small.max_width(), "400px");
        assert_eq!(ModalSize::Medium.max_width(), "500px");
        assert_eq!(ModalSize::Large.max_width(), "700px");
        assert_eq!(ModalSize::ExtraLarge.max_width(), "900px");
        assert_eq!(ModalSize::FullScreen.max_width(), "100vw");
    }

    #[test]
    fn drawer_position_defaults_to_right() {
        let position: DrawerPosition = Default::default();
        assert_eq!(position, DrawerPosition::Right);
    }
}
