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
use components::{ContactEntry, DetailContent, GroupEntry, SpaceModalTab};
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
    #[route("/about")]
    About {},
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
    // Create the toast signal BEFORE the context provider to avoid hook-in-hook.
    let toast_signal = use_signal(Vec::new);
    use_context_provider(|| components::toast_system::ToastManager::with_signal(toast_signal));
    use_context_provider(components::AnnouncerContext::new);

    rsx! {
        style { {GLOBAL_STYLES} }
        components::Announcer {}
        components::toast_system::ToastContainer {}
        components::OnboardingGate {
            Router::<Route> {}
        }
    }
}

// ── App shell (sidebar + content + status bar) ──────────────────────────────

/// Shared layout: sidebar | content | detail panel | status bar.
#[component]
fn AppShell(children: Element) -> Element {
    let navigator = use_navigator();
    let route: Route = use_route();

    // Detail panel state
    let mut detail_content = use_signal(|| DetailContent::None);

    // Create/join space modal state
    let mut show_space_modal = use_signal(|| false);
    let mut space_modal_tab = use_signal(|| SpaceModalTab::Create);
    let mut space_name = use_signal(String::new);
    let mut space_description = use_signal(String::new);
    let mut space_invite_link = use_signal(String::new);
    let mut space_display_name = use_signal(String::new);
    let mut space_submitting = use_signal(|| false);
    let mut space_error = use_signal(|| None::<String>);

    // Gather sidebar data from x0x daemon
    let mut groups = use_signal(Vec::<GroupEntry>::new);
    let mut contacts = use_signal(Vec::<ContactEntry>::new);
    let mut agent_id = use_signal(|| None::<String>);
    let mut identity_label = use_signal(|| "Local x0x".to_string());
    let mut identity_secondary = use_signal(|| None::<String>);
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
                let short_agent = x0x_contract::fallback_sender_name(&agent.agent_id);
                let secondary = format!(
                    "agent:{} · machine:{}",
                    short_agent,
                    x0x_contract::fallback_sender_name(&agent.machine_id)
                );
                let fallback_label = agent
                    .user_id
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| format!("agent:{short_agent}"));

                agent_id.set(Some(agent.agent_id.clone()));
                identity_secondary.set(Some(secondary));

                match client.agent_card(None, Some(false)).await {
                    Ok(card_resp) if !card_resp.card.display_name.trim().is_empty() => {
                        identity_label.set(card_resp.card.display_name.trim().to_string());
                    }
                    Ok(_) => identity_label.set(fallback_label),
                    Err(_) => identity_label.set(fallback_label),
                }
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
                        label: c
                            .label
                            .unwrap_or_else(|| x0x_contract::fallback_sender_name(&c.agent_id)),
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
        Route::About {} => "/about".to_string(),
        Route::MoreRoute {} => "/more".to_string(),
        // Legacy routes all map to "/"
        _ => "/".to_string(),
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; overflow: hidden;",

            // Main area: sidebar + content + detail panel
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                components::AppSidebar {
                    current_path: current_path,
                    groups: groups().clone(),
                    contacts: contacts().clone(),
                    agent_id: agent_id().clone(),
                    connected: connected(),
                    identity_label: Some(identity_label()),
                    identity_secondary: identity_secondary(),
                    on_identity_click: move |_| {
                        navigator.push(Route::MoreRoute {});
                    },
                    on_navigate: move |path: String| {
                        let route = match path.as_str() {
                            "/" => Route::Dashboard {},
                            "/people" => Route::People {},
                            "/network" => Route::Network {},
                            "/settings" => Route::Settings {},
                            "/about" => Route::About {},
                            other => {
                                if let Some(rest) = other.strip_prefix("/space/") {
                                    // Could be "/space/{id}" or "/space/{id}/{tab}"
                                    let mut parts = rest.splitn(2, '/');
                                    let space_id = parts.next().unwrap_or("").to_string();
                                    match parts.next() {
                                        Some(tab) if !tab.is_empty() => Route::SpaceTab {
                                            space_id,
                                            tab: tab.to_string(),
                                        },
                                        _ => Route::SpaceView { space_id },
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
                    on_contact_click: move |aid: String| {
                        detail_content.set(DetailContent::AgentProfile { agent_id: aid });
                    },
                    on_create_space: move |_: ()| {
                        show_space_modal.set(true);
                        space_modal_tab.set(SpaceModalTab::Create);
                        space_name.set(String::new());
                        space_description.set(String::new());
                        space_invite_link.set(String::new());
                        space_display_name.set(String::new());
                        space_submitting.set(false);
                        space_error.set(None);
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

                // Detail panel (right column)
                components::DetailPanel {
                    content: detail_content,
                }
            }

            // Status bar
            components::StatusBar {}
        }

        // Create/join space modal
        if show_space_modal() {
            components::CreateSpaceModal {
                active_tab: space_modal_tab(),
                space_name: space_name(),
                space_description: space_description(),
                invite_link: space_invite_link(),
                display_name: space_display_name(),
                submitting: space_submitting(),
                error: space_error(),
                on_tab_change: move |tab: SpaceModalTab| space_modal_tab.set(tab),
                on_name_change: move |val: String| space_name.set(val),
                on_description_change: move |val: String| space_description.set(val),
                on_invite_change: move |val: String| space_invite_link.set(val),
                on_display_name_change: move |val: String| space_display_name.set(val),
                on_cancel: move |_| {
                    if !space_submitting() {
                        show_space_modal.set(false);
                    }
                },
                on_create: move |_| {
                    let name = space_name().trim().to_string();
                    if name.is_empty() {
                        space_error.set(Some("Space name is required.".to_string()));
                        return;
                    }
                    let desc = space_description().trim().to_string();
                    let display = space_display_name().trim().to_string();
                    space_submitting.set(true);
                    space_error.set(None);
                    spawn(async move {
                        let client = communitas_x0x_client::X0xClient::new();
                        let desc_opt = if desc.is_empty() { None } else { Some(desc.as_str()) };
                        let display_opt = if display.is_empty() { None } else { Some(display.as_str()) };
                        match client.create_group(&name, desc_opt, display_opt).await {
                            Ok(created) => {
                                info!(target: "ui.app_shell", "Created space {} ({})", created.name, created.group_id);
                                show_space_modal.set(false);
                                navigator.push(Route::SpaceView { space_id: created.group_id });
                            }
                            Err(e) => {
                                space_error.set(Some(format!("Failed to create space: {e}")));
                            }
                        }
                        space_submitting.set(false);
                    });
                },
                on_join: move |_| {
                    let invite = space_invite_link().trim().to_string();
                    if invite.is_empty() {
                        space_error.set(Some("Invite link is required.".to_string()));
                        return;
                    }
                    let display = space_display_name().trim().to_string();
                    space_submitting.set(true);
                    space_error.set(None);
                    spawn(async move {
                        let client = communitas_x0x_client::X0xClient::new();
                        let display_opt = if display.is_empty() { None } else { Some(display.as_str()) };
                        match client.join_group(&invite, display_opt).await {
                            Ok(joined) => {
                                info!(target: "ui.app_shell", "Joined space {} ({})", joined.group_name, joined.group_id);
                                show_space_modal.set(false);
                                navigator.push(Route::SpaceView { space_id: joined.group_id });
                            }
                            Err(e) => {
                                space_error.set(Some(format!("Failed to join space: {e}")));
                            }
                        }
                        space_submitting.set(false);
                    });
                },
            }
        }
    }
}

// ── Route components ────────────────────────────────────────────────────────

#[component]
fn Dashboard() -> Element {
    let navigator = use_navigator();
    let mut first_space_id = use_signal(|| None::<String>);
    let mut loading_spaces = use_signal(|| true);
    let mut load_error = use_signal(|| None::<String>);

    use_future(move || async move {
        let client = X0xClient::new();
        match client.list_groups().await {
            Ok(mut groups) => {
                groups.sort_by(|left, right| left.name.cmp(&right.name));
                first_space_id.set(groups.into_iter().next().map(|group| group.group_id));
                load_error.set(None);
            }
            Err(err) => {
                load_error.set(Some(format!("Failed to load spaces: {err}")));
            }
        }
        loading_spaces.set(false);
    });

    use_effect(move || {
        if let Some(space_id) = first_space_id() {
            navigator.replace(Route::SpaceView { space_id });
        }
    });

    rsx! {
        AppShell {
            if loading_spaces() {
                div {
                    style: format!(
                        "display: flex; height: 100%; align-items: center; justify-content: center; color: {};",
                        colors::TEXT_MUTED,
                    ),
                    "Opening your local x0x workspace..."
                }
            } else if first_space_id().is_none() {
                div {
                    style: "display: flex; flex-direction: column; height: 100%;",
                    if let Some(err) = load_error() {
                        div {
                            style: format!(
                                "margin: 24px 24px 0; padding: 12px 14px; border-radius: 12px; background: rgba(255, 68, 102, 0.08); border: 1px solid rgba(255, 68, 102, 0.3); color: {};",
                                colors::DANGER,
                            ),
                            "{err}"
                        }
                    }
                    components::Dashboard {}
                }
            } else {
                div {
                    style: format!(
                        "display: flex; height: 100%; align-items: center; justify-content: center; color: {};",
                        colors::TEXT_MUTED,
                    ),
                    "Opening your local x0x workspace..."
                }
            }
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

#[component]
fn About() -> Element {
    use tokens::typography;
    rsx! {
        AppShell {
            div {
                style: format!(
                    "display: flex; flex-direction: column; align-items: center; \
                     justify-content: center; height: 100%; gap: 1rem; color: {};",
                    colors::TEXT_SECONDARY,
                ),
                div {
                    style: format!(
                        "font-size: 1.125rem; font-weight: 600; color: {};",
                        colors::TEXT_PRIMARY,
                    ),
                    "Communitas"
                }
                div {
                    style: format!("font-size: 0.875rem; color: {};", colors::TEXT_MUTED),
                    "Local-first, PQC-ready collaboration platform"
                }
                div {
                    style: format!(
                        "font-size: 0.75rem; font-family: {}; color: {};",
                        typography::FONT_MONO,
                        colors::TEXT_MUTED,
                    ),
                    {format!("v{}", env!("CARGO_PKG_VERSION"))}
                }
            }
        }
    }
}

// ── Legacy route components (redirect to new routes) ────────────────────────

#[component]
fn RouteRedirect(to: Route) -> Element {
    let nav = use_navigator();
    use_effect(move || {
        nav.replace(to.clone());
    });
    rsx! { div { "Redirecting..." } }
}

#[component]
fn LoginRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn CreateIdentityRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn RecoverIdentityRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn LoginOtherRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn DashboardRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn MessagesRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn ChannelsRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn ProjectsRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn ContactsRoute() -> Element {
    rsx! { RouteRedirect { to: Route::People {} } }
}

#[component]
fn NetworkRoute() -> Element {
    rsx! { RouteRedirect { to: Route::Network {} } }
}

#[component]
fn MoreRoute() -> Element {
    rsx! {
        AppShell {
            components::LocalX0xProfileView {}
        }
    }
}

#[component]
fn EntityDetailRoute(entity_type: String, entity_id: String) -> Element {
    let _ = (entity_type, entity_id);
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn EntityChatRoute(entity_type: String, entity_id: String) -> Element {
    let _ = (entity_type, entity_id);
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn EntityDriveRoute(entity_type: String, entity_id: String) -> Element {
    let _ = (entity_type, entity_id);
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn ProjectBoardRoute(project_id: String) -> Element {
    let _ = project_id;
    rsx! { RouteRedirect { to: Route::Dashboard {} } }
}

#[component]
fn ContactDetailRoute(contact_id: String) -> Element {
    let _ = contact_id;
    rsx! { RouteRedirect { to: Route::People {} } }
}

#[component]
fn ContactChatRoute(contact_id: String) -> Element {
    let _ = contact_id;
    rsx! { RouteRedirect { to: Route::People {} } }
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
        assert_eq!(format!("{route:?}"), "SpaceView { space_id: \"abc123\" }");
    }

    #[test]
    fn route_people_is_slash_people() {
        let route = Route::People {};
        assert_eq!(format!("{route:?}"), "People");
    }
}
