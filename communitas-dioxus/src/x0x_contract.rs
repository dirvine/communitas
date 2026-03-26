//! App-owned x0x interop helpers for Dioxus.
//!
//! This module freezes the GUI-compatible channel/topic/store conventions that
//! live above the shared transport crate.

use crate::models::channel::{ChannelMeta, ChatMessage, LegacyChannelIndex};
use base64::Engine as _;
use communitas_x0x_client::{GroupInfo, X0xClient};
use std::{collections::HashMap, fmt, path::PathBuf};
use tracing::warn;

pub const CHANNELS_INDEX_KEY: &str = "channels_index";
const HISTORY_LIMIT: usize = 200;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct LocalChatHistory {
    #[serde(default)]
    channels: HashMap<String, Vec<ChatMessage>>,
    #[serde(default)]
    threads: HashMap<String, Vec<ChatMessage>>,
    #[serde(default)]
    direct_messages: HashMap<String, Vec<ChatMessage>>,
}

#[derive(Debug)]
pub enum X0xContractError {
    Base64(base64::DecodeError),
    Json(serde_json::Error),
}

impl fmt::Display for X0xContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base64(err) => write!(f, "invalid base64 channel store payload: {err}"),
            Self::Json(err) => write!(f, "invalid channel store JSON payload: {err}"),
        }
    }
}

impl std::error::Error for X0xContractError {}

impl From<base64::DecodeError> for X0xContractError {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64(value)
    }
}

impl From<serde_json::Error> for X0xContractError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn group_prefix(group_id: &str) -> &str {
    let split_at = group_id
        .char_indices()
        .nth(16)
        .map_or(group_id.len(), |(idx, _)| idx);
    &group_id[..split_at]
}

pub fn channel_store_id(group_id: &str) -> String {
    format!("x0x-channels-{}", group_prefix(group_id))
}

pub fn channel_topic(group_id: &str, channel: &str) -> String {
    format!("x0x.group.{}.chat/{}", group_prefix(group_id), channel)
}

pub fn thread_topic(group_id: &str, message_id: &str) -> String {
    format!("x0x.group.{}.thread/{}", group_prefix(group_id), message_id)
}

