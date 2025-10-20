//! Tree View Component - Hierarchical data display with expand/collapse
//!
//! Provides a generic tree structure for displaying hierarchical data with:
//! - Expand/collapse functionality
//! - Keyboard navigation (Up/Down/Left/Right/Enter)
//! - Selection state management
//! - Generic data payload support
//!
//! Phase 4a: Core Structure & Navigation (Advanced features like lazy loading
//! and drag-drop are deferred to later phases)

use std::collections::HashSet;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Key, KeyModifiers};
use tuirealm::props::Props;
use tuirealm::{Component, Event, MockComponent, State, StateValue};

/// A node in the tree structure with generic data payload
#[derive(Debug, Clone)]
pub struct TreeNode<T> {
    /// Unique identifier for this node
    pub id: String,
    /// Display label
    pub label: String,
    /// Generic data payload
    pub data: T,
    /// Child nodes
    pub children: Vec<TreeNode<T>>,
    /// Optional icon character (e.g., '📁' for folder, '📄' for file)
    pub icon: Option<char>,
}

impl<T> TreeNode<T> {
    /// Create a new tree node
    pub fn new(id: impl Into<String>, label: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            data,
            children: Vec::new(),
            icon: None,
        }
    }

    /// Create a new tree node with an icon
    pub fn with_icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Add a child node
    pub fn with_child(mut self, child: TreeNode<T>) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn with_children(mut self, children: Vec<TreeNode<T>>) -> Self {
        self.children.extend(children);
        self
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Find a node by ID (depth-first search)
    pub fn find_node(&self, id: &str) -> Option<&TreeNode<T>> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(node) = child.find_node(id) {
                return Some(node);
            }
        }
        None
    }

    /// Find a node by ID (mutable version)
    pub fn find_node_mut(&mut self, id: &str) -> Option<&mut TreeNode<T>> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(node) = child.find_node_mut(id) {
                return Some(node);
            }
        }
        None
    }
}

/// Tree View component for hierarchical data display
#[derive(Debug)]
pub struct TreeView<T> {
    props: Props,
    /// Root node of the tree
    root: TreeNode<T>,
    /// Set of expanded node IDs
    expanded: HashSet<String>,
    /// Currently selected node ID
    selected: Option<String>,
    /// Currently hovered node ID (for mouse interaction)
    hovered: Option<String>,
    /// Flattened list of visible nodes (for navigation)
    visible_nodes: Vec<String>,
    /// Layout information for mouse interaction: (node_id, row, depth)
    layout_map: Vec<(String, u16, usize)>,
    /// Last render area for hit detection
    last_render_area: Option<tuirealm::ratatui::layout::Rect>,
}

impl<T: Clone> TreeView<T> {
    /// Create a new TreeView with the given root node
    pub fn new(root: TreeNode<T>) -> Self {
        let root_id = root.id.clone();
        let mut tree = Self {
            props: Props::default(),
            root,
            expanded: HashSet::new(),
            selected: None,
            hovered: None,
            visible_nodes: Vec::new(),
            layout_map: Vec::new(),
            last_render_area: None,
        };

        // Initially expand the root and select it
        tree.expanded.insert(root_id.clone());
        tree.selected = Some(root_id);
        tree.rebuild_visible_nodes();

        tree
    }

    /// Get the root node
    pub fn root(&self) -> &TreeNode<T> {
        &self.root
    }

    /// Get the currently selected node ID
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Check if a node is expanded
    pub fn is_expanded(&self, node_id: &str) -> bool {
        self.expanded.contains(node_id)
    }

    /// Toggle expand/collapse state of a node
    pub fn toggle_expanded(&mut self, node_id: &str) {
        if self.expanded.contains(node_id) {
            self.collapse_node(node_id);
        } else {
            self.expand_node(node_id);
        }
    }

    /// Expand a node
    pub fn expand_node(&mut self, node_id: &str) {
        if let Some(node) = self.root.find_node(node_id)
            && node.has_children()
        {
            self.expanded.insert(node_id.to_string());
            self.rebuild_visible_nodes();
        }
    }

