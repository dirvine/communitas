# Member Management UI Gap Analysis

**Date:** 2025-01-24  
**Analyzed by:** Librarian Agent  
**Status:** CRITICAL GAP IDENTIFIED

## Executive Summary

**Backend:** ✅ Fully implemented CRDT-based member management with complete Tauri commands  
**Frontend:** ❌ Zero UI components for member management exposed to users  
**Integration:** 🟡 Service layer exists but disconnected from UI

**Impact:** Users cannot add/remove members, manage roles, or view member lists despite backend fully supporting these operations.

---

## Backend Capabilities (Verified ✅)

### Tauri Commands Available

**File:** [`communitas-desktop/src/member_commands.rs`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/communitas-desktop/src/member_commands.rs)

```rust
#[tauri::command]
pub async fn member_add(request: AddMemberRequest) -> Result<MemberResponse, String>

#[tauri::command]
pub async fn member_list(entity_id: String, entity_type: String) -> Result<MemberListResponse, String>

#[tauri::command]
pub async fn member_remove(request: RemoveMemberRequest) -> Result<MemberResponse, String>

#[tauri::command]
pub async fn member_update_role(request: UpdateMemberRoleRequest) -> Result<MemberResponse, String>

#[tauri::command]
pub async fn member_prune_tombstones(entity_id: String, entity_type: String, days: u64) -> Result<u64, String>
```

### Member Types

**File:** [`src/types/memberManagement.ts`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/types/memberManagement.ts)

```typescript
export type MemberRole = 'owner' | 'admin' | 'member' | 'guest'

export type MemberEntityType = 
  | 'organization'
  | 'group' 
  | 'channel'
  | 'project'
  | 'individual'

export interface MemberInfo {
  member_id: string
  entity_id: string
  entity_type: MemberEntityType
  role: MemberRole
  added_at: string
  added_by: string
  last_seen?: string
  is_active: boolean
  is_tombstone?: boolean
}
```

### Service Layer

**File:** [`src/services/MemberManagementService.ts`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/services/MemberManagementService.ts)

- ✅ `addMember()` - Calls `member_add` Tauri command
- ✅ `listMembers()` - Calls `member_list` Tauri command
- ✅ `removeMember()` - Calls `member_remove` Tauri command
- ✅ `updateMemberRole()` - Calls `member_update_role` Tauri command
- ✅ Helper methods: `getActiveMembers()`, `isMember()`, `getMemberCount()`

**Status:** Fully functional, just needs UI to call it.

---

## Frontend UI Gaps (Critical ❌)

### 1. No Member List Display Components

**Where it should be:**
- Organization member directory
- Group member roster  
- Channel participant list
- Project collaborator list

**Current state:** 
- `ChannelView.tsx` has "Add members" button but shows nothing
- `GroupChatInterface.tsx` has "Members" menu item that does nothing
- No standalone member list component exists

**Missing component:** `src/components/members/MemberListPanel.tsx`

---

### 2. No Add Member Dialog

**File:** [`src/components/organization/ChannelView.tsx`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/components/organization/ChannelView.tsx) (lines 288-292):

```typescript
<Tooltip title="Add members">
  <IconButton 
    size="small"
    onClick={() => {
      // TODO: Implement add members dialog
    }}
  >
    <PersonAddIcon fontSize="small" />
  </IconButton>
</Tooltip>
```

**Status:** Button exists, but onClick does nothing.

**Missing component:** `src/components/members/AddMemberDialog.tsx`

---

### 3. No Member Management Actions

**Current:** No way for users to:
- Remove members from entities
- Change member roles
- View member permissions
- Accept/reject member requests

**Missing component:** `src/components/members/MemberCard.tsx` with action menu

---

### 4. Integration Layer Issues

**File:** [`src/contexts/EntityDirectoryContext.tsx`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/contexts/EntityDirectoryContext.tsx) (lines 1655-1751)

**Problem:** Calls non-existent Tauri commands:

```typescript
// ❌ WRONG: These commands don't exist
await invoke('add_group_member', { groupId, userId, role })
await invoke('remove_group_member', { groupId, userId })

// ✅ CORRECT: Should call
await memberManagementService.addMember({
  entity_type: 'group',
  entity_id: groupId,
  member_id: userId,
  role,
  added_by: currentUserId
})
```

**Impact:** Any UI calling these context methods will fail.

