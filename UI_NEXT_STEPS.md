# Communitas UI - Immediate Next Steps

**Date:** 2025-01-29  
**Status:** Ready to begin UI implementation  
**Timeline:** 2-3 weeks to production MVP

---

## 🚨 CRITICAL: Fix Existing Tauri Commands First

### Issue: `gossip_site_publish` is Broken

**File:** `communitas-desktop/src/gossip_commands.rs:446`

**Current code (BROKEN):**
```rust
let manifest = publisher
    .build_manifest(1, asset_paths) // ← Missing public_key parameter!
    .await
    .map_err(|e| format!("Failed to build manifest: {}", e))?;
```

**New signature (from our changes):**
```rust
pub async fn build_manifest(
    &self,
    public_key: &PublicKey,  // ← We added this!
    version: u64,
    asset_paths: Vec<(String, [u8; 32])>,
) -> Result<SiteManifest>
```

**Fix needed:**
```rust
// Get public key from CoreContext
let public_key = ctx.get_public_key();

let manifest = publisher
    .build_manifest(public_key, 1, asset_paths)
    .await
    .map_err(|e| format!("Failed to build manifest: {}", e))?;

// Sign the manifest
let signing_key = ctx.signing_key();
let mut signed_manifest = manifest;
signed_manifest.sign(signing_key)
    .map_err(|e| format!("Failed to sign manifest: {}", e))?;

// Store signed manifest
publisher.set_manifest(signed_manifest).await
    .map_err(|e| format!("Failed to store manifest: {}", e))?;
```

### Other Commands to Check

1. ✅ `gossip_site_fetch` - Looks good (already calls manifest verification)
2. ✅ `gossip_site_list` - Simple, should work
3. ✅ `gossip_site_providers` - Should work

**Action:** Fix gossip_commands.rs before building UI!

---

## 📅 DAY 1 TASK LIST (Detailed)

### Morning Session (9am - 1pm)

**Task 1.1: Fix Tauri Commands** (1 hour)
- [ ] Fix `gossip_site_publish` to use new signature
- [ ] Add manifest signing
- [ ] Test with existing SitesDemo
- [ ] Verify publishes successfully

**Task 1.2: Create Minimal Viewer Component** (2 hours)
- [ ] Create `src/components/websites/ViewerPage.tsx`
- [ ] Simple address bar (text input)
- [ ] Fetch button
- [ ] Loading state
- [ ] Error display
- [ ] Success: show fetched asset count

**Task 1.3: Test End-to-End** (1 hour)
- [ ] Open SitesDemo
- [ ] Publish a test site
- [ ] Copy SiteId
- [ ] Open ViewerPage
- [ ] Paste SiteId and fetch
- [ ] **VALIDATION: Does it work?**

### Afternoon Session (2pm - 6pm)

**Task 1.4: Add HTML Rendering** (2 hours)
- [ ] Find index.html in fetched assets
- [ ] Create sandboxed div
- [ ] Render HTML with dangerouslySetInnerHTML
- [ ] Basic CSS styling
- [ ] Test with real HTML content

**Task 1.5: Improve Status Indicators** (1 hour)
- [ ] "Fetching manifest..." spinner
- [ ] "Fetching blocks X/Y..." progress
- [ ] Success checkmark
- [ ] Error icon + message

**Task 1.6: Polish & Document** (1 hour)
- [ ] Add TypeScript types
- [ ] Add comments
- [ ] Create README for ViewerPage
- [ ] Screenshot for documentation

### End of Day 1 Checkpoint

**Must Have:**
- [x] Can paste SiteId and fetch
- [x] Can see HTML rendered
- [x] Errors show clearly
- [x] No crashes

**If Not Working:**
- Debug Tauri commands
- Test backend directly
- Check logs
- Don't proceed to Day 2 until this works!

---

## 📅 DAY 2 TASK LIST

### Morning: Four-Words Resolution

