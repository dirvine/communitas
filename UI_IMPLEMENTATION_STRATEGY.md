# Communitas UI Implementation - Deep Strategy

**Date:** 2025-01-29  
**Duration:** 2-3 weeks  
**Approach:** Risk-first, incremental validation

---

## 🎯 STRATEGIC THINKING

### The Riskiest Assumptions

**ASSUMPTION #1:** Users will understand DNS-free addressing  
**RISK:** Confusion, abandonment  
**MITIGATION:** Build Viewer first with excellent onboarding

**ASSUMPTION #2:** Discovery latency is acceptable  
**RISK:** Users expect instant load (like HTTP)  
**MITIGATION:** Clear progress indicators, cache aggressively

**ASSUMPTION #3:** TOFU security is intuitive  
**RISK:** Users click through without understanding  
**MITIGATION:** Simple but clear security dialogs

**ASSUMPTION #4:** Publishing workflow makes sense  
**RISK:** Users get stuck, don't publish  
**MITIGATION:** Wizard with clear steps, good defaults

### Validation Strategy

**Build in this order to validate assumptions early:**

1. **Day 1-2:** Minimal Viewer (validate discovery + fetch)
2. **Day 3-4:** Basic Publisher (validate end-to-end)
3. **Day 5-6:** Security UI (validate TOFU UX)
4. **Day 7-10:** Polish and error states
5. **Day 11-14:** Testing and iteration

---

## 📅 DAY-BY-DAY PLAN (Realistic, Not Optimistic)

### **DAY 1: Minimal Viewer Foundation**

**Goal:** Browse to a hardcoded SiteId and see content

**Morning (4h):**
1. Create `ViewerPage.tsx` component (1h)
   - Simple address bar (text input for SiteId hex)
   - Status text (resolving, fetching, etc.)
   - Content area (just show JSON for now)

2. Wire up `gossip_site_fetch` IPC (1h)
   - Handle loading states
   - Handle errors
   - Display fetched assets

3. Test with SitesDemo (1h)
   - Publish a site via existing SitesDemo
   - Copy SiteId
   - Paste into Viewer
   - Verify it fetches

4. **VALIDATION CHECKPOINT:** Can we fetch and display a site? (1h)

**Afternoon (4h):**
5. Add basic HTML rendering (2h)
   - Create `SandboxedRenderer.tsx`
   - Render index.html content
   - Basic CSS support
   - No external resources

6. Improve status indicators (1h)
   - "Fetching manifest..."
   - "Fetching blocks X/Y..."
   - Success/error states

7. **END OF DAY TEST:** Publish → Fetch → Render (1h)

**Deliverable:** Can browse to SiteId and see rendered HTML ✅

**Risk Mitigation:**
- If fetch fails: Debug with existing SitesDemo
- If render fails: Show raw HTML first
- If IPC breaks: Test with curl/manual commands

---

### **DAY 2: Four-Words + Discovery**

**Goal:** Type four-words and discover providers

**Morning (4h):**
1. Create `FourWordsInput.tsx` component (1.5h)
   - Text input with validation
   - Dictionary word checking (client-side)
   - Error states
   - Auto-hyphenation

2. Implement `names_resolve` backend command (1h)
   ```rust
   #[tauri::command]
   async fn names_resolve(four_words: String) -> Result<Option<String>, String>
   ```

3. Wire up name resolution in Viewer (1h)
   - Input four-words → resolve to SiteId
   - Show "Resolving..." state
   - Handle "not found" error

4. **VALIDATION:** Type "test-site-here-now" → resolves → fetches (0.5h)

**Afternoon (4h):**
5. Add provider discovery UI (2h)
   - Call `gossip_site_providers`
   - Show provider count
   - Display provider list (simple)

6. Implement auto-provider-selection (1h)
   - Pick first provider
   - Try next on failure
   - Show which provider is active

7. **END OF DAY TEST:** Four-words → Discover → Fetch → Render (1h)

**Deliverable:** DNS-free browsing works! ✅

**Risk Mitigation:**
- If discovery slow: Add timeout + cache
- If no providers: Show "offline mode" fallback
- If validation breaks: Allow SiteId paste as backup

---

### **DAY 3: Minimal Publisher (Step 1-2)**

**Goal:** Select folder and claim name

**Morning (4h):**
1. Create `PublisherWizard.tsx` skeleton (1h)
   - Material-UI Stepper
   - 4 step placeholders
   - Navigation (Back/Next)

