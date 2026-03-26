#![allow(non_snake_case)]
#![allow(clippy::print_stderr)] // eprintln! used for bootstrap errors before logger init

mod components;
pub mod contrast;
pub mod design_tokens;
pub mod hooks;
pub mod models;
pub mod onboarding;
#[allow(dead_code, unused_imports, unused_variables)]
pub mod pages;
mod platform;
pub mod styles;
pub mod styles_v2;
pub mod tokens;
pub mod version;
mod x0x_contract;

use communitas_ui_service::UiServices;
use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use native_dialog::{MessageDialog, MessageType};
use std::sync::{Arc, OnceLock};
use tracing::{error, info};

use communitas_ui_service::auth::AuthSession;
use components::{ContactEntry, GroupEntry};
use tokens::colors;

static UI_SERVICES: OnceLock<Arc<UiServices>> = OnceLock::new();

// ── Legacy compat layer (used by pages.rs, kept to avoid rewriting it) ──────

/// Auth state kept for backward compatibility with pages.rs.
#[derive(Clone, PartialEq)]
#[allow(dead_code)]
struct AuthState {
    phase: AuthPhase,
    session: Option<AuthSession>,
    error: Option<String>,
    pending_mnemonic: Option<String>,
    local_x0x_bypass: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum AuthPhase {
    LoggedOut,
    Authenticating,
    PendingMnemonic,
    Authenticated,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            phase: AuthPhase::Authenticated,
            session: None,
            error: None,
            pending_mnemonic: None,
            local_x0x_bypass: true,
        }
    }
}

#[allow(dead_code)]
impl AuthState {
    fn is_authenticated(&self) -> bool {
        matches!(self.phase, AuthPhase::Authenticated)
    }
}

#[allow(dead_code)]
fn use_auth() -> Signal<AuthState> {
    use_context::<Signal<AuthState>>()
}

// ── Routes ──────────────────────────────────────────────────────────────────

#[derive(Clone, Routable, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/space/:space_id")]
    SpaceView { space_id: String },
    #[route("/space/:space_id/:tab")]
    SpaceTab { space_id: String, tab: String },
    #[route("/dm/:agent_id")]
    DirectMessage { agent_id: String },
    #[route("/people")]
    People {},
    #[route("/network")]
    Network {},
    #[route("/settings")]
    Settings {},
    // Legacy routes kept for backward compatibility with pages.rs
    #[route("/login")]
    LoginRoute {},
    #[route("/create")]
    CreateIdentityRoute {},
    #[route("/recover")]
    RecoverIdentityRoute {},
    #[route("/login-other")]
    LoginOtherRoute {},
    #[route("/dashboard-legacy")]
    DashboardRoute {},
    #[route("/messages")]
    MessagesRoute {},
    #[route("/channels")]
    ChannelsRoute {},
    #[route("/projects")]
    ProjectsRoute {},
    #[route("/contacts")]
    ContactsRoute {},
    #[route("/network-legacy")]
    NetworkRoute {},
    #[route("/more")]
    MoreRoute {},
    #[route("/entity/:entity_type/:entity_id")]
    EntityDetailRoute {
        entity_type: String,
        entity_id: String,
    },
    #[route("/entity/:entity_type/:entity_id/chat")]
    EntityChatRoute {
        entity_type: String,
        entity_id: String,
    },
    #[route("/entity/:entity_type/:entity_id/drive")]
    EntityDriveRoute {
        entity_type: String,
        entity_id: String,
    },
    #[route("/project/:project_id/board")]
    ProjectBoardRoute { project_id: String },
    #[route("/contact/:contact_id")]
    ContactDetailRoute { contact_id: String },
    #[route("/contact/:contact_id/chat")]
    ContactChatRoute { contact_id: String },
}

// ── Entry point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    if let Err(err) = dioxus_logger::init(Level::INFO) {
        eprintln!("failed to init logger: {err}");
    }

    // Check WebView availability before anything else
    if let Err(err) = platform::check_webview_available() {
        eprintln!("WebView not available: {err}");
        let _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Communitas - Missing Dependency")
            .set_text(&format!(
                "Communitas cannot start because a required component is missing.\n\n{err}"
            ))
            .show_alert();
        std::process::exit(1);
    }

    let services = UiServices::bootstrap_async().await.unwrap_or_else(|err| {
        eprintln!("failed to initialize UI services: {err}");
        std::process::exit(1);
    });
    if UI_SERVICES.set(Arc::new(services)).is_err() {
        eprintln!("UI services already initialized");
        std::process::exit(1);
    }

    info!("starting Communitas (Deep Space Operations Console)");
    dioxus::launch(App);
}

// ── Global CSS ──────────────────────────────────────────────────────────────

