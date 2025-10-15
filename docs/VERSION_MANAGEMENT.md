# Version Management Guide

## Overview

Communitas uses **Semantic Versioning (SemVer)** with automated tools for version bumping, changelog generation, and release creation. This guide explains the complete version management workflow.

## Versioning Strategy

### Semantic Versioning (SemVer)

Version format: `MAJOR.MINOR.PATCH[-PRERELEASE]`

- **MAJOR**: Breaking changes, incompatible API changes
- **MINOR**: New features, backward-compatible functionality
- **PATCH**: Bug fixes, backward-compatible patches
- **PRERELEASE**: Optional suffix like `-beta.1`, `-rc.1`

**Examples**:
- `0.1.17` → `0.1.18` (patch bump)
- `0.1.17` → `0.2.0` (minor bump)
- `0.1.17` → `1.0.0` (major bump)
- `0.1.17` → `0.1.18-beta.1` (prerelease)

### Version Synchronization

Versions are synchronized across multiple files:
1. `Cargo.toml` - Workspace version (source of truth)
2. `package.json` - NPM package version
3. `communitas-desktop/Cargo.toml` - Desktop crate version
4. `communitas-desktop/tauri.conf.json` - Tauri app version

## Automated Version Bumping

### Using bump-version.sh

The `scripts/bump-version.sh` script automates version updates across all project files.

#### Basic Usage

```bash
# Patch bump (0.1.17 → 0.1.18)
./scripts/bump-version.sh patch

# Minor bump (0.1.17 → 0.2.0)
./scripts/bump-version.sh minor

# Major bump (0.1.17 → 1.0.0)
./scripts/bump-version.sh major

# Prerelease bump (0.1.17 → 0.1.18-beta.1)
./scripts/bump-version.sh prerelease
```

#### Dry-Run Mode

Preview changes without modifying files:

```bash
./scripts/bump-version.sh patch --dry-run
```

**Output Example**:
```
🔧 Communitas Version Bump Tool

📌 Current version: 0.1.17
🎯 New version:     0.1.18

🔍 DRY RUN MODE - No changes will be made

📝 Files to update:
  ✓ Cargo.toml
  ✓ package.json
  ✓ communitas-desktop/Cargo.toml
  ✓ communitas-desktop/tauri.conf.json

✓ Dry run complete - no files were modified
```

#### What It Does

1. **Reads** current version from workspace `Cargo.toml`
2. **Calculates** new version using SemVer rules
3. **Updates** all version fields in:
   - `Cargo.toml` (line: `version = "X.Y.Z"`)
   - `package.json` (field: `"version": "X.Y.Z"`)
   - `communitas-desktop/Cargo.toml`
   - `communitas-desktop/tauri.conf.json`
4. **Creates** backup files during sed operations (`.bak` files are auto-deleted)

#### Safety Features

- ✅ Interactive confirmation before applying changes
- ✅ Dry-run mode for preview
- ✅ Color-coded output for clarity
- ✅ Automatic backup file cleanup
- ✅ Pattern validation before sed operations

## Automated Release Creation

### Using create-release.sh

The `scripts/create-release.sh` script automates the complete release workflow including changelog generation, tagging, and pushing.

#### Basic Usage

```bash
# Create release v0.2.0
./scripts/create-release.sh 0.2.0

# Create release with type hint
./scripts/create-release.sh 0.2.0 minor
```

**Note**: Version should match what you just bumped with `bump-version.sh`

#### Interactive Workflow

The script provides step-by-step confirmations:

1. **Pre-flight Checks**:
   - ✅ Verifies working directory is clean (no uncommitted changes)
   - ✅ Warns if not on main branch (allows override)
   - ✅ Shows last git tag for reference

2. **Changelog Generation**:
   - Extracts commits since last tag
   - Categorizes by type: `feat:`, `fix:`, `docs:`, `chore:`
   - Generates formatted changelog entry
   - Shows preview for review

3. **Release Creation**:
   - Updates `CHANGELOG.md` with new entry
   - Commits changelog changes
   - Creates annotated git tag
   - Optionally pushes to remote

#### Changelog Categories

Commits are automatically categorized:

- **Added** (Features): `feat:` commits
  ```
  feat: add four-word identity generation
  feat: implement P2P auto-connection
  ```

- **Fixed** (Bug Fixes): `fix:` commits
  ```
  fix: resolve network connection timeout
  fix: handle DHT initialization errors
  ```

- **Documentation**: `docs:` commits
  ```
  docs: update API documentation
  docs: add version management guide
  ```

