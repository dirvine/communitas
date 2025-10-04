# Communitas Modern Interface Specification

_Last updated: 2025-10-03_

## 1. Design Goals

- **Familiar flow, Communitas power**: deliver WhatsApp-level ease of use while exposing organisations, channels, projects, storage status, and bootstrap diagnostics without modal sprawl.
- **Single code vocabulary**: shared tokens and component APIs for desktop (React/Tauri) and TUI (ratatui) clients.
- **Minimal cognitive load**: dark-first theme, low-chroma neutrals, limited accent colour, and predictable typography hierarchy.
- **Performance & accessibility**: virtualised lists, 60 fps scroll, keyboard-first navigation, WCAG AA contrast, high-contrast override.

## 2. Layout System

```
┌──────────────┬────────────────────────────┬───────────────────────────┐
│  A. System   │   B. Conversation List     │   C. Conversation Pane    │
│     Rail     │                            │                           │
│              │   B1. Header + Filters     │   C1. Header (chat info)  │
│   Home (Chats│   B2. Search & Quick Add   │   C2. Message Timeline    │
│   Orgs       │   B3. List Items           │   C3. Composer & FAB      │
│   Discover   │                            │   C4. Context Drawer      │
│   Storage    │                            │     (slide-in)            │
│   Settings   │                            │                           │
└──────────────┴────────────────────────────┴───────────────────────────┘
```

### A. System Rail (52 px)

- Primary shortcuts: Chats, Organisations, Discover, Calls, Storage, Settings.
- Footer: user avatar + presence indicator; long-press opens account switcher.
- Hover tooltips expose keyboard accelerators (`⌘+1` etc.).
- Storage icon opens overlay summarising encrypted vault size, FEC health, web node status.

### B. Conversation Column (320–360 px)

1. **Header Row** – Title + status pill (online/offline), overflow menu for column preferences.
2. **Filter Chips** – All · Unread · Favourites · Groups · Projects · People. Selected chip uses accent background and numeric badge. When scoped to an organisation, a pill shows `Saorsa Labs ▾` and clicking opens org switcher.
3. **Search & Quick Add** – Search field with smart tokens (`org:`, `project:`, `storage:low`). `+` button launches creation modal (direct message, project, channel, organisation invite).
4. **List Items**
   - Avatar treatments communicate type: circle for people (with online dot), rounded-square for groups, hex/card for projects, shield for organisations. Presence icon rendered top-right (green dot = online, hollow = offline, bolt = automation).
   - Primary text: name; secondary text: latest message snippet or activity (e.g., "Backup completed" for storage channel).
   - Status ticks (sent/delivered/read) mirrored from WhatsApp semantics.
   - Badges: unread count, "Pinned", "Muted", "Synced" (for storage tasks) using icon glyphs.
   - Swipe/hover affordances: favourite ☆, mute 🔕, archive ⬇.

### C. Conversation Pane (Flexible width)

1. **Header** – Avatar, title, context text (e.g. `Project · 6 members`), quick actions (call, video, share, drawer). Organisation breadcrumb chip returns to scoped list.
2. **Message Timeline**
   - Bubbles with 16 px radius, 12 px horizontal padding.
   - Sent bubbles accent 10% tint; received bubbles base grey.
   - Metadata row (time + ticks) aligned bottom-right. Reaction row reveals on hover or long-press.
   - Thread preview pill (`View thread · 3 replies`) below bubble with responder avatar; opens inline thread panel. Channels expose mode chips (`Chat | Threads | Files | Integrations`); projects expose (`Chat | Board | Tasks | Timeline`) for quick pivots into context views.
   - Quick actions on hover: emoji picker, `Reply`, `Reply privately`, `Start thread`, `Forward`.
   - System events ("Storage synced", "Member joined") as centred pill with subtle border.
   - File previews: card with icon, filename, size, FEC shards indicator. Images open immersive lightbox or web storage preview.
3. **Composer**
   - Pill container (44 px) on frosted backdrop; icons for emoji, attachments, recording, slash commands.
   - Slash commands produce inline popover (e.g., `/task`, `/note`, `/handoff`).
   - Mentions auto-complete for `@people`, `#projects`, `/storage` tasks.
4. **Inline Utilities** – Inline buttons for `Add members`, `Mute`, `Share link`, plus badges showing participant count and online presence. Header actions include `Add to project`, `Assign storage container`, `View org overview` (for admins).
5. **Floating Actions** – Bottom-right stack (`Call`, `Screen Share`) using accent gradient.
6. **Context Drawer** (320 px) – toggled icon. Tabs: Overview · Members · Files · Tasks · Timeline · Storage.
   - Overview: description, pinned highlights, invite links, encryption status, thread analytics.
   - Members: role chips, presence, quick DM, `Add`/`Remove` controls, `Reply privately` shortcut.
   - Files: filterable list, sync state indicator, FEC repair actions, web storage links.
   - Tasks: project Kanban summary with quick create (`/task`).
   - Timeline: audit events (join/leave, thread start, sync ops).
   - Storage: vault usage, virtual disks (org & personal web storage), remote node status, backup schedule, quick actions (`Open`, `Mount`, `Share`).

### Organisation Overview Mode

- Selecting an organisation switches the pane to an overview canvas (no timeline) with sections: Members, Projects, Channels, Storage. Each section is a hoverable card revealing its list (e.g. hover “Projects” to see pinned projects, hover “Channels” to preview membership counts). Quick actions: invite, create channel/project, open storage dashboard.
- Organisation storage module lists vault + virtual disk containers with usage bars and mount/share controls.

### Responsive & TUI

- ≤ 960 px: collapse context drawer into modal. System rail shrinks to icon strip with bottom sheet for overflow.
- TUI replicates three-pane grid with ASCII separators. Drawer toggles with `i` and overlays rightmost third.

