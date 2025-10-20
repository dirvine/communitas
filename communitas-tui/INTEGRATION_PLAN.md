# Communitas TUI - Component Integration & Polish Plan

**Date**: 2025-10-19
**Status**: Planning Phase
**Target**: Integrate 5 new advanced components + visual polish + plugin documentation

---

## 🎯 Overview

This plan covers the integration of the newly implemented components (Phases 3-7) into the main Communitas TUI application, adding visual polish with animations, and creating comprehensive plugin documentation.

### Components to Integrate:
1. **ResizableSplit** - Draggable panel dividers
2. **TreeView** - Hierarchical data display
3. **CommandPalette** - Fuzzy command search (Ctrl+K)
4. **Animation System** - Smooth transitions
5. **Plugin System** - Extensible architecture

---

## 📋 Phase 1: Component Integration (Estimated: 8-10 hours)

### 1.1: AppState Integration

**Goal**: Add new components to AppState and initialize them properly.

**Tasks**:
- [ ] Add fields to `AppState` struct in `src/state/app_state.rs`:
  ```rust
  // New component fields
  pub resizable_split: ResizableSplit,
  pub tree_view: TreeView<EntityTreeNode>,
  pub command_palette: CommandPalette,
  pub animation_manager: AnimationManager,
  pub plugin_manager: PluginManager,
  ```

- [ ] Initialize in `AppState::new()`:
  ```rust
  resizable_split: ResizableSplit::new()
      .with_orientation(Orientation::Vertical)
      .with_position(30)  // 30% left panel
      .with_min_size(20)
      .with_max_size(50),
  tree_view: TreeView::new(),
  command_palette: CommandPalette::new(),
  animation_manager: AnimationManager::new(),
  plugin_manager: PluginManager::new(),
  ```

**Acceptance Criteria**:
- ✅ All components added to AppState
- ✅ Proper initialization with sensible defaults
- ✅ Zero compilation errors
- ✅ Components accessible from state

**Estimated Time**: 1 hour

---

### 1.2: ResizableSplit Integration (Dashboard Layout)

**Goal**: Replace fixed dashboard layout with resizable split panels.

**Current Layout**: Fixed 30/70 split between sidebar and content
**Target Layout**: User-adjustable split with min/max constraints

**Implementation**:

**File**: `src/ui/dashboard.rs`

**Changes**:
1. Replace fixed `Constraint::Percentage(30)` with dynamic split from state
2. Add mouse event handler for divider dragging
3. Add double-click to reset split position
4. Persist split position to config (optional)

**Code Pattern**:
```rust
// In render function
let split_pos = state.resizable_split.get_position();
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(split_pos),
        Constraint::Percentage(100 - split_pos),
    ])
    .split(area);

// Render divider indicator
if state.resizable_split.is_hovered() {
    // Highlight divider
}
```

**Mouse Event Handler** (in `src/app.rs`):
```rust
async fn handle_mouse_event(&mut self, mouse: MouseEvent) {
    // Check if mouse is over divider
    if self.state.resizable_split.is_over_divider(mouse.column, mouse.row, area) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.state.resizable_split.start_drag(mouse.column, mouse.row);
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.state.resizable_split.handle_drag(mouse.column, mouse.row, area);
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.state.resizable_split.end_drag();
            }
            MouseEventKind::Moved => {
                self.state.resizable_split.update_hover(mouse.column, mouse.row, area);
            }
            _ => {}
        }
    }
}
```

**Keyboard Shortcuts**:
- `Ctrl+\` - Reset split to default position

**Acceptance Criteria**:
- ✅ Dashboard uses resizable split
- ✅ Drag divider with mouse to resize
- ✅ Double-click divider to reset
- ✅ Visual feedback on hover
- ✅ Min/max constraints enforced
- ✅ Keyboard shortcut to reset

**Estimated Time**: 2-3 hours

---

### 1.3: TreeView Integration (Organization Hierarchy)

**Goal**: Display organization hierarchy with expandable tree view.

**Use Cases**:
1. **Organization View**: Org → Projects → Issues
2. **File Browser**: Virtual disk navigation
3. **Channel Categories**: Channels grouped by category

**Implementation**:

**File**: `src/ui/organizations.rs`

**Data Structure**:
```rust
pub enum EntityTreeNode {
    Organization {
        id: String,
        name: String,
        projects: Vec<String>,
    },
    Project {
        id: String,
        name: String,
        issues: Vec<String>,
    },
    Issue {
        id: String,
        title: String,
    },
}

