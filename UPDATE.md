# Communitas Desktop Auto-Update Specification

## Scope
- Replace separate updater app with built-in self-updating.
- Platforms: macOS, Windows, Linux.
- Distribution: GitHub Releases only.
- Mobile platforms excluded.

## Release Backend
- Use **GitHub Releases** to host all artifacts.
- Each release must include:
  - Platform bundles (`.dmg`, `.msi`/`.exe`, `.AppImage`).
  - Detached signature files (`.sig`).
  - `latest.json` manifest.

Example `latest.json`:
```json
{
  "version": "1.2.3",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2025-09-07T12:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "url": "https://github.com/dirvine/communitas/releases/download/v1.2.3/communitas-macos-x86_64.dmg",
      "signature": "BASE64_SIGNATURE"
    },
    "darwin-aarch64": {
      "url": "https://github.com/dirvine/communitas/releases/download/v1.2.3/communitas-macos-arm64.dmg",
      "signature": "BASE64_SIGNATURE"
    },
    "windows-x86_64": {
      "url": "https://github.com/dirvine/communitas/releases/download/v1.2.3/communitas-windows-x86_64.msi",
      "signature": "BASE64_SIGNATURE"
    },
    "linux-x86_64": {
      "url": "https://github.com/dirvine/communitas/releases/download/v1.2.3/communitas-linux-x86_64.AppImage",
      "signature": "BASE64_SIGNATURE"
    }
  }
}
```

## Desktop GUI (Tauri v2)
- Integrate `tauri-plugin-updater`.
- Configure in `src-tauri/tauri.conf.json`:
```json
{
  "bundle": { "createUpdaterArtifacts": true },
  "plugins": {
    "updater": {
      "pubkey": "PASTE_PUBLIC_KEY",
      "endpoints": [
        "https://github.com/dirvine/communitas/releases/latest/download/latest.json"
      ]
    }
  }
}
```
- Application checks for updates on startup and via menu.
- On update: download → verify signature → install → relaunch.

## Headless CLI
- Use `self_update` crate with GitHub backend.
- Integrate into CLI:
```rust
pub fn try_self_update() -> anyhow::Result<Option<String>> {
  use self_update::cargo_crate_version;
  let status = self_update::backends::github::Update::configure()
    .repo_owner("dirvine")
    .repo_name("communitas")
    .bin_name("communitas-headless")
    .current_version(cargo_crate_version!())
    .build()?
    .update()?;
  Ok(Some(status.version().to_string()))
}
```
- Release archives must include binary + `.sig` file.

## Security
- Generate signing key with `tauri signer`.
- Embed **public** key in `tauri.conf.json`.
- Store **private** key in GitHub Actions secrets.
- CI signs all release artifacts automatically.

## CI/CD Workflow
- Trigger on version tags (`v*`).
- Jobs:
  1. Build for macOS, Windows, Linux.
  2. Code-sign and notarize (macOS, Windows).
  3. Generate signatures + `latest.json`.
  4. Upload to GitHub Release.
- Use `tauri-apps/tauri-action` for GUI, `actions-rs/cargo` for CLI.

## OS-Specific Notes
- **macOS**: Require Apple Developer ID + notarization.
- **Windows**: Sign installer with code-signing certificate.
- **Linux**: Ship as AppImage with `.sig` verification.

## Rollback & Safety
- Keep last N releases on GitHub.
- All updates verified with Ed25519 signatures.
- If verification fails, abort update and keep existing version.

---

## Developer Tasks
1. Add updater plugin to Communitas desktop app.
2. Implement `self_update` flow for headless CLI.
3. Create GitHub Actions workflow for multi-platform builds.
4. Add signing and notarization steps.
5. Publish `latest.json` with every release.
6. Document update process in `UPDATE.md`.
