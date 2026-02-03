//! Enhanced skeleton loader with shimmer animation effect.
//!
//! Provides a polished loading experience with animated shimmer effects
//! that indicate activity while content loads.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::shimmer_skeleton::{ShimmerSkeleton, ShimmerCard};
//!
//! rsx! {
//!     ShimmerCard { lines: 3, show_avatar: true }
//! }
//! ```

use dioxus::prelude::*;
use std::time::Duration;

use crate::design_tokens::{radius, semantic};

/// Sleep function for async delays.
async fn sleep(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::window;

        // Get window - in browser context this should always be available
        let Some(window) = window() else {
            tracing::warn!("window() returned None in WASM context, skipping sleep");
            return;
        };

        // Create timeout promise
        let promise = match js_sys::Promise::new(&mut |resolve, _| match window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                duration.as_millis() as i32,
            ) {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Failed to set timeout: {:?}", e);
            }
        }) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to create promise: {:?}", e);
                return;
            }
        };

        // Await the promise
        match JsFuture::from(promise).await {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Sleep promise rejected: {:?}", e);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(duration).await;
    }
}

/// Shimmer skeleton line component.
#[derive(Props, Clone, PartialEq)]
pub struct ShimmerSkeletonProps {
    /// Width as percentage (0-100).
    #[props(default = 100)]
    pub width_percent: u32,
    /// Height in pixels.
    #[props(default = 16)]
    pub height_px: u32,
    /// Border radius.
    #[props(default = "0.5rem".to_string())]
    pub border_radius: String,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Aria label for accessibility.
    #[props(default = "Loading".to_string())]
    pub aria_label: String,
}

/// Shimmer skeleton with animated gradient effect.
#[component]
pub fn ShimmerSkeleton(props: ShimmerSkeletonProps) -> Element {
    let mut shimmer_offset = use_signal(|| -100.0f32);

    // Continuous shimmer animation (runs once)
    use_future(move || {
        let mut shimmer_offset = shimmer_offset;
        async move {
            loop {
                // Animate from -100 to 200
                let start = -100.0f32;
                let end = 200.0f32;
                let duration_ms = 1500u64;
                let steps = 60usize;
                let step_duration = duration_ms / steps as u64;
                let step_size = (end - start) / steps as f32;

                for i in 0..=steps {
                    shimmer_offset.set(start + step_size * i as f32);
                    sleep(Duration::from_millis(step_duration)).await;
                }

                // Reset
                shimmer_offset.set(-100.0);
            }
        }
    });

    let shimmer_gradient = format!(
        "linear-gradient(90deg, \
         rgba(255,255,255,0.03) 0%, \
         rgba(255,255,255,0.08) {}%, \
         rgba(255,255,255,0.03) 100%)",
        shimmer_offset()
    );

    rsx! {
        div {
            style: format!(
                "width: {}%; \
                 height: {}px; \
                 background: {}; \
                 border-radius: {}; \
                 position: relative; \
                 overflow: hidden;",
                props.width_percent,
                props.height_px,
                semantic::BG_TERTIARY,
                props.border_radius
            ),
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "{props.aria_label}",

            // Shimmer overlay
            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     background: {};",
                    shimmer_gradient
                ),
            }
        }
    }
}

/// Shimmer circle for avatars and icons.
#[derive(Props, Clone, PartialEq)]
pub struct ShimmerCircleProps {
    /// Size in pixels.
    #[props(default = 40)]
    pub size_px: u32,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Aria label.
    #[props(default = "Loading".to_string())]
    pub aria_label: String,
}