impl std::fmt::Display for EntityTreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EntityTreeNode::Organization { name, .. } => write!(f, "🏢 {}", name),
            EntityTreeNode::Project { name, .. } => write!(f, "📋 {}", name),
            EntityTreeNode::Issue { title, .. } => write!(f, "📌 {}", title),
        }
    }
}
```

**Integration Steps**:
1. Build tree from entity data
2. Render tree view in organizations tab
3. Handle keyboard navigation (Up/Down/Left/Right/Enter)
4. Handle mouse clicks (select, expand, collapse)
5. Add icons for different node types

**Keyboard Shortcuts**:
- `Up/Down` or `k/j` - Navigate nodes
- `Left/Right` or `h/l` - Collapse/Expand
- `Enter` - Open selected item
- `Space` - Toggle expand/collapse

**Acceptance Criteria**:
- ✅ TreeView displays in organizations tab
- ✅ Keyboard navigation works
- ✅ Mouse click to expand/collapse
- ✅ Icons for different node types
- ✅ Selection highlighting
- ✅ Hover effects

**Estimated Time**: 3-4 hours

---

### 1.4: CommandPalette Integration (Global Command Search)

**Goal**: Add Ctrl+K command palette for quick access to all features.

**Commands to Include**:
- Navigation: "Go to Organizations", "Go to Projects", etc.
- Actions: "Create Organization", "Create Project", "Send Message"
- Settings: "Toggle Theme", "Toggle Performance Monitor", "Open Help"
- Network: "Connect", "Disconnect", "View Network Status"
- Accessibility: "Toggle Screen Reader", "Toggle High Contrast"

**Implementation**:

**File**: `src/app.rs` (event handler) + `src/state/commands.rs` (command definitions)

**Command Definitions**:
```rust
// src/state/commands.rs
pub fn build_command_list() -> Vec<Command> {
    vec![
        // Navigation
        Command::new("nav.orgs", "Go to Organizations", "Navigate to Organizations view", "Navigation")
            .with_shortcut("Alt+O"),
        Command::new("nav.projects", "Go to Projects", "Navigate to Projects view", "Navigation")
            .with_shortcut("Alt+P"),
        Command::new("nav.groups", "Go to Groups", "Navigate to Groups view", "Navigation")
            .with_shortcut("Alt+G"),

        // Actions
        Command::new("create.org", "Create Organization", "Create a new organization", "Actions")
            .with_shortcut("Ctrl+N O"),
        Command::new("create.project", "Create Project", "Create a new project", "Actions")
            .with_shortcut("Ctrl+N P"),

        // Settings
        Command::new("toggle.theme", "Toggle Theme", "Switch between light/dark theme", "Settings")
            .with_shortcut("Ctrl+T"),
        Command::new("toggle.perf", "Toggle Performance Monitor", "Show/hide performance overlay", "Settings")
            .with_shortcut("F12"),

        // Network
        Command::new("net.connect", "Connect to Network", "Connect to P2P network", "Network"),
        Command::new("net.disconnect", "Disconnect", "Disconnect from network", "Network"),

        // Accessibility
        Command::new("a11y.screen_reader", "Toggle Screen Reader", "Enable/disable screen reader", "Accessibility")
            .with_shortcut("Ctrl+S"),
        Command::new("a11y.high_contrast", "Toggle High Contrast", "Enable/disable high contrast mode", "Accessibility")
            .with_shortcut("Ctrl+H"),
    ]
}
```

**Event Handler**:
```rust
// In src/app.rs
async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
    // Ctrl+K to toggle command palette
    if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL) {
        self.state.command_palette.toggle();
        return Ok(());
    }

    // If command palette is visible, route events to it
    if self.state.command_palette.is_visible() {
        match key.code {
            KeyCode::Char(c) => self.state.command_palette.push_query(c),
            KeyCode::Backspace => self.state.command_palette.pop_query(),
            KeyCode::Up => self.state.command_palette.select_previous(),
            KeyCode::Down => self.state.command_palette.select_next(),
            KeyCode::Enter => {
                if let Some(command_id) = self.state.command_palette.execute_selected() {
                    self.execute_command(&command_id).await?;
                }
            }
            KeyCode::Esc => self.state.command_palette.hide(),
            _ => {}
        }
        return Ok(());
    }

    // Normal key handling...
}

