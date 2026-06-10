// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use communitas_x0x_client::X0xClient;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
#[ignore = "requires a running x0xd on localhost"]
async fn group_list_and_detail_match_live_contract() -> Result<(), Box<dyn std::error::Error>> {
    let client = X0xClient::new();

    let groups = client.list_groups().await?;
    let group_id = if let Some(group) = groups.first() {
        group.group_id.clone()
    } else {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        client
            .create_group(
                &format!("communitas-live-contract-{suffix}"),
                Some("created by the Communitas x0x live contract test"),
                Some("Communitas Live Contract"),
            )
            .await?
            .group_id
    };

    let group = client.get_group(&group_id).await?;

    assert_eq!(group.group_id, group_id);
    assert!(!group.name.is_empty(), "group name should be present");
    assert!(
        group
            .chat_topic
            .as_deref()
            .is_some_and(|topic| topic.contains(".chat/")),
        "group detail should expose a chat topic"
    );
    assert!(
        group
            .metadata_topic
            .as_deref()
            .is_some_and(|topic| topic.contains(".meta")),
        "group detail should expose a metadata topic"
    );
    assert!(group.members.len() <= 10_000, "member list should be sane");

    let agent = client.agent().await?;
    let welcome = client
        .create_mls_welcome(&group_id, &agent.agent_id)
        .await?;
    assert_eq!(welcome.group_id, group_id);
    assert!(
        !welcome.welcome.is_empty(),
        "welcome payload should be present"
    );
    Ok(())
}