/// Circular shimmer skeleton.
#[component]
pub fn ShimmerCircle(props: ShimmerCircleProps) -> Element {
    let mut shimmer_offset = use_signal(|| -100.0f32);

    use_future(move || {
        let mut shimmer_offset = shimmer_offset;
        async move {
            loop {
                // Animate from -100 to 200
                let start = -100.0f32;
                let end = 200.0f32;
                let duration_ms = 1500u64;
                let steps = 60usize;
                let step_duration = duration_ms / steps as u64;
                let step_size = (end - start) / steps as f32;

                for i in 0..=steps {
                    shimmer_offset.set(start + step_size * i as f32);
                    sleep(Duration::from_millis(step_duration)).await;
                }

                // Reset
                shimmer_offset.set(-100.0);
            }
        }
    });

    let shimmer_gradient = format!(
        "linear-gradient(90deg, \
         rgba(255,255,255,0.03) 0%, \
         rgba(255,255,255,0.08) {}%, \
         rgba(255,255,255,0.03) 100%)",
        shimmer_offset()
    );

    rsx! {
        div {
            style: format!(
                "width: {}px; \
                 height: {}px; \
                 background: {}; \
                 border-radius: 50%; \
                 position: relative; \
                 overflow: hidden;",
                props.size_px,
                props.size_px,
                semantic::BG_TERTIARY
            ),
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "{props.aria_label}",

            div {
                style: format!(
                    "position: absolute; \
                     inset: 0; \
                     background: {};",
                    shimmer_gradient
                ),
            }
        }
    }
}

/// Shimmer card with multiple lines.
#[derive(Props, Clone, PartialEq)]
pub struct ShimmerCardProps {
    /// Number of content lines.
    #[props(default = 3)]
    pub lines: usize,
    /// Show avatar circle.
    #[props(default = false)]
    pub show_avatar: bool,
    /// Show footer section.
    #[props(default = false)]
    pub show_footer: bool,
    /// Card height.
    #[props(default = "auto".to_string())]
    pub height: String,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Aria label.
    #[props(default = "Loading card".to_string())]
    pub aria_label: String,
}

/// Card-shaped shimmer skeleton.
#[component]
pub fn ShimmerCard(props: ShimmerCardProps) -> Element {
    rsx! {
        div {
            style: format!(
                "background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 padding: 1rem; \
                 height: {};",
                semantic::BG_SECONDARY,
                semantic::BORDER_DEFAULT,
                radius::LG,
                props.height
            ),
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "{props.aria_label}",

            // Header with optional avatar
            if props.show_avatar {
                div {
                    style: "display: flex; align-items: center; gap: 0.75rem; margin-bottom: 1rem;",
                    ShimmerCircle { size_px: 40, aria_label: "".to_string() }
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; gap: 0.5rem;",
                        ShimmerSkeleton { width_percent: 40, height_px: 16, border_radius: radius::SM.to_string(), aria_label: "".to_string() }
                        ShimmerSkeleton { width_percent: 25, height_px: 12, border_radius: radius::SM.to_string(), aria_label: "".to_string() }
                    }
                }
            }

            // Content lines
            div {
                style: "display: flex; flex-direction: column; gap: 0.5rem;",
                for i in 0..props.lines {
                    ShimmerSkeleton {
                        key: "{i}",
                        width_percent: if i == props.lines - 1 { 75 } else { 100 },
                        height_px: 14,
                        border_radius: radius::SM.to_string(),
                        aria_label: "".to_string()
                    }
                }
            }

            // Optional footer
            if props.show_footer {
                div {
                    style: format!(
                        "display: flex; \
                         justify-content: space-between; \
                         margin-top: 1rem; \
                         padding-top: 1rem; \
                         border-top: 1px solid {};",
                        semantic::BORDER_DEFAULT
                    ),
                    ShimmerSkeleton { width_percent: 30, height_px: 14, border_radius: radius::SM.to_string(), aria_label: "".to_string() }
                    ShimmerSkeleton { width_percent: 20, height_px: 14, border_radius: radius::SM.to_string(), aria_label: "".to_string() }
                }
            }
        }
    }
}

/// Shimmer list with multiple items.
#[derive(Props, Clone, PartialEq)]
pub struct ShimmerListProps {
    /// Number of items.
    #[props(default = 5)]
    pub count: usize,
    /// Show avatar for each item.
    #[props(default = true)]
    pub show_avatar: bool,
    /// Gap between items.
    #[props(default = "0.75rem".to_string())]
    pub gap: String,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Aria label.
    #[props(default = "Loading list".to_string())]
    pub aria_label: String,
}

