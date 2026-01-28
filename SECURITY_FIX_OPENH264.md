# Security Fix: OpenH264 Heap Overflow Vulnerability

**Date**: 2025-01-28
**Advisory**: RUSTSEC-2025-0008
**Severity**: CRITICAL

## Summary

A critical heap overflow vulnerability exists in `openh264-sys2 v0.7.1` that is being pulled in transitively via `saorsa-webrtc-codecs v0.3.0`.

## Current State

```
Workspace pin: openh264-sys2 = "0.8.0"  (in Cargo.toml line 43)
Actual dependency: openh264-sys2@0.7.1 (vulnerable)
Source: saorsa-webrtc-codecs v0.3.0 → openh264 v0.7.2 → openh264-sys2 v0.7.1
```

**Why workspace pin doesn't work**: The workspace dependency pin only affects direct dependencies, not transitive dependencies from external crates like `saorsa-webrtc-codecs`.

## Root Cause

`saorsa-webrtc-codecs v0.3.0` has a direct dependency on `openh264 v0.7.2`, which depends on `openh264-sys2 v0.7.1`.

## Required Actions

### Option 1: Update saorsa-webrtc-codecs (RECOMMENDED)

Coordinate with Saorsa Labs maintainers to release `saorsa-webrtc-codecs v0.3.1` with:
```toml
[dependencies]
openh264 = "0.8"  # or >=0.8.0
```

Then update in Cargo.toml:
```toml
saorsa-webrtc-codecs = "0.3.1"
```

### Option 2: Dependency Override (Temporary Fix)

Add to `.cargo/config.toml`:
```toml
[patch.crates-io]
# This patches the transitive dependency through saorsa-webrtc-codecs
openh264 = { git = "https://github.com/rustyvideo/rustyvideo", branch = "main" }
```

**Note**: This requires finding a fork or maintaining a patch.

### Option 3: Fork and Patch (Immediate Workaround)

1. Fork `saorsa-webrtc-codecs`
2. Update `Cargo.toml` to use openh264 0.8
3. Publish to private registry or use git dependency
4. Update workspace to use patched version

### Option 4: Disable WebRTC Until Fixed (Safest)

If video calling is not yet in production:
- Remove `saorsa-webrtc-codecs` and `saorsa-webrtc-core` dependencies
- Comment out WebRTC functionality
- Re-enable after upstream fix

## Verification Steps

After applying fix:

```bash
# Check only secure version remains
cargo tree -i openh264-sys2

# Should see ONLY:
# openh264-sys2 v0.8.x

# Run audit to confirm
cargo audit

# Should see:
# error: 0 vulnerabilities found!
```

## Timeline

- **Today**: Document vulnerability and mitigation options
- **This Week**: Coordinate with saorsa-webrtc maintainers
- **Next Release**: Include updated dependency
- **Immediate**: Consider disabling WebRTC if calling features not yet exposed to users

## Impact Assessment

**High Risk If**:
- Video calling/calling features are user-facing
- Malicious users can initiate calls
- Application runs with elevated privileges

**Lower Risk If**:
- Calling features are internal/dev only
- Network access is restricted
- Users are trusted

**Regardless of exposure level**, this vulnerability MUST be fixed before any production deployment.

## Recommendation

**IMMEDIATE**: If video calling is not yet a user feature, disable it temporarily:
- Remove or comment WebRTC dependencies
- Document in CHANGELOG
- Re-enable after upstream fix

**SHORT-TERM**: Coordinate with saorsa-webrtc maintainers for official update

**LONG-TERM**: Establish dependency review process for all external crates

---

**Status**: AWAITING FIX
**Next Review**: After saorsa-webrtc-codecs update available
