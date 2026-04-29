# Communitas Apple — Parity Matrix Evidence

This file records the test-and-surface evidence that closes the
🟡 cells in the **Apple column** of `x0x/docs/parity-matrix.md`.

The x0x team merges this into the matrix during their next parity
audit. Do not edit `x0x/docs/parity-matrix.md` directly from this
repo — the apple stream owns this evidence file, the dioxus stream
owns its own, and the x0x team merges them together to avoid PR
conflicts across streams.

**Run summary** (`proofs/apple-parity-20260428/`):

- `swift-test-gated.log`: `XCUITEST_SKIP=1 swift test` — package
  build plus decoder/smoke suites with live-daemon tests gated off.
- `swift-test-live.log`: `XCUITEST_SKIP=1 X0X_LIVE_TESTS=1 X0XD_BIN=…
  X0X_BIN=… swift test` — decoder-only suites + live round-trip suites
  against `x0xd 0.19.7`.

XCUITest target compiles in the Swift Package; running the actual
UI tests requires `xcodebuild -scheme Communitas -destination
'platform=macOS' -only-testing:CommunitasUITests test` against an
Xcode-generated project. Under `swift test` (no app host) the suite
is gated behind `XCUITEST_SKIP=1` so package CI stays green. The
XCUITest methods are UI-surface smoke checks; the live Swift Testing
suites below carry the daemon wire-proof.

---

## Cell-by-cell evidence

### Cell 1 — Identity / Export keypairs

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/SettingsView.swift` | "Export Identity Backup…" button (`export-keypair-button`) writes a private-key backup only after an explicit `NSSavePanel` confirmation |
| Helper | `Sources/X0xClient/IdentityBackup.swift` | `IdentityBackupExporter` reads `agent.key`, `machine.key`, optional `user.key`, optional `agent.cert`, and optional `agent_kem.key`; agent cards are explicitly not treated as backups |
| Round-trip test | `Tests/X0xClientTests/IdentityExportRoundTripTests.swift` | `keypairBackupContainsPrivateIdentityFiles` — boots a live `DaemonFixture`, exports the fixture key files into a JSON backup bundle, writes/decodes it, and asserts private key entries are present |
| Card import regression | `Tests/X0xClientTests/IdentityExportRoundTripTests.swift` | `agentCardExportAndImportRoundTrip` — keeps the public agent-card export/import path tested separately from private key backup |
| XCUITest smoke | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testIdentityExportSurfaceReachable` |

### Cell 2 — Connectivity / Connect to agent

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ContactsView.swift` | "Connect" button on each discovered-agent row (`connect-agent-button-<id>`) wired to `connectAgent` |
| UI | `Sources/Communitas/Views/ContactsView.swift` | Direct connections counter (`direct-connections-count`) refreshed after connect |
| Round-trip test | `Tests/X0xClientTests/ConnectivityRoundTripTests.swift` | `connectAgentSurfaceReachable`, `directConnectionsListIncludesSelf` |
| XCUITest | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testConnectAgentSurfaceReachable` |

### Cell 3 — Connectivity / Discover agents

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ContactsView.swift` | Discovered-agents section (`discovered-agents-list`, `discovered-agents-count`) |
| UI | `Sources/Communitas/Views/DashboardView.swift` | Identity card (`agent-id-display`) shows current agent post-launch |
| Round-trip test | `Tests/X0xClientTests/ConnectivityRoundTripTests.swift` | `discoverAgentsIncludesSelfAfterAnnounce` |
| XCUITest | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testDiscoverAgentsListPresent` |

### Cell 4 — Connectivity / Four-word network bootstrap

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ParityViews.swift` | `FourWordBootstrapView` — 4 input fields (`four-word-w1…w4`), connect button (`four-word-connect-button`), output panel (`four-word-output`) |
| Helper | `Sources/Communitas/Views/ParityViews.swift` | `FourWordResolver` — locates the `x0x` CLI binary, runs `x0x connect <words>` as a subprocess so Swift reuses the Rust `four_word_networking::FourWordAdaptiveEncoder` decoder rather than duplicating the dictionary |
| Round-trip test | `Tests/X0xClientTests/ConnectivityRoundTripTests.swift` | `fourWordBootstrapBinarySurface` — probes `x0x --version` then exercises the malformed-input error path of `x0x connect` |
| XCUITest | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testFourWordBootstrapInputPresent` |