pub fn normalize_channel_name(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut last_was_dash = false;

    for ch in name.trim().chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            slug.push(normalized);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

pub fn decode_channels_index(value_b64: &str) -> Result<Vec<ChannelMeta>, X0xContractError> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(value_b64)?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
pub fn encode_channels_index(channels: &[ChannelMeta]) -> Result<String, X0xContractError> {
    let bytes = serialize_channels_index(channels)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn serialize_channels_index(channels: &[ChannelMeta]) -> Result<Vec<u8>, X0xContractError> {
    Ok(serde_json::to_vec(channels)?)
}

pub fn default_general_channel(group: &GroupInfo) -> ChannelMeta {
    ChannelMeta {
        name: "general".to_string(),
        description: "General discussion".to_string(),
        creator: group.creator.clone().unwrap_or_default(),
        created_at: group.created_at.unwrap_or_default(),
        topic: group
            .chat_topic
            .clone()
            .unwrap_or_else(|| channel_topic(&group.group_id, "general")),
    }
}

pub fn ensure_general_channel(channels: &mut Vec<ChannelMeta>, group: &GroupInfo) {
    if channels.iter().any(|channel| channel.name == "general") {
        return;
    }
    channels.insert(0, default_general_channel(group));
}

pub fn sort_channels(channels: &mut [ChannelMeta]) {
    channels.sort_by(
        |left, right| match (left.name.as_str(), right.name.as_str()) {
            ("general", "general") => std::cmp::Ordering::Equal,
            ("general", _) => std::cmp::Ordering::Less,
            (_, "general") => std::cmp::Ordering::Greater,
            _ => left.name.cmp(&right.name),
        },
    );
}

pub fn fallback_sender_name(agent_id: &str) -> String {
    if agent_id.is_empty() {
        "unknown".to_string()
    } else {
        agent_id.chars().take(8).collect()
    }
}

fn local_history_path() -> PathBuf {
    if let Some(base) = std::env::var_os("COMMUNITAS_X0X_HISTORY_DIR") {
        PathBuf::from(base).join("x0x-chat-history.json")
    } else {
        dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("communitas")
            .join("x0x-chat-history.json")
    }
}

fn bounded_push(messages: &mut Vec<ChatMessage>, message: ChatMessage) {
    if messages.iter().any(|existing| existing.id == message.id) {
        return;
    }
    messages.push(message);
    messages.sort_by_key(|entry| entry.timestamp);
    if messages.len() > HISTORY_LIMIT {
        let excess = messages.len() - HISTORY_LIMIT;
        messages.drain(0..excess);
    }
}

fn channel_history_key(group_id: &str, channel_name: &str) -> String {
    format!("{group_id}:{channel_name}")
}

fn thread_history_key(group_id: &str, parent_message_id: &str) -> String {
    format!("{group_id}:{parent_message_id}")
}

async fn read_local_history() -> LocalChatHistory {
    let path = local_history_path();
    match tokio::fs::read(&path).await {
        Ok(bytes) => match serde_json::from_slice::<LocalChatHistory>(&bytes) {
            Ok(history) => history,
            Err(err) => {
                warn!(
                    target: "ui.x0x_contract",
                    "failed to decode local x0x history {}: {err}",
                    path.display()
                );
                LocalChatHistory::default()
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => LocalChatHistory::default(),
        Err(err) => {
            warn!(
                target: "ui.x0x_contract",
                "failed to read local x0x history {}: {err}",
                path.display()
            );
            LocalChatHistory::default()
        }
    }
}

async fn write_local_history(history: &LocalChatHistory) {
    let path = local_history_path();
    if let Some(parent) = path.parent()
        && let Err(err) = tokio::fs::create_dir_all(parent).await
    {
        warn!(
            target: "ui.x0x_contract",
            "failed to create local x0x history directory {}: {err}",
            parent.display()
        );
        return;
    }

    match serde_json::to_vec(history) {
        Ok(bytes) => {
            if let Err(err) = tokio::fs::write(&path, bytes).await {
                warn!(
                    target: "ui.x0x_contract",
                    "failed to write local x0x history {}: {err}",
                    path.display()
                );
            }
        }
        Err(err) => warn!(
            target: "ui.x0x_contract",
            "failed to encode local x0x history {}: {err}",
            path.display()
        ),
    }
}

pub async fn load_channel_history(group_id: &str, channel_name: &str) -> Vec<ChatMessage> {
    let history = read_local_history().await;
    let mut messages = history
        .channels
        .get(&channel_history_key(group_id, channel_name))
        .cloned()
        .unwrap_or_default();

    for message in &mut messages {
        message.reply_count = history
            .threads
            .get(&thread_history_key(group_id, &message.id))
            .map_or(0, Vec::len) as u32;
    }

    messages.sort_by_key(|entry| entry.timestamp);
    messages
}

pub async fn append_channel_history(group_id: &str, channel_name: &str, message: &ChatMessage) {
    let mut history = read_local_history().await;
    let entry = history
        .channels
        .entry(channel_history_key(group_id, channel_name))
        .or_default();
    bounded_push(entry, message.clone());
    write_local_history(&history).await;
}

pub async fn load_thread_history(group_id: &str, parent_message_id: &str) -> Vec<ChatMessage> {
    let history = read_local_history().await;
    let mut replies = history
        .threads
        .get(&thread_history_key(group_id, parent_message_id))
        .cloned()
        .unwrap_or_default();
    replies.sort_by_key(|entry| entry.timestamp);
    replies
}

pub async fn append_thread_history(group_id: &str, parent_message_id: &str, message: &ChatMessage) {
    let mut history = read_local_history().await;
    let entry = history
        .threads
        .entry(thread_history_key(group_id, parent_message_id))
        .or_default();
    bounded_push(entry, message.clone());
    write_local_history(&history).await;
}

/// Load direct message history for a specific peer agent.
pub async fn load_dm_history(peer_agent_id: &str) -> Vec<ChatMessage> {
    let history = read_local_history().await;
    let mut messages = history
        .direct_messages
        .get(peer_agent_id)
        .cloned()
        .unwrap_or_default();
    messages.sort_by_key(|entry| entry.timestamp);
    messages
}

/// Append a direct message to local history for a specific peer agent.
pub async fn append_dm_history(peer_agent_id: &str, message: &ChatMessage) {
    let mut history = read_local_history().await;
    let entry = history
        .direct_messages
        .entry(peer_agent_id.to_owned())
        .or_default();
    bounded_push(entry, message.clone());
    write_local_history(&history).await;
}

fn decode_legacy_channel_index(value_b64: &str) -> Option<LegacyChannelIndex> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(value_b64)
        .ok()?;
    serde_json::from_slice::<LegacyChannelIndex>(&decoded).ok()
}

async fn load_legacy_channels(
    client: &X0xClient,
    store_id: &str,
    legacy_index: &LegacyChannelIndex,
    group: &GroupInfo,
) -> Vec<ChannelMeta> {
    let mut channels = Vec::new();
    for channel_name in &legacy_index.channels {
        let key = format!("channel:{channel_name}");
        match client.get(store_id, &key).await {
            Ok(value) => match base64::engine::general_purpose::STANDARD.decode(&value.value) {
                Ok(decoded) => match serde_json::from_slice::<ChannelMeta>(&decoded) {
                    Ok(meta) => channels.push(meta),
                    Err(err) => warn!(
                        target: "ui.x0x_contract",
                        "failed to decode legacy channel metadata for {store_id}/{key}: {err}"
                    ),
                },
                Err(err) => warn!(
                    target: "ui.x0x_contract",
                    "failed to decode legacy channel metadata payload for {store_id}/{key}: {err}"
                ),
            },
            Err(err) => warn!(
                target: "ui.x0x_contract",
                "failed to load legacy channel metadata for {store_id}/{key}: {err}"
            ),
        }
    }

    ensure_general_channel(&mut channels, group);
    sort_channels(&mut channels);
    channels
}

pub async fn load_group_channels(client: &X0xClient, group: &GroupInfo) -> Vec<ChannelMeta> {
    let store_id = channel_store_id(&group.group_id);

    let mut channels = match client.get(&store_id, CHANNELS_INDEX_KEY).await {
        Ok(value) => match decode_channels_index(&value.value) {
            Ok(channels) => channels,
            Err(err) => {
                if let Some(legacy_index) = decode_legacy_channel_index(&value.value) {
                    let migrated =
                        load_legacy_channels(client, &store_id, &legacy_index, group).await;
                    if !migrated.is_empty() {
                        match serialize_channels_index(&migrated) {
                            Ok(bytes) => {
                                if let Err(write_err) = client
                                    .put(
                                        &store_id,
                                        CHANNELS_INDEX_KEY,
                                        &bytes,
                                        Some("application/json"),
                                    )
                                    .await
                                {
                                    warn!(
                                        target: "ui.x0x_contract",
                                        "failed to migrate legacy channel metadata for {store_id}: {write_err}"
                                    );
                                }
                            }
                            Err(write_err) => warn!(
                                target: "ui.x0x_contract",
                                "failed to encode migrated channel metadata for {store_id}: {write_err}"
                            ),
                        }
                    }
                    migrated
                } else {
                    warn!(
                        target: "ui.x0x_contract",
                        "failed to decode channels_index for {store_id}: {err}"
                    );
                    Vec::new()
                }
            }
        },
        Err(_) => Vec::new(),
    };

    ensure_general_channel(&mut channels, group);
    sort_channels(&mut channels);
    channels
}

#[cfg(test)]
mod tests {
    use super::*;

    static HISTORY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn sample_group() -> GroupInfo {
        GroupInfo {
            group_id: "1234567890abcdef1234".to_string(),
            name: "Space".to_string(),
            description: Some("Demo".to_string()),
            creator: Some("agent-1".to_string()),
            created_at: Some(1710000000000),
            member_count: Some(1),
            chat_topic: Some("x0x.group.1234567890abcdef.chat/general".to_string()),
            metadata_topic: Some("x0x.group.1234567890abcdef.meta".to_string()),
            members: vec!["agent-1".to_string()],
        }
    }

    #[test]
    fn group_prefix_uses_first_sixteen_chars() {
        assert_eq!(group_prefix("1234567890abcdef1234"), "1234567890abcdef");
        assert_eq!(group_prefix("short"), "short");
    }

    #[test]
    fn contract_topics_and_store_ids_match_frozen_schema() {
        assert_eq!(
            channel_store_id("1234567890abcdef1234"),
            "x0x-channels-1234567890abcdef"
        );
        assert_eq!(
            channel_topic("1234567890abcdef1234", "general"),
            "x0x.group.1234567890abcdef.chat/general"
        );
        assert_eq!(
            thread_topic("1234567890abcdef1234", "msg-1"),
            "x0x.group.1234567890abcdef.thread/msg-1"
        );
    }

    #[test]
    fn channel_store_round_trip_uses_array_schema() {
        let channels = vec![
            default_general_channel(&sample_group()),
            ChannelMeta {
                name: "dev".to_string(),
                description: "Developer discussion".to_string(),
                creator: "agent-1".to_string(),
                created_at: 1710000001000,
                topic: "x0x.group.1234567890abcdef.chat/dev".to_string(),
            },
        ];

        let encoded = encode_channels_index(&channels).expect("channels should encode");
        let decoded = decode_channels_index(&encoded).expect("channels should decode");

        assert_eq!(decoded, channels);
    }

    #[test]
    fn normalize_channel_name_matches_gui_slugging() {
        assert_eq!(normalize_channel_name("General Chat"), "general-chat");
        assert_eq!(normalize_channel_name("  Dev/Ops  "), "dev-ops");
        assert_eq!(normalize_channel_name("___"), "");
    }

    #[test]
    fn ensure_general_channel_adds_general_once() {
        let group = sample_group();
        let mut channels = vec![ChannelMeta {
            name: "dev".to_string(),
            description: String::new(),
            creator: "agent-1".to_string(),
            created_at: 1710000002000,
            topic: channel_topic(&group.group_id, "dev"),
        }];

        ensure_general_channel(&mut channels, &group);
        ensure_general_channel(&mut channels, &group);
        sort_channels(&mut channels);

        assert_eq!(channels[0].name, "general");
        assert_eq!(channels.len(), 2);
    }

    #[test]
    fn local_history_round_trip_restores_channel_and_thread_backfill() {
        let _guard = HISTORY_TEST_LOCK.lock().expect("history test lock");
        let temp = tempfile::tempdir().expect("temp dir");
        unsafe {
            std::env::set_var("COMMUNITAS_X0X_HISTORY_DIR", temp.path());
        }

        let root = ChatMessage {
            id: "root-1".to_string(),
            text: "hello".to_string(),
            sender_name: "Alice".to_string(),
            sender_id: "agent-1".to_string(),
            timestamp: 1710000000000,
            channel: "general".to_string(),
            thread_root: None,
            broadcast: false,
            reply_count: 0,
            reactions: HashMap::new(),
        };
        let reply = ChatMessage {
            id: "reply-1".to_string(),
            text: "thread reply".to_string(),
            sender_name: "Bob".to_string(),
            sender_id: "agent-2".to_string(),
            timestamp: 1710000000100,
            channel: "general".to_string(),
            thread_root: Some("root-1".to_string()),
            broadcast: false,
            reply_count: 0,
            reactions: HashMap::new(),
        };

        tokio_test::block_on(async {
            append_channel_history("group-1", "general", &root).await;
            append_thread_history("group-1", "root-1", &reply).await;

            let channel_history = load_channel_history("group-1", "general").await;
            let thread_history = load_thread_history("group-1", "root-1").await;

            assert_eq!(channel_history.len(), 1);
            assert_eq!(channel_history[0].id, "root-1");
            assert_eq!(channel_history[0].reply_count, 1);
            assert_eq!(thread_history.len(), 1);
            assert_eq!(thread_history[0].id, "reply-1");
        });

        unsafe {
            std::env::remove_var("COMMUNITAS_X0X_HISTORY_DIR");
        }
    }
}
