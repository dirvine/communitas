# Frontend Migration Guide: Stub → Working Commands

**Date**: 2025-10-12 • **Status**: CRITICAL - Frontend calling many stub commands ⚠️

This document identifies all frontend code calling stub backend commands and provides migration paths to working implementations.

---

## 🚨 CRITICAL ISSUE

**Problem**: Frontend code is calling 15 different stub commands across 11 files. All these calls return `"Not yet implemented"` errors from the backend.

**Impact**: Features appear broken or non-functional to users.

**Solution**: Replace stub commands with working equivalents from WORKING_COMMANDS.md.

---

## 📊 FRONTEND STUB USAGE SUMMARY

**Total Files Affected**: 11
**Total Stub Command Calls**: 21+ instances
**Stub Commands Used**: 15 different commands

### Files with Stub Command Calls

1. **MessagesPanel.tsx** - 3 stub calls
2. **EntityDirectoryContext.tsx** - 1 stub call
3. **TouchContainer.tsx** - 5 stub calls
4. **useDHTSync.ts** - 2 stub calls
5. **BackendService.ts** - 1 stub call
6. **OfflineStorageService.ts** - 4 stub calls
7. **ModernShellPrototype.tsx** - 5 stub calls
8. **messagingSubscription.ts** - 1 stub call

---

## 🔧 FILE-BY-FILE MIGRATION

### 1. MessagesPanel.tsx

**Location**: `src/components/entity/MessagesPanel.tsx`

#### Issue 1: Line 138 - core_messages_send (STUB)

```typescript
// ❌ BEFORE (stub command):
const message = await invoke('core_messages_send', {
  channelId,
  content: newMessage
});
```

```typescript
// ✅ AFTER (working command):
const message = await invoke('send_message', {
  request: {
    channel_id: channelId,
    author_id: currentUserId, // Add current user ID
    content: newMessage,
    thread_id: null  // null for main channel, or threadId for replies
  }
});
```

**Changes Required**:
- Import: Replace `core_messages_send` with `send_message`
- Add `author_id` parameter (get from auth context)
- Wrap parameters in `request` object
- Add optional `thread_id` parameter

#### Issue 2: Line 160 - core_messages_edit (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_messages_edit', {
  messageId,
  newContent
});
```

**Status**: ❌ **NO WORKING EQUIVALENT**

**Options**:
1. **Implement in backend**: Add `edit_message` to `org_commands.rs`
2. **Workaround**: Delete old message + send new message (preserves history)
3. **Disable feature**: Remove edit button until backend implements

**Recommendation**: Option 1 (implement in backend with CRDT support)

#### Issue 3: Line 185 - core_messages_delete (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_messages_delete', {
  messageId
});
```

**Status**: ❌ **NO WORKING EQUIVALENT**

**Options**:
1. **Implement in backend**: Add `delete_message` to `org_commands.rs`
2. **Workaround**: Mark message as deleted (soft delete)
3. **Disable feature**: Remove delete button until backend implements

**Recommendation**: Option 1 (implement with tombstone pattern for CRDT)

---

### 2. EntityDirectoryContext.tsx

**Location**: `src/contexts/EntityDirectoryContext.tsx`

#### Issue: Line 1722 - core_send_message_to_channel (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_send_message_to_channel', {
  channelId: entity.id,
  content: message
});
```

```typescript
// ✅ AFTER (working command):
await invoke('send_message', {
  request: {
    channel_id: entity.id,
    author_id: currentUserId, // Get from auth context
    content: message,
    thread_id: null
  }
});
```

---

### 3. TouchContainer.tsx

**Location**: `src/components/ui/TouchContainer.tsx`

#### Issue 1: Line 335 - core_send_message_to_channel (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_send_message_to_channel', {
  channelId: currentEntity.id,
  content: text
});
```

```typescript
// ✅ AFTER (working command):
await invoke('send_message', {
  request: {
    channel_id: currentEntity.id,
    author_id: userId,
    content: text,
    thread_id: null
  }
});
```

