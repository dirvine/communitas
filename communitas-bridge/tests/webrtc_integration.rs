#[cfg(test)]
mod tests {
    use communitas_bridge::webrtc::WebRtcBridge;
    use tokio::test;

    #[test]
    async fn test_bridge_call_flow() {
        let bridge = WebRtcBridge::new();
        let call_id = bridge.initiate_call("test-peer", Default::default()).await.unwrap();
        bridge.accept_call(call_id.clone()).await.unwrap();
        assert!(bridge.get_call(&call_id).is_some());
        bridge.end_call(call_id).await.unwrap();
    }
}