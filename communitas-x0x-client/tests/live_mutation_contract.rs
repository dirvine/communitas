// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "support/harness.rs"]
mod harness;

use base64::Engine as _;
use communitas_x0x_client::{TransferStatus, TrustLevel, WsInbound};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::time::timeout;

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
    let all_local =
        primary.kind.as_deref() == Some("local") && secondary.kind.as_deref() == Some("local");
    let run_direct_file = !all_local && harness::direct_file_enabled();

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
        let first_frame = timeout(Duration::from_secs(5), secondary_direct_ws.recv())
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
                "direct connection list did not become ready before send; trying delivery anyway"
            );
        }

        let direct_text = format!("contract-direct-{}", harness::unique_suffix());
        primary_client
            .send_direct(&secondary_agent.agent_id, direct_text.as_bytes())
            .await
            .expect("send direct message");

        let direct_payload = loop {
            match timeout(Duration::from_secs(20), secondary_direct_ws.recv())
                .await
                .expect("direct message timeout")
                .expect("direct websocket should stay open")
            {
                WsInbound::DirectMessage {
                    sender, payload, ..
                } if sender == primary_agent.agent_id => {
                    break payload;
                }
                WsInbound::Connected { .. } => continue,
                other => panic!("expected direct message from primary, got {other:?}"),
            }
        };
        let decoded_direct = base64::engine::general_purpose::STANDARD
            .decode(direct_payload)
            .expect("direct payload should decode");
        assert_eq!(decoded_direct, direct_text.as_bytes());

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

    // ── Named groups ─────────────────────────────────────────────────────
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
        .expect("join group via invite");
    assert_eq!(joined_group.group_id, created_group.group_id);

    secondary_client
        .set_group_display_name(&created_group.group_id, "secondary-renamed")
        .await
        .expect("set group display name");
    let group_info = primary_client
        .get_group(&created_group.group_id)
        .await
        .expect("get group info");
    assert_eq!(group_info.group_id, created_group.group_id);
    assert!(!group_info.name.is_empty());

    // ── KV store lifecycle ───────────────────────────────────────────────
    let store_name = format!("contract-store-{}", harness::unique_suffix());
    let store_topic = format!("contract.store.{}", harness::unique_suffix());
    let created_store = primary_client
        .create_store(&store_name, &store_topic)
        .await
        .expect("create store");
    secondary_client
        .join_store(&created_store.id)
        .await
        .expect("join store");

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
    let keys = primary_client
        .list_keys(&created_store.id)
        .await
        .expect("list store keys");
    assert!(keys.iter().any(|entry| entry.key == "hello"));
    let value = primary_client
        .get(&created_store.id, "hello")
        .await
        .expect("get store value");
    let decoded_store = base64::engine::general_purpose::STANDARD
        .decode(value.value)
        .expect("store value should decode");
    assert_eq!(decoded_store, store_payload);
    primary_client
        .delete_key(&created_store.id, "hello")
        .await
        .expect("delete store key");

    // ── Task lists ───────────────────────────────────────────────────────
    let task_topic = format!("contract.tasks.{}", harness::unique_suffix());
    let created_list = primary_client
        .create_task_list("contract-harness", &task_topic)
        .await
        .expect("create task list");
    let task_lists = primary_client
        .list_task_lists()
        .await
        .expect("list task lists");
    assert!(task_lists.iter().any(|list| list.id == created_list.id));

    primary_client
        .add_task(
            &created_list.id,
            "ship comprehensive harness",
            Some("contract test task"),
        )
        .await
        .expect("add task");
    let tasks = primary_client
        .list_tasks(&created_list.id)
        .await
        .expect("list tasks after add");
    let task = tasks
        .iter()
        .find(|task| task.title == "ship comprehensive harness")
        .expect("added task should exist")
        .clone();
    primary_client
        .claim_task(&created_list.id, &task.id)
        .await
        .expect("claim task");
    primary_client
        .complete_task(&created_list.id, &task.id)
        .await
        .expect("complete task");
    let completed_tasks = primary_client
        .list_tasks(&created_list.id)
        .await
        .expect("list tasks after complete");
    assert!(completed_tasks.iter().any(|entry| entry.id == task.id));

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

    let welcome = primary_client
        .create_mls_welcome(&mls_group.group_id, &secondary_agent.agent_id)
        .await
        .expect("create mls welcome");
    assert_eq!(welcome.group_id, mls_group.group_id);
    assert!(!welcome.welcome.is_empty());

    let add_member = primary_client
        .add_mls_member(&mls_group.group_id, &secondary_agent.agent_id)
        .await
        .expect("add mls member");
    assert!(add_member.member_count >= 1);
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
        .remove_mls_member(&mls_group.group_id, &secondary_agent.agent_id)
        .await
        .expect("remove mls member");

    // ── File transfers ───────────────────────────────────────────────────
    if !run_direct_file {
        eprintln!(
            "Skipping file-transfer mutation checks. Enable X0X_TEST_ENABLE_DIRECT_FILE=1 to exercise direct/file flows once the target topology supports them reliably."
        );
    } else {
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
            harness::wait_until(Duration::from_secs(20), Duration::from_secs(1), || {
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
            harness::wait_until(Duration::from_secs(30), Duration::from_secs(1), || {
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
    }

    // ── Cleanup of locally-created named group ───────────────────────────
    secondary_client
        .leave_group(&created_group.group_id)
        .await
        .expect("secondary leave group");
    primary_client
        .leave_group(&created_group.group_id)
        .await
        .expect("primary leave group");
}