---

### 5. Organization Service Mock Data

**File:** [`src/services/organization/OrganizationService.ts`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/services/organization/OrganizationService.ts) (lines 267-344)

**Problem:** Uses in-memory Map instead of backend:

```typescript
private organizationMembers = new Map<string, MemberInfo[]>()
private groupMembers = new Map<string, MemberInfo[]>()
private projectMembers = new Map<string, MemberInfo[]>()

async addMemberToOrganization(orgId: string, member: MemberInfo): Promise<void> {
  // ❌ Just updates local Map
  const members = this.organizationMembers.get(orgId) || []
  this.organizationMembers.set(orgId, [...members, member])
}
```

**Should be:**
```typescript
async addMemberToOrganization(orgId: string, memberId: string, role: MemberRole): Promise<void> {
  await memberManagementService.addMember({
    entity_type: 'organization',
    entity_id: orgId,
    member_id: memberId,
    role,
    added_by: this.currentUserId
  })
}
```

---

## Implementation Plan

### Phase 1: Core Components (Week 1)

#### Task 1.1: Build MemberListPanel Component
**File:** `src/components/members/MemberListPanel.tsx`

```typescript
import { MemberInfo, MemberRole } from '@/types/memberManagement'
import { MemberCard } from './MemberCard'
import { AddMemberDialog } from './AddMemberDialog'

interface MemberListPanelProps {
  entityType: MemberEntityType
  entityId: string
  currentUserId: string
  currentUserRole: MemberRole
}

export function MemberListPanel({ entityType, entityId, currentUserId, currentUserRole }: MemberListPanelProps) {
  const [members, setMembers] = useState<MemberInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [addDialogOpen, setAddDialogOpen] = useState(false)
  
  useEffect(() => {
    loadMembers()
  }, [entityId])
  
  const loadMembers = async () => {
    setLoading(true)
    try {
      const result = await memberManagementService.listMembers(entityId, entityType)
      if (result.success) {
        setMembers(result.members)
      }
    } finally {
      setLoading(false)
    }
  }
  
  const canManageMembers = currentUserRole === 'owner' || currentUserRole === 'admin'
  
  return (
    <Box>
      <Box display="flex" justifyContent="space-between" mb={2}>
        <Typography variant="h6">Members ({members.length})</Typography>
        {canManageMembers && (
          <Button
            startIcon={<PersonAdd />}
            onClick={() => setAddDialogOpen(true)}
          >
            Add Member
          </Button>
        )}
      </Box>
      
      {loading ? (
        <CircularProgress />
      ) : (
        <List>
          {members.map(member => (
            <MemberCard
              key={member.member_id}
              member={member}
              canManage={canManageMembers}
              onRemove={() => handleRemove(member.member_id)}
              onRoleChange={(newRole) => handleRoleChange(member.member_id, newRole)}
            />
          ))}
        </List>
      )}
      
      <AddMemberDialog
        open={addDialogOpen}
        onClose={() => setAddDialogOpen(false)}
        entityType={entityType}
        entityId={entityId}
        onMemberAdded={loadMembers}
      />
    </Box>
  )
}
```

**Tests:** Create `src/components/members/__tests__/MemberListPanel.test.tsx`

---

#### Task 1.2: Build AddMemberDialog Component
**File:** `src/components/members/AddMemberDialog.tsx`

```typescript
interface AddMemberDialogProps {
  open: boolean
  onClose: () => void
  entityType: MemberEntityType
  entityId: string
  onMemberAdded: () => void
}

export function AddMemberDialog({ open, onClose, entityType, entityId, onMemberAdded }: AddMemberDialogProps) {
  const [fourWords, setFourWords] = useState('')
  const [role, setRole] = useState<MemberRole>('member')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  
  const handleAdd = async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Validate four-word format
      if (!validateFourWords(fourWords)) {
        setError('Invalid four-word address format')
        return
      }
      
      // Call backend
      const result = await memberManagementService.addMember({
        entity_type: entityType,
        entity_id: entityId,
        member_id: fourWords,
        role,
        added_by: getCurrentUserId()
      })
      
      if (result.success) {
        onMemberAdded()
        onClose()
      } else {
        setError(result.error?.message || 'Failed to add member')
      }
    } catch (err) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }
  
  return (
    <Dialog open={open} onClose={onClose}>
      <DialogTitle>Add Member</DialogTitle>
      <DialogContent>
        <TextField
          label="Four-Word Address"
          placeholder="ocean-blue-eagle-star"
          value={fourWords}
          onChange={(e) => setFourWords(e.target.value)}
          fullWidth
          margin="normal"
          error={!!error}
          helperText={error}
        />
        
        <FormControl fullWidth margin="normal">
          <InputLabel>Role</InputLabel>
          <Select value={role} onChange={(e) => setRole(e.target.value as MemberRole)}>
            <MenuItem value="member">Member</MenuItem>
            <MenuItem value="admin">Admin</MenuItem>
            <MenuItem value="guest">Guest</MenuItem>
          </Select>
        </FormControl>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button onClick={handleAdd} disabled={loading} variant="contained">
          {loading ? <CircularProgress size={20} /> : 'Add'}
        </Button>
      </DialogActions>
    </Dialog>
  )
}
```

