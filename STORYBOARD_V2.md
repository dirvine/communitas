# Communitas V2 - Implementation Specification

## Quick Start Implementation Checklist

Based on `storyboard-canvas-v2.html` prototype. This is your implementation roadmap.

## 🎯 Priority 1: Core Shell (Week 1)

### Day 1-2: Base Layout
```tsx
// src/App.tsx
<AppShell>
  <EntitySidebar width={340} />     // Left panel
  <MainContent flex={1} />          // Center area  
  <InfoPanel width={320} />         // Right panel (optional)
</AppShell>
```

**CSS Variables to Set:**
```css
:root {
  --bg-primary: #161C20;
  --bg-secondary: #1a1f24;
  --bg-tertiary: #101518;
  --border: #2a3038;
  --accent: #2EB67D;
  --text-primary: #F4F6F8;
  --text-secondary: #9AA2AB;
  --sidebar-width: 340px;
}
```

### Day 3-4: Entity Sidebar Components

**1. Identity Selector Component**
```tsx
// Top of sidebar - the green box with "Ocean Forest Moon Star"
<IdentitySelector>
  <Avatar initials="OC" gradient={true} />
  <div>
    <Name>Ocean Forest Moon Star</Name>
    <FourWords>ocean-forest-moon-star</FourWords>
  </div>
</IdentitySelector>
```

**2. Filter Chips**
```tsx
// Two rows of filter buttons
<FilterRow>
  <FilterChip active>All Spaces</FilterChip>
  <FilterChip>Organizations</FilterChip>  
  <FilterChip>Personal</FilterChip>
</FilterRow>
<FilterRow>
  <FilterChip active>All Types</FilterChip>
  <FilterChip>Channels</FilterChip>
  <FilterChip>Projects</FilterChip>
  <FilterChip>Groups</FilterChip>
</FilterRow>
```

**3. Organization Tree**
```tsx
// The expandable org structure
<OrgTree>
  <OrgHeader expanded={true} onlineCount={3}>
    <Icon>🏢</Icon>
    <Name>ACME CORPORATION</Name>
  </OrgHeader>
  <OrgContent>
    <EntityItem type="channel" name="#general" status="Channel updates" />
    <EntityItem type="channel" name="#engineering" status="Channel updates" />
    <EntityItem type="project" name="Website Redesign" status="Active · 3 members" />
    <EntityItem type="team" name="Development Team" status="2 members · 1 admin" />
  </OrgContent>
</OrgTree>
```

### Day 5: Organization Dashboard Grid

**Main Content Area Structure:**
```tsx
<OrgDashboard>
  <Header>
    <OrgInfo>
      <Avatar>Ac</Avatar>
      <div>
        <Title>Acme Corporation</Title>
        <Subtitle>Organisation overview</Subtitle>
      </div>
    </OrgInfo>
    <Actions>
      <IconButton>📞</IconButton>
      <IconButton>🎥</IconButton>
      <IconButton>📁</IconButton>
      <IconButton>🌐</IconButton>
      <IconButton>ℹ️</IconButton>
    </Actions>
  </Header>
  
  <Grid columns={2} gap={20}>
    <Card title="Members" />
    <Card title="Projects" />
    <Card title="Channels" />
    <Card title="Storage" />
  </Grid>
</OrgDashboard>
```

## 🎯 Priority 2: Interactive Components (Week 2)

### Storage Meter Component
```tsx
interface StorageMeterProps {
  icon: string;        // "🗄️" or "☁️"
  title: string;       // "Org Vault"
  description: string; // "End-to-end encrypted..."
  used: number;        // 420
  total: number;       // 1000
  unit: string;        // "GB"
}

<StorageCard>
  <Header>
    <Icon>{icon}</Icon>
    <Info>
      <Title>{title}</Title>
      <Description>{description}</Description>
    </Info>
  </Header>
  <Stats>
    <Label>{used} {unit} / {total} {unit}</Label>
    <Percentage color={getColor(percent)}>{percent}%</Percentage>
  </Stats>
  <ProgressBar percent={percent} />
  <Actions>
    <Button primary>Open</Button>
    <Button>Manage</Button>
  </Actions>
</StorageCard>
```

### Member Card Component
```tsx
<MemberCard>
  <Avatar gradient="green">DA</Avatar>
  <Info>
    <Name>David Allan</Name>
    <Role>Owner · Online</Role>
  </Info>
  <Actions>
    <IconButton>✏️</IconButton>
    <IconButton>✉️</IconButton>
    <IconButton>🗑️</IconButton>
  </Actions>
</MemberCard>
```

## 🎯 Priority 3: Communication Features (Week 3)

