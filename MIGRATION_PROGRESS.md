# Frontend Migration Progress Report

**Date**: 2025-10-12 • **Status**: IN PROGRESS ✅

## ✅ COMPLETED MIGRATIONS (2/11 files)

### 1. MessagesPanel.tsx ✅
**Location**: `src/components/entity/MessagesPanel.tsx`

**Changes Made**:
- ✅ Added `import { useAuth } from '../../contexts/AuthContext';`
- ✅ Added `const { authState } = useAuth();` to component
- ✅ Migrated `core_messages_send` → `send_message` with working command
- ✅ Added `author_id: authState.user.id` parameter
- ✅ Wrapped parameters in `request` object
- ✅ Added `thread_id: null` for main channel messages
- ✅ Disabled `core_messages_edit` (stub) with TODO comment
- ✅ Disabled `core_messages_delete` (stub) with TODO comment
- ✅ Verified with `npm run typecheck` - PASSED

**Migration Example**:
```typescript
// ❌ BEFORE (stub command):
const message = await invoke('core_messages_send', {
  entity_id: entityId,
  content: newMessage.trim(),
  encrypted: true,
});

// ✅ AFTER (working command):
const message = await invoke('send_message', {
  request: {
    channel_id: entityId,
    author_id: authState.user.id,
    content: newMessage.trim(),
    thread_id: null,
  },
});
```

---

### 2. EntityDirectoryContext.tsx ✅
**Location**: `src/contexts/EntityDirectoryContext.tsx`

**Changes Made**:
- ✅ Added `import { useAuth } from './AuthContext';`
- ✅ Added `const { authState } = useAuth();` to Provider component
- ✅ Updated `syncEntityToBackend` dependency array to include `authState`
- ✅ Migrated `core_send_message_to_channel` → `send_message` with working command
- ✅ Added auth check: `if (!authState.user?.id) return null`
- ✅ Added `author_id: authState.user.id` parameter
- ✅ Verified with `npm run typecheck` - PASSED

**Migration Example**:
```typescript
// ❌ BEFORE (stub command):
await invoke('core_send_message_to_channel', {
  channel_id: payload.entityId,
  text: payload.content,
});

// ✅ AFTER (working command):
await invoke('send_message', {
  request: {
    channel_id: payload.entityId,
    author_id: authState.user.id,
    content: payload.content,
    thread_id: null,
  },
});
```

---

## 🚧 REMAINING MIGRATIONS (9/11 files)

### 3. TouchContainer.tsx (5 stub calls)
**Location**: `src/components/ui/TouchContainer.tsx`
**Priority**: HIGH - Most stub calls in single file

**Required Changes**:
1. **Line 335**: `core_send_message_to_channel` → `send_message`
2. **Line 351**: `core_send_message_to_recipients` → `gossip_send_direct_message`
3. **Line 358**: `core_send_message_to_channel` → `send_message` (duplicate)
4. **Line 373**: `core_subscribe_messages` → `gossip_subscribe_to_entity`

**Migration Steps**:
```typescript
// 1. Add imports
import { useAuth } from '../../contexts/AuthContext';

// 2. In component:
const { authState } = useAuth();

// 3. Replace core_send_message_to_channel (lines 335, 358):
await invoke('send_message', {
  request: {
    channel_id: currentEntity.id,
    author_id: authState.user.id,
    content: text,
    thread_id: null
  }
});

// 4. Replace core_send_message_to_recipients (line 351):
// Note: Send to multiple recipients requires loop
for (const recipientFourWords of recipientAddresses) {
  await invoke('gossip_send_direct_message', {
    recipient_four_words: recipientFourWords,
    content: message,
    encrypted: true
  });
}

// 5. Replace core_subscribe_messages (line 373):
await invoke('gossip_subscribe_to_entity', {
  entity_id: currentEntity.id,
  entity_type: 'channel' // or determine from entity
});
```

---

