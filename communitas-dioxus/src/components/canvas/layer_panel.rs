//! Layer panel component for canvas layer management.

use communitas_ui_api::Layer;
use dioxus::prelude::*;

/// Props for the LayerPanel component.
#[derive(Props, Clone, PartialEq)]
pub struct LayerPanelProps {
    /// List of layers.
    pub layers: Vec<Layer>,
    /// Currently active layer ID.
    pub active_layer_id: String,
    /// Whether the panel is collapsed.
    #[props(default = false)]
    pub collapsed: bool,
    /// Callback when layer is selected.
    #[props(default)]
    pub on_select: Option<EventHandler<String>>,
    /// Callback when visibility is toggled.
    #[props(default)]
    pub on_toggle_visibility: Option<EventHandler<String>>,
    /// Callback when lock is toggled.
    #[props(default)]
    pub on_toggle_lock: Option<EventHandler<String>>,
    /// Callback when opacity changes.
    #[props(default)]
    pub on_opacity_change: Option<EventHandler<(String, f32)>>,
    /// Callback when layer is renamed.
    #[props(default)]
    pub on_rename: Option<EventHandler<(String, String)>>,
    /// Callback when layer is deleted.
    #[props(default)]
    pub on_delete: Option<EventHandler<String>>,
    /// Callback when add layer is clicked.
    #[props(default)]
    pub on_add: Option<EventHandler<()>>,
    /// Callback when collapse toggle is clicked.
    #[props(default)]
    pub on_toggle_collapse: Option<EventHandler<()>>,
}