### Chat Interface
```tsx
<ChatInterface>
  <MessageList>
    <Message author="Alice" time="2:30 PM">
      <Text>Hey team, design docs updated!</Text>
      <Attachment>📄 design-v2.md</Attachment>
    </Message>
    <Message author="Bob" time="2:45 PM">
      <Text>Looking good! Few notes:</Text>
      <List>
        • Consider offline-first
        • Need CRDT strategy
      </List>
      <Reactions>
        <Reaction emoji="👍" count={2} />
        <Reaction emoji="💭" count={1} />
      </Reactions>
    </Message>
    <Message own={true}>
      <Text>I'll review the CRDTs</Text>
      <ReadReceipt>✓✓ Read</ReadReceipt>
    </Message>
  </MessageList>
  
  <InputBar>
    <Input placeholder="Type a message..." />
    <SendButton>Send</SendButton>
  </InputBar>
</ChatInterface>
```

## 🎯 Component Specifications

### 1. Color Usage Rules
```scss
// Status Colors
Online: #2EB67D (green dot)
Away: #F5B759 (yellow dot)  
Offline: #9AA2AB (gray/empty circle)

// Storage Thresholds
0-60%: #2EB67D (green)
60-80%: #F5B759 (yellow)
80-100%: #E25555 (red)

// Avatar Gradients (deterministic by user)
User 1: linear-gradient(135deg, #2EB67D, #26A86B)
User 2: linear-gradient(135deg, #1E88E5, #1976D2)
User 3: linear-gradient(135deg, #FF6B6B, #E55555)
Bots: linear-gradient(135deg, #9AA2AB, #6B7280)
```

### 2. Typography Rules
```css
/* Entity Names */
.org-name { 
  font-size: 13px; 
  font-weight: 600; 
  text-transform: uppercase;
}

.channel-name, .project-name { 
  font-size: 12px; 
  font-weight: 400;
}

/* Status Text */
.status-text { 
  font-size: 11px; 
  color: #9AA2AB;
}

/* Card Headers */
.card-title { 
  font-size: 14px; 
  font-weight: 600;
}

.card-description { 
  font-size: 12px; 
  color: #9AA2AB;
}
```

### 3. Spacing Standards
```scss
// Component Padding
.card { padding: 16px; }
.sidebar-section { padding: 12px; }
.entity-item { padding: 6px 8px; }

// Gaps
.filter-row { gap: 6px; }
.card-grid { gap: 20px; }
.member-list { gap: 8px; }

// Margins
.section + .section { margin-top: 12px; }
```

### 4. Interactive States
```scss
// Hover Effects
.entity-item:hover {
  background: rgba(46, 182, 125, 0.1);
  transform: translateX(5px);
}

.filter-chip:hover {
  background: rgba(46, 182, 125, 0.25);
  transform: translateY(-2px);
}

.card:hover {
  border-color: rgba(46, 182, 125, 0.3);
}

// Active States  
.filter-chip.active {
  background: #2EB67D;
  color: #101518;
}

.entity-item.selected {
  background: rgba(46, 182, 125, 0.1);
}
```

## 🎯 Data Models

### Identity
```typescript
interface Identity {
  fourWords: string;          // "ocean-forest-moon-star"
  displayName: string;        // "David"
  initials: string;          // "OC"
  publicKey: string;         // ML-DSA public key
  devices: Device[];
  createdAt: Date;
}
```

### Organization
```typescript
interface Organization {
  id: string;
  name: string;              // "ACME CORPORATION"
  avatar?: string;           // URL or gradient definition
  members: Member[];
  channels: Channel[];
  projects: Project[];
  teams: Team[];
  storage: {
    vault: StorageVault;
    webStorage: StorageVault;
  };
  onlineCount: number;
}
```

### Member
```typescript
interface Member {
  id: string;
  identity: Identity;
  role: 'Owner' | 'Admin' | 'Member' | 'Bot';
  department?: string;       // "Engineering", "Product"
  status: 'online' | 'away' | 'offline';
  joinedAt: Date;
}
```

### Storage
```typescript
interface StorageVault {
  id: string;
  type: 'org-vault' | 'web-storage';
  name: string;
  description: string;
  used: number;              // in bytes
  total: number;             // in bytes
  replicationFactor: number;
  encryption: 'ML-KEM' | 'AES-256';
  files: FileNode[];
}
```

## 🎯 State Management

### Zustand Store Example
```typescript
// src/stores/useAppStore.ts
interface AppStore {
  // Identity
  currentIdentity: Identity | null;
  setIdentity: (identity: Identity) => void;
  
  // Organization
  organizations: Organization[];
  selectedOrgId: string | null;
  selectOrg: (id: string) => void;
  expandedOrgs: Set<string>;
  toggleOrgExpansion: (id: string) => void;
  
  // Filters
  filters: {
    spaceType: 'all' | 'organizations' | 'personal';
    entityType: 'all' | 'channels' | 'projects' | 'groups';
  };
  setFilter: (type: string, value: string) => void;
  
  // UI State
  sidebarCollapsed: boolean;
  commandPaletteOpen: boolean;
  infoPanelOpen: boolean;
}

const useAppStore = create<AppStore>((set) => ({
  currentIdentity: null,
  organizations: [],
  selectedOrgId: null,
  expandedOrgs: new Set(['acme-corp']), // Default expanded
  filters: {
    spaceType: 'all',
    entityType: 'all'
  },
  // ... actions
}));
```

