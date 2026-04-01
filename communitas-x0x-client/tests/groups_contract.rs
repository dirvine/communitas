// SPDX-License-Identifier: MIT OR Apache-2.0

use communitas_x0x_client::X0xClient;

#[tokio::test]
#[ignore = "requires a running x0xd on localhost"]
async fn group_list_and_detail_match_live_contract() {
    let client = X0xClient::new();

    let groups = client.list_groups().await.expect("groups should decode");
    assert!(
        !groups.is_empty(),
        "expected at least one group in live daemon"
    );

    let group = client
        .get_group(&groups[0].group_id)
        .await
        .expect("group detail should decode");

    assert_eq!(group.group_id, groups[0].group_id);
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

    let welcome = client
        .create_mls_welcome(
            &groups[0].group_id,
            &client.agent().await.expect("agent").agent_id,
        )
        .await
        .expect("mls welcome should decode");
    assert_eq!(welcome.group_id, groups[0].group_id);
    assert!(
        !welcome.welcome.is_empty(),
        "welcome payload should be present"
    );
}
