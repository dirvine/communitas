# Communitas Dioxus x0x Parity Evidence

This file records Dioxus-specific parity evidence for the remaining Dioxus-column yellow cells in `../x0x/docs/parity-matrix.md`.

The Dioxus app consumes `communitas-x0x-client` directly. The E2E scaffold added here launches the real `communitas-dioxus` binary in `e2e-test-mode`, points it at a fresh `x0xd` fixture, and drives line-delimited JSON commands through the binary. Each command calls the same typed x0x client surface the UI uses, with daemon B used for remote-card and negative access-policy checks.

Run:

```bash
just e2e
```

Proof bundles land under `proofs/dioxus-parity-YYYYMMDD/` and include `stdout.log` / `stderr.log` from the full harness run.

## Harness files

- `tests/e2e/harness.rs` — boots two `x0x-test-harness::DaemonFixture` instances and launches the Dioxus binary against fixture A.
- `src/e2e_test_mode.rs` — feature-gated headless JSON driver (`COMMUNITAS_TEST_MODE=1`, feature `e2e-test-mode`).
- `tests/e2e.sh` — builds `x0xd`, builds the Dioxus binary with `e2e-test-mode`, runs all row tests, and archives proof logs.
- `justfile` / `../justfile` — `just e2e` entry points.

## Identity

### Get agent id / card

- Test: `tests/e2e/identity.rs::dioxus_parity_identity_get_agent_id_card`
- Evidence: Dioxus binary command `identity.agent_card` calls `X0xClient::agent` and `X0xClient::agent_card`; the test asserts the active daemon agent ID matches the generated `x0x://agent/` card.

### Import agent card

- Test: `tests/e2e/identity.rs::dioxus_parity_identity_import_agent_card`
- Evidence: fixture B generates a real agent card through `X0xClient::agent_card`; the Dioxus binary imports it into fixture A via `X0xClient::import_agent_card` and returns the imported agent ID.

### Export keypairs

- Test: `tests/e2e/identity.rs::dioxus_parity_identity_export_keypairs_gap_recorded`
- Evidence: the Dioxus driver records that this cell cannot be honestly closed yet because `communitas-x0x-client/src/client.rs` does not expose a keypair export/backup method and the x0x GUI parity notes also defer private-key export pending a consent/encryption design. The test prevents accidental fake coverage by requiring this explicit gap response.
- Follow-up: add a consent-gated x0xd + `communitas-x0x-client` keypair backup method before adding active Dioxus controls.

### User identity (opt-in)

- Test: `tests/e2e/identity.rs::dioxus_parity_identity_user_identity_opt_in_read`
- Evidence: Dioxus binary command `identity.user_id` calls `X0xClient::agent_user_id` and proves the opt-in user identity state is readable, including the expected `None`/not-configured path.

## Trust & contacts

### Add / block / trust contact

- Test: `tests/e2e/trust_contacts.rs::dioxus_parity_trust_add_block_trust_contact`
- Evidence: Dioxus binary command `trust.add_block_trust` calls `X0xClient::add_contact`, then `X0xClient::set_trust` through trusted and blocked states, and verifies the final contact row.

### Machine pinning

- Test: `tests/e2e/trust_contacts.rs::dioxus_parity_trust_machine_pinning`
- Evidence: Dioxus binary command `trust.machine_pin` calls `X0xClient::add_machine`, `X0xClient::pin_machine`, and `X0xClient::list_machines`; the test asserts the secondary daemon machine is pinned.

### Trust evaluator decision read

- Test: `tests/e2e/trust_contacts.rs::dioxus_parity_trust_evaluator_decision_read`
- Evidence: Dioxus binary command `trust.evaluate` calls `X0xClient::evaluate_trust` for fixture B's agent/machine pair and asserts a decision string is returned.

## Connectivity / discovery

### Discover agents (cache / FOAF)

- Test: `tests/e2e/connectivity.rs::dioxus_parity_connectivity_discover_agents_cache_foaf`
- Evidence: Dioxus binary command `connectivity.discover_agents` calls `X0xClient::discovered_agents` and `X0xClient::presence_foaf` against the live daemon.

### Four-word network bootstrap

- Test: `tests/e2e/connectivity.rs::dioxus_parity_connectivity_four_word_network_bootstrap`
- Evidence: Dioxus binary command `connectivity.four_word_bootstrap` calls `X0xClient::bootstrap_cache` and `X0xClient::network_status`, matching the Dioxus Network/Local identity surfaces that expose bootstrap and reachability state.

## Groups

### Policy (roles, bans)

- Test: `tests/e2e/groups.rs::dioxus_parity_groups_policy_roles_bans`
- Evidence: Dioxus binary command `groups.policy` creates a named group with `X0xClient::create_group_with_preset`, updates policy with `X0xClient::update_group_policy`, exercises member role/ban/unban calls where the daemon accepts them, and verifies the returned group policy is present.

### Discover groups (tag / nearby)

- Test: `tests/e2e/groups.rs::dioxus_parity_groups_discover_tag_nearby`
- Evidence: Dioxus binary command `groups.discover` creates a public discoverable group, calls `X0xClient::discover_groups` for query/tag-style lookup, and calls `X0xClient::discover_groups_nearby` for nearby discovery.

## KV store

### Create / list stores

- Test: `tests/e2e/kv_store.rs::dioxus_parity_kv_create_list_stores`
- Evidence: Dioxus binary command `kv.create_list` calls `X0xClient::create_store` and `X0xClient::list_stores`; the test asserts the created store appears in the list.

### PUT / GET / DELETE key

- Test: `tests/e2e/kv_store.rs::dioxus_parity_kv_put_get_delete_key`
- Evidence: Dioxus binary command `kv.put_get_delete` calls `X0xClient::put`, `X0xClient::get`, and `X0xClient::delete_key`; the test asserts the deleted key becomes unreadable.

### Access-policy enforcement

- Test: `tests/e2e/kv_store.rs::dioxus_parity_kv_access_policy_enforcement`
- Evidence: Dioxus binary command `kv.access_policy_setup` creates a private store through fixture A; the test then uses fixture B's client to prove the foreign daemon cannot read the primary daemon's private key.

## Presence

### FOAF walk

- Test: `tests/e2e/presence.rs::dioxus_parity_presence_foaf_walk`
- Evidence: Dioxus binary command `presence.foaf` calls `X0xClient::presence_foaf` against a live daemon and validates the response shape.

## Upgrade / self-update

### Check updates

- Test: `tests/e2e/upgrade.rs::dioxus_parity_upgrade_check_updates`
- Evidence: Dioxus binary command `upgrade.check` calls `X0xClient::check_upgrade`; if the daemon reports an upstream update-service error, the driver falls back to the raw `GET /upgrade` response so the test proves the x0xd route is wired without depending on GitHub availability.

### Apply update

- Test: `tests/e2e/upgrade.rs::dioxus_parity_upgrade_apply_update_endpoint`
- Evidence: Dioxus binary command `upgrade.apply` calls the typed `X0xClient::apply_upgrade` method against a live daemon and validates the structured `applied` / `version` / `reason` response shape. The Dioxus settings UI uses the same typed method, preserving `X0X_API_BASE` / `X0X_API_TOKEN` overrides for test targets.
