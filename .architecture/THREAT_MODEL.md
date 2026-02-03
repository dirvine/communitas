# Communitas Threat Model

## Overview

This document analyzes security threats to Communitas in its current state and with future Canvas/Agent expansion. It identifies:

1. **Current threats** and existing mitigations
2. **Future threats** introduced by Canvas and agent collaboration
3. **Required controls** that must be implemented before expansion

**Principle**: Security controls must be layered into infrastructure now, not retrofitted later.

---

## Threat Actor Categories

| Actor | Motivation | Capability | Targets |
|-------|------------|------------|---------|
| **Malicious User** | Data theft, abuse | Authenticated access | Other users' data, system resources |
| **Compromised Agent** | Escalation, exfiltration | Delegated token access | User data, agent privileges |
| **Network Attacker** | MITM, surveillance | Network position | P2P traffic, metadata |
| **Malicious Peer** | Sybil attacks, spam | P2P membership | Network stability, user experience |
| **Insider** | Privilege abuse | Admin/owner access | Organization data |
| **Nation State** | Surveillance, disruption | Advanced persistent | Cryptographic keys, metadata |

---

## 1. Current System Threats

### 1.1 Identity & Authentication

| Threat | Impact | Current Mitigation | Status |
|--------|--------|-------------------|--------|
| **T1.1**: Vault password brute force | Identity compromise | Argon2id key derivation (high memory cost) | **MITIGATED** |
| **T1.2**: Mnemonic theft | Permanent identity loss | Platform keyring storage, user education | **PARTIAL** |
| **T1.3**: Session hijacking | Temporary access | 10-minute unlock lease, session binding | **MITIGATED** |
| **T1.4**: Quantum key extraction | Future compromise | ML-DSA-87 (NIST Level 5), ML-KEM-768 | **MITIGATED** |
| **T1.5**: Phishing via four-words | Wrong peer connection | Four-word checksum validation | **MITIGATED** |

**Risk Level**: LOW - PQC and proper key management are in place.

### 1.2 Data Security

| Threat | Impact | Current Mitigation | Status |
|--------|--------|-------------------|--------|
| **T2.1**: Message content exposure | Privacy breach | E2E encryption (ChaCha20-Poly1305) | **MITIGATED** |
| **T2.2**: Metadata leakage | Privacy breach | Gossip overlay obscures routing | **PARTIAL** |
| **T2.3**: Storage at rest exposure | Data theft | Encrypted vault storage | **MITIGATED** |
| **T2.4**: CRDT conflict injection | Data corruption | Signed operations, author verification | **MITIGATED** |
| **T2.5**: File upload malware | System compromise | Content-type validation, sandboxed preview | **PARTIAL** |

**Risk Level**: LOW-MEDIUM - Metadata protection could be stronger.

### 1.3 Network Security

| Threat | Impact | Current Mitigation | Status |
|--------|--------|-------------------|--------|
| **T3.1**: Eclipse attack | Network isolation | Gossip redundancy (HyParView fanout) | **PARTIAL** |
| **T3.2**: Sybil attack | Network spam | Connection limits, reputation (basic) | **PARTIAL** |
| **T3.3**: Man-in-the-middle | Message interception | QUIC TLS 1.3 with ML-KEM | **MITIGATED** |
| **T3.4**: Denial of service | Service unavailability | Rate limiting, connection caps | **MITIGATED** |
| **T3.5**: Bootstrap poisoning | Wrong network | Signed bootstrap list, peer verification | **PARTIAL** |

**Risk Level**: MEDIUM - Sybil and eclipse attacks need more robust defenses.

### 1.4 Application Security

| Threat | Impact | Current Mitigation | Status |
|--------|--------|-------------------|--------|
| **T4.1**: Input injection | Code execution | Input validation module | **MITIGATED** |
| **T4.2**: Command injection | System compromise | Rust type safety, no shell calls | **MITIGATED** |
| **T4.3**: Memory safety | Corruption/crash | Rust (safe by default) | **MITIGATED** |
| **T4.4**: Panic/unwrap abuse | DoS | Clippy lints enforce Result types | **MITIGATED** |
| **T4.5**: Dependency vulnerability | Supply chain | cargo-audit, dependabot | **PARTIAL** |

**Risk Level**: LOW - Rust provides strong baseline security.

---

## 2. Future Threats (Canvas/Agent Expansion)

### 2.1 Agent-Specific Threats