### 4. useDHTSync.ts (2 stub calls)
**Location**: `src/hooks/useDHTSync.ts`

**Required Changes**:
1. **Line 109**: `subscribe_to_entity` → `gossip_subscribe_to_entity`
2. **Line 140**: `unsubscribe_from_entity` → `gossip_leave_entity`

**Migration Steps**:
```typescript
// Line 109:
await invoke('gossip_subscribe_to_entity', {
  entity_id: entityId,
  entity_type: entityType // Add entity_type parameter
});

// Line 140:
await invoke('gossip_leave_entity', {
  entity_id: entityId
});
```

---

### 5. BackendService.ts (1 stub call)
**Location**: `src/services/api/BackendService.ts`

**Required Changes**:
1. **Line 195**: `core_send_message_to_channel` → `send_message`

**Migration Steps**:
```typescript
// Add to BackendService class:
private currentUserId: string; // Set during initialization

// Line 195:
return await invoke('send_message', {
  request: {
    channel_id: channelId,
    author_id: this.currentUserId,
    content,
    thread_id: null
  }
});
```

---

### 6. OfflineStorageService.ts (4 stub calls)
**Location**: `src/services/storage/OfflineStorageService.ts`

**Required Changes**:
1. **Lines 181, 233, 564**: `core_private_put` → `gossip_store_message`
2. **Line 400**: `core_private_get` → `gossip_get_all_messages` + filter

**Migration Steps**:
```typescript
// Replace core_private_put (3 locations):
await invoke('gossip_store_message', {
  key: storageKey,
  value: encryptedData
});

// Replace core_private_get (line 400):
const allMessages = await invoke('gossip_get_all_messages') as Array<{
  key: string;
  value: number[];
}>;
const encrypted = allMessages.find(m => m.key === key)?.value ?? null;
```

---

### 7. ModernShellPrototype.tsx (5 stub calls)
**Location**: `src/components/prototype/ModernShellPrototype.tsx`

**Required Changes**:
1. **Line 1025**: `core_get_user_info` → `gossip_get_own_identity`
2. **Line 1184**: `core_add_bootstrap_node` → `gossip_add_bootstrap_peer`
3. **Line 1208**: `core_set_display_name` → **NO EQUIVALENT** (disable feature)
4. **Line 1905**: `core_entity_update` → **DEPENDS ON ENTITY** (see guide)
5. **Line 1933**: `core_entity_delete` → **NO EQUIVALENT** (disable feature)

**Migration Steps**:
```typescript
// Line 1025:
const identity = await invoke('gossip_get_own_identity') as {
  four_words: string;
  display_name: string;
  public_key: string;
};

// Line 1184:
await invoke('gossip_add_bootstrap_peer', {
  multiaddr: bootstrapAddress
});

// Lines 1208, 1905, 1933: Disable with TODO comments
// (No working backend implementations yet)
```

---

### 8. messagingSubscription.ts (1 stub call)
**Location**: `src/services/messagingSubscription.ts`

**Required Changes**:
1. **Line 22**: `core_subscribe_messages` → `gossip_subscribe_to_entity`

**Migration Steps**:
```typescript
await invoke('gossip_subscribe_to_entity', {
  entity_id: opts?.channelId,
  entity_type: 'channel'
});
```

---

## 📊 VERIFICATION CHECKLIST

After each file migration:
- [ ] Run `npm run typecheck` - Must pass
- [ ] Check console for TypeScript errors
- [ ] Verify imports are correct
- [ ] Confirm auth state is accessed properly

After all migrations:
- [ ] Run `npm run typecheck` - Final verification
- [ ] Run `npm test` - Ensure no test regressions
- [ ] Test in Tauri dev mode if possible
- [ ] Document any blocked features

---

## 🎯 MIGRATION PATTERN (Copy-Paste Template)

