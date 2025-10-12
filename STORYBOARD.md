# Communitas UX Storyboard & Implementation Guide

## Overview
This storyboard defines the complete user experience and implementation details for Communitas, matching the interactive prototype in `storyboard-canvas-v2.html`. Each screen includes visual design, user interactions, and technical implementation notes.

## Design System

### Color Palette
```scss
// Dark Theme (Default)
$background-primary: #161C20;    // Main content background
$background-secondary: #1a1f24;  // Sidebar & card background  
$background-tertiary: #101518;   // Input fields
$border-color: #2a3038;          // All borders
$border-secondary: #1F262C;      // Subtle borders

$accent-green: #2EB67D;          // Primary accent (success, online)
$accent-green-dark: #26A86B;     // Hover state
$accent-blue: #1E88E5;           // Secondary accent
$accent-blue-dark: #1976D2;      // Hover state
$accent-red: #E25555;            // Errors, end call
$accent-red-dark: #FF6B6B;       // Light red
$accent-yellow: #F5B759;         // Warnings (storage >60%)
$accent-yellow-dark: #E5A349;    // Dark yellow

$text-primary: #F4F6F8;          // Main text
$text-secondary: #9AA2AB;        // Muted text, descriptions
$text-tertiary: #6B7280;         // Very muted

// Gradients
$gradient-primary: linear-gradient(135deg, #2EB67D 0%, #1E88E5 100%);
$gradient-background: linear-gradient(135deg, #101518 0%, #1E252B 100%);
```

### Typography
```scss
$font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
$font-mono: 'Monaco', 'SF Mono', monospace;

// Font Sizes
$font-size-tiny: 11px;    // Labels, metadata
$font-size-small: 12px;   // Secondary text
$font-size-sm: 13px;      // Compact UI elements
$font-size-base: 14px;    // Body text
$font-size-md: 16px;      // Section headers
$font-size-lg: 18px;      // Page titles
$font-size-xl: 24px;      // Main headers

// Font Weights
$font-weight-normal: 400;
$font-weight-medium: 500;
$font-weight-semibold: 600;
$font-weight-bold: 700;
```

### Spacing & Layout
```scss
// Spacing Scale
$spacing-xs: 4px;
$spacing-sm: 8px;
$spacing-md: 12px;
$spacing-lg: 16px;
$spacing-xl: 20px;
$spacing-2xl: 24px;
$spacing-3xl: 30px;

// Layout Dimensions
$sidebar-width: 340px;           // Left entity sidebar
$header-height: 56px;             // Top header
$input-height: 40px;              // Standard input height
$button-height: 36px;             // Standard button height
$avatar-size-sm: 40px;            // Small avatars
$avatar-size-md: 60px;            // Medium avatars
$avatar-size-lg: 80px;            // Large avatars

// Border Radius
$radius-sm: 4px;
$radius-md: 6px;
$radius-base: 8px;
$radius-lg: 12px;
$radius-xl: 20px;
$radius-full: 50%;
```

## Application Architecture

### Component Structure
```
src/
├── components/
│   ├── shell/
│   │   ├── AppShell.tsx           // Main application wrapper
│   │   ├── EntitySidebar.tsx      // Left sidebar with orgs/channels
│   │   ├── MainContent.tsx        // Center content area
│   │   └── InfoPanel.tsx          // Right info panel (optional)
│   │
│   ├── identity/
│   │   ├── IdentitySelector.tsx   // Top identity dropdown
│   │   ├── IdentityCreation.tsx   // Create new identity flow
│   │   └── FourWordDisplay.tsx    // Four-word address display
│   │
│   ├── organization/
│   │   ├── OrgDashboard.tsx       // Main org view (grid layout)
│   │   ├── OrgTree.tsx            // Expandable org tree
│   │   ├── MemberCard.tsx         // Member display component
│   │   └── ProjectCard.tsx        // Project display component
│   │
│   ├── communication/
│   │   ├── ChatInterface.tsx      // Main chat view
│   │   ├── MessageBubble.tsx      // Individual message
│   │   ├── ThreadView.tsx         // Thread discussion
│   │   └── VideoCall.tsx          // Video call interface
│   │
│   ├── storage/
│   │   ├── StorageOverview.tsx    // Storage management
│   │   ├── StorageMeter.tsx       // Visual progress bar
│   │   ├── FileBrowser.tsx        // File listing
│   │   └── VaultSettings.tsx      // Vault configuration
│   │
│   └── common/
│       ├── Button.tsx              // Styled buttons
│       ├── Input.tsx               // Styled inputs
│       ├── Avatar.tsx              // User avatars
│       ├── StatusIndicator.tsx    // Online/offline dots
│       └── LoadingSpinner.tsx     // Loading states
```

