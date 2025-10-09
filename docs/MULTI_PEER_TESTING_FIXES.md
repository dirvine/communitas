# Multi-Peer Testing Fixes

**Date**: 2025-10-07  
**Issues Fixed**: Port conflicts & Missing user indicator

---

## ❌ Issues Reported

### Issue 1: Port Conflict
**Problem**: When starting the first node (Alice), it uses port 5173 for Vite dev server. When trying to start the second node (Bob), there's a port conflict and Bob doesn't start.

**Root Cause**: `npm run tauri dev` tries to start a Vite dev server on port 5173 for each instance, causing conflicts.

### Issue 2: No Current User Display
**Problem**: The UI doesn't show which user is currently logged in, making it confusing to know which instance you're using (Alice vs Bob).

---

## ✅ Fixes Applied

### Fix 1: Port Conflict Resolution

**File**: `scripts/test-multi-peer.sh`

**Solution**: Modified the script to use production mode instead of dev mode:

1. **Build frontend once**: Runs `npm run build` if `dist/` doesn't exist
2. **Use built files**: Both instances share the same built frontend
3. **Run Tauri directly**: Uses `cargo run --release` instead of `npm run tauri dev`

**Changes**:
```bash
# OLD (caused port conflicts):
npm run tauri dev  # Each instance tries to start Vite on port 5173

# NEW (no port conflicts):
ensure_build()  # Build once if needed
cd communitas-desktop
cargo run --release  # Use built frontend, no Vite server
```

**Added environment variables**:
- `COMMUNITAS_USER_NAME="Alice"` (or "Bob")
- Allows distinguishing users in logs and potentially in UI

**Benefits**:
- ✅ No port conflicts - each instance runs independently
- ✅ Faster startup - no Vite compilation per instance
- ✅ More production-like behavior for testing
- ✅ Only builds frontend once

---

### Fix 2: Current User Display

**File**: `src/components/prototype/ModernShellPrototype.tsx`

**Solution**: Added a prominent user indicator in the chat list header (B1 section, lines 2256-2292).

**Visual Design**:
```
┌───────────────────────────────────┐
│ Chats                         + ⋯ │
├───────────────────────────────────┤
│  ┌─────────────────────────────┐  │
│  │ OC  Ocean Forest Moon Star  │  │  ← Avatar + Name
│  │     ocean-forest-moon-star  │  │  ← Peer ID (monospace)
│  └─────────────────────────────┘  │
└───────────────────────────────────┘
```

**Features**:
- **Avatar**: Shows first 2 letters of first word (OC for Ocean, RI for River)
- **Name**: Four-word ID formatted as title case (Ocean Forest Moon Star)
- **Peer ID**: Raw four-word ID in monospace font
- **Styling**: Accent-colored box with border, clearly visible
- **Position**: Directly below "Chats" header, always visible

**Code**:
```typescript
{ourPeerId && (
  <Box sx={{
    mt: 1.5,
    p: 1.5,
    bgcolor: alpha(TOKENS.accent, 0.1),
    borderRadius: 2,
    border: `1px solid ${alpha(TOKENS.accent, 0.3)}`
  }}>
    <Stack direction="row" spacing={1} alignItems="center">
      <Avatar sx={{ width: 32, height: 32, bgcolor: TOKENS.accent }}>
        {ourPeerId.split('-')[0].substring(0, 2).toUpperCase()}
      </Avatar>
      <Box>
        <Typography variant="body2" fontWeight={600}>
          {ourPeerId.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
        </Typography>
        <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
          {ourPeerId}
        </Typography>
      </Box>
    </Stack>
  </Box>
)}
```

**Benefits**:
- ✅ Immediately obvious which instance you're using
- ✅ Shows both human-readable name and technical peer ID
- ✅ Distinct avatar for quick visual identification
- ✅ Always visible at top of chat list

---

### Fix 3: Peer ID Synchronization (Backend ↔ Frontend)

**Issue Discovered**: After Fix 2, both Alice and Bob instances showed the same peer ID ("Ocean Forest Moon Star") instead of their unique identities.

**Root Cause**: The frontend was reading peer ID from URL parameters or localStorage instead of querying the Tauri backend, which has the correct unique peer ID from environment variables (COMMUNITAS_PEER_ID).

**Files Modified**:
1. `communitas-desktop/src/core_commands.rs` (Backend)
2. `src/components/prototype/ModernShellPrototype.tsx` (Frontend)
3. `communitas-desktop/src/main.rs` (Command registration)

