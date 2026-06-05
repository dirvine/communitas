// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact typed client for the x0xd REST API.
//!
//! Methods in this file mirror daemon routes and payloads on `127.0.0.1:12700`.
//! This module intentionally stays transport-focused and does not define
//! Communitas-specific app semantics such as channel schemas or topic rules.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::config::X0xConfig;
use crate::error::{Result, X0xError};
use crate::types::*;

/// HTTP client for the x0xd daemon.
///
/// Wraps the REST surface intentionally covered by this crate on localhost.
/// Real-time WebSocket and SSE surfaces are provided by [`crate::X0xWebSocket`]
/// and [`crate::X0xSseStream`] respectively. All methods are async and return
/// typed responses.
///
/// Authentication is handled automatically: the client reads the `api.port` and
/// `api-token` files written by x0xd and attaches `Authorization: Bearer <token>`
/// to every request. If the config files are absent (e.g. during development
/// before x0xd has been started), it falls back to `http://127.0.0.1:12700`
/// with no token.
#[derive(Debug, Clone)]
pub struct X0xClient {
    base_url: String,
    client: reqwest::Client,
}

impl X0xClient {
    /// Create a new client.
    ///
    /// Attempts to auto-discover the x0x daemon address and Bearer token from
    /// the x0xd config files (`api.port` and `api-token`). Falls back to
    /// `http://127.0.0.1:12700` with no authentication if the files are not
    /// found, preserving backward compatibility for environments where x0xd has
    /// not yet written its config.
    pub fn new() -> Self {
        match crate::config::discover() {
            Ok(cfg) => Self::from_config(cfg),
            Err(e) => {
                tracing::debug!(
                    "x0x config discovery failed ({e}); falling back to http://127.0.0.1:12700 with no auth"
                );
                Self::with_base_url("http://127.0.0.1:12700")
            }
        }
    }

    /// Create a client from a discovered [`X0xConfig`].
    ///
    /// The Bearer token is attached to every outgoing request via a default
    /// header so callers never need to manage it manually.
    pub fn from_config(config: X0xConfig) -> Self {
        let base_url = format!("http://{}", config.address.trim_end_matches('/'));
        let client = Self::build_authenticated_client(&config.token);
        Self { base_url, client }
    }

    /// Discover the x0x daemon config and create an authenticated client.
    ///
    /// # Errors
    ///
    /// Returns [`X0xError::Config`] if the `api.port` or `api-token` files
    /// cannot be read, or [`X0xError::Http`] if the reqwest client cannot be
    /// constructed.
    pub fn discover() -> Result<Self> {
        let cfg = crate::config::discover()?;
        Ok(Self::from_config(cfg))
    }

