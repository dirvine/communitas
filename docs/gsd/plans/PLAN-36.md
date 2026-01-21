# PLAN-36: Phase 6.1 - Auth Hardening

**Milestone**: M6 Beta-Ready (Apple Desktop)
**Phase**: 6.1 Auth Hardening
**Created**: 2026-01-21
**Status**: Planning

## Goal

Solidify authentication for beta users with multi-identity UX, macOS biometric unlock (deferred - password-only for beta), comprehensive audit logging, and session management polish.

## Current State Analysis

Based on codebase exploration:

| Component | Status | Location |
|-----------|--------|----------|
| AuthService (UI) | Production | `communitas-ui-service/src/auth.rs` |
| SessionManager | Production | `communitas-core/src/encrypted_storage/session.rs` |
| RecentIdentity tracking | Ready | `communitas-core/src/encrypted_storage/app_config.rs` |
| PasskeyManager | Partial | `communitas-core/src/encrypted_storage/passkey.rs` |
| Recovery (BIP39) | Production | `communitas-core/src/recovery/` |
| Audit logging | Basic tracing only | Scattered `tracing::info!()` calls |
| Device fingerprinting | Missing | N/A |
| Session encryption | TODO in code | `session.rs` line with TODO comment |

## Tasks

### Task 1: Persistent Audit Log Infrastructure
**Files**: `communitas-core/src/security/audit_log.rs` (new), `communitas-core/src/security/mod.rs`

Create structured audit logging system for security-relevant events.

**What I'll do**:
1. Create `AuditLog` struct with append-only file-based storage
2. Define `AuditEvent` enum: Login, Logout, IdentitySwitch, FailedLogin, DeviceChange, Recovery, PasskeyRegister
3. Include timestamp, device_fingerprint, four_words (redacted), event_type, success, metadata
4. Add log rotation (keep 60 days, max 10MB per file)
5. Use ChaCha20-Poly1305 encryption for audit file (derive key from device secret)

**Verification**:
- `cargo fmt --all -- --check`
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core audit`

**Done when**: Audit log writes encrypted events to `~/.communitas/audit/events.enc`

---

### Task 2: Device Fingerprinting
**Files**: `communitas-core/src/security/device.rs` (new), `communitas-core/src/security/mod.rs`

Track unique device identifiers for new device detection.

**What I'll do**:
1. Create `DeviceFingerprint` struct with machine_id, os_version, hostname hash
2. Compute stable fingerprint using Blake3 hash
3. Add `is_new_device()` check against stored known devices
4. Store known devices in vault metadata (encrypted)
5. Emit `DeviceChange` audit event when new device detected

**Verification**:
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core device`

**Done when**: New device access triggers audit event and can optionally require confirmation

---

### Task 3: Failed Login Tracking
**Files**: `communitas-core/src/auth_service.rs`, `communitas-core/src/security/rate_limiter.rs`

Track and limit failed authentication attempts.

**What I'll do**:
1. Add failed attempt counter per four_words identity
2. Implement exponential backoff (1s, 2s, 4s, 8s, 16s... up to 5min)
3. Log failed attempts to audit log
4. Add `get_lockout_remaining()` method for UI feedback
5. Reset counter on successful login

**Verification**:
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core auth_service`

**Done when**: Failed logins tracked, exponential backoff enforced, audit logged

---

### Task 4: Session Persistence Encryption
**Files**: `communitas-core/src/encrypted_storage/session.rs`

Complete the TODO to encrypt session storage with device key.

**What I'll do**:
1. Derive session encryption key from device fingerprint + vault key
2. Encrypt `sessions.json` to `sessions.enc` using ChaCha20-Poly1305
3. Add authenticated encryption with nonce rotation
4. Handle migration from plaintext sessions.json if exists
5. Update `SessionStorage::load()` and `save()` methods

**Verification**:
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core session`

**Done when**: Session file encrypted, existing plaintext migrated

---

### Task 5: Session Timeout & Refresh UX
**Files**: `communitas-ui-service/src/auth.rs`, `communitas-core/src/encrypted_storage/session.rs`

Implement session timeout warnings and refresh flow.

**What I'll do**:
1. Add `session_expires_at()` method to AuthService
2. Emit warning event when session has <5 minutes remaining
3. Add `refresh_session()` method (re-validates and extends)
4. Add session state to AuthStateSnapshot: Authenticated(session, expires_soon: bool)
5. Update reactive subscriber to push timeout warnings

**Verification**:
- `cargo clippy -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service auth`

**Done when**: UI receives timeout warnings, can refresh session

---

### Task 6: Multi-Identity Quick Switch API
**Files**: `communitas-ui-service/src/auth.rs`

Expose multi-identity switching through UI service layer.

