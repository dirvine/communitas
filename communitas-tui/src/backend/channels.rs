use super::Backend;
use anyhow::Result;
use saorsa_core::chat::{Channel, ChannelType};

impl Backend {
    /// Create a new channel
    pub async fn create_channel(&mut self, name: String, description: String) -> Result<Channel> {
        let ctx = self.context_mut()?;

        ctx.chat
            .create_channel(name, description, ChannelType::Public, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create channel: {}", e))
    }

    /// Get list of channels for current user
    pub async fn get_channels(&mut self) -> Result<Vec<Channel>> {
        let ctx = self.context_mut()?;

        let channel_ids = ctx
            .chat
            .get_user_channels()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get channel list: {}", e))?;

        let mut channels = Vec::new();
        for id in channel_ids {
            let channel = ctx
                .chat
                .get_channel(&id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get channel: {}", e))?;
            channels.push(channel);
        }

        Ok(channels)
    }

    /// Get channel by ID
    pub async fn get_channel(&mut self, channel_id: &str) -> Result<Channel> {
        let ctx = self.context_mut()?;

        let id = saorsa_core::chat::ChannelId(channel_id.to_string());

        ctx.chat
            .get_channel(&id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get channel: {}", e))
    }
}
