# Modern Shell Prototype Testing Checklist

**URL**: http://localhost:5173/prototype/modern-shell

## Test Cases

### 1. Navigation - System Rail (Left Side)
- [ ] Click **Home** button (house icon) - should reset to default conversation
- [ ] Click **Chats** button - should show chat-related conversations
- [ ] Click **Organizations** button - should filter to organizations
- [ ] Click **Discover** button - explore feature
- [ ] Click **Storage** button - storage view
- [ ] Click **Website** button (only visible when entity has website) - should highlight in accent color
- [ ] Click **Calls** button - phone/video interface
- [ ] Click **Settings** button (bottom) - settings panel
- [ ] Click **Avatar** (bottom) - user profile

**Expected**: All buttons should respond with hover effects (light up) and proper color transitions

---

### 2. Conversation List Filters (Top of middle panel)
- [ ] Click **All** - shows all conversations
- [ ] Click **Unread** - filters to conversations with unread count > 0
- [ ] Click **Favourites** - shows pinned conversations
- [ ] Click **Groups** - shows only group conversations
- [ ] Click **Projects** - shows only project conversations
- [ ] Click **People** - shows only person-to-person conversations

**Expected**: Active filter should have accent green background, filtered list updates correctly

---

### 3. Conversation Selection (Middle Panel)
Test clicking through each conversation type:

#### A. Groups
- [ ] Click **"Saorsa Labs (Group)"**
  - Should show group chat
  - Header shows "3 online"
  - Chat timeline with messages visible
  - Composer bar at bottom

