// Tauri commands for rich messaging integration
use saorsa_core::messaging::{
    MessagingService, SendMessageRequest, RichMessage, MessageContent, 
    ChannelId, MessageId, ThreadId, MarkdownContent, Attachment,
    SearchQuery, DateRange, ReactionManager, ThreadManager,
    MessageComposer, MessageSearch, MediaProcessor, RealtimeSync,
    SecureMessaging, DhtClient, MessageStore,
};
use saorsa_core::identity::FourWordAddress;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Application state for messaging
pub struct MessagingState {
    pub service: Arc<RwLock<MessagingService>>,
    pub channels: Arc<RwLock<HashMap<String, ChannelInfo>>>,
    pub presence: Arc<RwLock<HashMap<String, UserPresence>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub avatar: Option<String>,
    pub last_message: Option<LastMessage>,
    pub unread_count: u32,
    pub members: Vec<String>,
    pub is_online: bool,
    pub is_muted: bool,
    pub is_pinned: bool,
    pub four_word_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastMessage {
    pub content: String,
    pub timestamp: String,
    pub sender: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub status: String, // "online", "away", "busy", "offline"
    pub last_seen: DateTime<Utc>,
    pub activity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRequest {
    pub channel_id: String,
    pub content: MessageContentDto,
    pub attachments: Vec<AttachmentDto>,
    pub thread_id: Option<String>,
    pub reply_to: Option<String>,
    pub mentions: Vec<String>,
    pub ephemeral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentDto {
    pub text: Option<String>,
    pub markdown: Option<String>,
    pub mentions: Vec<String>,
    pub links: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentDto {
    pub id: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub name: String,
    pub size: usize,
    pub url: String,
    pub thumbnail: Option<String>,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichMessageDto {
    pub id: String,
    pub sender: SenderInfo,
    pub channel_id: String,
    pub content: MessageContentDto,
    pub attachments: Vec<AttachmentDto>,
    pub thread_id: Option<String>,
    pub reply_to: Option<ReplyInfo>,
    pub reactions: Vec<ReactionInfo>,
    pub timestamp: String,
    pub edited_at: Option<String>,
    pub deleted_at: Option<String>,
    pub read_by: Vec<String>,
    pub status: String,
    pub encrypted: bool,
    pub ephemeral: bool,
    pub pinned: bool,
    pub starred: bool,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub four_word_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyInfo {
    pub id: String,
    pub sender: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionInfo {
    pub emoji: String,
    pub users: Vec<String>,
}

/// Initialize the unified messaging system
#[tauri::command]
pub async fn initialize_unified_messaging(
    user_id: String,
    four_word_address: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<(), String> {
    let mut state_guard = state.write().await;
    
    // Create identity from four-word address
    let identity = FourWordAddress::from_str(&four_word_address)
        .map_err(|e| format!("Invalid four-word address: {}", e))?;
    
    // Initialize messaging service with real DHT
    let service = MessagingService::new(identity)
        .await
        .map_err(|e| format!("Failed to initialize messaging: {}", e))?;
    
    // Create initial state
    let messaging_state = MessagingState {
        service: Arc::new(RwLock::new(service)),
        channels: Arc::new(RwLock::new(HashMap::new())),
        presence: Arc::new(RwLock::new(HashMap::new())),
    };
    
    *state_guard = Some(messaging_state);
    
    Ok(())
}

/// Get all channels for a user
#[tauri::command]
pub async fn get_channels(
    user_id: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<Vec<ChannelInfo>, String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let channels = messaging_state.channels.read().await;
    
    // Return mock channels for now
    if channels.is_empty() {
        // Create default channels
        let default_channels = vec![
            ChannelInfo {
                id: "general".to_string(),
                name: "General".to_string(),
                channel_type: "public".to_string(),
                avatar: None,
                last_message: Some(LastMessage {
                    content: "Welcome to the P2P network!".to_string(),
                    timestamp: Utc::now().to_rfc3339(),
                    sender: "System".to_string(),
                }),
                unread_count: 0,
                members: vec!["ocean-forest-moon-star".to_string()],
                is_online: true,
                is_muted: false,
                is_pinned: true,
                four_word_address: None,
            },
            ChannelInfo {
                id: "tech-discussion".to_string(),
                name: "Tech Discussion".to_string(),
                channel_type: "group".to_string(),
                avatar: None,
                last_message: None,
                unread_count: 2,
                members: vec![
                    "ocean-forest-moon-star".to_string(),
                    "river-mountain-sun-cloud".to_string(),
                ],
                is_online: true,
                is_muted: false,
                is_pinned: false,
                four_word_address: None,
            },
        ];
        
        Ok(default_channels)
    } else {
        Ok(channels.values().cloned().collect())
    }
}

/// Create a new channel
#[tauri::command]
pub async fn create_channel(
    request: CreateChannelRequest,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<ChannelInfo, String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let channel_id = format!("channel-{}", uuid::Uuid::new_v4());
    
    let channel = ChannelInfo {
        id: channel_id.clone(),
        name: request.name,
        channel_type: request.channel_type,
        avatar: None,
        last_message: None,
        unread_count: 0,
        members: request.members,
        is_online: true,
        is_muted: false,
        is_pinned: false,
        four_word_address: None,
    };
    
    let mut channels = messaging_state.channels.write().await;
    channels.insert(channel_id, channel.clone());
    
    Ok(channel)
}

/// Get messages for a channel
#[tauri::command]
pub async fn get_channel_messages(
    channel_id: String,
    limit: usize,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<Vec<RichMessageDto>, String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let service = messaging_state.service.read().await;
    
    let channel_id = ChannelId::from(channel_id.as_str());
    let messages = service.get_channel_messages(channel_id, limit, None)
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;
    
    // Convert to DTOs
    let message_dtos: Vec<RichMessageDto> = messages.into_iter()
        .map(|msg| convert_message_to_dto(msg))
        .collect();
    
    Ok(message_dtos)
}

/// Send a rich message
#[tauri::command]
pub async fn send_rich_message(
    message: MessageRequest,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
    app_handle: tauri::AppHandle,
) -> Result<RichMessageDto, String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let mut service = messaging_state.service.write().await;
    
    // Convert DTO to domain model
    let content = if let Some(markdown) = message.content.markdown {
        MessageContent::RichText(MarkdownContent {
            raw: message.content.text.unwrap_or_default(),
            formatted: markdown,
            mentions: message.content.mentions.iter()
                .map(|m| FourWordAddress::from(m.as_str()))
                .collect(),
            links: message.content.links,
        })
    } else {
        MessageContent::Text(message.content.text.unwrap_or_default())
    };
    
    let request = SendMessageRequest {
        channel_id: ChannelId::from(message.channel_id.as_str()),
        content,
        attachments: vec![], // TODO: Handle attachments
        thread_id: message.thread_id.map(|id| ThreadId::from(id.as_str())),
        reply_to: message.reply_to.map(|id| MessageId::from(id.as_str())),
        mentions: message.mentions.iter()
            .map(|m| FourWordAddress::from(m.as_str()))
            .collect(),
        ephemeral: message.ephemeral,
    };
    
    let sent_message = service.send_message(request)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))?;
    
    let message_dto = convert_message_to_dto(sent_message);
    
    // Emit event for real-time updates
    app_handle.emit_all("new-message", &message_dto)
        .map_err(|e| format!("Failed to emit event: {}", e))?;
    
    Ok(message_dto)
}

/// Add a reaction to a message
#[tauri::command]
pub async fn add_reaction(
    message_id: String,
    emoji: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let mut service = messaging_state.service.write().await;
    
    service.add_reaction(MessageId::from(message_id.as_str()), emoji)
        .await
        .map_err(|e| format!("Failed to add reaction: {}", e))?;
    
    Ok(())
}

/// Edit a message
#[tauri::command]
pub async fn edit_message(
    message_id: String,
    new_content: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let mut service = messaging_state.service.write().await;
    
    service.edit_message(
        MessageId::from(message_id.as_str()),
        MessageContent::Text(new_content),
    )
    .await
    .map_err(|e| format!("Failed to edit message: {}", e))?;
    
    Ok(())
}

/// Delete a message
#[tauri::command]
pub async fn delete_message(
    message_id: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let mut service = messaging_state.service.write().await;
    
    service.delete_message(MessageId::from(message_id.as_str()))
        .await
        .map_err(|e| format!("Failed to delete message: {}", e))?;
    
    Ok(())
}

/// Mark messages as read
#[tauri::command]
pub async fn mark_channel_as_read(
    channel_id: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<(), String> {
    // TODO: Implement marking messages as read
    Ok(())
}

/// Send typing indicator
#[tauri::command]
pub async fn send_typing_indicator(
    channel_id: String,
    is_typing: bool,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let mut service = messaging_state.service.write().await;
    
    let channel = ChannelId::from(channel_id.as_str());
    
    if is_typing {
        service.start_typing(channel).await
            .map_err(|e| format!("Failed to send typing indicator: {}", e))?;
    } else {
        service.stop_typing(channel).await
            .map_err(|e| format!("Failed to send typing indicator: {}", e))?;
    }
    
    // Emit typing event
    app_handle.emit_all("user-typing", serde_json::json!({
        "channelId": channel_id,
        "userId": "current-user", // TODO: Get actual user ID
        "isTyping": is_typing,
    }))
    .map_err(|e| format!("Failed to emit typing event: {}", e))?;
    
    Ok(())
}

/// Get thread messages
#[tauri::command]
pub async fn get_thread_messages(
    thread_id: String,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<Vec<RichMessageDto>, String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let service = messaging_state.service.read().await;
    
    let thread = service.get_thread_messages(ThreadId::from(thread_id.as_str()))
        .await
        .map_err(|e| format!("Failed to get thread messages: {}", e))?;
    
    // Convert messages to DTOs
    let message_dtos: Vec<RichMessageDto> = thread.messages.into_iter()
        .map(|msg| convert_message_to_dto(msg))
        .collect();
    
    Ok(message_dtos)
}

/// Search messages
#[tauri::command]
pub async fn search_messages(
    query: String,
    limit: usize,
    state: State<'_, Arc<RwLock<Option<MessagingState>>>>,
) -> Result<Vec<RichMessageDto>, String> {
    let state_guard = state.read().await;
    let messaging_state = state_guard.as_ref()
        .ok_or_else(|| "Messaging not initialized".to_string())?;
    
    let service = messaging_state.service.read().await;
    
    let search_query = SearchQuery {
        text: Some(query),
        from: None,
        in_channels: None,
        has_attachments: None,
        has_reactions: None,
        is_thread: None,
        date_range: None,
        limit,
    };
    
    let messages = service.search_messages(search_query)
        .await
        .map_err(|e| format!("Failed to search messages: {}", e))?;
    
    let message_dtos: Vec<RichMessageDto> = messages.into_iter()
        .map(|msg| convert_message_to_dto(msg))
        .collect();
    
    Ok(message_dtos)
}

// Helper function to convert domain model to DTO
fn convert_message_to_dto(message: RichMessage) -> RichMessageDto {
    let content = match message.content {
        MessageContent::Text(text) => MessageContentDto {
            text: Some(text),
            markdown: None,
            mentions: vec![],
            links: vec![],
        },
        MessageContent::RichText(rich) => MessageContentDto {
            text: Some(rich.raw),
            markdown: Some(rich.formatted),
            mentions: rich.mentions.iter().map(|m| m.to_string()).collect(),
            links: rich.links,
        },
        _ => MessageContentDto {
            text: Some("Unsupported content type".to_string()),
            markdown: None,
            mentions: vec![],
            links: vec![],
        },
    };
    
    RichMessageDto {
        id: message.id.to_string(),
        sender: SenderInfo {
            id: message.sender.to_string(),
            name: message.sender.to_string().split('-').next().unwrap_or("Unknown").to_string(),
            avatar: None,
            four_word_address: message.sender.to_string(),
        },
        channel_id: message.channel_id.to_string(),
        content,
        attachments: message.attachments.into_iter()
            .map(|a| AttachmentDto {
                id: a.id.to_string(),
                attachment_type: match a.attachment_type {
                    saorsa_core::messaging::types::AttachmentType::Image => "image",
                    saorsa_core::messaging::types::AttachmentType::Video => "video",
                    saorsa_core::messaging::types::AttachmentType::Audio => "audio",
                    saorsa_core::messaging::types::AttachmentType::File => "file",
                }.to_string(),
                name: a.filename,
                size: a.size,
                url: format!("/api/attachments/{}", a.id),
                thumbnail: a.thumbnail.map(|t| format!("/api/thumbnails/{}", t)),
                mime_type: a.mime_type,
            })
            .collect(),
        thread_id: message.thread_id.map(|id| id.to_string()),
        reply_to: message.reply_to.map(|id| ReplyInfo {
            id: id.to_string(),
            sender: "Unknown".to_string(), // TODO: Fetch actual sender
            preview: "Message preview".to_string(), // TODO: Fetch actual preview
        }),
        reactions: message.reactions.into_iter()
            .map(|r| ReactionInfo {
                emoji: r.emoji,
                users: r.users.iter().map(|u| u.to_string()).collect(),
            })
            .collect(),
        timestamp: message.created_at.to_rfc3339(),
        edited_at: message.edited_at.map(|t| t.to_rfc3339()),
        deleted_at: message.deleted_at.map(|t| t.to_rfc3339()),
        read_by: message.read_receipts.into_iter()
            .map(|r| r.user.to_string())
            .collect(),
        status: match message.delivery_status {
            saorsa_core::messaging::types::DeliveryStatus::Sending => "sending",
            saorsa_core::messaging::types::DeliveryStatus::Sent => "sent",
            saorsa_core::messaging::types::DeliveryStatus::Delivered => "delivered",
            saorsa_core::messaging::types::DeliveryStatus::Read => "read",
            saorsa_core::messaging::types::DeliveryStatus::Failed => "failed",
        }.to_string(),
        encrypted: message.encrypted,
        ephemeral: message.ephemeral,
        pinned: message.pinned,
        starred: message.starred,
        signature: Some(format!("{:?}", message.signature)),
    }
}

// Export function to register all commands
pub fn register_messaging_commands() -> Vec<Box<dyn Fn() -> String + Send + Sync>> {
    vec![
        Box::new(|| "initialize_unified_messaging".to_string()),
        Box::new(|| "get_channels".to_string()),
        Box::new(|| "create_channel".to_string()),
        Box::new(|| "get_channel_messages".to_string()),
        Box::new(|| "send_rich_message".to_string()),
        Box::new(|| "add_reaction".to_string()),
        Box::new(|| "edit_message".to_string()),
        Box::new(|| "delete_message".to_string()),
        Box::new(|| "mark_channel_as_read".to_string()),
        Box::new(|| "send_typing_indicator".to_string()),
        Box::new(|| "get_thread_messages".to_string()),
        Box::new(|| "search_messages".to_string()),
    ]
}