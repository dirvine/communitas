# communitas-x0x-client Test Harness

`communitas-x0x-client` is the daemon-contract harness for `x0xd`.

It is the right layer for proving that Communitas talks to the **real** x0x daemon correctly, across:

- local scratch daemons
- all reachable VPS x0xd nodes
- REST + WebSocket + SSE surfaces

It is **not** the GUI harness. Dioxus and Swift parity still need their own app-level E2E.

## Scope

This harness is intended to validate the `communitas-x0x-client` transport contract against real daemons.

### Covered now

- core daemon health and identity surfaces
- discovery / presence / reachability
- WebSocket session establishment and live message flow
- SSE connection + event delivery for `/events`, `/direct/events`, and `/presence/events`
- contact import / trust / machine pinning / revocation
- named groups / invites / space-chat topic plumbing
- key-value stores lifecycle
- task lists lifecycle
- MLS group member add/remove + encrypt/decrypt
- direct messaging
- file transfer accept / reject lifecycle
- constitution / upgrade endpoints
- graceful shutdown on ephemeral targets

### Covered in the reference x0x test suite

Some semantics are better proven inside canonical `../x0x` tests than through the daemon API harness alone. For release validation we also run:

- `tests/crdt_convergence_concurrent.rs`
- `tests/crdt_partition_tolerance.rs`
- `tests/proptest_crdt.rs`
- `tests/trust_evaluation_test.rs`
- `tests/proptest_groups.rs`
- `tests/proptest_presence.rs`

That gives us explicit CRDT merge / convergence coverage, trust semantics coverage, and property-based checks against the canonical implementation without modifying `../x0x`.

### Current topology caveat

Cross-host scratch daemons on the VPS fleet use ephemeral QUIC ports that are not reliably firewall-exposed, so the remote scratch mutation lane validates broad stateful REST/SSE/WS contract behavior but does **not** currently prove direct/file delivery between those ephemeral VPS daemons. Those real-time delivery paths are validated in the local scratch mesh, where we now seed named instances through an explicit bootstrap peer.

## Files

### Test support
- `tests/support/harness.rs`
  - loads target matrices from `X0X_TEST_MATRIX_FILE`
  - falls back to local auto-discovery when no matrix is provided
  - creates typed HTTP / WebSocket / SSE clients per target

### Read-only matrix suite
- `tests/live_matrix_contract.rs`
  - safe against shared daemons
  - loops across every configured target
  - validates core REST, WebSocket, and SSE connection surfaces
  - validates multi-target discovery when 2+ targets are present

### Mutation suite
- `tests/live_mutation_contract.rs`
  - only runs when `X0X_TEST_ALLOW_MUTATION=1`
  - intended for ephemeral local scratch daemons
  - exercises the stateful lifecycle endpoints

### Runner
- `scripts/tests/x0x_client_contract_harness.sh`
  - `local`: starts a 3-node local scratch mesh and runs read-only + mutation suites
  - `vps`: discovers configured VPS daemons over SSH and runs the read-only matrix suite
  - `vps-mutation`: starts 3 scratch VPS daemons and runs the mutation suite
  - `all`: local first, then VPS, then scratch VPS mutation
- `scripts/tests/x0x_release_validation.sh`
  - runs the full communitas-x0x-client harness
  - then runs canonical `../x0x` semantic tests for CRDT/trust/group/presence coverage

## Usage

### Local scratch harness

This starts three named local daemons, waits for them to become healthy, builds a target matrix, and runs the full stateful contract suite.

```bash
bash scripts/tests/x0x_client_contract_harness.sh local
```

What it does:
- starts 3 named instances with isolated data dirs
- builds them into a scratch mesh via an explicit bootstrap peer
- waits for `api.port` + `api-token`
- runs:
  - `live_matrix_contract`
  - `live_mutation_contract`

Current note:
- local scratch is the lane that validates direct messaging + file transfer delivery
- it also validates SSE delivery and the space-chat topic path

### VPS matrix harness

This discovers all configured VPS nodes over SSH, reads each node's `api.port` and `api-token`, builds a matrix, and runs the read-only contract suite.

```bash
bash scripts/tests/x0x_client_contract_harness.sh vps
```

What it does:
- targets the Saorsa Labs VPS fleet
- uses the real daemon state on each node
- avoids destructive local-state mutation on shared VPS nodes

### Scratch VPS mutation harness

This starts dedicated named `x0xd` scratch instances on three compatible VPS nodes, tunnels their loopback APIs back to the local machine, then runs the stateful mutation suite against those ephemeral daemons.

```bash
bash scripts/tests/x0x_client_contract_harness.sh vps-mutation
```

Current note:
- this proves the mutation harness can run against real remote daemons
- because scratch VPS daemons sit on ephemeral QUIC ports, this lane currently validates broad stateful API behavior rather than direct/file delivery between the remote scratch daemons themselves

### Full run

```bash
bash scripts/tests/x0x_client_contract_harness.sh all
```

## Matrix format

The runner writes a JSON file like this and passes it through `X0X_TEST_MATRIX_FILE`:

```json
{
  "targets": [
    {
      "name": "saorsa-2",
      "address": "127.0.0.1:12700",
      "token": "...",
      "role": "bootstrap",
      "region": "NYC1, US",
      "kind": "remote"
    }
  ]
}
```

You can also create one manually and run a suite directly:

```bash
X0X_TEST_MATRIX_FILE=/tmp/matrix.json \
  cargo test -p communitas-x0x-client --test live_matrix_contract -- --ignored
```

## Safety model

### Safe on shared VPS nodes
`live_matrix_contract.rs`

This suite only uses:
- reads
- announcements
- websocket connects
- discovery queries

It is designed to be safe against long-lived shared daemons.

### Only for ephemeral scratch daemons
`live_mutation_contract.rs`

This suite mutates daemon state:
- imports contacts
- updates trust
- adds machine records
- creates groups/stores/task lists/MLS groups
- exercises SSE/WS messaging paths
- sends direct messages and files where topology allows
- revokes contacts
- shuts down an ephemeral target

Do **not** run that suite against shared production-like VPS daemons unless you deliberately want those side effects.

## Current release-quality posture

This harness gets us much closer to the right release gate:

1. **Local mutation proof** against a real 3-node scratch mesh
2. **Fleet-wide VPS contract proof** against real remote nodes
3. **Scratch VPS mutation proof** against ephemeral named instances on real VPS nodes
4. **Swift parity guard** via `tests/swift_parity.rs`
5. **API surface guard** via `tests/client_coverage.rs`
6. **Canonical x0x semantic proof** via the reference CRDT / trust / group / presence tests

What is still missing for literal 100% daemon E2E through `communitas-x0x-client` alone:
- cross-host direct/file delivery on scratch VPS daemons with firewall-exposed QUIC ports
- a daemon API for explicit task-list joining, which would make multi-node CRDT API testing stronger

## Next gaps

To make this even stronger, the next additions should be:

1. **Cross-host scratch direct/file lane**
   - remote scratch daemons with firewall-exposed QUIC ports
   - proves direct/file delivery on the VPS fleet itself

2. **Task-list join endpoint in x0xd**
   - would let the client harness prove multi-node CRDT convergence through the public API

3. **Soak / resilience lane**
   - long-running matrix checks
   - connection churn
   - file-transfer retry validation
   - multi-node gossip persistence checks

4. **CI / workflow integration**
   - manual workflow_dispatch with SSH secrets
   - artifact upload of generated matrices and logs

That is the path to a true full x0xd API-contract harness plus semantic release gate.
