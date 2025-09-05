use communitas_tauri::test_harness::*;
use saorsa_core::dht::*;

#[tokio::test]
async fn local_dht_put_get_send() {
    let dht = LocalDht::new("node-a".to_string());
    dht.add_peer("node-b".to_string()).await;

    // put/get
    dht.put(b"k".to_vec(), b"v".to_vec()).await.expect("put ok");
    let v = dht.get(b"k".to_vec()).await.expect("get ok");
    assert_eq!(v, Some(b"v".to_vec()));

    // send
    let resp = dht
        .send("node-b".to_string(), "topic".into(), b"payload".to_vec())
        .await
        .expect("send ok");
    assert!(resp.is_empty());

    // drain inbox (test helper) to assert message arrived
    let inbox = dht.drain_inbox("node-b").await;
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0], b"payload".to_vec());
}