const GLOBAL_STYLES: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }
html, body { height: 100%; overflow: hidden; }
body { background-color: #0a0c14; color: #e4e6f0; font-family: system-ui, -apple-system, sans-serif; }
#main { height: 100vh; display: flex; flex-direction: column; }

@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
}

.skeleton {
    background: linear-gradient(90deg, rgba(0,212,255,0.03) 0%, rgba(0,212,255,0.08) 50%, rgba(0,212,255,0.03) 100%);
    background-size: 200% 100%;
    animation: shimmer 1.5s infinite ease-in-out;
    border-radius: 0.375rem;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

.tab-content { animation: fadeIn 200ms ease-out; }

:focus-visible {
    outline: 2px solid #00d4ff;
    outline-offset: 2px;
}

* { scroll-behavior: smooth; }

::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: #252940; border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: #353a52; }
"#;

// ── App component ───────────────────────────────────────────────────────────

#[component]
fn App() -> Element {
    let services = match UI_SERVICES.get().cloned() {
        Some(svc) => svc,
        None => {
            error!("UI services not initialized");
            std::process::exit(1);
        }
    };
    use_context_provider(|| services);
    use_context_provider(components::AnnouncerContext::new);

    rsx! {
        style { {GLOBAL_STYLES} }
        components::Announcer {}
        Router::<Route> {}
    }
}

// ── App shell (sidebar + content + status bar) ──────────────────────────────

/// Shared layout: sidebar | content | status bar.
#[component]
fn AppShell(children: Element) -> Element {
    let navigator = use_navigator();
    let route: Route = use_route();

    // Gather sidebar data from x0x daemon
    let mut groups = use_signal(Vec::<GroupEntry>::new);
    let mut contacts = use_signal(Vec::<ContactEntry>::new);
    let mut agent_id = use_signal(|| None::<String>);
    let mut connected = use_signal(|| false);

    // Poll groups, contacts, agent
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        loop {
            match client.health().await {
                Ok(_) => connected.set(true),
                Err(_) => connected.set(false),
            }

            if let Ok(agent) = client.agent().await {
                agent_id.set(Some(agent.agent_id));
            }

            if let Ok(group_list) = client.list_groups().await {
                let entries: Vec<GroupEntry> = group_list
                    .into_iter()
                    .map(|g| GroupEntry {
                        id: g.group_id,
                        name: g.name,
                        member_count: g.member_count.unwrap_or(0),
                    })
                    .collect();
                groups.set(entries);
            }

            if let Ok(contact_list) = client.list_contacts().await {
                let entries: Vec<ContactEntry> = contact_list
                    .into_iter()
                    .map(|c| ContactEntry {
                        agent_id: c.agent_id.clone(),
                        label: c.label.unwrap_or_else(|| {
                            if c.agent_id.len() > 12 {
                                format!("{}..{}", &c.agent_id[..6], &c.agent_id[c.agent_id.len() - 4..])
                            } else {
                                c.agent_id.clone()
                            }
                        }),
                        online: false, // presence will be wired later
                    })
                    .collect();
                contacts.set(entries);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(8)).await;
        }
    });

    let current_path = match &route {
        Route::Dashboard {} => "/".to_string(),
        Route::SpaceView { space_id } => format!("/space/{space_id}"),
        Route::SpaceTab { space_id, tab } => format!("/space/{space_id}/{tab}"),
        Route::DirectMessage { agent_id } => format!("/dm/{agent_id}"),
        Route::People {} => "/people".to_string(),
        Route::Network {} => "/network".to_string(),
        Route::Settings {} => "/settings".to_string(),
        // Legacy routes all map to "/"
        _ => "/".to_string(),
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; overflow: hidden;",

            // Main area: sidebar + content
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                components::AppSidebar {
                    current_path: current_path,
                    groups: groups().clone(),
                    contacts: contacts().clone(),
                    agent_id: agent_id().clone(),
                    connected: connected(),
                    on_navigate: move |path: String| {
                        let route = match path.as_str() {
                            "/" => Route::Dashboard {},
                            "/people" => Route::People {},
                            "/network" => Route::Network {},
                            "/settings" => Route::Settings {},
                            other => {
                                if let Some(space_id) = other.strip_prefix("/space/") {
                                    Route::SpaceView {
                                        space_id: space_id.to_string(),
                                    }
                                } else if let Some(agent_id) = other.strip_prefix("/dm/") {
                                    Route::DirectMessage {
                                        agent_id: agent_id.to_string(),
                                    }
                                } else {
                                    Route::Dashboard {}
                                }
                            }
                        };
                        navigator.push(route);
                    },
                }

                // Content area
                div {
                    style: format!(
                        "flex: 1; overflow: hidden; background-color: {};",
                        colors::SURFACE_BG,
                    ),
                    {children}
                }
            }

            // Status bar
            components::StatusBar {}
        }
    }
}

