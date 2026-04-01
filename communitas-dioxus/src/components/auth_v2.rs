// SPDX-License-Identifier: MIT OR Apache-2.0

//! Enhanced authentication screens with Digital Forest Sanctuary theme.
//!
//! Features:
//! - Glass morphism cards with warm jade glow
//! - Animated mesh gradient backgrounds
//! - Smooth micro-interactions
//! - Accessible keyboard navigation

use crate::design_tokens::{
    gradients, motion, palette, radius, semantic, shadow, spacing, typography,
};
use crate::styles_v2::{self as styles, heading, text};
use dioxus::prelude::*;

/// Logo component with subtle glow animation.
#[component]
pub fn Logo(#[props(default = false)] large: bool) -> Element {
    let size = if large { "48px" } else { "32px" };
    let icon_size = if large { "28px" } else { "18px" };

    rsx! {
        div {
            style: format!(
                "width: {}; \
                 height: {}; \
                 background: {}; \
                 border-radius: {}; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 box-shadow: {}; \
                 position: relative;",
                size, size,
                gradients::BUTTON_PRIMARY,
                radius::LG,
                shadow::GLOW_MD
            ),
            // Inner glow effect
            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     background: {}; \
                     border-radius: {}; \
                     opacity: 0.5;",
                    gradients::GLASS_OVERLAY,
                    radius::LG
                ),
            }
            // Logo icon (stylized 'C' or tree symbol)
            span {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: white; \
                     position: relative; \
                     z-index: 1;",
                    icon_size,
                    typography::WEIGHT_BOLD
                ),
                "⬡" // Hexagonal shape suggesting community/network
            }
        }
    }
}

/// Animated background with mesh gradients.
#[component]
pub fn AuthBackground() -> Element {
    rsx! {
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 background: {}; \
                 z-index: -1;",
                gradients::AUTH_BG
            ),
            // Floating orbs for depth
            div {
                style: "position: absolute; \
                        top: 10%; \
                        left: 5%; \
                        width: 400px; \
                        height: 400px; \
                        background: radial-gradient(circle, rgba(16, 185, 129, 0.06) 0%, transparent 70%); \
                        border-radius: 50%; \
                        filter: blur(60px); \
                        animation: float1 20s ease-in-out infinite;",
            }
            div {
                style: "position: absolute; \
                        bottom: 20%; \
                        right: 10%; \
                        width: 300px; \
                        height: 300px; \
                        background: radial-gradient(circle, rgba(6, 78, 59, 0.08) 0%, transparent 70%); \
                        border-radius: 50%; \
                        filter: blur(50px); \
                        animation: float2 25s ease-in-out infinite;",
            }
            div {
                style: "position: absolute; \
                        top: 60%; \
                        left: 30%; \
                        width: 200px; \
                        height: 200px; \
                        background: radial-gradient(circle, rgba(52, 211, 153, 0.04) 0%, transparent 70%); \
                        border-radius: 50%; \
                        filter: blur(40px); \
                        animation: float3 18s ease-in-out infinite;",
            }
            // Subtle grid pattern overlay
            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     background-image: linear-gradient(rgba(52, 211, 153, 0.02) 1px, transparent 1px), \
                                       linear-gradient(90deg, rgba(52, 211, 153, 0.02) 1px, transparent 1px); \
                     background-size: 60px 60px; \
                     mask-image: radial-gradient(ellipse at center, black 0%, transparent 80%);",
                ),
            }
        }
        // Keyframes would be in CSS - here we just define the structure
        style {
            r#"
            @keyframes float1 {{
                0%, 100% {{ transform: translate(0, 0) scale(1); }}
                50% {{ transform: translate(30px, -20px) scale(1.05); }}
            }}
            @keyframes float2 {{
                0%, 100% {{ transform: translate(0, 0) scale(1); }}
                50% {{ transform: translate(-20px, 30px) scale(1.1); }}
            }}
            @keyframes float3 {{
                0%, 100% {{ transform: translate(0, 0) scale(1); }}
                50% {{ transform: translate(15px, 15px) scale(0.95); }}
            }}
            "#
        }
    }
}

