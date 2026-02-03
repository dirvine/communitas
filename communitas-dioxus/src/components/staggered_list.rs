//! Staggered list component with entrance animations.
//!
//! Animates list items with a cascading delay effect for a polished,
//! professional appearance.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::staggered_list::{StaggeredList, StaggeredListItem};
//!
//! rsx! {
//!     StaggeredList {
//!         items: vec!["Item 1", "Item 2", "Item 3"],
//!         render_item: |item| rsx! { span { "{item}" } },
//!         stagger_ms: 50.0,
//!     }
//! }
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::animations::springs;

/// Properties for the StaggeredList component.
#[derive(Props, Clone, PartialEq)]
pub struct StaggeredListProps<T: Clone + PartialEq + 'static> {
    /// List of items to render.
    pub items: Vec<T>,
    /// Render function for each item.
    pub render_item: Callback<T, Element>,
    /// Delay between each item animation (in milliseconds).
    #[props(default = 50.0)]
    pub stagger_ms: f32,
    /// Initial delay before starting animations.
    #[props(default = 0.0)]
    pub initial_delay_ms: f32,
    /// Animation direction.
    #[props(default = AnimationDirection::Up)]
    pub direction: AnimationDirection,
    /// Additional CSS classes for the container.
    #[props(default = String::new())]
    pub class: String,
    /// Gap between items.
    #[props(default = None)]
    pub gap: Option<String>,
}

/// Animation direction for list items.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum AnimationDirection {
    /// Animate from bottom to top (slide up).
    #[default]
    Up,
    /// Animate from top to bottom (slide down).
    Down,
    /// Animate from left to right (slide right).
    Left,
    /// Animate from right to left (slide left).
    Right,
    /// Fade in without translation.
    Fade,
}

/// Staggered list with entrance animations.
///
/// Each item animates in sequence with a configurable delay between items.
#[component]
pub fn StaggeredList<T: Clone + PartialEq + 'static>(props: StaggeredListProps<T>) -> Element {
    let gap = props.gap.clone().unwrap_or_else(|| "0".to_string());

    rsx! {
        div {
            style: format!("display: flex; flex-direction: column; gap: {};", gap),
            class: "{props.class}",

            for (index, item) in props.items.iter().enumerate() {
                StaggeredListItem {
                    key: "{index}",
                    index,
                    item: item.clone(),
                    render_item: props.render_item,
                    stagger_ms: props.stagger_ms,
                    initial_delay_ms: props.initial_delay_ms,
                    direction: props.direction,
                }
            }
        }
    }
}

/// Properties for individual staggered list items.
#[derive(Props, Clone, PartialEq)]
pub struct StaggeredListItemProps<T: Clone + PartialEq + 'static> {
    /// Item index for calculating delay.
    pub index: usize,
    /// The item data.
    pub item: T,
    /// Render function.
    pub render_item: Callback<T, Element>,
    /// Stagger delay between items.
    pub stagger_ms: f32,
    /// Initial delay before first item.
    pub initial_delay_ms: f32,
    /// Animation direction.
    pub direction: AnimationDirection,
}

/// Individual list item with entrance animation.
#[component]
fn StaggeredListItem<T: Clone + PartialEq + 'static>(props: StaggeredListItemProps<T>) -> Element {
    // Determine initial values based on direction
    let (initial_tx, initial_ty) = match props.direction {
        AnimationDirection::Up => (0.0f32, 20.0f32),
        AnimationDirection::Down => (0.0f32, -20.0f32),
        AnimationDirection::Left => (20.0f32, 0.0f32),
        AnimationDirection::Right => (-20.0f32, 0.0f32),
        AnimationDirection::Fade => (0.0f32, 0.0f32),
    };

    let mut opacity = use_motion(0.0f32);
    let mut translate_x = use_motion(initial_tx);
    let mut translate_y = use_motion(initial_ty);
    let mut scale = use_motion(1.0f32);

    // Calculate total delay based on index
    let total_delay_ms = (props.initial_delay_ms + (props.index as f32 * props.stagger_ms)) as u64;

    // Animate to final values
    use_effect(move || {
        use std::time::Duration;
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(total_delay_ms)),
        );
        translate_x.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(total_delay_ms)),
        );
        translate_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(total_delay_ms)),
        );
        scale.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(total_delay_ms)),
        );
    });

    let transform = if initial_tx != 0.0 || initial_ty != 0.0 {
        format!(
            "translate({}px, {}px)",
            translate_x.get_value(),
            translate_y.get_value()
        )
    } else {
        String::new()
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

            {props.render_item.call(props.item)}
        }
    }
}