## 3. Visual Tokens

| Token | Dark | Light | Usage |
|-------|------|-------|-------|
| `bg.base` | #101518 | #FFFFFF | App background |
| `bg.raised` | #161C20 | #F4F6F8 | Cards, drawers |
| `surface.active` | #1E252B | #E9EEF2 | Selected list items |
| `border.subtle` | #1F262C | #D2D8DF | Hairlines |
| `text.primary` | #F4F6F8 | #1F262C | Titles, body |
| `text.secondary` | #9AA2AB | #4B5865 | Metadata |
| `text.inverse` | #0C0F11 | #FFFFFF | Accent text |
| `accent.primary` | #2EB67D | #15694A | Buttons, ticks |
| `accent.muted` | rgba(46, 182, 125, 0.15) | rgba(21, 105, 74, 0.12) | Selected chips |
| `danger` | #E25555 | #C13C3C | Alerts |
| `warning` | #F5B759 | #C88722 | Storage warnings |
| `success` | #4FB579 | #2A7C4E | Completed sync |

**Typography**

| Style | Weight | Size/Line | Usage |
|-------|--------|-----------|-------|
| Display | 600 | 18 / 24 | Column headings |
| Title | 600 | 16 / 22 | Conversation header |
| Body | 400 | 14 / 20 | Messages, lists |
| Caption | 500 | 12 / 16 | Metadata, ticks |
| Mono | 500 | 12 / 16 | Commands, code blocks |

**Spacing Grid** – 4 px base. Key increments: 8 (chips), 12 (bubbles), 16 (containers), 24 (drawer padding).

**Radius** – 8 px global, 16 px for message bubbles, 22 px for composer pill, 28 px for floating button.

## 4. Interaction Model

- **Global palette** (`⌘+K` / `Ctrl+K`) – search, navigation, commands. Supports natural-language tokens ("new project for marketing").
- **Slash commands** – `/task`, `/note`, `/storage-status`, `/handoff`. Each inserts structured component in timeline.
- **Hover states** – subtle scale (1.02) + accent border on list items; message actions appear as faded icons above bubble.
- **Reactions** – long press (touch) or hover > `:` to show emoji picker (recent + custom sets). Display inline chips.
- **Keyboard** – `⌘+↑/↓` conversation navigation, `⌘+Shift+M` toggle drawer, `⌘+Shift+S` open storage overlay.

## 5. Storage & Web Features Placement

- Storage icon in system rail, context drawer `Storage` tab, slash command `/storage-status`, conversation list badges (e.g., "Backup failed" with warning dot). Home icon in the rail exits scoped organisation/project contexts; Website icon lights up when the entity publishes a site.
- Web integration: `Discover` rail icon opens marketplace panel overlay with embedded web view for available web apps/integrations. Projects can pin web links into drawer `Files` section and the Website rail button opens the entity site when present.
- Organisation-level actions accessible from drawer when scoped to organisation conversation (invite, governance, storage plan). Section cards show hover previews; clicking pivots the middle column into scoped lists (Channels, Projects, Groups, Contacts).

## 6. Accessibility & Internationalisation

- WCAG AA min contrast ensured by token table (contrast ratios documented in `tokens.json`).
- Keyboard trap prevention; focus ring uses `accent.primary` #2EB67D at 2 px.
- RTL support: bubble alignment flips, rail icons mirrored; ensure icons symmetrical.
- Font scaling 0.85×–1.3× via settings, affecting tokens.

## 7. Component Inventory

1. `AppShell` – wraps layout, manages theme, keyboard shortcuts.
2. `SystemRail` – icon buttons, tooltip system, presence bubble.
3. `ConversationFilters` – chip group with counters.
4. `ConversationList` – virtualised list, item renderer accepts conversation meta and contextual badges (threads, storage alerts).
5. `ConversationHeader` – avatar, title, actions (add/remove members, call, screen share, share link).
6. `MessageTimeline` – bubble renderer with grouping, reactions, thread previews, reply-private actions, system events.
7. `Composer` – text area, attachments, emoji picker, slash commands, thread target indicator.
8. `ContextDrawer` – tabbed container for metadata modules (Overview/Members/Files/Tasks/Timeline/Storage).
9. `StorageOverviewCard` – reused in drawer + storage overlay; shows capacity, FEC shards, virtual disk state, recent tasks.
10. `StorageContainerList` – surfaces hierarchical virtual disks (org containers + web storage) with actions (`Open`, `Mount`, `Manage`).
11. `CommandPalette` – overlay modal.

TUI variants named `RailView`, `ListPane`, `MessagePane`, `DrawerPane` with same props.

## 8. Implementation Roadmap

1. Create shared tokens package (`packages/ui-tokens`) and theme providers for React and Ratatui.
2. Build Storybook stories for primitives, ensuring test coverage (Playwright visually).
3. Implement new layout in feature branch `ui/modern-shell`, behind feature flag `modernLayout`.
4. Migrate conversation list and message timeline to new components; then integrate drawer modules.
5. Port slash command framework, command palette, and storage status surfaces.
6. Roll out TUI adaptation using token map and new layout grid.
7. Launch beta toggle in Settings; gather telemetry (navigation time, command usage).

## 9. Open Questions

- Should projects default to their own conversation or remain nested under organisations? (Current plan: projects behave like conversations with context drawer for tasks.)
- How granular should storage alerts be in chat list vs dedicated overlay? (Proposal: aggregated badge on storage icon, detailed alerts in `Storage` tab.)
- Do we expose bootstrap/node metrics inline or keep them in Settings → Diagnostics? (Recommendation: add summary in storage drawer, deep diagnostics in Settings.)

---

This spec is kept in-repo for iteration. Update the "Last updated" stamp when changes are made.