**Task 2.1: Implement `names_resolve` Tauri Command** (1.5 hours)
```rust
#[tauri::command]
pub async fn names_resolve(
    state: tauri::State<'_, GossipState>,
    four_words: String,
) -> Result<Option<String>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;
    
    // Access NameRegistry from context
    // (Need to add to GossipContext first!)
    let registry = ctx.name_registry.as_ref()
        .ok_or("NameRegistry not initialized")?;
    
    let site_id = registry.resolve(&four_words).await;
    Ok(site_id.map(|id| hex::encode(id.hash)))
}
```

**Task 2.2: Create FourWordsInput Component** (2 hours)
- Validation against dictionary
- Auto-hyphenation
- Error states
- Clean styling

**Task 2.3: Integrate into Viewer** (0.5 hours)
- Replace text input with FourWordsInput
- Add "Resolve" step
- Show SiteId after resolution

### Afternoon: Provider Discovery

**Task 2.4: Add Provider Display** (2 hours)
- Show provider count
- List providers (simple)
- Show which one is being used

**Task 2.5: Test Discovery** (1 hour)
- Publish site
- Register name
- Browse by four-words
- **VALIDATION: Discovery works?**

**Task 2.6: Error Handling** (1 hour)
- "Name not found"
- "No providers"
- Clear error messages

---

## 📅 DAY 3-4: Publisher Wizard

(Detailed tasks in UI_IMPLEMENTATION_STRATEGY.md)

---

## 🎯 FIRST THING TO BUILD RIGHT NOW

### Component: Minimal ViewerPage

**Location:** `src/components/websites/ViewerPage.tsx`

**Code:** (See UI_IMPLEMENTATION_STRATEGY.md lines 266-353)

**Why This:**
1. Simplest possible
2. Tests backend immediately  
3. Validates entire stack
4. Foundation for full Viewer

**Build This First:** (Next 2 hours)
- Copy code skeleton from strategy doc
- Create file
- Test with SitesDemo
- Verify it works

**Success = Backend validated!**

---

## 📊 REALISTIC TIMELINE

### Week 1: Core Functionality
- **Day 1:** Minimal Viewer ✅
- **Day 2:** Four-words + Discovery ✅
- **Day 3:** Publisher Steps 1-2 ✅
- **Day 4:** Publisher Steps 3-4 + **END-TO-END TEST** ✅
- **Day 5:** Security (TOFU) ✅

**Week 1 Goal:** Working publish → browse flow

### Week 2: Production Polish
- **Day 6-7:** Progress indicators + error states
- **Day 8-9:** Cache + offline mode
- **Day 10:** Integration + polish

**Week 2 Goal:** Production-quality UX

### Week 3: Testing & Launch Prep
- **Day 11-12:** Bug fixes
- **Day 13:** User testing
- **Day 14:** Final polish + docs

**Week 3 Goal:** Ready for alpha launch

---

## ✅ IMMEDIATE ACTION ITEMS

### Right Now (Next 2 Hours)

1. **Fix `gossip_site_publish` command**
   - Add public_key parameter
   - Add manifest signing
   - Test with SitesDemo

2. **Create minimal ViewerPage**
   - Use skeleton from strategy doc
   - Wire up gossip_site_fetch
   - Test fetching

3. **Validate end-to-end**
   - Publish with SitesDemo
   - Fetch with ViewerPage
   - **Prove it works!**

### Today (Next 8 Hours)

4. Add HTML rendering to Viewer
5. Add progress indicators
6. Polish error states
7. Document what works

### This Week

- Complete Viewer (4 days)
- Complete Publisher (2 days)
- **Ship working MVP!**

---

## 🎯 SUCCESS CRITERIA

### Minimum Viable Product

**Can publish a static website:**
- Select folder ✓
- Claim name ✓
- Sign with ML-DSA ✓
- Provider starts on port 5001 ✓

**Can browse websites:**
- Type four-words ✓
- Discover providers ✓
- Fetch content ✓
- Verify signatures ✓
- Render HTML/CSS ✓

**Security visible:**
- TOFU dialog ✓
- Signature verification ✓
- Clear trust indicators ✓

**This is enough for alpha launch!**

---

## 🚀 LET'S BUILD!

**Next immediate step:**  
Create the minimal ViewerPage component and test it works.

**Estimated time:** 2 hours  
**Validation:** Can fetch and render a site  
**This proves everything we built works!**

**Ready to start coding?** 🎯
