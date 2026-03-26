use base64::Engine as _;
use communitas_x0x_client::{WsInbound, X0xClient, X0xWebSocket};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, timeout};

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos()
}

#[tokio::test]
#[ignore = "requires a running x0xd on localhost"]
async fn websocket_subscribe_and_publish_follow_live_contract() {
    let client = X0xClient::new();
    let mut ws = X0xWebSocket::connect()
        .await
        .expect("ws://127.0.0.1:12700/ws should connect");

    let suffix = unique_suffix();
    let topic = format!("contract.ws.{suffix}");
    let payload = b"hello from websocket contract test";

    let connected = timeout(Duration::from_secs(5), ws.recv())
        .await
        .expect("connected frame should arrive in time")
        .expect("websocket should remain open before first frame");
    match connected {
        WsInbound::Connected { .. } => {}
        other => panic!("expected connected frame, got {other:?}"),
    }

    ws.subscribe(vec![topic.clone()])
        .expect("subscribe frame should send");

    loop {
        let inbound = timeout(Duration::from_secs(5), ws.recv())
            .await
            .expect("expected subscribed frame in time")
            .expect("websocket should remain open while subscribing");
        match inbound {
            WsInbound::Subscribed { topics } if topics.contains(&topic) => break,
            WsInbound::Connected { .. } => continue,
            _ => continue,
        }
    }

    client
        .publish(&topic, payload)
        .await
        .expect("rest publish should succeed");

    loop {
        let inbound = timeout(Duration::from_secs(5), ws.recv())
            .await
            .expect("expected published message in time")
            .expect("websocket should remain open while waiting for message");
        match inbound {
            WsInbound::Message {
                topic: inbound_topic,
                payload: inbound_payload,
                ..
            } if inbound_topic == topic => {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(inbound_payload)
                    .expect("message payload should decode from base64");
                assert_eq!(decoded, payload);
                break;
            }
            _ => continue,
        }
    }
}