**Tests:** Create `src/components/members/__tests__/AddMemberDialog.test.tsx`

---

#### Task 1.3: Build MemberCard Component
**File:** `src/components/members/MemberCard.tsx`

```typescript
interface MemberCardProps {
  member: MemberInfo
  canManage: boolean
  onRemove?: (memberId: string) => void
  onRoleChange?: (memberId: string, newRole: MemberRole) => void
}

export function MemberCard({ member, canManage, onRemove, onRoleChange }: MemberCardProps) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null)
  
  const getRoleBadgeColor = (role: MemberRole) => {
    switch (role) {
      case 'owner': return 'error'
      case 'admin': return 'warning'
      case 'member': return 'primary'
      case 'guest': return 'default'
    }
  }
  
  return (
    <ListItem
      secondaryAction={canManage && (
        <IconButton onClick={(e) => setAnchorEl(e.currentTarget)}>
          <MoreVert />
        </IconButton>
      )}
    >
      <ListItemAvatar>
        <Avatar>
          <Badge
            color={member.is_active ? 'success' : 'default'}
            variant="dot"
            overlap="circular"
          >
            <Person />
          </Badge>
        </Avatar>
      </ListItemAvatar>
      
      <ListItemText
        primary={member.member_id}
        secondary={
          <Box display="flex" gap={1} alignItems="center">
            <Chip 
              label={member.role.toUpperCase()} 
              size="small" 
              color={getRoleBadgeColor(member.role)}
            />
            {member.last_seen && (
              <Typography variant="caption" color="textSecondary">
                Last seen: {formatRelativeTime(member.last_seen)}
              </Typography>
            )}
          </Box>
        }
      />
      
      {canManage && (
        <Menu
          anchorEl={anchorEl}
          open={Boolean(anchorEl)}
          onClose={() => setAnchorEl(null)}
        >
          <MenuItem onClick={() => {
            setAnchorEl(null)
            // Show role change dialog
          }}>
            Change Role
          </MenuItem>
          <MenuItem onClick={() => {
            setAnchorEl(null)
            if (onRemove) onRemove(member.member_id)
          }}>
            Remove Member
          </MenuItem>
        </Menu>
      )}
    </ListItem>
  )
}
```

---

### Phase 2: Integration (Week 2)

#### Task 2.1: Fix EntityDirectoryContext

**File:** [`src/contexts/EntityDirectoryContext.tsx`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/contexts/EntityDirectoryContext.tsx) (lines 1655-1751)

**Replace current implementation:**

```typescript
// ❌ CURRENT: Wrong commands
const addGroupMember = useCallback(async (groupId: string, userId: string, role: MemberRole = 'member'): Promise<void> => {
  await invoke('add_group_member', { groupId, userId, role }) // DOESN'T EXIST
}, [])

// ✅ CORRECT: Use MemberManagementService
const addGroupMember = useCallback(async (groupId: string, memberId: string, role: MemberRole = 'member'): Promise<void> => {
  const result = await memberManagementService.addMember({
    entity_type: 'group',
    entity_id: groupId,
    member_id: memberId,
    role,
    added_by: state.currentUserId || 'unknown'
  })
  
  if (!result.success) {
    throw new Error(result.error?.message || 'Failed to add member')
  }
  
  // Refresh entity to get updated member list
  await refreshEntity(groupId, 'group')
}, [state.currentUserId])
```

---

#### Task 2.2: Wire ChannelView Add Members Button