## 🎯 Implementation Order

### Phase 1: Static UI (Days 1-5)
1. ✅ Create layout shell with three panels
2. ✅ Build EntitySidebar with static data
3. ✅ Create OrgDashboard grid layout
4. ✅ Style all components to match design
5. ✅ Add hover states and transitions

### Phase 2: Interactivity (Days 6-10)
1. ⬜ Wire up Zustand store
2. ⬜ Implement filter functionality
3. ⬜ Add org expansion/collapse
4. ⬜ Create command palette (⌘K)
5. ⬜ Add entity selection

### Phase 3: Mock Data (Days 11-15)
1. ⬜ Create mock data generators
2. ⬜ Implement chat interface
3. ⬜ Add file browser
4. ⬜ Create member management
5. ⬜ Build storage meters

### Phase 4: P2P Integration (Days 16-20)
1. ⬜ Connect to P2P network
2. ⬜ Implement identity creation
3. ⬜ Wire up real org data
4. ⬜ Enable message sending
5. ⬜ Test network features

## 🎯 CSS Classes to Create

```scss
// Layout
.app-shell { display: flex; height: 100vh; }
.entity-sidebar { width: 340px; background: #1a1f24; }
.main-content { flex: 1; background: #161C20; }

// Components
.identity-selector { padding: 12px; border-bottom: 1px solid #2a3038; }
.filter-chips { display: flex; gap: 6px; padding: 12px; }
.entity-tree { padding: 0 12px; }
.org-header { padding: 8px; background: rgba(46, 182, 125, 0.1); }
.entity-item { padding: 6px 8px; margin-left: 20px; }

// Cards
.dashboard-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
.dashboard-card { background: #1a1f24; border-radius: 8px; padding: 16px; }
.member-card { display: flex; align-items: center; gap: 12px; }
.storage-meter { height: 8px; background: #1F262C; border-radius: 4px; }
.storage-meter-fill { height: 100%; background: #2EB67D; transition: width 0.5s; }
```

## 🎯 Quick Copy-Paste Components

### Avatar Component
```tsx
const Avatar = ({ initials, size = 40, gradient = true }) => (
  <div
    style={{
      width: size,
      height: size,
      borderRadius: '50%',
      background: gradient 
        ? `linear-gradient(135deg, #2EB67D, #1E88E5)`
        : '#2EB67D',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: 'white',
      fontWeight: 'bold',
      fontSize: size * 0.4
    }}
  >
    {initials}
  </div>
);
```

### Status Dot Component
```tsx
const StatusDot = ({ status }) => (
  <span
    style={{
      width: 6,
      height: 6,
      borderRadius: '50%',
      backgroundColor: status === 'online' ? '#2EB67D' : 
                      status === 'away' ? '#F5B759' : '#9AA2AB',
      display: 'inline-block'
    }}
  />
);
```

### Filter Chip Component
```tsx
const FilterChip = ({ active, children, onClick }) => (
  <button
    onClick={onClick}
    style={{
      padding: '6px 12px',
      background: active 
        ? '#2EB67D' 
        : 'transparent',
      color: active 
        ? '#101518' 
        : '#9AA2AB',
      border: `1px solid ${active ? '#2EB67D' : '#2a3038'}`,
      borderRadius: 6,
      fontSize: 12,
      cursor: 'pointer',
      transition: 'all 0.3s'
    }}
  >
    {children}
  </button>
);
```

## 🎯 Testing Each Component

```typescript
// Quick test for each component
describe('Storyboard Components', () => {
  test('EntitySidebar renders orgs', () => {
    expect(screen.getByText('ACME CORPORATION')).toBeInTheDocument();
  });
  
  test('Filters toggle correctly', () => {
    const orgFilter = screen.getByText('Organizations');
    fireEvent.click(orgFilter);
    expect(orgFilter).toHaveClass('active');
  });
  
  test('Storage meter shows percentage', () => {
    const meter = screen.getByText('42%');
    expect(meter).toHaveStyle({ color: '#2EB67D' });
  });
  
  test('Member card shows online status', () => {
    const member = screen.getByText('David Allan');
    const status = member.parentElement.querySelector('.status-dot');
    expect(status).toHaveStyle({ background: '#2EB67D' });
  });
});
```

## 🚀 Start Here Tomorrow

1. Copy the HTML from `storyboard-canvas-v2.html`
2. Convert each section to a React component
3. Use the exact colors and spacing from this guide
4. Test each component matches the prototype
5. Once UI is perfect, add interactivity

Remember: **The UI is already designed** in the HTML file. Your job is to make React components that look EXACTLY like it.
