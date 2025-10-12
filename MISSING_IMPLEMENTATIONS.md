# Missing Implementations - Communitas Storyboard

**Version**: 1.0 • **Date**: 2025-10-12

This document lists all missing implementations needed to complete the storyboard design in both Tauri and TUI apps.

---

## 🔴 Critical Missing Backend Commands

### 1. Organization Management

**Location**: `communitas-desktop/src/commands/org_commands.rs`

```rust
// ❌ Missing Commands

#[tauri::command]
pub async fn core_org_create(
    four_words: String,
    name: String,
    description: Option<String>,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Organization, String> {
    // Implementation needed:
    // 1. Validate four_words format
    // 2. Create organization entity in CoreContext
    // 3. Initialize organization storage vault
    // 4. Create default channels (#general)
    // 5. Set creator as owner
    // 6. Publish to gossip overlay
    unimplemented!("core_org_create")
}

#[tauri::command]
pub async fn core_org_list(
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<Organization>, String> {
    // Implementation needed:
    // 1. Query all organizations user is member of
    // 2. Include role information
    // 3. Include online member count
    // 4. Sort by recent activity
    unimplemented!("core_org_list")
}

#[tauri::command]
pub async fn core_org_get(
    org_id: String,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<OrganizationDetails, String> {
    // Implementation needed:
    // 1. Fetch organization metadata
    // 2. Include member list with roles
    // 3. Include channel list
    // 4. Include project list
    // 5. Include storage info
    unimplemented!("core_org_get")
}

#[tauri::command]
pub async fn core_org_list_members(
    org_id: String,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<Member>, String> {
    // Implementation needed:
    // 1. List all organization members
    // 2. Include roles (Owner/Admin/Member/Bot)
    // 3. Include online status (via gossip presence)
    // 4. Include department/team info
    unimplemented!("core_org_list_members")
}

#[tauri::command]
pub async fn core_org_update_member_role(
    org_id: String,
    member_id: String,
    new_role: String, // "Owner" | "Admin" | "Member"
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<(), String> {
    // Implementation needed:
    // 1. Verify caller has permission (Owner/Admin)
    // 2. Update member role in organization
    // 3. Sync via CRDT
    unimplemented!("core_org_update_member_role")
}
```

