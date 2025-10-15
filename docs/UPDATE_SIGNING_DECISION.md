# Update Signing Decision - Ed25519 for Now

## Decision Made: 2025-10-15

**Decision**: Use Ed25519 signatures (via Tauri's standard updater) for update signing, with plans to migrate to PQC signatures in the future.

## Rationale

### Why Ed25519 Now
1. **Fast Implementation**: Zero additional work beyond standard Tauri setup
2. **Battle-Tested**: Well-proven system used by thousands of applications
3. **Fully Supported**: Native Tauri integration with automatic verification
4. **Time to Market**: Allows immediate deployment of update system
5. **Classical Security**: Ed25519 remains secure against all known classical attacks

### Acknowledged Trade-offs
- ❌ Not quantum-resistant (vulnerable to Shor's algorithm on quantum computers)
- ⚠️ Inconsistent with rest of system (which uses PQC throughout)
- ⚠️ Updates become potential weak link in long-term security

### Risk Assessment
- **Timeline**: Quantum computers capable of breaking Ed25519 estimated 10-30 years away
- **Mitigation**: Can migrate to PQC signatures before quantum threat materializes
- **Impact**: Update signatures could be forged by sufficiently powerful quantum computer
- **Probability**: Low risk in near-term (5-10 years)

## Current Implementation

### Signature Algorithm
- **Algorithm**: Ed25519 (EdDSA using Curve25519)
- **Key Size**: 256-bit (32 bytes)
- **Signature Size**: 512-bit (64 bytes)
- **Library**: Minisign (via Tauri updater plugin)

### Security Properties
- ✅ Collision-resistant
- ✅ Strong against classical attacks
- ✅ Fast verification
- ✅ Small signatures
- ❌ Vulnerable to quantum attacks (Shor's algorithm)

## Future Migration Path

### Phase 1: Current (2025)
- Use Ed25519 signatures exclusively
- Deploy update system to production
- Document migration plan

### Phase 2: Dual-Signature (2026-2027)
When quantum threat becomes more imminent:
1. Add ML-DSA-65 signatures alongside Ed25519
2. Verify both signatures before accepting updates
3. See `docs/PQC_UPDATE_SIGNING_ANALYSIS.md` for implementation plan
4. **Effort**: 2-3 days implementation

### Phase 3: Pure PQC (2028+)
When quantum computers approach practical capability:
1. Either fork Tauri updater plugin for ML-DSA support
2. Or implement custom update system with pure PQC
3. Drop Ed25519 completely
4. **Effort**: 2-4 weeks implementation

## Monitoring & Triggers

### When to Reconsider
Monitor these triggers for migration:
- [ ] NIST announces advances in quantum computing
- [ ] First demonstration of Shor's algorithm at scale
- [ ] Industry moves to mandate PQC in software distribution
- [ ] Tauri adds native PQC signature support
- [ ] Customer/regulatory requirements for PQC compliance

### Regular Review
- **Frequency**: Annual review of quantum threat landscape
- **Owner**: Security team
- **Next Review**: 2026-01-01

## Related Documents

- `docs/PQC_UPDATE_SIGNING_ANALYSIS.md` - Full analysis of all options
- `docs/UPDATE_SYSTEM_SETUP.md` - Implementation guide
- `SELF_UPDATE_AND_MESH_PLAN.md` - Overall project plan

## Decision Log

| Date | Decision | Made By | Status |
|------|----------|---------|--------|
| 2025-10-15 | Use Ed25519 for initial release | David Irvine | ✅ Active |
| TBD | Migrate to dual-signature | TBD | 📋 Planned |
| TBD | Migrate to pure PQC | TBD | 📋 Future |

## Technical Debt

This decision creates technical debt that should be tracked:

**Debt Item**: Update signatures use classical cryptography (Ed25519)
**Risk**: Medium-Low (10+ year timeline)
**Effort to Fix**: 2-3 days (dual-signature) or 3-4 weeks (pure PQC)
**Priority**: Low (monitor quantum computing progress)
**Mitigation**: Annual review, documented migration path

## Conclusion

Using Ed25519 for update signing is the right pragmatic choice for 2025:
- ✅ Allows immediate deployment
- ✅ Secure for foreseeable future (10+ years)
- ✅ Clear migration path exists
- ✅ Can adapt based on quantum threat evolution

We acknowledge this introduces a quantum-vulnerable component but accept this risk given:
1. Timeline to quantum threat (10-30 years)
2. Clear migration path available
3. Ability to react before threat materializes
4. Cost/benefit of delaying deployment

**Status**: Decision approved, proceed with Ed25519 implementation.