### Cell 5 — Pub/sub / WebSocket live feed

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ParityViews.swift` | `LiveFeedView` + `LiveFeedModel` — subscribe via REST, open `X0xWebSocket(/ws)`, parse base64 payload from gossip frames into `pubsub-last-received`. Includes inline publisher (`pubsub-publish`). Topic input (`pubsub-topic`), payload editor (`pubsub-payload`), frames list (`live-feed-frames`) |
| Wiring | `Sources/Communitas/Views/DashboardView.swift` | "Open Pub/Sub" button (`open-pubsub`) navigates to LiveFeedView |
| Round-trip test | `Tests/X0xClientTests/PubSubRoundTripTests.swift` | `subscribeAndPublishRoundTrip`, `webSocketSessionListEndToEnd`, `wsSchemeConversion` |
| XCUITest | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testPublishAndSubscribeTopic`, `testLiveFeedReachable` |

### Cell 6 — Direct / File transfer

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/FilesView.swift` | Existing space-scoped Files view — accept (`file-accept-button-<id>`), reject (`file-reject-button-<id>`), choose-file (`file-send-button`), incoming row (`file-incoming-row-<id>`), transfer list (`file-transfer-list`) |
| Round-trip test | `Tests/X0xClientTests/FileTransferRoundTripTests.swift` | `listTransfersDecodesEmpty`, `sendFileToDisconnectedPeerErrors` (proves the structured 500-error envelope round-trips through the Swift `X0xError` path), `rejectFileMethodVariants` (covers both `rejectFile(transferId:)` and `rejectFile(transferId:reason:)`) |
| XCUITest smoke | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testFileTransferSendButtonPresent` — package-host smoke only; byte-level transfer remains covered by the x0x live-network script noted below |

> Note: Two-daemon byte-level transfer requires an active QUIC peer
> connection between two `x0xd` instances. The localhost ephemeral
> `DaemonFixture` boots with empty bootstrap lists by design and so
> never establishes a direct connection. End-to-end byte-level proof
> for the matrix lives in `tests/e2e_live_network.sh` against the
> real bootstrap fleet — the Swift round-trip tests above cover the
> Apple-column wire-shape contract.

### Cell 7 — Groups / Policy (roles, bans)

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ManageGroupSheet.swift` | Five-axis policy form + Apply button (`apply-policy-button`, `group-policy-section`); per-member ban (`member-ban-button-<id>`), unban (`member-unban-button-<id>`), promote (`member-promote-button-<id>`) |
| Wiring | `Sources/Communitas/Views/GroupsView.swift` | "Manage…" context-menu entry on each group row opens the sheet |
| Round-trip test | `Tests/X0xClientTests/GroupsRoundTripTests.swift` | `updateGroupPolicyRoundTrip` (PATCH /policy then read-back), `banUnbanMemberRoundTrip` (add → ban → confirm → unban → confirm) |
| XCUITest smoke | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testGroupPolicySurfaceReachable` — verifies the Groups surface is reachable; the live Swift test above proves policy/ban wire behavior |