    /// Collapse a node
    pub fn collapse_node(&mut self, node_id: &str) {
        self.expanded.remove(node_id);
        self.rebuild_visible_nodes();
    }

    /// Expand all nodes
    pub fn expand_all(&mut self) {
        let root_clone = self.root.clone();
        Self::collect_all_node_ids_static(&root_clone, &mut self.expanded);
        self.rebuild_visible_nodes();
    }

    /// Collapse all nodes (except root)
    pub fn collapse_all(&mut self) {
        let root_id = self.root.id.clone();
        self.expanded.clear();
        self.expanded.insert(root_id);
        self.rebuild_visible_nodes();
    }

    /// Collect all node IDs into a set (static helper to avoid borrow issues)
    fn collect_all_node_ids_static(node: &TreeNode<T>, ids: &mut HashSet<String>) {
        ids.insert(node.id.clone());
        for child in &node.children {
            Self::collect_all_node_ids_static(child, ids);
        }
    }

    /// Rebuild the flattened list of visible nodes
    fn rebuild_visible_nodes(&mut self) {
        self.visible_nodes.clear();
        self.collect_visible_nodes(&self.root.clone());
    }

    /// Recursively collect visible nodes based on expand state
    fn collect_visible_nodes(&mut self, node: &TreeNode<T>) {
        self.visible_nodes.push(node.id.clone());

        if self.is_expanded(&node.id) {
            for child in &node.children {
                self.collect_visible_nodes(child);
            }
        }
    }

    /// Get the index of a node in the visible list
    fn get_visible_index(&self, node_id: &str) -> Option<usize> {
        self.visible_nodes.iter().position(|id| id == node_id)
    }

    /// Select a node by ID
    pub fn select_node(&mut self, node_id: &str) {
        if self.root.find_node(node_id).is_some() {
            self.selected = Some(node_id.to_string());
        }
    }

    /// Navigate to the previous visible node
    pub fn navigate_up(&mut self) {
        if let Some(selected_id) = &self.selected
            && let Some(current_index) = self.get_visible_index(selected_id)
            && current_index > 0
        {
            self.selected = Some(self.visible_nodes[current_index - 1].clone());
        }
    }

    /// Navigate to the next visible node
    pub fn navigate_down(&mut self) {
        if let Some(selected_id) = &self.selected
            && let Some(current_index) = self.get_visible_index(selected_id)
            && current_index + 1 < self.visible_nodes.len()
        {
            self.selected = Some(self.visible_nodes[current_index + 1].clone());
        }
    }

    /// Navigate left (collapse current node or move to parent)
    pub fn navigate_left(&mut self) {
        if let Some(selected_id) = self.selected.clone() {
            // If node is expanded, collapse it
            if self.is_expanded(&selected_id) {
                self.collapse_node(&selected_id);
            } else {
                // Move to parent node
                if let Some(parent_id) = self.find_parent_id(&selected_id) {
                    self.selected = Some(parent_id);
                }
            }
        }
    }

    /// Navigate right (expand current node or move to first child)
    pub fn navigate_right(&mut self) {
        if let Some(selected_id) = self.selected.clone()
            && let Some(node) = self.root.find_node(&selected_id)
            && node.has_children()
        {
            if self.is_expanded(&selected_id) {
                // Already expanded, move to first child
                if let Some(first_child) = node.children.first() {
                    self.selected = Some(first_child.id.clone());
                }
            } else {
                // Expand the node
                self.expand_node(&selected_id);
            }
        }
    }

    /// Find the parent ID of a given node
    fn find_parent_id(&self, child_id: &str) -> Option<String> {
        Self::find_parent_id_recursive(&self.root, child_id)
    }

    /// Recursive helper to find parent ID
    fn find_parent_id_recursive(node: &TreeNode<T>, child_id: &str) -> Option<String> {
        for child in &node.children {
            if child.id == child_id {
                return Some(node.id.clone());
            }
            if let Some(parent_id) = Self::find_parent_id_recursive(child, child_id) {
                return Some(parent_id);
            }
        }
        None
    }