2. Implement Step 1: Select Content (2h)
   - Folder picker using Tauri dialog API
   - File scanner backend command
   - Show file list preview
   - Estimate blocks

3. Backend: `sites_scan_folder` command (1h)
   ```rust
   #[tauri::command]
   async fn sites_scan_folder(
       path: String,
       exclude_patterns: Vec<String>,
   ) -> Result<ScanResult, String>
   ```

**Afternoon (4h):**
4. Implement Step 2: Name & Key (2.5h)
   - Four-words input (reuse from Viewer)
   - Name availability check
   - Key generation UI
   - Fingerprint display

5. Backend commands (1h)
   - `identity_generate_four_words`
   - `names_check_availability`
   - `sites_generate_keypair`

6. **END OF DAY TEST:** Select folder → Claim name (0.5h)

**Deliverable:** Can prepare content for publishing ✅

---

### **DAY 4: Publisher (Step 3-4) - Critical Day**

**Goal:** Complete publish flow end-to-end

**Morning (4h):**
1. Implement Step 3: Sign & Review (2h)
   - Manifest preview table
   - Sign buttons
   - Show fingerprint/lock when signed

2. Backend signing commands (2h)
   - `sites_build_manifest`
   - `sites_sign_manifest`
   - `names_create_and_sign`

**Afternoon (4h):**
3. Implement Step 4: Publish (2h)
   - Publication status
   - Progress indicators
   - Share card (copy four-words)

4. Backend: `sites_start_provider` (1.5h)
   - Start SitesListener on port 5001
   - Store signed manifest
   - Publish ProviderSummary

5. **CRITICAL VALIDATION:** Full publish flow (0.5h)
   - Wizard: Select → Name → Sign → Publish
   - Verify provider starts
   - **TEST:** Use Viewer to fetch published site
   - **END-TO-END PROOF!**

**Deliverable:** Can publish AND browse sites! ✅

**Risk Mitigation:**
- If publish fails: Test each IPC command individually
- If provider doesn't start: Check port 5001 binding
- If fetch fails after publish: Debug transport routing

---

### **DAY 5: Security & TOFU**

**Goal:** Users understand and trust the security model

**Morning (4h):**
1. Create `PQCLockIndicator.tsx` (1h)
   - Green lock when verified
   - Orange for TOFU
   - Red for mismatch
   - Click to show details

2. Create `TOFUDialog.tsx` (2h)
   - First-time key prompt
   - Show fingerprint
   - Options: Trust&Pin, Trust Once, Reject
   - Clear explanation

3. Backend: `security_check_tofu` (1h)
   ```rust
   #[tauri::command]
   async fn security_check_tofu(
       site_id: String,
       public_key: Vec<u8>,
   ) -> Result<TOFUStatus, String>
   ```

**Afternoon (4h):**
4. Integrate TOFU into Viewer (2h)
   - Check TOFU status after fetch_manifest
   - Show dialog on first-time
   - Block rendering if rejected
   - Store trust decisions

5. Add `SiteInfoDrawer.tsx` skeleton (1.5h)
   - Identity section (four-words, SiteId)
   - Security section (fingerprint, trust status)
   - Manifest section (version, timestamp)

6. **VALIDATION:** Browse new site → TOFU prompt → Trust → Render (0.5h)

**Deliverable:** Security model visible and functional ✅

---

### **DAY 6-7: Progress & Feedback**

**Goal:** Users know what's happening

**Day 6 Morning (4h):**
1. Create `FetchProgress.tsx` (1.5h)
   - Progress bar for blocks
   - Bytes transferred
   - Speed/ETA

2. Add fetch progress to Viewer (1h)
   - Listen to `sites:fetch_progress` events
   - Update progress bar
   - Show provider being used

3. Backend: Progress events (1.5h)
   - Emit events during fetch_blocks
   - Track bytes transferred
   - Calculate speed

**Day 6 Afternoon (4h):**
4. Create `PublishProgress.tsx` (1.5h)
   - Timeline (Chunking → Signing → Publishing)
   - Block preparation progress
   - Provider status

5. Add to Publisher Step 4 (1h)
   - Wire up progress component
   - Listen to publish events
   - Show completion state

6. Polish transitions (1.5h)
   - Loading states
   - Skeleton screens
   - Smooth animations

**Day 7: Error States & Edge Cases (8h)**
- "No providers found" handling
- "Name conflict" UI
- "Signature invalid" warning
- "Fetch timeout" retry
- Network offline handling
- Cache fallback UI

**Deliverable:** Professional UX with clear feedback ✅

