# FEC (Forward Error Correction) Removal Plan

**Date**: 2025-10-11
**Status**: Analysis Complete - Ready for Execution

## Executive Summary

Comprehensive search found **18 Rust files with FEC references**, **14 files with shard references**, and **37 documentation files** mentioning FEC/sharding. This plan systematically removes all FEC/Reed-Solomon/sharding code to align with the current architecture design which does NOT use FEC.

---

## Files to DELETE Entirely

### Core FEC Implementation Files
1. **communitas-core/src/encrypted_storage/fec_storage.rs** (174 lines)
   - Stub FEC storage implementation
   - Contains: `FecStorage`, `FecMetadata`, `FecStats`
   - Status: Entire file is FEC-only code

2. **communitas-core/src/storage/reed_solomon_manager.rs** (659 lines)
   - Complete Reed-Solomon implementation with saorsa-fec integration
   - Contains: `EnhancedReedSolomonManager`, `Shard`, `ShardType`, `ReedSolomonConfig`
   - Contains: `ShardDistributionPlan`, `ReconstructionStatus`, `IntegrityStatus`
   - Status: Entire file is FEC-only code

3. **communitas-core/src/reed_solomon_manager.rs** (if exists)
   - Possible duplicate or older version
   - Status: DELETE if exists

---

## Files to MODIFY (Remove FEC References)

### Module Exports

1. **communitas-core/src/encrypted_storage/mod.rs**
   - Line 40: `pub mod fec_storage;` - DELETE
   - Lines 7-36: Documentation describes FEC layer - REWRITE
   - Action: Remove FEC module export and update architecture documentation

2. **communitas-core/src/storage/mod.rs**
   - Line 9: Already commented out: `// pub mod reed_solomon_manager;` ✅
   - Line 14: Already commented out: `// pub use reed_solomon_manager::*;` ✅
   - Status: Already correctly disabled

3. **communitas-core/src/mod.rs**
   - Search for any `reed_solomon` or `fec` exports
   - Action: Remove if found

### Core Implementation Files

4. **communitas-core/src/core_context.rs**
   - Search for FEC field references
   - Action: Remove any FEC-related struct fields or initialization

5. **communitas-core/src/storage/local_storage.rs**
   - Contains FEC references (8 matches found)
   - Action: Review and remove FEC-related code paths

6. **communitas-core/src/storage/metrics.rs**
   - Contains shard references
   - Action: Remove shard-related metrics

7. **communitas-core/src/encrypted_storage/vault.rs**
   - Contains FEC/shard references
   - Action: Remove FEC integration from vault

8. **communitas-core/src/local_storage.rs**
   - Duplicate of storage/local_storage.rs?
   - Action: Check and clean

9. **communitas-core/src/metrics.rs**
   - Duplicate of storage/metrics.rs?
   - Action: Check and clean

### Gossip Module Files

10. **communitas-core/src/gossip/peer_cache.rs**
    - Contains FEC references
    - Action: Review and remove if related to FEC sharding

11. **communitas-core/src/gossip/rendezvous.rs**
    - Contains shard references
    - Action: Check if "shard" refers to FEC or something else

12. **communitas-core/src/gossip/sites.rs**
    - Contains shard references
    - Action: Check if "shard" refers to FEC or routing shards

### Desktop Application Files

13. **communitas-desktop/src/entity_storage.rs**
    - Contains FEC and shard references
    - Action: Remove FEC integration

14. **communitas-desktop/src/sync.rs**
    - Contains FEC and shard references
    - Action: Remove FEC sync logic

15. **communitas-desktop/src/core_storage.rs**
    - Contains FEC references
    - Action: Remove FEC integration

16. **communitas-desktop/src/main.rs**
    - Contains FEC references
    - Action: Remove FEC initialization

### Headless Node Files

17. **communitas-headless/src/main.rs**
    - Contains FEC references
    - Action: Remove FEC initialization

### Container Crate

18. **crates/communitas-container/src/lib.rs**
    - Contains FEC and shard references
    - Action: Review and remove FEC code

### Test Files

19. **communitas-core/tests/saorsa_p2p_immediate_send.rs**
    - Contains FEC references
    - Action: Remove FEC test cases

20. **communitas-core/tests/ant_quic_comprehensive.rs**
    - Contains FEC references
    - Action: Remove FEC test cases

21. **communitas-desktop/tests/tauri_commands_test.rs**
    - Contains FEC references
    - Action: Remove FEC test cases

---

## Cargo.toml Dependencies to Check

Search for these FEC-related dependencies and REMOVE if found:

- `saorsa-fec` - FEC codec library
- `saorsa-seal` - Sealing with FEC (may need to keep, but remove FEC features)
- `reed-solomon` - Reed-Solomon implementation
- `reed-solomon-erasure` - Alternative RS implementation
- Any other erasure coding libraries

### Files to Check:
- communitas-core/Cargo.toml
- communitas-desktop/Cargo.toml
- communitas-headless/Cargo.toml
- Cargo.toml (workspace root)

---

## Documentation Files to UPDATE (37 files)

### Priority Documentation (High Impact)

1. **CLAUDE.md** - Line 24 mentions "Virtual disks with FEC"
2. **DESIGN.md** - Architecture description includes FEC
3. **docs/AGENTS_API.md** - API surface documentation
4. **docs/RESTORATION_SUMMARY.md** - Restoration documentation
5. **ARCHITECTURE.md** - Core architecture document

