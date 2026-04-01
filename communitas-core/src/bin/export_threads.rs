// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export thread list as JSON for MCP parity testing.
//!
//! Outputs a JSON snapshot of conversation threads that matches the format
//! returned by the `list_threads` MCP tool.
//!
//! Usage: `export_threads <four_words> <display_name> <storage_dir> [filter]`
//!
//! Filter options: all (default), unread, entities, contacts

#![allow(clippy::print_stdout)] // CLI tool outputs JSON to stdout

use anyhow::{Context, Result};
use communitas_bindings::{
    app::CommunitasApp,
    command::{Query, QueryResponse},
};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct ThreadSnapshot {
    threads: Vec<ThreadSummary>,
    total_count: usize,
    filter: String,
}

#[derive(Serialize)]
struct ThreadSummary {
    thread_id: String,
    entity_id: Option<String>,
    entity_type: Option<String>,
    contact_id: Option<String>,
    display_name: String,
    last_message_preview: String,
    last_message_timestamp: u64,
    unread_count: u32,
    is_muted: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let four_words = args
        .next()
        .context("usage: export_threads <four_words> <display_name> <storage_dir> [filter]")?;
    let display_name = args.next().context("missing <display_name> argument")?;
    let storage_dir = args.next().context("missing <storage_dir> argument")?;
    let filter = args.next().unwrap_or_else(|| "all".to_string());

    let app = CommunitasApp::new(
        four_words,
        display_name,
        "parity-cli".to_string(),
        storage_dir,
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to initialize CommunitasApp: {e}"))?;

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut threads: Vec<ThreadSummary> = Vec::new();

    // Get entity threads if filter allows
    if (filter == "all" || filter == "entities" || filter == "unread")
        && let Ok(QueryResponse::EntityList(entities)) = app.query(Query::ListEntities).await
    {
        for entity in entities {
            let entity_type_str = match entity.entity_type.as_str() {
                "Organisation" => "organisation",
                "Project" => "project",
                "Group" => "group",
                "Channel" => "channel",
                other => other,
            };

            threads.push(ThreadSummary {
                thread_id: format!("entity:{}", entity.id),
                entity_id: Some(entity.id),
                entity_type: Some(entity_type_str.to_string()),
                contact_id: None,
                display_name: entity.name,
                last_message_preview: String::new(),
                last_message_timestamp: now_ms.saturating_sub(3_600_000), // Placeholder
                unread_count: 0,
                is_muted: false,
            });
        }
    }

    // Get contact threads (DMs) if filter allows
    if (filter == "all" || filter == "contacts" || filter == "unread")
        && let Ok(QueryResponse::ContactList(contacts)) = app.query(Query::ListContacts).await
    {
        for contact in contacts {
            threads.push(ThreadSummary {
                thread_id: format!("contact:{}", contact.id),
                entity_id: None,
                entity_type: None,
                contact_id: Some(contact.id),
                display_name: contact.display_name,
                last_message_preview: String::new(),
                last_message_timestamp: now_ms.saturating_sub(7_200_000), // Placeholder
                unread_count: 0,
                is_muted: false,
            });
        }
    }

    // Sort by timestamp descending
    threads.sort_by(|a, b| b.last_message_timestamp.cmp(&a.last_message_timestamp));

    let total_count = threads.len();
    let snapshot = ThreadSnapshot {
        threads,
        total_count,
        filter,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&snapshot).context("failed to serialize snapshot")?
    );

    Ok(())
}
