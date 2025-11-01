# Release Workflow Guide

## Overview

The release system has been refactored to prevent conflicts and ensure a single source of truth for releases.

## How Releases Work

When you push a tag matching `v*` (e.g., `v1.0.0`), the following happens:

1. **`release-headless.yml`** (orchestrator) triggers and:
   - Creates a draft GitHub release
   - Builds headless binaries (Linux, macOS, Windows)
   - Builds TUI binaries for all platforms
   - Calls the desktop build workflow
   - Publishes the release when all builds complete

2. **`tauri-release.yml`** (reusable workflow):
   - Called by the orchestrator with the release ID
   - Builds Tauri desktop apps (macOS universal, Ubuntu, Windows)
   - Attaches desktop artifacts to the same release

## Triggering a Release

### Automatic (Recommended)
```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

### Manual
Go to Actions → "Release Headless Binaries" → Run workflow

## Workflow Files

- **`release-headless.yml`**: Main orchestrator - creates release, builds headless/TUI, calls desktop build
- **`tauri-release.yml`**: Reusable workflow for desktop builds only (no longer triggers on tags)
- **`release.yml`**: DEPRECATED - disabled to prevent conflicts

## Key Improvements

✅ Single release creation (no more race conditions)  
✅ Concurrency guard prevents duplicate releases  
✅ Desktop artifacts attach to the same release as headless  
✅ Explicit `releaseId` passing ensures correct artifact attachment  
✅ Frontend build step included before Tauri build  
✅ Proper config path: `-c communitas-desktop/tauri.conf.json`  

## Secrets Required

### For Desktop Builds
- `GH_RELEASE_TOKEN` or `GITHUB_TOKEN` (auto-provided)
- `APPLE_CERTIFICATE` (macOS signing)
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID` (notarization)
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`
- `WINDOWS_CERTIFICATE_THUMBPRINT` (Windows signing)
- `TAURI_UPDATER_PUBKEY` (for auto-updates)

### For Headless Builds
- `GH_RELEASE_TOKEN` or `GITHUB_TOKEN`

## Troubleshooting

**Issue**: Multiple releases created for the same tag  
**Fix**: Ensure old `release.yml` is disabled (tag trigger removed)

**Issue**: Desktop artifacts missing from release  
**Fix**: Check that `release-headless.yml` includes `build-desktop` in publish needs

**Issue**: Tauri build fails with "config not found"  
**Fix**: Verify `-c communitas-desktop/tauri.conf.json` is in args

**Issue**: Frontend assets not found  
**Fix**: Ensure `npm run build` runs before Tauri build step