## Screen Implementations

### 1. Main Dashboard (Complete Shell)

**File:** `src/components/shell/AppShell.tsx`

```typescript
interface AppShellLayout {
  sidebar: {
    width: 340px;
    sections: {
      identity: IdentitySelector;      // Ocean Forest Moon Star
      filters: FilterChips[];          // All Spaces | Organizations | Personal
      search: CommandPalette;          // ⌘K search
      entityList: EntityTree[];        // Orgs with nested channels/projects
    };
  };
  mainContent: {
    header: EntityHeader;             // Org name, actions
    content: ReactNode;               // Dynamic based on selection
  };
  infoPanel?: InfoPanel;              // Optional right panel
}
```

**Key Features:**
- **Identity Selector:** Shows current four-word identity with avatar
- **Filter System:** Two-tier filtering (Space type + Entity type)
- **Entity Tree:** Organizations shown first, with expandable structure
- **Search:** Command palette with ⌘K shortcut
- **Main Content:** Dynamic based on selected entity

**State Management:**
```typescript
interface AppState {
  currentIdentity: Identity;
  selectedOrg: Organization | null;
  selectedEntity: Entity | null;
  filters: {
    spaceType: 'all' | 'organizations' | 'personal';
    entityType: 'all' | 'channels' | 'projects' | 'groups';
  };
  expandedOrgs: Set<string>;
  searchQuery: string;
}
```

### 2. Entity Sidebar Implementation

**File:** `src/components/shell/EntitySidebar.tsx`

```tsx
const EntitySidebar: React.FC = () => {
  return (
    <div className="entity-sidebar">
      {/* Identity Section */}
      <div className="identity-section">
        <IdentitySelector 
          identity={currentIdentity}
          onClick={openIdentitySwitcher}
        />
      </div>

      {/* Filter Chips */}
      <div className="filter-section">
        <div className="filter-row">
          <FilterChip active={true}>All Spaces</FilterChip>
          <FilterChip>Organizations</FilterChip>
          <FilterChip>Personal</FilterChip>
        </div>
        <div className="filter-row">
          <FilterChip active={true}>All Types</FilterChip>
          <FilterChip>Channels</FilterChip>
          <FilterChip>Projects</FilterChip>
          <FilterChip>Groups</FilterChip>
        </div>
      </div>

      {/* Search Bar */}
      <div className="search-section">
        <SearchInput 
          placeholder="🔍 Search or jump (⌘K)"
          onFocus={openCommandPalette}
        />
      </div>

      {/* Entity List */}
      <div className="entity-list">
        {organizations.map(org => (
          <OrgTreeNode 
            key={org.id}
            org={org}
            expanded={expandedOrgs.has(org.id)}
            onToggle={toggleOrgExpansion}
          >
            {/* Channels */}
            {org.channels.map(channel => (
              <EntityItem 
                key={channel.id}
                icon="#"
                name={channel.name}
                status={channel.updates ? 'Channel updates' : null}
                online={channel.hasActivity}
              />
            ))}
            
            {/* Projects */}
            {org.projects.map(project => (
              <EntityItem 
                key={project.id}
                icon="📁"
                name={project.name}
                status={`${project.status} · ${project.memberCount} members`}
                online={project.isActive}
              />
            ))}
            
            {/* Teams */}
            {org.teams.map(team => (
              <EntityItem 
                key={team.id}
                icon="👥"
                name={team.name}
                status={`${team.memberCount} members · ${team.adminCount} admins`}
                online={team.hasOnlineMembers}
              />
            ))}
          </OrgTreeNode>
        ))}
      </div>
    </div>
  );
};
```

