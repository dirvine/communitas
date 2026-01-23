//! Tour overlay component for the onboarding experience.
//!
//! Provides a spotlight overlay that highlights UI elements with a tooltip
//! containing tour step information and navigation controls.

use dioxus::prelude::*;

use super::types::{TOUR_STEPS, TooltipPosition, TourState, TourStep};

/// Props for the main tour overlay component.
#[derive(Clone, PartialEq, Props)]
pub struct TourOverlayProps {
    /// Current tour state signal.
    pub tour_state: Signal<TourState>,
    /// Callback when next button is clicked.
    pub on_next: EventHandler<()>,
    /// Callback when previous button is clicked.
    pub on_previous: EventHandler<()>,
    /// Callback when skip button is clicked.
    pub on_skip: EventHandler<()>,
    /// Callback when finish button is clicked.
    pub on_finish: EventHandler<()>,
}

/// Bounding rectangle for spotlight cutout.
#[derive(Clone, Copy, PartialEq, Props)]
pub struct SpotlightCutoutProps {
    /// X coordinate of the spotlight.
    pub x: f32,
    /// Y coordinate of the spotlight.
    pub y: f32,
    /// Width of the spotlight area.
    pub width: f32,
    /// Height of the spotlight area.
    pub height: f32,
}

/// Props for the tour tooltip component.
#[derive(Clone, PartialEq, Props)]
pub struct TourTooltipProps {
    /// Current step information.
    pub step: TourStep,
    /// Current step index (0-based).
    pub step_index: usize,
    /// Total number of steps.
    pub total_steps: usize,
    /// Tooltip position relative to target.
    pub position: TooltipPosition,
    /// Callback when next button is clicked.
    pub on_next: EventHandler<()>,
    /// Callback when previous button is clicked.
    pub on_previous: EventHandler<()>,
    /// Callback when skip button is clicked.
    pub on_skip: EventHandler<()>,
    /// Callback when finish button is clicked.
    pub on_finish: EventHandler<()>,
}

/// Main tour overlay component with backdrop and spotlight.
#[component]
pub fn TourOverlay(props: TourOverlayProps) -> Element {
    let tour_state = props.tour_state.read();

    // Get current step from the tour state
    let step_index = tour_state.current_step;
    let current_step = TOUR_STEPS.get(step_index).copied();

    // Handle keyboard navigation
    let on_keydown = move |evt: KeyboardEvent| {
        let key = evt.key();
        if key == Key::Escape {
            props.on_skip.call(());
        } else if key == Key::Enter {
            if step_index >= TOUR_STEPS.len() - 1 {
                props.on_finish.call(());
            } else {
                props.on_next.call(());
            }
        }
    };

    // Return empty if no current step or tour not active
    let Some(step) = current_step else {
        return rsx! {};
    };

    if !tour_state.active {
        return rsx! {};
    }

    // Default target rect for centered modal (full viewport center)
    let target_rect = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);

    rsx! {
        div {
            class: "tour-overlay",
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "tour-title",
            tabindex: "-1",
            onkeydown: on_keydown,

            // Backdrop with spotlight cutout (only if targeting an element)
            if step.target_selector.is_some() {
                SpotlightCutout {
                    x: target_rect.0,
                    y: target_rect.1,
                    width: target_rect.2,
                    height: target_rect.3,
                }
            }

            // Tooltip
            TourTooltip {
                step: step,
                step_index: step_index,
                total_steps: TOUR_STEPS.len(),
                position: step.position,
                on_next: props.on_next,
                on_previous: props.on_previous,
                on_skip: props.on_skip,
                on_finish: props.on_finish,
            }

            // ARIA live region for step announcements
            div {
                class: "sr-only",
                aria_live: "polite",
                "Step {step_index + 1} of {TOUR_STEPS.len()}: {step.title}"
            }
        }
    }
}

/// Spotlight cutout component that creates a visible hole in the backdrop.
#[component]
pub fn SpotlightCutout(props: SpotlightCutoutProps) -> Element {
    // Calculate clip-path for spotlight effect
    let padding = 8.0;
    let x = props.x - padding;
    let y = props.y - padding;
    let width = props.width + (padding * 2.0);
    let height = props.height + (padding * 2.0);

    let clip_path = format!(
        "polygon(0% 0%, 0% 100%, 100% 100%, 100% 0%, 0% 0%, \
         {}px {}px, {}px {}px, {}px {}px, {}px {}px, {}px {}px)",
        x,
        y,
        x + width,
        y,
        x + width,
        y + height,
        x,
        y + height,
        x,
        y
    );

    rsx! {
        div {
            class: "tour-spotlight",
            style: "clip-path: {clip_path}; -webkit-clip-path: {clip_path};",
        }
    }
}

/// Tour tooltip component with step information and navigation controls.
#[component]
pub fn TourTooltip(props: TourTooltipProps) -> Element {
    let is_first = props.step_index == 0;
    let is_last = props.step_index >= props.total_steps - 1;

    // Calculate tooltip position class based on position
    let position_class = match props.position {
        TooltipPosition::Top => "tour-tooltip-top",
        TooltipPosition::Bottom => "tour-tooltip-bottom",
        TooltipPosition::Left => "tour-tooltip-left",
        TooltipPosition::Right => "tour-tooltip-right",
        TooltipPosition::Center => "tour-tooltip-center",
    };

    rsx! {
        div {
            class: "tour-tooltip {position_class}",

            // Tooltip header
            div {
                class: "tour-tooltip-header",
                h3 {
                    id: "tour-title",
                    class: "tour-tooltip-title",
                    "{props.step.title}"
                }
                span {
                    class: "tour-tooltip-counter",
                    "{props.step_index + 1} of {props.total_steps}"
                }
            }

            // Tooltip content
            div {
                class: "tour-tooltip-content",
                p { "{props.step.description}" }
            }

            // Tooltip actions
            div {
                class: "tour-tooltip-actions",

                // Skip button (always visible)
                button {
                    class: "tour-button tour-button-secondary",
                    r#type: "button",
                    onclick: move |_| props.on_skip.call(()),
                    "Skip Tour"
                }

                // Navigation buttons
                div {
                    class: "tour-navigation",

                    // Previous button (hidden on first step)
                    if !is_first {
                        button {
                            class: "tour-button tour-button-secondary",
                            r#type: "button",
                            onclick: move |_| props.on_previous.call(()),
                            "Previous"
                        }
                    }

                    // Next or Finish button
                    if is_last {
                        button {
                            class: "tour-button tour-button-primary",
                            r#type: "button",
                            autofocus: true,
                            onclick: move |_| props.on_finish.call(()),
                            "Finish"
                        }
                    } else {
                        button {
                            class: "tour-button tour-button-primary",
                            r#type: "button",
                            autofocus: true,
                            onclick: move |_| props.on_next.call(()),
                            "Next"
                        }
                    }
                }
            }
        }
    }
}

// Tour navigation helper functions

/// Get the current tour state signal from context.
pub fn use_tour_state() -> Signal<TourState> {
    use_context::<Signal<TourState>>()
}

/// Advance to the next step in the tour.
pub fn next_step(state: &mut TourState) {
    state.next();
}

/// Go back to the previous step in the tour.
pub fn previous_step(state: &mut TourState) {
    state.previous();
}

/// Skip the tour and mark it as skipped.
pub fn skip_tour(state: &mut TourState) {
    state.skip();
}

/// Complete the tour.
pub fn finish_tour(state: &mut TourState) {
    state.complete();
}