/// Auth layout wrapper with centered glass card.
#[component]
pub fn AuthLayoutV2(
    title: String,
    subtitle: Option<String>,
    #[props(default)] footer: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div {
            style: format!(
                "min-height: 100vh; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 padding: {}; \
                 font-family: {};",
                spacing::XL,
                typography::FONT_BODY
            ),

            AuthBackground {}

            div {
                style: format!(
                    "width: 100%; \
                     max-width: 420px; \
                     {}",
                    styles::glass_card_glow()
                ),

                // Card content
                div {
                    style: format!("padding: {};", spacing::XXL),

                    // Header section
                    div {
                        style: format!(
                            "text-align: center; \
                             margin-bottom: {};",
                            spacing::XXL
                        ),

                        // Logo
                        div {
                            style: format!(
                                "display: flex; \
                                 justify-content: center; \
                                 margin-bottom: {};",
                                spacing::XL
                            ),
                            Logo { large: true }
                        }

                        // Brand name
                        div {
                            style: format!(
                                "font-family: {}; \
                                 font-size: {}; \
                                 font-weight: {}; \
                                 letter-spacing: {}; \
                                 color: {}; \
                                 text-transform: uppercase; \
                                 margin-bottom: {};",
                                typography::FONT_DISPLAY,
                                typography::SIZE_XS,
                                typography::WEIGHT_SEMIBOLD,
                                typography::TRACKING_WIDER,
                                semantic::PRIMARY,
                                spacing::BASE
                            ),
                            "Communitas"
                        }

                        // Title
                        h1 {
                            style: format!(
                                "{}; \
                                 font-size: {}; \
                                 margin-bottom: {};",
                                heading::h2(),
                                typography::SIZE_2XL,
                                spacing::SM
                            ),
                            "{title}"
                        }

                        // Subtitle
                        if let Some(sub) = subtitle {
                            p {
                                style: format!(
                                    "{}; \
                                     max-width: 320px; \
                                     margin: 0 auto;",
                                    text::secondary()
                                ),
                                "{sub}"
                            }
                        }
                    }

                    // Main content (form)
                    {children}

                    // Footer
                    if let Some(foot) = footer {
                        div {
                            style: format!(
                                "margin-top: {}; \
                                 padding-top: {}; \
                                 border-top: 1px solid {}; \
                                 text-align: center;",
                                spacing::XL,
                                spacing::XL,
                                semantic::BORDER_SUBTLE
                            ),
                            {foot}
                        }
                    }
                }
            }
        }
    }
}

/// Styled form input with label.
#[component]
pub fn FormField(
    label: String,
    #[props(default = "text".to_string())] input_type: String,
    placeholder: Option<String>,
    value: String,
    #[props(default = false)] disabled: bool,
    #[props(default = false)] required: bool,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let mut focused = use_signal(|| false);

    let border_color = if focused() {
        semantic::PRIMARY
    } else {
        semantic::BORDER_SUBTLE
    };

    let ring = if focused() {
        "box-shadow: 0 0 0 3px rgba(16, 185, 129, 0.15);".to_string()
    } else {
        String::new()
    };

    rsx! {
        div {
            style: format!("margin-bottom: {};", spacing::XL),

            label {
                style: format!(
                    "display: block; \
                     color: {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     margin-bottom: {};",
                    semantic::TEXT_SECONDARY,
                    typography::SIZE_SM,
                    typography::WEIGHT_MEDIUM,
                    spacing::SM
                ),
                "{label}"
                if required {
                    span {
                        style: format!("color: {}; margin-left: {};", palette::ROSE_400, spacing::XXS),
                        "*"
                    }
                }
            }

            input {
                r#type: "{input_type}",
                placeholder: placeholder.unwrap_or_default(),
                value: "{value}",
                disabled: disabled,
                required: required,
                style: format!(
                    "width: 100%; \
                     box-sizing: border-box; \
                     background: {}; \
                     color: {}; \
                     font-family: {}; \
                     font-size: {}; \
                     padding: {} {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     outline: none; \
                     transition: {}; \
                     {} \
                     {}",
                    semantic::BG_TERTIARY,
                    semantic::TEXT_PRIMARY,
                    typography::FONT_BODY,
                    typography::SIZE_BASE,
                    spacing::MD,
                    spacing::BASE,
                    border_color,
                    radius::LG,
                    motion::transition("all"),
                    ring,
                    if disabled { "opacity: 0.5; cursor: not-allowed;" } else { "" }
                ),
                onfocus: move |_| focused.set(true),
                onblur: move |_| focused.set(false),
                oninput: move |evt| oninput.call(evt),
            }
        }
    }
}

