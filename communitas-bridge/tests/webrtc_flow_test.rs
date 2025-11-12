#[cfg(test)]
mod tests {
    use communitas_bridge::{CallState, WebRtcBridge};
    use tokio::test;

    #[test]
    async fn test_call_flow_integration() {
        let bridge = WebRtcBridge::new();
        let call_id = bridge
            .initiate_call("peer1", Default::default())
            .await
            .unwrap();
        assert_eq!(bridge.get_call_state(&call_id), CallState::Initiating);

        bridge.accept_call(call_id.clone()).await.unwrap();
        assert_eq!(bridge.get_call_state(&call_id), CallState::Active);

        bridge.end_call(call_id).await.unwrap();
        assert_eq!(bridge.get_call_state(&call_id), CallState::Ended);
        assert!(bridge.active_calls().is_empty());
    }
}
