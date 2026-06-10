// SPDX-License-Identifier: MIT OR Apache-2.0

//! Headless parity driver for the Dioxus binary.
//!
//! The module is compiled only with the `e2e-test-mode` feature and is entered
//! only when `COMMUNITAS_TEST_MODE=1` is present. It gives the E2E harness a
//! deterministic line-delimited JSON channel into the same binary that normally
//! launches the Dioxus desktop shell. Each operation exercises the typed
//! `communitas-x0x-client` surface (or records the intentional client gap where
//! the x0xd endpoint is newer than the client crate).

use std::io::{self, BufRead, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use communitas_x0x_client::{
    GroupPolicyPreset, GroupRole, TrustLevel, UpdateGroupPolicyRequest, WsInbound, X0xClient,
    X0xWebSocket,
};
use serde_json::{Value, json};
use tokio::time::timeout;

/// Run the line-delimited JSON E2E command loop.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(command) => handle_command(command).await,
            Err(err) => json!({
                "ok": false,
                "error": format!("invalid json command: {err}"),
            }),
        };

        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_command(command: Value) -> Value {
    let op = command
        .get("op")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if op == "handshake" {
        return json!({
            "ok": true,
            "driver": "communitas-dioxus-e2e",
            "mode": "headless-json",
        });
    }

    match handle_operation(op, &command).await {
        Ok(data) => {
            let mut object = serde_json::Map::new();
            object.insert("ok".to_string(), Value::Bool(true));
            if let Value::Object(fields) = data {
                object.extend(fields);
            } else {
                object.insert("data".to_string(), data);
            }
            Value::Object(object)
        }
        Err(err) => json!({ "ok": false, "error": err }),
    }
}

async fn handle_operation(op: &str, command: &Value) -> Result<Value, String> {
    let client = client_from_env();
    match op {
        "identity.agent_card" => identity_agent_card(&client).await,
        "identity.import_card" => identity_import_card(&client, command).await,
        "identity.export_keypairs" => Ok(json!({
            "status": "unsupported",
            "reason": "communitas-x0x-client does not expose a keypair export/backup method; x0x GUI also defers this pending private-key export design",
            "follow_up": "Add a consent-gated keypair backup API to x0xd and communitas-x0x-client before wiring Dioxus UI export controls.",
        })),
        "identity.user_id" => identity_user_id(&client).await,
        "trust.add_block_trust" => trust_add_block_trust(&client, command).await,
        "trust.machine_pin" => trust_machine_pin(&client, command).await,
        "trust.evaluate" => trust_evaluate(&client, command).await,
        "connectivity.discover_agents" => connectivity_discover_agents(&client).await,
        "connectivity.four_word_bootstrap" => connectivity_four_word_bootstrap(&client).await,
        "messaging.pubsub_roundtrip" => messaging_pubsub_roundtrip(&client).await,
        "groups.policy" => groups_policy(&client, command).await,
        "groups.discover" => groups_discover(&client).await,
        "kv.create_list" => kv_create_list(&client).await,
        "kv.put_get_delete" => kv_put_get_delete(&client).await,
        "kv.access_policy_setup" => kv_access_policy_setup(&client).await,
        "presence.foaf" => presence_foaf(&client).await,
        "upgrade.check" => upgrade_check(&client).await,
        "upgrade.apply" => upgrade_apply(&client).await,
        _ => Err(format!("unknown e2e op: {op}")),
    }
}

fn client_from_env() -> X0xClient {
    let base = std::env::var("X0X_API_BASE")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let token = std::env::var("X0X_API_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty());

    match (base, token) {
        (Some(base), Some(token)) => X0xClient::with_base_url_and_token(&base, &token),
        (Some(base), None) => X0xClient::with_base_url(&base),
        _ => X0xClient::new(),
    }
}

fn required_str(command: &Value, field: &str) -> Result<String, String> {
    command
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("missing required string field `{field}`"))
}

fn optional_str(command: &Value, field: &str) -> Option<String> {
    command
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

fn unique_suffix(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}-{nanos}")
}

async fn identity_agent_card(client: &X0xClient) -> Result<Value, String> {
    let identity = client.agent().await.map_err(|err| err.to_string())?;
    let card = client
        .agent_card(None, Some(true))
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "agent_id": identity.agent_id,
        "machine_id": identity.machine_id,
        "user_id": identity.user_id,
        "card_agent_id": card.card.agent_id,
        "card_machine_id": card.card.machine_id,
        "link": card.link,
        "addresses": card.card.addresses.len(),
    }))
}