- **Internal** (Maintenance): `chore:`, `refactor:`, `style:` commits
  ```
  chore: bump dependencies
  refactor: improve error handling
  ```

#### Example Output

```
🚀 Creating Release v0.2.0

📝 Updating CHANGELOG.md...
Last tag: v0.1.17

📋 Release Notes:
## [0.2.0] - 2025-10-15

### Added
- feat: add version management automation
- feat: implement release creation script

### Fixed
- fix: resolve version synchronization issues

✓ CHANGELOG.md updated

Continue with release creation? (y/N)
```

#### What It Does

1. **Validates** prerequisites (clean working dir, correct branch)
2. **Extracts** commits since last tag using `git log`
3. **Categorizes** commits by conventional commit prefixes
4. **Generates** changelog entry with date and categories
5. **Inserts** entry into `CHANGELOG.md` (before first existing release)
6. **Commits** changelog changes: `chore: update CHANGELOG for v0.2.0`
7. **Creates** annotated git tag: `git tag -a v0.2.0 -m "Release v0.2.0..."`
8. **Pushes** to remote (with confirmation):
   - Push branch: `git push origin main`
   - Push tag: `git push origin v0.2.0`

#### Safety Features

- ✅ Verifies clean working directory
- ✅ Warns if not on main branch
- ✅ Shows preview before committing
- ✅ Interactive confirmations at each step
- ✅ Optional push (can create locally without pushing)
- ✅ Provides undo instructions if needed

## Complete Release Workflow

### Step-by-Step Process

#### 1. Prepare Your Changes

```bash
# Ensure all changes are committed
git status

# Make sure you're on main branch
git checkout main

# Pull latest changes
git pull origin main
```

#### 2. Bump Version

```bash
# Preview version bump
./scripts/bump-version.sh patch --dry-run

# Apply version bump
./scripts/bump-version.sh patch

# Review changes
git diff
```

**Files Changed**:
- `Cargo.toml`
- `package.json`
- `communitas-desktop/Cargo.toml`
- `communitas-desktop/tauri.conf.json`

#### 3. Commit Version Bump

```bash
# Stage version changes
git add -A

# Commit with conventional format
git commit -m "chore: bump version to 0.1.18"
```

#### 4. Create Release

```bash
# Run release script
./scripts/create-release.sh 0.1.18

# Follow interactive prompts:
# - Review generated changelog
# - Confirm release creation
# - Confirm push to remote
```

#### 5. Verify GitHub Actions

After pushing the tag:

1. **Navigate to GitHub Actions**: https://github.com/dirvine/communitas/actions
2. **Monitor** the release workflow
3. **Verify** build completes successfully
4. **Check** draft release is created

#### 6. Publish Release

1. Go to https://github.com/dirvine/communitas/releases
2. Find the draft release for your tag
3. Review auto-generated release notes
4. Add additional notes if needed
5. Click **Publish release**

### Quick Release (Combined)

For experienced users, you can combine steps:

```bash
# 1. Bump version and commit
./scripts/bump-version.sh patch
git add -A && git commit -m "chore: bump version to $(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')"

# 2. Create and push release
./scripts/create-release.sh $(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
```

## Version Display in UI

### Backend (Tauri)

Version is available via Tauri configuration:

```rust
// src-tauri/src/lib.rs
use tauri::Manager;

#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> String {
    app.config().version.clone().unwrap_or_else(|| "unknown".to_string())
}
```

### Frontend (React)

Display version in UI components:

```typescript
// src/components/VersionDisplay.tsx
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export function VersionDisplay() {
  const [version, setVersion] = useState<string>('');

  useEffect(() => {
    invoke<string>('get_app_version').then(setVersion);
  }, []);

  return <span>v{version}</span>;
}
```

Add to header, settings, or about dialog:

```typescript
// src/components/Header.tsx
import { VersionDisplay } from './VersionDisplay';

export function Header() {
  return (
    <header>
      <h1>Communitas</h1>
      <VersionDisplay />
    </header>
  );
}
```

## Conventional Commits

### Commit Message Format

```
<type>: <description>

[optional body]

[optional footer]
```

### Commit Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation only
- **style**: Formatting, missing semicolons, etc.
- **refactor**: Code restructuring without behavior change
- **test**: Adding or updating tests
- **chore**: Maintenance tasks (deps, build, etc.)

### Examples

```bash
# Feature
git commit -m "feat: add four-word identity generation"

# Bug fix
git commit -m "fix: resolve network connection timeout"

# Documentation
git commit -m "docs: update version management guide"

# Chore
git commit -m "chore: bump dependencies to latest versions"

# Breaking change
git commit -m "feat!: redesign authentication API

BREAKING CHANGE: Auth tokens now use JWT format"
```

