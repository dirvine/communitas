# URGENT SECURITY ADVISORY - Communitas v0.8.2

**Date**: 2025-01-28
**Severity**: CRITICAL
**Status**: REQUIRES IMMEDIATE ACTION

## Vulnerability Summary

**1 CRITICAL VULNERABILITY DETECTED** in dependency tree that affects the WebRTC/call functionality.

### Vulnerability #1: OpenH264 Heap Overflow (CRITICAL)

**ID**: RUSTSEC-2025-0008
**Crate**: openh264-sys2 v0.7.1
**Title**: Openh264 Decoding Functions Heap Overflow Vulnerability
**Date**: 2025-02-24
**CVSS**: High (heap buffer overflow in video decoding)
**Affected Component**: WebRTC video calling/calling features

**Description**:
A heap overflow vulnerability exists in the OpenH264 decoding functions that could allow remote code execution when processing malicious video streams during calls.

**Impact**:
- Remote attackers could trigger the vulnerability by initiating a WebRTC call
- Successful exploitation could lead to:
  - Remote code execution with application privileges
  - Application crash (DoS)
  - Potential sandbox escape depending on platform

**Affected Features**:
- Voice/video calling functionality
- Screen sharing features
- Any feature using saorsa-webrtc-codecs

**Remediation**:
```
UPGRADE REQUIRED: openh264-sys2 from 0.7.1 to >=0.8.0
```

**Current State**:
The workspace Cargo.toml already pins openh264-sys2 to 0.8.0 (line 43):
```toml
openh264-sys2 = "0.8.0"
```

**Problem**: Despite the pin, the dependency tree shows 0.7.1 is being pulled in. This indicates a transitive dependency issue.

**Required Actions**:

1. **IMMEDIATE**: Investigate why openh264-sys2 0.7.1 is being pulled in
2. **IMMEDIATE**: Update saorsa-webrtc-codecs to use openh264-sys2 0.8.0
3. **IMMEDIATE**: Force update with: `cargo update -p openh264-sys2 --precise 0.8.0`
4. **SHORT-TERM**: Coordinate with saorsa-webrtc maintainers for official update
5. **VERIFY**: Run `cargo tree -i openh264-sys2` to confirm 0.8.0 is used

---

## Additional Warnings (19 Unmaintained Dependencies)

The following dependencies are unmaintained but pose lower immediate risk:

### GTK3 Bindings (Low Risk - UI Only)
- atk, atk-sys, gtk, gtk-sys (RUSTSEC-2024-0413, RUSTSEC-2024-0416)
- **Impact**: Dioxus desktop UI framework
- **Mitigation**: These are GUI libraries, attack surface is limited to user interaction
- **Recommendation**: Monitor for GTK4 migration in Dioxus ecosystem

### Bincode (Medium Risk - Serialization)
- bincode 1.3.3 (RUSTSEC-2025-0141)
- **Impact**: WebRTC DTLS serialization
- **Mitigation**: Used in networking code, not user input parsing
- **Recommendation**: Plan migration to bincode 2.0 or alternative (serde-based)

---

## Updated Security Grade: **D (60/100)**

**Previous Grade**: A- (92/100)
**Downgrade Reason**: Critical unpatched vulnerability in production code

**Blocking Issue**: The OpenH264 vulnerability MUST be fixed before any production deployment.

---

## Verification Commands

```bash
# Check current version
cargo tree -i openh264-sys2

# Attempt forced update
cargo update -p openh264-sys2 --precise 0.8.0

# Verify fix
cargo audit

# Rebuild to ensure compatibility
cargo build --release
```

---

## Timeline

- **Immediate (Today)**: Investigate and patch OpenH264 vulnerability
- **Today**: Verify no video calling features are exposed until fixed
- **This Week**: Coordinate fix with upstream saorsa-webrtc maintainers
- **Next Release**: Include OpenH264 fix as critical security update

---

## Recommendation

**DO NOT DEPLOY** current build to production with video calling enabled. If video calling is not yet a user-facing feature, the risk is reduced but the vulnerability should still be patched immediately.

**For deployments without video calling**: Risk is lower but still present - patch immediately.

**For development builds**: Acceptable to continue development, but flag video calling features as disabled until fixed.

---

## Contact

For questions about this advisory, contact:
- Project Maintainer: david@saorsalabs.com
- Security Issues: Report via GitHub Security Advisories

---

**END OF ADVISORY**