**What I'll do**:
1. Add `get_recent_identities()` to AuthService trait
2. Add `switch_identity(four_words)` to AuthService trait
3. Add `active_sessions()` listing all logged-in identities
4. Wire to existing SessionManager and AppConfigManager
5. Emit identity switch events through watch channel
6. Add audit logging for identity switches

**Verification**:
- `cargo clippy -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service auth`

**Done when**: UI can list recent identities and switch between them

---

### Task 7: Multi-Identity Dioxus Component
**Files**: `communitas-dioxus/src/components/identity_switcher.rs` (new), `communitas-dioxus/src/components/mod.rs`

Create UI component for identity switching.

**What I'll do**:
1. Create `IdentitySwitcher` component showing recent identities
2. Display four_words (masked), display_name, last_used timestamp
3. Show "active" badge on currently authenticated identity
4. Add "Switch" button triggering `switch_identity()`
5. Show loading state during switch
6. Handle errors gracefully with toast notifications

**Verification**:
- `dx check --platform desktop`
- Manual testing of identity switch flow

**Done when**: Users can visually switch between identities from UI

---

### Task 8: Recovery Flow Documentation
**Files**: `docs/user-guide/recovery.md` (new)

Document the recovery process for users.

**What I'll do**:
1. Document mnemonic generation and safe storage
2. Explain recovery process step-by-step
3. Add warnings about mnemonic security
4. Document what IS and ISN'T recovered (keys yes, local data no)
5. Add troubleshooting section for common issues

**Verification**:
- Markdown lint check
- Review for accuracy against code

**Done when**: User-facing recovery guide complete and accurate

---

### Task 9: Recovery Integration Tests
**Files**: `communitas-ui-service/tests/auth_integration.rs`

Add comprehensive tests for recovery flows.

**What I'll do**:
1. Test create → recover cycle with same mnemonic
2. Test recovery produces same public key
3. Test recovery with optional passphrase
4. Test invalid mnemonic handling
5. Test recovery when vault already exists (should update, not duplicate)

**Verification**:
- `cargo test -p communitas-ui-service recovery`

**Done when**: Recovery flows have integration test coverage

---

### Task 10: Audit Log MCP Tools
**Files**: `communitas-mcp/src/tools.rs`

Expose audit log through MCP for automation.

**What I'll do**:
1. Add `get_audit_log` tool (requires auth, returns recent events)
2. Add `export_audit_log` tool (exports date range to JSON)
3. Add filtering by event_type, date_range
4. Redact sensitive fields in output
5. Add tool documentation

**Verification**:
- `cargo clippy -p communitas-mcp --all-features -- -D warnings`
- `cargo test -p communitas-mcp audit`

**Done when**: MCP clients can query and export audit logs

---

### Task 11: Auth Hardening Integration Tests
**Files**: `communitas-ui-service/tests/auth_hardening_integration.rs` (new)

Comprehensive integration tests for all auth hardening features.

**What I'll do**:
1. Test audit logging captures all event types
2. Test device fingerprint detection
3. Test failed login backoff timing
4. Test session timeout and refresh
5. Test identity switch audit trail
6. Test encrypted session persistence

**Verification**:
- `cargo test -p communitas-ui-service auth_hardening`

**Done when**: All auth hardening features have integration test coverage

---

## Task Summary

| # | Task | Files | Est. Complexity |
|---|------|-------|-----------------|
| 1 | Persistent Audit Log Infrastructure | 2 new | Medium |
| 2 | Device Fingerprinting | 2 new | Medium |
| 3 | Failed Login Tracking | 2 existing | Low |
| 4 | Session Persistence Encryption | 1 existing | Medium |
| 5 | Session Timeout & Refresh UX | 2 existing | Low |
| 6 | Multi-Identity Quick Switch API | 1 existing | Low |
| 7 | Multi-Identity Dioxus Component | 2 new | Medium |
| 8 | Recovery Flow Documentation | 1 new | Low |
| 9 | Recovery Integration Tests | 1 existing | Low |
| 10 | Audit Log MCP Tools | 1 existing | Medium |
| 11 | Auth Hardening Integration Tests | 1 new | Medium |

## Dependencies

- Task 1 must complete before Tasks 3, 6, 10
- Task 2 must complete before Task 4
- Task 6 must complete before Task 7
- Tasks 1-6 should complete before Task 11

## Success Criteria

- [ ] Audit log persists to encrypted file
- [ ] Device changes detected and logged
- [ ] Failed logins trigger exponential backoff
- [ ] Sessions encrypted at rest
- [ ] Session timeout warnings emitted
- [ ] Identity switching works from UI
- [ ] Recovery documented for users
- [ ] All auth features have integration tests
- [ ] Zero compilation warnings
- [ ] All tests pass
