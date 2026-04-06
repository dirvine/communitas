// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "support/harness.rs"]
mod harness;

use communitas_x0x_client::{WsInbound, X0xError};
use tokio::time::{Duration, timeout};

#[tokio::test]
#[ignore = "requires live x0x targets via X0X_TEST_MATRIX_FILE or local discovery"]
async fn all_targets_expose_core_daemon_surfaces() {
    let targets = harness::load_targets();

    for target in targets {
        let client = target.client();

        let health = client
            .health()
            .await
            .unwrap_or_else(|err| panic!("{} health failed: {err}", target.summary()));
        assert!(
            !health.status.is_empty(),
            "{} health.status",
            target.summary()
        );
        assert!(
            !health.version.is_empty(),
            "{} health.version",
            target.summary()
        );

        let status = client
            .status()
            .await
            .unwrap_or_else(|err| panic!("{} status failed: {err}", target.summary()));
        assert!(
            !status.status.is_empty(),
            "{} status.status",
            target.summary()
        );
        assert!(
            !status.version.is_empty(),
            "{} status.version",
            target.summary()
        );
        assert!(
            !status.api_address.is_empty(),
            "{} status.api_address",
            target.summary()
        );

        let agent = client
            .agent()
            .await
            .unwrap_or_else(|err| panic!("{} agent failed: {err}", target.summary()));
        assert!(
            !agent.agent_id.is_empty(),
            "{} agent.agent_id",
            target.summary()
        );
        assert!(
            !agent.machine_id.is_empty(),
            "{} agent.machine_id",
            target.summary()
        );

        let user_id = client
            .agent_user_id()
            .await
            .unwrap_or_else(|err| panic!("{} agent_user_id failed: {err}", target.summary()));
        assert_eq!(
            user_id,
            agent.user_id,
            "{} agent and agent_user_id should agree",
            target.summary()
        );

        let card = client
            .agent_card(Some("ContractHarness"), Some(true))
            .await
            .unwrap_or_else(|err| panic!("{} agent_card failed: {err}", target.summary()));
        assert_eq!(
            card.card.agent_id,
            agent.agent_id,
            "{} card agent id",
            target.summary()
        );
        assert_eq!(
            card.card.machine_id,
            agent.machine_id,
            "{} card machine id",
            target.summary()
        );
        assert!(!card.link.is_empty(), "{} card link", target.summary());

        let _ = client
            .announce()
            .await
            .unwrap_or_else(|err| panic!("{} announce failed: {err}", target.summary()));
        if target.kind.as_deref() != Some("remote") {
            let _ = client
                .announce_with_options(false, false)
                .await
                .unwrap_or_else(|err| {
                    panic!("{} announce_with_options failed: {err}", target.summary())
                });
        }

        let _ = client
            .peers()
            .await
            .unwrap_or_else(|err| panic!("{} peers failed: {err}", target.summary()));
        let _ = client
            .discovered_agents()
            .await
            .unwrap_or_else(|err| panic!("{} discovered_agents failed: {err}", target.summary()));
        let _ = client
            .presence()
            .await
            .unwrap_or_else(|err| panic!("{} presence failed: {err}", target.summary()));
        let _ = client
            .presence_online()
            .await
            .unwrap_or_else(|err| panic!("{} presence_online failed: {err}", target.summary()));
        if target.kind.as_deref() != Some("remote") {
            let _ = client
                .presence_foaf(Some(2), Some(2_000))
                .await
                .unwrap_or_else(|err| panic!("{} presence_foaf failed: {err}", target.summary()));
        }

        let status_self = client
            .presence_status(&agent.agent_id)
            .await
            .unwrap_or_else(|err| panic!("{} presence_status failed: {err}", target.summary()));
        if let Some(found_agent) = status_self.agent {
            assert_eq!(found_agent.agent_id, agent.agent_id);
        }

        let network = client
            .network_status()
            .await
            .unwrap_or_else(|err| panic!("{} network_status failed: {err}", target.summary()));
        assert!(network.connected_peers <= 10_000);

        let cache = client
            .bootstrap_cache()
            .await
            .unwrap_or_else(|err| panic!("{} bootstrap_cache failed: {err}", target.summary()));
        assert_eq!(cache.connected_peers.len() as u32, cache.connection_count);

        let sessions = client
            .ws_sessions()
            .await
            .unwrap_or_else(|err| panic!("{} ws_sessions failed: {err}", target.summary()));
        assert!(sessions.shared_subscriptions.len() <= 10_000);

        let constitution = client
            .constitution()
            .await
            .unwrap_or_else(|err| panic!("{} constitution failed: {err}", target.summary()));
        assert!(constitution.contains("Law") || constitution.contains("Constitution"));

        let constitution_json = client
            .constitution_json()
            .await
            .unwrap_or_else(|err| panic!("{} constitution_json failed: {err}", target.summary()));
        assert!(!constitution_json.version.is_empty());
        assert!(!constitution_json.status.is_empty());
        assert!(!constitution_json.content.is_empty());

        if target.kind.as_deref() != Some("remote") {
            match client.check_upgrade().await {
                Ok(upgrade) => {
                    assert!(
                        upgrade.is_object(),
                        "{} upgrade response shape",
                        target.summary()
                    );
                }
                Err(X0xError::Daemon(message)) => {
                    assert!(
                        message.contains("upgrade") || message.contains("update"),
                        "{} unexpected upgrade error: {}",
                        target.summary(),
                        message
                    );
                }
                Err(err) => panic!("{} check_upgrade failed: {err}", target.summary()),
            }
        }
    }
}