#### B. Organizations
- [ ] Click **"Saorsa Labs (Org)"**
  - Should show **Organization Overview Grid** (not chat)
  - 4 cards: Members, Projects, Channels, Storage
  - NO composer bar (organization overview doesn't have chat)

#### C. Channels
- [ ] Click **"General (Channel)"**
  - Header shows channel topic and member count
  - **View mode chips appear** below header: Chat | Threads | Files | Integrations
  - Default view: Chat mode

**Channel View Mode Tests**:
- [ ] Click **Chat** chip - shows message timeline
- [ ] Click **Threads** chip - shows thread previews
- [ ] Click **Files** chip - shows file list with Open/Share buttons
- [ ] Click **Integrations** chip - shows connected integrations

#### D. Projects
- [ ] Click **"Project Lumos"**
  - Header shows status, owner, completion percentage
  - **View mode chips appear**: Chat | Board | Tasks | Timeline
  - Default view: Chat mode

**Project View Mode Tests**:
- [ ] Click **Chat** chip - shows message timeline
- [ ] Click **Board** chip - shows kanban board (Backlog/In Progress/Review)
- [ ] Click **Tasks** chip - shows task list
- [ ] Click **Timeline** chip - shows activity timeline

#### E. Storage
- [ ] Click **"Storage Ops"**
  - Shows storage-related chat
  - Storage icon badge on avatar

#### F. People (1:1 Chats)
- [ ] Click **"Ben Thomson"** - online indicator (green badge)
- [ ] Click **"Lauren McFadyen"** - offline (gray badge)

**Expected**: Each conversation type shows correct avatar shape, badges, and content area

---

### 4. Organization Overview Actions
When viewing **"Saorsa Labs (Org)"**:

#### Members Card
Hover over each member row:
- [ ] **Edit button** appears (pencil icon)
- [ ] **Message privately** button appears (reply icon)
- [ ] **Remove button** appears (trash icon, red color)
- [ ] Click Edit - logs to console
- [ ] Click Remove - logs to console

#### Projects Card
Hover over each project:
- [ ] **Archive button** appears (archive icon)
- [ ] Click Archive - logs to console

#### Channels Card
- [ ] Hover shows members preview (currently just styling change)

#### Storage Card
For each storage container:
- [ ] Progress bars show correct percentage
- [ ] Color coding: Green (Healthy) or Orange (Syncing)
- [ ] Action buttons: "Open", "Manage", "Mount", "Share link"

---

### 5. Context Drawer (Right Panel)
- [ ] Click **Info icon** (top right of header) - drawer slides in from right
- [ ] Click **Info icon** again - drawer slides out
- [ ] When drawer open, test all tabs:
  - [ ] Overview
  - [ ] Members
  - [ ] Files
  - [ ] Tasks
  - [ ] Timeline
  - [ ] Storage

**Expected**: Active tab has accent green background, content area shows placeholder text

---

### 6. Message Interactions (Chat View)
When viewing a conversation with messages:

#### Hover Actions
- [ ] Hover over message - action toolbar appears below message
- [ ] Toolbar shows icons for:
  - [ ] More options dropdown (⌄)
  - [ ] React (emoji)
  - [ ] Reply
  - [ ] Reply privately (lock)
  - [ ] Start new thread
  - [ ] Share link

#### Message Features
- [ ] Messages show timestamp
- [ ] Sent messages show read receipts (blue checkmark)
- [ ] Delivered messages show double gray checkmark
- [ ] Reactions appear below messages (thumbs up, party, etc.)
- [ ] Thread previews show "View thread · 3 · Latest from Lauren"

#### More Options Menu
- [ ] Click dropdown arrow on message
- [ ] Menu appears with options:
  - Reply
  - Forward
  - Copy
  - Star
  - Report
  - Delete

---

### 7. Search Functionality
- [ ] Click search box in conversation list
- [ ] Type search term
- [ ] Search box highlights on focus

---

### 8. Composer (Message Input)
Only visible in chat modes, NOT in organization overview:
- [ ] Emoji button (left)
- [ ] Attach file button
- [ ] Text input field (type message)
- [ ] Send button (right, accent green)

---

### 9. Visual Quality Checks

#### Colors & Theme
- [ ] Background: Dark theme (#101518 base)
- [ ] Accent color: Green (#2EB67D) for active states
- [ ] Text: White primary, gray secondary
- [ ] Borders: Subtle dark lines

#### Shapes & Avatars
- [ ] Person: Circular avatar
- [ ] Group: Rounded square (16px radius)
- [ ] Project: House-shaped (rounded bottom)
- [ ] Channel: Rounded square with white border
- [ ] Storage: Rounded square with subtle border
- [ ] Organization: Rounded square with green accent border

#### Badge Icons
- [ ] Storage: Green storage icon
- [ ] Organization: Green building icon
- [ ] Project: Green folder icon
- [ ] Channel: Gray grid icon
- [ ] Group: Gray people icon
- [ ] Person: Green dot (online) or gray dot (offline)

#### Hover Effects
- [ ] All interactive elements have smooth transitions (120ms)
- [ ] Conversation items lift slightly on hover
- [ ] Buttons change color on hover
- [ ] No jarring or broken animations

---

### 10. Responsive Behavior
- [ ] Resize browser window to small width
- [ ] UI adjusts to compact layout (280px conversation list)
- [ ] All features remain accessible

---

### 11. Console Check
Open browser DevTools (F12 or Cmd+Opt+I):

**Console Tab**:
- [ ] No red errors
- [ ] No warnings about missing props
- [ ] Navigation events log correctly
- [ ] Action clicks log to console

**Network Tab**:
- [ ] All assets load successfully (no 404s)
- [ ] No failed requests

**Performance**:
- [ ] Smooth scrolling in conversation list
- [ ] Smooth scrolling in message timeline
- [ ] No lag when switching conversations
- [ ] No lag when switching view modes

---

## Known Issues to Watch For

1. **Mode Switching**: View mode chips should reset to "Chat" when switching conversations
2. **Drawer State**: Drawer should remain open/closed when switching conversations
3. **Filter State**: Active filter should persist across conversation switches
4. **Message Menu**: Context menu should close when clicking outside

---

## Screenshot Checklist

Please capture screenshots of:
1. **Default view** (Saorsa Labs Group selected)
2. **Organization overview** (all 4 cards visible)
3. **Channel threads view** (thread list)
4. **Project board view** (kanban columns)
5. **Project tasks view** (task list)
6. **Context drawer open** (showing tabs)
7. **Message hover state** (action toolbar visible)
8. **Message context menu** (dropdown open)
9. **Different avatar shapes** (all entity types in list)
10. **Storage container card** (with progress bars)

---

## Bug Report Template

If you find issues, report using this format:

```
**Bug**: [Brief description]
**Steps to Reproduce**:
1.
2.
3.

**Expected**: [What should happen]
**Actual**: [What actually happens]
**Screenshot**: [Attach if visual issue]
**Console Error**: [Copy any error messages]
```

---

## Performance Notes

- Check for smooth 60fps scrolling
- Mode switches should be instant (<100ms)
- No layout shifts or flashing content
- Hover effects should be smooth and consistent