**Solution**: Created Tauri commands to expose backend peer ID to frontend

#### Backend Changes (core_commands.rs)

Added two new Tauri commands after `core_initialize`:

```rust
/// Get the current user's peer ID (four-word address)
#[tauri::command]
pub async fn core_get_peer_id(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<String, String> {
    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "Core not initialized".to_string())?;
    Ok(ctx.four_words.clone())
}

/// Get current user info (peer ID and display name)
#[tauri::command]
pub async fn core_get_user_info(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<serde_json::Value, String> {
    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "Core not initialized".to_string())?;

    Ok(serde_json::json!({
        "peerId": ctx.four_words,
        "displayName": ctx.display_name,
        "identityHex": hex::encode(ctx.identity.identity.identity_key_bytes()),
    }))
}
```

#### Frontend Changes (ModernShellPrototype.tsx)

**State Management**: Added display name state alongside peer ID (line 732):
```typescript
const [ourPeerId, setOurPeerId] = useState<string>('')
const [ourDisplayName, setOurDisplayName] = useState<string>('')
```

**Modified `initializeMessaging`** (lines 867-897) to query `core_get_user_info`:

```typescript
const initializeMessaging = async () => {
  let testPeerId = 'ocean-forest-moon-star' // default fallback
  let testDisplayName = '' // default empty

  // Check if running in Tauri - try to get user info from backend
  if (window.__TAURI__?.tauri?.invoke) {
    try {
      const userInfo = await window.__TAURI__.tauri.invoke<{ peerId: string; displayName: string }>('core_get_user_info')
      if (userInfo && userInfo.peerId) {
        testPeerId = userInfo.peerId
        testDisplayName = userInfo.displayName || ''
        console.log('✅ Got user info from Tauri backend:', { peerId: testPeerId, displayName: testDisplayName })
      }
    } catch (err) {
      console.log('⚠️  Core not initialized yet, will use fallback values')
    }
  } else {
    // Browser mode - use URL param or localStorage
    const urlParams = new URLSearchParams(window.location.search)
    testPeerId = urlParams.get('peerId') || localStorage.getItem('testPeerId') || testPeerId
    testDisplayName = localStorage.getItem('testDisplayName') || ''
  }

  // Save to localStorage and set state
  localStorage.setItem('testPeerId', testPeerId)
  localStorage.setItem('testDisplayName', testDisplayName)
  setOurPeerId(testPeerId)
  setOurDisplayName(testDisplayName)
  // ... rest of initialization
}
```

**Updated User Indicator** (lines 2276-2315) to show display name:

```typescript
{/* Current User Indicator */}
{ourPeerId && (
  <Box sx={{ mt: 1.5, p: 1.5, bgcolor: alpha(TOKENS.accent, 0.1), borderRadius: 2, border: `1px solid ${alpha(TOKENS.accent, 0.3)}` }}>
    <Stack direction="row" spacing={1} alignItems="center">
      <Avatar sx={{ width: 32, height: 32, bgcolor: TOKENS.accent, fontSize: 14, fontWeight: 600 }}>
        {ourDisplayName
          ? ourDisplayName.substring(0, 2).toUpperCase()
          : ourPeerId.split('-')[0].substring(0, 2).toUpperCase()
        }
      </Avatar>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="body2" fontWeight={600} sx={{ color: TOKENS.textPrimary }}>
          {ourDisplayName || ourPeerId.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
        </Typography>
        <Typography variant="caption" sx={{ color: TOKENS.textSecondary, fontSize: 11, fontFamily: 'monospace' }}>
          {ourPeerId}
        </Typography>
      </Box>
    </Stack>
  </Box>
)}
```

#### Command Registration (main.rs)

Registered the new commands in the Tauri app (lines 151-152):

```rust
core_commands::core_initialize,
core_commands::core_get_peer_id,
core_commands::core_get_user_info,
core_commands::core_create_channel,
```

**Benefits**:
- ✅ Each instance now shows its unique identity from backend
- ✅ Alice shows "Alice" with peer ID "ocean-forest-moon-star"
- ✅ Bob shows "Bob" with peer ID "river-mountain-sun-cloud"
- ✅ Avatar uses display name initials (AL for Alice, BO for Bob)
- ✅ Automatic fallback to URL/localStorage in browser mode
- ✅ Graceful handling when core not yet initialized

---

## 🧪 Testing Instructions

### Updated Launch Process

**Step 1: Build Frontend (Once)**
```bash
cd /Users/davidirvine/Desktop/Devel/projects/communitas
npm run build
```