**File:** [`src/components/organization/ChannelView.tsx`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/components/organization/ChannelView.tsx) (line 288)

```typescript
// Add state
const [addMemberDialogOpen, setAddMemberDialogOpen] = useState(false)

// Update button
<Tooltip title="Add members">
  <IconButton 
    size="small"
    onClick={() => setAddMemberDialogOpen(true)}
  >
    <PersonAddIcon fontSize="small" />
  </IconButton>
</Tooltip>

// Add dialog at end of component
<AddMemberDialog
  open={addMemberDialogOpen}
  onClose={() => setAddMemberDialogOpen(false)}
  entityType="channel"
  entityId={channel.id}
  onMemberAdded={() => {
    // Refresh channel data
    loadChannelMembers()
  }}
/>
```

---

#### Task 2.3: Wire GroupChatInterface Invite Members

**File:** [`src/components/chat/GroupChatInterface.tsx`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/components/chat/GroupChatInterface.tsx) (line 577)

```typescript
<MenuItem onClick={() => {
  setGroupMenuAnchor(null)
  setAddMemberDialogOpen(true) // Add this state
}}>
  <PersonAdd sx={{ mr: 1 }} />
  Invite Members
</MenuItem>

{/* Add dialog */}
<AddMemberDialog
  open={addMemberDialogOpen}
  onClose={() => setAddMemberDialogOpen(false)}
  entityType="group"
  entityId={groupId}
  onMemberAdded={() => refreshGroupMembers()}
/>
```

---

### Phase 3: Advanced Features (Week 3)

#### Task 3.1: Add Members Tab to ModernShellPrototype

