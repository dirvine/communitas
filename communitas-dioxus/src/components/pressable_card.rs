//! Pressable card component with tactile feedback and elevation animations.
//!
//! Cards that respond to press with tactile feedback and smooth elevation changes.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::pressable_card::PressableCard;
//!
//! rsx! {
//!     PressableCard {
//!         on_click: move |_| println!("Card clicked!"),
//!         children: rsx! {
//!             h3 { "Card Title" }
//!             p { "Card content goes here" }
//!         }
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::springs;
use crate::design_tokens::{radius, semantic, shadow, spacing};

/// Properties for the PressableCard component.
#[derive(Props, Clone, PartialEq)]
pub struct PressableCardProps {
    /// Card content.
    pub children: Element,
    /// Click handler.
    #[props(default = None)]
    pub on_click: Option<EventHandler<()>>,
    /// Whether the card is interactive.
    #[props(default = true)]
    pub interactive: bool,
    /// Whether the card is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Custom padding.
    #[props(default = None)]
    pub padding: Option<String>,
    /// Card elevation when not hovered (0-3).
    #[props(default = 0)]
    pub base_elevation: u8,
    /// Card elevation when hovered (0-3).
    #[props(default = 2)]
    pub hover_elevation: u8,
    /// Border radius override.
    #[props(default = None)]
    pub border_radius: Option<String>,
    /// Background color override.
    #[props(default = None)]
    pub background: Option<String>,
}