    /// Create a new client pointing at a custom base URL with no authentication.
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new client pointing at a custom base URL with an explicit Bearer token.
    pub fn with_base_url_and_token(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: Self::build_authenticated_client(token),
        }
    }

    /// Build a `reqwest::Client` that injects `Authorization: Bearer <token>`
    /// into every request.
    fn build_authenticated_client(token: &str) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(value) = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert(reqwest::header::AUTHORIZATION, value);
        } else {
            tracing::warn!("x0x api token contains non-ASCII characters; auth header not set");
        }
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_default()
    }

    // ── helpers ──────────────────────────────────────────────────────────

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Parse a daemon response, handling x0x's flattened `{"ok":true,...}` payloads.
    async fn parse<T: serde::de::DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        let body = resp.text().await?;

        if let Ok(envelope) = serde_json::from_str::<ApiResponse<T>>(&body) {
            if envelope.ok {
                if let Some(data) = envelope.data {
                    return Ok(data);
                }
                return serde_json::from_str::<T>(&body).map_err(X0xError::Json);
            }
            return Err(X0xError::Daemon(
                envelope.error.unwrap_or_else(|| format!("HTTP {status}")),
            ));
        }

        match serde_json::from_str::<T>(&body) {
            Ok(parsed) => Ok(parsed),
            Err(err) if status.is_success() => Err(X0xError::Json(err)),
            Err(_) => Err(X0xError::Daemon(format!("HTTP {status}: {body}"))),
        }
    }

    /// Fire-and-forget request that only validates the x0x `ok` contract.
    async fn request_ok(&self, builder: reqwest::RequestBuilder) -> Result<()> {
        let resp = builder.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if let Ok(envelope) = serde_json::from_str::<ApiResponse<serde_json::Value>>(&text) {
            if envelope.ok {
                return Ok(());
            }
            return Err(X0xError::Daemon(
                envelope.error.unwrap_or_else(|| format!("HTTP {status}")),
            ));
        }
        if status.is_success() {
            Ok(())
        } else {
            Err(X0xError::Daemon(format!("HTTP {status}: {text}")))
        }
    }

    /// Fire-and-forget POST that only checks `ok`.
    async fn post_ok(&self, path: &str, body: &impl serde::Serialize) -> Result<()> {
        self.request_ok(self.client.post(self.url(path)).json(body))
            .await
    }

    /// DELETE that only checks `ok`.
    async fn delete_ok(&self, path: &str) -> Result<()> {
        self.request_ok(self.client.delete(self.url(path))).await
    }

    /// Fetch a plain-text endpoint such as `/constitution` or `/gui`.
    async fn get_text(&self, path: &str) -> Result<String> {
        let resp = self
            .client
            .get(self.url(path))
            .send()
            .await
            .map_err(|_| X0xError::NotReachable(self.base_url.clone()))?;
        let status = resp.status();
        let body = resp.text().await?;
        if status.is_success() {
            Ok(body)
        } else {
            Err(X0xError::Daemon(format!("HTTP {status}: {body}")))
        }
    }

    // ── System & Identity ───────────────────────────────────────────────

    /// Health probe. Returns `Ok(status)` if the daemon is reachable.
    pub async fn health(&self) -> Result<HealthStatus> {
        let resp = self
            .client
            .get(self.url("/health"))
            .send()
            .await
            .map_err(|_| X0xError::NotReachable(self.base_url.clone()))?;
        self.parse(resp).await
    }

    /// Rich status with connectivity state, external addresses, warnings.
    pub async fn status(&self) -> Result<DaemonStatus> {
        let resp = self.client.get(self.url("/status")).send().await?;
        self.parse(resp).await
    }

    /// Gracefully stop the local daemon.
    pub async fn shutdown(&self) -> Result<()> {
        self.post_ok("/shutdown", &serde_json::json!({})).await
    }

    /// Local agent identity (agent_id, machine_id, optional user_id).
    pub async fn agent(&self) -> Result<AgentIdentity> {
        let resp = self.client.get(self.url("/agent")).send().await?;
        self.parse(resp).await
    }

    /// Current user identity binding, if configured.
    pub async fn agent_user_id(&self) -> Result<Option<String>> {
        let resp = self.client.get(self.url("/agent/user-id")).send().await?;
        let status: UserIdStatus = self.parse(resp).await?;
        Ok(status.user_id)
    }

    /// Sign a base64-encoded payload with the local agent signing key.
    pub async fn agent_sign(&self, payload_b64: &str) -> Result<AgentSignResponse> {
        let req = AgentSignRequest {
            payload_b64: payload_b64.to_owned(),
        };
        let resp = self
            .client
            .post(self.url("/agent/sign"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Generate a shareable agent card and x0x://agent link.
    pub async fn agent_card(
        &self,
        display_name: Option<&str>,
        include_groups: Option<bool>,
    ) -> Result<AgentCardResponse> {
        let mut query: Vec<(String, String)> = Vec::new();
        if let Some(name) = display_name {
            query.push(("display_name".to_owned(), name.to_owned()));
        }
        if let Some(include) = include_groups {
            query.push(("include_groups".to_owned(), include.to_string()));
        }

        let resp = self
            .client
            .get(self.url("/agent/card"))
            .query(&query)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Import an x0x://agent card into local contacts.
    pub async fn import_agent_card(
        &self,
        card: &str,
        trust_level: Option<TrustLevel>,
    ) -> Result<ImportCardResponse> {
        let req = ImportCardRequest {
            card: card.to_owned(),
            trust_level,
        };
        let resp = self
            .client
            .post(self.url("/agent/card/import"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Fetch this agent's trust-scoped introduction card.
    pub async fn introduction(&self, peer_agent_id: Option<&str>) -> Result<IntroductionCard> {
        let mut req = self.client.get(self.url("/introduction"));
        if let Some(peer) = peer_agent_id {
            req = req.query(&[("peer", peer)]);
        }
        let resp = req.send().await?;
        self.parse(resp).await
    }

    /// Connected gossip peers from `GET /peers`.
    pub async fn peers(&self) -> Result<Vec<PeerInfo>> {
        let resp = self.client.get(self.url("/peers")).send().await?;
        let list: PeerList = self.parse(resp).await?;
        Ok(list.peers)
    }

    /// Force re-announce identity to the network with explicit consent flags.
    pub async fn announce_with_options(
        &self,
        include_user_identity: bool,
        human_consent: bool,
    ) -> Result<()> {
        let req = AnnounceRequest {
            include_user_identity,
            human_consent,
        };
        self.post_ok("/announce", &req).await
    }

    /// Force re-announce identity to the network using default x0x-safe flags.
    pub async fn announce(&self) -> Result<()> {
        self.announce_with_options(false, false).await
    }

    // ── Discovery ───────────────────────────────────────────────────────

    /// All discovered agents on the network.
    pub async fn discovered_agents(&self) -> Result<Vec<DiscoveredAgent>> {
        let resp = self
            .client
            .get(self.url("/agents/discovered"))
            .send()
            .await?;
        let list: DiscoveredAgentList = self.parse(resp).await?;
        Ok(list.agents)
    }

    /// Details for a specific discovered agent.
    pub async fn discovered_agent(&self, agent_id: &str) -> Result<DiscoveredAgent> {
        let resp = self
            .client
            .get(self.url(&format!("/agents/discovered/{agent_id}")))
            .send()
            .await?;
        let wrapper: DiscoveredAgentWrapper = self.parse(resp).await?;
        Ok(wrapper.agent)
    }

    /// Current discovered machine endpoint for an agent.
    pub async fn machine_for_agent(&self, agent_id: &str) -> Result<AgentMachine> {
        let resp = self
            .client
            .get(self.url(&format!("/agents/{agent_id}/machine")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// All discovered machine endpoints on the network.
    pub async fn discovered_machines(&self, unfiltered: bool) -> Result<Vec<DiscoveredMachine>> {
        let mut req = self.client.get(self.url("/machines/discovered"));
        if unfiltered {
            req = req.query(&[("unfiltered", "true")]);
        }
        let resp = req.send().await?;
        let list: DiscoveredMachineList = self.parse(resp).await?;
        Ok(list.machines)
    }

    /// Details for a specific discovered machine endpoint.
    pub async fn discovered_machine(
        &self,
        machine_id: &str,
        wait: bool,
    ) -> Result<DiscoveredMachine> {
        let mut req = self
            .client
            .get(self.url(&format!("/machines/discovered/{machine_id}")));
        if wait {
            req = req.query(&[("wait", "true")]);
        }
        let resp = req.send().await?;
        let wrapper: DiscoveredMachineWrapper = self.parse(resp).await?;
        Ok(wrapper.machine)
    }

    /// Discovered machine endpoints associated with a user identity.
    pub async fn machines_by_user(&self, user_id: &str) -> Result<UserMachineList> {
        let resp = self
            .client
            .get(self.url(&format!("/users/{user_id}/machines")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Online agent ids from `GET /presence`.
    pub async fn presence(&self) -> Result<Vec<String>> {
        let resp = self.client.get(self.url("/presence")).send().await?;
        let list: PresenceList = self.parse(resp).await?;
        Ok(list.agents)
    }

    /// NAT traversal, relay, and connection diagnostics.
    pub async fn network_status(&self) -> Result<NetworkStatus> {
        let resp = self.client.get(self.url("/network/status")).send().await?;
        self.parse(resp).await
    }

    /// Bootstrap cache and currently connected peers.
    pub async fn bootstrap_cache(&self) -> Result<BootstrapCacheStatus> {
        let resp = self
            .client
            .get(self.url("/network/bootstrap-cache"))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Full ant-quic connectivity diagnostic snapshot.
    pub async fn connectivity_diagnostics(&self) -> Result<ConnectivityDiagnostics> {
        let resp = self
            .client
            .get(self.url("/diagnostics/connectivity"))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// ant-quic ACK-v2 diagnostics from `GET /diagnostics/ack`.
    pub async fn diagnostics_ack(&self) -> Result<DiagnosticsSnapshot> {
        let resp = self.client.get(self.url("/diagnostics/ack")).send().await?;
        self.parse(resp).await
    }

    /// Direct-message transport diagnostics from `GET /diagnostics/dm`.
    pub async fn diagnostics_dm(&self) -> Result<DiagnosticsSnapshot> {
        let resp = self.client.get(self.url("/diagnostics/dm")).send().await?;
        self.parse(resp).await
    }

    /// Remote exec diagnostics from `GET /diagnostics/exec`.
    pub async fn diagnostics_exec(&self) -> Result<DiagnosticsSnapshot> {
        let resp = self
            .client
            .get(self.url("/diagnostics/exec"))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Named-group ingest diagnostics from `GET /diagnostics/groups`.
    pub async fn diagnostics_groups(&self) -> Result<DiagnosticsSnapshot> {
        let resp = self
            .client
            .get(self.url("/diagnostics/groups"))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// PubSub drop-detection counters (x0x 0.18+).
    ///
    /// Returns the 9 atomic counters that `x0xd` tracks across the publish
    /// → transport → incoming → decode → deliver pipeline. A zero
    /// `decode_to_delivery_drops` proves no messages were lost inside the
    /// local pipeline since the daemon started.
    pub async fn gossip_stats(&self) -> Result<GossipStats> {
        let resp = self
            .client
            .get(self.url("/diagnostics/gossip"))
            .send()
            .await?;
        let wrapper: GossipStatsWrapper = self.parse(resp).await?;
        Ok(wrapper.stats)
    }

    /// Active-liveness probe against a peer (ant-quic 0.27.2 `probe_peer`).
    ///
    /// `peer_id_hex` is the 32-byte peer id in hex (== MachineId). Returns
    /// measured round-trip time. The `timeout_ms` argument is clamped by
    /// x0xd to `[100, 30_000]` ms.
    pub async fn probe_peer(&self, peer_id_hex: &str, timeout_ms: u64) -> Result<ProbePeerResult> {
        let url = self.url(&format!("/peers/{peer_id_hex}/probe"));
        let resp = self
            .client
            .post(url)
            .query(&[("timeout_ms", timeout_ms)])
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Connection health snapshot for a peer (ant-quic 0.27.1 #170).
    pub async fn peer_health(&self, peer_id_hex: &str) -> Result<PeerHealth> {
        let resp = self
            .client
            .get(self.url(&format!("/peers/{peer_id_hex}/health")))
            .send()
            .await?;
        self.parse(resp).await
    }

    // ── Gossip pub/sub ──────────────────────────────────────────────────

    /// Publish a payload to a gossip topic.
    pub async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let req = PublishRequest {
            topic: topic.to_owned(),
            payload: BASE64.encode(payload),
        };
        self.post_ok("/publish", &req).await
    }

    /// Subscribe to a gossip topic. Returns a subscription ID for unsubscribing.
    pub async fn subscribe(&self, topic: &str) -> Result<String> {
        let req = SubscribeRequest {
            topic: topic.to_owned(),
        };
        let resp = self
            .client
            .post(self.url("/subscribe"))
            .json(&req)
            .send()
            .await?;
        let sub: SubscribeResponse = self.parse(resp).await?;
        Ok(sub.subscription_id)
    }

    /// Unsubscribe from a topic by subscription ID.
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        self.delete_ok(&format!("/subscribe/{subscription_id}"))
            .await
    }

    // ── Direct messaging ────────────────────────────────────────────────

    /// Establish a QUIC connection to a discovered agent.
    pub async fn connect_agent(&self, agent_id: &str) -> Result<()> {
        let req = ConnectRequest {
            agent_id: agent_id.to_owned(),
        };
        self.post_ok("/agents/connect", &req).await
    }

    /// Establish a QUIC connection to a discovered machine endpoint.
    pub async fn connect_machine(&self, machine_id: &str) -> Result<ConnectMachineResponse> {
        let req = ConnectMachineRequest {
            machine_id: machine_id.to_owned(),
        };
        let resp = self
            .client
            .post(self.url("/machines/connect"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Run a strictly allowlisted command on a remote daemon.
    pub async fn exec_run(&self, request: &ExecRunRequest) -> Result<ExecRunResponse> {
        let resp = self
            .client
            .post(self.url("/exec/run"))
            .json(request)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Cancel an in-flight remote exec request.
    pub async fn exec_cancel(&self, request_id: &str, agent_id: Option<&str>) -> Result<()> {
        let req = ExecCancelRequest {
            request_id: request_id.to_owned(),
            agent_id: agent_id.map(ToOwned::to_owned),
        };
        self.post_ok("/exec/cancel", &req).await
    }

    /// List local pending and remote active exec sessions.
    pub async fn exec_sessions(&self) -> Result<ExecSessionsSnapshot> {
        let resp = self.client.get(self.url("/exec/sessions")).send().await?;
        self.parse(resp).await
    }

    /// Send a direct (point-to-point) message.
    pub async fn send_direct(&self, agent_id: &str, payload: &[u8]) -> Result<()> {
        let req = DirectSendRequest {
            agent_id: agent_id.to_owned(),
            payload: BASE64.encode(payload),
        };
        self.post_ok("/direct/send", &req).await
    }

    /// List active direct connections.
    pub async fn direct_connections(&self) -> Result<Vec<DirectConnection>> {
        let resp = self
            .client
            .get(self.url("/direct/connections"))
            .send()
            .await?;
        let list: DirectConnectionList = self.parse(resp).await?;
        Ok(list.connections)
    }

    // ── Contacts & trust ────────────────────────────────────────────────

    /// List all contacts.
    pub async fn list_contacts(&self) -> Result<Vec<Contact>> {
        let resp = self.client.get(self.url("/contacts")).send().await?;
        let list: ContactList = self.parse(resp).await?;
        Ok(list.contacts)
    }

    /// Add a new contact.
    pub async fn add_contact(
        &self,
        agent_id: &str,
        trust_level: TrustLevel,
        label: Option<&str>,
    ) -> Result<()> {
        let req = AddContactRequest {
            agent_id: agent_id.to_owned(),
            trust_level,
            label: label.map(str::to_owned),
        };
        self.post_ok("/contacts", &req).await
    }

    /// Quick trust-level update.
    pub async fn set_trust(&self, agent_id: &str, level: TrustLevel) -> Result<()> {
        let req = SetTrustRequest {
            agent_id: agent_id.to_owned(),
            level,
        };
        self.post_ok("/contacts/trust", &req).await
    }

    /// Update a contact's trust level and/or identity type.
    ///
    /// `identity_type` should be one of `"anonymous"`, `"known"`, `"trusted"`,
    /// or `"pinned"` when provided.
    pub async fn update_contact(
        &self,
        agent_id: &str,
        trust_level: Option<TrustLevel>,
        identity_type: Option<&str>,
    ) -> Result<()> {
        let req = UpdateContactRequest {
            trust_level,
            identity_type: identity_type.map(str::to_owned),
        };
        self.request_ok(
            self.client
                .patch(self.url(&format!("/contacts/{agent_id}")))
                .json(&req),
        )
        .await
    }

    /// Remove a contact.
    pub async fn remove_contact(&self, agent_id: &str) -> Result<()> {
        self.delete_ok(&format!("/contacts/{agent_id}")).await
    }

    /// Revoke a contact relationship. The `reason` field is required by the daemon.
    pub async fn revoke_contact(&self, agent_id: &str, reason: &str) -> Result<()> {
        self.post_ok(
            &format!("/contacts/{agent_id}/revoke"),
            &serde_json::json!({ "reason": reason }),
        )
        .await
    }

    /// List recorded revocations for a contact.
    pub async fn revocations(&self, agent_id: &str) -> Result<Vec<serde_json::Value>> {
        let resp = self
            .client
            .get(self.url(&format!("/contacts/{agent_id}/revocations")))
            .send()
            .await?;
        let list: RevocationList = self.parse(resp).await?;
        Ok(list.revocations)
    }

    /// List machine records for a contact.
    pub async fn list_machines(&self, agent_id: &str) -> Result<Vec<MachineRecord>> {
        let resp = self
            .client
            .get(self.url(&format!("/contacts/{agent_id}/machines")))
            .send()
            .await?;
        let list: MachineList = self.parse(resp).await?;
        Ok(list.machines)
    }

    /// Add a machine record to a contact.
    pub async fn add_machine(
        &self,
        agent_id: &str,
        machine_id: &str,
        label: Option<&str>,
        pinned: Option<bool>,
    ) -> Result<()> {
        let req = AddMachineRequest {
            machine_id: machine_id.to_owned(),
            label: label.map(str::to_owned),
            pinned,
        };
        self.post_ok(&format!("/contacts/{agent_id}/machines"), &req)
            .await
    }

    /// Remove a machine record from a contact.
    pub async fn remove_machine(&self, agent_id: &str, machine_id: &str) -> Result<()> {
        self.delete_ok(&format!("/contacts/{agent_id}/machines/{machine_id}"))
            .await
    }

    /// Pin a contact to a specific machine.
    pub async fn pin_machine(&self, agent_id: &str, machine_id: &str) -> Result<()> {
        self.post_ok(
            &format!("/contacts/{agent_id}/machines/{machine_id}/pin"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Remove a machine pin from a contact.
    pub async fn unpin_machine(&self, agent_id: &str, machine_id: &str) -> Result<()> {
        self.delete_ok(&format!("/contacts/{agent_id}/machines/{machine_id}/pin"))
            .await
    }

    /// Evaluate the trust decision x0x would make for an agent/machine pair.
    pub async fn evaluate_trust(
        &self,
        agent_id: &str,
        machine_id: &str,
    ) -> Result<TrustEvaluation> {
        let req = EvaluateTrustRequest {
            agent_id: agent_id.to_owned(),
            machine_id: machine_id.to_owned(),
        };
        let resp = self
            .client
            .post(self.url("/trust/evaluate"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    // ── MLS groups ──────────────────────────────────────────────────────

    /// Create an MLS encrypted group.
    pub async fn create_mls_group(&self, group_id: Option<&str>) -> Result<MlsGroup> {
        let req = CreateMlsGroupRequest {
            group_id: group_id.map(str::to_owned),
        };
        let resp = self
            .client
            .post(self.url("/mls/groups"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// List all MLS groups.
    pub async fn list_mls_groups(&self) -> Result<Vec<MlsGroup>> {
        let resp = self.client.get(self.url("/mls/groups")).send().await?;
        let list: MlsGroupList = self.parse(resp).await?;
        Ok(list.groups)
    }

    /// Get details and members of an MLS group.
    pub async fn get_mls_group(&self, group_id: &str) -> Result<MlsGroup> {
        let resp = self
            .client
            .get(self.url(&format!("/mls/groups/{group_id}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Add a member to an MLS group.
    pub async fn add_mls_member(
        &self,
        group_id: &str,
        agent_id: &str,
    ) -> Result<AddMlsMemberResponse> {
        let req = AddMlsMemberRequest {
            agent_id: agent_id.to_owned(),
        };
        let resp = self
            .client
            .post(self.url(&format!("/mls/groups/{group_id}/members")))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Remove a member from an MLS group.
    pub async fn remove_mls_member(&self, group_id: &str, agent_id: &str) -> Result<()> {
        self.delete_ok(&format!("/mls/groups/{group_id}/members/{agent_id}"))
            .await
    }

    /// Encrypt a payload with the group's current key.
    pub async fn encrypt(&self, group_id: &str, payload: &[u8]) -> Result<EncryptResponse> {
        let req = EncryptRequest {
            payload: BASE64.encode(payload),
        };
        let resp = self
            .client
            .post(self.url(&format!("/mls/groups/{group_id}/encrypt")))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Decrypt a ciphertext from the group at a given epoch.
    pub async fn decrypt(&self, group_id: &str, ciphertext: &str, epoch: u64) -> Result<Vec<u8>> {
        let req = DecryptRequest {
            ciphertext: ciphertext.to_owned(),
            epoch,
        };
        let resp = self
            .client
            .post(self.url(&format!("/mls/groups/{group_id}/decrypt")))
            .json(&req)
            .send()
            .await?;
        let dec: DecryptResponse = self.parse(resp).await?;
        Ok(BASE64.decode(&dec.payload)?)
    }

    /// Create a welcome message for a prospective MLS group member.
    pub async fn create_mls_welcome(
        &self,
        group_id: &str,
        agent_id: &str,
    ) -> Result<WelcomeResponse> {
        let req = CreateWelcomeRequest {
            agent_id: agent_id.to_owned(),
        };
        let resp = self
            .client
            .post(self.url(&format!("/mls/groups/{group_id}/welcome")))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    // ── Named groups (high-level) ───────────────────────────────────────

    /// Create a named group.
    pub async fn create_group(
        &self,
        name: &str,
        description: Option<&str>,
        display_name: Option<&str>,
    ) -> Result<CreatedGroup> {
        let req = CreateGroupRequest {
            name: name.to_owned(),
            description: description.map(str::to_owned),
            display_name: display_name.map(str::to_owned),
        };
        let resp = self
            .client
            .post(self.url("/groups"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// List all named groups.
    pub async fn list_groups(&self) -> Result<Vec<GroupSummary>> {
        let resp = self.client.get(self.url("/groups")).send().await?;
        let list: GroupList = self.parse(resp).await?;
        Ok(list.groups)
    }

    /// Get full info for a named group.
    pub async fn get_group(&self, group_id: &str) -> Result<GroupInfo> {
        let resp = self
            .client
            .get(self.url(&format!("/groups/{group_id}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Generate an invite link for a group.
    pub async fn invite(&self, group_id: &str, expiry_secs: Option<u64>) -> Result<InviteResponse> {
        let req = InviteRequest { expiry_secs };
        let resp = self
            .client
            .post(self.url(&format!("/groups/{group_id}/invite")))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Join a group via an invite link or token.
    pub async fn join_group(
        &self,
        invite: &str,
        display_name: Option<&str>,
    ) -> Result<JoinGroupResponse> {
        let req = JoinGroupRequest {
            invite: invite.to_owned(),
            display_name: display_name.map(str::to_owned),
        };
        let resp = self
            .client
            .post(self.url("/groups/join"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Set your display name within a group.
    pub async fn set_group_display_name(&self, group_id: &str, name: &str) -> Result<()> {
        let req = SetDisplayNameRequest {
            name: name.to_owned(),
        };
        self.request_ok(
            self.client
                .put(self.url(&format!("/groups/{group_id}/display-name")))
                .json(&req),
        )
        .await
    }

    /// Leave a group.
    pub async fn leave_group(&self, group_id: &str) -> Result<()> {
        self.delete_ok(&format!("/groups/{group_id}")).await
    }

    // ── Task lists (CRDTs) ──────────────────────────────────────────────

    /// Create a collaborative task list bound to a gossip topic.
    pub async fn create_task_list(&self, name: &str, topic: &str) -> Result<CreatedTaskList> {
        let req = CreateTaskListRequest {
            name: name.to_owned(),
            topic: topic.to_owned(),
        };
        let resp = self
            .client
            .post(self.url("/task-lists"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// List all task lists.
    pub async fn list_task_lists(&self) -> Result<Vec<TaskListSummary>> {
        let resp = self.client.get(self.url("/task-lists")).send().await?;
        let list: TaskListIndex = self.parse(resp).await?;
        Ok(list.task_lists)
    }

    /// List tasks in a task list.
    pub async fn list_tasks(&self, list_id: &str) -> Result<Vec<Task>> {
        let resp = self
            .client
            .get(self.url(&format!("/task-lists/{list_id}/tasks")))
            .send()
            .await?;
        let index: TaskIndex = self.parse(resp).await?;
        Ok(index.tasks)
    }

    /// Add a task to a list.
    pub async fn add_task(
        &self,
        list_id: &str,
        title: &str,
        description: Option<&str>,
    ) -> Result<()> {
        let req = AddTaskRequest {
            title: title.to_owned(),
            description: description.map(str::to_owned),
        };
        self.post_ok(&format!("/task-lists/{list_id}/tasks"), &req)
            .await
    }

    /// Claim a task (assign it to yourself).
    pub async fn claim_task(&self, list_id: &str, task_id: &str) -> Result<()> {
        let req = UpdateTaskRequest {
            action: "claim".to_owned(),
        };
        self.request_ok(
            self.client
                .patch(self.url(&format!("/task-lists/{list_id}/tasks/{task_id}")))
                .json(&req),
        )
        .await
    }

    /// Complete a task.
    pub async fn complete_task(&self, list_id: &str, task_id: &str) -> Result<()> {
        let req = UpdateTaskRequest {
            action: "complete".to_owned(),
        };
        self.request_ok(
            self.client
                .patch(self.url(&format!("/task-lists/{list_id}/tasks/{task_id}")))
                .json(&req),
        )
        .await
    }

    // ── Key-value stores ────────────────────────────────────────────────

    /// Create a key-value store.
    pub async fn create_store(&self, name: &str, topic: &str) -> Result<CreatedStore> {
        let req = CreateStoreRequest {
            name: name.to_owned(),
            topic: topic.to_owned(),
        };
        let resp = self
            .client
            .post(self.url("/stores"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Join an existing store by its topic.
    pub async fn join_store(&self, store_id: &str) -> Result<()> {
        self.post_ok(&format!("/stores/{store_id}/join"), &serde_json::json!({}))
            .await
    }

    /// List all stores.
    pub async fn list_stores(&self) -> Result<Vec<StoreSummary>> {
        let resp = self.client.get(self.url("/stores")).send().await?;
        let list: StoreIndex = self.parse(resp).await?;
        Ok(list.stores)
    }

    /// List keys in a store.
    pub async fn list_keys(&self, store_id: &str) -> Result<Vec<StoreKeyEntry>> {
        let resp = self
            .client
            .get(self.url(&format!("/stores/{store_id}/keys")))
            .send()
            .await?;
        let index: StoreKeyIndex = self.parse(resp).await?;
        Ok(index.keys)
    }

    /// Put a value in a store.
    pub async fn put(
        &self,
        store_id: &str,
        key: &str,
        value: &[u8],
        content_type: Option<&str>,
    ) -> Result<()> {
        let req = PutValueRequest {
            value: BASE64.encode(value),
            content_type: content_type.map(str::to_owned),
        };
        self.request_ok(
            self.client
                .put(self.url(&format!("/stores/{store_id}/{key}")))
                .json(&req),
        )
        .await
    }

    /// Get a raw store entry, including its base64 value and metadata.
    pub async fn get(&self, store_id: &str, key: &str) -> Result<StoreValue> {
        let resp = self
            .client
            .get(self.url(&format!("/stores/{store_id}/{key}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Delete a key from a store.
    pub async fn delete_key(&self, store_id: &str, key: &str) -> Result<()> {
        self.delete_ok(&format!("/stores/{store_id}/{key}")).await
    }

    // ── File transfer ───────────────────────────────────────────────────

    /// Initiate a file send to another agent.
    ///
    /// `sha256` is required by the daemon and must be the hex-encoded SHA-256
    /// digest of the file content. `path` is an optional local filesystem path
    /// for x0xd to read the file from directly.
    pub async fn send_file(
        &self,
        agent_id: &str,
        filename: &str,
        size: u64,
        sha256: &str,
        path: Option<&str>,
    ) -> Result<String> {
        let req = SendFileRequest {
            agent_id: agent_id.to_owned(),
            filename: filename.to_owned(),
            size,
            sha256: sha256.to_owned(),
            path: path.map(str::to_owned),
        };
        let resp = self
            .client
            .post(self.url("/files/send"))
            .json(&req)
            .send()
            .await?;
        let created: SendFileResponse = self.parse(resp).await?;
        Ok(created.transfer_id)
    }

    /// List all file transfers (sending and receiving).
    pub async fn transfers(&self) -> Result<Vec<FileTransfer>> {
        let resp = self.client.get(self.url("/files/transfers")).send().await?;
        let list: TransferList = self.parse(resp).await?;
        Ok(list.transfers)
    }

    /// Get status of a specific transfer.
    pub async fn transfer_status(&self, transfer_id: &str) -> Result<FileTransfer> {
        let resp = self
            .client
            .get(self.url(&format!("/files/transfers/{transfer_id}")))
            .send()
            .await?;
        let wrapper: FileTransferWrapper = self.parse(resp).await?;
        Ok(wrapper.transfer)
    }

    /// Accept an incoming file transfer.
    pub async fn accept_file(&self, transfer_id: &str) -> Result<()> {
        self.post_ok(
            &format!("/files/accept/{transfer_id}"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Reject an incoming file transfer.
    pub async fn reject_file(&self, transfer_id: &str, reason: Option<&str>) -> Result<()> {
        let req = RejectFileRequest {
            reason: reason.map(str::to_owned),
        };
        self.post_ok(&format!("/files/reject/{transfer_id}"), &req)
            .await
    }

    // ── Upgrade ─────────────────────────────────────────────────────────

    /// Check for x0xd updates.
    pub async fn check_upgrade(&self) -> Result<UpgradeStatus> {
        let resp = self.client.get(self.url("/upgrade")).send().await?;
        self.parse(resp).await
    }

    /// Apply a pending x0xd update.
    pub async fn apply_upgrade(&self) -> Result<ApplyUpgradeResponse> {
        let resp = self.client.post(self.url("/upgrade/apply")).send().await?;
        self.parse(resp).await
    }

    // ── Constitution ───────────────────────────────────────────────────────

    /// Fetch the raw markdown text of the x0x constitution.
    ///
    /// Calls `GET /constitution` which returns `text/markdown` and requires no
    /// authentication.
    pub async fn constitution(&self) -> Result<String> {
        self.get_text("/constitution").await
    }

    /// Fetch the x0x constitution with version metadata.
    ///
    /// Calls `GET /constitution/json` which returns a JSON payload containing
    /// `version`, `status`, and the full markdown `content`. No authentication
    /// required.
    pub async fn constitution_json(&self) -> Result<ConstitutionInfo> {
        let resp = self
            .client
            .get(self.url("/constitution/json"))
            .send()
            .await
            .map_err(|_| X0xError::NotReachable(self.base_url.clone()))?;
        self.parse(resp).await
    }

    /// Fetch the embedded x0x daemon GUI HTML.
    pub async fn gui_html(&self) -> Result<String> {
        self.get_text("/gui").await
    }

    /// Inspect active WebSocket sessions and their shared subscriptions.
    pub async fn ws_sessions(&self) -> Result<WsSessionList> {
        let resp = self.client.get(self.url("/ws/sessions")).send().await?;
        self.parse(resp).await
    }

    // ── Presence (extended) ─────────────────────────────────────────────

    /// List all currently online agents (network view, non-blocked).
    pub async fn presence_online(&self) -> Result<Vec<DiscoveredAgent>> {
        let resp = self.client.get(self.url("/presence/online")).send().await?;
        let list: PresenceOnlineList = self.parse(resp).await?;
        Ok(list.agents)
    }

    /// FOAF random-walk discovery of nearby agents (social view: Trusted + Known only).
    ///
    /// `ttl` controls hop depth (default 3), `timeout_ms` controls max wait (default 5000).
    pub async fn presence_foaf(
        &self,
        ttl: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> Result<Vec<DiscoveredAgent>> {
        let mut req = self.client.get(self.url("/presence/foaf"));
        if let Some(t) = ttl {
            req = req.query(&[("ttl", t.to_string())]);
        }
        if let Some(ms) = timeout_ms {
            req = req.query(&[("timeout_ms", ms.to_string())]);
        }
        let resp = req.send().await?;
        let list: FoafDiscoveryList = self.parse(resp).await?;
        Ok(list.agents)
    }

    /// Find a specific agent by ID via FOAF random walk.
    ///
    /// Returns `None` if the agent was not found within the walk limits.
    pub async fn presence_find(
        &self,
        agent_id: &str,
        ttl: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> Result<Option<DiscoveredAgent>> {
        let mut req = self
            .client
            .get(self.url(&format!("/presence/find/{agent_id}")));
        if let Some(t) = ttl {
            req = req.query(&[("ttl", t.to_string())]);
        }
        if let Some(ms) = timeout_ms {
            req = req.query(&[("timeout_ms", ms.to_string())]);
        }
        let resp = req.send().await?;
        let result: PresenceFindResponse = self.parse(resp).await?;
        Ok(result.agent)
    }

    /// Get local cache presence status for a specific agent — no network I/O.
    pub async fn presence_status(&self, agent_id: &str) -> Result<PresenceStatusResponse> {
        let resp = self
            .client
            .get(self.url(&format!("/presence/status/{agent_id}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    // ── Agent discovery (extended) ──────────────────────────────────────

    /// Check NAT traversal reachability for an agent before connecting.
    pub async fn agent_reachability(&self, agent_id: &str) -> Result<ReachabilityInfo> {
        let resp = self
            .client
            .get(self.url(&format!("/agents/reachability/{agent_id}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Actively search for an agent (3-stage: cache → shard → rendezvous).
    ///
    /// Returns addresses if found, empty vec if not.
    pub async fn find_agent(&self, agent_id: &str) -> Result<FindAgentResponse> {
        let resp = self
            .client
            .post(self.url(&format!("/agents/find/{agent_id}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// Look up all agents belonging to a user ID.
    pub async fn user_agents(&self, user_id: &str) -> Result<Vec<DiscoveredAgent>> {
        let resp = self
            .client
            .get(self.url(&format!("/users/{user_id}/agents")))
            .send()
            .await?;
        let list: UserAgentsList = self.parse(resp).await?;
        Ok(list.agents)
    }

    // ── Named groups: policy + extended surface ─────────────────────────

    /// Create a named group with an optional policy preset.
    ///
    /// Equivalent to [`Self::create_group`] but accepts a preset name
    /// (`"private_secure"`, `"public_request_secure"`, `"public_open"`,
    /// `"public_announce"`). When `preset` is `None`, x0xd applies
    /// `private_secure` as default.
    pub async fn create_group_with_preset(
        &self,
        name: &str,
        description: Option<&str>,
        display_name: Option<&str>,
        preset: Option<GroupPolicyPreset>,
    ) -> Result<CreatedGroup> {
        let req = CreateNamedGroupRequest {
            name: name.to_owned(),
            description: description.map(str::to_owned),
            display_name: display_name.map(str::to_owned),
            preset: preset.and_then(|p| {
                serde_json::to_value(p)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
            }),
        };
        let resp = self
            .client
            .post(self.url("/groups"))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `PATCH /groups/:id` — update name and/or description (admin+).
    pub async fn update_named_group(
        &self,
        group_id: &str,
        patch: &UpdateNamedGroupRequest,
    ) -> Result<()> {
        self.request_ok(
            self.client
                .patch(self.url(&format!("/groups/{group_id}")))
                .json(patch),
        )
        .await
    }

    /// `PATCH /groups/:id/policy` — update policy axes (owner only).
    pub async fn update_group_policy(
        &self,
        group_id: &str,
        patch: &UpdateGroupPolicyRequest,
    ) -> Result<()> {
        self.request_ok(
            self.client
                .patch(self.url(&format!("/groups/{group_id}/policy")))
                .json(patch),
        )
        .await
    }

    /// `GET /groups/:id/members` — list roster members.
    pub async fn list_named_group_members(&self, group_id: &str) -> Result<Vec<NamedGroupMember>> {
        let resp = self
            .client
            .get(self.url(&format!("/groups/{group_id}/members")))
            .send()
            .await?;
        let list: NamedGroupMembersResponse = self.parse(resp).await?;
        Ok(list.members)
    }

    /// `POST /groups/:id/members` — add a member directly (admin+).
    pub async fn add_named_group_member(
        &self,
        group_id: &str,
        agent_id: &str,
        display_name: Option<&str>,
    ) -> Result<()> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            agent_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            display_name: Option<&'a str>,
        }
        self.post_ok(
            &format!("/groups/{group_id}/members"),
            &Body {
                agent_id,
                display_name,
            },
        )
        .await
    }

    /// `DELETE /groups/:id/members/:agent_id` — remove a member (admin+).
    pub async fn remove_named_group_member(&self, group_id: &str, agent_id: &str) -> Result<()> {
        self.delete_ok(&format!("/groups/{group_id}/members/{agent_id}"))
            .await
    }

    /// `PATCH /groups/:id/members/:agent_id/role` — change role (admin+).
    pub async fn set_named_group_member_role(
        &self,
        group_id: &str,
        agent_id: &str,
        role: GroupRole,
    ) -> Result<()> {
        self.request_ok(
            self.client
                .patch(self.url(&format!("/groups/{group_id}/members/{agent_id}/role")))
                .json(&UpdateMemberRoleRequest { role }),
        )
        .await
    }

    /// `POST /groups/:id/ban/:agent_id` — ban a member (admin+).
    pub async fn ban_group_member(&self, group_id: &str, agent_id: &str) -> Result<()> {
        self.request_ok(
            self.client
                .post(self.url(&format!("/groups/{group_id}/ban/{agent_id}"))),
        )
        .await
    }

    /// `DELETE /groups/:id/ban/:agent_id` — unban a member (admin+).
    pub async fn unban_group_member(&self, group_id: &str, agent_id: &str) -> Result<()> {
        self.delete_ok(&format!("/groups/{group_id}/ban/{agent_id}"))
            .await
    }

    // ── Join requests ───────────────────────────────────────────────────

    /// `GET /groups/:id/requests` — list pending join requests (admin+).
    pub async fn list_join_requests(&self, group_id: &str) -> Result<Vec<JoinRequest>> {
        let resp = self
            .client
            .get(self.url(&format!("/groups/{group_id}/requests")))
            .send()
            .await?;
        let list: JoinRequestListResponse = self.parse(resp).await?;
        Ok(list.requests)
    }

    /// `POST /groups/:id/requests` — submit a join request.
    pub async fn create_join_request(
        &self,
        group_id: &str,
        message: Option<&str>,
    ) -> Result<JoinRequest> {
        let body = CreateJoinRequestBody {
            message: message.map(str::to_owned),
        };
        let resp = self
            .client
            .post(self.url(&format!("/groups/{group_id}/requests")))
            .json(&body)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `POST /groups/:id/requests/:request_id/approve` — approve (admin+).
    pub async fn approve_join_request(&self, group_id: &str, request_id: &str) -> Result<()> {
        self.request_ok(
            self.client
                .post(self.url(&format!("/groups/{group_id}/requests/{request_id}/approve"))),
        )
        .await
    }

    /// `POST /groups/:id/requests/:request_id/reject` — reject (admin+).
    pub async fn reject_join_request(&self, group_id: &str, request_id: &str) -> Result<()> {
        self.request_ok(
            self.client
                .post(self.url(&format!("/groups/{group_id}/requests/{request_id}/reject"))),
        )
        .await
    }

    /// `DELETE /groups/:id/requests/:request_id` — cancel own request.
    pub async fn cancel_join_request(&self, group_id: &str, request_id: &str) -> Result<()> {
        self.delete_ok(&format!("/groups/{group_id}/requests/{request_id}"))
            .await
    }

    // ── Discovery (Phase C + C.2) ───────────────────────────────────────

    /// `GET /groups/discover?q=<query>` — partition-local search across
    /// shard-observed + locally-owned discoverable groups.
    pub async fn discover_groups(&self, query: Option<&str>) -> Result<Vec<GroupCard>> {
        let mut req = self.client.get(self.url("/groups/discover"));
        if let Some(q) = query.filter(|s| !s.is_empty()) {
            req = req.query(&[("q", q)]);
        }
        let resp = req.send().await?;
        let list: GroupCardList = self.parse(resp).await?;
        Ok(list.groups)
    }

    /// `GET /groups/discover/nearby` — shard-only witness of
    /// PublicDirectory groups observable via presence-social browse.
    pub async fn discover_groups_nearby(&self) -> Result<Vec<GroupCard>> {
        let resp = self
            .client
            .get(self.url("/groups/discover/nearby"))
            .send()
            .await?;
        let list: GroupCardList = self.parse(resp).await?;
        Ok(list.groups)
    }

    /// `GET /groups/discover/subscriptions` — list active shard subs.
    pub async fn list_shard_subscriptions(&self) -> Result<ShardSubscriptionsResponse> {
        let resp = self
            .client
            .get(self.url("/groups/discover/subscriptions"))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `POST /groups/discover/subscribe` — subscribe to a tag/name/id shard.
    pub async fn subscribe_directory_shard(
        &self,
        kind: ShardKind,
        key: Option<&str>,
        shard: Option<u32>,
    ) -> Result<SubscribeShardResponse> {
        let kind_str = match kind {
            ShardKind::Tag => "tag",
            ShardKind::Name => "name",
            ShardKind::Id => "id",
        };
        let body = SubscribeShardRequest {
            kind: kind_str.to_owned(),
            key: key.map(str::to_owned),
            shard,
        };
        let resp = self
            .client
            .post(self.url("/groups/discover/subscribe"))
            .json(&body)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `DELETE /groups/discover/subscribe/:kind/:shard` — unsubscribe.
    pub async fn unsubscribe_directory_shard(&self, kind: ShardKind, shard: u32) -> Result<()> {
        let kind_str = match kind {
            ShardKind::Tag => "tag",
            ShardKind::Name => "name",
            ShardKind::Id => "id",
        };
        self.delete_ok(&format!("/groups/discover/subscribe/{kind_str}/{shard}"))
            .await
    }

    /// `GET /groups/cards/:id` — fetch a single signed card.
    pub async fn get_group_card(&self, group_id: &str) -> Result<GroupCard> {
        let resp = self
            .client
            .get(self.url(&format!("/groups/cards/{group_id}")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `POST /groups/cards/import` — import a discovered signed card.
    pub async fn import_group_card(&self, card: &GroupCard) -> Result<()> {
        self.post_ok("/groups/cards/import", card).await
    }

    // ── Public messaging (Phase E) ──────────────────────────────────────

    /// `POST /groups/:id/send` — publish a signed message to a
    /// SignedPublic group. Fails for MlsEncrypted groups; use
    /// [`Self::secure_group_encrypt`] + the gossip path for those.
    pub async fn send_group_public_message(
        &self,
        group_id: &str,
        body: &str,
        kind: Option<&str>,
    ) -> Result<GroupPublicMessage> {
        let req = SendGroupMessageRequest {
            body: body.to_owned(),
            kind: kind.map(str::to_owned),
        };
        let resp = self
            .client
            .post(self.url(&format!("/groups/{group_id}/send")))
            .json(&req)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `GET /groups/:id/messages` — retrieve cached signed messages.
    pub async fn get_group_public_messages(
        &self,
        group_id: &str,
    ) -> Result<Vec<GroupPublicMessage>> {
        let resp = self
            .client
            .get(self.url(&format!("/groups/{group_id}/messages")))
            .send()
            .await?;
        let list: GroupMessagesResponse = self.parse(resp).await?;
        Ok(list.messages)
    }

    // ── State-commit chain (Phase D.3) ──────────────────────────────────

    /// `GET /groups/:id/state` — inspect the signed state-commit chain.
    ///
    /// Returns the daemon's derived view of the chain (stable `group_id`,
    /// genesis record if present, current revision, state hash, previous
    /// hash, and component commitments). Member-only — the daemon returns
    /// 403 for non-members.
    pub async fn get_group_state(&self, group_id: &str) -> Result<GroupStateResponse> {
        let resp = self
            .client
            .get(self.url(&format!("/groups/{group_id}/state")))
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `POST /groups/:id/state/seal` — advance the state chain.
    pub async fn seal_group_state(&self, group_id: &str) -> Result<()> {
        self.request_ok(
            self.client
                .post(self.url(&format!("/groups/{group_id}/state/seal"))),
        )
        .await
    }

    /// `POST /groups/:id/state/withdraw` — seal a terminal withdrawal.
    pub async fn withdraw_group_state(&self, group_id: &str) -> Result<()> {
        self.request_ok(
            self.client
                .post(self.url(&format!("/groups/{group_id}/state/withdraw"))),
        )
        .await
    }

    // ── Secure plane (Phase D.2) ────────────────────────────────────────

    /// `POST /groups/:id/secure/encrypt` — AEAD-encrypt with the group's
    /// shared secret (member-only).
    pub async fn secure_group_encrypt(
        &self,
        group_id: &str,
        payload: &[u8],
    ) -> Result<SecureEncryptResponse> {
        use base64::Engine as _;
        let body = SecureEncryptRequest {
            payload_b64: base64::engine::general_purpose::STANDARD.encode(payload),
        };
        let resp = self
            .client
            .post(self.url(&format!("/groups/{group_id}/secure/encrypt")))
            .json(&body)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `POST /groups/:id/secure/decrypt` — AEAD-decrypt with epoch match.
    pub async fn secure_group_decrypt(
        &self,
        group_id: &str,
        ciphertext_b64: &str,
        nonce_b64: &str,
        secret_epoch: u64,
    ) -> Result<Vec<u8>> {
        use base64::Engine as _;
        let body = SecureDecryptRequest {
            ciphertext_b64: ciphertext_b64.to_owned(),
            nonce_b64: nonce_b64.to_owned(),
            secret_epoch,
        };
        let resp = self
            .client
            .post(self.url(&format!("/groups/{group_id}/secure/decrypt")))
            .json(&body)
            .send()
            .await?;
        let parsed: SecureDecryptResponse = self.parse(resp).await?;
        base64::engine::general_purpose::STANDARD
            .decode(parsed.plaintext_b64.as_bytes())
            .map_err(|e| X0xError::Daemon(format!("bad plaintext_b64: {e}")))
    }

    /// `POST /groups/:id/secure/reseal` — produce an ML-KEM envelope
    /// sealed to a recipient member's KEM public key.
    pub async fn secure_group_reseal(
        &self,
        group_id: &str,
        recipient_agent_id: &str,
    ) -> Result<SecureShareEnvelope> {
        let body = SecureResealRequest {
            recipient: recipient_agent_id.to_owned(),
        };
        let resp = self
            .client
            .post(self.url(&format!("/groups/{group_id}/secure/reseal")))
            .json(&body)
            .send()
            .await?;
        self.parse(resp).await
    }

    /// `POST /groups/secure/open-envelope` — adversarial test: attempt
    /// to open an envelope with this daemon's ML-KEM private key.
    pub async fn secure_open_envelope(
        &self,
        envelope: &SecureShareEnvelope,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url("/groups/secure/open-envelope"))
            .json(envelope)
            .send()
            .await?;
        self.parse(resp).await
    }
}

impl Default for X0xClient {
    fn default() -> Self {
        Self::new()
    }
}