    /// Handle Enter key (toggle expand/collapse)
    pub fn handle_enter(&mut self) {
        if let Some(selected_id) = &self.selected.clone() {
            self.toggle_expanded(selected_id);
        }
    }

    /// Get the number of visible nodes
    pub fn visible_count(&self) -> usize {
        self.visible_nodes.len()
    }

    // ===== PHASE 4b: MOUSE INTERACTION =====

    /// Build layout map for mouse interaction (called during rendering)
    pub fn build_layout_map(&mut self, area: tuirealm::ratatui::layout::Rect) {
        self.layout_map.clear();
        self.last_render_area = Some(area);

        let mut row = area.y;
        for node_id in &self.visible_nodes {
            let depth = self.get_node_depth(node_id);
            self.layout_map.push((node_id.clone(), row, depth));
            row += 1;

            // Stop if we run out of vertical space
            if row >= area.y + area.height {
                break;
            }
        }
    }

    /// Get the depth of a node in the tree (0 = root)
    fn get_node_depth(&self, node_id: &str) -> usize {
        let mut depth = 0;
        let mut current_id = node_id.to_string();

        while let Some(parent_id) = self.find_parent_id(&current_id) {
            depth += 1;
            current_id = parent_id;
        }

        depth
    }

    /// Handle mouse click at the given coordinates
    /// Returns true if the click was handled (state changed)
    pub fn handle_mouse_click(&mut self, x: u16, y: u16) -> bool {
        if let Some(area) = self.last_render_area {
            // Check if click is within the tree area
            if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
                return false;
            }

            // Find which node was clicked based on row
            if let Some((node_id, _, depth)) = self
                .layout_map
                .iter()
                .find(|(_, row, _)| *row == y)
                .cloned()
            {
                let indent = (depth * 2) as u16; // 2 spaces per depth level
                let icon_area_start = area.x + indent;
                let icon_area_end = icon_area_start + 2; // Icon takes 2 chars ("▶ " or "▼ ")

                // Check if this node has children (can be expanded)
                if let Some(node) = self.root.find_node(&node_id)
                    && node.has_children()
                    && x >= icon_area_start
                    && x < icon_area_end
                {
                    // Click on expand/collapse icon
                    self.toggle_expanded(&node_id);
                    return true;
                }

                // Click anywhere on the row (including icon area for leaf nodes)
                if x >= icon_area_start {
                    self.select_node(&node_id);
                    return true;
                }
            }
        }

        false
    }

    /// Handle mouse movement for hover effects
    /// Returns true if hover state changed
    pub fn handle_mouse_move(&mut self, x: u16, y: u16) -> bool {
        if let Some(area) = self.last_render_area {
            // Check if mouse is within the tree area
            if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
                let had_hover = self.hovered.is_some();
                self.hovered = None;
                return had_hover;
            }

            // Find which node is being hovered
            let new_hover = self
                .layout_map
                .iter()
                .find(|(_, row, _)| *row == y)
                .map(|(node_id, _, _)| node_id.clone());

            let changed = self.hovered != new_hover;
            self.hovered = new_hover;
            changed
        } else {
            false
        }
    }

    /// Get the currently hovered node ID
    pub fn hovered(&self) -> Option<&str> {
        self.hovered.as_deref()
    }

    /// Check if the mouse is over an expand/collapse icon
    pub fn is_over_expand_icon(&self, x: u16, y: u16) -> bool {
        if let Some(area) = self.last_render_area
            && let Some((node_id, _, depth)) = self.layout_map.iter().find(|(_, row, _)| *row == y)
        {
            let indent = (*depth * 2) as u16;
            let icon_area_start = area.x + indent;
            let icon_area_end = icon_area_start + 2;

            // Check if node has children and cursor is over icon
            if let Some(node) = self.root.find_node(node_id) {
                return node.has_children() && x >= icon_area_start && x < icon_area_end;
            }
        }
        false
    }
}