**Types Needed** (`communitas-core/src/types.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub four_words: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub member_count: usize,
    pub online_member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDetails {
    pub id: String,
    pub four_words: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub members: Vec<Member>,
    pub channels: Vec<ChannelSummary>,
    pub projects: Vec<ProjectSummary>,
    pub storage_info: StorageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub four_words: String,
    pub display_name: String,
    pub initials: String,
    pub role: MemberRole,
    pub department: Option<String>,
    pub status: PresenceStatus,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceStatus {
    Online,
    Away,
    Offline,
}
```

### 2. Storage Visualization

**Location**: `communitas-desktop/src/storage_fs.rs`

```rust
// ❌ Missing Commands

#[tauri::command]
pub async fn core_storage_get_vault_info(
    entity_id: String,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<VaultInfo, String> {
    // Implementation needed:
    // 1. Get vault for entity
    // 2. Calculate total size
    // 3. Calculate used size
    // 4. Get encryption type
    // 5. Get replication factor
    unimplemented!("core_storage_get_vault_info")
}

#[tauri::command]
pub async fn core_storage_update_vault_settings(
    entity_id: String,
    encryption_type: Option<String>, // "ML-KEM" | "AES-256"
    replication_factor: Option<u8>,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<(), String> {
    // Implementation needed:
    // 1. Verify caller has permission
    // 2. Update vault settings
    // 3. Re-encrypt if encryption type changed
    // 4. Update replication via gossip
    unimplemented!("core_storage_update_vault_settings")
}
```

**Types Needed**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub vault_id: String,
    pub entity_id: String,
    pub vault_type: VaultType,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub encryption_type: String,
    pub replication_factor: u8,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultType {
    OrgVault,      // Encrypted, replicated across bootstrap nodes
    WebStorage,    // S3-compatible virtual disk
    Personal,      // Private encrypted storage
}
```

### 3. Search & Discovery

**Location**: `communitas-desktop/src/core_commands.rs`

```rust
// ❌ Missing Commands

#[tauri::command]
pub async fn core_search_entities(
    query: String,
    entity_types: Option<Vec<String>>, // ["organization", "channel", "project", "group"]
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<SearchResult>, String> {
    // Implementation needed:
    // 1. Search organizations by name/four-words
    // 2. Search channels by name
    // 3. Search projects by name
    // 4. Search groups by name
    // 5. Rank by relevance
    unimplemented!("core_search_entities")
}

#[tauri::command]
pub async fn core_search_messages(
    query: String,
    channel_id: Option<String>,
    from_date: Option<String>,
    to_date: Option<String>,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<MessageSearchResult>, String> {
    // Implementation needed:
    // 1. Search message content
    // 2. Search message attachments
    // 3. Filter by channel if specified
    // 4. Filter by date range
    // 5. Rank by relevance
    unimplemented!("core_search_messages")
}

#[tauri::command]
pub async fn core_search_files(
    query: String,
    entity_id: Option<String>,
    file_types: Option<Vec<String>>, // ["image", "document", "video", etc.]
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<FileSearchResult>, String> {
    // Implementation needed:
    // 1. Search file names
    // 2. Search file content (for text files)
    // 3. Filter by entity if specified
    // 4. Filter by file type
    // 5. Rank by relevance
    unimplemented!("core_search_files")
}
```

**Types Needed**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchResult {
    Organization {
        id: String,
        four_words: String,
        name: String,
        member_count: usize,
    },
    Channel {
        id: String,
        org_id: String,
        name: String,
        member_count: usize,
    },
    Project {
        id: String,
        org_id: String,
        name: String,
        status: String,
    },
    Group {
        id: String,
        four_words: String,
        name: String,
        member_count: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSearchResult {
    pub message_id: String,
    pub channel_id: String,
    pub channel_name: String,
    pub author_name: String,
    pub content: String,
    pub match_snippet: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub file_path: String,
    pub entity_id: String,
    pub entity_name: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub match_snippet: Option<String>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}
```

### 4. Group Management (Expansion)

**Location**: `communitas-desktop/src/core_groups.rs`

```rust
// ❌ Missing Commands (to complement existing ones)

#[tauri::command]
pub async fn core_group_list(
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<Group>, String> {
    // Implementation needed:
    // 1. List all groups user is member of
    // 2. Include member count
    // 3. Include online member count
    // 4. Sort by recent activity
    unimplemented!("core_group_list")
}

#[tauri::command]
pub async fn core_group_get(
    group_id: String,
    core_state: tauri::State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<GroupDetails, String> {
    // Implementation needed:
    // 1. Get group metadata
    // 2. Get member list
    // 3. Get recent messages
    // 4. Get storage info
    unimplemented!("core_group_get")
}
```

**Types Needed**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub four_words: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub member_count: usize,
    pub online_member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDetails {
    pub id: String,
    pub four_words: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub members: Vec<Member>,
    pub storage_info: StorageInfo,
}
```

---

## 🟡 Missing React Components (Tauri Frontend)

### 1. Shell Components

**Location**: `src/components/shell/`

```typescript
// ❌ Missing: AppShell.tsx
export interface AppShellProps {
  children?: React.ReactNode;
}

export const AppShell: React.FC<AppShellProps> = ({ children }) => {
  // Three-panel layout:
  // - EntitySidebar (340px fixed left)
  // - MainContent (flex-grow center)
  // - InfoPanel (320px fixed right, optional)

  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      <EntitySidebar width={340} />
      <MainContent flex={1} />
      {/* InfoPanel is optional and toggleable */}
    </Box>
  );
};

// ❌ Missing: EntitySidebar.tsx
export const EntitySidebar: React.FC<{ width: number }> = ({ width }) => {
  // Sections:
  // 1. Identity Selector (top)
  // 2. Filter Chips (two rows)
  // 3. Search Bar (⌘K)
  // 4. Entity Tree (scrollable)

  const [orgs, setOrgs] = useState<Organization[]>([]);
  const [expandedOrgs, setExpandedOrgs] = useState<Set<string>>(new Set());

  useEffect(() => {
    // Load organizations
    invoke<Organization[]>('core_org_list').then(setOrgs);
  }, []);

  return (
    <Box sx={{ width, background: '#1a1f24', overflowY: 'auto' }}>
      <IdentitySelector />
      <FilterChips />
      <SearchBar />
      <OrgTree orgs={orgs} expanded={expandedOrgs} />
    </Box>
  );
};

// ❌ Missing: MainContent.tsx
export const MainContent: React.FC<{ flex: number }> = ({ flex }) => {
  // Dynamic content based on selection
  const { selectedEntity } = useEntityContext();

  return (
    <Box sx={{ flex, background: '#161C20', overflow: 'auto' }}>
      {selectedEntity?.type === 'organization' && <OrgDashboard org={selectedEntity} />}
      {selectedEntity?.type === 'channel' && <ChatInterface channel={selectedEntity} />}
      {selectedEntity?.type === 'storage' && <StorageBrowser entity={selectedEntity} />}
    </Box>
  );
};

// ❌ Missing: InfoPanel.tsx
export const InfoPanel: React.FC<{ width: number }> = ({ width }) => {
  // Shows details about selected entity
  // - Member list
  // - File preview
  // - Settings

  return (
    <Box sx={{ width, background: '#1a1f24', overflowY: 'auto' }}>
      {/* Entity-specific details */}
    </Box>
  );
};
```

### 2. Organization Components

**Location**: `src/components/organization/`

```typescript
// ❌ Missing: OrgDashboard.tsx
export const OrgDashboard: React.FC<{ org: Organization }> = ({ org }) => {
  // 2x2 grid layout:
  // - Members (top-left)
  // - Projects (top-right)
  // - Channels (bottom-left)
  // - Storage (bottom-right)

  return (
    <Box>
      <OrgHeader org={org} />
      <Grid container spacing={2.5}>
        <Grid item xs={12} md={6}>
          <MembersCard orgId={org.id} />
        </Grid>
        <Grid item xs={12} md={6}>
          <ProjectsCard orgId={org.id} />
        </Grid>
        <Grid item xs={12} md={6}>
          <ChannelsCard orgId={org.id} />
        </Grid>
        <Grid item xs={12} md={6}>
          <StorageCard orgId={org.id} />
        </Grid>
      </Grid>
    </Box>
  );
};

// ❌ Missing: OrgTree.tsx
export const OrgTree: React.FC<{
  orgs: Organization[];
  expanded: Set<string>;
  onToggle: (orgId: string) => void;
}> = ({ orgs, expanded, onToggle }) => {
  // Expandable tree with:
  // - Organization header (name, online count)
  // - Channels list
  // - Projects list
  // - Teams list

  return (
    <List>
      {orgs.map(org => (
        <OrgTreeNode
          key={org.id}
          org={org}
          expanded={expanded.has(org.id)}
          onToggle={() => onToggle(org.id)}
        />
      ))}
    </List>
  );
};

// ❌ Missing: MemberCard.tsx
export const MemberCard: React.FC<{ member: Member }> = ({ member }) => {
  // Displays:
  // - Avatar (gradient, initials)
  // - Name
  // - Role badge
  // - Status indicator (online/away/offline)
  // - Actions (edit, message, remove)

  return (
    <Card>
      <Avatar gradient={getGradientForUser(member.id)}>
        {member.initials}
      </Avatar>
      <Box>
        <Typography>{member.display_name}</Typography>
        <Typography variant="caption">
          {member.role} · {member.status}
        </Typography>
      </Box>
      <MemberActions member={member} />
    </Card>
  );
};

// ❌ Missing: ProjectCard.tsx
export const ProjectCard: React.FC<{ project: Project }> = ({ project }) => {
  // Displays:
  // - Project name
  // - Status (Active/Archived)
  // - Member count
  // - Actions (open, archive)

  return (
    <Card>
      <Typography variant="h6">{project.name}</Typography>
      <Typography variant="caption">
        {project.status} · {project.member_count} members
      </Typography>
      <ProjectActions project={project} />
    </Card>
  );
};
```

### 3. Storage Components

**Location**: `src/components/storage/`

```typescript
// ❌ Missing: StorageMeter.tsx
export const StorageMeter: React.FC<{
  used: number;
  total: number;
  unit: string;
  variant?: 'success' | 'warning' | 'danger';
}> = ({ used, total, unit, variant }) => {
  const percentage = (used / total) * 100;

  // Color based on threshold:
  // - 0-60%: green (#2EB67D)
  // - 60-80%: yellow (#F5B759)
  // - 80-100%: red (#E25555)

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
        <Typography>{used} {unit} / {total} {unit}</Typography>
        <Typography color={getColorForPercentage(percentage)}>
          {percentage.toFixed(0)}%
        </Typography>
      </Box>
      <LinearProgress
        variant="determinate"
        value={percentage}
        sx={{
          height: 8,
          borderRadius: 4,
          backgroundColor: '#1F262C',
          '& .MuiLinearProgress-bar': {
            backgroundColor: getColorForPercentage(percentage),
          },
        }}
      />
    </Box>
  );
};

// ❌ Missing: VaultSettings.tsx
export const VaultSettings: React.FC<{
  entityId: string;
  vaultInfo: VaultInfo;
}> = ({ entityId, vaultInfo }) => {
  // Settings:
  // - Encryption type (ML-KEM / AES-256)
  // - Replication factor (1-5)
  // - Sync settings (auto-sync on/off)

  return (
    <Box>
      <FormControl>
        <InputLabel>Encryption Type</InputLabel>
        <Select value={vaultInfo.encryption_type}>
          <MenuItem value="ML-KEM">ML-KEM (Post-Quantum)</MenuItem>
          <MenuItem value="AES-256">AES-256</MenuItem>
        </Select>
      </FormControl>

      <FormControl>
        <InputLabel>Replication Factor</InputLabel>
        <Select value={vaultInfo.replication_factor}>
          {[1, 2, 3, 4, 5].map(n => (
            <MenuItem key={n} value={n}>{n}</MenuItem>
          ))}
        </Select>
      </FormControl>
    </Box>
  );
};
```

### 4. Common Components

**Location**: `src/components/common/`

```typescript
// ❌ Missing: FilterChip.tsx
export const FilterChip: React.FC<{
  active: boolean;
  children: React.ReactNode;
  onClick: () => void;
}> = ({ active, children, onClick }) => {
  return (
    <Button
      variant={active ? 'contained' : 'outlined'}
      onClick={onClick}
      sx={{
        padding: '6px 12px',
        background: active ? '#2EB67D' : 'transparent',
        color: active ? '#101518' : '#9AA2AB',
        border: `1px solid ${active ? '#2EB67D' : '#2a3038'}`,
        borderRadius: 1.5,
        fontSize: 12,
        transition: 'all 0.3s',
        '&:hover': {
          background: active ? '#26A86B' : 'rgba(46, 182, 125, 0.1)',
          transform: 'translateY(-2px)',
        },
      }}
    >
      {children}
    </Button>
  );
};

// ❌ Missing: Avatar.tsx
export const Avatar: React.FC<{
  initials: string;
  size?: number;
  gradient?: boolean;
}> = ({ initials, size = 40, gradient = true }) => {
  // Deterministic gradient based on user ID
  const gradientStyle = gradient
    ? 'linear-gradient(135deg, #2EB67D, #1E88E5)'
    : '#2EB67D';

  return (
    <Box
      sx={{
        width: size,
        height: size,
        borderRadius: '50%',
        background: gradientStyle,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: 'white',
        fontWeight: 'bold',
        fontSize: size * 0.4,
      }}
    >
      {initials}
    </Box>
  );
};
```

---

## 🟠 Missing TUI Components

### Location: `communitas-tui/src/ui/`

```rust
// ❌ Missing: shell.rs
pub struct ShellLayout {
    pub sidebar: EntitySidebar,
    pub main_content: MainContent,
    pub info_panel: Option<InfoPanel>,
}

impl ShellLayout {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Split into three panels
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(40), // Sidebar
                Constraint::Min(0),      // Main content
                Constraint::Length(30),  // Info panel (if visible)
            ])
            .split(area);

        self.sidebar.render(frame, chunks[0]);
        self.main_content.render(frame, chunks[1]);

        if let Some(ref panel) = self.info_panel {
            panel.render(frame, chunks[2]);
        }
    }
}