---

### **DAY 8-9: Cache & Offline**

**Goal:** Offline-first works seamlessly

**Day 8:**
1. `OfflineChip.tsx` component
2. Cache status in SiteInfoDrawer
3. Pin/Unpin functionality
4. "Viewing from cache" indicators

**Day 9:**
5. Offline mode when no providers
6. Cache management UI
7. Storage stats
8. Clear cache functionality

**Deliverable:** Offline viewing works perfectly ✅

---

### **DAY 10: Polish & Integration**

**Goal:** Everything feels cohesive

1. Consistent styling (2h)
2. Keyboard shortcuts (1h)
3. Navigation (back/forward) (1h)
4. Bookmarks (2h)
5. History (2h)

**Deliverable:** Feels like a real product ✅

---

### **DAY 11-14: Testing & Iteration**

**Goal:** Bulletproof quality

**Day 11: Manual Testing**
- Test every user flow
- Try to break it
- Document bugs

**Day 12: Bug Fixes**
- Fix critical issues
- Improve error messages
- Performance optimization

**Day 13: User Testing**
- 3-5 friendly users
- Watch them use it
- Collect feedback

**Day 14: Final Polish**
- Address feedback
- Documentation
- Release notes

**Deliverable:** Production-ready UI! ✅

---

## 🎯 MVP FEATURE SET (Absolute Minimum)

### Publisher (MVP)
- ✅ Select folder (single folder, not recursive)
- ✅ Claim four-words (manual entry, no random generator)
- ✅ Auto-generate key (no import for MVP)
- ✅ Sign manifest (one button)
- ✅ Publish (start provider)
- ✅ Share four-words (copy button)

### Viewer (MVP)
- ✅ Address bar (four-words input)
- ✅ Name resolution (four-words → SiteId)
- ✅ Provider discovery (show count)
- ✅ Fetch manifest + blocks
- ✅ Verify signatures
- ✅ Render HTML/CSS (no JS)
- ✅ TOFU dialog (Trust/Reject only)

### Security (MVP)
- ✅ Signature verification
- ✅ TOFU prompt
- ✅ Fingerprint display
- ✅ Basic trust store

### NOT in MVP (Defer to v1.1)
- ❌ Diff preview for updates
- ❌ Multi-provider failover UI
- ❌ Advanced key management
- ❌ Recovery kit export
- ❌ Bookmarks/history
- ❌ Full Site Info drawer
- ❌ Cache management UI
- ❌ JavaScript execution
- ❌ Provider scoring UI

---

## 🚀 INCREMENTAL VALIDATION

### Validation Points (Must Pass Before Continuing)

**Day 1 END:**
- [ ] Can fetch SiteId and see raw data
- [ ] Can render basic HTML
- **If fail:** Backend IPC broken, fix before continuing

**Day 2 END:**
- [ ] Can type four-words and resolve
- [ ] Can discover providers
- **If fail:** Name resolution broken, critical

**Day 4 END:**
- [ ] Can publish complete site
- [ ] Can browse own published site
- **If fail:** End-to-end broken, STOP and debug

**Day 5 END:**
- [ ] TOFU dialog shows and works
- [ ] Can trust/reject keys
- **If fail:** Security UX broken, redesign

**Day 7 END:**
- [ ] All error states handled gracefully
- [ ] No crashes on edge cases
- **If fail:** Need more error handling

---

## 🧪 TESTING STRATEGY

### Manual Testing Scenarios

**Scenario 1: Happy Path**
```
1. Open Publisher Wizard
2. Select folder ~/test-site
3. Type "my-cool-site-here"
4. Click through wizard
5. See "Published!" success
6. Copy four-words
7. Open Viewer
8. Paste four-words
9. See site render
✓ SUCCESS
```

**Scenario 2: Error Handling**
```
1. Viewer: Type non-existent four-words
   → See "Name not found" error ✓
2. Viewer: Disconnect network
   → See "No providers" with offline fallback ✓
3. Publisher: Try to claim taken name
   → See "Name conflict (TOFU)" error ✓
```

**Scenario 3: Security**
```
1. Browse to new site
   → TOFU dialog appears ✓
2. Reject key
   → Site blocked, clear message ✓
3. Browse again, trust
   → Site renders ✓
4. Simulate key change
   → "KEY CHANGED" warning ✓
```

### Automated Tests (Later)