#### Issue 2: Line 351 - core_send_message_to_recipients (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_send_message_to_recipients', {
  recipients: recipientIds,
  content: message
});
```

```typescript
// ✅ AFTER (working command):
await invoke('gossip_send_direct_message', {
  recipient_four_words: recipientFourWords, // Convert ID to four-words
  content: message,
  encrypted: true
});
```

**Changes Required**:
- Use `gossip_send_direct_message` for direct messaging
- Convert recipient IDs to four-word addresses
- Set `encrypted: true` for private messages

**Note**: If sending to multiple recipients, loop over recipients:
```typescript
for (const recipientFourWords of recipientAddresses) {
  await invoke('gossip_send_direct_message', {
    recipient_four_words: recipientFourWords,
    content: message,
    encrypted: true
  });
}
```

#### Issue 3: Line 358 - core_send_message_to_channel (STUB)

Same fix as Issue 1 above.

#### Issue 4: Line 373 - core_subscribe_messages (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_subscribe_messages', {
  channelId: currentEntity.id
});
```

```typescript
// ✅ AFTER (working command):
await invoke('gossip_subscribe_to_entity', {
  entity_id: currentEntity.id,
  entity_type: 'channel' // or 'project', 'group'
});
```

---

### 4. useDHTSync.ts

**Location**: `src/hooks/useDHTSync.ts`

#### Issue 1: Line 109 - subscribe_to_entity (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('subscribe_to_entity', {
  entityId
});
```

```typescript
// ✅ AFTER (working command):
await invoke('gossip_subscribe_to_entity', {
  entity_id: entityId,
  entity_type: entityType // 'channel' | 'project' | 'group'
});
```

**Changes Required**:
- Add `entity_type` parameter (determine from entity data)
- Prefix with `gossip_` for gossip overlay pub/sub

#### Issue 2: Line 140 - unsubscribe_from_entity (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('unsubscribe_from_entity', {
  entityId
});
```

**Status**: ⚠️ **NO EXPLICIT UNSUBSCRIBE COMMAND**

**Workaround**:
- Gossip overlay automatically unsubscribes on disconnect
- Call `gossip_leave_entity` for explicit unsubscribe:

```typescript
// ✅ WORKAROUND:
await invoke('gossip_leave_entity', {
  entity_id: entityId
});
```

---

### 5. BackendService.ts

**Location**: `src/services/api/BackendService.ts`

#### Issue: Line 195 - core_send_message_to_channel (STUB)

```typescript
// ❌ BEFORE (stub command):
return await invoke('core_send_message_to_channel', {
  channelId,
  content
});
```

```typescript
// ✅ AFTER (working command):
return await invoke('send_message', {
  request: {
    channel_id: channelId,
    author_id: this.currentUserId, // Get from service state
    content,
    thread_id: null
  }
});
```

---

### 6. OfflineStorageService.ts

**Location**: `src/services/storage/OfflineStorageService.ts`

#### Issue 1-3: Lines 181, 233, 564 - core_private_put (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_private_put', {
  key: storageKey,
  value: encryptedData
});
```

```typescript
// ✅ AFTER (working command):
await invoke('gossip_store_message', {
  key: storageKey,
  value: encryptedData
});
```

**Changes Required**:
- Use `gossip_store_message` for CRDT-backed key-value storage
- Same signature, different command name

#### Issue 4: Line 400 - core_private_get (STUB)

```typescript
// ❌ BEFORE (stub command):
const encrypted = await invoke('core_private_get', {
  key
}) as number[] | null;
```

```typescript
// ✅ AFTER (working command):
// Get all messages, then filter by key
const allMessages = await invoke('gossip_get_all_messages') as Array<{
  key: string;
  value: number[];
}>;
const encrypted = allMessages.find(m => m.key === key)?.value ?? null;
```

**Changes Required**:
- `gossip_get_all_messages` returns all key-value pairs
- Filter client-side for specific key
- Consider caching results for performance

**Alternative** (if performance is issue):
- Implement `gossip_get_message` in backend for single key retrieval

---

### 7. ModernShellPrototype.tsx

**Location**: `src/components/prototype/ModernShellPrototype.tsx`

#### Issue 1: Line 1025 - core_get_user_info (STUB)

```typescript
// ❌ BEFORE (stub command):
const userInfo = await (window as any).__TAURI__.tauri.invoke('core_get_user_info') as {
  peerId: string;
  displayName: string
}
```

```typescript
// ✅ AFTER (working command):
const identity = await invoke('gossip_get_own_identity') as {
  four_words: string;
  display_name: string;
  public_key: string;
};
```

**Changes Required**:
- Use `gossip_get_own_identity` instead
- Update type definitions (different structure)
- `four_words` replaces `peerId`

#### Issue 2: Line 1184 - core_add_bootstrap_node (STUB)

```typescript
// ❌ BEFORE (stub command):
await (window as any).__TAURI__.tauri.invoke('core_add_bootstrap_node', {
  node: bootstrapAddress
});
```

```typescript
// ✅ AFTER (working command):
await invoke('gossip_add_bootstrap_peer', {
  multiaddr: bootstrapAddress
});
```

#### Issue 3: Line 1208 - core_set_display_name (STUB)

```typescript
// ❌ BEFORE (stub command):
await (window as any).__TAURI__.tauri.invoke('core_set_display_name', {
  displayName: newName
});
```

**Status**: ❌ **NO WORKING EQUIVALENT**

**Workaround**:
- Display names are set during initialization (`core_initialize` or `gossip_initialize`)
- To change: re-initialize with new name (loses session)
- **Recommendation**: Implement `update_profile` command in backend

#### Issue 4: Line 1905 - core_entity_update (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_entity_update', {
  entityId,
  updates: { name: newName, description }
});
```