/// Layer panel for managing canvas layers.
#[component]
pub fn LayerPanel(props: LayerPanelProps) -> Element {
    if props.collapsed {
        return rsx! {
            div {
                class: "layer-panel-collapsed w-8 bg-slate-800 rounded-lg p-2",
                button {
                    class: "w-full p-1 hover:bg-slate-700 rounded text-slate-300 transition-colors",
                    title: "Expand layers",
                    aria_label: "Expand layer panel",
                    onclick: {
                        let on_toggle = props.on_toggle_collapse;
                        move |_| {
                            if let Some(handler) = &on_toggle {
                                handler.call(());
                            }
                        }
                    },
                    span { "◀" }
                }
            }
        };
    }

    // Sort layers by z_index (highest first for visual stacking)
    let mut sorted_layers = props.layers.clone();
    sorted_layers.sort_by(|a, b| b.z_index.cmp(&a.z_index));

    rsx! {
        div {
            class: "layer-panel w-56 bg-slate-800 rounded-lg overflow-hidden flex flex-col",
            role: "region",
            aria_label: "Layer panel",
            // Header
            div {
                class: "flex items-center justify-between p-2 border-b border-slate-700",
                h3 {
                    class: "text-sm font-medium text-white",
                    "Layers"
                }
                div {
                    class: "flex items-center gap-1",
                    // Add layer button
                    button {
                        class: "p-1 hover:bg-slate-700 rounded text-slate-300 transition-colors",
                        title: "Add layer",
                        aria_label: "Add new layer",
                        onclick: {
                            let on_add = props.on_add;
                            move |_| {
                                if let Some(handler) = &on_add {
                                    handler.call(());
                                }
                            }
                        },
                        span { "+" }
                    }
                    // Collapse button
                    button {
                        class: "p-1 hover:bg-slate-700 rounded text-slate-300 transition-colors",
                        title: "Collapse panel",
                        aria_label: "Collapse layer panel",
                        onclick: {
                            let on_toggle = props.on_toggle_collapse;
                            move |_| {
                                if let Some(handler) = &on_toggle {
                                    handler.call(());
                                }
                            }
                        },
                        span { "▶" }
                    }
                }
            }
            // Layer list
            div {
                class: "flex-1 overflow-y-auto",
                if sorted_layers.is_empty() {
                    div {
                        class: "p-4 text-center text-slate-500 text-sm",
                        "No layers"
                    }
                } else {
                    for layer in sorted_layers.iter() {
                        LayerItem {
                            key: "{layer.id}",
                            layer: layer.clone(),
                            is_active: layer.id == props.active_layer_id,
                            on_select: {
                                let on_select = props.on_select;
                                let id = layer.id.clone();
                                move |_| {
                                    if let Some(handler) = &on_select {
                                        handler.call(id.clone());
                                    }
                                }
                            },
                            on_toggle_visibility: {
                                let on_toggle = props.on_toggle_visibility;
                                let id = layer.id.clone();
                                move |_| {
                                    if let Some(handler) = &on_toggle {
                                        handler.call(id.clone());
                                    }
                                }
                            },
                            on_toggle_lock: {
                                let on_toggle = props.on_toggle_lock;
                                let id = layer.id.clone();
                                move |_| {
                                    if let Some(handler) = &on_toggle {
                                        handler.call(id.clone());
                                    }
                                }
                            },
                            on_delete: {
                                let on_delete = props.on_delete;
                                let id = layer.id.clone();
                                move |_| {
                                    if let Some(handler) = &on_delete {
                                        handler.call(id.clone());
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Props for individual layer item.
#[derive(Props, Clone, PartialEq)]
struct LayerItemProps {
    layer: Layer,
    is_active: bool,
    on_select: EventHandler<()>,
    on_toggle_visibility: EventHandler<()>,
    on_toggle_lock: EventHandler<()>,
    on_delete: EventHandler<()>,
}

/// Individual layer item in the panel.
#[component]
fn LayerItem(props: LayerItemProps) -> Element {
    let layer = &props.layer;

    let bg_class = if props.is_active {
        "bg-emerald-900/30 border-l-2 border-emerald-500"
    } else {
        "hover:bg-slate-700/50"
    };

    let text_class = if layer.visible {
        "text-white"
    } else {
        "text-slate-500"
    };

    rsx! {
        div {
            class: format!("layer-item flex items-center gap-2 p-2 cursor-pointer transition-colors {}", bg_class),
            role: "button",
            aria_selected: props.is_active,
            aria_label: format!("Layer: {}", layer.name),
            onclick: move |_| props.on_select.call(()),
            // Visibility toggle
            button {
                class: format!(
                    "p-1 rounded transition-colors {}",
                    if layer.visible { "text-slate-300 hover:bg-slate-600" } else { "text-slate-600 hover:bg-slate-600" }
                ),
                title: if layer.visible { "Hide layer" } else { "Show layer" },
                aria_label: if layer.visible { "Hide layer" } else { "Show layer" },
                onclick: move |evt| {
                    evt.stop_propagation();
                    props.on_toggle_visibility.call(());
                },
                span { class: "text-sm", if layer.visible { "👁" } else { "👁‍🗨" } }
            }
            // Lock toggle
            button {
                class: format!(
                    "p-1 rounded transition-colors {}",
                    if layer.locked { "text-amber-500 hover:bg-slate-600" } else { "text-slate-500 hover:bg-slate-600" }
                ),
                title: if layer.locked { "Unlock layer" } else { "Lock layer" },
                aria_label: if layer.locked { "Unlock layer" } else { "Lock layer" },
                onclick: move |evt| {
                    evt.stop_propagation();
                    props.on_toggle_lock.call(());
                },
                span { class: "text-sm", if layer.locked { "🔒" } else { "🔓" } }
            }
            // Layer name
            span {
                class: format!("flex-1 text-sm truncate {}", text_class),
                "{layer.name}"
            }
            // Opacity indicator
            span {
                class: "text-xs text-slate-500",
                "{(layer.opacity * 100.0) as i32}%"
            }
            // Delete button
            button {
                class: "p-1 text-slate-500 hover:text-red-400 hover:bg-slate-600 rounded transition-colors",
                title: "Delete layer",
                aria_label: "Delete layer",
                onclick: move |evt| {
                    evt.stop_propagation();
                    props.on_delete.call(());
                },
                span { class: "text-sm", "🗑" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layer(id: &str, name: &str, z_index: i32) -> Layer {
        Layer {
            id: id.to_string(),
            name: name.to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            z_index,
        }
    }

    #[test]
    fn layers_sort_by_z_index() {
        let mut layers = vec![
            make_layer("1", "Bottom", 0),
            make_layer("2", "Middle", 5),
            make_layer("3", "Top", 10),
        ];

        // Sort highest first (for visual stacking order)
        layers.sort_by(|a, b| b.z_index.cmp(&a.z_index));

        assert_eq!(layers[0].name, "Top");
        assert_eq!(layers[1].name, "Middle");
        assert_eq!(layers[2].name, "Bottom");
    }

    #[test]
    fn layer_visibility_affects_text_class() {
        let visible = true;
        let hidden = false;

        let visible_class = if visible {
            "text-white"
        } else {
            "text-slate-500"
        };
        let hidden_class = if hidden {
            "text-white"
        } else {
            "text-slate-500"
        };

        assert_eq!(visible_class, "text-white");
        assert_eq!(hidden_class, "text-slate-500");
    }

    #[test]
    fn layer_active_state() {
        let layer = make_layer("layer-1", "Background", 0);
        let active_id = "layer-1";
        let is_active = layer.id == active_id;
        assert!(is_active);
    }

    #[test]
    fn opacity_percentage_display() {
        let opacity = 0.75_f32;
        let percent = (opacity * 100.0) as i32;
        assert_eq!(percent, 75);
    }
}
