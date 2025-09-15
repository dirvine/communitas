use communitas_core::CoreContext;
use saorsa_core::chat::{Channel, ChannelId, ChannelType, MessageId, Thread};
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
    // use ctx.messaging directly below

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

    // Resolve recipients from channel membership using four-word formatted user_ids when available
    let ch = ctx
        .chat
        .get_channel(&ChannelId(channel_id.clone()))
        .await
        .map_err(|e| format!("get_channel failed: {}", e))?;

    let mut recipients: Vec<saorsa_core::identity::FourWordAddress> = Vec::new();
    for m in ch.members {
        // Heuristic: treat user_id as four-word address if it contains 4 hyphen-separated words
        if m.user_id.split('-').count() == 4 {
            recipients.push(saorsa_core::identity::FourWordAddress(
                m.user_id.to_lowercase(),
            ));
        }
    }

    if recipients.is_empty() {
        // Fallback to self so user sees message locally if channel member IDs are not four-words yet
        recipients.push(saorsa_core::identity::FourWordAddress(
            ctx.four_words.clone(),
        ));
    }

    let (msg_id, _receipt) = ctx
        .messaging
        .send_message(
            recipients,
            saorsa_core::messaging::MessageContent::Text(text),
            MessagingChannelId(uuid::Uuid::parse_str(&channel_id).unwrap_or_default()),
            Default::default(),
        )
        .await
        .map_err(|e| format!("send_message failed: {}", e))?;
    Ok(msg_id.to_string())
}

/// Invite a member to a channel by four-word address; registers mapping and adds to channel
#[tauri::command]
pub async fn core_channel_invite_by_words(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    invitee_words: [String; 4],
    _role: Option<String>,
) -> Result<bool, String> {
    // Validate words
    if !saorsa_core::fwid::fw_check(invitee_words.clone()) {
        return Err("Invalid four-word address format".to_string());
    }

    let _invitee_key = saorsa_core::fwid::fw_to_key(invitee_words.clone())
        .map_err(|e| format!("fw_to_key failed: {}", e))?;

    // TODO: The saorsa_core v0.3.17 doesn't have get_user_by_four_words function
    // and ChatManager doesn't have add_member method
    // This functionality needs to be implemented when the API is available
    Err("Channel member addition not yet implemented in current saorsa_core version".to_string())
}

/// Get channel recipients
#[tauri::command]
pub async fn core_channel_recipients(_channel_id: String) -> Result<Vec<String>, String> {
    // When running inside Communitas, channel membership is managed by ChatManager in state,
    // so this Tauri command should be called via core_send_message_to_channel instead.
    // Keep a minimal implementation that returns an empty list to signal the UI to compute or skip.
    Ok(Vec::new())
}

#[derive(serde::Serialize)]
pub struct ChannelMemberEntry {
    pub user_id: String,
    pub role: String,
    pub four_words: Option<String>,
}

/// List channel members with resolved four-word addresses when available
#[tauri::command]
pub async fn core_channel_list_members(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
) -> Result<Vec<ChannelMemberEntry>, String> {
    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "Core not initialized".to_string())?;
    let ch = ctx
        .chat
        .get_channel(&ChannelId(channel_id))
        .await
        .map_err(|e| format!("get_channel failed: {}", e))?;
    let mut out = Vec::with_capacity(ch.members.len());
    for m in ch.members {
        // TODO: saorsa_core v0.3.17 doesn't have get_user_four_words function
        let words_opt: Option<String> = None;
        out.push(ChannelMemberEntry {
            user_id: m.user_id,
            role: format!("{:?}", m.role),
            four_words: words_opt,
        });
    }
    Ok(out)
}

/// Resolve member addresses for a channel in the background and emit per-member events.
/// Emits `channel-member-resolved` with payload `{ user_id, role, four_words }` as each resolves.
#[tauri::command]
pub async fn core_resolve_channel_members(
    app: AppHandle,
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "Core not initialized".to_string())?;

    let ch = ctx
        .chat
        .get_channel(&ChannelId(channel_id))
        .await
        .map_err(|e| format!("get_channel failed: {}", e))?;

    let app_clone = app.clone();
    tokio::spawn(async move {
        for m in ch.members {
            // TODO: saorsa_core v0.3.17 doesn't have get_user_four_words function
            let words_opt: Option<Vec<String>> = None;
            let payload = serde_json::json!({
                "user_id": m.user_id,
                "role": format!("{:?}", m.role),
                "four_words": words_opt,
            });
            let _ = app_clone.emit("channel-member-resolved", payload);
        }
    });

    Ok(true)
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
        while let Ok(rec) = rx.recv().await {
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

/// Get bootstrap nodes from configuration
#[tauri::command]
pub async fn core_get_bootstrap_nodes() -> Result<Vec<String>, String> {
    // Load bootstrap configuration from file
    let config_path = std::path::PathBuf::from("bootstrap.toml");

    if !config_path.exists() {
        // Return default bootstrap nodes if config doesn't exist
        return Ok(vec![
            "ocean-forest-moon-star:443".to_string(),
            "river-mountain-sun-cloud:443".to_string(),
        ]);
    }

    // Parse TOML configuration
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read bootstrap config: {}", e))?;

    let config: toml::Value =
        toml::from_str(&content).map_err(|e| format!("Failed to parse bootstrap config: {}", e))?;

    // Extract seeds array
    let seeds = config
        .get("seeds")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "No seeds array found in bootstrap config".to_string())?;

    let mut bootstrap_nodes = Vec::new();
    for seed in seeds {
        if let Some(seed_str) = seed.as_str() {
            bootstrap_nodes.push(seed_str.to_string());
        }
    }

    Ok(bootstrap_nodes)
}

/// Update bootstrap nodes configuration
#[tauri::command]
pub async fn core_update_bootstrap_nodes(nodes: Vec<String>) -> Result<bool, String> {
    use std::collections::BTreeMap;

    let mut config = BTreeMap::new();
    config.insert(
        "seeds".to_string(),
        toml::Value::Array(nodes.into_iter().map(toml::Value::String).collect()),
    );

    let content =
        toml::to_string(&config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write("bootstrap.toml", content)
        .map_err(|e| format!("Failed to write bootstrap config: {}", e))?;

    Ok(true)
}