**File:** [`src/components/prototype/ModernShellPrototype.tsx`](file:///Users/davidirvine/Desktop/Devel/projects/communitas/src/components/prototype/ModernShellPrototype.tsx)

Add new tab to right drawer:

```typescript
const [rightDrawerTab, setRightDrawerTab] = useState<'activity' | 'members' | 'files'>('activity')

// In drawer content
{rightDrawerTab === 'members' && selectedEntity && (
  <MemberListPanel
    entityType={selectedEntity.type}
    entityId={selectedEntity.id}
    currentUserId={currentUser?.id || ''}
    currentUserRole={getCurrentUserRole(selectedEntity)}
  />
)}
```

---

#### Task 3.2: Permission-Based UI Visibility

**Create helper hook:** `src/hooks/useMemberPermissions.ts`

```typescript
export function useMemberPermissions(entityId: string, entityType: MemberEntityType) {
  const [permissions, setPermissions] = useState({
    canAddMembers: false,
    canRemoveMembers: false,
    canChangeRoles: false,
    canViewMembers: true
  })
  
  useEffect(() => {
    // Load current user's role in this entity
    memberManagementService.listMembers(entityId, entityType).then(result => {
      if (result.success) {
        const currentMember = result.members.find(m => m.member_id === currentUserId)
        if (currentMember) {
          setPermissions({
            canAddMembers: ['owner', 'admin'].includes(currentMember.role),
            canRemoveMembers: ['owner', 'admin'].includes(currentMember.role),
            canChangeRoles: currentMember.role === 'owner',
            canViewMembers: true
          })
        }
      }
    })
  }, [entityId, entityType])
  
  return permissions
}
```

**Usage:**
```typescript
const permissions = useMemberPermissions(channelId, 'channel')

{permissions.canAddMembers && (
  <Button onClick={() => setAddMemberDialogOpen(true)}>
    Add Member
  </Button>
)}
```

---

## Testing Requirements

### Unit Tests (Vitest)

1. **`MemberListPanel.test.tsx`**
   - Renders member list correctly
   - Shows Add Member button for admins only
   - Handles loading states
   - Handles empty member list

2. **`AddMemberDialog.test.tsx`**
   - Validates four-word input
   - Calls memberManagementService.addMember()
   - Shows error messages
   - Disables submit while loading

3. **`MemberCard.test.tsx`**
   - Displays member info correctly
   - Shows role badge with correct color
   - Shows action menu for admins only
   - Online/offline badge displays

### Integration Tests (Playwright)

**File:** `tests/member-management.spec.ts`

```typescript
test('admin can add member to channel', async ({ page }) => {
  // Navigate to channel
  await page.click('[data-testid="channel-ocean-blue"]')
  
  // Click Add Member button
  await page.click('[data-testid="add-member-button"]')
  
  // Fill four-word address
  await page.fill('[data-testid="member-four-words"]', 'apple-banana-cherry-date')
  
  // Select role
  await page.selectOption('[data-testid="member-role"]', 'member')
  
  // Submit
  await page.click('[data-testid="add-member-submit"]')
  
  // Verify member appears in list
  await expect(page.locator('text=apple-banana-cherry-date')).toBeVisible()
})

test('non-admin cannot see Add Member button', async ({ page }) => {
  // Login as regular member
  await loginAsMember(page)
  
  // Navigate to channel
  await page.click('[data-testid="channel-ocean-blue"]')
  
  // Add Member button should not exist
  await expect(page.locator('[data-testid="add-member-button"]')).not.toBeVisible()
})
```

---

## File Structure

### New Files to Create

```
src/components/members/
├── MemberListPanel.tsx          # Main member list component
├── MemberCard.tsx               # Individual member display
├── AddMemberDialog.tsx          # Add member dialog
├── RemoveMemberDialog.tsx       # Confirmation dialog
├── MemberRoleEditor.tsx         # Role selection component
├── MemberSearchFilter.tsx       # Search and filter UI
└── __tests__/
    ├── MemberListPanel.test.tsx
    ├── MemberCard.test.tsx
    └── AddMemberDialog.test.tsx

src/hooks/
└── useMemberPermissions.ts      # Permission checking hook

tests/
└── member-management.spec.ts    # E2E Playwright tests
```

---

## Migration Checklist

### Week 1: Core Components
- [ ] Create `src/components/members/` directory
- [ ] Build `MemberListPanel.tsx`
- [ ] Build `AddMemberDialog.tsx`
- [ ] Build `MemberCard.tsx`
- [ ] Write unit tests (3 test files)
- [ ] Fix `EntityDirectoryContext` command names
- [ ] Fix `OrganizationService` to use backend instead of Map

### Week 2: Integration
- [ ] Wire `ChannelView` Add Members button
- [ ] Wire `GroupChatInterface` Invite Members
- [ ] Add Members tab to `ModernShellPrototype`
- [ ] Create `useMemberPermissions` hook
- [ ] Add permission checks throughout UI

### Week 3: Polish
- [ ] Add member search/filter
- [ ] Add online/offline status indicators
- [ ] Add invitation link generation
- [ ] Write Playwright E2E tests
- [ ] Add member management to Storybook

---

## Success Criteria

### Definition of Done

- [ ] Users can add members via UI (four-word address + role)
- [ ] Users can view member lists with roles and status
- [ ] Admins can remove members with confirmation
- [ ] Owners can change member roles
- [ ] Non-admins cannot access management features
- [ ] All operations use actual backend (no mock data)
- [ ] 100% test coverage for member components
- [ ] E2E tests verify full workflows
- [ ] Works across all entity types (org, group, channel, project)

---

## Risk Mitigation

### Technical Risks

1. **CRDT Sync Delays**: Member additions may not appear immediately
   - **Mitigation:** Optimistic updates + CRDT reconciliation
   - **UI:** Show "Adding..." state, then sync indicator

2. **Four-Word Resolution**: Member IDs must be valid and reachable
   - **Mitigation:** Validate before submission, show resolution errors
   - **UI:** Real-time validation, suggestion list from contacts

3. **Permission Conflicts**: User permissions may change during operation
   - **Mitigation:** Re-check permissions before destructive actions
   - **UI:** Refresh permissions periodically

### UX Risks

1. **Discoverability**: Users may not find member management features
   - **Mitigation:** Add prominent "Members" tab/section
   - **UI:** Contextual hints, onboarding tooltips

2. **Bulk Operations**: Adding many members one-by-one is tedious
   - **Future:** Batch add, CSV import (not in current scope)

---

## Related Documentation

- [AGENTS_API.md](docs/AGENTS_API.md) - Tauri commands reference
- [memberManagement.ts](src/types/memberManagement.ts) - Type definitions
- [MemberManagementService.ts](src/services/MemberManagementService.ts) - Service layer
- [member_commands.rs](communitas-desktop/src/member_commands.rs) - Backend implementation

---

**Next Steps:** Begin Phase 1 implementation with TDD approach (write component tests first, then implement components to pass tests).
