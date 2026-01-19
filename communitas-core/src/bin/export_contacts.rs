//! Export contact list with presence as JSON for MCP parity testing.
//!
//! Outputs a JSON snapshot of contacts with presence info that matches
//! the format returned by the `list_contacts` MCP tool.
//!
//! Usage: export_contacts <four_words> <display_name> <storage_dir> [filter] [--no-presence]
//! Filter options: all (default), online, favorites

#![allow(clippy::print_stdout)] // CLI tool outputs JSON to stdout

use anyhow::{Context, Result};
use communitas_bindings::{
    app::CommunitasApp,
    command::{Query, QueryResponse},
};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct ContactSnapshot {
    contacts: Vec<ContactWithPresence>,
    count: usize,
    filter: String,
    include_presence: bool,
}

#[derive(Serialize)]
struct ContactWithPresence {
    id: String,
    display_name: String,
    four_words: String,
    is_favourite: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_online: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_seen: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_status: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 3 {
        anyhow::bail!(
            "usage: export_contacts <four_words> <display_name> <storage_dir> [filter] [--no-presence]"
        );
    }

    let four_words = args[0].clone();
    let display_name = args[1].clone();
    let storage_dir = args[2].clone();

    // Parse optional arguments
    let mut filter = "all".to_string();
    let mut include_presence = true;

    for arg in args.iter().skip(3) {
        if arg == "--no-presence" {
            include_presence = false;
        } else if !arg.starts_with('-') {
            filter = arg.clone();
        }
    }

    let app = CommunitasApp::new(
        four_words,
        display_name,
        "parity-cli".to_string(),
        storage_dir,
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to initialize CommunitasApp: {e}"))?;

    // Choose query based on filter
    let query = match filter.as_str() {
        "favorites" => Query::ListFavouriteContacts,
        _ => Query::ListContacts,
    };

    let contacts = match app.query(query).await {
        Ok(QueryResponse::ContactList(items)) => items,
        Ok(other) => anyhow::bail!("unexpected response for contact list: {:?}", other),
        Err(e) => anyhow::bail!("failed to query contacts: {}", e.message),
    };

    // Apply online filter if needed
    let filtered: Vec<_> = if filter == "online" {
        contacts.into_iter().filter(|c| c.is_online).collect()
    } else {
        contacts
    };

    let contacts_with_presence: Vec<ContactWithPresence> = filtered
        .into_iter()
        .map(|c| {
            let (is_online, last_seen, presence_status) = if include_presence {
                let status = if c.is_online { "online" } else { "offline" };
                (Some(c.is_online), c.last_seen, Some(status.to_string()))
            } else {
                (None, None, None)
            };

            ContactWithPresence {
                id: c.id,
                display_name: c.display_name,
                four_words: c.four_words.unwrap_or_default(),
                is_favourite: c.is_favourite,
                is_online,
                last_seen,
                presence_status,
            }
        })
        .collect();

    let count = contacts_with_presence.len();
    let snapshot = ContactSnapshot {
        contacts: contacts_with_presence,
        count,
        filter,
        include_presence,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&snapshot).context("failed to serialize snapshot")?
    );

    Ok(())
}