// ── Route components ────────────────────────────────────────────────────────

#[component]
fn Dashboard() -> Element {
    rsx! {
        AppShell {
            components::Dashboard {}
        }
    }
}

#[component]
fn SpaceView(space_id: String) -> Element {
    rsx! {
        AppShell {
            components::SpaceView {
                space_id: space_id,
            }
        }
    }
}

#[component]
fn SpaceTab(space_id: String, tab: String) -> Element {
    rsx! {
        AppShell {
            components::SpaceView {
                space_id: space_id,
                initial_tab: Some(tab),
            }
        }
    }
}

#[component]
fn DirectMessage(agent_id: String) -> Element {
    rsx! {
        AppShell {
            components::DmView { agent_id: agent_id }
        }
    }
}

#[component]
fn People() -> Element {
    rsx! {
        AppShell {
            components::PeopleView {}
        }
    }
}

#[component]
fn Network() -> Element {
    rsx! {
        AppShell {
            components::NetworkView {}
        }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        AppShell {
            components::SettingsView {}
        }
    }
}

// ── Legacy route components (redirect to new routes) ────────────────────────

#[component]
fn LoginRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn CreateIdentityRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn RecoverIdentityRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn LoginOtherRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn DashboardRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn MessagesRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn ChannelsRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn ProjectsRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn ContactsRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::People {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn NetworkRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Network {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn MoreRoute() -> Element {
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Settings {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn EntityDetailRoute(entity_type: String, entity_id: String) -> Element {
    let _ = (entity_type, entity_id);
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn EntityChatRoute(entity_type: String, entity_id: String) -> Element {
    let _ = (entity_type, entity_id);
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn EntityDriveRoute(entity_type: String, entity_id: String) -> Element {
    let _ = (entity_type, entity_id);
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn ProjectBoardRoute(project_id: String) -> Element {
    let _ = project_id;
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::Dashboard {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn ContactDetailRoute(contact_id: String) -> Element {
    let _ = contact_id;
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::People {}); });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn ContactChatRoute(contact_id: String) -> Element {
    let _ = contact_id;
    let nav = use_navigator();
    use_effect(move || { nav.replace(Route::People {}); });
    rsx! { div { "Redirecting..." } }
}

// ── Channel creation helper ─────────────────────────────────────────────────

pub(crate) async fn create_channel(
    group_id: &str,
    raw_name: &str,
    raw_description: &str,
) -> Result<models::channel::ChannelMeta, String> {
    let channel_name = x0x_contract::normalize_channel_name(raw_name);
    if channel_name.is_empty() {
        return Err("Channel name must contain letters, numbers, or dashes.".to_string());
    }

    let client = X0xClient::new();
    let group = client
        .get_group(group_id)
        .await
        .map_err(|err| format!("Failed to load space details: {err}"))?;

    let store_id = x0x_contract::channel_store_id(group_id);
    let stores = client
        .list_stores()
        .await
        .map_err(|err| format!("Failed to list x0x stores: {err}"))?;

    if !stores.iter().any(|store| store.id == store_id) {
        client
            .create_store("Channels", &store_id)
            .await
            .map_err(|err| format!("Failed to create channel store: {err}"))?;
    }

    let mut channels = x0x_contract::load_group_channels(&client, &group).await;

    if channels.iter().any(|channel| channel.name == channel_name) {
        return Err(format!("Channel #{channel_name} already exists."));
    }

    let agent = client
        .agent()
        .await
        .map_err(|err| format!("Failed to load agent identity: {err}"))?;
    let new_channel = models::channel::ChannelMeta {
        name: channel_name.clone(),
        description: raw_description.trim().to_string(),
        creator: agent.agent_id,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0),
        topic: x0x_contract::channel_topic(group_id, &channel_name),
    };

    channels.push(new_channel.clone());
    x0x_contract::sort_channels(&mut channels);

    let bytes = x0x_contract::serialize_channels_index(&channels)
        .map_err(|err| format!("Failed to encode channel metadata: {err}"))?;
    client
        .put(
            &store_id,
            x0x_contract::CHANNELS_INDEX_KEY,
            &bytes,
            Some("application/json"),
        )
        .await
        .map_err(|err| format!("Failed to write channel metadata: {err}"))?;

    Ok(new_channel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_dashboard_is_root() {
        let route = Route::Dashboard {};
        assert_eq!(format!("{route:?}"), "Dashboard");
    }

    #[test]
    fn route_space_view_parses() {
        let route = Route::SpaceView {
            space_id: "abc123".into(),
        };
        assert_eq!(
            format!("{route:?}"),
            "SpaceView { space_id: \"abc123\" }"
        );
    }

    #[test]
    fn route_people_is_slash_people() {
        let route = Route::People {};
        assert_eq!(format!("{route:?}"), "People");
    }
}