/// Animated list item with hover effects.
#[derive(Props, Clone, PartialEq)]
pub struct AnimatedListItemProps {
    /// Item content.
    pub children: Element,
    /// Click handler.
    #[props(default = None)]
    pub on_click: Option<EventHandler<()>>,
    /// Whether the item is selected.
    #[props(default = false)]
    pub selected: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
    /// Index for stagger animation.
    #[props(default = 0)]
    pub index: usize,
    /// Stagger delay.
    #[props(default = 50.0)]
    pub stagger_ms: f32,
}

/// List item with entrance and hover animations.
#[component]
pub fn AnimatedListItem(props: AnimatedListItemProps) -> Element {
    let mut opacity = use_motion(0.0f32);
    let mut translate_y = use_motion(20.0f32);
    let mut scale = use_motion(1.0f32);
    let mut bg_opacity = use_motion(0.0f32);

    // Entrance animation
    use_effect(move || {
        use std::time::Duration;
        let delay_ms = (props.index as f32 * props.stagger_ms) as u64;
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(delay_ms)),
        );
        translate_y.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(springs::list_item_enter()))
                .with_delay(Duration::from_millis(delay_ms)),
        );
    });

    let handle_press_start = move |_| {
        if props.on_click.is_some() {
            scale.animate_to(
                0.98,
                AnimationConfig::new(AnimationMode::Spring(springs::card_press())),
            );
        }
    };

    let handle_press_end = move |_| {
        if let Some(handler) = props.on_click {
            scale.animate_to(
                1.0,
                AnimationConfig::new(AnimationMode::Spring(springs::button_release())),
            );
            handler.call(());
        }
    };

    let handle_mouse_enter = move |_| {
        bg_opacity.animate_to(
            0.05,
            AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
        );
    };

    let handle_mouse_leave = move |_| {
        scale.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
        );
        bg_opacity.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(springs::card_hover())),
        );
    };

    let cursor = if props.on_click.is_some() {
        "pointer"
    } else {
        "default"
    };

    rsx! {
        div {
            style: format!(
                "opacity: {}; \
                 transform: translateY({}px) scale({}); \
                 background: rgba(255,255,255,{}); \
                 cursor: {}; \
                 will-change: transform, opacity;",
                opacity.get_value(),
                translate_y.get_value(),
                scale.get_value(),
                if props.selected { 0.1 } else { bg_opacity.get_value() },
                cursor
            ),
            onmousedown: handle_press_start,
            onmouseup: handle_press_end,
            onmouseenter: handle_mouse_enter,
            onmouseleave: handle_mouse_leave,
            class: "{props.class}",

            {props.children}
        }
    }
}

/// Grid with staggered item animations.
#[derive(Props, Clone, PartialEq)]
pub struct StaggeredGridProps<T: Clone + PartialEq + 'static> {
    /// Grid items.
    pub items: Vec<T>,
    /// Render function for each item.
    pub render_item: Callback<T, Element>,
    /// Number of columns.
    #[props(default = 3)]
    pub columns: usize,
    /// Gap between items.
    #[props(default = "1rem".to_string())]
    pub gap: String,
    /// Stagger delay between items.
    #[props(default = 50.0)]
    pub stagger_ms: f32,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Grid with staggered entrance animations.
#[component]
pub fn StaggeredGrid<T: Clone + PartialEq + 'static>(props: StaggeredGridProps<T>) -> Element {
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

            for (index, item) in props.items.iter().enumerate() {
                StaggeredListItem {
                    key: "{index}",
                    index,
                    item: item.clone(),
                    render_item: props.render_item,
                    stagger_ms: props.stagger_ms,
                    initial_delay_ms: 0.0,
                    direction: AnimationDirection::Up,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_direction_defaults_to_up() {
        let direction: AnimationDirection = Default::default();
        assert_eq!(direction, AnimationDirection::Up);
    }

    #[test]
    fn staggered_list_props_default() {
        // Props can only be tested within a Dioxus runtime
        // Verify the component compiles correctly by checking AnimationDirection
        let direction: AnimationDirection = Default::default();
        assert_eq!(direction, AnimationDirection::Up);
    }
}
