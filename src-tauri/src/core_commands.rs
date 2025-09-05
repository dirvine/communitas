use crate::core_context::CoreContext;
use saorsa_core::DelegatedWriteAuth;
use saorsa_core::api::{
    ContainerManifestV1, FecParams, PutPolicy, container_manifest_fetch, container_manifest_put,
};
use saorsa_core::chat::{Attachment, Channel, ChannelId, ChannelType, Message, MessageId, Thread};
use saorsa_core::dht::Key as DhtKey;
use saorsa_core::identity::FourWordAddress;
use saorsa_core::identity::enhanced::DeviceType;
use saorsa_core::messaging::ChannelId as MessagingChannelId;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::State;
use tokio::sync::RwLock;

/// Initialize saorsa-core wiring and cache it in state
#[tauri::command]
pub async fn core_initialize(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    four_words: String,
    display_name: String,
    device_name: Option<String>,
    device_type: Option<String>,
) -> Result<bool, String> {
    let dev_type = match device_type
        .unwrap_or_else(|| "Desktop".to_string())
        .as_str()
    {
        "Desktop" | "desktop" => DeviceType::Desktop,
        "Mobile" | "mobile" => DeviceType::Mobile,
        "Tablet" | "tablet" => DeviceType::Tablet,
        "Web" | "web" => DeviceType::Web,
        other => return Err(format!("Unknown device type: {}", other)),
    };

    let ctx = CoreContext::initialize(
        four_words,
        display_name,
        device_name.unwrap_or_else(|| "device".to_string()),
        dev_type,
    )
    .await?;

    let mut guard = shared.write().await;
    *guard = Some(ctx);
    Ok(true)
}

/// Create a public channel
#[tauri::command]
pub async fn core_create_channel(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    name: String,
    description: String,
) -> Result<Channel, String> {
    // Validate input
    if name.is_empty() {
        return Err("Channel name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Channel name too long (max 100 characters)".to_string());
    }
    if description.len() > 500 {
        return Err("Channel description too long (max 500 characters)".to_string());
    }

    // Basic name validation (alphanumeric, hyphens, underscores)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        return Err("Channel name contains invalid characters".to_string());
    }
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    chat.create_channel(name, description, ChannelType::Public, None)
        .await
        .map_err(|e| format!("create_channel failed: {}", e))
}

