# ADR-023: Unlock Grants and Capability Tokens

## Status

Accepted

## Context

Communitas runs a local MCP server that must be usable by LLM clients (ChatGPT,
Claude, saorsa-canvas, etc.) without exposing passphrases, passkeys, or vault
secrets to the model. The vault is encrypted at rest, but MCP tools need
controlled access to decrypted data during a session.

We also operate in a P2P environment with no central auth server, so all access
control must be local and user-approved.

## Decision

Adopt a **two-stage access model** for vault-backed MCP operations:

1. **Unlock Grants** (single-use, request-bound)
2. **Unlock Leases** (sliding idle window)
3. **Capability Tokens** (scoped tool permissions)

### Unlock Grant

An unlock grant is issued only by a trusted UI shell (Dioxus desktop or future
phone app). It is:

- Single-use
- Bound to a specific request hash
- Signed by the trusted UI
- Short-lived (minutes)

The server verifies the grant, unseals the vault key, and immediately converts
the grant into an unlock lease. The grant is then marked as spent.

`create_unlock_grant` and `get_unlock_status` are only available to authenticated
non-delegate sessions; delegate tokens cannot issue or inspect unlock grants.

### Unlock Lease (Sliding)

The unlock lease keeps the vault key in memory for a short idle window:

- Default idle timeout: **10 minutes**
- Each authorized request resets `expires_at = now + 10 minutes`
- On timeout, the key is zeroized and the lease expires
- Optional **max total unlock duration** may be enforced
  - Default is `0` (unlimited)

### Capability Tokens

Capability tokens are issued by the server and stored in **`communitas-mcp-bridge`**.
LLM clients never see these tokens. Tokens are:

- Scoped (tool, entity, path, or operation scopes)
- Time-limited
- Bound to a client key when possible

`communitas-mcp-bridge` attaches tokens to MCP calls and can perform proof-of-possession
signing without exposing secrets to the LLM.

## Consequences

### Benefits

- Prevents model access to passwords, passkeys, or vault keys
- Supports long interactive sessions with a sliding idle window
- Works offline and P2P without a central auth server
- Clear boundary between trusted UI and untrusted LLM clients

### Trade-offs

- Requires `communitas-mcp-bridge` for LLM clients
- Adds complexity to session and unlock state handling

## References

- ADR-011: Encrypted Vault Storage
- ADR-018: MCP External Integration Architecture
- ADR-019: Shared Rust UI Service Layer
- ADR-022: MCP Apps Integration
