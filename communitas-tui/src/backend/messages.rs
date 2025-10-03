use super::Backend;
use anyhow::Result;
use saorsa_core::identity::FourWordAddress;
use saorsa_core::messaging::{ChannelId as MessagingChannelId, MessageContent};

impl Backend {
    /// Get messages from a channel
    pub async fn get_channel_messages(
        &mut self,
        channel_id: String,
    ) -> Result<Vec<saorsa_core::chat::Message>> {
        let ctx = self.context_mut()?;

        // Try to load messages from storage
        let storage_key = format!("chat:channel:{}:messages", channel_id);

        match ctx
            .storage
            .get_encrypted::<Vec<saorsa_core::chat::Message>>(&storage_key)
            .await
        {
            Ok(messages) => Ok(messages),
            Err(e) => {
                // Storage error or no messages yet - return empty vec
                tracing::debug!("Failed to load messages: {}", e);
                Ok(Vec::new())
            }
        }
    }
    /// Send message to channel (all members)
    pub async fn send_message_to_channel(
        &mut self,
        channel_id: String,
        text: String,
    ) -> Result<String> {
        let ctx = self.context_mut()?;

        // Get channel to find members
        let ch_id = saorsa_core::chat::ChannelId(channel_id.clone());
        let channel = ctx.chat.get_channel(&ch_id).await?;

        // Convert members to four-word addresses
        let mut recipients = Vec::new();
        for member in channel.members {
            // Try to resolve four-word address for member
            if let Ok(Some(addr)) = saorsa_core::get_user_four_words(&member.user_id).await {
                recipients.push(addr);
            } else if member.user_id.split('-').count() == 4 {
                // Assume it's already a four-word address
                recipients.push(FourWordAddress(member.user_id.to_lowercase()));
            }
        }

        // Fallback to self if no recipients found
        if recipients.is_empty() {
            recipients.push(FourWordAddress(ctx.four_words.clone()));
        }

        // Send message
        let channel_uuid = uuid::Uuid::parse_str(&channel_id)?;
        let (msg_id, _receipt) = ctx
            .messaging
            .send_message(
                recipients,
                MessageContent::Text(text),
                MessagingChannelId(channel_uuid),
                Default::default(),
            )
            .await?;

        Ok(msg_id.to_string())
    }

    /// Send message to specific recipients
    pub async fn send_message_to_recipients(
        &mut self,
        recipients: Vec<String>,
        channel_id: String,
        text: String,
    ) -> Result<String> {
        let ctx = self.context_mut()?;

        let recipients: Vec<FourWordAddress> =
            recipients.into_iter().map(FourWordAddress).collect();

        let channel_uuid = uuid::Uuid::parse_str(&channel_id)?;
        let (msg_id, _receipt) = ctx
            .messaging
            .send_message(
                recipients,
                MessageContent::Text(text),
                MessagingChannelId(channel_uuid),
                Default::default(),
            )
            .await?;

        Ok(msg_id.to_string())
    }

    /// Add reaction to message
    pub async fn add_reaction(
        &mut self,
        channel_id: String,
        message_id: String,
        emoji: String,
    ) -> Result<()> {
        let ctx = self.context_mut()?;

        let ch_id = saorsa_core::chat::ChannelId(channel_id);
        let msg_id = saorsa_core::chat::MessageId(message_id);

        ctx.chat.add_reaction(&ch_id, &msg_id, emoji).await?;

        Ok(())
    }

    /// Create a thread from a message
    pub async fn create_thread(
        &mut self,
        channel_id: String,
        parent_message_id: String,
    ) -> Result<String> {
        let ctx = self.context_mut()?;

        let ch_id = saorsa_core::chat::ChannelId(channel_id);
        let msg_id = saorsa_core::chat::MessageId(parent_message_id);

        let thread = ctx.chat.create_thread(&ch_id, &msg_id).await?;

        Ok(thread.id.0)
    }

    /// Send reply to a thread
    pub async fn send_thread_reply(
        &mut self,
        channel_id: String,
        thread_id: String,
        text: String,
    ) -> Result<String> {
        let ctx = self.context_mut()?;

        // Get channel to find members
        let ch_id = saorsa_core::chat::ChannelId(channel_id.clone());
        let channel = ctx.chat.get_channel(&ch_id).await?;

        // Convert members to four-word addresses
        let mut recipients = Vec::new();
        for member in channel.members {
            if let Ok(Some(addr)) = saorsa_core::get_user_four_words(&member.user_id).await {
                recipients.push(addr);
            } else if member.user_id.split('-').count() == 4 {
                recipients.push(FourWordAddress(member.user_id.to_lowercase()));
            }
        }

        if recipients.is_empty() {
            recipients.push(FourWordAddress(ctx.four_words.clone()));
        }

        // Send message with thread_id
        let channel_uuid = uuid::Uuid::parse_str(&channel_id)?;

        let (msg_id, _receipt) = ctx
            .messaging
            .send_message(
                recipients,
                MessageContent::Text(text),
                MessagingChannelId(channel_uuid),
                Default::default(),
            )
            .await?;

        // Store thread association
        let thread_key = format!("chat:thread:{}:messages", thread_id);
        let mut thread_messages: Vec<String> = ctx
            .storage
            .get_encrypted(&thread_key)
            .await
            .unwrap_or_default();
        thread_messages.push(msg_id.to_string());
        ctx.storage
            .store_encrypted(
                &thread_key,
                &thread_messages,
                std::time::Duration::from_secs(86400 * 30),
                None,
            )
            .await?;

        Ok(msg_id.to_string())
    }

    /// Get thread messages
    pub async fn get_thread_messages(
        &mut self,
        thread_id: String,
    ) -> Result<Vec<saorsa_core::chat::Message>> {
        let ctx = self.context_mut()?;

        let thread_key = format!("chat:thread:{}:messages", thread_id);

        match ctx.storage.get_encrypted::<Vec<String>>(&thread_key).await {
            Ok(message_ids) => {
                let mut messages = Vec::new();
                for msg_id in message_ids {
                    let msg_key = format!("chat:message:{}", msg_id);
                    if let Ok(msg) = ctx
                        .storage
                        .get_encrypted::<saorsa_core::chat::Message>(&msg_key)
                        .await
                    {
                        messages.push(msg);
                    }
                }
                Ok(messages)
            }
            Err(_) => Ok(Vec::new()),
        }
    }
}