| Threat | Impact | Current Mitigation | Required Control |
|--------|--------|-------------------|------------------|
| **T5.1**: Agent privilege escalation | Unauthorized actions | Scope-limited tokens | **Policy Kernel** |
| **T5.2**: Agent impersonation | Trust manipulation | Token binding | **Principal verification** |
| **T5.3**: Agent resource exhaustion | DoS | Basic rate limits | **Per-principal quotas** |
| **T5.4**: Agent data exfiltration | Privacy breach | Scope restrictions | **Capability firewall** |
| **T5.5**: Compromised agent token | Long-term access | Token expiration (10min) | **Token revocation** |
| **T5.6**: Agent collusion | Coordinated abuse | None | **Multi-agent monitoring** |

**Risk Level**: HIGH without Policy Kernel - agents have broad access today.

### 2.2 Canvas-Specific Threats

| Threat | Impact | Current Mitigation | Required Control |
|--------|--------|-------------------|------------------|
| **T6.1**: Canvas auth bypass | Unauthorized access | N/A (Canvas not built) | **Principal::Canvas type** |
| **T6.2**: Canvas-specific vulnerabilities | Exploitation | N/A | **Same security model as Dioxus** |
| **T6.3**: Canvas-agent confusion | Privilege confusion | N/A | **Clear principal separation** |
| **T6.4**: Canvas data divergence | Inconsistency | N/A | **Same UiServices backend** |
| **T6.5**: Canvas UI manipulation | Social engineering | N/A | **Consistent approval UI** |

**Risk Level**: N/A currently - controls required before Canvas exists.

### 2.3 Approval Queue Threats

| Threat | Impact | Current Mitigation | Required Control |
|--------|--------|-------------------|------------------|
| **T7.1**: Approval fatigue | Rubber-stamping | N/A | **Smart defaults, batching** |
| **T7.2**: Approval bypass | Unauthorized execution | N/A | **Kernel-enforced gates** |
| **T7.3**: Approval replay | Duplicate execution | N/A | **Nonce-based receipts** |
| **T7.4**: Approval expiration abuse | Stale actions | N/A | **Strict TTL enforcement** |
| **T7.5**: Approval UI spoofing | Wrong action approved | N/A | **Signed action previews** |

**Risk Level**: N/A currently - approval queue not built.

### 2.4 Policy Kernel Threats

| Threat | Impact | Current Mitigation | Required Control |
|--------|--------|-------------------|------------------|
| **T8.1**: Rule misconfiguration | Overly permissive | N/A | **Default-deny + tests** |
| **T8.2**: Rule bypass | Unauthorized access | N/A | **Single enforcement point** |
| **T8.3**: Receipt forgery | Fake audit trail | N/A | **ML-DSA-87 signatures** |
| **T8.4**: Rule race conditions | Inconsistent decisions | N/A | **Atomic evaluation** |
| **T8.5**: Kernel denial of service | System unavailable | N/A | **Circuit breakers** |

**Risk Level**: N/A currently - Policy Kernel not built.

---

## 3. Attack Scenarios

### 3.1 Current System: Malicious Peer

```
1. Attacker joins network via bootstrap
2. Attacker creates multiple identities (Sybil)
3. Attacker targets specific user for eclipse
4. Attacker isolates user from honest peers
5. Attacker feeds user false CRDT state

Mitigations (EXISTS):
- Connection limits per IP
- Gossip redundancy
- Signed CRDT operations

Mitigations (NEEDED):
- Reputation system with stake
- Enhanced Sybil detection
- Peer diversity requirements
```

### 3.2 Future System: Compromised Agent

```
1. User grants agent "WriteMessages" scope
2. Agent is compromised by malicious actor
3. Agent sends spam/phishing to user's contacts
4. Agent exfiltrates message history

Mitigations (EXISTS):
- Scope restrictions
- Token expiration (10min)

Mitigations (NEEDED):
- Per-operation approval for sensitive actions
- Anomaly detection on agent behavior
- One-click agent revocation
- Audit trail with receipts
```

### 3.3 Future System: Canvas + Agent Coordination

```
1. User opens Canvas with agent
2. Agent proposes batch of operations
3. Canvas UI shows approval dialog
4. User approves without careful review
5. Agent executes harmful operation

Mitigations (NEEDED):
- Clear action previews in approval UI
- Operation grouping and summarization
- Anomaly flags on unusual operations
- "Explain this" button for actions
- Rate limits on batch approvals
```