// ❌ Missing: sidebar.rs
pub struct EntitySidebar {
    pub orgs: Vec<Organization>,
    pub selected_index: usize,
    pub expanded_orgs: HashSet<String>,
}

impl EntitySidebar {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render organization tree with expandable nodes
        let items: Vec<ListItem> = self.orgs
            .iter()
            .enumerate()
            .flat_map(|(i, org)| self.render_org_node(i, org))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Entities"))
            .highlight_style(Style::default().bg(Color::Rgb(46, 182, 125)))
            .highlight_symbol("► ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}

// ❌ Missing: org_dashboard.rs
pub struct OrgDashboard {
    pub org: Organization,
    pub members: Vec<Member>,
    pub projects: Vec<Project>,
    pub channels: Vec<Channel>,
    pub storage_info: StorageInfo,
}

impl OrgDashboard {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // 2x2 grid layout
        let grid = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(grid[0]);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(grid[1]);

        self.render_members(frame, top[0]);
        self.render_projects(frame, top[1]);
        self.render_channels(frame, bottom[0]);
        self.render_storage(frame, bottom[1]);
    }
}

// ❌ Missing: storage_meter.rs
pub struct StorageMeter {
    pub used: u64,
    pub total: u64,
    pub label: String,
}

impl StorageMeter {
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let percentage = (self.used as f64 / self.total as f64) * 100.0;

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(&self.label))
            .gauge_style(Self::color_for_percentage(percentage))
            .percent(percentage as u16)
            .label(format!("{:.1}% ({} / {})",
                percentage,
                Self::format_size(self.used),
                Self::format_size(self.total)
            ));