async fn execute_command(&mut self, command_id: &str) -> Result<()> {
    match command_id {
        "nav.orgs" => self.state.navigation.view_stack.push(View::Dashboard),
        "nav.projects" => { /* ... */ }
        "create.org" => { /* ... */ }
        "toggle.theme" => self.state.theme_manager.cycle_theme(),
        "toggle.perf" => self.state.performance_monitor.toggle(),
        // ... etc
        _ => {}
    }
    Ok(())
}
```

**Rendering** (in `src/ui/mod.rs`):
```rust
pub fn render(f: &mut Frame, state: &mut AppState) {
    // Render main UI...

    // Render command palette overlay (if visible)
    if state.command_palette.is_visible() {
        let area = centered_rect(80, 50, f.size());
        render_command_palette(f, area, &mut state.command_palette);
    }
}
```

**Acceptance Criteria**:
- ✅ Ctrl+K opens command palette
- ✅ Fuzzy search works
- ✅ Up/Down navigate results
- ✅ Enter executes command
- ✅ Esc closes palette
- ✅ Recent commands prioritized
- ✅ Category badges displayed
- ✅ Keyboard shortcuts shown

**Estimated Time**: 2-3 hours

---

### 1.5: Animation System Integration (Visual Polish)

**Goal**: Add smooth animations for modal transitions, notifications, and UI feedback.

**Animation Manager**:
Create wrapper to manage multiple concurrent animations:

```rust
// src/state/animation_manager.rs
pub struct AnimationManager {
    active_animations: HashMap<String, Animation>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            active_animations: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: impl Into<String>, animation: Animation) {
        self.active_animations.insert(id.into(), animation);
    }

    pub fn update_all(&mut self) {
        self.active_animations.retain(|_, anim| {
            anim.update();
            !anim.is_completed()
        });
    }

    pub fn get(&self, id: &str) -> Option<&Animation> {
        self.active_animations.get(id)
    }
}
```

**Use Cases**:

1. **Modal Fade In** (Command Palette, Context Menu):
```rust
// When showing command palette
let fade = Animation::fade_in(Duration::from_millis(200));
state.animation_manager.add("command_palette_fade", fade);

// In render
if let Some(anim) = state.animation_manager.get("command_palette_fade") {
    if let AnimationValue::Opacity(opacity) = anim.current_value() {
        // Apply opacity to palette rendering
    }
}
```

2. **Notification Pulse** (Status messages):
```rust
// When showing notification
let pulse = Animation::pulse(180, 255, Duration::from_millis(1000));
state.animation_manager.add("notification_pulse", pulse);
```

3. **Error Shake** (Invalid input):
```rust
// On error
let shake = Animation::shake(5, Duration::from_millis(300));
state.animation_manager.add("input_shake", shake);
```

4. **Panel Slide** (Sidebar open/close):
```rust
// Opening sidebar
let slide = Animation::slide(-100, 0, Axis::Horizontal, Duration::from_millis(250));
state.animation_manager.add("sidebar_slide", slide);
```

**Integration Points**:
- Command Palette show/hide
- Context menu show/hide
- Status message notifications
- Error displays
- Modal dialogs
- Panel transitions

**Acceptance Criteria**:
- ✅ Animations managed centrally
- ✅ Multiple concurrent animations
- ✅ Smooth 60fps rendering
- ✅ No performance degradation
- ✅ Proper cleanup of completed animations

**Estimated Time**: 2 hours

---

## 📋 Phase 2: Visual Polish & Animations (Estimated: 6-8 hours)

### 2.1: Modal & Overlay Animations

**Components to Animate**:
- Command Palette
- Context Menus
- Help Overlay
- Settings Dialog (future)

**Animation Patterns**:
```rust
// Fade in on show
let fade_in = Animation::fade_in(Duration::from_millis(150));

// Slide up from bottom
let slide_up = Animation::slide(50, 0, Axis::Vertical, Duration::from_millis(200));

// Combined: fade + slide for professional feel
```

**Implementation**:
1. Add animation field to modal components
2. Trigger on show/hide events
3. Apply opacity and offset during render
4. Use EaseOut for natural motion

**Estimated Time**: 2 hours

---

### 2.2: Notification & Status Animations

**Notifications**:
- Slide in from top when appearing
- Pulse to grab attention
- Fade out when dismissing

**Implementation**:
```rust
// Status message appears
let notification_anim = Animation::slide(-30, 0, Axis::Vertical, Duration::from_millis(300))
    .with_easing(EasingFunction::EaseOut);

