# Release Workflow Guide

## Overview

Releases currently package **headless binaries** via `release-headless.yml`.
The Flutter app is built in CI (`ci.yml`) and published separately when ready.

## How Releases Work

When you push a tag matching `v*` (e.g., `v1.0.0`):

1. **`release-headless.yml`** triggers and:
   - Creates a draft GitHub release
   - Builds headless binaries (Linux, macOS, Windows)
   - Uploads artifacts to the release

## Triggering a Release

### Automatic (Recommended)
```bash
git tag v1.0.0
git push origin v1.0.0
```

### Manual
Go to Actions → "Release Headless Binaries" → Run workflow

## Workflow Files

- **`release-headless.yml`**: Draft release + headless binaries
- **`release-binaries.yml`**: CI artifacts for headless builds (non-release)

## Flutter App Packaging

Flutter desktop/mobile builds are produced in CI and stored as artifacts.
Packaging/signing for store distribution is tracked separately.

## Secrets Required

- `GITHUB_TOKEN` (auto-provided)
- Optional signing/notarization secrets for future Flutter packaging
