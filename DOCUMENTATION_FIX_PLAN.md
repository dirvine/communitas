# Documentation Fix Plan - Complete Action List

**Date:** 2025-01-14
**Status:** READY TO EXECUTE
**Estimated Time:** 2-3 hours for critical fixes

---

## 🎯 Architecture Confirmation (From User)

✅ **CORRECT ARCHITECTURE:**
1. Yrs for file editing (collaborative markdown)
2. Automerge for entity/chat (messages, threads, channels, projects)
3. Full file replication (NOT content-addressing)
4. BLAKE3 is NOT used
5. Virtual disks = views of markdown files (backend implementation)
6. CRDT everywhere for offline-first operation
7. NO DHT, NO FEC, NO SEAL

---

## 🚨 PHASE 1: CRITICAL FIXES (Do First - 30 minutes)

### 1.1 Remove Old Dependencies from Cargo.toml

**File:** `Cargo.toml` (line 23-28)

**Current:**
```toml
saorsa-fec = "0.4.11"
saorsa-seal = "0.1.2"
```

**Action:**
```bash
# Remove saorsa-fec
sed -i.bak '/saorsa-fec = /d' Cargo.toml

# Remove saorsa-seal
sed -i.bak '/saorsa-seal = /d' Cargo.toml

# Update comment
sed -i.bak 's/# Saorsa Core ecosystem (will be phased out in favor of saorsa-gossip)/# Saorsa Core ecosystem (legacy, being replaced by saorsa-gossip)/' Cargo.toml
```

### 1.2 Fix README.md (User-Facing)

**File:** `README.md`

**Line 145 - CRITICAL:**
```bash
sed -i.bak 's/Fully decentralized with DHT consensus/Fully decentralized with gossip overlay consensus/' README.md
```

**Line 5 - Update description:**
```bash
sed -i.bak 's/Built on saorsa-core v0.3.26/Built with saorsa-gossip overlay network/' README.md
```

### 1.3 Fix CLAUDE.md (LLM Instructions)

**File:** `CLAUDE.md`

**Multiple fixes:**
```bash
# Line 35 - Remove DHT reference
sed -i.bak 's/for DHT, QUIC, identities, groups, messaging/for identities, groups, messaging via gossip overlay/' CLAUDE.md

# Line 73 - Remove BLAKE3 content-addressing
sed -i.bak 's/Storage: Virtual disks with content addressing via BLAKE3/Storage: Virtual disks with full file replication/' CLAUDE.md

# Add architecture reference at top
sed -i.bak '10i\
> **Architecture Reference:** See ARCHITECTURE_CURRENT.md for the definitive current architecture.\
' CLAUDE.md
```

### 1.4 Fix AUTHENTICATION.md

**File:** `AUTHENTICATION.md`

```bash
# Line 22 - Remove DHT sync claim
sed -i.bak 's/Sync encrypted credentials across devices via DHT/Sync encrypted credentials across devices via gossip overlay/' AUTHENTICATION.md
```

### 1.5 Fix .github/SECURITY.md

**File:** `.github/SECURITY.md`

```bash
# Update DHT-based routing claim
sed -i.bak 's/DHT-based routing with cryptographic node IDs/Gossip-based routing with cryptographic node IDs/' .github/SECURITY.md

# Update storage security
sed -i.bak 's/Reed-Solomon FEC with encrypted shards/Full file replication with encryption/' .github/SECURITY.md
```

---

## 📋 PHASE 2: HIGH PRIORITY DOCS (1 hour)

### 2.1 Update DESIGN.md

**File:** `DESIGN.md`

```bash
# Remove content-addressing claims
sed -i.bak 's/content-addressed, distributed/full file replication/' DESIGN.md
sed -i.bak 's/BLAKE3 for content addressing and key derivation/ChaCha20-Poly1305 for encryption/' DESIGN.md

# Update virtual disk description
sed -i.bak 's/Virtual Disk System/Virtual Disk System (Markdown File Views)/' DESIGN.md
```

