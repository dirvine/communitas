use communitas_x0x_client::{TrustLevel, X0xClient};

#[tokio::test]
#[ignore = "requires a running x0xd on localhost"]
async fn contacts_and_direct_connections_match_live_contract() {
    let client = X0xClient::new();

    let contacts = client
        .list_contacts()
        .await
        .expect("contacts should decode");
    for contact in &contacts {
        match contact.trust_level {
            TrustLevel::Blocked | TrustLevel::Unknown | TrustLevel::Known | TrustLevel::Trusted => {
            }
        }
    }

    let direct = client
        .direct_connections()
        .await
        .expect("direct connections should decode");
    for connection in &direct {
        assert!(
            !connection.agent_id.is_empty(),
            "agent_id should be present"
        );
        if let Some(machine_id) = &connection.machine_id {
            assert!(
                !machine_id.is_empty(),
                "machine_id should not be empty when present"
            );
        }
    }

    let presence = client.presence().await.expect("presence should decode");
    for agent_id in &presence {
        assert!(
            !agent_id.is_empty(),
            "presence entries should be non-empty agent ids"
        );
    }

    let peers = client.peers().await.expect("peers should decode");
    for peer in &peers {
        assert!(!peer.id.is_empty(), "peer ids should be non-empty");
    }

    let agent = client.agent().await.expect("agent should decode");
    let evaluation = client
        .evaluate_trust(&agent.agent_id, &agent.machine_id)
        .await
        .expect("trust/evaluate should decode");
    assert!(
        !evaluation.decision.is_empty(),
        "trust decision should be present"
    );
}