// Important messages pulse
if is_important {
    let pulse = Animation::pulse(200, 255, Duration::from_millis(800));
    state.animation_manager.add("status_pulse", pulse);
}
```

**Estimated Time**: 1-2 hours

---

### 2.3: Error Feedback Animations

**Error Patterns**:
- Shake on invalid input
- Red flash on error
- Gentle bounce on validation failure

**Implementation**:
```rust
// Input validation failed
let shake = Animation::shake(8, Duration::from_millis(400));
state.animation_manager.add("input_shake", shake);

// Apply in render
if let Some(shake_anim) = state.animation_manager.get("input_shake") {
    if let AnimationValue::Offset(offset, _) = shake_anim.current_value() {
        // Offset input field by shake amount
        let adjusted_area = Rect {
            x: area.x.saturating_add_signed(offset),
            ..area
        };
        render_input(f, adjusted_area, state);
    }
}
```

**Estimated Time**: 1-2 hours

---

### 2.4: Loading & Progress Animations

**Loading Indicators**:
- Spinner for network operations
- Progress bar for file uploads
- Pulsing dot for background tasks

**Implementation**:
```rust
// Network connecting
let spinner = Animation::pulse(100, 255, Duration::from_millis(600));
state.animation_manager.add("network_spinner", spinner);

// File upload progress
let progress = Animation::scale(0.0, 1.0, Duration::from_secs(5));
state.animation_manager.add("upload_progress", progress);
```

**Estimated Time**: 2 hours

---

### 2.5: Transition Animations

**View Transitions**:
- Fade between views
- Slide content when navigating
- Smooth tab switching

**Implementation**:
```rust
// View change
let fade_out = Animation::fade_out(Duration::from_millis(150));
let fade_in = Animation::fade_in(Duration::from_millis(150));

// Sequence: fade out old view, change view, fade in new view
```

**Estimated Time**: 1-2 hours

---

## 📋 Phase 3: Plugin System Documentation & Examples (Estimated: 4-6 hours)

### 3.1: Plugin Development Guide

**File**: `docs/PLUGIN_DEVELOPMENT.md`

**Contents**:
1. **Overview**
   - What are plugins
   - Architecture overview
   - Static vs dynamic loading

2. **Getting Started**
   - Minimal plugin example
   - Plugin trait implementation
   - Metadata definition

3. **Lifecycle Hooks**
   - on_load / on_unload
   - Event handling
   - Command registration

4. **Event System**
   - PluginEvent types
   - Broadcasting events
   - Event filtering

5. **Command System**
   - Registering commands
   - Command execution
   - Command aliases

6. **Best Practices**
   - Error handling
   - Resource cleanup
   - Testing plugins
   - Performance considerations

**Estimated Time**: 2 hours

---

### 3.2: Example Plugins

**Plugin 1: Notification Plugin**
```rust
// examples/plugins/notifications.rs
pub struct NotificationPlugin {
    metadata: PluginMetadata,
    enabled: bool,
    sound_enabled: bool,
}

impl Plugin for NotificationPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(
            "notifications",
            "1.0.0",
            "Communitas Team",
            "Desktop notifications for new messages"
        )
    }

    fn on_event(&mut self, event: &PluginEvent) -> PluginResult<bool> {
        match event {
            PluginEvent::MessageReceived { content, sender } => {
                self.send_desktop_notification(sender, content)?;
                if self.sound_enabled {
                    self.play_sound()?;
                }
                Ok(true)
            }
            _ => Ok(false)
        }
    }
}
```

**Plugin 2: Auto-Reply Plugin**
```rust
// examples/plugins/auto_reply.rs
pub struct AutoReplyPlugin {
    metadata: PluginMetadata,
    enabled: bool,
    away_message: String,
}

impl Plugin for AutoReplyPlugin {
    fn on_event(&mut self, event: &PluginEvent) -> PluginResult<bool> {
        if !self.enabled {
            return Ok(false);
        }

        match event {
            PluginEvent::MessageReceived { sender, .. } => {
                // Send auto-reply
                Ok(true)
            }
            _ => Ok(false)
        }
    }

    fn commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand::new(
                "away",
                "Set away status",
                "away [message]"
            ),
            PluginCommand::new(
                "back",
                "Clear away status",
                "back"
            ),
        ]
    }
}
```

**Plugin 3: Message Logger Plugin**
```rust
// examples/plugins/message_logger.rs
pub struct MessageLoggerPlugin {
    metadata: PluginMetadata,
    log_file: PathBuf,
    enabled: bool,
}