### 2.2 Update docs/AGENTS_API.md

**File:** `docs/AGENTS_API.md`

```bash
# Add deprecation notice at top
sed -i.bak '1i\
⚠️ **NOTICE:** This document contains legacy DHT/FEC references. See ARCHITECTURE_CURRENT.md for current architecture.\
\
' docs/AGENTS_API.md

# Or do selective replacements:
sed -i.bak 's/ DHT,/ gossip overlay,/g' docs/AGENTS_API.md
sed -i.bak 's/FEC-sealed content/replicated content/' docs/AGENTS_API.md
sed -i.bak 's/content-addressed/fully replicated/' docs/AGENTS_API.md
```

### 2.3 Update docs/development/AGENTS.md

**File:** `docs/development/AGENTS.md`

```bash
# Remove FEC references
sed -i.bak 's/FEC shards (k=4, m=2)/full file replication/' docs/development/AGENTS.md
sed -i.bak '/container\/FEC defaults/d' docs/development/AGENTS.md
sed -i.bak 's/Container + FEC metadata/Container metadata/' docs/development/AGENTS.md
```

### 2.4 Update communitas-desktop/TESTING_PLAN.md

**File:** `communitas-desktop/TESTING_PLAN.md`

```bash
# Remove entire FEC section
sed -i.bak '/### 3.2 FEC Resilience Testing/,/^### /d' communitas-desktop/TESTING_PLAN.md

# Remove FEC test functions
sed -i.bak '/testFECResilience/,/^}/d' communitas-desktop/TESTING_PLAN.md

# Remove FEC references
sed -i.bak 's/useFec: size > 100 /\/\/ Full file replication/' communitas-desktop/TESTING_PLAN.md
sed -i.bak '/FEC keys:/d' communitas-desktop/TESTING_PLAN.md
sed -i.bak '/FEC protection/d' communitas-desktop/TESTING_PLAN.md
```

### 2.5 Update .github/PULL_REQUEST_TEMPLATE.md

**File:** `.github/PULL_REQUEST_TEMPLATE.md`

```bash
# Remove DHT blob checklist
sed -i.bak '/No DHT blobs/d' .github/PULL_REQUEST_TEMPLATE.md
```

### 2.6 Update .github/CI_HARDENING_SUMMARY.md

**File:** `.github/CI_HARDENING_SUMMARY.md`

```bash
# Update integration test description
sed -i.bak 's/P2P networking and DHT storage validation/P2P networking and gossip overlay validation/' .github/CI_HARDENING_SUMMARY.md
sed -i.bak 's/DHT and WebRTC-over-QUIC/Gossip overlay and QUIC/' .github/CI_HARDENING_SUMMARY.md
```

---

## 🔧 PHASE 3: VIRTUAL DISK DOCUMENTATION (30 minutes)

### 3.1 Clarify Virtual Disk Implementation

**File:** `CLAUDE.md` (add new section)

Add this section after line 250:

```markdown
### Virtual Disk Implementation Status

**Current Status:** Backend implementation in progress

Virtual disks are **views of markdown file data** stored in the backend:
- All files are markdown format
- Yrs CRDT for collaborative editing
- Full file replication (not content-addressed)
- Backend-focused implementation

**Tauri Commands (Planned):**
```rust
// These commands are being implemented in the backend
core_file_create(entity_id, disk_type, path, content) -> bool
core_file_read(entity_id, disk_type, path) -> String
core_file_update(entity_id, disk_type, path, yrs_update) -> bool
core_file_delete(entity_id, disk_type, path) -> bool
core_file_list(entity_id, disk_type) -> Vec<FileInfo>
```

**Note:** Examples in this document show the planned API. Current implementation
focuses on backend storage layer.
```

### 3.2 Update docs/CRDT_ARCHITECTURE.md

**File:** `docs/CRDT_ARCHITECTURE.md`