```typescript
describe('PublisherWizard E2E', () => {
  it('completes full publish flow', async () => {
    // Mock all IPC commands
    // Step through wizard
    // Verify calls made in order
  });
});

describe('Viewer E2E', () => {
  it('fetches and renders site', async () => {
    // Mock resolve, fetch, providers
    // Type four-words
    // Verify rendering
  });
});
```

---

## 🎨 FIRST COMPONENT TO BUILD

### Component: `ViewerPage.tsx` (Minimal)

**Purpose:** Prove backend works end-to-end

**Code Skeleton:**

```typescript
import React, { useState } from 'react';
import { Box, TextField, Button, Typography, CircularProgress, Alert } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';

interface ViewerState {
  siteIdInput: string;
  loading: boolean;
  error: string | null;
  fetchedSite: { path: string; content: string }[] | null;
  renderedHtml: string | null;
}

export const ViewerPage: React.FC = () => {
  const [state, setState] = useState<ViewerState>({
    siteIdInput: '',
    loading: false,
    error: null,
    fetchedSite: null,
    renderedHtml: null,
  });

  const handleFetch = async () => {
    setState(s => ({ ...s, loading: true, error: null }));

    try {
      // Fetch site
      const site = await invoke<{ assets: { path: string; content_base64: string }[] }>(
        'gossip_site_fetch',
        { site_id_hex: state.siteIdInput }
      );

      // Decode assets
      const assets = site.assets.map(asset => ({
        path: asset.path,
        content: atob(asset.content_base64),
      }));

      // Find index.html
      const indexAsset = assets.find(a => a.path === 'index.html');
      
      setState(s => ({
        ...s,
        loading: false,
        fetchedSite: assets,
        renderedHtml: indexAsset?.content || null,
        error: indexAsset ? null : 'No index.html found',
      }));
    } catch (err) {
      setState(s => ({
        ...s,
        loading: false,
        error: `Failed to fetch: ${err}`,
      }));
    }
  };

  return (
    <Box sx={{ p: 3, maxWidth: 1200, mx: 'auto' }}>
      <Typography variant="h4" gutterBottom>
        Communitas Website Viewer
      </Typography>

      <Box sx={{ display: 'flex', gap: 2, mb: 3 }}>
        <TextField
          fullWidth
          label="Site ID (hex) or Four-Words"
          value={state.siteIdInput}
          onChange={(e) => setState(s => ({ ...s, siteIdInput: e.target.value }))}
          placeholder="Enter SiteId hex or four-words..."
          disabled={state.loading}
        />
        <Button
          variant="contained"
          onClick={handleFetch}
          disabled={!state.siteIdInput || state.loading}
        >
          {state.loading ? <CircularProgress size={24} /> : 'Browse'}
        </Button>
      </Box>

      {state.error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {state.error}
        </Alert>
      )}

      {state.renderedHtml && (
        <Box
          sx={{
            border: '1px solid',
            borderColor: 'divider',
            borderRadius: 1,
            p: 2,
            bgcolor: 'background.paper',
          }}
        >
          <Typography variant="caption" color="text.secondary" gutterBottom>
            Rendered Content:
          </Typography>
          <div dangerouslySetInnerHTML={{ __html: state.renderedHtml }} />
        </Box>
      )}

      {state.fetchedSite && (
        <Box sx={{ mt: 2 }}>
          <Typography variant="subtitle2" gutterBottom>
            Fetched {state.fetchedSite.length} assets
          </Typography>
          {state.fetchedSite.map(asset => (
            <Typography key={asset.path} variant="caption" display="block">
              • {asset.path} ({asset.content.length} bytes)
            </Typography>
          ))}
        </Box>
      )}
    </Box>
  );
};

export default ViewerPage;
```

**Why This First:**
1. Simplest possible UI
2. Tests backend immediately
3. Proves end-to-end works
4. Foundation for full Viewer

**Success Criteria:**
- Can paste SiteId
- Can fetch site
- Can see HTML rendered
- **This validates the entire backend!**

---

## 🎓 CRITICAL SUCCESS FACTORS

### 1. Start Simple, Add Complexity

**DON'T:**
- Build all 4 wizard steps at once
- Try to perfect UI before testing backend
- Add features before basics work

**DO:**
- Build minimal viewer first (validate backend)
- Add one feature at a time
- Test after each addition

### 2. Validate Assumptions Early

**Day 1:** Does fetch work?  
**Day 2:** Does discovery work?  
**Day 4:** Does end-to-end work?  
**Day 5:** Is TOFU understandable?

**If any fails, STOP and fix before continuing!**

### 3. Keep MVP Scope Tight

