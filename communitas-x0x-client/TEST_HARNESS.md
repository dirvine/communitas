# communitas-x0x-client Test Harness

`communitas-x0x-client` is the daemon-contract harness for `x0xd`.

It is the right layer for proving that Communitas talks to the **real** x0x daemon correctly, across:

- local scratch daemons
- all reachable VPS x0xd nodes
- REST + WebSocket surfaces

It is **not** the GUI harness. Dioxus and Swift parity still need their own app-level E2E.

## Scope

This harness is intended to validate the `communitas-x0x-client` transport contract against real daemons.

### Covered now

- core daemon health and identity surfaces
- discovery / presence / reachability
- WebSocket session establishment
- contact import / trust / machine pinning
- named groups + invites
- key-value stores
- task lists
- MLS group basics
- constitution / upgrade endpoints

### Covered conditionally

- direct messaging
- file transfer accept / reject lifecycle

Those two flows are wired into `live_mutation_contract.rs`, but they are currently skipped when the harness is running entirely on same-host local named instances. Same-host named daemons hairpin through external discovery addresses today, so the reliable place to validate those flows is a future scratch-VPS mutation lane.

### Intentionally not covered by this crate

Per `src/lib.rs`, SSE is still out of scope for the client crate:

- `/events`
- `/direct/events`
- `/presence/events`

If we want literal 100% daemon transport coverage, SSE support or a raw-SSE companion harness still needs to be added.

## Files

### Test support
- `tests/support/harness.rs`
  - loads target matrices from `X0X_TEST_MATRIX_FILE`
  - falls back to local auto-discovery when no matrix is provided
  - creates typed HTTP / WebSocket clients per target

### Read-only matrix suite
- `tests/live_matrix_contract.rs`
  - safe against shared daemons
  - loops across every configured target
  - validates core REST and WebSocket surfaces
  - validates multi-target discovery when 2+ targets are present

### Mutation suite
- `tests/live_mutation_contract.rs`
  - only runs when `X0X_TEST_ALLOW_MUTATION=1`
  - intended for ephemeral local scratch daemons
  - exercises the stateful lifecycle endpoints

### Runner
- `scripts/tests/x0x_client_contract_harness.sh`
  - `local`: starts 3 scratch named `x0xd` instances and runs read-only + mutation suites
  - `vps`: discovers configured VPS daemons over SSH and runs the read-only matrix suite
  - `all`: local first, then VPS

## Usage

### Local scratch harness

This starts three named local daemons, waits for them to become healthy, builds a target matrix, and runs the full stateful contract suite.

```bash
bash scripts/tests/x0x_client_contract_harness.sh local
```

What it does:
- starts 3 named instances with isolated data dirs
- waits for `api.port` + `api-token`
- runs:
  - `live_matrix_contract`
  - `live_mutation_contract`

Current note:
- local scratch validates the broad stateful API surface
- direct/file mutation checks are intentionally skipped on same-host local daemons until we add scratch VPS mutation coverage

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
- direct/file checks remain opt-in via `X0X_TEST_ENABLE_DIRECT_FILE=1` until those flows are made reliable in the scratch topology

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
- sends direct messages and files
- revokes contacts

Do **not** run that suite against shared production-like VPS daemons unless you deliberately want those side effects.

## Current release-quality posture

This harness gets us much closer to the right release gate:

1. **Local mutation proof** against real scratch `x0xd`
2. **Fleet-wide VPS contract proof** against real remote nodes
3. **Scratch VPS mutation proof** against ephemeral named instances on real VPS nodes
4. **Swift parity guard** via `tests/swift_parity.rs`
5. **API surface guard** via `tests/client_coverage.rs`

What is still missing for literal 100% daemon E2E:
- enabling direct messaging + file transfer in the scratch mutation lane by default
- SSE coverage

## Next gaps

To make this truly exhaustive, the next additions should be:

1. **SSE harness**
   - `/events`
   - `/direct/events`
   - `/presence/events`

2. **Dedicated VPS scratch instances**
   - ephemeral named daemons on every node
   - lets the mutation suite run remotely too

3. **Soak / resilience lane**
   - long-running matrix checks
   - connection churn
   - file-transfer retry validation
   - multi-node gossip persistence checks

4. **CI / workflow integration**
   - manual workflow_dispatch with SSH secrets
   - artifact upload of generated matrices and logs

That is the path to a true full x0xd API-contract harness.