/// Styled textarea for multi-line input.
#[component]
pub fn FormTextarea(
    label: String,
    placeholder: Option<String>,
    value: String,
    #[props(default = false)] disabled: bool,
    #[props(default = 4)] rows: u32,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let mut focused = use_signal(|| false);

    let border_color = if focused() {
        semantic::PRIMARY
    } else {
        semantic::BORDER_SUBTLE
    };

    rsx! {
        div {
            style: format!("margin-bottom: {};", spacing::XL),

            label {
                style: format!(
                    "display: block; \
                     color: {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     margin-bottom: {};",
                    semantic::TEXT_SECONDARY,
                    typography::SIZE_SM,
                    typography::WEIGHT_MEDIUM,
                    spacing::SM
                ),
                "{label}"
            }

            textarea {
                placeholder: placeholder.unwrap_or_default(),
                value: "{value}",
                disabled: disabled,
                rows: "{rows}",
                style: format!(
                    "width: 100%; \
                     box-sizing: border-box; \
                     background: {}; \
                     color: {}; \
                     font-family: {}; \
                     font-size: {}; \
                     padding: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     outline: none; \
                     resize: vertical; \
                     min-height: 100px; \
                     transition: {};",
                    semantic::BG_TERTIARY,
                    semantic::TEXT_PRIMARY,
                    typography::FONT_BODY,
                    typography::SIZE_BASE,
                    spacing::BASE,
                    border_color,
                    radius::LG,
                    motion::transition("all")
                ),
                onfocus: move |_| focused.set(true),
                onblur: move |_| focused.set(false),
                oninput: move |evt| oninput.call(evt),
            }
        }
    }
}

/// Styled select dropdown.
#[component]
pub fn FormSelect(
    label: String,
    value: String,
    #[props(default = false)] disabled: bool,
    onchange: EventHandler<FormEvent>,
    children: Element,
) -> Element {
    let mut focused = use_signal(|| false);

    rsx! {
        div {
            style: format!("margin-bottom: {};", spacing::XL),

            label {
                style: format!(
                    "display: block; \
                     color: {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     margin-bottom: {};",
                    semantic::TEXT_SECONDARY,
                    typography::SIZE_SM,
                    typography::WEIGHT_MEDIUM,
                    spacing::SM
                ),
                "{label}"
            }

            div {
                style: "position: relative;",

                select {
                    value: "{value}",
                    disabled: disabled,
                    style: format!(
                        "width: 100%; \
                         appearance: none; \
                         background: {}; \
                         color: {}; \
                         font-family: {}; \
                         font-size: {}; \
                         padding: {} {}; \
                         padding-right: {}; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         outline: none; \
                         cursor: pointer; \
                         transition: {};",
                        semantic::BG_TERTIARY,
                        semantic::TEXT_PRIMARY,
                        typography::FONT_BODY,
                        typography::SIZE_BASE,
                        spacing::MD,
                        spacing::BASE,
                        spacing::XXL,
                        if focused() { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                        radius::LG,
                        motion::transition("all")
                    ),
                    onfocus: move |_| focused.set(true),
                    onblur: move |_| focused.set(false),
                    onchange: move |evt| onchange.call(evt),
                    {children}
                }

                // Dropdown arrow
                div {
                    style: format!(
                        "position: absolute; \
                         right: {}; \
                         top: 50%; \
                         transform: translateY(-50%); \
                         pointer-events: none; \
                         color: {};",
                        spacing::BASE,
                        semantic::TEXT_MUTED
                    ),
                    "▾"
                }
            }
        }
    }
}

/// Primary action button.
#[component]
pub fn PrimaryButton(
    #[props(default = false)] disabled: bool,
    #[props(default = false)] loading: bool,
    #[props(default = "submit".to_string())] button_type: String,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let mut hovered = use_signal(|| false);

    let transform = if hovered() && !disabled && !loading {
        "transform: translateY(-1px);"
    } else {
        ""
    };

    let box_shadow = if hovered() && !disabled && !loading {
        format!("{}, {}", shadow::LG, shadow::GLOW_MD)
    } else {
        format!("{}, {}", shadow::MD, shadow::GLOW_SM)
    };

    rsx! {
        button {
            r#type: "{button_type}",
            disabled: disabled || loading,
            style: format!(
                "width: 100%; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 gap: {}; \
                 background: {}; \
                 color: white; \
                 font-family: {}; \
                 font-size: {}; \
                 font-weight: {}; \
                 padding: {} {}; \
                 border: none; \
                 border-radius: {}; \
                 cursor: {}; \
                 transition: {}; \
                 box-shadow: {}; \
                 {} \
                 {}",
                spacing::SM,
                if hovered() && !disabled && !loading {
                    "linear-gradient(135deg, #34d399 0%, #10b981 100%)"
                } else {
                    gradients::BUTTON_PRIMARY
                },
                typography::FONT_BODY,
                typography::SIZE_BASE,
                typography::WEIGHT_SEMIBOLD,
                spacing::MD,
                spacing::XL,
                radius::LG,
                if disabled || loading { "not-allowed" } else { "pointer" },
                motion::transition("all"),
                box_shadow,
                transform,
                if disabled { "opacity: 0.5;" } else { "" }
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },

            if loading {
                // Spinner
                div {
                    style: "width: 18px; \
                            height: 18px; \
                            border: 2px solid rgba(255,255,255,0.3); \
                            border-top-color: white; \
                            border-radius: 50%; \
                            animation: spin 0.8s linear infinite;",
                }
            }
            {children}
        }

        if loading {
            style {
                r#"
                @keyframes spin {{
                    to {{ transform: rotate(360deg); }}
                }}
                "#
            }
        }
    }
}

/// Secondary/ghost button.
#[component]
pub fn SecondaryButton(
    #[props(default = false)] disabled: bool,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        button {
            r#type: "button",
            disabled: disabled,
            style: format!(
                "display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 gap: {}; \
                 background: {}; \
                 color: {}; \
                 font-family: {}; \
                 font-size: {}; \
                 font-weight: {}; \
                 padding: {} {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: {}; \
                 transition: {};",
                spacing::SM,
                if hovered() { semantic::BG_HOVER } else { "transparent" },
                if hovered() { palette::JADE_400 } else { semantic::TEXT_PRIMARY },
                typography::FONT_BODY,
                typography::SIZE_BASE,
                typography::WEIGHT_MEDIUM,
                spacing::MD,
                spacing::XL,
                if hovered() { semantic::BORDER_STRONG } else { semantic::BORDER_DEFAULT },
                radius::LG,
                if disabled { "not-allowed" } else { "pointer" },
                motion::transition("all")
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| {
                if let Some(handler) = &onclick {
                    handler.call(evt);
                }
            },
            {children}
        }
    }
}

