//! HTTP client for the x0xd daemon REST API.
//!
//! All methods correspond to x0xd endpoints on `127.0.0.1:12700`.
//! Payloads are automatically base64-encoded/decoded where needed.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::error::{Result, X0xError};
use crate::types::*;

/// HTTP client for the x0xd daemon.
///
/// Wraps the full REST API exposed by x0xd on localhost.
/// All methods are async and return typed responses.
#[derive(Debug, Clone)]
pub struct X0xClient {
    base_url: String,
    client: reqwest::Client,
}

impl X0xClient {
    /// Create a new client pointing at the default daemon address.
    pub fn new() -> Self {
        Self::with_base_url("http://127.0.0.1:12700")
    }

    /// Create a new client pointing at a custom base URL.
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            client: reqwest::Client::new(),
        }
    }

    // ── helpers ──────────────────────────────────────────────────────────

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Parse a JSON response, extracting the data or returning a daemon error.
    async fn parse<T: serde::de::DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        let body = resp.text().await?;

        // Try to parse as API envelope first.
        if let Ok(envelope) = serde_json::from_str::<ApiResponse<T>>(&body) {
            if envelope.ok {
                if let Some(data) = envelope.data {
                    return Ok(data);
                }
                // ok=true but no data fields — try parsing T directly from the envelope.
                return serde_json::from_str::<T>(&body).map_err(X0xError::Json);
            }
            return Err(X0xError::Daemon(
                envelope.error.unwrap_or_else(|| format!("HTTP {status}")),
            ));
        }

        // Fallback: try parsing T directly.
        serde_json::from_str::<T>(&body).map_err(X0xError::Json)
    }

    /// Fire-and-forget POST that only checks `ok`.
    async fn post_ok(&self, path: &str, body: &impl serde::Serialize) -> Result<()> {
        let resp = self.client.post(self.url(path)).json(body).send().await?;
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

    /// DELETE that only checks `ok`.
    async fn delete_ok(&self, path: &str) -> Result<()> {
        let resp = self.client.delete(self.url(path)).send().await?;
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

    /// Local agent identity (agent_id, machine_id, optional user_id).
    pub async fn agent(&self) -> Result<AgentIdentity> {
        let resp = self.client.get(self.url("/agent")).send().await?;
        self.parse(resp).await
    }

    /// Connected gossip peer IDs.
    pub async fn peers(&self) -> Result<Vec<String>> {
        let resp = self.client.get(self.url("/peers")).send().await?;
        let list: PeerList = self.parse(resp).await?;
        Ok(list.peers)
    }

    /// Force re-announce identity to the network.
    pub async fn announce(&self) -> Result<()> {
        self.post_ok("/announce", &serde_json::json!({})).await
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
        self.parse(resp).await
    }

    /// Agent presence beacons.
    pub async fn presence(&self) -> Result<Vec<PresenceBeacon>> {
        let resp = self.client.get(self.url("/presence")).send().await?;
        let list: PresenceList = self.parse(resp).await?;
        Ok(list.agents)
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
        Ok(sub.id)
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

    /// Update a contact's trust level and/or label.
    pub async fn update_contact(
        &self,
        agent_id: &str,
        trust_level: Option<TrustLevel>,
        label: Option<&str>,
    ) -> Result<()> {
        let req = UpdateContactRequest {
            trust_level,
            label: label.map(str::to_owned),
        };
        let resp = self
            .client
            .patch(self.url(&format!("/contacts/{agent_id}")))
            .json(&req)
            .send()
            .await?;
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

    /// Remove a contact.
    pub async fn remove_contact(&self, agent_id: &str) -> Result<()> {
        self.delete_ok(&format!("/contacts/{agent_id}")).await
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
        let resp = self
            .client
            .put(self.url(&format!("/groups/{group_id}/display-name")))
            .json(&req)
            .send()
            .await?;
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
        let resp = self
            .client
            .patch(self.url(&format!("/task-lists/{list_id}/tasks/{task_id}")))
            .json(&req)
            .send()
            .await?;
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

    /// Complete a task.
    pub async fn complete_task(&self, list_id: &str, task_id: &str) -> Result<()> {
        let req = UpdateTaskRequest {
            action: "complete".to_owned(),
        };
        let resp = self
            .client
            .patch(self.url(&format!("/task-lists/{list_id}/tasks/{task_id}")))
            .json(&req)
            .send()
            .await?;
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
        let resp = self
            .client
            .put(self.url(&format!("/stores/{store_id}/{key}")))
            .json(&req)
            .send()
            .await?;
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

    /// Get a value from a store. Returns the raw bytes.
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
    pub async fn send_file(
        &self,
        agent_id: &str,
        filename: &str,
        size: u64,
        sha256: Option<&str>,
    ) -> Result<String> {
        let req = SendFileRequest {
            agent_id: agent_id.to_owned(),
            filename: filename.to_owned(),
            size,
            sha256: sha256.map(str::to_owned),
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
        self.parse(resp).await
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
    pub async fn check_upgrade(&self) -> Result<serde_json::Value> {
        let resp = self.client.get(self.url("/upgrade")).send().await?;
        self.parse(resp).await
    }
}

impl Default for X0xClient {
    fn default() -> Self {
        Self::new()
    }
}