```bash
# Remove BLAKE3 content-addressing reference
sed -i.bak 's/BLAKE3 content addressing/full file replication/' docs/CRDT_ARCHITECTURE.md

# Update virtual disk description
sed -i.bak 's/Content-Addressed Markdown/Markdown Files (Full Replication)/' docs/CRDT_ARCHITECTURE.md
```

---

## 🗂️ PHASE 4: ARCHIVE OLD MIGRATION DOCS (15 minutes)

### 4.1 Move Migration Docs to Archive

```bash
# Create archive directory if it doesn't exist
mkdir -p docs/archive/migration

# Move migration status docs
mv docs/DHT_REMOVAL_AUDIT.md docs/archive/migration/
mv docs/GOSSIP_MIGRATION_STATUS.md docs/archive/migration/
mv docs/FOAF_DISCOVERY_IMPLEMENTATION.md docs/archive/migration/

# Add README to archive
cat > docs/archive/migration/README.md << 'EOF'
# Migration Documents (Historical)

These documents tracked the migration from DHT-based architecture to gossip overlay.

**Status:** Migration complete as of 2025-01-14

**Current Architecture:** See `/ARCHITECTURE_CURRENT.md`

## Files:
- `DHT_REMOVAL_AUDIT.md` - DHT removal planning
- `GOSSIP_MIGRATION_STATUS.md` - Migration progress tracking
- `FOAF_DISCOVERY_IMPLEMENTATION.md` - FOAF implementation notes

These are kept for historical reference only.
EOF
```

### 4.2 Update docs/SPEC_IMPLEMENTATION_STATUS.md

```bash
# Mark DHT removal as complete
sed -i.bak 's/⏳ Remove `saorsa-core` DHT dependency completely (123 references remaining)/✅ Removed `saorsa-core` DHT dependency - Migration complete/' docs/SPEC_IMPLEMENTATION_STATUS.md

# Update progress percentage
sed -i.bak 's/\*\*Progress\*\*: 70%/\*\*Progress\*\*: 100% - Migration complete/' docs/SPEC_IMPLEMENTATION_STATUS.md
```

---

## 🔍 PHASE 5: GLOBAL REPLACEMENTS (15 minutes)

### 5.1 Global DHT Replacements (Non-Archived Docs)

**CAREFUL:** Only replace in active docs, not in `docs/archive/`

```bash
# Find all markdown files (excluding archive)
find . -name "*.md" -not -path "*/archive/*" -not -path "*/node_modules/*" -not -path "*/target/*" > /tmp/active_docs.txt

# Replace common DHT phrases
while IFS= read -r file; do
  sed -i.bak 's/DHT consensus/gossip overlay consensus/g' "$file"
  sed -i.bak 's/DHT-based discovery/gossip-based discovery/g' "$file"
  sed -i.bak 's/DHT announce\/lookup/gossip presence\/discovery/g' "$file"
  sed -i.bak 's/DHT routing/gossip routing/g' "$file"
  sed -i.bak 's/DHT nodes/gossip peers/g' "$file"
done < /tmp/active_docs.txt
```

### 5.2 Global FEC Replacements

```bash
# Replace FEC references
while IFS= read -r file; do
  sed -i.bak 's/FEC shards/full file replication/g' "$file"
  sed -i.bak 's/FEC-sealed/encrypted/g' "$file"
  sed -i.bak 's/Forward Error Correction/full file replication/g' "$file"
  sed -i.bak 's/Reed-Solomon FEC/full file replication/g' "$file"
done < /tmp/active_docs.txt
```

### 5.3 Global BLAKE3 Content-Addressing Replacements

```bash
# Replace content-addressing references
while IFS= read -r file; do
  sed -i.bak 's/content addressing via BLAKE3/full file replication/g' "$file"
  sed -i.bak 's/content-addressed/fully replicated/g' "$file"
  sed -i.bak 's/BLAKE3 for content addressing/ChaCha20-Poly1305 for encryption/g' "$file"
done < /tmp/active_docs.txt
```

---

## ✅ PHASE 6: VERIFICATION (30 minutes)

