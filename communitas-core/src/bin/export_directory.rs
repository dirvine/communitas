#![allow(clippy::print_stdout)] // CLI tool outputs JSON to stdout

use anyhow::{Context, Result};
use communitas_bindings::{
    app::CommunitasApp,
    command::{Query, QueryResponse},
};
use serde::Serialize;
use serde_json::json;
use std::env;

#[derive(Serialize)]
struct Snapshot {
    profile: Profile,
    entities: Vec<EntitySummary>,
    contacts: Vec<ContactSummary>,
}

#[derive(Serialize)]
struct Profile {
    four_words: String,
    display_name: String,
    device_name: String,
    device_type: String,
}

#[derive(Serialize)]
struct EntitySummary {
    id: String,
    name: String,
    entity_type: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct ContactSummary {
    id: String,
    display_name: String,
    is_online: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let four_words = args.next().context(
        "usage: export_directory <four_words> <display_name> <storage_dir> [device_name]",
    )?;
    let display_name = args.next().context("missing <display_name> argument")?;
    let storage_dir = args.next().context("missing <storage_dir> argument")?;
    let device_name = args.next().unwrap_or_else(|| "parity-cli".to_string());

    let app = CommunitasApp::new(
        four_words.clone(),
        display_name.clone(),
        device_name,
        storage_dir.clone(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to initialize CommunitasApp: {e}"))?;

    let profile = match app.query(Query::GetProfile).await? {
        QueryResponse::Profile {
            four_words,
            display_name,
            device_name,
            device_type,
        } => Profile {
            four_words,
            display_name,
            device_name,
            device_type,
        },
        other => anyhow::bail!("unexpected response for profile: {:?}", other),
    };

    let entities = match app.query(Query::ListEntities).await? {
        QueryResponse::EntityList(items) => items
            .into_iter()
            .map(|entity| EntitySummary {
                id: entity.id,
                name: entity.name,
                entity_type: format!("{:?}", entity.entity_type),
                description: entity.description,
            })
            .collect(),
        other => anyhow::bail!("unexpected response for entity list: {:?}", other),
    };

    let contacts = match app.query(Query::ListContacts).await? {
        QueryResponse::ContactList(items) => items
            .into_iter()
            .map(|contact| ContactSummary {
                id: contact.id,
                display_name: contact.display_name,
                is_online: contact.is_online,
            })
            .collect(),
        other => anyhow::bail!("unexpected response for contact list: {:?}", other),
    };

    let snapshot = Snapshot {
        profile,
        entities,
        contacts,
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&json!(snapshot)).context("failed to serialize snapshot")?
    );

    Ok(())
}
