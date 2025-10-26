# Tauri Version Compatibility Issue - Detailed Explanation

## The Problem

Our project encountered a build failure with the error:
```
thread 'main' panicked at tauri-build-2.5.1/src/lib.rs:418:29:
missing `cargo:dev` instruction, please update tauri to latest: NotPresent
```

## Root Cause Analysis

### What is the `cargo:dev` Instruction?

The `cargo:dev` instruction is a Cargo build script directive that `tauri-build` expects to receive from the `tauri` runtime crate. In Rust's build system:

1. When crate A depends on crate B, and B has a build script
2. B's build script can emit `cargo:KEY=VALUE` instructions  
3. These become environment variables like `DEP_B_KEY` for crate A
4. `tauri-build` (at line 418) checks for `DEP_TAURI_DEV` environment variable
5. This variable should be set by the `tauri` crate's build script

### Version Compatibility Matrix

```
Component          | Version Used | Latest Stable | Issue
-------------------|--------------|---------------|-------
tauri              | 2.9.1        | 2.9.1         | ✓ Latest
tauri-build        | 2.5.1        | 2.5.1         | ✓ Latest  
tauri-plugin-*     | 2.4-2.9      | Various       | Mixed
```

**The Core Issue:**
- `tauri-build` 2.5.1 is the latest **stable** version on crates.io
- `tauri` runtime has progressed to 2.9.1
- Between 2.5.x and 2.9.x, Tauri changed how build scripts communicate
- `tauri-build` 2.5.1 expects a `cargo:dev` instruction that `tauri` 2.9.x emits differently (or not at all)

### Why This Happens

Tauri uses a **fast release cycle** where:
1. Runtime (`tauri` crate) releases frequently (2.8, 2.9, etc.)
2. Build tools (`tauri-build`) release more slowly (still at 2.5.1)
3. The assumption is that newer runtimes are backwards-compatible
4. However, build-time contract changes break this assumption

### Plugin Version Requirements

From dependency analysis:
- `tauri-plugin-dialog` 2.4 requires `tauri >= 2.8.2`
- `tauri-plugin-log` 2.5 requires `tauri >= 2.8`
- `tauri-plugin-updater` 2.5-2.9 have varying requirements

This means we cannot downgrade to `tauri` 2.5.1 without losing plugin support.

## The Solution

### What We Tried (Failed Approaches):

1. ❌ **Adding `println!("cargo:dev=false")` in our build.rs**
   - Doesn't work because `tauri-build` checks AFTER running our code
   - The instruction must come FROM `tauri`'s build script, not ours

2. ❌ **Downgrading to tauri 2.5.1**
   - Plugins require >= 2.8.2
   - Creates new dependency conflicts

3. ❌ **Upgrading tauri-build to 2.9**
   - Version 2.9.x doesn't exist on crates.io yet
   - Only 2.5.1 is available as stable

### Final Solution: Use Tauri 2.8.x

We downgraded from `tauri` 2.9.x to 2.8.5:

```toml
# communitas-desktop/Cargo.toml
[build-dependencies]
tauri-build = "2.5.1"   # Latest stable

[dependencies]
tauri = { version = ">=2.8, <2.9", features = ["custom-protocol"] }
tauri-plugin-dialog = "2.4"
tauri-plugin-log = "2.5"
tauri-plugin-updater = "2.5"
```

**Why This Works:**
- Tauri 2.8.5 (latest in 2.8.x series) still emits the `cargo:dev` instruction expected by tauri-build 2.5.1
- All plugins are compatible with 2.8.x
- tauri-build 2.5.1 works correctly with tauri 2.8.x

## Version Compatibility Rules for Tauri 2.x

Based on investigation:

1. **tauri-build version should match tauri MINOR version**
   - `tauri-build 2.5.x` works with `tauri 2.5.x` and `tauri 2.8.x`
   - `tauri-build 2.5.x` does NOT work with `tauri 2.9.x`

2. **Plugins generally follow tauri runtime version**
   - Match plugin major.minor to tauri major.minor where possible
   - Check plugin documentation for minimum tauri version

3. **When in doubt, use matching versions**
   - If using `tauri 2.8.x`, use `tauri-build 2.5.x` (highest stable)
   - If using `tauri 2.9.x`, wait for `tauri-build 2.9.x` or use git deps

## Future Proofing

When `tauri-build 2.9.x` is published to crates.io:

1. Update to matching versions:
   ```toml
   tauri-build = "2.9"
   tauri = "2.9"
   tauri-plugin-* = "2.9"
   ```

2. Or use the latest stable set:
   ```toml
   tauri-build = { version = "2" }
   tauri = { version = "2" }
   ```
   This auto-updates to latest compatible 2.x versions

## Related GitHub Issues

- [tauri#10591](https://github.com/tauri-apps/tauri/issues/10591) - Original report of cargo:dev NotPresent error
- Resolution: Update dependencies with `cargo update` in src-tauri directory

## Testing Matrix

After fix:
- ✅ macOS build: SUCCESS  
- ✅ Linux build: SUCCESS
- ✅ Windows build: SUCCESS
- ✅ All tests pass
- ✅ Clippy passes
