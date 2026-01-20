# Issues Backlog

_Last updated: 2026-01-19_

## P0: Blockers (Immediate)

None currently.

## P1: Next Milestone (M5 - Stabilization)

- [ ] Signed installer pipeline (macOS notarization, Windows code signing)
- [ ] Linux packaging (AppImage, deb, rpm)
- [ ] WebView bootstrap scripts for first-run experience
- [ ] Telemetry dashboard setup
- [ ] MCP regression suite automation

## P2: Technical Debt

- [ ] Call networking integration (deferred from M3, requires live network)
- [ ] Performance benchmarking under load (60fps validation)
- [ ] Memory usage profiling (<200MB baseline target)
- [ ] Bundle size optimization

## P3: Future Enhancements

- [ ] NAT traversal coordination improvements
- [ ] Relay node selection optimization
- [ ] Geographic peer preference
- [ ] Peer reputation/trust scoring
- [ ] Bootstrap cache sharing across devices (same identity)
- [ ] Localization scaffolding (i18n)
- [ ] Demo mode telemetry guardrails

---

## Resolved (M1-M4)

- [x] saorsa-gossip cache exposure (v0.2.1)
- [x] PeerListRequest/Response message format
- [x] Peer info fields (addr, score, nat_class, roles)
- [x] Address reflection (automatic via ant-quic)
- [x] Design token system
- [x] Accessibility P1 remediation
- [x] MCP-Dioxus parity for all surfaces
