# Security Policy

## Supported Versions

We actively support and provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| main    | ✅ Current development |
| develop | ✅ Active development  |

## Security Standards

### Cryptographic Requirements

All encryption operations in Communitas MUST use the **saorsa-fec** crate. This is enforced through:

- Automated CI checks scanning for direct cryptographic library usage
- Code review requirements for security-critical modules
- Runtime validation of encryption implementations

### Critical Security Rules

1. **NO `unwrap()` or `expect()` in production code** - All error conditions must be handled gracefully
2. **NO `panic!()` in production code** - Use `Result` types and proper error propagation
3. **ALL encryption via saorsa-fec** - No direct use of ChaCha20Poly1305 or other crypto libraries
4. **NO hardcoded secrets** - All credentials must be externally configured
5. **Memory safety** - Rust's ownership system prevents many vulnerabilities, but we use additional tools like `zeroize` for sensitive data

### Automated Security Enforcement

Our CI pipeline includes:

- **Security vulnerability scanning** with `cargo audit`
- **Dependency license checking** with `cargo deny`
- **Static code analysis** for security anti-patterns
- **Hardcoded secrets detection**
- **Encryption compliance validation**

## Reporting a Vulnerability

### For Security Issues

If you discover a security vulnerability, please:

1. **DO NOT** open a public GitHub issue
2. **DO NOT** discuss the vulnerability publicly
3. **Email** security concerns to: [dirvine@maidsafe.net](mailto:dirvine@maidsafe.net)

Include in your report:
- Description of the vulnerability
- Steps to reproduce (if applicable)
- Potential impact assessment
- Suggested fix (if you have one)

### Response Timeline

- **Initial response**: Within 24 hours
- **Vulnerability assessment**: Within 72 hours  
- **Fix timeline**: Depends on severity
  - Critical: Within 7 days
  - High: Within 14 days
  - Medium: Within 30 days
  - Low: Next planned release

### Security Severity Classification

**Critical**: Remote code execution, privilege escalation, cryptographic bypass
- Immediate patching required
- Public disclosure after fix is deployed

**High**: Data exposure, authentication bypass, significant DoS
- Fast-track patching within 2 weeks
- Coordinated disclosure

**Medium**: Limited information disclosure, local privilege escalation
- Regular patching cycle
- Standard disclosure process

**Low**: Minor information leaks, configuration issues
- Next release cycle
- Public discussion acceptable after fix

## Security Architecture

### Encryption Architecture

Communitas implements defense-in-depth encryption:

1. **Transport Layer**: WebRTC-over-QUIC with ant-quic NAT traversal
2. **Application Layer**: saorsa-fec with ChaCha20-Poly1305 AEAD
3. **Storage Layer**: Reed-Solomon FEC with encrypted shards
4. **Identity Layer**: Ed25519 cryptographic identities

### Key Management

- **Key Generation**: Cryptographically secure random generation
- **Key Storage**: Platform-specific secure storage (Keychain, Credential Manager, etc.)
- **Key Rotation**: Automatic rotation for long-lived keys
- **Key Zeroization**: Sensitive material cleared from memory

### Network Security

- **P2P Security**: DHT-based routing with cryptographic node IDs
- **Geographic Routing**: Location-aware peer selection
- **NAT Traversal**: ant-quic protocol for direct peer connections
- **Message Authentication**: Digital signatures on all messages

### Memory Safety

- **Rust Ownership**: Compile-time memory safety guarantees
- **Zeroization**: Sensitive data automatically cleared
- **Constant-time Operations**: Timing attack resistance
- **Stack Protection**: Compiler-level stack overflow protection

## Security Testing

### Automated Testing

Our CI pipeline runs:

- Unit tests for all cryptographic operations
- Integration tests for P2P security
- Property-based testing with QuickCheck
- Fuzzing for input validation
- Static analysis with Clippy security lints

### Manual Security Review

Required for:
- All cryptographic code changes
- Network protocol modifications
- Identity system updates
- Storage layer changes
- CI/CD pipeline modifications

### Security Audits

We perform regular security audits of:
- Cryptographic implementations
- P2P network protocols  
- Key management systems
- Attack surface analysis

## Dependency Security

### Dependency Management

- **Vulnerability Scanning**: Weekly automated scans
- **License Compliance**: Approved license list enforcement
- **Supply Chain Security**: Hash verification of dependencies
- **Minimal Dependencies**: Reduce attack surface

### Approved Cryptographic Libraries

Only these cryptographic libraries are approved:

- **saorsa-fec**: Primary encryption library (REQUIRED)
- **ChaCha20Poly1305**: Only via saorsa-fec wrapper
- **Ed25519**: For digital signatures in saorsa-core
- **Blake3**: For cryptographic hashing
- **rand**: For secure random number generation

### Prohibited Practices

❌ Direct use of:
- OpenSSL (prefer Rust-native alternatives)
- Custom cryptographic implementations
- Deprecated cryptographic algorithms
- Weak random number generators

## Incident Response

### Security Incident Procedure

1. **Detection**: Automated monitoring and manual reporting
2. **Assessment**: Severity classification and impact analysis
3. **Containment**: Immediate measures to limit damage
4. **Eradication**: Remove the vulnerability from all systems
5. **Recovery**: Restore normal operations securely
6. **Lessons Learned**: Post-incident analysis and improvements

### Communication Plan

- **Internal Team**: Immediate notification via secure channels
- **Users**: Coordinated disclosure after fix deployment
- **Community**: Security advisory with mitigation steps
- **Authorities**: As required by applicable regulations

## Compliance and Standards

### Security Frameworks

We align with:
- **OWASP Top 10**: Web application security risks
- **NIST Cybersecurity Framework**: Risk management approach
- **ISO 27001**: Information security management
- **GDPR**: Data protection and privacy

### Audit Trail

All security-relevant events are logged:
- Authentication attempts
- Cryptographic operations
- Network connections
- Data access patterns
- Configuration changes

## Security Contact

For all security-related matters:

**Primary Contact**: David Irvine
- Email: [dirvine@maidsafe.net](mailto:dirvine@maidsafe.net)
- GitHub: [@dirvine](https://github.com/dirvine)

**GPG Key**: Available on request for encrypted communications

---

**This security policy is reviewed quarterly and updated as needed to reflect current threats and best practices.**

Last Updated: January 2025