async fn identity_import_card(client: &X0xClient, command: &Value) -> Result<Value, String> {
    let card = required_str(command, "card")?;
    let imported = client
        .import_agent_card(&card, Some(TrustLevel::Known))
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "agent_id": imported.agent_id,
        "display_name": imported.display_name,
        "trust_level": imported.trust_level,
    }))
}

async fn identity_user_id(client: &X0xClient) -> Result<Value, String> {
    let user_id = client
        .agent_user_id()
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "user_id": user_id,
        "opt_in_state_read": true,
    }))
}

async fn trust_add_block_trust(client: &X0xClient, command: &Value) -> Result<Value, String> {
    let agent_id = required_str(command, "agent_id")?;
    let label = optional_str(command, "label");
    client
        .add_contact(&agent_id, TrustLevel::Known, label.as_deref())
        .await
        .map_err(|err| err.to_string())?;
    client
        .set_trust(&agent_id, TrustLevel::Trusted)
        .await
        .map_err(|err| err.to_string())?;
    client
        .set_trust(&agent_id, TrustLevel::Blocked)
        .await
        .map_err(|err| err.to_string())?;
    let contacts = client
        .list_contacts()
        .await
        .map_err(|err| err.to_string())?;
    let contact = contacts
        .iter()
        .find(|contact| contact.agent_id == agent_id)
        .ok_or_else(|| format!("contact {agent_id} not found after add/block/trust flow"))?;
    Ok(json!({
        "agent_id": agent_id,
        "final_trust": format!("{:?}", contact.trust_level),
        "label": contact.label,
    }))
}

async fn trust_machine_pin(client: &X0xClient, command: &Value) -> Result<Value, String> {
    let agent_id = required_str(command, "agent_id")?;
    let machine_id = required_str(command, "machine_id")?;
    let label = optional_str(command, "label");
    let _ = client
        .add_contact(&agent_id, TrustLevel::Known, label.as_deref())
        .await;
    client
        .add_machine(&agent_id, &machine_id, Some("fixture machine"), Some(false))
        .await
        .map_err(|err| err.to_string())?;
    client
        .pin_machine(&agent_id, &machine_id)
        .await
        .map_err(|err| err.to_string())?;
    let machines = client
        .list_machines(&agent_id)
        .await
        .map_err(|err| err.to_string())?;
    let pinned = machines
        .iter()
        .any(|machine| machine.machine_id == machine_id && machine.pinned);
    Ok(json!({
        "agent_id": agent_id,
        "machine_id": machine_id,
        "pinned": pinned,
        "machine_count": machines.len(),
    }))
}

async fn trust_evaluate(client: &X0xClient, command: &Value) -> Result<Value, String> {
    let agent_id = required_str(command, "agent_id")?;
    let machine_id = required_str(command, "machine_id")?;
    let _ = client.add_contact(&agent_id, TrustLevel::Known, None).await;
    let _ = client
        .add_machine(&agent_id, &machine_id, Some("fixture machine"), Some(true))
        .await;
    let evaluation = client
        .evaluate_trust(&agent_id, &machine_id)
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "agent_id": agent_id,
        "machine_id": machine_id,
        "decision": evaluation.decision,
    }))
}

async fn connectivity_discover_agents(client: &X0xClient) -> Result<Value, String> {
    let agents = client
        .discovered_agents()
        .await
        .map_err(|err| err.to_string())?;
    let foaf = client
        .presence_foaf(Some(1), Some(500))
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "discovered_agents": agents.len(),
        "foaf_agents": foaf.len(),
    }))
}

async fn connectivity_four_word_bootstrap(client: &X0xClient) -> Result<Value, String> {
    let bootstrap = client
        .bootstrap_cache()
        .await
        .map_err(|err| err.to_string())?;
    let network = client
        .network_status()
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "bootstrap_connection_count": bootstrap.connection_count,
        "bootstrap_peers": bootstrap.connected_peers.len(),
        "can_receive_direct": network.can_receive_direct,
        "external_addresses": network.external_addrs.len(),
    }))
}

