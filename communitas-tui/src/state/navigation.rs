/// Current view in the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// Authentication view (login or signup)
    Auth,
    /// Main dashboard with entity type selector
    Dashboard,
    /// Organizations view (list of channels)
    Organizations,
    /// Channel messages view
    Channel { channel_id: String },
    /// Thread replies view
    Thread {
        channel_id: String,
        thread_id: String,
    },
    /// Projects view (list of projects)
    Projects,
    /// Project issues view
    ProjectIssues { project_id: String },
    /// Issue detail view
    IssueDetail { issue_id: String },
    /// Groups view (list of groups)
    Groups,
    /// Group messages view
    GroupMessages { group_id: String },
    /// Contacts view (list of contacts)
    Contacts,
    /// Direct messages view
    DirectMessages { contact_id: String },
    /// Network status view
    NetworkStatus,
    /// Help screen
    Help,
}

/// Navigation state with view stack
#[derive(Debug)]
pub struct Navigation {
    /// Stack of views for back navigation
    pub view_stack: Vec<View>,
    /// Currently focused panel/widget
    pub focused_panel: FocusedPanel,
    /// Selection index in current list
    pub selected_index: usize,
}

/// Which panel has focus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusedPanel {
    Main,
    Sidebar,
    Input,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            view_stack: vec![View::Dashboard],
            focused_panel: FocusedPanel::Main,
            selected_index: 0,
        }
    }

    /// Get the current view
    pub fn current_view(&self) -> &View {
        self.view_stack
            .last()
            .unwrap_or(&View::Dashboard)
    }

    /// Push a new view onto the stack
    pub fn push_view(&mut self, view: View) {
        self.view_stack.push(view);
        self.selected_index = 0; // Reset selection
    }

    /// Pop the current view and return to previous
    pub fn pop_view(&mut self) -> Option<View> {
        if self.view_stack.len() > 1 {
            self.view_stack.pop()
        } else {
            None
        }
    }

    /// Go back to dashboard
    pub fn go_to_dashboard(&mut self) {
        self.view_stack.clear();
        self.view_stack.push(View::Dashboard);
        self.selected_index = 0;
    }

    /// Navigate to a specific view
    pub fn navigate_to(&mut self, view: View) {
        self.push_view(view);
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, max: usize) {
        if self.selected_index < max.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Reset selection to top
    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
    }

    /// Cycle focus to next panel
    pub fn cycle_focus(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Main => FocusedPanel::Sidebar,
            FocusedPanel::Sidebar => FocusedPanel::Input,
            FocusedPanel::Input => FocusedPanel::Main,
        };
    }
}

impl Default for Navigation {
    fn default() -> Self {
        Self::new()
    }
}