---

## 4. Security Control Requirements

### 4.1 P0: Must Have Before Canvas Alpha

| Control | Purpose | Blocks |
|---------|---------|--------|
| **Policy Kernel** | Unified authorization gate | All Canvas operations |
| **Principal::Canvas** | Canvas identity type | Canvas authentication |
| **Receipt signing** | Tamper-proof audit | Agent accountability |
| **Token revocation** | Emergency access removal | Compromised agent response |
| **Scope validation** | Capability enforcement | Agent privilege limits |

### 4.2 P1: Must Have Before Canvas Beta

| Control | Purpose | Blocks |
|---------|---------|--------|
| **Approval Queue** | Human-in-the-loop for agents | Autonomous agent actions |
| **Approval UI (Dioxus)** | User approves agent proposals | Agent action execution |
| **Per-principal rate limits** | Resource exhaustion prevention | Agent DoS |
| **Audit log export** | Compliance and investigation | Enterprise deployment |
| **Anomaly detection** | Unusual behavior flagging | Compromised agent detection |

### 4.3 P2: Must Have Before Canvas GA

| Control | Purpose | Blocks |
|---------|---------|--------|
| **Quarantine layer** | Untrusted code isolation | Third-party agents |
| **Trust escalation** | Progressive permission grants | Advanced agent workflows |
| **Multi-agent coordination** | Prevent agent conflicts | Complex agent scenarios |
| **Performance monitoring** | SLA enforcement | Production reliability |
| **Security audit** | External validation | Enterprise adoption |

---

## 5. Trust Boundaries

### 5.1 Current System

