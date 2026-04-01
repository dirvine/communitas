// SPDX-License-Identifier: MIT OR Apache-2.0

use base64::Engine as _;
use communitas_x0x_client::X0xClient;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos()
}

#[tokio::test]
#[ignore = "requires a running x0xd on localhost"]
async fn store_create_put_get_list_and_delete_follow_live_contract() {
    let client = X0xClient::new();
    let suffix = unique_suffix();
    let store_name = format!("contract-store-{suffix}");
    let topic = format!("contract.store.{suffix}");
    let key = "hello";
    let payload = b"hello from contract test";

    let created = client
        .create_store(&store_name, &topic)
        .await
        .expect("store should be created");

    let stores = client.list_stores().await.expect("stores should list");
    assert!(stores.iter().any(|store| store.id == created.id));

    client
        .put(&created.id, key, payload, Some("text/plain"))
        .await
        .expect("put should succeed");

    let keys = client
        .list_keys(&created.id)
        .await
        .expect("keys should list after put");
    assert!(keys.iter().any(|entry| entry.key == key));

    let value = client
        .get(&created.id, key)
        .await
        .expect("get should succeed");
    assert_eq!(value.key, key);
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(value.value)
            .expect("store value should be valid base64"),
        payload
    );
    assert_eq!(value.content_type.as_deref(), Some("text/plain"));

    client
        .delete_key(&created.id, key)
        .await
        .expect("delete should succeed");

    let keys_after_delete = client
        .list_keys(&created.id)
        .await
        .expect("keys should list after delete");
    assert!(!keys_after_delete.iter().any(|entry| entry.key == key));
}