**Step 2: Launch Alice (Terminal 1)**
```bash
./scripts/test-multi-peer.sh alice
```

**Expected Output**:
```
═══════════════════════════════════════════════════════
  Launching Alice's Instance (Production Mode)
═══════════════════════════════════════════════════════
  Peer ID:   ocean-forest-moon-star
  Data Dir:  ~/.communitas-data-alice
  Port:      8080
═══════════════════════════════════════════════════════

Using existing frontend build

Starting Tauri (production mode to avoid port conflicts)...
Press Ctrl+C to stop
```

**Step 3: Launch Bob (Terminal 2)**
```bash
./scripts/test-multi-peer.sh bob
```

**Expected Output**:
```
═══════════════════════════════════════════════════════
  Launching Bob's Instance (Production Mode)
═══════════════════════════════════════════════════════
  Peer ID:   river-mountain-sun-cloud
  Data Dir:  ~/.communitas-data-bob
  Port:      8081
═══════════════════════════════════════════════════════

Using existing frontend build

Starting Tauri (production mode to avoid port conflicts)...
Press Ctrl+C to stop
```

**Step 4: Verify in UI**

**In Alice's Window**:
- Look at top of chat list
- Should see user indicator box:
  - Avatar: "OC"
  - Name: "Ocean Forest Moon Star"
  - Peer ID: "ocean-forest-moon-star"

**In Bob's Window**:
- Look at top of chat list
- Should see user indicator box:
  - Avatar: "RI"
  - Name: "River Mountain Sun Cloud"
  - Peer ID: "river-mountain-sun-cloud"

---

## 📊 Verification Checklist

### Port Conflict Fix
- [ ] Alice starts successfully
- [ ] Bob starts successfully (no port conflict)
- [ ] Both instances run simultaneously
- [ ] Each instance has its own data directory
- [ ] Frontend built only once

### User Display Fix
- [ ] Alice shows "Ocean Forest Moon Star"
- [ ] Bob shows "River Mountain Sun Cloud"
- [ ] Avatar shows correct initials (OC vs RI)
- [ ] Peer ID visible in monospace
- [ ] User indicator always visible at top

### Multi-Peer Testing
- [ ] Can distinguish instances at a glance
- [ ] Can send messages from Alice to Bob
- [ ] Can send messages from Bob to Alice
- [ ] CRDT sync works between instances
- [ ] Messages appear with correct author

---

## 🎯 Expected Behavior

### Successful Launch
1. First terminal runs Alice with "ocean-forest-moon-star"
2. Second terminal runs Bob with "river-mountain-sun-cloud"
3. No port conflicts or errors
4. Both windows clearly labeled with user name
5. Ready for P2P messaging tests

### Visual Identification
```
┌─ Alice's Window ────────┐  ┌─ Bob's Window ──────────┐
│ [OC] Ocean Forest...    │  │ [RI] River Mountain...  │
│ ocean-forest-moon-star  │  │ river-mountain-sun-...  │
└─────────────────────────┘  └─────────────────────────┘
```

---

## 📁 Files Modified

1. `scripts/test-multi-peer.sh`
   - Added `ensure_build()` function
   - Changed from `npm run tauri dev` to `cargo run --release`
   - Added `COMMUNITAS_USER_NAME` environment variable

2. `src/components/prototype/ModernShellPrototype.tsx`
   - Added user indicator component (lines 2256-2292)
   - Shows avatar, formatted name, and peer ID
   - Modified `initializeMessaging` to call Tauri backend for peer ID (lines 867-896)

3. `communitas-desktop/src/core_commands.rs`
   - Added `core_get_peer_id()` command
   - Added `core_get_user_info()` command

4. `communitas-desktop/src/main.rs`
   - Registered new commands in `generate_handler!` (lines 151-152)

---

## 🚀 Next Steps

With these fixes in place:

1. ✅ **Launch both instances** using the test script
2. ✅ **Verify user displays** are correct in each window
3. **Test messaging** between Alice and Bob
4. **Verify CRDT sync** with vector clocks
5. **Test all scenarios** from CRDT_MULTI_PEER_TEST.md

---

## 📚 References

- [Test Script](../scripts/test-multi-peer.sh)
- [Multi-Peer Test Guide](./CRDT_MULTI_PEER_TEST.md)
- [Backend Integration Status](./CRDT_TAURI_BACKEND_STATUS.md)
- [UI Component](../src/components/prototype/ModernShellPrototype.tsx)