/// Pressable card with tactile feedback.
///
/// Features:
/// - Scale down on press (0.98x)
/// - Elevation increase on hover
/// - Smooth spring animations
/// - Disabled state support
#[component]
pub fn PressableCard(props: PressableCardProps) -> Element {
    let mut scale = use_motion(1.0f32);
    let mut elevation = use_motion(props.base_elevation as f32);

    let is_disabled = props.disabled || !props.interactive;
    let has_on_click = props.on_click.is_some();

    let handle_press_start = move |_| {
        if !is_disabled && has_on_click {
            scale.animate_to(
                0.98,
                AnimationConfig::new(AnimationMode::Spring(springs::card_press())),
            );
        }
    };

    let handle_press_end = move |_| {
        if !is_disabled && has_on_click {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            if let Some(handler) = props.on_click {
                handler.call(());
            }
        }
    };

    let handle_mouse_enter = move |_| {
        if !is_disabled {
            elevation.animate_to(
                props.hover_elevation as f32,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let handle_mouse_leave = move |_| {
        if !is_disabled {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
            elevation.animate_to(
                props.base_elevation as f32,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let cursor = if is_disabled || !has_on_click {
        "default"
    } else {
        "pointer"
    };

    let opacity = if props.disabled { 0.6 } else { 1.0 };

    // Calculate shadow based on elevation
    let shadow_style = match elevation.get_value() as u8 {
        0 => shadow::NONE,
        1 => shadow::SM,
        2 => shadow::MD,
        3 => shadow::LG,
        _ => shadow::XL,
    };

    let padding = props.padding.unwrap_or_else(|| spacing::BASE.to_string());
    let border_radius = props
        .border_radius
        .unwrap_or_else(|| radius::LG.to_string());
    let background = props
        .background
        .unwrap_or_else(|| semantic::BG_SECONDARY.to_string());

    rsx! {
        div {
            style: format!(
                "transform: scale({}); \
                 box-shadow: {}; \
                 background: {}; \
                 padding: {}; \
                 border-radius: {}; \
                 cursor: {}; \
                 opacity: {}; \
                 border: 1px solid {}; \
                 transition: border-color 0.2s ease;",
                scale.get_value(),
                shadow_style,
                background,
                padding,
                border_radius,
                cursor,
                opacity,
                semantic::BORDER_DEFAULT
            ),
            onmousedown: handle_press_start,
            onmouseup: handle_press_end,
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,

            {props.children}
        }
    }
}

/// Selectable card with active state.
#[derive(Props, Clone, PartialEq)]
pub struct SelectableCardProps {
    /// Card content.
    pub children: Element,
    /// Whether the card is selected.
    #[props(default = false)]
    pub selected: bool,
    /// Selection change handler.
    pub on_select: EventHandler<bool>,
    /// Whether the card is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Card that can be selected with visual feedback.
#[component]
pub fn SelectableCard(props: SelectableCardProps) -> Element {
    let mut scale = use_motion(1.0f32);
    let mut border_opacity = use_motion(0.0f32);

    // Animate border when selected state changes
    use_effect(move || {
        let target = if props.selected { 1.0f32 } else { 0.0f32 };
        border_opacity.animate_to(
            target,
            AnimationConfig::new(AnimationMode::Spring(springs::input_focus())),
        );
    });

    let handle_click = move |_| {
        if !props.disabled {
            props.on_select.call(!props.selected);
        }
    };

    let handle_press_start = move |_| {
        if !props.disabled {
            scale.animate_to(
                0.98,
                AnimationConfig::new(AnimationMode::Spring(springs::card_press())),
            );
        }
    };

    let handle_press_end = move |_| {
        if !props.disabled {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
        }
    };

    let cursor = if props.disabled {
        "not-allowed"
    } else {
        "pointer"
    };
    let opacity = if props.disabled { 0.6 } else { 1.0 };

    rsx! {
        div {
            style: format!(
                "transform: scale({}); \
                 background: {}; \
                 padding: {}; \
                 border-radius: {}; \
                 cursor: {}; \
                 opacity: {}; \
                 border: 2px solid rgba(16, 185, 129, {}); \
                 box-shadow: {}; \
                 transition: box-shadow 0.2s ease;",
                scale.get_value(),
                semantic::BG_SECONDARY,
                spacing::BASE,
                radius::LG,
                cursor,
                opacity,
                border_opacity.get_value(),
                if props.selected { shadow::GLOW_SM } else { shadow::SM }
            ),
            onmousedown: handle_press_start,
            onmouseup: handle_press_end,
            onclick: handle_click,

            {props.children}
        }
    }
}

/// Expandable card with smooth height animation.
#[derive(Props, Clone, PartialEq)]
pub struct ExpandableCardProps {
    /// Card header content (always visible).
    pub header: Element,
    /// Expandable content.
    pub children: Element,
    /// Whether the card is expanded.
    pub expanded: bool,
    /// Expansion toggle handler.
    pub on_toggle: EventHandler<()>,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Card that expands to show more content.
#[component]
pub fn ExpandableCard(props: ExpandableCardProps) -> Element {
    let mut chevron_rotation = use_motion(0.0f32);
    let mut content_opacity = use_motion(0.0f32);

    // Animate when expanded state changes
    use_effect(move || {
        if props.expanded {
            chevron_rotation.animate_to(
                180.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
            content_opacity.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        } else {
            chevron_rotation.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
            content_opacity.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    });

    let handle_toggle = move |_| {
        props.on_toggle.call(());
    };

    rsx! {
        div {
            style: format!(
                "background: {}; \
                 border-radius: {}; \
                 border: 1px solid {}; \
                 overflow: hidden;",
                semantic::BG_SECONDARY,
                radius::LG,
                semantic::BORDER_DEFAULT
            ),

            // Header with toggle button
            div {
                style: format!(
                    "padding: {}; \
                     display: flex; \
                     align-items: center; \
                     justify-content: space-between; \
                     cursor: pointer;",
                    spacing::BASE
                ),
                onclick: handle_toggle,

                {props.header}

                // Chevron icon with rotation
                div {
                    style: format!(
                        "transform: rotate({}deg); \
                         transition: transform 0.3s ease;",
                        chevron_rotation.get_value()
                    ),
                    "▼"
                }
            }

            // Expandable content
            if props.expanded {
                div {
                    style: format!(
                        "padding: {}; \
                         padding-top: 0; \
                         opacity: {}; \
                         border-top: 1px solid {};",
                        spacing::BASE,
                        content_opacity.get_value(),
                        semantic::BORDER_DEFAULT
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
    fn pressable_card_props_default() {
        let props = PressableCardProps::builder().children(rsx! {}).build();
        // Props are wrapped in a builder struct, verify through component behavior
        let _ = props;
    }
}
