//! Canvas toolbar component for tool and drawing option selection.

use communitas_ui_api::ToolType;
use dioxus::prelude::*;

/// Props for the CanvasToolbar component.
#[derive(Props, Clone, PartialEq)]
pub struct CanvasToolbarProps {
    /// Currently selected tool.
    #[props(default)]
    pub selected_tool: ToolType,
    /// Callback when tool is selected.
    #[props(default)]
    pub on_tool_change: Option<EventHandler<ToolType>>,
    /// Callback when undo is clicked.
    #[props(default)]
    pub on_undo: Option<EventHandler<()>>,
    /// Callback when redo is clicked.
    #[props(default)]
    pub on_redo: Option<EventHandler<()>>,
    /// Whether undo is available.
    #[props(default = true)]
    pub can_undo: bool,
    /// Whether redo is available.
    #[props(default = false)]
    pub can_redo: bool,
}

/// Canvas toolbar for tool selection and drawing options.
#[component]
pub fn CanvasToolbar(props: CanvasToolbarProps) -> Element {
    let selected = props.selected_tool;

    rsx! {
        div {
            class: "canvas-toolbar flex items-center gap-1 p-2 bg-slate-800 rounded-lg",
            role: "toolbar",
            aria_label: "Canvas tools",
            // Drawing tools
            div {
                class: "flex items-center gap-1 pr-2 border-r border-slate-700",
                for tool in ToolType::all() {
                    ToolButton {
                        key: "{tool:?}",
                        tool: *tool,
                        selected: *tool == selected,
                        on_click: {
                            let on_change = props.on_tool_change;
                            move |t| {
                                if let Some(handler) = &on_change {
                                    handler.call(t);
                                }
                            }
                        },
                    }
                }
            }
            // Undo/Redo
            div {
                class: "flex items-center gap-1 pl-2",
                button {
                    class: format!(
                        "p-2 rounded-lg transition-colors {}",
                        if props.can_undo {
                            "hover:bg-slate-700 text-slate-300"
                        } else {
                            "text-slate-600 cursor-not-allowed"
                        }
                    ),
                    disabled: !props.can_undo,
                    title: "Undo (Ctrl+Z)",
                    aria_label: "Undo",
                    onclick: {
                        let on_undo = props.on_undo;
                        move |_| {
                            if let Some(handler) = &on_undo {
                                handler.call(());
                            }
                        }
                    },
                    span { class: "text-lg", "↶" }
                }
                button {
                    class: format!(
                        "p-2 rounded-lg transition-colors {}",
                        if props.can_redo {
                            "hover:bg-slate-700 text-slate-300"
                        } else {
                            "text-slate-600 cursor-not-allowed"
                        }
                    ),
                    disabled: !props.can_redo,
                    title: "Redo (Ctrl+Y)",
                    aria_label: "Redo",
                    onclick: {
                        let on_redo = props.on_redo;
                        move |_| {
                            if let Some(handler) = &on_redo {
                                handler.call(());
                            }
                        }
                    },
                    span { class: "text-lg", "↷" }
                }
            }
        }
    }
}

/// Props for individual tool button.
#[derive(Props, Clone, PartialEq)]
struct ToolButtonProps {
    tool: ToolType,
    selected: bool,
    on_click: EventHandler<ToolType>,
}

/// Individual tool button in the toolbar.
#[component]
fn ToolButton(props: ToolButtonProps) -> Element {
    let tool = props.tool;
    let icon = tool_icon(tool);
    let label = tool.label();

    let bg_class = if props.selected {
        "bg-emerald-600 text-white"
    } else {
        "hover:bg-slate-700 text-slate-300"
    };

    rsx! {
        button {
            class: format!("p-2 rounded-lg transition-colors {}", bg_class),
            title: "{label}",
            aria_label: "{label}",
            aria_pressed: props.selected,
            onclick: move |_| props.on_click.call(tool),
            span { class: "text-lg", "{icon}" }
        }
    }
}

/// Get the emoji icon for a tool type.
#[allow(dead_code)]
fn tool_icon(tool: ToolType) -> &'static str {
    match tool {
        ToolType::Select => "↖",
        ToolType::Pen => "✏",
        ToolType::Brush => "🖌",
        ToolType::Eraser => "🧹",
        ToolType::Rectangle => "▭",
        ToolType::Ellipse => "○",
        ToolType::Text => "T",
        ToolType::Pan => "✋",
        ToolType::Zoom => "🔍",
    }
}

/// Color picker for stroke/fill colors.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerProps {
    /// Current color value (CSS color string).
    pub value: String,
    /// Label for the picker.
    #[props(default = "Color")]
    pub label: &'static str,
    /// Callback when color changes.
    #[props(default)]
    pub on_change: Option<EventHandler<String>>,
}

#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    let colors = [
        "#000000", "#ffffff", "#ff0000", "#00ff00", "#0000ff", "#ffff00", "#ff00ff", "#00ffff",
        "#808080", "#c0c0c0",
    ];

    rsx! {
        div {
            class: "color-picker",
            label {
                class: "block text-xs font-medium text-slate-400 mb-1",
                "{props.label}"
            }
            div {
                class: "flex flex-wrap gap-1",
                for color in colors {
                    button {
                        key: "{color}",
                        class: format!(
                            "w-6 h-6 rounded border-2 transition-transform {}",
                            if props.value == color {
                                "border-emerald-500 scale-110"
                            } else {
                                "border-slate-600 hover:scale-105"
                            }
                        ),
                        style: "background-color: {color}",
                        title: "{color}",
                        aria_label: "Select color {color}",
                        onclick: {
                            let on_change = props.on_change;
                            let color = color.to_string();
                            move |_| {
                                if let Some(handler) = &on_change {
                                    handler.call(color.clone());
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Stroke width slider.
#[derive(Props, Clone, PartialEq)]
pub struct StrokeWidthSliderProps {
    /// Current stroke width.
    pub value: f32,
    /// Callback when width changes.
    #[props(default)]
    pub on_change: Option<EventHandler<f32>>,
}

#[component]
pub fn StrokeWidthSlider(props: StrokeWidthSliderProps) -> Element {
    rsx! {
        div {
            class: "stroke-width-slider",
            label {
                class: "block text-xs font-medium text-slate-400 mb-1",
                "Stroke: {props.value:.1}px"
            }
            input {
                r#type: "range",
                class: "w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer",
                min: "1",
                max: "20",
                step: "0.5",
                value: "{props.value}",
                aria_label: "Stroke width",
                oninput: {
                    let on_change = props.on_change;
                    move |evt: Event<FormData>| {
                        if let Ok(val) = evt.value().parse::<f32>()
                            && let Some(handler) = &on_change
                        {
                            handler.call(val);
                        }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_icons_all_defined() {
        for tool in ToolType::all() {
            let icon = tool_icon(*tool);
            assert!(!icon.is_empty(), "Tool {:?} should have an icon", tool);
        }
    }

    #[test]
    fn tool_type_labels_match() {
        assert_eq!(ToolType::Select.label(), "Select");
        assert_eq!(ToolType::Pen.label(), "Pen");
        assert_eq!(ToolType::Brush.label(), "Brush");
        assert_eq!(ToolType::Eraser.label(), "Eraser");
    }

    #[test]
    fn default_props() {
        let tool = ToolType::default();
        assert_eq!(tool, ToolType::Select);
    }
}