/// List of shimmer skeleton items.
#[component]
pub fn ShimmerList(props: ShimmerListProps) -> Element {
    rsx! {
        div {
            style: format!("display: flex; flex-direction: column; gap: {};", props.gap),
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "{props.aria_label}",

            for i in 0..props.count {
                div {
                    key: "{i}",
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: 0.75rem; \
                         padding: 0.75rem; \
                         background: {}; \
                         border-radius: {};",
                        semantic::BG_SECONDARY,
                        radius::MD
                    ),

                    if props.show_avatar {
                        ShimmerCircle { size_px: 36, aria_label: "".to_string() }
                    }

                    div {
                        style: "flex: 1; display: flex; flex-direction: column; gap: 0.375rem;",
                        ShimmerSkeleton {
                            width_percent: if i % 2 == 0 { 75 } else { 60 },
                            height_px: 14,
                            border_radius: radius::SM.to_string(),
                            aria_label: "".to_string()
                        }
                        ShimmerSkeleton {
                            width_percent: 50,
                            height_px: 12,
                            border_radius: radius::SM.to_string(),
                            aria_label: "".to_string()
                        }
                    }
                }
            }
        }
    }
}

/// Shimmer grid layout.
#[derive(Props, Clone, PartialEq)]
pub struct ShimmerGridProps {
    /// Number of items.
    #[props(default = 6)]
    pub count: usize,
    /// Number of columns.
    #[props(default = 3)]
    pub columns: usize,
    /// Gap between items.
    #[props(default = "1rem".to_string())]
    pub gap: String,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Aria label.
    #[props(default = "Loading grid".to_string())]
    pub aria_label: String,
}

/// Grid of shimmer skeleton cards.
#[component]
pub fn ShimmerGrid(props: ShimmerGridProps) -> Element {
    let grid_template = format!("repeat({}, 1fr)", props.columns);

    rsx! {
        div {
            style: format!(
                "display: grid; \
                 grid-template-columns: {}; \
                 gap: {};",
                grid_template,
                props.gap
            ),
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "{props.aria_label}",

            for i in 0..props.count {
                ShimmerCard {
                    key: "{i}",
                    lines: 2,
                    aria_label: "".to_string()
                }
            }
        }
    }
}

/// Shimmer text block with multiple lines.
#[derive(Props, Clone, PartialEq)]
pub struct ShimmerTextProps {
    /// Number of lines.
    #[props(default = 4)]
    pub lines: usize,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Aria label.
    #[props(default = "Loading text".to_string())]
    pub aria_label: String,
}

/// Text block shimmer skeleton.
#[component]
pub fn ShimmerText(props: ShimmerTextProps) -> Element {
    let widths = [100, 100, 85, 75, 80, 65];

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 0.5rem;",
            class: "{props.class}",
            role: "status",
            aria_busy: "true",
            aria_label: "{props.aria_label}",

            for i in 0..props.lines {
                ShimmerSkeleton {
                    key: "{i}",
                    width_percent: widths[i % widths.len()],
                    height_px: 14,
                    border_radius: radius::SM.to_string(),
                    aria_label: "".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shimmer_skeleton_default_props() {
        let props = ShimmerSkeletonProps::builder().build();
        assert_eq!(props.width_percent, 100);
        assert_eq!(props.height_px, 16);
        assert_eq!(props.aria_label, "Loading");
    }

    #[test]
    fn shimmer_card_default_props() {
        let props = ShimmerCardProps::builder().build();
        assert_eq!(props.lines, 3);
        assert!(!props.show_avatar);
        assert!(!props.show_footer);
    }

    #[test]
    fn shimmer_list_default_props() {
        let props = ShimmerListProps::builder().build();
        assert_eq!(props.count, 5);
        assert!(props.show_avatar);
    }
}