/// Link styled as text.
#[component]
pub fn TextLink(href: String, children: Element) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        a {
            href: "{href}",
            style: format!(
                "color: {}; \
                 text-decoration: {}; \
                 font-weight: {}; \
                 transition: {};",
                if hovered() { palette::JADE_400 } else { semantic::PRIMARY },
                if hovered() { "underline" } else { "none" },
                typography::WEIGHT_MEDIUM,
                motion::transition("color")
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            {children}
        }
    }
}

/// Error banner for form validation.
#[component]
pub fn ErrorBanner(message: String) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 background: rgba(239, 68, 68, 0.1); \
                 color: {}; \
                 font-size: {}; \
                 padding: {} {}; \
                 border-radius: {}; \
                 border: 1px solid rgba(239, 68, 68, 0.2); \
                 margin-bottom: {};",
                spacing::SM,
                palette::ROSE_400,
                typography::SIZE_SM,
                spacing::MD,
                spacing::BASE,
                radius::MD,
                spacing::XL
            ),
            span { "⚠" }
            span { "{message}" }
        }
    }
}

/// Password strength indicator.
#[component]
pub fn PasswordStrength(password: String) -> Element {
    // Simple strength calculation
    let len = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let score = [
        len >= 8,
        len >= 12,
        has_upper,
        has_lower,
        has_digit,
        has_special,
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    let (strength, color) = match score {
        0..=1 => ("Weak", palette::ROSE_400),
        2..=3 => ("Fair", palette::AMBER_400),
        4..=5 => ("Good", palette::JADE_400),
        _ => ("Strong", semantic::SUCCESS),
    };

    if password.is_empty() {
        return rsx! { Fragment {} };
    }

    rsx! {
        div {
            style: format!("margin-top: {}; margin-bottom: {};", spacing::SM, spacing::SM),

            // Strength bars
            div {
                style: format!(
                    "display: flex; \
                     gap: {}; \
                     margin-bottom: {};",
                    spacing::XS,
                    spacing::XS
                ),

                for i in 0..4 {
                    div {
                        style: format!(
                            "flex: 1; \
                             height: 3px; \
                             border-radius: {}; \
                             background: {}; \
                             transition: {};",
                            radius::FULL,
                            if i < (score / 2).max(1) { color } else { semantic::BG_ELEVATED },
                            motion::transition("background")
                        ),
                    }
                }
            }

            // Label
            span {
                style: format!(
                    "font-size: {}; \
                     color: {};",
                    typography::SIZE_XS,
                    color
                ),
                "{strength}"
            }
        }
    }
}
