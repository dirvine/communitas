# Security Architecture Decisions

## Certificate Pinning Approach

### Decision: Use Raw Public Key Pinning (RFC 7250)

**Date**: 2025-01-14

**Status**: Implemented

**Context**: 
The project initially had two approaches for certificate pinning in QUIC connections:
1. `tls_pinning.rs` - Traditional certificate pinning using full X.509 certificates in PEM format
2. `raw_spki.rs` - Raw public key pinning using Ed25519 public keys directly

**Decision**:
We chose to use raw public key pinning (raw_spki module) exclusively and removed the tls_pinning module.

**Rationale**:
1. **Simplicity**: Raw public keys are simpler - just 32 bytes for Ed25519 vs full certificate chains
2. **Post-Quantum Ready**: Aligns with our PQC strategy by focusing on keys rather than certificate infrastructure
3. **Efficiency**: Smaller payloads (32 bytes vs kilobytes for certificates)
4. **Flexibility**: Supports multiple input formats (hex, base64, SPKI, raw key bytes)
5. **Maintenance**: Single approach reduces code duplication and maintenance burden

**Implementation**:
- The `raw_spki` module provides two Tauri commands:
  - `sync_set_quic_pinned_spki` - Sets a pinned public key
  - `sync_clear_quic_pinned_spki` - Clears the pinned key
- Supports environment variable `COMMUNITAS_QUIC_PINNED_SPKI` for configuration
- Used by the QUIC sync functionality in `sync.rs` for secure P2P connections

**Consequences**:
- Positive: Cleaner codebase, single source of truth for certificate pinning
- Positive: Better alignment with RFC 7250 (Raw Public Keys in TLS)
- Negative: Cannot pin traditional CA certificates (not needed for our P2P use case)

**References**:
- RFC 7250: Using Raw Public Keys in Transport Layer Security
- communitas-desktop/src/security/raw_spki.rs
- communitas-desktop/src/sync.rs (lines 119-125)