### 3. Organization Dashboard Grid

**File:** `src/components/organization/OrgDashboard.tsx`

```tsx
const OrgDashboard: React.FC<{org: Organization}> = ({org}) => {
  return (
    <div className="org-dashboard">
      {/* Header */}
      <OrgHeader 
        org={org}
        actions={['📞 Call', '🎥 Video', '📁 Files', '🌐 Web', 'ℹ️ Info']}
      />

      {/* Content Grid - 2 columns */}
      <div className="dashboard-grid">
        {/* Members Section */}
        <DashboardCard title="Members" description="Hover to see details or manage participants.">
          {org.members.map(member => (
            <MemberCard 
              key={member.id}
              member={member}
              actions={['✏️', '✉️', '🗑️']}
              showStatus={true}
            />
          ))}
        </DashboardCard>

        {/* Projects Section */}
        <DashboardCard title="Projects" description="Hover to open project or archive.">
          {org.projects.map(project => (
            <ProjectCard 
              key={project.id}
              project={project}
              onOpen={openProject}
              onArchive={archiveProject}
            />
          ))}
        </DashboardCard>

        {/* Channels Section */}
        <DashboardCard title="Channels" description="Hover to preview members.">
          <div className="channel-grid">
            {org.channels.map(channel => (
              <ChannelTile 
                key={channel.id}
                channel={channel}
                onClick={openChannel}
              />
            ))}
          </div>
        </DashboardCard>

        {/* Storage Section */}
        <DashboardCard title="Storage" description="Manage encrypted vaults and virtual disks.">
          <StorageItem 
            icon="🗄️"
            name="Org Vault"
            description="End-to-end encrypted vault replicated across bootstrap nodes."
            used={420}
            total={1000}
            unit="GB"
            percentage={42}
            actions={[
              { label: 'Open', primary: true },
              { label: 'Manage', primary: false }
            ]}
          />
          
          <StorageItem 
            icon="☁️"
            name="Web Storage (Virtual Disk)"
            description="S3-compatible virtual disk for org web apps."
            used={340}
            total={500}
            unit="GB"
            percentage={68}
            actions={[
              { label: 'Open', primary: true },
              { label: 'Mount', primary: false }
            ]}
          />
        </DashboardCard>
      </div>
    </div>
  );
};
```

### 4. Storage Meter Component

**File:** `src/components/storage/StorageMeter.tsx`

```tsx
interface StorageMeterProps {
  used: number;
  total: number;
  unit: string;
  percentage: number;
  variant?: 'success' | 'warning' | 'danger';
}

const StorageMeter: React.FC<StorageMeterProps> = ({
  used, total, unit, percentage, variant = 'success'
}) => {
  const getColorClass = () => {
    if (percentage < 60) return 'storage-success';  // Green
    if (percentage < 80) return 'storage-warning';   // Yellow
    return 'storage-danger';                         // Red
  };

  return (
    <div className="storage-meter-container">
      <div className="storage-meter-info">
        <span className="storage-meter-label">
          {used} {unit} / {total} {unit}
        </span>
        <span className={`storage-meter-percentage ${getColorClass()}`}>
          {percentage}%
        </span>
      </div>
      <div className="storage-meter">
        <div 
          className={`storage-meter-fill ${getColorClass()}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
};
```

### 5. Member Card Component

**File:** `src/components/organization/MemberCard.tsx`

```tsx
interface MemberCardProps {
  member: {
    id: string;
    name: string;
    initials: string;
    role: 'Owner' | 'Admin' | 'Member' | 'Bot';
    department?: string;
    status: 'online' | 'away' | 'offline';
    fourWords?: string;
  };
  actions?: string[];
  showStatus?: boolean;
}

