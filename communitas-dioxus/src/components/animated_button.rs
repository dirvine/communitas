//! Animated button component with tactile feedback and spring physics.
//!
//! Provides satisfying press animations, loading states, and accessibility support.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::animated_button::AnimatedButton;
//!
//! rsx! {
//!     AnimatedButton {
//!         label: "Click Me".to_string(),
//!         on_click: move |_| println!("Clicked!"),
//!         variant: ButtonVariant::Primary,
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::springs;
use crate::design_tokens::{palette, radius, semantic, spacing, typography};

/// Button visual variants.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonVariant {
    /// Primary action button with gradient background.
    #[default]
    Primary,
    /// Secondary button with outline style.
    Secondary,
    /// Ghost button with minimal styling.
    Ghost,
    /// Danger button for destructive actions.
    Danger,
}

/// Button size variants.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ButtonSize {
    /// Small button for compact spaces.
    Small,
    /// Standard button size.
    #[default]
    Medium,
    /// Large button for emphasis.
    Large,
}

/// Properties for the AnimatedButton component.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedButtonProps {
    /// Button label text.
    pub label: String,
    /// Click handler.
    pub on_click: EventHandler<()>,
    /// Visual variant.
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    /// Size variant.
    #[props(default = ButtonSize::Medium)]
    pub size: ButtonSize,
    /// Whether the button is in loading state.
    #[props(default = false)]
    pub loading: bool,
    /// Whether the button is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Optional icon element to display before label.
    #[props(default = None)]
    pub icon: Option<Element>,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Button type attribute.
    #[props(default = "button".to_string())]
    pub button_type: String,
    /// Full width button.
    #[props(default = false)]
    pub full_width: bool,
}