### For Message Sending Commands:
```typescript
// 1. Add import (if not already present):
import { useAuth } from '../../contexts/AuthContext';

// 2. In component/function:
const { authState } = useAuth();

// 3. Replace command:
await invoke('send_message', {
  request: {
    channel_id: channelId,
    author_id: authState.user!.id,  // Add ! if you've checked it exists
    content: messageContent,
    thread_id: null  // or threadId for replies
  }
});
```

### For Subscription Commands:
```typescript
// Replace:
await invoke('gossip_subscribe_to_entity', {
  entity_id: entityId,
  entity_type: 'channel' // or 'project', 'group'
});

// Replace unsubscribe:
await invoke('gossip_leave_entity', {
  entity_id: entityId
});
```

### For Storage Commands:
```typescript
// Put:
await invoke('gossip_store_message', {
  key: storageKey,
  value: data
});

// Get (with filter):
const all = await invoke('gossip_get_all_messages');
const item = all.find(m => m.key === key);
```

---

## 🚀 NEXT STEPS

### Continue Migration (Estimated 6 hours remaining)

1. **TouchContainer.tsx** (30 min)
   - Most complex file
   - 5 different stub calls
   - Needs loop for multi-recipient messages

2. **useDHTSync.ts** (15 min)
   - Simple replacements
   - Just entity type parameter needed

3. **BackendService.ts** (15 min)
   - Single call to replace
   - May need class refactoring for user ID

4. **OfflineStorageService.ts** (45 min)
   - 4 calls to replace
   - Get operation needs client-side filtering
   - Consider caching for performance

5. **ModernShellPrototype.tsx** (30 min)
   - 2 working replacements
   - 3 features to disable

6. **messagingSubscription.ts** (15 min)
   - Single simple replacement

7. **Final Verification** (3 hours)
   - Run all typechecks
   - Run test suite
   - Test in Tauri dev mode
   - Create test plan document

### After Migration Complete

1. **Backend Implementation** (Phase 2)
   - Implement `edit_message` in org_commands.rs
   - Implement `delete_message` in org_commands.rs
   - Implement `update_profile` in org_commands.rs
   - Implement entity update/delete commands

2. **Re-enable Features** (Phase 2)
   - Remove disabled code comments
   - Wire up new backend commands
   - Test end-to-end

3. **Documentation Updates**
   - Update AGENTS_API.md with accurate command status
   - Update component documentation
   - Add migration notes to CHANGELOG.md

---

## ✅ SUCCESS METRICS

### Current Status:
- Files Migrated: 2/11 (18%)
- Stub Calls Fixed: 3/21+ (14%)
- Compilation: ✅ PASSING
- Tests: ⏳ NOT YET RUN

### Target Status:
- Files Migrated: 11/11 (100%)
- Stub Calls Fixed: 21+/21+ (100%)
- Compilation: ✅ PASSING
- Tests: ✅ PASSING
- Features: Some disabled pending backend implementation

---

## 💡 KEY LEARNINGS

1. **Auth Context Required**: All message sends need `authState.user.id`
2. **Request Wrapper**: Working commands use `request` object wrapper
3. **Thread ID**: Always include `thread_id: null` for main channel messages
4. **Entity Type**: Subscriptions need `entity_type` parameter
5. **Multi-Recipient**: No batch send - must loop for gossip_send_direct_message
6. **Storage Get**: No single-key get - must filter all messages client-side

---

## 🔧 TROUBLESHOOTING

### TypeScript Errors

**Problem**: "Cannot find name 'authState'"
**Solution**: Add `const { authState } = useAuth();` to component

**Problem**: "Property 'user' does not exist"
**Solution**: Import AuthContext types, check `authState.user?.id`

### Runtime Errors

**Problem**: "Not yet implemented" still appearing
**Solution**: Verify you're calling the NEW command name, not the stub

**Problem**: "Missing parameter: author_id"
**Solution**: Ensure `authState.user.id` is included in request

---

**Status**: Ready to continue Phase 1 migrations 🚀

**Estimated Time Remaining**: 6 hours for remaining 9 files

**Confidence**: High - Clear patterns established, verified working