> The Swift `NamedGroupMember.updatedAt` field was changed from
> `UInt64` to `UInt64?` to match the daemon's actual `GET
> /groups/:id/members` wire shape (see
> `src/bin/x0xd.rs::named_group_member_values` which omits
> `updated_at` from the public roster). Without that fix the
> ban-unban round-trip failed with a `keyNotFound` decode error.

### Cell 8 — Groups / Discover groups (tag/nearby)

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/GroupDiscoveryView.swift` | Query field (`group-discover-query`), mode picker (`group-discover-mode-picker`), result list (`group-discover-list`) and per-row entries (`group-discover-row-<id>`) |
| Wiring | `Sources/Communitas/Views/GroupsView.swift` | "Discover" toolbar entry (`group-discover-toggle`) opens the sheet |
| Round-trip test | `Tests/X0xClientTests/GroupsRoundTripTests.swift` | `discoverableGroupAppearsInIndex` — creates a `public_open` group, polls `discoverGroups()` until it appears, and asserts `discoverGroupsNearby()` decodes (empty tolerated on isolated CI) |
| XCUITest | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testGroupDiscoverSurfaceReachable` |

### Cell 9 — Presence / FOAF walk

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ContactsView.swift` | "Run FOAF Walk" button (`presence-foaf-button`), status label (`presence-foaf-status`), per-result rows (`presence-foaf-result-<id>`) |
| Round-trip test | `Tests/X0xClientTests/PresenceRoundTripTests.swift` | `foafWalkDecodes` |
| XCUITest | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testPresenceFoafButtonPresent` |

### Cell 10 — Presence / Status & reachability

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ContactsView.swift` | "Inspect" button (`inspect-agent-button-<id>`) opens reachability + presence-status diagnostic — surfaces both `presenceStatus` and `agentReachability` into a `presence-status-text` label |
| Round-trip test | `Tests/X0xClientTests/PresenceRoundTripTests.swift` | `presenceStatusForSelf` |
| XCUITest smoke | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testPresenceStatusSurfaceReachable` — verifies the People/presence surface is reachable; the live Swift test above proves status/reachability decoding |

### Cell 11 — Presence / Events SSE

| Surface | File | Symbol |
|---|---|---|
| UI | `Sources/Communitas/Views/ParityViews.swift` | `PresenceToastModel` consumes `X0xSseStream.connectPresence`; `PresenceToastView` renders a transient banner (`presence-event-toast`) overlaid on the main window |
| Wiring | `Sources/Communitas/ContentView.swift` | `@StateObject presenceToast` + `.task { presenceToast.start(config:) }`; `.onDisappear { presenceToast.stop() }`; conditional overlay alignment top |
| Round-trip test | `Tests/X0xClientTests/PresenceRoundTripTests.swift` | `presenceSseConnects` — opens `/presence/events` against a live daemon and tolerates an empty stream within a 3 s window |
| XCUITest smoke | `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` | `testPresenceSseToastWiring` — verifies the app launches with the toast model wired; `presenceSseConnects` is the live SSE proof |

---

## Wire-shape changes

- `Sources/X0xClient/Models/Group.swift::NamedGroupMember.updatedAt`
  changed from `UInt64` to `UInt64?`. Reason: the daemon's public
  roster endpoint (`GET /groups/:id/members`) omits `updated_at` —
  it's only emitted by the admin-scope `…_all` variant in
  `src/bin/x0xd.rs`. This matches `removed_by` and
  `kem_public_key_b64`, which were already optional.

## New X0xClient surfaces

No new REST methods were added to `Sources/X0xClient/X0xClient.swift` —
the matrix cells ride the existing daemon API surface. A local-only
`IdentityBackupExporter` helper was added in
`Sources/X0xClient/IdentityBackup.swift` so the Apple app can back up
private key files without misusing public agent cards as key backups.

## File inventory (new + modified)

**New files**:

- `Sources/Communitas/Views/ParityViews.swift` — bundles
  `LiveFeedView`, `KvStoresView`, `FourWordBootstrapView`,
  `FourWordResolver`, `PresenceToastModel`, `PresenceToastView`.
- `Tests/X0xClientTests/ConnectivityRoundTripTests.swift` —
  cells 2/3/4 round-trip suite.
- `Tests/X0xClientTests/PubSubRoundTripTests.swift` — cell 5.
- `Tests/X0xClientTests/FileTransferRoundTripTests.swift` — cell 6.
- `Tests/X0xClientTests/GroupsRoundTripTests.swift` — cells 7/8.
- `Tests/X0xClientTests/PresenceRoundTripTests.swift` — cells 9/10/11.
- `Sources/X0xClient/IdentityBackup.swift` — local private-key backup bundle helper.
- `Tests/X0xClientTests/IdentityExportRoundTripTests.swift` — cell 1.
- `proofs/apple-parity-20260428/{swift-test-gated.log,swift-test-live.log,run.txt}`.

**Modified**:

- `Sources/Communitas/CommunitasApp.swift` — no functional change.
- `Sources/Communitas/ContentView.swift` — wires `PresenceToastModel`
  and the new `SystemPage` cases (`liveFeed`, `kvStores`, `fourWord`,
  `groups`).
- `Sources/Communitas/Models/NavigationItem.swift` — adds
  `groups`, `liveFeed`, `kvStores`, `fourWord` cases.
- `Sources/Communitas/Views/DashboardView.swift` — accessibility ids
  (`agent-id-display`, `machine-id-display`, `dashboard-create-space`,
  `open-pubsub`, `open-groups`, `open-kv-stores`).
- `Sources/Communitas/Views/SettingsView.swift` — private identity
  backup export plus public agent-card import flow (`export-keypair-button`,
  `import-keypair-button`, `import-keypair-buffer`,
  `import-keypair-confirm`, `export-keypair-status`, `imported-agent-id`,
  `settings-agent-id`, `settings-machine-id`, `settings-agent-card-link`).
- `Sources/Communitas/Views/ContactsView.swift` — Connect / FOAF /
  presence-status / compose-DM surfaces, all accessibility ids
  required by the matrix.
- `Sources/Communitas/Views/FilesView.swift` — accessibility ids on
  send / accept / reject / list controls.
- `Sources/Communitas/Views/ManageGroupSheet.swift` — accessibility
  ids on policy + roster controls.
- `Sources/Communitas/Views/GroupDiscoveryView.swift` — accessibility
  ids on query / mode picker / list / per-row entries.
- `Sources/Communitas/Views/GroupsView.swift` — accessibility ids on
  toolbar (`group-create-new`, `group-discover-toggle`), per-row
  (`group-row-<id>`, `group-row-title`), create form
  (`group-name`, `group-create-confirm`).
- `Sources/X0xClient/Models/Group.swift` — `NamedGroupMember.updatedAt`
  → optional (see "Wire-shape changes").
- `Tests/CommunitasUITests/CommunitasGoldenPathsUITests.swift` —
  promoted from "5 SKIP-prone golden paths" to 16 UI-surface smoke
  methods. Under Swift Package testing these are compiled and skipped;
  full UI execution still requires an Xcode-hosted XCUITest run.

## Validation transcript

```
$ XCUITEST_SKIP=1 swift test --package-path communitas-apple
✔ Test run with 87 tests in 10 suites passed after 0.006 seconds.

$ XCUITEST_SKIP=1 \
  X0X_LIVE_TESTS=1 \
  X0XD_BIN=$WORKSPACE/x0x/target/release/x0xd \
  X0X_BIN=$WORKSPACE/x0x/target/release/x0x \
  swift test --package-path communitas-apple
✔ Test run with 87 tests in 10 suites passed after 3.391 seconds.
```

Both transcripts archived under `proofs/apple-parity-20260428/`.

## Out of scope

- `xcodebuild`-driven XCUITest run: deferred — `Package.swift`
  alone does not generate an Xcode project with a host application
  the XCUITest target can drive. Generating an `.xcodeproj`/Xcode
  workspace + signing the host app is tracked in
  `docs/next-session-communitas-parity.md`.
- Two-daemon file transfer: requires real QUIC peer connectivity
  (covered by `tests/e2e_live_network.sh` in the `x0x` repo).
- Edits to `x0x/docs/parity-matrix.md`: by design — the x0x team
  merges this evidence file in to avoid PR conflicts across the
  Apple / Dioxus / GUI streams.