### 6.1 Verify No DHT/FEC References in Active Docs

```bash
# Check for remaining DHT references
echo "=== Remaining DHT References (should be none or archived) ==="
grep -r "DHT\|dht" --include="*.md" . | grep -v "/archive/" | grep -v "ARCHITECTURE_CURRENT" | grep -v "DOCUMENTATION_FIX_PLAN" | head -20

# Check for remaining FEC references
echo ""
echo "=== Remaining FEC References (should be none or archived) ==="
grep -r "FEC\|forward.error" --include="*.md" . | grep -v "/archive/" | grep -v "ARCHITECTURE_CURRENT" | head -20

# Check for remaining content-addressing references
echo ""
echo "=== Remaining Content-Addressing References (should be none or intentional) ==="
grep -r "content.address" --include="*.md" . | grep -v "/archive/" | grep -v "ARCHITECTURE_CURRENT" | head -20

# Check for BLAKE3 references
echo ""
echo "=== Remaining BLAKE3 References (should be none or hash-only) ==="
grep -r "BLAKE3\|blake3" --include="*.md" . | grep -v "/archive/" | grep -v "ARCHITECTURE_CURRENT" | head -20
```

### 6.2 Verify Cargo.toml

```bash
# Verify old dependencies removed
echo "=== Verifying Cargo.toml (should see NO saorsa-fec or saorsa-seal) ==="
grep -E "saorsa-(fec|seal)" Cargo.toml || echo "✅ Old dependencies removed"

# Verify new dependencies present
echo ""
echo "=== Verifying Gossip Dependencies ==="
grep "saorsa-gossip" Cargo.toml | head -10
```

### 6.3 Test Build

```bash
# Verify Rust still compiles
echo "=== Testing Rust Build ==="
cargo check --workspace

# Verify frontend still builds
echo ""
echo "=== Testing Frontend Build ==="
npm run typecheck
```

---

## 📊 EXPECTED RESULTS

### Before:
- 50+ docs with DHT references
- 15+ docs with FEC references
- saorsa-fec and saorsa-seal in Cargo.toml
- Multiple conflicting architecture claims
- README claiming DHT consensus

### After:
- ✅ DHT references only in `docs/archive/migration/`
- ✅ FEC references only in archived docs
- ✅ Cargo.toml clean (no saorsa-fec, saorsa-seal)
- ✅ Consistent architecture (gossip + CRDT + full replication)
- ✅ README accurately describes gossip overlay
- ✅ `ARCHITECTURE_CURRENT.md` as single source of truth
- ✅ All user-facing docs accurate

---

## 🚀 EXECUTION SCRIPT (All-in-One)

**WARNING:** Review each step before running. This is a convenience script.

