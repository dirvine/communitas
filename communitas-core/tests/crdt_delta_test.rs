#[cfg(test)]
mod tests {
    use yrs::{Doc, encode_state_v1, decode_state_v1};
    use communitas_core::crdt::sync::{sync_document, CrdtManager}; // Assume manager exposes sync
    use std::sync::Arc;

    #[tokio::test]
    async fn test_delta_sync_smaller_than_full() {
        let manager = CrdtManager::new_mock(); // Mock for isolation
        let doc = Arc::new(Doc::new());
        let initial_state = encode_state_v1(&doc); // Empty: small

        // Apply change
        let text = doc.get_text("shared");
        text.insert(0, "Initial text");

        // Sync (should use delta)
        sync_document(&manager, "doc1", &doc).await.unwrap();

        let full_state = encode_state_v1(&doc);
        let sent_update = manager.get_last_sent("doc1").unwrap(); // Mock getter

        // FAIL: Expect delta smaller
        assert!(sent_update.len() < full_state.len(), "Delta should be smaller than full state");
    }
}