const MemberCard: React.FC<MemberCardProps> = ({
  member, actions = [], showStatus = true
}) => {
  const getAvatarGradient = () => {
    // Deterministic gradient based on member ID
    const gradients = [
      'linear-gradient(135deg, #2EB67D, #26A86B)',  // Green
      'linear-gradient(135deg, #1E88E5, #1976D2)',  // Blue
      'linear-gradient(135deg, #FF6B6B, #E55555)',  // Red
      'linear-gradient(135deg, #9AA2AB, #6B7280)',  // Gray
    ];
    return gradients[member.id.charCodeAt(0) % gradients.length];
  };

  return (
    <div className="member-card">
      <div 
        className="member-avatar"
        style={{ background: getAvatarGradient() }}
      >
        {member.initials}
      </div>
      
      <div className="member-info">
        <div className="member-name">{member.name}</div>
        <div className="member-meta">
          {member.role} 
          {member.department && ` · ${member.department}`}
          {showStatus && ` · ${member.status}`}
        </div>
      </div>
      
      {actions.length > 0 && (
        <div className="member-actions">
          {actions.map((action, idx) => (
            <button key={idx} className="member-action-btn">
              {action}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
```

## User Flows

### 1. Onboarding Flow

**Screens:**
1. **Welcome Screen** → Introduces core features (Post-quantum security, P2P, Four-words)
2. **Identity Creation** → Generate four-word address, set display name
3. **Network Bootstrap** → Connect to peers, find bootstrap nodes
4. **Enter Application** → Transition to main dashboard

**Implementation Notes:**
- Store identity in system keyring using ML-DSA encryption
- Bootstrap connection shows real-time status updates
- Animated transitions between onboarding steps

### 2. Organization Management Flow

**Key Interactions:**
1. **Create Organization** → Set name, generate org keys, invite initial members
2. **Manage Members** → Add/remove, set roles (Owner/Admin/Member)
3. **Create Channels/Projects** → Nested under organization
4. **Storage Management** → Configure vaults, set replication factor

### 3. Communication Flow

**Chat Features:**
- **Message Types:** Text, files, reactions, threads
- **Rich Previews:** File attachments show inline
- **Thread Support:** Branched discussions
- **Read Receipts:** Double checkmarks for read messages
- **Reactions:** Emoji reactions with counts

**Video Call Features:**
- **Grid Layout:** Active speaker prominent
- **Controls:** Mute, video, screenshare, end call
- **Participant List:** Show who's in call

### 4. Storage Flow

**Vault Features:**
- **Org Vault:** Encrypted, replicated across bootstrap nodes
- **Web Storage:** S3-compatible for web apps
- **File Browser:** Navigate folders, upload/download
- **Settings:** Replication factor, encryption type, sync settings

## State Management

### Redux Store Structure
```typescript
interface AppStore {
  auth: {
    currentIdentity: Identity;
    passkey?: PasskeyCredential;
    devices: Device[];
  };
  
  entities: {
    organizations: Map<string, Organization>;
    selectedOrgId: string | null;
    selectedEntityId: string | null;
    expandedOrgs: Set<string>;
  };
  
  communication: {
    messages: Map<string, Message[]>;  // channelId -> messages
    threads: Map<string, Thread[]>;    // messageId -> replies
    activeCall?: VideoCall;
    typing: Map<string, User[]>;       // channelId -> typing users
  };
  
  storage: {
    vaults: Vault[];
    currentPath: string;
    files: FileNode[];
    uploadQueue: Upload[];
  };
  
  ui: {
    theme: 'dark' | 'light';
    sidebarCollapsed: boolean;
    infoPanelOpen: boolean;
    commandPaletteOpen: boolean;
    filters: FilterState;
  };
}
```

## P2P Integration Points

### Network Layer
```typescript
interface P2PNetwork {
  // Identity & Auth
  createIdentity(): Promise<FourWordAddress>;
  authenticateWithPasskey(credential: PasskeyCredential): Promise<void>;
  
  // Organization Management
  createOrganization(name: string): Promise<Organization>;
  joinOrganization(fourWords: FourWordAddress): Promise<void>;
  
  // Messaging
  sendMessage(channelId: string, content: MessageContent): Promise<void>;
  subscribeToChannel(channelId: string, callback: MessageCallback): void;
  
  // Storage
  uploadToVault(file: File, vaultId: string): Promise<string>;
  replicateVault(vaultId: string, factor: number): Promise<void>;
  
  // Network
  connectToPeer(address: FourWordAddress): Promise<PeerConnection>;
  discoverPeers(): AsyncIterator<Peer>;
  getNetworkStatus(): NetworkStatus;
}
```

### CRDT Synchronization
- **Entities:** Organization structure uses CRDT for conflict-free updates
- **Messages:** Append-only log with vector clocks
- **Files:** Content-addressed storage with Merkle DAGs
- **Presence:** Last-write-wins register for online status

## Performance Considerations

### Optimizations
1. **Virtual Scrolling:** For large message/file lists
2. **Lazy Loading:** Load org content on-demand
3. **Message Pagination:** Load messages in chunks
4. **Image Optimization:** Lazy load avatars and previews
5. **WebWorker:** Run crypto operations off main thread

### Caching Strategy
```typescript
interface CacheLayer {
  // LRU cache for messages
  messageCache: LRUCache<string, Message[]>;
  
  // Persistent cache for org structure
  entityCache: IndexedDB<Entity>;
  
  // Memory cache for avatars
  avatarCache: Map<string, Blob>;
  
  // Service worker for offline support
  offlineCache: CacheStorage;
}
```

## Accessibility

### Requirements
- **Keyboard Navigation:** Full app navigable via keyboard
- **Screen Readers:** ARIA labels on all interactive elements
- **High Contrast:** Support system high contrast mode
- **Focus Indicators:** Clear focus states
- **Reduced Motion:** Respect prefers-reduced-motion

## Testing Strategy

### Component Tests
```typescript
// Example: MemberCard.test.tsx
describe('MemberCard', () => {
  it('shows online indicator for online members');
  it('displays correct role badge');
  it('shows action buttons on hover');
  it('handles click on member name');
});
```

### E2E Test Scenarios
1. **Onboarding:** Complete flow from welcome to dashboard
2. **Create Org:** Create org, invite members, create channel
3. **Send Message:** Send text, file, thread reply
4. **Storage:** Upload file, create folder, change settings

## Deployment Notes

### Environment Variables
```env
VITE_BOOTSTRAP_NODES=node1.communitas.network,node2.communitas.network
VITE_DEFAULT_REPLICATION=3
VITE_ENABLE_DEVTOOLS=false
VITE_PASSKEY_RPID=communitas.app
```

### Build Configuration
```typescript
// vite.config.ts
export default {
  build: {
    target: 'esnext',  // For top-level await
    minify: 'terser',
    sourcemap: true,
  },
  optimizeDeps: {
    include: ['@mlkem/mlkem', '@mldsa/mldsa'],  // Pre-bundle crypto
  }
};
```

## Next Steps

1. **Phase 1:** Implement core shell with mock data
2. **Phase 2:** Integrate P2P networking layer
3. **Phase 3:** Add storage and file management
4. **Phase 4:** Implement video/audio calls
5. **Phase 5:** Polish animations and transitions

## Component Library

We should use a combination of:
- **Radix UI:** For accessible primitives (Dialog, Dropdown, etc.)
- **Framer Motion:** For animations and transitions
- **React Query:** For data fetching and caching
- **Zustand or Redux Toolkit:** For state management

## References

- Interactive Prototype: `storyboard-canvas-v2.html`
- Design System: See color/typography sections above
- Architecture: `ARCHITECTURE.md`
- P2P Protocol: `communitas-core/README.md`
