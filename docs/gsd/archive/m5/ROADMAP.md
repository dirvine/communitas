# Communitas Roadmap

_Last updated: 2026-01-21_

## Current Milestone: M5 Stabilization

**Scope**: Packaging, installers, telemetry, perf QA, MCP validation

**Key deliverables** (from dioxus_desktop_prototype_plan.md):
- Signed installers (macOS DMG, notarized)
- Runtime WebView provisioning
- Performance benchmarks vs production baseline
- MCP regression suite

### Phases

| Phase | Description | Status |
|-------|-------------|--------|
| **5.1** | Packaging & Installers | ✅ Complete |
| **5.2** | WebView Provisioning | ✅ Complete |
| **5.3** | Performance QA | ✅ Complete |
| **5.4** | MCP Validation Suite | ✅ Complete |

---

## Phase 5.1: Packaging & Installers ✅

**Goal**: Produce signed, distributable installers for macOS.

**Completed**:
1. ✅ Created macOS app icon (SVG → PNG → ICNS)
2. ✅ Configured Dioxus.toml bundle settings
3. ✅ Created entitlements.plist for hardened runtime
4. ✅ Created release-desktop.yml workflow (build, sign, notarize, DMG)
5. ✅ Added smoke-test-dmg.sh for verification
6. ✅ Documented bundle process in docs/deployment/

**Artifacts**:
- `.github/workflows/release-desktop.yml` - Full CI/CD pipeline
- `communitas-dioxus/entitlements.plist` - macOS sandbox permissions
- `scripts/generate-icon.sh` - Icon generation from SVG
- `scripts/tests/smoke-test-dmg.sh` - DMG verification
- `docs/deployment/macos-desktop-bundle.md` - Complete documentation

---

## Phase 5.2: WebView Provisioning

**Goal**: Handle missing WebView dependencies gracefully.

**Tasks**:
1. Add WebView detection at startup
2. Implement bootstrap/install prompts
3. Block UI until dependency satisfied
4. Test on clean macOS VMs

---

## Phase 5.3: Performance QA

**Goal**: Establish performance baselines and benchmarks.

**Tasks**:
1. Instrument core flows (auth, messaging, kanban, drive)
2. Define performance targets (<100ms local, <500ms remote)
3. Create benchmark suite with criterion
4. Add CI performance regression alerts (>5% deviation)
5. Document baseline measurements

---

## Phase 5.4: MCP Validation Suite

**Goal**: Comprehensive MCP regression testing.

**Tasks**:
1. Expand parity tests to cover all MCP tools
2. Add golden data comparisons
3. Create MCP-driven E2E test workflows
4. Document MCP testing strategy

---

## Completed Milestones

| Milestone | Description | Status |
|-----------|-------------|--------|
| **M0** | Discovery & Inventory | ✅ Complete |
| **M1** | Bootstrap Node Enhancement | ✅ Complete |
| **M2** | Messaging & Entities | ✅ Complete |
| **M3** | Advanced Surfaces (UI) | ✅ Complete |
| **M4** | Polish & Performance (UI) | ✅ Complete |
| **M3.1** | Remediation - Wire Services | ✅ Complete (47 tasks) |

---

## Reference

- Main plan: `docs/architecture/dioxus_desktop_prototype_plan.md`
- M3.1 archive: `docs/gsd/archive/m3.1/`
- Legacy archive: `.planning/archive/`