**Status**: ⚠️ **DEPENDS ON ENTITY TYPE**

**Solution by Entity Type**:

```typescript
// For channels - NO UPDATE COMMAND YET
// Recommendation: Implement update_channel in org_commands.rs

// For projects - NO UPDATE COMMAND YET
// Recommendation: Implement update_project in org_commands.rs

// For issues - USE update_issue_status, update_issue_priority
await invoke('update_issue_status', {
  issue_id: entityId,
  new_status: 'in-progress'
});
```

#### Issue 5: Line 1933 - core_entity_delete (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_entity_delete', {
  entityId
});
```

**Status**: ❌ **NO WORKING EQUIVALENT**

**Recommendation**: Implement delete commands in org_commands.rs:
- `delete_channel(channel_id)`
- `delete_project(project_id)`
- Issue deletion is already implemented: use `update_issue_status` with `'canceled'`

---

### 8. messagingSubscription.ts

**Location**: `src/services/messagingSubscription.ts`

#### Issue: Line 22 - core_subscribe_messages (STUB)

```typescript
// ❌ BEFORE (stub command):
await invoke('core_subscribe_messages', {
  channelId: opts?.channelId
})
```

```typescript
// ✅ AFTER (working command):
await invoke('gossip_subscribe_to_entity', {
  entity_id: opts?.channelId,
  entity_type: 'channel'
});
```

---

## 📋 MIGRATION CHECKLIST

### Phase 1: Immediate Fixes (Working Equivalents)

- [ ] **MessagesPanel.tsx** - Replace `core_messages_send` with `send_message`
- [ ] **EntityDirectoryContext.tsx** - Replace `core_send_message_to_channel` with `send_message`
- [ ] **TouchContainer.tsx** (3 fixes)
  - [ ] Replace `core_send_message_to_channel` with `send_message`
  - [ ] Replace `core_send_message_to_recipients` with `gossip_send_direct_message`
  - [ ] Replace `core_subscribe_messages` with `gossip_subscribe_to_entity`
- [ ] **useDHTSync.ts** (2 fixes)
  - [ ] Replace `subscribe_to_entity` with `gossip_subscribe_to_entity`
  - [ ] Replace `unsubscribe_from_entity` with `gossip_leave_entity`
- [ ] **BackendService.ts** - Replace `core_send_message_to_channel` with `send_message`
- [ ] **OfflineStorageService.ts** (2 fixes)
  - [ ] Replace `core_private_put` with `gossip_store_message`
  - [ ] Replace `core_private_get` with `gossip_get_all_messages` + filter
- [ ] **ModernShellPrototype.tsx** (2 fixes)
  - [ ] Replace `core_get_user_info` with `gossip_get_own_identity`
  - [ ] Replace `core_add_bootstrap_node` with `gossip_add_bootstrap_peer`
- [ ] **messagingSubscription.ts** - Replace `core_subscribe_messages` with `gossip_subscribe_to_entity`

### Phase 2: Backend Implementation Required

These features need new backend commands to work:

- [ ] **Message Editing** - Implement `edit_message` in org_commands.rs
- [ ] **Message Deletion** - Implement `delete_message` in org_commands.rs
- [ ] **Display Name Update** - Implement `update_profile` in org_commands.rs
- [ ] **Entity Updates** - Implement `update_channel`, `update_project` in org_commands.rs
- [ ] **Entity Deletion** - Implement `delete_channel`, `delete_project` in org_commands.rs

### Phase 3: Testing

- [ ] Test all migrated commands in dev environment
- [ ] Verify error handling for missing features
- [ ] Add loading states for async operations
- [ ] Test offline behavior with CRDT sync
- [ ] Verify P2P messaging works across peers

---

## 🎯 PRIORITY ORDER

### HIGH PRIORITY (Blocking Core Features)

1. **Message Sending** - Used in 5+ places, core functionality
   - Files: MessagesPanel, EntityDirectoryContext, TouchContainer, BackendService
   - Migration: `core_messages_send` → `send_message`

2. **Message Subscription** - Required for real-time updates
   - Files: messagingSubscription, TouchContainer, useDHTSync
   - Migration: `core_subscribe_messages` → `gossip_subscribe_to_entity`

3. **Storage Operations** - Used by offline storage system
   - Files: OfflineStorageService
   - Migration: `core_private_put` → `gossip_store_message`

### MEDIUM PRIORITY (Feature Functionality)

4. **User Identity** - Used in prototype shell
   - Files: ModernShellPrototype
   - Migration: `core_get_user_info` → `gossip_get_own_identity`

5. **Direct Messaging** - P2P communication
   - Files: TouchContainer
   - Migration: `core_send_message_to_recipients` → `gossip_send_direct_message`

### LOW PRIORITY (Can Disable Temporarily)

6. **Message Edit/Delete** - No backend equivalent yet
   - Recommendation: Disable UI buttons until implemented

7. **Entity Updates** - Partial backend support
   - Recommendation: Implement missing commands in backend

---

## 🧪 TESTING STRATEGY

### Unit Tests

For each migrated file, add tests:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';

vi.mock('@tauri-apps/api/tauri');

describe('Message Sending Migration', () => {
  it('should call send_message with correct parameters', async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue({ id: 'msg-123', content: 'test' });

    await sendMessage('channel-1', 'test message', 'user-1');

    expect(mockInvoke).toHaveBeenCalledWith('send_message', {
      request: {
        channel_id: 'channel-1',
        author_id: 'user-1',
        content: 'test message',
        thread_id: null
      }
    });
  });
});
```

