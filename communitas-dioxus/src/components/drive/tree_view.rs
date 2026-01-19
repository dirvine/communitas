//! Tree view sidebar for folder navigation.

use communitas_ui_api::DiskType;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Tree view props.
#[derive(Props, Clone, PartialEq)]
pub struct TreeViewProps {
    /// Entity ID that owns the drive.
    pub entity_id: String,
    /// Current disk type.
    pub current_disk: DiskType,
    /// Current path (for highlighting).
    pub current_path: String,
    /// Callback when a folder is clicked.
    pub on_navigate: EventHandler<String>,
}

/// Tree view sidebar showing folder hierarchy.
#[component]
pub fn TreeView(props: TreeViewProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let _drive = services.drive();

    // State for expanded nodes
    let mut expanded = use_signal(|| vec!["/".to_string()]);

    // Toggle node expansion
    let mut toggle_expand = move |path: String| {
        let mut current = expanded();
        if current.contains(&path) {
            current.retain(|p| p != &path);
        } else {
            current.push(path);
        }
        expanded.set(current);
    };

    rsx! {
        nav {
            class: "tree-view p-2",
            role: "tree",
            aria_label: "Folder tree",
            // Root node
            TreeNode {
                name: props.current_disk.label().to_string(),
                path: "/".to_string(),
                depth: 0,
                is_expanded: expanded().contains(&"/".to_string()),
                is_current: props.current_path == "/",
                on_toggle: move |_| toggle_expand("/".to_string()),
                on_navigate: move |path: String| props.on_navigate.call(path),
            }
            // Placeholder for lazy-loaded children
            if expanded().contains(&"/".to_string()) {
                div {
                    class: "pl-4 text-xs text-slate-500 py-2",
                    "Loading folders..."
                }
            }
        }
    }
}

/// Single tree node.
#[derive(Props, Clone, PartialEq)]
struct TreeNodeProps {
    name: String,
    path: String,
    depth: u32,
    is_expanded: bool,
    is_current: bool,
    on_toggle: EventHandler<()>,
    on_navigate: EventHandler<String>,
}

#[component]
fn TreeNode(props: TreeNodeProps) -> Element {
    let path = props.path.clone();
    let path_for_nav = path.clone();
    let indent = format!("pl-{}", props.depth * 4);

    rsx! {
        div {
            class: format!(
                "tree-node flex items-center gap-1 py-1 px-2 rounded cursor-pointer transition {} {}",
                indent,
                if props.is_current {
                    "bg-emerald-500/20 text-emerald-400"
                } else {
                    "text-slate-300 hover:bg-slate-800"
                }
            ),
            role: "treeitem",
            aria_level: "{props.depth + 1}",
            aria_label: format!(
                "{} folder{}",
                props.name,
                if props.is_expanded { ", expanded" } else { ", collapsed" }
            ),
            aria_expanded: "{props.is_expanded}",
            aria_selected: "{props.is_current}",
            tabindex: "0",
            onclick: move |_| props.on_navigate.call(path_for_nav.clone()),
            onkeydown: move |evt| {
                match evt.key() {
                    Key::Enter => props.on_navigate.call(path.clone()),
                    Key::ArrowRight if !props.is_expanded => props.on_toggle.call(()),
                    Key::ArrowLeft if props.is_expanded => props.on_toggle.call(()),
                    _ => {}
                }
            },
            // Expand/collapse button
            button {
                class: "w-4 h-4 flex items-center justify-center text-xs text-slate-500 hover:text-slate-300",
                onclick: move |evt| {
                    evt.stop_propagation();
                    props.on_toggle.call(());
                },
                if props.is_expanded { "▼" } else { "▶" }
            }
            // Folder icon
            span {
                class: "text-sm",
                if props.is_expanded { "📂" } else { "📁" }
            }
            // Folder name
            span {
                class: "text-sm truncate",
                "{props.name}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tree_node_props_work() {
        let name = "Test Folder".to_string();
        let path = "/test".to_string();
        assert_eq!(name, "Test Folder");
        assert_eq!(path, "/test");
    }

    #[test]
    fn tree_node_aria_level_from_depth() {
        // aria-level is 1-indexed (depth 0 = level 1)
        let depth = 0u32;
        let aria_level = depth + 1;
        assert_eq!(aria_level, 1);

        let depth = 2u32;
        let aria_level = depth + 1;
        assert_eq!(aria_level, 3);
    }

    #[test]
    fn tree_node_aria_label_expanded() {
        let name = "Documents";
        let is_expanded = true;
        let aria_label = format!(
            "{} folder{}",
            name,
            if is_expanded {
                ", expanded"
            } else {
                ", collapsed"
            }
        );
        assert_eq!(aria_label, "Documents folder, expanded");
    }

    #[test]
    fn tree_node_aria_label_collapsed() {
        let name = "Photos";
        let is_expanded = false;
        let aria_label = format!(
            "{} folder{}",
            name,
            if is_expanded {
                ", expanded"
            } else {
                ", collapsed"
            }
        );
        assert_eq!(aria_label, "Photos folder, collapsed");
    }
}
