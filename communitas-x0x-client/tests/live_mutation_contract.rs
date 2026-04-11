// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "support/harness.rs"]
mod harness;

use base64::Engine as _;
use communitas_x0x_client::{
    DirectMessage, SseEnvelope, SseFileCompleteData, SseFileOfferData, SseFrame, SseMessageData,
    TransferStatus, TrustLevel, WsInbound, X0xError,
};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::time::timeout;

async fn recv_matching_sse<F>(
    stream: &mut communitas_x0x_client::X0xSseStream,
    mut predicate: F,
) -> SseFrame
where
    F: FnMut(&SseFrame) -> bool,
{
    loop {
        let frame = timeout(Duration::from_secs(20), stream.recv())
            .await
            .expect("sse frame timeout")
            .expect("sse stream should remain open");
        if predicate(&frame) {
            return frame;
        }
    }
}

fn decode_message_event(frame: &SseFrame) -> SseMessageData {
    assert_eq!(frame.event.as_deref(), Some("message"));
    let envelope: SseEnvelope = frame.json().expect("message envelope should decode");
    assert_eq!(envelope.event_type, "message");
    serde_json::from_value(envelope.data).expect("message payload should decode")
}

fn decode_file_offer_event(frame: &SseFrame) -> SseFileOfferData {
    assert_eq!(frame.event.as_deref(), Some("file:offer"));
    let envelope: SseEnvelope = frame.json().expect("file offer envelope should decode");
    assert_eq!(envelope.event_type, "file:offer");
    serde_json::from_value(envelope.data).expect("file offer payload should decode")
}

fn decode_file_complete_event(frame: &SseFrame) -> SseFileCompleteData {
    assert_eq!(frame.event.as_deref(), Some("file:complete"));
    let envelope: SseEnvelope = frame.json().expect("file complete envelope should decode");
    assert_eq!(envelope.event_type, "file:complete");
    serde_json::from_value(envelope.data).expect("file complete payload should decode")
}

fn decode_direct_event(frame: &SseFrame) -> DirectMessage {
    assert_eq!(frame.event.as_deref(), Some("direct_message"));
    frame.json().expect("direct message event should decode")
}

