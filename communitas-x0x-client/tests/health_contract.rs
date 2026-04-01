// SPDX-License-Identifier: MIT OR Apache-2.0

use communitas_x0x_client::X0xClient;

#[tokio::test]
#[ignore = "requires a running x0xd on localhost"]
async fn health_status_and_agent_match_live_daemon_contract() {
    let client = X0xClient::new();

    let health = client.health().await.expect("health should decode");
    assert!(!health.status.is_empty(), "health.status should be present");
    assert!(
        !health.version.is_empty(),
        "health.version should be present"
    );

    let status = client.status().await.expect("status should decode");
    assert!(!status.status.is_empty(), "status.status should be present");
    assert!(
        !status.version.is_empty(),
        "status.version should be present"
    );
    assert!(
        !status.api_address.is_empty(),
        "status.api_address should be present"
    );
    assert!(
        !status.agent_id.is_empty(),
        "status.agent_id should be present"
    );

    let agent = client.agent().await.expect("agent should decode");
    assert!(
        !agent.agent_id.is_empty(),
        "agent.agent_id should be present"
    );
    assert!(
        !agent.machine_id.is_empty(),
        "agent.machine_id should be present"
    );

    let user_id = client
        .agent_user_id()
        .await
        .expect("agent/user-id should decode");
    assert_eq!(
        user_id, agent.user_id,
        "agent and user-id endpoint should agree"
    );

    let network = client
        .network_status()
        .await
        .expect("network/status should decode");
    assert!(
        network.connected_peers <= 10_000,
        "connected peer count should be sane"
    );

    let cache = client
        .bootstrap_cache()
        .await
        .expect("network/bootstrap-cache should decode");
    assert_eq!(cache.connected_peers.len() as u32, cache.connection_count);

    let card = client
        .agent_card(Some("ContractCheck"), Some(true))
        .await
        .expect("agent/card should decode");
    assert_eq!(card.card.agent_id, agent.agent_id);
    assert_eq!(card.card.machine_id, agent.machine_id);
    assert!(!card.link.is_empty(), "agent card link should be present");

    let sessions = client
        .ws_sessions()
        .await
        .expect("ws/sessions should decode");
    assert!(sessions.shared_subscriptions.len() <= 10_000);
}