### Integration Tests

Test full flows with working commands:

1. Initialize core (`core_initialize`)
2. Initialize gossip (`gossip_initialize`)
3. Create channel (`create_channel`)
4. Send message (`send_message`)
5. Subscribe to updates (`gossip_subscribe_to_entity`)
6. Verify message received

### Manual Testing

- [ ] Open app in Tauri dev mode
- [ ] Create test channel
- [ ] Send messages (verify they appear)
- [ ] Test offline mode (verify CRDT sync)
- [ ] Test with multiple peers
- [ ] Verify error messages are clear

---

## 📊 ESTIMATED EFFORT

- **Phase 1 (Immediate Fixes)**: 8 hours
  - 11 files to update
  - Straightforward replacements
  - Test each change

- **Phase 2 (Backend Implementation)**: 16 hours
  - 5 new backend commands needed
  - CRDT integration
  - Testing

- **Phase 3 (Testing & Polish)**: 8 hours
  - Unit tests
  - Integration tests
  - Bug fixes

**Total**: 32 hours (~1 week)

---

## ✅ SUCCESS CRITERIA

- [ ] Zero frontend calls to stub commands
- [ ] All tests passing
- [ ] No "Not yet implemented" errors in production
- [ ] Features work offline with CRDT sync
- [ ] P2P messaging works across peers
- [ ] Clear error messages for missing features

---

**Status**: Ready to begin Phase 1 migrations 🚀

**Verification Date**: 2025-10-12