/// List user's channels (basic)
#[tauri::command]
pub async fn core_get_channels(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<Channel>, String> {
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    let ids = chat
        .get_user_channels()
        .await
        .map_err(|e| format!("get_user_channels failed: {}", e))?;
    let mut channels = Vec::new();
    for id in ids {
        let ch = chat
            .get_channel(&id)
            .await
            .map_err(|e| format!("get_channel failed: {}", e))?;
        channels.push(ch);
    }
    Ok(channels)
}

/// Add a reaction to a message in a channel
#[tauri::command]
pub async fn core_add_reaction(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    message_id: String,
    emoji: String,
) -> Result<bool, String> {
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    chat.add_reaction(&ChannelId(channel_id), &MessageId(message_id), emoji)
        .await
        .map_err(|e| format!("add_reaction failed: {}", e))?;
    Ok(true)
}

/// Send a message via MessagingService to explicit recipients (four-word addresses)
#[tauri::command]
pub async fn core_send_message_to_recipients(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    recipients: Vec<String>,
    text: String,
) -> Result<String, String> {
    // Validate input
    if text.is_empty() {
        return Err("Message text cannot be empty".to_string());
    }
    if text.len() > 10 * 1024 {
        // 10KB limit
        return Err("Message too long (max 10KB)".to_string());
    }
    if recipients.is_empty() {
        return Err("Must specify at least one recipient".to_string());
    }
    if recipients.len() > 100 {
        // Reasonable limit
        return Err("Too many recipients (max 100)".to_string());
    }

    // Validate recipient format (basic check)
    for recipient in &recipients {
        if recipient.split('-').count() != 4 {
            return Err(format!("Invalid recipient format: {}", recipient));
        }
    }
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let messaging = &mut ctx.messaging;

    let mapped: Vec<FourWordAddress> = recipients.into_iter().map(FourWordAddress).collect();
    let channel_uuid = uuid::Uuid::parse_str(&channel_id)
        .map_err(|e| format!("Invalid channel ID format: {}", e))?;
    let (msg_id, _receipt) = ctx
        .messaging
        .send_message(
            mapped,
            saorsa_core::messaging::MessageContent::Text(text),
            MessagingChannelId(channel_uuid),
            Default::default(),
        )
        .await
        .map_err(|e| format!("send_message failed: {}", e))?;
    Ok(msg_id.to_string())
}

/// Create a thread under a parent message in a channel
#[tauri::command]
pub async fn core_create_thread(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    parent_message_id: String,
) -> Result<Thread, String> {
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    chat.create_thread(&ChannelId(channel_id), &MessageId(parent_message_id))
        .await
        .map_err(|e| format!("create_thread failed: {}", e))
}

/// Send a message to all channel members
#[tauri::command]
pub async fn core_send_message_to_channel(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    text: String,
) -> Result<String, String> {
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let (msg_id, _receipt) = ctx
        .messaging
        .send_message_to_channel(
            MessagingChannelId(uuid::Uuid::parse_str(&channel_id).unwrap_or_default()),
            saorsa_core::messaging::MessageContent::Text(text),
            Default::default(),
        )
        .await
        .map_err(|e| format!("send_message_to_channel failed: {}", e))?;
    Ok(msg_id.to_string())
}

/// Get channel recipients
#[tauri::command]
pub async fn core_channel_recipients(channel_id: String) -> Result<Vec<String>, String> {
    let recips = saorsa_core::messaging::service::channel_recipients(&MessagingChannelId(
        uuid::Uuid::parse_str(&channel_id).unwrap_or_default(),
    ))
    .await
    .map_err(|e| format!("channel_recipients failed: {}", e))?;
    Ok(recips.into_iter().map(|fw| fw.0).collect())
}

/// Subscribe to messages
#[tauri::command]
pub async fn core_subscribe_messages(
    app: AppHandle,
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: Option<String>,
) -> Result<bool, String> {
    let rx = {
        let guard = shared.read().await;
        let ctx = guard
            .as_ref()
            .ok_or_else(|| "Core not initialized".to_string())?;
        ctx.messaging
            .subscribe_messages(
                channel_id
                    .map(|id| MessagingChannelId(uuid::Uuid::parse_str(&id).unwrap_or_default())),
            )
            .await
    };

    let shared_clone = shared.inner().clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        let mut rx = rx;
        loop {
            match rx.recv().await {
                Ok(rec) => {
                    let payload = {
                        let guard = shared_clone.read().await;
                        if let Some(ctx) = guard.as_ref() {
                            match ctx.messaging.decrypt_message(rec.message.clone()).await {
                                Ok(rich) => serde_json::to_value(&rich).unwrap_or_else(|_| {
                                    serde_json::json!({
                                        "error": "serialize_failed"
                                    })
                                }),
                                Err(_) => {
                                    serde_json::json!({ "encrypted": true, "receivedAt": rec.received_at.to_rfc3339() })
                                }
                            }
                        } else {
                            serde_json::json!({ "error": "core_not_initialized" })
                        }
                    };
                    let _ = app_clone.emit("message-received", payload);
                }
                Err(_) => break,
            }
        }
    });

    Ok(true)
}

/// Store private encrypted content
#[tauri::command]
pub async fn core_private_put(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
    content: Vec<u8>,
) -> Result<bool, String> {
    let mut guard = shared.write().await;
    let ctx = guard
        .as_mut()
        .ok_or_else(|| "Core not initialized".to_string())?;
    ctx.storage
        .store_encrypted(
            &key,
            &content,
            std::time::Duration::from_secs(365 * 24 * 60 * 60),
            None,
        )
        .await
        .map_err(|e| format!("store_encrypted failed: {}", e))?;
    Ok(true)
}

/// Retrieve private encrypted content
#[tauri::command]
pub async fn core_private_get(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
) -> Result<Vec<u8>, String> {
    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "Core not initialized".to_string())?;
    ctx.storage
        .get_encrypted::<Vec<u8>>(&key)
        .await
        .map_err(|e| format!("get_encrypted failed: {}", e))
}