### Other Documentation Files

6. STORYBOARD.md
7. communitas-desktop/TESTING_PLAN.md
8. QUALITY_VERIFICATION.md
9. MANUAL_TEST_PLAN.md
10. docs/SPRINT_3_3_STATUS.md
11. docs/SPRINT_3_2_STATUS.md
12. docs/RC1B_MIGRATION_PROGRESS.md
13. docs/CRDT_MULTI_PEER_TEST.md
14. docs/SAORSA_GOSSIP_ARCHITECTURE.md
15. docs/SAORSA_SITES_COMPLETE.md
16. docs/archive/SPEC2.md
17. docs/GOSSIP_TAURI_INTEGRATION_SUMMARY.md
18. docs/SESSION_SUMMARY.md
19. docs/FOAF_DISCOVERY_IMPLEMENTATION.md
20. docs/GOSSIP_MIGRATION_STATUS.md
21. docs/CRDT_QUICK_REFERENCE.md
22. docs/CRDT_INTEGRATION_STATUS.md
23. README_TESTING.md
24. VISUAL_TEST_REPORT.md
25. MODERN_SHELL_TEST_SUMMARY.md
26. TEST_MODERN_SHELL.md
27. docs/COMMUNITAS_UX_SPEC.md
28. finalise/DEPLOY_TESTNET.md
29. docs/development/AGENTS.md
30. mcp-desktop-control/README.md
31. P2P_CONNECTION_COMPLETE.md
32. README.md
33. FINAL_TEST_REPORT.md
34. BROWSER_TEST_REPORT.md
35. .github/CI_HARDENING_SUMMARY.md
36. .github/SECURITY.md
37. communitas-core/P2P_CONNECTION_ISSUE_ROOT_CAUSE_2025-10-02.md

---

## Execution Plan

### Phase 1: File Deletion (Safe - these are entirely FEC)
1. Delete `communitas-core/src/encrypted_storage/fec_storage.rs`
2. Delete `communitas-core/src/storage/reed_solomon_manager.rs`
3. Check and delete `communitas-core/src/reed_solomon_manager.rs` if exists

### Phase 2: Module Export Cleanup
1. Edit `communitas-core/src/encrypted_storage/mod.rs` - remove fec_storage module
2. Update architecture documentation in mod.rs
3. Verify `communitas-core/src/storage/mod.rs` (already clean)
4. Check `communitas-core/src/mod.rs` for exports

### Phase 3: Dependency Removal
1. Search all Cargo.toml files for `saorsa-fec`
2. Search all Cargo.toml files for `saorsa-seal` FEC features
3. Search all Cargo.toml files for `reed-solomon*`
4. Remove found dependencies
5. Run `cargo update` to update lock files

### Phase 4: Code Reference Cleanup
For each file in "Files to MODIFY" section:
1. Read file to understand FEC usage
2. Remove FEC-related:
   - Struct fields
   - Function parameters
   - Function calls
   - Import statements
   - Type definitions
3. Simplify code paths that used FEC
4. Update related tests

### Phase 5: Documentation Updates
1. Update CLAUDE.md - remove "FEC" from Virtual disks description
2. Update DESIGN.md - remove FEC architecture layer
3. Update ARCHITECTURE.md - remove FEC references
4. Update other high-priority docs
5. Batch update remaining docs with search/replace

### Phase 6: Verification
1. Run `cargo check --all-features --all-targets`
2. Run `cargo clippy --all-features --all-targets`
3. Run `cargo test --all-features`
4. Fix any compilation errors
5. Verify all tests pass

### Phase 7: Commit
1. Create comprehensive commit message
2. Include this removal plan in commit
3. Push to feature branch

---

## Verification Checklist

After completion, verify:

- [ ] No files contain `FEC` or `fec` (except in comments explaining removal)
- [ ] No files contain `shard` or `Shard` types related to FEC
- [ ] No files contain `reed.solomon` or `reed_solomon`
- [ ] No files contain `erasure` coding references
- [ ] All Cargo.toml files clean of FEC dependencies
- [ ] Documentation accurately reflects non-FEC architecture
- [ ] Code compiles with zero errors and zero warnings
- [ ] All tests pass
- [ ] No panic!/unwrap/expect in production code (per quality standards)

---

## Risk Assessment

**Low Risk:**
- File deletions (fec_storage.rs, reed_solomon_manager.rs are not used)
- Module export removals (already commented out in storage/mod.rs)
- Dependency removal (no active FEC usage)

**Medium Risk:**
- Code reference cleanup in core files (need careful review)
- Gossip module changes (verify "shard" doesn't mean something else)
- Test file updates (may need rewriting not just deletion)

**High Risk:**
- None identified - FEC appears to be unused/stub code

---

## Estimated Effort

- Phase 1-3: 30 minutes (straightforward deletions and exports)
- Phase 4: 2-3 hours (careful code review and cleanup)
- Phase 5: 1 hour (documentation updates)
- Phase 6: 30 minutes (verification and fixes)
- **Total**: ~4-5 hours for comprehensive removal

---

## Next Steps

Ready to proceed with execution. Recommend starting with Phase 1 (safe file deletions) and working through phases sequentially with verification at each step.