impl Plugin for MessageLoggerPlugin {
    fn on_event(&mut self, event: &PluginEvent) -> PluginResult<bool> {
        match event {
            PluginEvent::MessageReceived { content, sender } => {
                self.log_message(sender, content)?;
                Ok(false) // Don't consume event
            }
            _ => Ok(false)
        }
    }
}
```

**Estimated Time**: 2 hours

---

### 3.3: Plugin API Documentation

**File**: `docs/api/PLUGIN_API.md`

**Contents**:
- Plugin trait reference
- PluginManager API
- Event types
- Error handling
- Testing utilities

**Auto-generated from rustdoc**:
```bash
cargo doc --no-deps --open --package communitas-tui
```

**Estimated Time**: 1 hour

---

### 3.4: Plugin Integration Examples

**File**: `docs/PLUGIN_INTEGRATION.md`

**Examples**:

**Registering Plugins**:
```rust
// In main.rs or app initialization
let mut plugin_manager = PluginManager::new();

// Register built-in plugins
plugin_manager.register(Box::new(NotificationPlugin::new()))?;
plugin_manager.register(Box::new(MessageLoggerPlugin::new()))?;

// Load all plugins
for name in ["notifications", "message_logger"] {
    plugin_manager.load(name)?;
}

// Add to AppState
state.plugin_manager = plugin_manager;
```

**Broadcasting Events**:
```rust
// When message received
let event = PluginEvent::MessageReceived {
    content: message.content.clone(),
    sender: message.sender.clone(),
};

let handlers = state.plugin_manager.broadcast_event(&event)?;
tracing::info!("Message handled by {} plugins", handlers.len());
```

**Executing Commands**:
```rust
// Parse command
if input.starts_with('/') {
    let parts: Vec<&str> = input.trim_start_matches('/').split_whitespace().collect();
    if let Some(command) = parts.first() {
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        if let Some(result) = state.plugin_manager.execute_command(command, &args)? {
            state.set_status(&result);
        } else {
            state.set_status(&format!("Unknown command: /{}", command));
        }
    }
}
```

**Estimated Time**: 1 hour

---

## 📊 Implementation Schedule

### Week 1: Component Integration
- **Day 1-2**: AppState integration + ResizableSplit
- **Day 3**: TreeView integration
- **Day 4-5**: CommandPalette integration + Animation system

### Week 2: Polish & Documentation
- **Day 1-2**: Animation integration (modals, notifications, errors)
- **Day 3**: Loading & transition animations
- **Day 4-5**: Plugin documentation + examples

---

## ✅ Testing Strategy

### Unit Tests
- [ ] ResizableSplit mouse interaction
- [ ] TreeView keyboard navigation
- [ ] CommandPalette fuzzy search
- [ ] Animation timing and easing
- [ ] Plugin lifecycle

### Integration Tests
- [ ] Dashboard split persistence
- [ ] Tree view with real entity data
- [ ] Command palette execution
- [ ] Animation performance (60fps)
- [ ] Plugin event broadcasting

### Manual Testing
- [ ] Resize panels with mouse
- [ ] Navigate tree with keyboard and mouse
- [ ] Execute all commands via palette
- [ ] Verify animations are smooth
- [ ] Test plugin loading/unloading

---

## 🎯 Success Criteria

**Phase 1: Component Integration**
- ✅ All 5 components integrated into app
- ✅ Zero compilation errors
- ✅ All keyboard shortcuts working
- ✅ Mouse interaction fully functional
- ✅ Components accessible from all views

**Phase 2: Visual Polish**
- ✅ Smooth 60fps animations
- ✅ Professional modal transitions
- ✅ Clear visual feedback for all actions
- ✅ Consistent animation timing
- ✅ No jank or stuttering

**Phase 3: Plugin Documentation**
- ✅ Comprehensive development guide
- ✅ 3+ working example plugins
- ✅ API documentation complete
- ✅ Integration examples clear
- ✅ Easy for developers to extend

---

## 📝 Notes

### Performance Considerations
- Animation updates should be <1ms per frame
- Plugin event broadcasting should be async-safe
- TreeView should virtualize for 1000+ nodes
- Command palette search should be <50ms

### Accessibility
- All animations should respect reduced motion preference
- Keyboard alternatives for all mouse actions
- Screen reader announcements for state changes
- High contrast mode compatibility

### Future Enhancements
- Dynamic plugin loading (.so/.dylib/.dll)
- Lua/Rhai scripting support
- Plugin marketplace
- Animation presets and themes
- Custom command palette themes

---

**Last Updated**: 2025-10-19
**Next Review**: After Phase 1 completion