/// Animated button with spring physics feedback.
///
/// Features:
/// - Press down scale animation (0.95x)
/// - Release spring back animation
/// - Loading spinner state
/// - Disabled state styling
/// - Accessible focus states
#[component]
pub fn AnimatedButton(props: AnimatedButtonProps) -> Element {
    let mut scale = use_motion(1.0f32);
    let mut elevation = use_motion(0.0f32);

    let is_disabled = props.disabled || props.loading;

    // Style calculations based on variant and size
    let (bg_style, text_color, border_style) = match props.variant {
        ButtonVariant::Primary => (
            format!(
                "linear-gradient(135deg, {} 0%, {} 100%)",
                palette::JADE_500,
                palette::JADE_600
            ),
            semantic::TEXT_PRIMARY,
            "none".to_string(),
        ),
        ButtonVariant::Secondary => (
            "transparent".to_string(),
            semantic::PRIMARY,
            format!("1px solid {}", semantic::BORDER_DEFAULT),
        ),
        ButtonVariant::Ghost => (
            "transparent".to_string(),
            semantic::TEXT_SECONDARY,
            "none".to_string(),
        ),
        ButtonVariant::Danger => (
            format!(
                "linear-gradient(135deg, {} 0%, {} 100%)",
                palette::ROSE_500,
                palette::ROSE_400
            ),
            semantic::TEXT_PRIMARY,
            "none".to_string(),
        ),
    };

    let (padding, font_size, height) = match props.size {
        ButtonSize::Small => (
            format!("{} {}", spacing::XS, spacing::SM),
            typography::SIZE_XS,
            "32px",
        ),
        ButtonSize::Medium => (
            format!("{} {}", spacing::SM, spacing::BASE),
            typography::SIZE_SM,
            "40px",
        ),
        ButtonSize::Large => (
            format!("{} {}", spacing::BASE, spacing::LG),
            typography::SIZE_BASE,
            "48px",
        ),
    };

    let width = if props.full_width { "100%" } else { "auto" };
    let mut suppress_click = use_signal(|| false);

    let handle_press_start = move |_| {
        if !is_disabled {
            scale.animate_to(
                0.95,
                AnimationConfig::new(AnimationMode::Spring(springs::button_press())),
            );
            elevation.animate_to(
                -2.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_press())),
            );
        }
    };

    let handle_press_end = move |_| {
        if !is_disabled {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            elevation.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
        }
    };

    let on_click = props.on_click.clone();
    let handle_click = move |_| {
        if suppress_click() {
            suppress_click.set(false);
            return;
        }
        if !is_disabled {
            on_click.call(());
        }
    };

    let mut scale_keydown = scale.clone();
    let mut elevation_keydown = elevation.clone();
    let handle_key_down = move |evt: KeyboardEvent| {
        if is_disabled {
            return;
        }
        let key = evt.key();
        if key == "Enter" || key == " " {
            evt.prevent_default();
            scale_keydown.animate_to(
                0.95,
                AnimationConfig::new(AnimationMode::Spring(springs::button_press())),
            );
            elevation_keydown.animate_to(
                -2.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_press())),
            );
        }
    };

    let mut scale_keyup = scale.clone();
    let mut elevation_keyup = elevation.clone();
    let on_click_keyup = props.on_click.clone();
    let handle_key_up = move |evt: KeyboardEvent| {
        if is_disabled {
            return;
        }
        let key = evt.key();
        if key == "Enter" || key == " " {
            evt.prevent_default();
            scale_keyup.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            elevation_keyup.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            suppress_click.set(true);
            on_click_keyup.call(());
        }
    };

    let handle_mouse_enter = move |_| {
        if !is_disabled {
            elevation.animate_to(
                4.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let handle_mouse_leave = move |_| {
        if !is_disabled {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            elevation.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let cursor = if is_disabled {
        "not-allowed"
    } else {
        "pointer"
    };
    let opacity = if is_disabled { 0.5 } else { 1.0 };

    rsx! {
        button {
            type: "{props.button_type}",
            disabled: is_disabled,
            style: format!(
                "transform: scale({}); \
                 box-shadow: 0 {}px {}px rgba(0,0,0,0.2); \
                 background: {}; \
                 color: {}; \
                 border: {}; \
                 padding: {}; \
                 font-size: {}; \
                 font-family: {}; \
                 font-weight: {}; \
                 border-radius: {}; \
                 height: {}; \
                 width: {}; \
                 cursor: {}; \
                 opacity: {}; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 gap: {}; \
                 transition: box-shadow 0.2s ease; \
                 outline: none; \
                 position: relative; \
                 overflow: hidden;",
                scale.get_value(),
                4.0 + elevation.get_value(),
                8.0 + elevation.get_value() * 2.0,
                bg_style,
                text_color,
                border_style,
                padding,
                font_size,
                typography::FONT_BODY,
                typography::WEIGHT_MEDIUM,
                radius::MD,
                height,
                width,
                cursor,
                opacity,
                spacing::XS
            ),
            onmousedown: handle_press_start,
            onmouseup: handle_press_end,
            onclick: handle_click,
            onkeydown: handle_key_down,
            onkeyup: handle_key_up,
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,
            onmouseout: handle_mouse_leave,

            if props.loading {
                LoadingSpinner { size: props.size }
            } else {
                {props.icon.clone()}
                span { "{props.label}" }
            }
        }
    }
}

/// Loading spinner for button loading state.
#[component]
fn LoadingSpinner(size: ButtonSize) -> Element {
    let spinner_size = match size {
        ButtonSize::Small => "16px",
        ButtonSize::Medium => "20px",
        ButtonSize::Large => "24px",
    };

    rsx! {
        div {
            style: format!(
                "width: {}; \
                 height: {}; \
                 border: 2px solid rgba(255,255,255,0.3); \
                 border-top-color: white; \
                 border-radius: 50%; \
                 animation: spin 0.8s linear infinite;",
                spinner_size,
                spinner_size
            ),
        }
    }
}

/// Animated icon button for toolbars and actions.
#[derive(Props, Clone, PartialEq)]
pub struct IconButtonProps {
    /// Icon element to display.
    pub icon: Element,
    /// Click handler.
    pub on_click: EventHandler<()>,
    /// Tooltip label.
    #[props(default = None)]
    pub label: Option<String>,
    /// Whether the button is disabled.
    #[props(default = false)]
    pub disabled: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Button size.
    #[props(default = ButtonSize::Medium)]
    pub size: ButtonSize,
}

/// Icon button with press animation.
#[component]
pub fn AnimatedIconButton(props: IconButtonProps) -> Element {
    let mut scale = use_motion(1.0f32);
    let mut bg_opacity = use_motion(0.0f32);
    let mut suppress_click = use_signal(|| false);

    let (size_px, icon_size) = match props.size {
        ButtonSize::Small => ("32px", "16px"),
        ButtonSize::Medium => ("40px", "20px"),
        ButtonSize::Large => ("48px", "24px"),
    };

    let handle_press_start = move |_| {
        if !props.disabled {
            scale.animate_to(
                0.9,
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

    let on_click = props.on_click.clone();
    let handle_click = move |_| {
        if suppress_click() {
            suppress_click.set(false);
            return;
        }
        if !props.disabled {
            on_click.call(());
        }
    };

    let mut scale_keydown = scale.clone();
    let handle_key_down = move |evt: KeyboardEvent| {
        if props.disabled {
            return;
        }
        let key = evt.key();
        if key == "Enter" || key == " " {
            evt.prevent_default();
            scale_keydown.animate_to(
                0.9,
                AnimationConfig::new(AnimationMode::Spring(springs::button_press())),
            );
        }
    };

    let mut scale_keyup = scale.clone();
    let on_click_keyup = props.on_click.clone();
    let handle_key_up = move |evt: KeyboardEvent| {
        if props.disabled {
            return;
        }
        let key = evt.key();
        if key == "Enter" || key == " " {
            evt.prevent_default();
            scale_keyup.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            suppress_click.set(true);
            on_click_keyup.call(());
        }
    };

    let handle_mouse_enter = move |_| {
        if !props.disabled {
            bg_opacity.animate_to(
                0.1,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    let handle_mouse_leave = move |_| {
        if !props.disabled {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            bg_opacity.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
            );
        }
    };

    rsx! {
        button {
            type: "button",
            disabled: props.disabled,
            aria_label: props.label.clone().unwrap_or_default(),
            style: format!(
                "width: {}; \
                 height: {}; \
                 transform: scale({}); \
                 background: rgba(255,255,255,{}); \
                 border: none; \
                 border-radius: {}; \
                 cursor: {}; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 outline: none; \
                 transition: background 0.2s ease;",
                size_px,
                size_px,
                scale.get_value(),
                bg_opacity.get_value(),
                radius::MD,
                if props.disabled { "not-allowed" } else { "pointer" }
            ),
            onmousedown: handle_press_start,
            onmouseup: handle_press_end,
            onclick: handle_click,
            onkeydown: handle_key_down,
            onkeyup: handle_key_up,
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,

            div {
                style: format!("width: {}; height: {}; display: flex; align-items: center; justify-content: center;", icon_size, icon_size),
                {props.icon}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_variant_defaults_to_primary() {
        let variant: ButtonVariant = Default::default();
        assert_eq!(variant, ButtonVariant::Primary);
    }

    #[test]
    fn button_size_defaults_to_medium() {
        let size: ButtonSize = Default::default();
        assert_eq!(size, ButtonSize::Medium);
    }
}