async fn messaging_pubsub_roundtrip(client: &X0xClient) -> Result<Value, String> {
    let mut ws = X0xWebSocket::connect()
        .await
        .map_err(|err| format!("connect /ws: {err}"))?;
    let topic = unique_suffix("dioxus-pubsub");
    let message = format!("hello-{topic}");

    let connected = timeout(Duration::from_secs(5), ws.recv())
        .await
        .map_err(|_| "timed out waiting for /ws connected frame".to_string())?
        .ok_or_else(|| "websocket closed before connected frame".to_string())?;
    if !matches!(connected, WsInbound::Connected { .. }) {
        return Err(format!("expected connected frame, got {connected:?}"));
    }

    ws.subscribe(vec![topic.clone()])
        .map_err(|err| format!("subscribe {topic}: {err}"))?;

    loop {
        let inbound = timeout(Duration::from_secs(5), ws.recv())
            .await
            .map_err(|_| format!("timed out waiting for subscribed frame on {topic}"))?
            .ok_or_else(|| "websocket closed before subscribed frame".to_string())?;
        match inbound {
            WsInbound::Subscribed { topics } if topics.contains(&topic) => break,
            WsInbound::Connected { .. } => continue,
            _ => continue,
        }
    }

    client
        .publish(&topic, message.as_bytes())
        .await
        .map_err(|err| format!("publish {topic}: {err}"))?;

    loop {
        let inbound = timeout(Duration::from_secs(10), ws.recv())
            .await
            .map_err(|_| format!("timed out waiting for message on {topic}"))?
            .ok_or_else(|| "websocket closed before message frame".to_string())?;
        match inbound {
            WsInbound::Message {
                topic: inbound_topic,
                payload,
                ..
            } if inbound_topic == topic => {
                let decoded = BASE64
                    .decode(payload.as_bytes())
                    .map_err(|err| format!("decode payload: {err}"))?;
                let echoed = String::from_utf8(decoded)
                    .map_err(|err| format!("payload was not UTF-8: {err}"))?;
                if echoed != message {
                    return Err(format!(
                        "expected echoed payload {message:?}, got {echoed:?}"
                    ));
                }
                return Ok(json!({
                    "topic": topic,
                    "payload": echoed,
                    "stream": "ws",
                }));
            }
            _ => continue,
        }
    }
}

async fn groups_policy(client: &X0xClient, command: &Value) -> Result<Value, String> {
    let member_agent_id = required_str(command, "member_agent_id")?;
    let group_name = unique_suffix("dioxus-policy");
    let created = client
        .create_group_with_preset(
            &group_name,
            Some("Dioxus parity policy group"),
            Some("Dioxus E2E"),
            Some(GroupPolicyPreset::PublicOpen),
        )
        .await
        .map_err(|err| err.to_string())?;
    let patch = UpdateGroupPolicyRequest {
        preset: Some("public_announce".to_string()),
        ..UpdateGroupPolicyRequest::default()
    };
    client
        .update_group_policy(&created.group_id, &patch)
        .await
        .map_err(|err| err.to_string())?;
    let _ = client
        .add_named_group_member(&created.group_id, &member_agent_id, Some("fixture member"))
        .await;
    let _ = client
        .set_named_group_member_role(&created.group_id, &member_agent_id, GroupRole::Moderator)
        .await;
    let _ = client
        .ban_group_member(&created.group_id, &member_agent_id)
        .await;
    let _ = client
        .unban_group_member(&created.group_id, &member_agent_id)
        .await;
    let info = client
        .get_group(&created.group_id)
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "group_id": created.group_id,
        "name": info.name,
        "policy_present": info.policy.is_some(),
        "member_count": info.member_count,
    }))
}

async fn groups_discover(client: &X0xClient) -> Result<Value, String> {
    let group_name = unique_suffix("dioxus-discover");
    let created = client
        .create_group_with_preset(
            &group_name,
            Some("Dioxus parity discoverable group"),
            Some("Dioxus E2E"),
            Some(GroupPolicyPreset::PublicOpen),
        )
        .await
        .map_err(|err| err.to_string())?;
    let by_query = client
        .discover_groups(Some(&group_name))
        .await
        .map_err(|err| err.to_string())?;
    let nearby = client
        .discover_groups_nearby()
        .await
        .map_err(|err| err.to_string())?;
    let found_query = by_query
        .iter()
        .any(|group| group.group_id == created.group_id || group.name == group_name);
    Ok(json!({
        "group_id": created.group_id,
        "query_count": by_query.len(),
        "nearby_count": nearby.len(),
        "found_query": found_query,
    }))
}

