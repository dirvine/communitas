use crate::core_context::CoreContext;
use saorsa_core::chat::{Attachment, Channel, ChannelId, ChannelType, Message, MessageId, Thread};
use saorsa_core::messaging::ChannelId as MessagingChannelId;
use saorsa_core::identity::FourWordAddress;
use saorsa_core::identity::enhanced::DeviceType;
use saorsa_core::api::{ContainerManifestV1, FecParams, container_manifest_put, container_manifest_fetch, PutPolicy};
use saorsa_core::dht::Key as DhtKey;
use saorsa_core::DelegatedWriteAuth;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tauri::AppHandle;
use tauri::Emitter;

/// Initialize saorsa-core wiring and cache it in state
#[tauri::command]
pub async fn core_initialize(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    four_words: String,
    display_name: String,
    device_name: Option<String>,
    device_type: Option<String>,
) -> Result<bool, String> {
    let dev_type = match device_type.unwrap_or_else(|| "Desktop".to_string()).as_str() {
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
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
        return Err("Channel name contains invalid characters".to_string());
    }
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    chat
        .create_channel(name, description, ChannelType::Public, None)
        .await
        .map_err(|e| format!("create_channel failed: {}", e))
}

/// List user's channels (basic)
#[tauri::command]
pub async fn core_get_channels(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<Channel>, String> {
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
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
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    chat
        .add_reaction(&ChannelId(channel_id), &MessageId(message_id), emoji)
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
    if text.len() > 10 * 1024 { // 10KB limit
        return Err("Message too long (max 10KB)".to_string());
    }
    if recipients.is_empty() {
        return Err("Must specify at least one recipient".to_string());
    }
    if recipients.len() > 100 { // Reasonable limit
        return Err("Too many recipients (max 100)".to_string());
    }

    // Validate recipient format (basic check)
    for recipient in &recipients {
        if recipient.split('-').count() != 4 {
            return Err(format!("Invalid recipient format: {}", recipient));
        }
    }
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    let mapped: Vec<FourWordAddress> = recipients.into_iter().map(FourWordAddress).collect();
    let channel_uuid = uuid::Uuid::parse_str(&channel_id)
        .map_err(|e| format!("Invalid channel ID format: {}", e))?;
    let (msg_id, _receipt) = ctx
        .messaging
        .send_message(mapped, saorsa_core::messaging::MessageContent::Text(text), MessagingChannelId(channel_uuid), Default::default())
        .await
        .map_err(|e| format!("send_message failed: {}", e))?;
    Ok(msg_id.to_string())
}

/// Send a message to all channel members via saorsa-core's channel delivery
#[tauri::command]
pub async fn core_send_message_to_channel(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    text: String,
) -> Result<String, String> {
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    let channel_uuid = uuid::Uuid::parse_str(&channel_id)
        .map_err(|e| format!("Invalid channel ID format: {}", e))?;
    let (msg_id, _receipt) = ctx
        .messaging
        .send_message_to_channel(MessagingChannelId(channel_uuid), saorsa_core::messaging::MessageContent::Text(text), Default::default())
        .await
        .map_err(|e| format!("send_message_to_channel failed: {}", e))?;
    Ok(msg_id.to_string())
}

/// Get channel recipients (four-word addresses)
#[tauri::command]
pub async fn core_channel_recipients(channel_id: String) -> Result<Vec<String>, String> {
    let channel_uuid = uuid::Uuid::parse_str(&channel_id)
        .map_err(|e| format!("Invalid channel ID format: {}", e))?;
    let recips = saorsa_core::messaging::service::channel_recipients(&MessagingChannelId(channel_uuid))
        .await
        .map_err(|e| format!("channel_recipients failed: {}", e))?;
    Ok(recips.into_iter().map(|fw| fw.0).collect())
}

// --------------- Virtual Disk wrappers ---------------

#[derive(serde::Deserialize)]
pub struct DiskWriteInput {
    pub entity_hex: String,
    pub disk_type: String,
    pub path: String,
    pub content_base64: String,
    pub mime_type: Option<String>,
}

fn parse_disk_type(s: &str) -> Result<saorsa_core::virtual_disk::DiskType, String> {
    match s {
        "Private" | "private" => Ok(saorsa_core::virtual_disk::DiskType::Private),
        "Public" | "public" => Ok(saorsa_core::virtual_disk::DiskType::Public),
        "Shared" | "shared" => Ok(saorsa_core::virtual_disk::DiskType::Shared),
        other => Err(format!("Unknown disk type: {}", other)),
    }
}

#[tauri::command]
pub async fn core_disk_write(input: DiskWriteInput) -> Result<serde_json::Value, String> {
    use saorsa_core::virtual_disk as vd;
    let entity = parse_hex_key(&input.entity_hex)?;
    let dt = parse_disk_type(&input.disk_type)?;
    let handle = match vd::disk_mount(entity.clone().into(), dt).await {
        Ok(h) => h,
        Err(_) => vd::disk_create(entity.clone().into(), dt, vd::DiskConfig::default()).await.map_err(|e| e.to_string())?,
    };
    let content = base64::engine::general_purpose::STANDARD.decode(&input.content_base64).map_err(|e| format!("invalid base64: {}", e))?;
    let meta = vd::FileMetadata { mime_type: input.mime_type, attributes: std::collections::HashMap::new(), permissions: 0o644 };
    let receipt = vd::disk_write(&handle, &input.path, &content, meta).await.map_err(|e| e.to_string())?;
    serde_json::to_value(&receipt).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn core_disk_read(entity_hex: String, disk_type: String, path: String) -> Result<Vec<u8>, String> {
    use saorsa_core::virtual_disk as vd;
    let entity = parse_hex_key(&entity_hex)?;
    let dt = parse_disk_type(&disk_type)?;
    let handle = vd::disk_mount(entity.into(), dt).await.map_err(|e| e.to_string())?;
    vd::disk_read(&handle, &path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn core_disk_list(entity_hex: String, disk_type: String, path: String, recursive: bool) -> Result<serde_json::Value, String> {
    use saorsa_core::virtual_disk as vd;
    let entity = parse_hex_key(&entity_hex)?;
    let dt = parse_disk_type(&disk_type)?;
    let handle = vd::disk_mount(entity.into(), dt).await.map_err(|e| e.to_string())?;
    let entries = vd::disk_list(&handle, &path, recursive).await.map_err(|e| e.to_string())?;
    serde_json::to_value(&entries).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn core_disk_delete(entity_hex: String, disk_type: String, path: String) -> Result<bool, String> {
    use saorsa_core::virtual_disk as vd;
    let entity = parse_hex_key(&entity_hex)?;
    let dt = parse_disk_type(&disk_type)?;
    let handle = vd::disk_mount(entity.into(), dt).await.map_err(|e| e.to_string())?;
    vd::disk_delete(&handle, &path).await.map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn core_disk_sync(entity_hex: String, disk_type: String) -> Result<serde_json::Value, String> {
    use saorsa_core::virtual_disk as vd;
    let entity = parse_hex_key(&entity_hex)?;
    let dt = parse_disk_type(&disk_type)?;
    let handle = vd::disk_mount(entity.into(), dt).await.map_err(|e| e.to_string())?;
    let status = vd::disk_sync(&handle).await.map_err(|e| e.to_string())?;
    serde_json::to_value(&status).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct WebsiteAssetInput { pub path: String, pub content_base64: String, pub mime_type: String }

#[tauri::command]
pub async fn core_website_set_home(entity_hex: String, markdown_content: String, assets: Vec<WebsiteAssetInput>) -> Result<bool, String> {
    use saorsa_core::virtual_disk as vd;

    // Validate input sizes
    if markdown_content.len() > 10 * 1024 * 1024 { // 10MB limit
        return Err("Markdown content too large (max 10MB)".to_string());
    }

    if assets.len() > 100 { // Reasonable limit on number of assets
        return Err("Too many assets (max 100)".to_string());
    }

    // Validate total asset size
    let total_asset_size: usize = assets.iter()
        .map(|a| a.content_base64.len() * 3 / 4) // Rough base64 to binary conversion
        .sum();
    if total_asset_size > 50 * 1024 * 1024 { // 50MB total limit
        return Err("Total asset size too large (max 50MB)".to_string());
    }

    let entity = parse_hex_key(&entity_hex)?;
    let handle = match vd::disk_mount(entity.clone().into(), vd::DiskType::Public).await {
        Ok(h) => h,
        Err(_) => vd::disk_create(entity.clone().into(), vd::DiskType::Public, vd::DiskConfig::default()).await.map_err(|e| e.to_string())?,
    };
    let asset_vec: Vec<vd::Asset> = assets
        .into_iter()
        .map(|a| {
            let content = base64::engine::general_purpose::STANDARD.decode(&a.content_base64)
                .map_err(|e| format!("Invalid base64 content for asset {}: {}", a.path, e))?;
            Ok(vd::Asset { path: a.path, content, mime_type: a.mime_type })
        })
        .collect::<Result<Vec<_>, String>>()?;
    vd::website_set_home(&handle, &markdown_content, asset_vec).await.map_err(|e| e.to_string())?;
    Ok(true)
}

/// Update identity's website_root using a detached ML-DSA signature (hex inputs)
#[tauri::command]
pub async fn core_identity_set_website_root(id_hex: String, website_root_hex: String, sig_hex: String) -> Result<bool, String> {
    use saorsa_core::api::{identity_set_website_root};
    use saorsa_core::auth::Sig;

    let id = parse_hex_key(&id_hex)?;
    let root = parse_hex_key(&website_root_hex)?;
    let sig_bytes = hex::decode(sig_hex).map_err(|e| e.to_string())?;
    let sig = Sig::new(sig_bytes);
    identity_set_website_root(id.into(), root.into(), sig).await.map_err(|e| e.to_string())?;
    Ok(true)
}

/// Publish a website and return a publish receipt
#[tauri::command]
pub async fn core_website_publish_receipt(entity_hex: String, website_root_hex: String) -> Result<serde_json::Value, String> {
    use saorsa_core::virtual_disk as vd;
    let entity = parse_hex_key(&entity_hex)?;
    let website_root = parse_hex_key(&website_root_hex)?;
    let receipt = vd::website_publish(entity.into(), website_root.into()).await.map_err(|e| e.to_string())?;
    serde_json::to_value(&receipt).map_err(|e| e.to_string())
}

/// Publish a website and optionally update identity.website_root if signature provided
#[tauri::command]
pub async fn core_website_publish_and_update_identity(
    entity_hex: String,
    website_root_hex: String,
    sig_hex: Option<String>,
) -> Result<serde_json::Value, String> {
    use saorsa_core::virtual_disk as vd;
    use saorsa_core::api::{identity_set_website_root};
    use saorsa_core::auth::Sig;
    let entity = parse_hex_key(&entity_hex)?;
    let website_root = parse_hex_key(&website_root_hex)?;
    let receipt = vd::website_publish(entity.clone().into(), website_root.clone().into())
        .await
        .map_err(|e| e.to_string())?;

    if let Some(sig_h) = sig_hex {
        let sig_bytes = hex::decode(sig_h).map_err(|e| e.to_string())?;
        let sig = Sig::new(sig_bytes);
        identity_set_website_root(entity.into(), website_root.into(), sig)
            .await
            .map_err(|e| e.to_string())?;
    }

    serde_json::to_value(&receipt).map_err(|e| e.to_string())
}

/// Subscribe to messages (emits `message-received` events). Optional channel filter.
#[tauri::command]
pub async fn core_subscribe_messages(
    app: AppHandle,
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: Option<String>,
) -> Result<bool, String> {
    let rx = {
        let guard = shared.read().await;
        let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
        ctx.messaging
            .subscribe_messages(channel_id.map(|id| MessagingChannelId(uuid::Uuid::parse_str(&id).unwrap_or_default())))
            .await
    };

    let shared_clone = shared.inner().clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        let mut rx = rx;
        loop {
            match rx.recv().await {
                Ok(rec) => {
                    // Try to decrypt; fall back to encrypted payload marker
                    let payload = {
                        let guard = shared_clone.read().await;
                        if let Some(ctx) = guard.as_ref() {
                            match ctx.messaging.decrypt_message(rec.message.clone()).await {
                                Ok(rich) => serde_json::to_value(&rich).unwrap_or_else(|_| serde_json::json!({
                                    "error": "serialize_failed"
                                })),
                                Err(_) => serde_json::json!({ "encrypted": true, "receivedAt": rec.received_at.to_rfc3339() }),
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

fn parse_hex_key(hexstr: &str) -> Result<DhtKey, String> {
    // Validate input length (64 hex chars = 32 bytes)
    if hexstr.len() != 64 {
        return Err(format!("hex key must be exactly 64 characters (32 bytes), got {}", hexstr.len()));
    }

    // Validate hex format
    if !hexstr.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("hex key contains invalid characters".to_string());
    }

    let bytes = hex::decode(hexstr).map_err(|e| format!("invalid hex: {}", e))?;

    // Double-check length after decoding
    if bytes.len() != 32 {
        return Err(format!("decoded key must be 32 bytes, got {}", bytes.len()));
    }

    let mut arr = [0u8;32];
    arr.copy_from_slice(&bytes);
    Ok(DhtKey::from(arr))
}

#[derive(serde::Deserialize)]
pub struct WebsiteManifestInput {
    pub object_hex: String,
    pub k: u16,
    pub m: u16,
    pub shard_size: u32,
    pub assets_hex: Vec<String>,
    pub sealed_meta_hex: Option<String>,
}

/// Publish a website container manifest to the DHT (does not modify identity.website_root)
#[tauri::command]
pub async fn core_website_publish(manifest: WebsiteManifestInput) -> Result<bool, String> {
    let object = parse_hex_key(&manifest.object_hex)?;
    let assets: Result<Vec<DhtKey>, String> = manifest
        .assets_hex
        .iter()
        .map(|h| parse_hex_key(h))
        .collect();
    let assets = assets?;
    let sealed_meta = match manifest.sealed_meta_hex {
        Some(h) => Some(parse_hex_key(&h)?),
        None => None,
    };

    let cm = ContainerManifestV1 {
        v: 1,
        object: object.into(),
        fec: FecParams { k: manifest.k, m: manifest.m, shard_size: manifest.shard_size },
        assets: assets.into_iter().map(|a| a.into()).collect(),
        sealed_meta: sealed_meta.map(|s| s.into()),
    };

    let pol = PutPolicy { quorum: 3, ttl: None, auth: Box::new(DelegatedWriteAuth::new(vec![])) };
    container_manifest_put(&cm, &pol)
        .await
        .map_err(|e| format!("manifest_put failed: {}", e))?;
    Ok(true)
}

/// Fetch a website container manifest by object key
#[tauri::command]
pub async fn core_website_get_manifest(object_hex: String) -> Result<serde_json::Value, String> {
    let object = parse_hex_key(&object_hex)?;
    let cm = container_manifest_fetch(&object)
        .await
        .map_err(|e| format!("manifest_fetch failed: {}", e))?;
    serde_json::to_value(&cm).map_err(|e| format!("serialize failed: {}", e))
}

/// Store private encrypted content under a namespaced key
#[tauri::command]
pub async fn core_private_put(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
    content: Vec<u8>,
) -> Result<bool, String> {
    let mut guard = shared.write().await;
    let ctx = guard.as_mut().ok_or_else(|| "Core not initialized".to_string())?;
    ctx.storage
        .store_encrypted(&key, &content, std::time::Duration::from_secs(365 * 24 * 60 * 60), None)
        .await
        .map_err(|e| format!("store_encrypted failed: {}", e))?;
    Ok(true)
}

/// Retrieve private encrypted content by key
#[tauri::command]
pub async fn core_private_get(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
) -> Result<Vec<u8>, String> {
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    ctx.storage
        .get_encrypted::<Vec<u8>>(&key)
        .await
        .map_err(|e| format!("get_encrypted failed: {}", e))
}

/// Post a text message to a channel (optionally in a thread)
#[tauri::command]
pub async fn core_post_message(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    text: String,
    thread_id: Option<String>,
) -> Result<Message, String> {
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;

    // Map provided thread id string into ThreadId if present
    let thread_opt = thread_id.map(|t| saorsa_core::chat::ThreadId(t));

    let msg = chat
        .send_message(
            &saorsa_core::chat::ChannelId(channel_id),
            saorsa_core::chat::MessageContent::Text(text),
            thread_opt,
            Vec::<Attachment>::new(),
        )
        .await
        .map_err(|e| format!("send_message failed: {}", e))?;

    // Optional: attempt network delivery via MessagingService if needed in future
    Ok(msg)
}

/// Create a thread under a parent message in a channel
#[tauri::command]
pub async fn core_create_thread(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    channel_id: String,
    parent_message_id: String,
) -> Result<Thread, String> {
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;
    let chat = &mut ctx.chat;
    chat
        .create_thread(&ChannelId(channel_id), &MessageId(parent_message_id))
        .await
        .map_err(|e| format!("create_thread failed: {}", e))
}