**Essential:**
- Basic publish
- Basic browse
- Signature verification
- TOFU dialog

**Can Wait:**
- Advanced features
- Perfect UX
- Edge case handling
- Performance optimization

### 4. Real User Testing Early

**Day 7:** Give to 1-2 technical users  
**Day 10:** Give to 3-5 less technical users  
**Day 13:** Broader testing

**Watch them use it, don't tell them how!**

---

## 📊 RISK MITIGATION

### High-Risk Areas

**RISK:** Backend IPC doesn't work as expected  
**MITIGATION:** Test Day 1, fix before continuing

**RISK:** Discovery too slow  
**MITIGATION:** Add timeout + cache, show progress

**RISK:** TOFU confusing  
**MITIGATION:** User test Day 5, iterate based on feedback

**RISK:** No providers for testing  
**MITIGATION:** Publish from same machine, fetch from same machine

**RISK:** Scope creep  
**MITIGATION:** Strict MVP feature list, defer aggressively

---

## 🎯 WEEK 1 GOAL

**By End of Week 1:**
- ✅ Can publish a site
- ✅ Can browse a site
- ✅ Signatures verify
- ✅ TOFU works
- ✅ Basic error handling

**This proves the system works!**

---

## 🎯 WEEK 2 GOAL

**By End of Week 2:**
- ✅ All error states handled
- ✅ Progress indicators polished
- ✅ Cache/offline mode works
- ✅ 5+ people have tested it
- ✅ Critical bugs fixed

**This proves the UX works!**

---

## 🎯 WEEK 3 GOAL (if needed)

**By End of Week 3:**
- ✅ Feedback addressed
- ✅ Documentation complete
- ✅ Polish applied
- ✅ Ready for alpha launch

**This proves it's ready to ship!**

---

## 💡 DEEP INSIGHTS

### Insight #1: The Backend Validates the Vision

**We have:**
- Working signatures
- Working QUIC
- Working cache
- Working discovery
- Working names

**This means:** The hard part is done!

**UI is "just" wiring** - if backend works, UI will work.

### Insight #2: Viewer Before Publisher

**Why Viewer first:**
- Tests backend immediately
- Can use SitesDemo to publish test content
- Validates the entire fetch path
- Lower risk (read-only)

**Why not Publisher first:**
- Higher complexity (4 steps)
- Can't test without Viewer
- Signing is complex
- Key management is tricky

**Decision:** Viewer first = de-risk faster

### Insight #3: TOFU is the UX Challenge

**Users understand:**
- HTTPS lock icon
- Green = secure
- Certificate warnings

**Users DON'T understand:**
- "First-time key"
- "Trust on first use"
- "Fingerprint verification"

**Solution:**
- Use familiar metaphors (lock icon)
- Simple language ("First time seeing this site")
- Clear actions ("Trust & Pin" vs "Reject")
- Don't over-explain PQC internals

### Insight #4: Discovery Latency is Reality

**HTTP/DNS:**
- Instant (cached DNS, CDN)
- Users expect <100ms

**Our reality:**
- Rendezvous discovery: 1-5 seconds
- Provider connection: 500ms-2s
- Block fetching: 1-10 seconds

**Solution:**
- Show progress immediately
- Cache aggressively
- Prefer LAN providers
- Set expectations ("Discovering providers...")

---

## ✅ FINAL STRATEGY

### Build Order (Optimized for Risk)

1. **Day 1:** Minimal Viewer (fetch + render)
2. **Day 2:** Four-words + discovery
3. **Day 3-4:** Basic Publisher
4. **Day 5:** Security (TOFU)
5. **Day 6-7:** Progress + errors
6. **Day 8-9:** Cache + offline
7. **Day 10:** Polish
8. **Day 11-14:** Test + iterate

### Success Metrics

**Week 1:**
- End-to-end publish/browse works
- 1-2 people can use it

**Week 2:**
- Error states covered
- 5+ people can use it

**Week 3:**
- Feedback addressed
- Ready for alpha

### If Things Go Wrong

**Problem:** Can't finish in 2 weeks  
**Solution:** Ship MVP at end of Week 2, polish in Week 3

**Problem:** Critical bugs found  
**Solution:** Focus on fixes, defer polish

**Problem:** UX confusing  
**Solution:** More onboarding, clearer labels

---

**This is a realistic, risk-mitigated plan that will deliver a working product!** 🎯

**Next Step:** Build the minimal Viewer (Day 1 Morning task) to validate backend immediately.