        frame.render_widget(gauge, area);
    }

    fn color_for_percentage(percentage: f64) -> Style {
        if percentage < 60.0 {
            Style::default().fg(Color::Rgb(46, 182, 125)) // Green
        } else if percentage < 80.0 {
            Style::default().fg(Color::Rgb(245, 183, 89)) // Yellow
        } else {
            Style::default().fg(Color::Rgb(226, 85, 85)) // Red
        }
    }
}
```

---

## 📅 Implementation Timeline

### Week 1: Backend Foundation
- **Day 1-2**: Implement organization management commands
- **Day 3**: Implement storage visualization commands
- **Day 4**: Implement search commands
- **Day 5**: Testing and integration

### Week 2: Tauri Components
- **Day 1-2**: Create shell components (AppShell, EntitySidebar, MainContent)
- **Day 3**: Create organization components (OrgDashboard, OrgTree, MemberCard)
- **Day 4**: Create storage components (StorageMeter, VaultSettings)
- **Day 5**: Create common components (FilterChip, Avatar)

### Week 3: TUI Implementation
- **Day 1-2**: Refactor TUI structure, create shell layout
- **Day 3**: Implement org dashboard in TUI
- **Day 4**: Implement storage meters and file browser in TUI
- **Day 5**: Polish and testing

### Week 4: Testing & Integration
- **Day 1-2**: E2E testing in Tauri
- **Day 3**: E2E testing in TUI
- **Day 4**: Performance testing and optimization
- **Day 5**: Documentation and final polish

---

**Total Estimated Effort**: 4 weeks (160 hours)

**Priority Levels**:
- 🔴 Critical - Blocks storyboard completion
- 🟡 Important - Enhances storyboard features
- 🟠 Nice-to-have - Can be added later

All items listed above are **🔴 Critical** for storyboard completion.

---

**End of Missing Implementations Document**
