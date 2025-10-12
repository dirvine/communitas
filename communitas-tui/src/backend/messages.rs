use super::Backend;
use anyhow::Result;
use communitas_core::crdt::{CRDTMessage, EntityType, MessageContent};

impl Backend {
    /// Get messages for an entity (contact, group, channel, etc.)
    pub async fn get_entity_messages(&mut self, entity_id: String) -> Result<Vec<CRDTMessage>> {
        let ctx = self.context_mut()?;

        // Get all messages from message sync service
        let sync_response = ctx
            .message_sync
            .get_all_messages(&entity_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get messages: {}", e))?;

        Ok(sync_response.messages)
    }

    /// Send message to an entity (direct message to contact, or group message)
    pub async fn send_message(
        &mut self,
        entity_id: String,
        entity_type: EntityType,
        text: String,
    ) -> Result<String> {
        let ctx = self.context_mut()?;

        // Create message content with author info
        let content = MessageContent {
            text,
            author: ctx.display_name.clone(),
            attachments: None,
        };

        // Send message via message sync service
        let message = ctx
            .message_sync
            .send_message(entity_id, entity_type, content, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;

        Ok(message.metadata.id)
    }

    /// Send reply to a message (threaded reply)
    pub async fn send_reply(
        &mut self,
        entity_id: String,
        entity_type: EntityType,
        reply_to_id: String,
        text: String,
    ) -> Result<String> {
        let ctx = self.context_mut()?;

        // Create message content with author info
        let content = MessageContent {
            text,
            author: ctx.display_name.clone(),
            attachments: None,
        };

        // Send message with reply_to_id
        let message = ctx
            .message_sync
            .send_message(entity_id, entity_type, content, Some(reply_to_id))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send reply: {}", e))?;

        Ok(message.metadata.id)
    }

    /// Get messages for a specific thread (all replies to a message)
    pub async fn get_thread_messages(
        &mut self,
        entity_id: String,
        parent_message_id: String,
    ) -> Result<Vec<CRDTMessage>> {
        // Get all messages for entity
        let all_messages = self.get_entity_messages(entity_id).await?;

        // Filter to just replies to the parent message
        let thread_messages: Vec<CRDTMessage> = all_messages
            .into_iter()
            .filter(|msg| {
                msg.metadata
                    .reply_to_id
                    .as_ref()
                    .map(|id| id == &parent_message_id)
                    .unwrap_or(false)
            })
            .collect();

        Ok(thread_messages)
    }

    // ========================================================================
    // Compatibility methods for existing handlers
    // ========================================================================

    /// Get channel messages (compatibility method)
    pub async fn get_channel_messages(&mut self, channel_id: String) -> Result<Vec<CRDTMessage>> {
        self.get_entity_messages(channel_id).await
    }

    /// Send message to channel (compatibility method)
    pub async fn send_message_to_channel(
        &mut self,
        channel_id: String,
        text: String,
    ) -> Result<String> {
        self.send_message(channel_id, EntityType::Channel, text)
            .await
    }

    /// Send thread reply (compatibility method)
    pub async fn send_thread_reply(
        &mut self,
        channel_id: String,
        thread_id: String,
        text: String,
    ) -> Result<String> {
        self.send_reply(channel_id, EntityType::Channel, thread_id, text)
            .await
    }
}