```
┌─────────────────────────────────────────────────────────────────┐
│                      TRUSTED (User's Device)                     │
│                                                                  │
│  ┌───────────────┐   ┌───────────────┐   ┌───────────────────┐ │
│  │ Dioxus App    │   │ Platform      │   │ Vault (Encrypted) │ │
│  │               │   │ Keyring       │   │                   │ │
│  └───────────────┘   └───────────────┘   └───────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                        Trust Boundary 1
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SEMI-TRUSTED (Local Services)                 │
│                                                                  │
│  ┌───────────────┐   ┌───────────────┐                         │
│  │ MCP Server    │   │ Headless Node │                         │
│  │ (Local Only)  │   │               │                         │
│  └───────────────┘   └───────────────┘                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                        Trust Boundary 2
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      UNTRUSTED (Network)                         │
│                                                                  │
│  ┌───────────────┐   ┌───────────────┐   ┌───────────────────┐ │
│  │ Remote Peers  │   │ Bootstrap     │   │ Internet          │ │
│  │               │   │ Nodes         │   │                   │ │
│  └───────────────┘   └───────────────┘   └───────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Future System (with Canvas/Agents)

```
┌─────────────────────────────────────────────────────────────────┐
│                      TRUSTED (User's Device)                     │
│                                                                  │
│  ┌───────────────┐   ┌───────────────┐   ┌───────────────────┐ │
│  │ Dioxus App    │   │ Saorsa Canvas │   │ Vault (Encrypted) │ │
│  │ (TrustedUi)   │   │ (TrustedUi)   │   │                   │ │
│  └───────────────┘   └───────────────┘   └───────────────────┘ │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                        Trust Boundary 1 (Policy Kernel Gate)
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SCOPED TRUST (Delegated)                      │
│                                                                  │
│  ┌───────────────┐   ┌───────────────┐   ┌───────────────────┐ │
│  │ Agent A       │   │ Agent B       │   │ Third-Party       │ │
│  │ (ReadMessages)│   │ (WriteFiles)  │   │ Integration       │ │
│  └───────────────┘   └───────────────┘   └───────────────────┘ │
│                                                                  │
│  Approval Queue: Agent actions require user approval             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                        Trust Boundary 2 (Quarantine)
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    UNTRUSTED (Sandboxed)                         │
│                                                                  │
│  ┌───────────────┐   ┌───────────────┐                         │
│  │ Untrusted     │   │ Experimental  │                         │
│  │ Agent Code    │   │ Plugins       │                         │
│  └───────────────┘   └───────────────┘                         │
│                                                                  │
│  Quarantine: Isolated execution with strict resource limits      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                        Trust Boundary 3 (Network)
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      UNTRUSTED (Network)                         │
│                                                                  │
│  Remote Peers │ Bootstrap Nodes │ Internet                       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Cryptographic Assumptions

| Algorithm | Purpose | Security Level | Assumption |
|-----------|---------|----------------|------------|
| ML-DSA-87 | Identity signatures | NIST Level 5 | Lattice hardness |
| ML-KEM-768 | Key encapsulation | NIST Level 3 | MLWE hardness |
| ChaCha20-Poly1305 | Symmetric encryption | 256-bit | No practical attacks |
| BLAKE3 | Hashing | 256-bit | No practical attacks |
| Argon2id | Password derivation | High memory | Time/memory tradeoff |

**Quantum Readiness**:
- Signature and key exchange are post-quantum
- Symmetric crypto remains secure against quantum
- No migration needed when quantum computers arrive

---

## 7. Residual Risks

### 7.1 Accepted Risks

| Risk | Rationale | Monitoring |
|------|-----------|------------|
| Sybil attacks at low volume | Cost-benefit for MVP | Connection metrics |
| Metadata timing analysis | Complex mitigation for later | Research roadmap |
| Social engineering | User education focus | Community reports |

### 7.2 Mitigated-but-Watch Risks

| Risk | Current Mitigation | Watch For |
|------|-------------------|-----------|
| PQC algorithm weaknesses | Using NIST finalists | NIST announcements |
| CRDT semantic attacks | Signed operations | Academic research |
| WebView vulnerabilities | Tauri security model | CVE feeds |

### 7.3 Unmitigated Risks (Require Future Work)

| Risk | Required Work | Priority |
|------|---------------|----------|
| Eclipse attacks at scale | Enhanced peer diversity | P1 |
| Agent behavior anomalies | ML-based detection | P2 |
| Approval fatigue | UX research and design | P2 |

---

## 8. Security Testing Requirements

### 8.1 Current (Implemented)

| Test Type | Coverage | Frequency |
|-----------|----------|-----------|
| Unit tests (crypto) | Core algorithms | Every commit |
| Integration tests | P2P networking | Every commit |
| Property-based tests | CRDT operations | Every commit |

### 8.2 Required (Before Canvas)

| Test Type | Coverage | Priority |
|-----------|----------|----------|
| Policy Kernel fuzzing | Rule evaluation | P0 |
| Receipt verification | Signature validation | P0 |
| Token lifecycle tests | Grant/revoke/expire | P0 |
| Approval flow tests | UI + backend | P1 |
| Penetration testing | Full system | P2 |

---

## 9. Incident Response

### 9.1 Current Capabilities

| Capability | Status |
|------------|--------|
| Token revocation | EXISTS (10min expiry, manual only) |
| Audit log query | EXISTS (AuditService) |
| Network isolation | EXISTS (connection limits) |
| Key rotation | PARTIAL (vault re-creation) |

### 9.2 Required Capabilities (Before Canvas)

| Capability | Purpose | Priority |
|------------|---------|----------|
| Immediate token revocation | Compromised agent response | P0 |
| Receipt export | Investigation support | P1 |
| Agent blacklist | Repeat offender blocking | P1 |
| Emergency shutdown | System-wide halt | P2 |

---

## 10. Summary

### Current System Security: GOOD

- PQC cryptography protects against future quantum threats
- Rust provides memory safety baseline
- Scope-limited tokens prevent broad agent access
- Rate limiting prevents resource exhaustion

### Canvas/Agent Expansion: REQUIRES WORK

Before Canvas Alpha:
- [ ] Policy Kernel for unified authorization
- [ ] Principal::Canvas type definition
- [ ] Receipt signing with ML-DSA-87
- [ ] Token revocation mechanism
- [ ] Scope validation enforcement

Before Canvas Beta:
- [ ] Approval Queue implementation
- [ ] Dioxus approval UI
- [ ] Per-principal rate limits
- [ ] Audit log export
- [ ] Anomaly detection framework

Before Canvas GA:
- [ ] Quarantine layer for untrusted code
- [ ] Trust escalation workflows
- [ ] Multi-agent coordination
- [ ] External security audit

---

## Appendix: Threat ID Reference

| ID Range | Category |
|----------|----------|
| T1.x | Identity & Authentication |
| T2.x | Data Security |
| T3.x | Network Security |
| T4.x | Application Security |
| T5.x | Agent-Specific (Future) |
| T6.x | Canvas-Specific (Future) |
| T7.x | Approval Queue (Future) |
| T8.x | Policy Kernel (Future) |

