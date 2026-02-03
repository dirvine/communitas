//! Animated navigation item with hover and active state animations.
//!
//! Provides smooth transitions for navigation items with visual feedback
//! on hover, press, and active states.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::animated_nav_item::AnimatedNavItem;
//!
//! rsx! {
//!     AnimatedNavItem {
//!         icon: rsx! { "🏠" },
//!         label: "Home".to_string(),
//!         active: true,
//!         on_click: move |_| println!("Home clicked"),
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::springs;
use crate::design_tokens::{palette, radius, semantic, spacing, typography};

/// Properties for the AnimatedNavItem component.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedNavItemProps {
    /// Icon element to display.
    pub icon: Element,
    /// Navigation label.
    pub label: String,
    /// Whether this item is active/selected.
    #[props(default = false)]
    pub active: bool,
    /// Click handler.
    pub on_click: EventHandler<()>,
    /// Badge content (optional).
    #[props(default = None)]
    pub badge: Option<Element>,
    /// Whether the item is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Whether to show the label (for collapsed sidebar).
    #[props(default = true)]
    pub show_label: bool,
    /// Tooltip for collapsed state.
    #[props(default = None)]
    pub tooltip: Option<String>,
}

/// Animated navigation item with hover and active states.
///
/// Features:
/// - Background opacity animation on hover
/// - Active indicator animation
/// - Scale feedback on press
/// - Badge support
#[component]
pub fn AnimatedNavItem(props: AnimatedNavItemProps) -> Element {
    let mut bg_opacity = use_motion(0.0f32);
    let mut indicator_height = use_motion(0.0f32);
    let mut scale = use_motion(1.0f32);
    let mut icon_scale = use_motion(1.0f32);

    // Animate indicator when active state changes
    use_effect(move || {
        let target = if props.active { 100.0f32 } else { 0.0f32 };
        indicator_height.animate_to(
            target,
            AnimationConfig::new(AnimationMode::Spring(springs::input_focus())),
        );
    });

    let handle_click = move |_| {
        if !props.disabled {
            props.on_click.call(());
        }
    };

    let handle_press_start = move |_| {
        if !props.disabled {
            scale.animate_to(
                0.97,
                AnimationConfig::new(AnimationMode::Spring(springs::button_press())),
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

    let handle_mouse_enter = move |_| {
        if !props.disabled && !props.active {
            bg_opacity.animate_to(
                0.08,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
            icon_scale.animate_to(
                1.1,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let handle_mouse_leave = move |_| {
        if !props.disabled {
            bg_opacity.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
            icon_scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let cursor = if props.disabled {
        "not-allowed"
    } else {
        "pointer"
    };
    let opacity = if props.disabled { 0.5 } else { 1.0 };

    let text_color = if props.active {
        semantic::TEXT_PRIMARY
    } else {
        semantic::TEXT_SECONDARY
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {}; \
                 margin: {}; \
                 background: rgba(255,255,255,{}); \
                 border-radius: {}; \
                 cursor: {}; \
                 opacity: {}; \
                 transform: scale({}); \
                 position: relative; \
                 overflow: hidden; \
                 transition: color 0.2s ease;",
                spacing::SM,
                spacing::SM,
                "2px 0",
                if props.active { 0.1 } else { bg_opacity.get_value() },
                radius::MD,
                cursor,
                opacity,
                scale.get_value()
            ),
            onmousedown: handle_press_start,
            onmouseup: handle_press_end,
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,
            onclick: handle_click,
            title: props.tooltip.clone().unwrap_or_default(),
            class: "{props.class}",

            // Active indicator bar
            div {
                style: format!(
                    "position: absolute; \
                     left: 0; \
                     top: 50%; \
                     transform: translateY(-50%); \
                     width: 3px; \
                     height: {}%; \
                     background: {}; \
                     border-radius: 0 2px 2px 0; \
                     transition: height 0.3s ease;",
                    indicator_height.get_value(),
                    palette::JADE_500
                ),
            }

            // Icon container
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     width: 24px; \
                     height: 24px; \
                     transform: scale({}); \
                     color: {};",
                    icon_scale.get_value(),
                    if props.active { palette::JADE_400 } else { text_color }
                ),
                {props.icon}
            }

            // Label
            if props.show_label {
                span {
                    style: format!(
                        "flex: 1; \
                         font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        if props.active { typography::WEIGHT_MEDIUM } else { typography::WEIGHT_NORMAL },
                        text_color
                    ),
                    "{props.label}"
                }
            }

            // Badge
            if let Some(badge) = props.badge {
                div {
                    style: "display: flex; align-items: center;",
                    {badge}
                }
            }
        }
    }
}

/// Collapsible navigation group with animated expand/collapse.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedNavGroupProps {
    /// Group title.
    pub title: String,
    /// Icon element.
    pub icon: Element,
    /// Child navigation items.
    pub children: Element,
    /// Whether the group is expanded.
    #[props(default = false)]
    pub expanded: bool,
    /// Expansion toggle handler.
    pub on_toggle: EventHandler<()>,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Navigation group with animated expand/collapse.
#[component]
pub fn AnimatedNavGroup(props: AnimatedNavGroupProps) -> Element {
    let mut chevron_rotation = use_motion(0.0f32);
    let _content_height = use_motion(0.0f32);
    let mut content_opacity = use_motion(0.0f32);

    use_effect(move || {
        if props.expanded {
            chevron_rotation.animate_to(
                90.0,
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
            class: "{props.class}",

            // Group header
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {}; \
                     padding: {}; \
                     cursor: pointer; \
                     border-radius: {};",
                    spacing::SM,
                    spacing::SM,
                    radius::MD
                ),
                onclick: handle_toggle,

                // Chevron
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         width: 16px; \
                         height: 16px; \
                         transform: rotate({}deg); \
                         color: {}; \
                         transition: transform 0.2s ease;",
                        chevron_rotation.get_value(),
                        semantic::TEXT_MUTED
                    ),
                    "▶"
                }

                // Icon
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         width: 24px; \
                         height: 24px; \
                         color: {};",
                        semantic::TEXT_SECONDARY
                    ),
                    {props.icon}
                }

                // Title
                span {
                    style: format!(
                        "flex: 1; \
                         font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_SECONDARY
                    ),
                    "{props.title}"
                }
            }

            // Expandable content
            div {
                style: format!(
                    "padding-left: {}; \
                     opacity: {}; \
                     overflow: hidden; \
                     transition: opacity 0.3s ease;",
                    "2rem",
                    content_opacity.get_value()
                ),

                {props.children}
            }
        }
    }
}

/// Navigation divider with optional label.
#[derive(Props, Clone, PartialEq)]
pub struct NavDividerProps {
    /// Optional label text.
    #[props(default = None)]
    pub label: Option<String>,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Navigation divider with animated appearance.
#[component]
pub fn NavDivider(props: NavDividerProps) -> Element {
    let mut opacity = use_motion(0.0f32);
    let mut width = use_motion(0.0f32);

    use_effect(move || {
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter())),
        );
        width.animate_to(
            100.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter())),
        );
    });

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 margin: {}; \
                 opacity: {};",
                spacing::SM,
                "0.5rem 0",
                opacity.get_value()
            ),
            class: "{props.class}",

            // Left line
            div {
                style: format!(
                    "flex: 1; \
                     height: 1px; \
                     background: {}; \
                     width: {}%;",
                    semantic::BORDER_DEFAULT,
                    width.get_value()
                ),
            }

            // Label
            if let Some(label) = props.label {
                span {
                    style: format!(
                        "font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         text-transform: uppercase; \
                         letter-spacing: 0.05em;",
                        typography::FONT_BODY,
                        typography::SIZE_XXS,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_MUTED
                    ),
                    "{label}"
                }
            }

            // Right line
            div {
                style: format!(
                    "flex: 1; \
                     height: 1px; \
                     background: {}; \
                     width: {}%;",
                    semantic::BORDER_DEFAULT,
                    width.get_value()
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NavDividerProps;

    #[test]
    fn nav_divider_props_default() {
        // Verify NavDividerProps can be constructed with defaults
        let props = NavDividerProps {
            label: None,
            class: String::new(),
        };
        assert!(props.label.is_none());
        assert!(props.class.is_empty());
    }
}