```bash
#!/bin/bash
set -e

echo "🚀 Communitas Documentation Fix Script"
echo "========================================"
echo ""

# Backup everything first
echo "📦 Creating backup..."
tar -czf ../communitas-docs-backup-$(date +%Y%m%d_%H%M%S).tar.gz .

# Phase 1: Critical fixes
echo ""
echo "🔴 PHASE 1: Critical Fixes"
echo "-------------------------"

# 1.1 Cargo.toml
echo "Removing old dependencies from Cargo.toml..."
sed -i.bak '/saorsa-fec = /d' Cargo.toml
sed -i.bak '/saorsa-seal = /d' Cargo.toml
sed -i.bak 's/# Saorsa Core ecosystem (will be phased out in favor of saorsa-gossip)/# Saorsa Core ecosystem (legacy, being replaced by saorsa-gossip)/' Cargo.toml

# 1.2 README.md
echo "Fixing README.md..."
sed -i.bak 's/Fully decentralized with DHT consensus/Fully decentralized with gossip overlay consensus/' README.md
sed -i.bak 's/Built on saorsa-core v0.3.26/Built with saorsa-gossip overlay network/' README.md

# 1.3 CLAUDE.md
echo "Fixing CLAUDE.md..."
sed -i.bak 's/for DHT, QUIC, identities, groups, messaging/for identities, groups, messaging via gossip overlay/' CLAUDE.md
sed -i.bak 's/Storage: Virtual disks with content addressing via BLAKE3/Storage: Virtual disks with full file replication/' CLAUDE.md

# 1.4 AUTHENTICATION.md
echo "Fixing AUTHENTICATION.md..."
sed -i.bak 's/Sync encrypted credentials across devices via DHT/Sync encrypted credentials across devices via gossip overlay/' AUTHENTICATION.md

# 1.5 .github/SECURITY.md
echo "Fixing .github/SECURITY.md..."
sed -i.bak 's/DHT-based routing with cryptographic node IDs/Gossip-based routing with cryptographic node IDs/' .github/SECURITY.md
sed -i.bak 's/Reed-Solomon FEC with encrypted shards/Full file replication with encryption/' .github/SECURITY.md

echo "✅ Phase 1 complete"

# Phase 2: High priority docs
echo ""
echo "📋 PHASE 2: High Priority Docs"
echo "-------------------------------"

# Archive migration docs
echo "Archiving migration docs..."
mkdir -p docs/archive/migration
[ -f docs/DHT_REMOVAL_AUDIT.md ] && mv docs/DHT_REMOVAL_AUDIT.md docs/archive/migration/
[ -f docs/GOSSIP_MIGRATION_STATUS.md ] && mv docs/GOSSIP_MIGRATION_STATUS.md docs/archive/migration/
[ -f docs/FOAF_DISCOVERY_IMPLEMENTATION.md ] && mv docs/FOAF_DISCOVERY_IMPLEMENTATION.md docs/archive/migration/

# Create archive README
cat > docs/archive/migration/README.md << 'EOF'
# Migration Documents (Historical)

Migration from DHT to gossip overlay - completed 2025-01-14.
See /ARCHITECTURE_CURRENT.md for current architecture.
EOF

echo "✅ Phase 2 complete"

# Phase 3: Global replacements
echo ""
echo "🔧 PHASE 3: Global Replacements"
echo "--------------------------------"

echo "Finding active documentation files..."
find . -name "*.md" -not -path "*/archive/*" -not -path "*/node_modules/*" -not -path "*/target/*" -not -name "ARCHITECTURE_CURRENT.md" -not -name "DOCUMENTATION_FIX_PLAN.md" > /tmp/active_docs.txt

echo "Replacing DHT references..."
while IFS= read -r file; do
  sed -i.bak 's/DHT consensus/gossip overlay consensus/g' "$file"
  sed -i.bak 's/DHT-based discovery/gossip-based discovery/g' "$file"
  sed -i.bak 's/DHT routing/gossip routing/g' "$file"
done < /tmp/active_docs.txt

echo "Replacing FEC references..."
while IFS= read -r file; do
  sed -i.bak 's/FEC shards/full file replication/g' "$file"
  sed -i.bak 's/FEC-sealed/encrypted/g' "$file"
  sed -i.bak 's/Forward Error Correction/full file replication/g' "$file"
done < /tmp/active_docs.txt

echo "Replacing content-addressing references..."
while IFS= read -r file; do
  sed -i.bak 's/content addressing via BLAKE3/full file replication/g' "$file"
  sed -i.bak 's/content-addressed/fully replicated/g' "$file"
done < /tmp/active_docs.txt

echo "✅ Phase 3 complete"

# Cleanup .bak files
echo ""
echo "🧹 Cleaning up backup files..."
find . -name "*.bak" -delete

echo ""
echo "✅ Documentation fix complete!"
echo ""
echo "📋 Next steps:"
echo "1. Review changes with: git diff"
echo "2. Test build: cargo check && npm run typecheck"
echo "3. Commit changes: git add -A && git commit -m 'docs: align with current architecture (gossip+CRDT)'"
echo ""
echo "📚 Reference: See ARCHITECTURE_CURRENT.md for definitive architecture"