#[tokio::test]
#[ignore = "requires at least two live x0x targets"]
async fn multi_target_discovery_and_websocket_contract_hold() {
    let targets = harness::load_targets();
    if targets.len() < 2 {
        eprintln!("Skipping multi-target discovery test: need at least 2 targets");
        return;
    }

    if !matches!(
        std::env::var("X0X_TEST_ENABLE_MULTI_TARGET").as_deref(),
        Ok("1" | "true" | "yes")
    ) {
        eprintln!(
            "Skipping multi-target discovery test: X0X_TEST_ENABLE_MULTI_TARGET is not enabled"
        );
        return;
    }

    let primary = targets[0].clone();
    let secondary = targets[1].clone();
    if primary.kind.as_deref() == Some("local") && secondary.kind.as_deref() == Some("local") {
        eprintln!(
            "Skipping multi-target discovery test for local scratch daemons; local harness covers stateful flows via live_mutation_contract"
        );
        return;
    }
    let primary_client = primary.client();
    let secondary_client = secondary.client();

    let primary_agent = primary_client.agent().await.expect("primary agent");
    let secondary_agent = secondary_client.agent().await.expect("secondary agent");

    primary_client.announce().await.expect("primary announce");
    secondary_client
        .announce()
        .await
        .expect("secondary announce");

    let discovered = harness::wait_until(Duration::from_secs(60), Duration::from_secs(2), || {
        let client = primary.client();
        let target_agent = secondary_agent.agent_id.clone();
        async move {
            client
                .discovered_agents()
                .await
                .map(|agents| agents.iter().any(|agent| agent.agent_id == target_agent))
                .unwrap_or(false)
        }
    })
    .await;
    assert!(
        discovered,
        "{} should discover {} within 60s",
        primary.summary(),
        secondary.summary()
    );

    let discovered_agent = primary_client
        .discovered_agent(&secondary_agent.agent_id)
        .await
        .expect("discovered_agent query should decode");
    assert_eq!(discovered_agent.agent_id, secondary_agent.agent_id);

    let presence_found = primary_client
        .presence_find(&secondary_agent.agent_id, Some(3), Some(5_000))
        .await
        .expect("presence_find should decode");
    assert!(
        presence_found.is_some(),
        "presence_find should locate secondary agent"
    );

    let active_find = primary_client
        .find_agent(&secondary_agent.agent_id)
        .await
        .expect("find_agent should decode");
    assert!(active_find.found, "find_agent should report found=true");

    let reachability = primary_client
        .agent_reachability(&secondary_agent.agent_id)
        .await
        .expect("agent_reachability should decode");
    assert!(
        !reachability.addresses.is_empty()
            || reachability.likely_direct
            || reachability.needs_coordination
    );

    if let Some(user_id) = secondary_agent.user_id.clone() {
        let agents = primary_client
            .user_agents(&user_id)
            .await
            .expect("user_agents should decode when user_id exists");
        assert!(
            agents
                .iter()
                .any(|agent| agent.agent_id == secondary_agent.agent_id)
        );
    }

    let mut ws = primary.ws().await.expect("general websocket connect");
    match timeout(Duration::from_secs(5), ws.recv())
        .await
        .expect("connected frame should arrive in time")
        .expect("websocket should remain open")
    {
        WsInbound::Connected { agent_id, .. } => assert_eq!(agent_id, primary_agent.agent_id),
        other => panic!("expected connected frame, got {other:?}"),
    }

    ws.ping().expect("ping frame should send");
    let mut received_pong = false;
    for _ in 0..3 {
        match timeout(Duration::from_secs(5), ws.recv())
            .await
            .expect("websocket frame should arrive")
            .expect("websocket should stay open")
        {
            WsInbound::Pong => {
                received_pong = true;
                break;
            }
            WsInbound::Connected { .. } => continue,
            other => panic!("expected pong or connected frame, got {other:?}"),
        }
    }
    assert!(received_pong, "ping should produce a pong");
}