## Troubleshooting

### Issue: "Working directory has uncommitted changes"

**Cause**: You have unstaged or uncommitted changes

**Solution**:
```bash
# Option 1: Commit changes
git add -A
git commit -m "fix: your changes"

# Option 2: Stash changes
git stash
./scripts/create-release.sh 0.1.18
git stash pop
```

### Issue: "Not on main branch"

**Cause**: You're on a feature branch

**Solution**:
```bash
# Option 1: Switch to main
git checkout main
git merge feature-branch

# Option 2: Continue anyway (when prompted)
# Press 'y' when script asks "Continue anyway?"
```

### Issue: "No previous tags found"

**Cause**: First release in repository

**Solution**: Script will include all commits in changelog. This is expected for first release.

### Issue: Version mismatch between files

**Cause**: Manual edits or incomplete previous bump

**Solution**:
```bash
# Use bump-version.sh to resync all versions
./scripts/bump-version.sh patch
```

### Issue: Changelog entry inserted in wrong place

**Cause**: CHANGELOG.md format doesn't match expected pattern

**Solution**:
1. Manually edit CHANGELOG.md to follow Keep a Changelog format
2. Ensure existing entries start with `## [version] - date`
3. Re-run create-release.sh

## Best Practices

### DO ✅

1. **Use bump-version.sh** for all version changes
2. **Test with dry-run** before applying changes
3. **Use conventional commits** for automatic categorization
4. **Review changelog** before finalizing release
5. **Keep CHANGELOG.md** clean and well-formatted
6. **Tag releases** consistently with `v` prefix (v0.1.18)
7. **Document breaking changes** in commit body

### DON'T ❌

1. **Don't manually edit** version numbers in files
2. **Don't skip version bump** before release
3. **Don't forget** to commit version changes
4. **Don't mix** multiple version bumps in one commit
5. **Don't force push** tags to remote
6. **Don't delete** tags without good reason
7. **Don't skip** conventional commit format

## Integration with CI/CD

### GitHub Actions Workflow

The release process integrates with `.github/workflows/release.yml`:

**Trigger**: Pushing a tag starting with `v`

```bash
git push origin v0.1.18  # Triggers release workflow
```

**Workflow Steps**:
1. **Checkout** code at tag
2. **Build** for all platforms (macOS, Linux, Windows)
3. **Sign** binaries with Ed25519
4. **Create** updater artifacts (.sig files)
5. **Generate** latest.json manifest
6. **Upload** assets to GitHub Release
7. **Create** draft release

### Monitoring

After pushing a tag:

```bash
# Watch workflow progress
gh run watch

# Or visit GitHub Actions:
# https://github.com/dirvine/communitas/actions
```

### Release Assets

Generated artifacts:
- `Communitas_0.1.18_x64.dmg` (macOS)
- `Communitas_0.1.18_x64.dmg.tar.gz` (compressed)
- `Communitas_0.1.18_x64.dmg.tar.gz.sig` (signature)
- `communitas_0.1.18_amd64.AppImage` (Linux)
- `communitas_0.1.18_amd64.AppImage.tar.gz` (compressed)
- `communitas_0.1.18_amd64.AppImage.tar.gz.sig` (signature)
- `Communitas_0.1.18_x64_en-US.msi` (Windows)
- `Communitas_0.1.18_x64_en-US.msi.zip` (compressed)
- `Communitas_0.1.18_x64_en-US.msi.zip.sig` (signature)
- `latest.json` (update manifest)

## Version History

### Current Version: 0.1.17

See `CHANGELOG.md` for complete version history.

### Upcoming Releases

- **v0.2.0** (Minor): Version management automation
- **v0.3.0** (Minor): Enhanced update UI components
- **v1.0.0** (Major): Production-ready release

## Related Documentation

- **UPDATE_SYSTEM_SETUP.md** - Complete auto-update system setup
- **UPDATE_KEYS_SETUP.md** - Signing key generation and GitHub setup
- **UPDATE_SIGNING_DECISION.md** - Why we chose Ed25519 signatures
- **PHASE1_TASK2_COMPLETE.md** - GitHub release automation summary

## Summary

Version management in Communitas is fully automated:

1. **Bump** versions across all files: `./scripts/bump-version.sh patch`
2. **Create** release with changelog: `./scripts/create-release.sh 0.1.18`
3. **Push** tag triggers GitHub Actions
4. **Monitor** workflow and publish release

This ensures consistent versioning, proper changelogs, and reliable releases across all platforms.