impl<T: Clone + 'static> MockComponent for TreeView<T> {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::ratatui::layout::Rect) {
        use tuirealm::ratatui::style::{Color, Modifier, Style};
        use tuirealm::ratatui::text::{Line, Span};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        // Build layout map for mouse interaction
        self.build_layout_map(area);

        // Create styled lines for each visible node
        let mut lines: Vec<Line> = Vec::new();

        for node_id in &self.visible_nodes.clone() {
            if let Some(node) = self.root.find_node(node_id) {
                let depth = self.get_node_depth(node_id);
                let is_selected = self.selected.as_ref() == Some(node_id);
                let is_hovered = self.hovered.as_ref() == Some(node_id);

                // Build line content
                let mut spans: Vec<Span> = Vec::new();

                // Indentation
                let indent = "  ".repeat(depth);
                spans.push(Span::raw(indent));

                // Expand/collapse icon
                let expand_icon = if node.has_children() {
                    if self.is_expanded(node_id) {
                        "▼ "
                    } else {
                        "▶ "
                    }
                } else {
                    "  "
                };
                spans.push(Span::raw(expand_icon));

                // Optional node icon
                if let Some(icon) = node.icon {
                    spans.push(Span::raw(format!("{} ", icon)));
                }

                // Node label with styling
                let mut style = Style::default();

                if is_selected {
                    style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                } else if is_hovered {
                    style = style.fg(Color::Cyan).add_modifier(Modifier::UNDERLINED);
                }

                spans.push(Span::styled(node.label.clone(), style));

                lines.push(Line::from(spans));

                // Stop if we exceed the area height
                if lines.len() >= area.height as usize {
                    break;
                }
            }
        }

        // Render the tree
        let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));

        frame.render_widget(paragraph, area);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        if let Some(selected_id) = &self.selected {
            State::One(StateValue::String(selected_id.clone()))
        } else {
            State::None
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(tuirealm::command::Direction::Up) => {
                self.navigate_up();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(tuirealm::command::Direction::Down) => {
                self.navigate_down();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(tuirealm::command::Direction::Left) => {
                self.navigate_left();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(tuirealm::command::Direction::Right) => {
                self.navigate_right();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => {
                self.handle_enter();
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

impl<T: Clone + 'static> Component<tuirealm::NoUserEvent, tuirealm::NoUserEvent> for TreeView<T> {
    fn on(&mut self, ev: Event<tuirealm::NoUserEvent>) -> Option<tuirealm::NoUserEvent> {
        match ev {
            Event::Keyboard(ke) if ke.code == Key::Up && ke.modifiers == KeyModifiers::NONE => {
                self.navigate_up();
                None
            }
            Event::Keyboard(ke) if ke.code == Key::Down && ke.modifiers == KeyModifiers::NONE => {
                self.navigate_down();
                None
            }
            Event::Keyboard(ke) if ke.code == Key::Left && ke.modifiers == KeyModifiers::NONE => {
                self.navigate_left();
                None
            }
            Event::Keyboard(ke) if ke.code == Key::Right && ke.modifiers == KeyModifiers::NONE => {
                self.navigate_right();
                None
            }
            Event::Keyboard(ke) if ke.code == Key::Enter && ke.modifiers == KeyModifiers::NONE => {
                self.handle_enter();
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::StateValue;
    use tuirealm::event::{Key, KeyEvent, KeyModifiers};

    // Test helper: Create a simple test tree
    fn create_test_tree() -> TreeNode<String> {
        TreeNode::new("root", "Root", "root_data".to_string())
            .with_icon('📁')
            .with_children(vec![
                TreeNode::new("child1", "Child 1", "child1_data".to_string())
                    .with_icon('📄')
                    .with_children(vec![
                        TreeNode::new("grandchild1", "Grandchild 1", "gc1_data".to_string()),
                        TreeNode::new("grandchild2", "Grandchild 2", "gc2_data".to_string()),
                    ]),
                TreeNode::new("child2", "Child 2", "child2_data".to_string()).with_icon('📄'),
                TreeNode::new("child3", "Child 3", "child3_data".to_string()).with_children(vec![
                    TreeNode::new("grandchild3", "Grandchild 3", "gc3_data".to_string()),
                ]),
            ])
    }

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new("test", "Test Node", 42);
        assert_eq!(node.id, "test");
        assert_eq!(node.label, "Test Node");
        assert_eq!(node.data, 42);
        assert!(node.children.is_empty());
        assert_eq!(node.icon, None);
    }

    #[test]
    fn test_tree_node_with_icon() {
        let node = TreeNode::new("test", "Test", 42).with_icon('📁');
        assert_eq!(node.icon, Some('📁'));
    }

    #[test]
    fn test_tree_node_with_child() {
        let child = TreeNode::new("child", "Child", 1);
        let parent = TreeNode::new("parent", "Parent", 2).with_child(child);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].id, "child");
    }

    #[test]
    fn test_tree_node_with_children() {
        let children = vec![
            TreeNode::new("child1", "Child 1", 1),
            TreeNode::new("child2", "Child 2", 2),
        ];
        let parent = TreeNode::new("parent", "Parent", 0).with_children(children);
        assert_eq!(parent.children.len(), 2);
    }

    #[test]
    fn test_tree_node_has_children() {
        let leaf = TreeNode::new("leaf", "Leaf", 1);
        assert!(!leaf.has_children());

        let parent =
            TreeNode::new("parent", "Parent", 2).with_child(TreeNode::new("child", "Child", 3));
        assert!(parent.has_children());
    }

    #[test]
    fn test_tree_node_find_node() {
        let tree = create_test_tree();

        assert!(tree.find_node("root").is_some());
        assert!(tree.find_node("child1").is_some());
        assert!(tree.find_node("grandchild2").is_some());
        assert!(tree.find_node("nonexistent").is_none());

        let found = tree.find_node("grandchild1").unwrap();
        assert_eq!(found.label, "Grandchild 1");
    }

    #[test]
    fn test_tree_view_creation() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // Root should be selected by default
        assert_eq!(view.selected(), Some("root"));

        // Root should be expanded by default
        assert!(view.is_expanded("root"));

        // Should have at least root visible
        assert!(view.visible_count() >= 1);
    }

    #[test]
    fn test_expand_collapse() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Expand child1
        view.expand_node("child1");
        assert!(view.is_expanded("child1"));

        // Collapse child1
        view.collapse_node("child1");
        assert!(!view.is_expanded("child1"));
    }

    #[test]
    fn test_toggle_expanded() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let initial_state = view.is_expanded("child1");
        view.toggle_expanded("child1");
        assert_eq!(view.is_expanded("child1"), !initial_state);

        view.toggle_expanded("child1");
        assert_eq!(view.is_expanded("child1"), initial_state);
    }

    #[test]
    fn test_expand_all() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_all();

        assert!(view.is_expanded("root"));
        assert!(view.is_expanded("child1"));
        assert!(view.is_expanded("child3"));
    }

    #[test]
    fn test_collapse_all() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_all();
        view.collapse_all();

        // Root should still be expanded
        assert!(view.is_expanded("root"));

        // All others should be collapsed
        assert!(!view.is_expanded("child1"));
        assert!(!view.is_expanded("child3"));
    }

    #[test]
    fn test_select_node() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child2");
        assert_eq!(view.selected(), Some("child2"));

        // Selecting invalid node should not change selection
        view.select_node("nonexistent");
        assert_eq!(view.selected(), Some("child2"));
    }

    #[test]
    fn test_navigate_down() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Start at root
        assert_eq!(view.selected(), Some("root"));

        // Navigate down should go to first child
        view.navigate_down();
        assert_eq!(view.selected(), Some("child1"));

        view.navigate_down();
        assert_eq!(view.selected(), Some("child2"));
    }

    #[test]
    fn test_navigate_up() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child2");
        view.navigate_up();
        assert_eq!(view.selected(), Some("child1"));

        view.navigate_up();
        assert_eq!(view.selected(), Some("root"));

        // At root, should not go further up
        view.navigate_up();
        assert_eq!(view.selected(), Some("root"));
    }

    #[test]
    fn test_navigate_right_expands() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");
        assert!(!view.is_expanded("child1"));

        // Navigate right should expand child1
        view.navigate_right();
        assert!(view.is_expanded("child1"));
    }

    #[test]
    fn test_navigate_right_moves_to_child() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");
        view.expand_node("child1");

        // Navigate right on already expanded node moves to first child
        view.navigate_right();
        assert_eq!(view.selected(), Some("grandchild1"));
    }

    #[test]
    fn test_navigate_left_collapses() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");
        view.expand_node("child1");

        // Navigate left should collapse
        view.navigate_left();
        assert!(!view.is_expanded("child1"));
    }

    #[test]
    fn test_navigate_left_moves_to_parent() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_node("child1");
        view.select_node("grandchild1");

        // Navigate left on collapsed node moves to parent
        view.navigate_left();
        assert_eq!(view.selected(), Some("child1"));
    }

    #[test]
    fn test_handle_enter_toggles_expand() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");
        let initial_state = view.is_expanded("child1");

        view.handle_enter();
        assert_eq!(view.is_expanded("child1"), !initial_state);

        view.handle_enter();
        assert_eq!(view.is_expanded("child1"), initial_state);
    }

    #[test]
    fn test_visible_nodes_updates_on_expand() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let initial_count = view.visible_count();

        view.expand_node("child1");
        let expanded_count = view.visible_count();

        // Should have more visible nodes after expanding
        assert!(expanded_count > initial_count);
    }

    #[test]
    fn test_visible_nodes_updates_on_collapse() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_node("child1");
        let expanded_count = view.visible_count();

        view.collapse_node("child1");
        let collapsed_count = view.visible_count();

        // Should have fewer visible nodes after collapsing
        assert!(collapsed_count < expanded_count);
    }

    #[test]
    fn test_mock_component_state() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        match view.state() {
            State::One(StateValue::String(id)) => {
                assert_eq!(id, "root");
            }
            _ => panic!("Expected State::One with selected node ID"),
        }
    }

    #[test]
    fn test_mock_component_perform_up() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child2");
        view.perform(Cmd::Move(tuirealm::command::Direction::Up));

        assert_eq!(view.selected(), Some("child1"));
    }

    #[test]
    fn test_mock_component_perform_down() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.perform(Cmd::Move(tuirealm::command::Direction::Down));
        assert_eq!(view.selected(), Some("child1"));
    }

    #[test]
    fn test_mock_component_perform_submit() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");
        let initial_state = view.is_expanded("child1");

        view.perform(Cmd::Submit);
        assert_eq!(view.is_expanded("child1"), !initial_state);
    }

    #[test]
    fn test_component_keyboard_navigation() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Test Down arrow
        view.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(view.selected(), Some("child1"));

        // Test Up arrow
        view.on(Event::Keyboard(KeyEvent::new(Key::Up, KeyModifiers::NONE)));
        assert_eq!(view.selected(), Some("root"));
    }

    #[test]
    fn test_component_keyboard_enter() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");
        let initial_state = view.is_expanded("child1");

        view.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));

        assert_eq!(view.is_expanded("child1"), !initial_state);
    }

    // ===== PHASE 4b: MOUSE INTERACTION TESTS =====

    use tuirealm::ratatui::layout::Rect;

    #[test]
    fn test_build_layout_map() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Should have layout entries for visible nodes
        assert!(!view.layout_map.is_empty());
        assert_eq!(view.last_render_area, Some(area));

        // Root should be at row 0, depth 0
        let root_entry = view.layout_map.iter().find(|(id, _, _)| id == "root");
        assert!(root_entry.is_some());
        let (_, row, depth) = root_entry.unwrap();
        assert_eq!(*row, 0);
        assert_eq!(*depth, 0);
    }

    #[test]
    fn test_get_node_depth() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        assert_eq!(view.get_node_depth("root"), 0);
        assert_eq!(view.get_node_depth("child1"), 1);
        assert_eq!(view.get_node_depth("grandchild1"), 2);
    }

    #[test]
    fn test_mouse_click_to_select() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Build layout map
        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Click on child1 (row 1, past the icon area)
        let handled = view.handle_mouse_click(5, 1);
        assert!(handled);
        assert_eq!(view.selected(), Some("child1"));
    }

    #[test]
    fn test_mouse_click_to_expand() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Build layout map
        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // child1 starts collapsed
        assert!(!view.is_expanded("child1"));

        // Click on the expand icon for child1 (row 1, x=2-3, depth 1 means indent of 2)
        let handled = view.handle_mouse_click(2, 1);
        assert!(handled);
        assert!(view.is_expanded("child1"));
    }

    #[test]
    fn test_mouse_click_to_collapse() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Expand child1 first
        view.expand_node("child1");

        // Build layout map
        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        assert!(view.is_expanded("child1"));

        // Click on the collapse icon
        let handled = view.handle_mouse_click(2, 1);
        assert!(handled);
        assert!(!view.is_expanded("child1"));
    }

    #[test]
    fn test_mouse_click_outside_area() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Click outside the area should not be handled
        assert!(!view.handle_mouse_click(25, 5)); // Beyond width
        assert!(!view.handle_mouse_click(5, 15)); // Beyond height
    }

    #[test]
    fn test_mouse_click_on_leaf_node() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // child2 has no children, so clicking icon area should just select
        let handled = view.handle_mouse_click(2, 2); // child2 is at row 2
        assert!(handled);
        assert_eq!(view.selected(), Some("child2"));
        // Should not affect expand state (leaf node)
        assert!(!view.is_expanded("child2"));
    }

    #[test]
    fn test_mouse_move_hover() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Initially no hover
        assert_eq!(view.hovered(), None);

        // Move mouse over child1 (row 1)
        let changed = view.handle_mouse_move(5, 1);
        assert!(changed);
        assert_eq!(view.hovered(), Some("child1"));
    }

    #[test]
    fn test_mouse_move_hover_change() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Hover over child1
        view.handle_mouse_move(5, 1);
        assert_eq!(view.hovered(), Some("child1"));

        // Move to child2
        let changed = view.handle_mouse_move(5, 2);
        assert!(changed);
        assert_eq!(view.hovered(), Some("child2"));
    }

    #[test]
    fn test_mouse_move_outside_clears_hover() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Hover over child1
        view.handle_mouse_move(5, 1);
        assert_eq!(view.hovered(), Some("child1"));

        // Move outside
        let changed = view.handle_mouse_move(25, 5);
        assert!(changed);
        assert_eq!(view.hovered(), None);
    }

    #[test]
    fn test_is_over_expand_icon() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // child1 is at row 1, depth 1 (indent = 2), icon area is x=2-3
        assert!(view.is_over_expand_icon(2, 1));
        assert!(view.is_over_expand_icon(3, 1));
        assert!(!view.is_over_expand_icon(4, 1)); // Past icon area
        assert!(!view.is_over_expand_icon(1, 1)); // Before icon area
    }

    #[test]
    fn test_is_over_expand_icon_leaf_node() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // child2 is a leaf (no children), so icon area should return false
        assert!(!view.is_over_expand_icon(2, 2)); // child2 at row 2
    }

    #[test]
    fn test_mouse_click_at_different_depths() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Expand child1 to show grandchildren
        view.expand_node("child1");

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        // Find grandchild1 in layout
        let grandchild_entry = view
            .layout_map
            .iter()
            .find(|(id, _, _)| id == "grandchild1")
            .cloned();
        assert!(grandchild_entry.is_some());

        let (_, row, depth) = grandchild_entry.unwrap();
        assert_eq!(depth, 2);

        // Click on grandchild (depth 2 means indent of 4)
        let handled = view.handle_mouse_click(6, row);
        assert!(handled);
        assert_eq!(view.selected(), Some("grandchild1"));
    }

    #[test]
    fn test_layout_map_respects_area_height() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_all();

        // Small area that can't fit all nodes
        let area = Rect::new(0, 0, 20, 3);
        view.build_layout_map(area);

        // Should only have entries for rows that fit
        assert!(view.layout_map.len() <= 3);
    }

    // ===== PHASE 4c: VISUAL RENDERING TESTS =====

    #[test]
    fn test_rendering_builds_layout_map() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        // Create a mock frame and area
        use tuirealm::ratatui::Terminal;
        use tuirealm::ratatui::backend::TestBackend;

        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 20);
                view.view(frame, area);
            })
            .unwrap();

        // Layout map should be built
        assert!(!view.layout_map.is_empty());
        assert!(view.last_render_area.is_some());
    }

    #[test]
    fn test_expand_icon_for_collapsed_node() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // child1 has children and is collapsed
        if let Some(node) = view.root.find_node("child1") {
            assert!(node.has_children());
            assert!(!view.is_expanded("child1"));
            // In rendering, this would show "▶ "
        }
    }

    #[test]
    fn test_expand_icon_for_expanded_node() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_node("child1");

        // child1 is now expanded
        if let Some(node) = view.root.find_node("child1") {
            assert!(node.has_children());
            assert!(view.is_expanded("child1"));
            // In rendering, this would show "▼ "
        }
    }

    #[test]
    fn test_no_expand_icon_for_leaf_node() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // child2 has no children
        if let Some(node) = view.root.find_node("child2") {
            assert!(!node.has_children());
            // In rendering, this would show "  " (spaces)
        }
    }

    #[test]
    fn test_node_with_custom_icon() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // root has a custom icon
        assert_eq!(view.root.icon, Some('📁'));
    }

    #[test]
    fn test_node_without_custom_icon() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // grandchild1 has no custom icon
        if let Some(node) = view.root.find_node("grandchild1") {
            assert_eq!(node.icon, None);
        }
    }

    #[test]
    fn test_selected_node_styling() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.select_node("child1");

        // child1 should be marked as selected for rendering
        assert_eq!(view.selected(), Some("child1"));
    }

    #[test]
    fn test_hovered_node_styling() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        let area = Rect::new(0, 0, 20, 10);
        view.build_layout_map(area);

        view.handle_mouse_move(5, 1); // Hover over child1

        // child1 should be marked as hovered for rendering
        assert_eq!(view.hovered(), Some("child1"));
    }

    #[test]
    fn test_visible_nodes_for_rendering() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // Only root and its direct children should be visible (root is expanded by default)
        assert!(view.visible_count() >= 1); // At least root
        assert!(view.visible_count() <= 10); // Not all grandchildren (they're collapsed)
    }

    #[test]
    fn test_depth_indentation_calculation() {
        let tree = create_test_tree();
        let view = TreeView::new(tree);

        // Root at depth 0
        assert_eq!(view.get_node_depth("root"), 0);

        // Children at depth 1
        assert_eq!(view.get_node_depth("child1"), 1);
        assert_eq!(view.get_node_depth("child2"), 1);

        // Grandchildren at depth 2
        assert_eq!(view.get_node_depth("grandchild1"), 2);
        assert_eq!(view.get_node_depth("grandchild2"), 2);
    }

    #[test]
    fn test_rendering_with_expanded_tree() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_all();

        use tuirealm::ratatui::Terminal;
        use tuirealm::ratatui::backend::TestBackend;

        let backend = TestBackend::new(40, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 20);
                view.view(frame, area);
            })
            .unwrap();

        // All nodes should be in layout map
        assert!(view.layout_map.len() >= 7); // root + 3 children + 3 grandchildren
    }

    #[test]
    fn test_rendering_respects_area_constraints() {
        let tree = create_test_tree();
        let mut view = TreeView::new(tree);

        view.expand_all();

        use tuirealm::ratatui::Terminal;
        use tuirealm::ratatui::backend::TestBackend;

        let backend = TestBackend::new(40, 3); // Very small height
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 40, 3);
                view.view(frame, area);
            })
            .unwrap();

        // Should only render nodes that fit
        assert!(view.layout_map.len() <= 3);
    }
}