async fn kv_create_list(client: &X0xClient) -> Result<Value, String> {
    let topic = unique_suffix("dioxus-kv-list");
    let created = client
        .create_store("Dioxus parity store", &topic)
        .await
        .map_err(|err| err.to_string())?;
    let stores = client.list_stores().await.map_err(|err| err.to_string())?;
    let listed = stores.iter().any(|store| store.id == created.id);
    Ok(json!({
        "store_id": created.id,
        "listed": listed,
        "store_count": stores.len(),
    }))
}

async fn kv_put_get_delete(client: &X0xClient) -> Result<Value, String> {
    let topic = unique_suffix("dioxus-kv-crud");
    let created = client
        .create_store("Dioxus parity CRUD", &topic)
        .await
        .map_err(|err| err.to_string())?;
    client
        .put(
            &created.id,
            "parity-key",
            b"dioxus-parity",
            Some("text/plain"),
        )
        .await
        .map_err(|err| err.to_string())?;
    let value = client
        .get(&created.id, "parity-key")
        .await
        .map_err(|err| err.to_string())?;
    client
        .delete_key(&created.id, "parity-key")
        .await
        .map_err(|err| err.to_string())?;
    let missing_after_delete = client.get(&created.id, "parity-key").await.is_err();
    Ok(json!({
        "store_id": created.id,
        "key": value.key,
        "content_type": value.content_type,
        "missing_after_delete": missing_after_delete,
    }))
}

async fn kv_access_policy_setup(client: &X0xClient) -> Result<Value, String> {
    let topic = unique_suffix("dioxus-kv-private");
    let created = client
        .create_store("Dioxus private parity store", &topic)
        .await
        .map_err(|err| err.to_string())?;
    client
        .put(&created.id, "secret", b"private", Some("text/plain"))
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "store_id": created.id,
        "key": "secret",
    }))
}

async fn presence_foaf(client: &X0xClient) -> Result<Value, String> {
    let agents = client
        .presence_foaf(Some(2), Some(500))
        .await
        .map_err(|err| err.to_string())?;
    Ok(json!({
        "agents": agents.len(),
    }))
}

async fn upgrade_check(client: &X0xClient) -> Result<Value, String> {
    match client.check_upgrade().await {
        Ok(status) => Ok(json!({
            "http_status": 200,
            "update_available": status.update_available,
            "version": status.version,
            "current_version": status.current_version,
            "body": {
                "ok": true,
                "update_available": status.update_available,
                "version": status.version,
                "current_version": status.current_version,
            },
        })),
        Err(client_error) => {
            let raw = raw_x0x_request(reqwest::Method::GET, "/upgrade").await?;
            Ok(json!({
                "http_status": raw.http_status,
                "client_error": client_error.to_string(),
                "body": raw.body,
            }))
        }
    }
}

async fn upgrade_apply(client: &X0xClient) -> Result<Value, String> {
    match client.apply_upgrade().await {
        Ok(response) => Ok(json!({
            "http_status": 200,
            "body": {
                "ok": true,
                "applied": response.applied,
                "version": response.version,
                "reason": response.reason,
            },
        })),
        Err(client_error) => {
            let raw = raw_x0x_request(reqwest::Method::POST, "/upgrade/apply").await?;
            Ok(json!({
                "http_status": raw.http_status,
                "client_error": client_error.to_string(),
                "body": raw.body,
            }))
        }
    }
}

struct RawX0xResponse {
    http_status: u16,
    body: Value,
}

async fn raw_x0x_request(method: reqwest::Method, path: &str) -> Result<RawX0xResponse, String> {
    let base = std::env::var("X0X_API_BASE")
        .map_err(|_| format!("X0X_API_BASE is required for raw {method} {path}"))?;
    let token = std::env::var("X0X_API_TOKEN").unwrap_or_default();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|err| err.to_string())?;
    let mut request = client.request(method, format!("{}{}", base.trim_end_matches('/'), path));
    if !token.trim().is_empty() {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.map_err(|err| err.to_string())?;
    let status = response.status().as_u16();
    let body = response
        .json::<Value>()
        .await
        .map_err(|err| format!("{path} returned non-json body: {err}"))?;
    Ok(RawX0xResponse {
        http_status: status,
        body,
    })
}