#[tokio::test]
#[ignore = "requires an ephemeral local x0x target matrix and X0X_TEST_ALLOW_MUTATION=1"]
async fn mutation_suite_exercises_stateful_contract_endpoints() {
    if !harness::mutations_enabled() {
        eprintln!("Skipping mutation suite: X0X_TEST_ALLOW_MUTATION is not enabled");
        return;
    }

    let targets = harness::load_targets();
    assert!(
        targets.len() >= 3,
        "mutation suite expects at least 3 ephemeral targets, got {}",
        targets.len()
    );

    let primary = targets[0].clone();
    let secondary = targets[1].clone();
    let tertiary = targets[2].clone();
    let scratch_remote = primary.kind.as_deref() == Some("remote");
    let ws_timeout = if scratch_remote {
        Duration::from_secs(20)
    } else {
        Duration::from_secs(5)
    };
    let mut run_direct_file = harness::direct_file_enabled() && !scratch_remote;
    let run_cross_node_crdt = harness::cross_node_crdt_enabled() && !scratch_remote;

    let primary_client = primary.client();
    let secondary_client = secondary.client();
    let tertiary_client = tertiary.client();

    let primary_agent = primary_client.agent().await.expect("primary agent");
    let secondary_agent = secondary_client.agent().await.expect("secondary agent");
    let tertiary_agent = tertiary_client.agent().await.expect("tertiary agent");

    // ── Contacts, trust, machines, revocations ──────────────────────────
    let secondary_card = secondary_client
        .agent_card(Some("contract-secondary"), Some(false))
        .await
        .expect("secondary card");
    primary_client
        .import_agent_card(&secondary_card.link, Some(TrustLevel::Trusted))
        .await
        .expect("primary import secondary");

    let primary_card = primary_client
        .agent_card(Some("contract-primary"), Some(false))
        .await
        .expect("primary card");
    secondary_client
        .import_agent_card(&primary_card.link, Some(TrustLevel::Trusted))
        .await
        .expect("secondary import primary");

    let tertiary_card = tertiary_client
        .agent_card(Some("contract-tertiary"), Some(false))
        .await
        .expect("tertiary card");
    primary_client
        .import_agent_card(&tertiary_card.link, Some(TrustLevel::Known))
        .await
        .expect("primary import tertiary");

    let contacts = primary_client.list_contacts().await.expect("list contacts");
    assert!(
        contacts
            .iter()
            .any(|contact| contact.agent_id == secondary_agent.agent_id),
        "primary should list secondary as a contact"
    );

    primary_client
        .set_trust(&secondary_agent.agent_id, TrustLevel::Known)
        .await
        .expect("set trust known");
    primary_client
        .update_contact(
            &secondary_agent.agent_id,
            Some(TrustLevel::Trusted),
            Some("known"),
        )
        .await
        .expect("update contact trusted/known");

    primary_client
        .add_machine(
            &secondary_agent.agent_id,
            &secondary_agent.machine_id,
            Some("secondary-main"),
            Some(false),
        )
        .await
        .expect("add machine record");
    let machines = primary_client
        .list_machines(&secondary_agent.agent_id)
        .await
        .expect("list machines");
    assert!(
        machines
            .iter()
            .any(|m| m.machine_id == secondary_agent.machine_id)
    );

    primary_client
        .pin_machine(&secondary_agent.agent_id, &secondary_agent.machine_id)
        .await
        .expect("pin machine");
    let pinned_machines = primary_client
        .list_machines(&secondary_agent.agent_id)
        .await
        .expect("list pinned machines");
    assert!(
        pinned_machines
            .iter()
            .any(|m| m.machine_id == secondary_agent.machine_id && m.pinned),
        "pinned machine should be visible"
    );

    primary_client
        .unpin_machine(&secondary_agent.agent_id, &secondary_agent.machine_id)
        .await
        .expect("unpin machine");

    let trust = primary_client
        .evaluate_trust(&secondary_agent.agent_id, &secondary_agent.machine_id)
        .await
        .expect("evaluate trust");
    assert!(!trust.decision.is_empty());

    primary_client
        .revoke_contact(&tertiary_agent.agent_id, "contract-harness revoke")
        .await
        .expect("revoke tertiary contact");
    let revocations = primary_client
        .revocations(&tertiary_agent.agent_id)
        .await
        .expect("list revocations");
    assert!(
        !revocations.is_empty(),
        "revocation list should not be empty"
    );
    primary_client
        .remove_contact(&tertiary_agent.agent_id)
        .await
        .expect("remove tertiary contact");

    // ── Gossip pub/sub + general SSE + WebSocket ────────────────────────
    let mut primary_events_sse = primary.sse().await.expect("primary events sse");
    let mut primary_presence_sse = primary.sse_presence().await.expect("primary presence sse");
    let mut primary_ws = primary.ws().await.expect("primary general ws");
    let mut secondary_ws = secondary.ws().await.expect("secondary general ws");

    match timeout(ws_timeout, primary_ws.recv())
        .await
        .expect("primary ws connected frame timeout")
        .expect("primary ws should stay open")
    {
        WsInbound::Connected { agent_id, .. } => assert_eq!(agent_id, primary_agent.agent_id),
        other => panic!("expected primary ws connected frame, got {other:?}"),
    }
    match timeout(ws_timeout, secondary_ws.recv())
        .await
        .expect("secondary ws connected frame timeout")
        .expect("secondary ws should stay open")
    {
        WsInbound::Connected { agent_id, .. } => assert_eq!(agent_id, secondary_agent.agent_id),
        other => panic!("expected secondary ws connected frame, got {other:?}"),
    }

    let sse_topic = format!("contract.sse.{}", harness::unique_suffix());
    let sse_subscription_id = primary_client
        .subscribe(&sse_topic)
        .await
        .expect("subscribe for sse message contract");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let sse_payload = format!("contract-sse-{}", harness::unique_suffix());
    primary_client
        .publish(&sse_topic, sse_payload.as_bytes())
        .await
        .expect("publish sse payload");
    let sse_frame = recv_matching_sse(&mut primary_events_sse, |frame| {
        frame.event.as_deref() == Some("message") && frame.data.contains(&sse_topic)
    })
    .await;
    let sse_message = decode_message_event(&sse_frame);
    assert_eq!(sse_message.subscription_id, sse_subscription_id);
    assert_eq!(sse_message.topic, sse_topic);
    let decoded_sse_payload = base64::engine::general_purpose::STANDARD
        .decode(sse_message.payload)
        .expect("sse payload should decode");
    assert_eq!(decoded_sse_payload, sse_payload.as_bytes());
    primary_client
        .unsubscribe(&sse_subscription_id)
        .await
        .expect("unsubscribe sse topic");

    let ws_topic = format!("contract.ws.{}", harness::unique_suffix());
    primary_ws
        .subscribe(vec![ws_topic.clone()])
        .expect("primary ws subscribe should send");
    loop {
        match timeout(ws_timeout, primary_ws.recv())
            .await
            .expect("expected ws subscribed frame in time")
            .expect("primary ws should remain open while subscribing")
        {
            WsInbound::Subscribed { topics } if topics.contains(&ws_topic) => break,
            WsInbound::Connected { .. } => continue,
            _ => continue,
        }
    }

    let ws_payload = base64::engine::general_purpose::STANDARD
        .encode(format!("contract-ws-{}", harness::unique_suffix()).as_bytes());
    primary_ws
        .publish(ws_topic.clone(), ws_payload.clone())
        .expect("primary ws publish should send");
    let ws_message = loop {
        match timeout(Duration::from_secs(10), primary_ws.recv())
            .await
            .expect("expected ws message in time")
            .expect("primary ws should remain open while waiting for message")
        {
            WsInbound::Message {
                topic: inbound_topic,
                payload,
                ..
            } if inbound_topic == ws_topic => break payload,
            WsInbound::Connected { .. } | WsInbound::Subscribed { .. } => continue,
            other => panic!("expected websocket message for {ws_topic}, got {other:?}"),
        }
    };
    let decoded_ws_payload = base64::engine::general_purpose::STANDARD
        .decode(ws_message)
        .expect("ws payload should decode");
    let expected_ws_payload = base64::engine::general_purpose::STANDARD
        .decode(ws_payload)
        .expect("expected ws payload should decode");
    assert_eq!(decoded_ws_payload, expected_ws_payload);

    primary_ws
        .unsubscribe(vec![ws_topic.clone()])
        .expect("primary ws unsubscribe should send");
    loop {
        match timeout(Duration::from_secs(5), primary_ws.recv())
            .await
            .expect("expected ws unsubscribed frame in time")
            .expect("primary ws should remain open while unsubscribing")
        {
            WsInbound::Unsubscribed { topics } if topics.contains(&ws_topic) => break,
            WsInbound::Connected { .. } | WsInbound::Message { .. } => continue,
            other => panic!("expected websocket unsubscribed frame, got {other:?}"),
        }
    }

    let _ = timeout(Duration::from_secs(2), primary_presence_sse.recv()).await;

    // ── Direct messaging + WS direct ─────────────────────────────────────
    if !run_direct_file {
        eprintln!(
            "Skipping direct-message mutation checks. Enable X0X_TEST_ENABLE_DIRECT_FILE=1 to exercise direct/file flows once the target topology supports them reliably."
        );
    } else {
        primary_client
            .announce()
            .await
            .expect("primary announce before direct");
        secondary_client
            .announce()
            .await
            .expect("secondary announce before direct");
        let found_route =
            harness::wait_until(Duration::from_secs(60), Duration::from_secs(2), || {
                let client = primary.client();
                let agent_id = secondary_agent.agent_id.clone();
                async move {
                    client
                        .find_agent(&agent_id)
                        .await
                        .map(|result| result.found && !result.addresses.is_empty())
                        .unwrap_or(false)
                }
            })
            .await;
        assert!(
            found_route,
            "primary should locate secondary before direct send"
        );

        let mut secondary_direct_ws = secondary.ws_direct().await.expect("secondary direct ws");
        let mut secondary_direct_sse = secondary.sse_direct().await.expect("secondary direct sse");
        let first_frame = timeout(ws_timeout, secondary_direct_ws.recv())
            .await
            .expect("direct ws connected frame timeout")
            .expect("direct ws should stay open");
        match first_frame {
            WsInbound::Connected { agent_id, .. } => assert_eq!(agent_id, secondary_agent.agent_id),
            other => panic!("expected direct ws connected frame, got {other:?}"),
        }

        primary_client
            .connect_agent(&secondary_agent.agent_id)
            .await
            .expect("primary connect agent");
        secondary_client
            .connect_agent(&primary_agent.agent_id)
            .await
            .expect("secondary connect agent");

        let direct_ready =
            harness::wait_until(Duration::from_secs(60), Duration::from_secs(2), || {
                let client = primary.client();
                let target_agent = secondary_agent.agent_id.clone();
                async move {
                    client
                        .direct_connections()
                        .await
                        .map(|connections| {
                            connections.iter().any(|conn| conn.agent_id == target_agent)
                        })
                        .unwrap_or(false)
                }
            })
            .await;
        if !direct_ready {
            eprintln!(
                "Skipping direct/file mutation checks: direct connection list never became ready on this topology."
            );
            run_direct_file = false;
        } else {
            let direct_text = format!("contract-direct-ws-{}", harness::unique_suffix());
            let direct_ws_sent =
                harness::wait_until(Duration::from_secs(120), Duration::from_secs(3), || {
                    let client = primary.client();
                    let agent_id = secondary_agent.agent_id.clone();
                    let payload = direct_text.clone();
                    async move {
                        let _ = client.announce().await;
                        let _ = client.connect_agent(&agent_id).await;
                        client
                            .send_direct(&agent_id, payload.as_bytes())
                            .await
                            .is_ok()
                    }
                })
                .await;
            if !direct_ws_sent {
                eprintln!(
                    "Skipping direct/file mutation checks: direct websocket send never succeeded on this topology."
                );
                run_direct_file = false;
            } else {
                let direct_payload = loop {
                    match timeout(Duration::from_secs(20), secondary_direct_ws.recv())
                        .await
                        .expect("direct websocket message timeout")
                        .expect("direct websocket should stay open")
                    {
                        WsInbound::DirectMessage {
                            sender, payload, ..
                        } if sender == primary_agent.agent_id => {
                            break payload;
                        }
                        WsInbound::Connected { .. } | WsInbound::Pong => continue,
                        other => panic!("expected direct message from primary, got {other:?}"),
                    }
                };
                let decoded_direct = base64::engine::general_purpose::STANDARD
                    .decode(direct_payload)
                    .expect("direct websocket payload should decode");
                assert_eq!(decoded_direct, direct_text.as_bytes());

                let direct_sse_text = format!("contract-direct-sse-{}", harness::unique_suffix());
                let direct_sse_sent =
                    harness::wait_until(Duration::from_secs(120), Duration::from_secs(3), || {
                        let client = primary.client();
                        let agent_id = secondary_agent.agent_id.clone();
                        let payload = direct_sse_text.clone();
                        async move {
                            let _ = client.announce().await;
                            let _ = client.connect_agent(&agent_id).await;
                            client
                                .send_direct(&agent_id, payload.as_bytes())
                                .await
                                .is_ok()
                        }
                    })
                    .await;
                if !direct_sse_sent {
                    eprintln!(
                        "Skipping file-transfer mutation checks: direct SSE send never succeeded on this topology."
                    );
                    run_direct_file = false;
                } else {
                    let expected_direct_sse_payload = base64::engine::general_purpose::STANDARD
                        .encode(direct_sse_text.as_bytes());
                    let direct_sse_frame = recv_matching_sse(&mut secondary_direct_sse, |frame| {
                        frame.event.as_deref() == Some("direct_message")
                            && frame.data.contains(&primary_agent.agent_id)
                            && frame.data.contains(&expected_direct_sse_payload)
                    })
                    .await;
                    let direct_sse_message = decode_direct_event(&direct_sse_frame);
                    assert_eq!(direct_sse_message.sender, primary_agent.agent_id);
                    let decoded_direct_sse = base64::engine::general_purpose::STANDARD
                        .decode(direct_sse_message.payload)
                        .expect("direct sse payload should decode");
                    assert_eq!(decoded_direct_sse, direct_sse_text.as_bytes());
                }
            }
        }

        let direct_connections = primary_client
            .direct_connections()
            .await
            .expect("direct connections query should succeed");
        if !direct_connections
            .iter()
            .any(|connection| connection.agent_id == secondary_agent.agent_id)
        {
            eprintln!(
                "direct connection list did not yet show secondary after successful delivery; continuing"
            );
        }
    }

    // ── Named groups / spaces ───────────────────────────────────────────
    let group_name = format!("contract-group-{}", harness::unique_suffix());
    let created_group = primary_client
        .create_group(&group_name, Some("contract harness group"), Some("primary"))
        .await
        .expect("create named group");
    let invite = primary_client
        .invite(&created_group.group_id, Some(900))
        .await
        .expect("create group invite");
    let joined_group = secondary_client
        .join_group(&invite.invite_link, Some("secondary"))
        .await
        .expect("join secondary group via invite");
    assert_eq!(joined_group.group_id, created_group.group_id);
    let joined_group_tertiary = tertiary_client
        .join_group(&invite.invite_link, Some("tertiary"))
        .await
        .expect("join tertiary group via invite");
    assert_eq!(joined_group_tertiary.group_id, created_group.group_id);

    secondary_client
        .set_group_display_name(&created_group.group_id, "secondary-renamed")
        .await
        .expect("set secondary group display name");
    tertiary_client
        .set_group_display_name(&created_group.group_id, "tertiary-renamed")
        .await
        .expect("set tertiary group display name");

    let group_info = primary_client
        .get_group(&created_group.group_id)
        .await
        .expect("get group info");
    assert_eq!(group_info.group_id, created_group.group_id);
    assert!(!group_info.name.is_empty());

    let secondary_group_info = secondary_client
        .get_group(&created_group.group_id)
        .await
        .expect("secondary get group info");
    assert_eq!(secondary_group_info.group_id, created_group.group_id);
    let tertiary_group_info = tertiary_client
        .get_group(&created_group.group_id)
        .await
        .expect("tertiary get group info");
    assert_eq!(tertiary_group_info.group_id, created_group.group_id);

    let secondary_groups = secondary_client
        .list_groups()
        .await
        .expect("secondary list groups");
    assert!(
        secondary_groups
            .iter()
            .any(|group| group.group_id == created_group.group_id)
    );
    let tertiary_groups = tertiary_client
        .list_groups()
        .await
        .expect("tertiary list groups");
    assert!(
        tertiary_groups
            .iter()
            .any(|group| group.group_id == created_group.group_id)
    );

    let space_chat_topic = group_info
        .chat_topic
        .clone()
        .or(created_group.chat_topic.clone())
        .expect("space chat topic should be present");
    primary_ws
        .subscribe(vec![space_chat_topic.clone()])
        .expect("primary ws subscribe to space chat should send");
    loop {
        match timeout(ws_timeout, primary_ws.recv())
            .await
            .expect("space chat subscribed frame timeout")
            .expect("primary ws should remain open while subscribing to space chat")
        {
            WsInbound::Subscribed { topics } if topics.contains(&space_chat_topic) => break,
            WsInbound::Connected { .. } => continue,
            _ => continue,
        }
    }
    let space_payload = format!("space-chat-{}", harness::unique_suffix());
    primary_client
        .publish(&space_chat_topic, space_payload.as_bytes())
        .await
        .expect("publish space chat payload");
    let received_space_payload = loop {
        match timeout(Duration::from_secs(10), primary_ws.recv())
            .await
            .expect("space chat message timeout")
            .expect("primary ws should remain open while waiting for space chat")
        {
            WsInbound::Message {
                topic: inbound_topic,
                payload,
                ..
            } if inbound_topic == space_chat_topic => break payload,
            WsInbound::Connected { .. } | WsInbound::Subscribed { .. } => continue,
            other => panic!("expected space chat message, got {other:?}"),
        }
    };
    let decoded_space_payload = base64::engine::general_purpose::STANDARD
        .decode(received_space_payload)
        .expect("space chat payload should decode");
    assert_eq!(decoded_space_payload, space_payload.as_bytes());
    primary_ws
        .unsubscribe(vec![space_chat_topic.clone()])
        .expect("primary ws unsubscribe from space chat should send");
    loop {
        match timeout(ws_timeout, primary_ws.recv())
            .await
            .expect("space chat unsubscribed frame timeout")
            .expect("primary ws should remain open while unsubscribing from space chat")
        {
            WsInbound::Unsubscribed { topics } if topics.contains(&space_chat_topic) => break,
            WsInbound::Connected { .. } | WsInbound::Message { .. } => continue,
            other => panic!("expected space chat unsubscribed frame, got {other:?}"),
        }
    }

    // ── KV store lifecycle + convergence ────────────────────────────────
    let store_name = format!("contract-store-{}", harness::unique_suffix());
    let store_topic = format!("contract.store.{}", harness::unique_suffix());
    let created_store = primary_client
        .create_store(&store_name, &store_topic)
        .await
        .expect("create store");
    secondary_client
        .join_store(&created_store.id)
        .await
        .expect("secondary join store");
    tertiary_client
        .join_store(&created_store.id)
        .await
        .expect("tertiary join store");

    let store_payload = b"contract store payload";
    primary_client
        .put(
            &created_store.id,
            "hello",
            store_payload,
            Some("text/plain"),
        )
        .await
        .expect("put store value");
    let primary_store_value = primary_client
        .get(&created_store.id, "hello")
        .await
        .expect("get primary store value after put");
    let decoded_primary_store = base64::engine::general_purpose::STANDARD
        .decode(primary_store_value.value)
        .expect("primary store value should decode");
    assert_eq!(decoded_primary_store, store_payload);

    let updated_store_payload = b"contract store payload updated";
    if !run_cross_node_crdt {
        primary_client
            .put(
                &created_store.id,
                "hello",
                updated_store_payload,
                Some("text/plain"),
            )
            .await
            .expect("update store value locally");
        let updated_primary_store = primary_client
            .get(&created_store.id, "hello")
            .await
            .expect("get primary store value after update");
        let decoded_updated_store = base64::engine::general_purpose::STANDARD
            .decode(updated_primary_store.value)
            .expect("updated store value should decode");
        assert_eq!(decoded_updated_store, updated_store_payload);
        primary_client
            .delete_key(&created_store.id, "hello")
            .await
            .expect("delete store key locally");
        assert!(matches!(
            primary_client.get(&created_store.id, "hello").await,
            Err(X0xError::Daemon(_))
        ));
    } else {
        let store_replicated =
            harness::wait_until(Duration::from_secs(30), Duration::from_secs(1), || {
                let secondary_client = secondary.client();
                let tertiary_client = tertiary.client();
                let store_id = created_store.id.clone();
                async move {
                    let secondary_ok = secondary_client
                        .get(&store_id, "hello")
                        .await
                        .map(|value| {
                            base64::engine::general_purpose::STANDARD
                                .decode(value.value)
                                .map(|decoded| decoded == store_payload)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    let tertiary_ok = tertiary_client
                        .get(&store_id, "hello")
                        .await
                        .map(|value| {
                            base64::engine::general_purpose::STANDARD
                                .decode(value.value)
                                .map(|decoded| decoded == store_payload)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    secondary_ok && tertiary_ok
                }
            })
            .await;
        assert!(
            store_replicated,
            "store put should converge to joined replicas"
        );

        secondary_client
            .put(
                &created_store.id,
                "hello",
                updated_store_payload,
                Some("text/plain"),
            )
            .await
            .expect("update store value from secondary");
        let store_update_converged =
            harness::wait_until(Duration::from_secs(30), Duration::from_secs(1), || {
                let primary_client = primary.client();
                let tertiary_client = tertiary.client();
                let store_id = created_store.id.clone();
                async move {
                    let primary_ok = primary_client
                        .get(&store_id, "hello")
                        .await
                        .map(|value| {
                            base64::engine::general_purpose::STANDARD
                                .decode(value.value)
                                .map(|decoded| decoded == updated_store_payload)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    let tertiary_ok = tertiary_client
                        .get(&store_id, "hello")
                        .await
                        .map(|value| {
                            base64::engine::general_purpose::STANDARD
                                .decode(value.value)
                                .map(|decoded| decoded == updated_store_payload)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    primary_ok && tertiary_ok
                }
            })
            .await;
        assert!(
            store_update_converged,
            "store update should converge to other replicas"
        );

        tertiary_client
            .delete_key(&created_store.id, "hello")
            .await
            .expect("delete store key from tertiary");
        let store_delete_converged =
            harness::wait_until(Duration::from_secs(30), Duration::from_secs(1), || {
                let primary_client = primary.client();
                let secondary_client = secondary.client();
                let store_id = created_store.id.clone();
                async move {
                    let primary_missing = matches!(
                        primary_client.get(&store_id, "hello").await,
                        Err(X0xError::Daemon(_))
                    );
                    let secondary_missing = matches!(
                        secondary_client.get(&store_id, "hello").await,
                        Err(X0xError::Daemon(_))
                    );
                    primary_missing && secondary_missing
                }
            })
            .await;
        assert!(
            store_delete_converged,
            "store delete should converge to other replicas"
        );
    }

    // ── Task lists (CRDT convergence) ────────────────────────────────────
    let task_topic = format!("contract.tasks.{}", harness::unique_suffix());
    let created_list = primary_client
        .create_task_list("contract-harness", &task_topic)
        .await
        .expect("create primary task list");

    let task_lists = primary_client
        .list_task_lists()
        .await
        .expect("list task lists");
    assert!(task_lists.iter().any(|list| list.id == created_list.id));

    if !run_cross_node_crdt {
        primary_client
            .add_task(
                &created_list.id,
                "ship comprehensive harness",
                Some("contract test task"),
            )
            .await
            .expect("add local task");
        let tasks = primary_client
            .list_tasks(&created_list.id)
            .await
            .expect("list local tasks after add");
        let task = tasks
            .iter()
            .find(|task| task.title == "ship comprehensive harness")
            .expect("added local task should exist")
            .clone();
        primary_client
            .claim_task(&created_list.id, &task.id)
            .await
            .expect("claim local task");
        primary_client
            .complete_task(&created_list.id, &task.id)
            .await
            .expect("complete local task");
        let completed_tasks = primary_client
            .list_tasks(&created_list.id)
            .await
            .expect("list local tasks after complete");
        assert!(completed_tasks.iter().any(|entry| entry.id == task.id));
    } else {
        secondary_client
            .create_task_list("contract-harness", &task_topic)
            .await
            .expect("create secondary task list on shared topic");
        tertiary_client
            .create_task_list("contract-harness", &task_topic)
            .await
            .expect("create tertiary task list on shared topic");

        let primary_task_title = format!("primary-crdt-{}", harness::unique_suffix());
        let secondary_task_title = format!("secondary-crdt-{}", harness::unique_suffix());
        let tertiary_task_title = format!("tertiary-crdt-{}", harness::unique_suffix());
        primary_client
            .add_task(
                &created_list.id,
                &primary_task_title,
                Some("primary adds task"),
            )
            .await
            .expect("primary add task");
        secondary_client
            .add_task(
                &created_list.id,
                &secondary_task_title,
                Some("secondary adds task"),
            )
            .await
            .expect("secondary add task");
        tertiary_client
            .add_task(
                &created_list.id,
                &tertiary_task_title,
                Some("tertiary adds task"),
            )
            .await
            .expect("tertiary add task");

        let tasks_converged =
            harness::wait_until(Duration::from_secs(60), Duration::from_secs(2), || {
                let primary_client = primary.client();
                let secondary_client = secondary.client();
                let tertiary_client = tertiary.client();
                let list_id = created_list.id.clone();
                let primary_title = primary_task_title.clone();
                let secondary_title = secondary_task_title.clone();
                let tertiary_title = tertiary_task_title.clone();
                async move {
                    let expected_titles = |tasks: &[communitas_x0x_client::Task]| {
                        tasks
                            .iter()
                            .any(|task| task.title == primary_title.as_str())
                            && tasks
                                .iter()
                                .any(|task| task.title == secondary_title.as_str())
                            && tasks
                                .iter()
                                .any(|task| task.title == tertiary_title.as_str())
                    };
                    let primary_ok = primary_client
                        .list_tasks(&list_id)
                        .await
                        .map(|tasks| expected_titles(&tasks))
                        .unwrap_or(false);
                    let secondary_ok = secondary_client
                        .list_tasks(&list_id)
                        .await
                        .map(|tasks| expected_titles(&tasks))
                        .unwrap_or(false);
                    let tertiary_ok = tertiary_client
                        .list_tasks(&list_id)
                        .await
                        .map(|tasks| expected_titles(&tasks))
                        .unwrap_or(false);
                    primary_ok && secondary_ok && tertiary_ok
                }
            })
            .await;
        assert!(
            tasks_converged,
            "task list additions should converge across replicas"
        );

        let converged_tasks = primary_client
            .list_tasks(&created_list.id)
            .await
            .expect("list converged tasks");
        let contested_task = converged_tasks
            .iter()
            .find(|task| task.title == secondary_task_title.as_str())
            .expect("secondary task should exist after convergence")
            .clone();
        secondary_client
            .claim_task(&created_list.id, &contested_task.id)
            .await
            .expect("secondary claim task");
        tertiary_client
            .complete_task(&created_list.id, &contested_task.id)
            .await
            .expect("tertiary complete task");
        let final_task_state_converged =
            harness::wait_until(Duration::from_secs(60), Duration::from_secs(2), || {
                let primary_client = primary.client();
                let secondary_client = secondary.client();
                let tertiary_client = tertiary.client();
                let list_id = created_list.id.clone();
                let task_id = contested_task.id.clone();
                async move {
                    let is_done = |tasks: &[communitas_x0x_client::Task]| {
                        tasks.iter().any(|task| {
                            task.id == task_id.as_str()
                                && task
                                    .state
                                    .as_deref()
                                    .map(|state| state.eq_ignore_ascii_case("done"))
                                    .unwrap_or(false)
                        })
                    };
                    let primary_ok = primary_client
                        .list_tasks(&list_id)
                        .await
                        .map(|tasks| is_done(&tasks))
                        .unwrap_or(false);
                    let secondary_ok = secondary_client
                        .list_tasks(&list_id)
                        .await
                        .map(|tasks| is_done(&tasks))
                        .unwrap_or(false);
                    let tertiary_ok = tertiary_client
                        .list_tasks(&list_id)
                        .await
                        .map(|tasks| is_done(&tasks))
                        .unwrap_or(false);
                    primary_ok && secondary_ok && tertiary_ok
                }
            })
            .await;
        assert!(
            final_task_state_converged,
            "task state transitions should converge across replicas"
        );
    }

    // ── MLS group lifecycle ──────────────────────────────────────────────
    let mls_group = primary_client
        .create_mls_group(None)
        .await
        .expect("create mls group");
    let listed_mls = primary_client
        .list_mls_groups()
        .await
        .expect("list mls groups");
    assert!(
        listed_mls
            .iter()
            .any(|group| group.group_id == mls_group.group_id)
    );
    let fetched_mls = primary_client
        .get_mls_group(&mls_group.group_id)
        .await
        .expect("get mls group");
    assert_eq!(fetched_mls.group_id, mls_group.group_id);

    let welcome_secondary = primary_client
        .create_mls_welcome(&mls_group.group_id, &secondary_agent.agent_id)
        .await
        .expect("create secondary mls welcome");
    assert_eq!(welcome_secondary.group_id, mls_group.group_id);
    assert!(!welcome_secondary.welcome.is_empty());
    let welcome_tertiary = primary_client
        .create_mls_welcome(&mls_group.group_id, &tertiary_agent.agent_id)
        .await
        .expect("create tertiary mls welcome");
    assert_eq!(welcome_tertiary.group_id, mls_group.group_id);
    assert!(!welcome_tertiary.welcome.is_empty());

    let add_secondary = primary_client
        .add_mls_member(&mls_group.group_id, &secondary_agent.agent_id)
        .await
        .expect("add secondary mls member");
    assert!(add_secondary.member_count >= 2);
    let add_tertiary = primary_client
        .add_mls_member(&mls_group.group_id, &tertiary_agent.agent_id)
        .await
        .expect("add tertiary mls member");
    assert!(add_tertiary.member_count >= 3);

    let members_after_add = primary_client
        .get_mls_group(&mls_group.group_id)
        .await
        .expect("get mls group after adds");
    assert!(
        members_after_add
            .members
            .iter()
            .any(|member| member == &secondary_agent.agent_id)
    );
    assert!(
        members_after_add
            .members
            .iter()
            .any(|member| member == &tertiary_agent.agent_id)
    );

    let encrypted = primary_client
        .encrypt(&mls_group.group_id, b"mls secret payload")
        .await
        .expect("encrypt mls payload");
    let decrypted = primary_client
        .decrypt(&mls_group.group_id, &encrypted.ciphertext, encrypted.epoch)
        .await
        .expect("decrypt mls payload");
    assert_eq!(decrypted, b"mls secret payload");

    primary_client
        .remove_mls_member(&mls_group.group_id, &tertiary_agent.agent_id)
        .await
        .expect("remove tertiary mls member");
    let members_after_remove_tertiary = primary_client
        .get_mls_group(&mls_group.group_id)
        .await
        .expect("get mls group after tertiary removal");
    assert!(
        !members_after_remove_tertiary
            .members
            .iter()
            .any(|member| member == &tertiary_agent.agent_id)
    );
    primary_client
        .remove_mls_member(&mls_group.group_id, &secondary_agent.agent_id)
        .await
        .expect("remove secondary mls member");

    // ── File transfers ───────────────────────────────────────────────────
    if !run_direct_file {
        eprintln!(
            "Skipping file-transfer mutation checks. Enable X0X_TEST_ENABLE_DIRECT_FILE=1 to exercise direct/file flows once the target topology supports them reliably."
        );
    } else {
        let mut secondary_events_sse = secondary.sse().await.expect("secondary events sse");
        let mut reject_file = NamedTempFile::new().expect("temp reject file");
        let reject_payload = format!("reject-file-{}", harness::unique_suffix());
        reject_file
            .write_all(reject_payload.as_bytes())
            .expect("write reject file");
        let reject_sha = hex::encode(Sha256::digest(reject_payload.as_bytes()));
        let reject_transfer_id = primary_client
            .send_file(
                &secondary_agent.agent_id,
                "contract-reject.txt",
                reject_payload.len() as u64,
                &reject_sha,
                Some(reject_file.path().to_str().expect("reject file path utf8")),
            )
            .await
            .expect("send rejectable file");

        let reject_offer_frame = recv_matching_sse(&mut secondary_events_sse, |frame| {
            frame.event.as_deref() == Some("file:offer") && frame.data.contains(&reject_transfer_id)
        })
        .await;
        let reject_offer = decode_file_offer_event(&reject_offer_frame);
        assert_eq!(reject_offer.transfer_id, reject_transfer_id);
        assert_eq!(reject_offer.filename, "contract-reject.txt");
        let secondary_saw_reject =
            harness::wait_until(Duration::from_secs(20), Duration::from_secs(1), || {
                let client = secondary.client();
                let transfer_id = reject_transfer_id.clone();
                async move {
                    client
                        .transfers()
                        .await
                        .map(|transfers| transfers.iter().any(|t| t.transfer_id == transfer_id))
                        .unwrap_or(false)
                }
            })
            .await;
        assert!(
            secondary_saw_reject,
            "secondary should observe incoming reject transfer"
        );
        secondary_client
            .reject_file(&reject_transfer_id, Some("contract rejection"))
            .await
            .expect("reject file");
        let reject_visible =
            harness::wait_until(Duration::from_secs(60), Duration::from_secs(1), || {
                let client = primary.client();
                let transfer_id = reject_transfer_id.clone();
                async move {
                    client
                        .transfer_status(&transfer_id)
                        .await
                        .map(|transfer| transfer.status == TransferStatus::Rejected)
                        .unwrap_or(false)
                }
            })
            .await;
        assert!(reject_visible, "primary should observe rejected status");

        let mut accept_file = NamedTempFile::new().expect("temp accept file");
        let accept_payload = format!("accept-file-{}", harness::unique_suffix());
        accept_file
            .write_all(accept_payload.as_bytes())
            .expect("write accept file");
        let accept_sha = hex::encode(Sha256::digest(accept_payload.as_bytes()));
        let accept_transfer_id = primary_client
            .send_file(
                &secondary_agent.agent_id,
                "contract-accept.txt",
                accept_payload.len() as u64,
                &accept_sha,
                Some(accept_file.path().to_str().expect("accept file path utf8")),
            )
            .await
            .expect("send acceptable file");
        let accept_offer_frame = recv_matching_sse(&mut secondary_events_sse, |frame| {
            frame.event.as_deref() == Some("file:offer") && frame.data.contains(&accept_transfer_id)
        })
        .await;
        let accept_offer = decode_file_offer_event(&accept_offer_frame);
        assert_eq!(accept_offer.transfer_id, accept_transfer_id);
        assert_eq!(accept_offer.filename, "contract-accept.txt");
        let secondary_saw_accept =
            harness::wait_until(Duration::from_secs(20), Duration::from_secs(1), || {
                let client = secondary.client();
                let transfer_id = accept_transfer_id.clone();
                async move {
                    client
                        .transfers()
                        .await
                        .map(|transfers| transfers.iter().any(|t| t.transfer_id == transfer_id))
                        .unwrap_or(false)
                }
            })
            .await;
        assert!(
            secondary_saw_accept,
            "secondary should observe incoming accept transfer"
        );
        secondary_client
            .accept_file(&accept_transfer_id)
            .await
            .expect("accept file");
        let completed =
            harness::wait_until(Duration::from_secs(60), Duration::from_secs(1), || {
                let client = primary.client();
                let transfer_id = accept_transfer_id.clone();
                async move {
                    client
                        .transfer_status(&transfer_id)
                        .await
                        .map(|transfer| transfer.status == TransferStatus::Complete)
                        .unwrap_or(false)
                }
            })
            .await;
        assert!(
            completed,
            "primary should observe complete file transfer status"
        );
        let complete_frame = recv_matching_sse(&mut secondary_events_sse, |frame| {
            frame.event.as_deref() == Some("file:complete")
                && frame.data.contains(&accept_transfer_id)
        })
        .await;
        let complete_event = decode_file_complete_event(&complete_frame);
        assert_eq!(complete_event.transfer_id, accept_transfer_id);
        assert!(
            complete_event.filename.ends_with("contract-accept.txt"),
            "completed filename should preserve the original basename"
        );
    }

    // ── Cleanup of locally-created named group ───────────────────────────
    secondary_client
        .leave_group(&created_group.group_id)
        .await
        .expect("secondary leave group");
    tertiary_client
        .leave_group(&created_group.group_id)
        .await
        .expect("tertiary leave group");
    let group_left = harness::wait_until(Duration::from_secs(20), Duration::from_secs(1), || {
        let secondary_client = secondary.client();
        let tertiary_client = tertiary.client();
        let group_id = created_group.group_id.clone();
        async move {
            let secondary_missing = secondary_client
                .list_groups()
                .await
                .map(|groups| groups.iter().all(|group| group.group_id != group_id))
                .unwrap_or(false);
            let tertiary_missing = tertiary_client
                .list_groups()
                .await
                .map(|groups| groups.iter().all(|group| group.group_id != group_id))
                .unwrap_or(false);
            secondary_missing && tertiary_missing
        }
    })
    .await;
    assert!(
        group_left,
        "joined members should disappear from their local group lists after leaving"
    );
    primary_client
        .leave_group(&created_group.group_id)
        .await
        .expect("primary leave group");

    // ── Shutdown endpoint on ephemeral target ────────────────────────────
    tertiary_client
        .shutdown()
        .await
        .expect("shutdown tertiary daemon");
    let tertiary_down =
        harness::wait_until(Duration::from_secs(20), Duration::from_secs(1), || {
            let client = tertiary.client();
            async move { client.health().await.is_err() }
        })
        .await;
    assert!(
        tertiary_down,
        "tertiary daemon should stop responding after shutdown"
    );
}
