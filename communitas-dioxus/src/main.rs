#![allow(non_snake_case)]
#![allow(clippy::print_stderr)] // eprintln! used for bootstrap errors before logger init

use communitas_core::generate_id_words;
use communitas_ui_api::{OrganizationCategory, SampleWords, UnifiedEntity, UnifiedEntityType};
use communitas_ui_service::{
    UiServices,
    auth::{AuthController, AuthService, AuthSession},
    directory::DirectorySnapshot,
    navigation::{EntityNavigationKey, NavigationService, NavigationStateSnapshot},
};
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use futures::StreamExt;
use std::{
    borrow::Cow,
    sync::{Arc, OnceLock},
};
use tracing::{error, info};

static UI_SERVICES: OnceLock<Arc<UiServices>> = OnceLock::new();

#[allow(clippy::expect_used)] // OnceLock guaranteed initialized in main() before UI renders
fn ui_services() -> Arc<UiServices> {
    UI_SERVICES
        .get()
        .expect("UI services not initialized")
        .clone()
}

fn main() {
    if let Err(err) = dioxus_logger::init(Level::INFO) {
        eprintln!("failed to init logger: {err}");
    }
    let services = UiServices::bootstrap().unwrap_or_else(|err| {
        eprintln!("failed to initialize UI services: {err}");
        std::process::exit(1);
    });
    if UI_SERVICES.set(Arc::new(services)).is_err() {
        eprintln!("UI services already initialized");
        std::process::exit(1);
    }
    info!("starting Communitas Dioxus prototype");
    dioxus::launch(App);
}

#[derive(Clone, Routable, Debug, PartialEq)]
#[allow(clippy::enum_variant_names)] // Dioxus router convention uses Route suffix
enum Route {
    #[route("/login")]
    LoginRoute {},
    #[route("/create")]
    CreateIdentityRoute {},
    #[route("/recover")]
    RecoverIdentityRoute {},
    #[route("/")]
    DashboardRoute {},
    #[route("/messages")]
    MessagesRoute {},
    #[route("/projects")]
    ProjectsRoute {},
    #[route("/contacts")]
    ContactsRoute {},
    #[route("/network")]
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
    #[route("/contact/:contact_id/chat")]
    ContactChatRoute { contact_id: String },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthPhase {
    LoggedOut,
    Authenticating,
    Authenticated,
}

#[derive(Clone)]
struct AuthState {
    phase: AuthPhase,
    session: Option<AuthSession>,
    error: Option<String>,
}

impl AuthState {
    fn is_authenticated(&self) -> bool {
        matches!(self.phase, AuthPhase::Authenticated)
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            phase: AuthPhase::LoggedOut,
            session: None,
            error: None,
        }
    }
}

impl PartialEq for AuthState {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase && self.error == other.error
    }
}

fn use_auth() -> Signal<AuthState> {
    use_context::<Signal<AuthState>>()
}

fn use_navigation_snapshot() -> Signal<NavigationStateSnapshot> {
    let services = use_context::<Arc<UiServices>>();
    let snapshot = use_signal(|| services.navigation().current_snapshot());
    let mut nav_signal = snapshot;
    use_future(move || {
        let mut rx = services.navigation().subscribe();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                nav_signal.set(rx.borrow().clone());
            }
        }
    });
    snapshot
}

fn use_directory_snapshot() -> Signal<DirectorySnapshot> {
    let services = use_context::<Arc<UiServices>>();
    let snapshot = use_signal(|| services.directory().current_snapshot());
    let mut dir_signal = snapshot;
    use_future(move || {
        let mut rx = services.directory().subscribe();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                dir_signal.set(rx.borrow().clone());
            }
        }
    });
    snapshot
}

#[component]
fn App() -> Element {
    let services = ui_services();
    let services_clone = services.clone();
    use_context_provider(|| services_clone);
    use_context_provider(|| Signal::new(AuthState::default()));
    rsx! {
        AppLifecycleManager {}
        RouteObserver {}
        Router::<Route> {}
    }
}

#[component]
fn AppLifecycleManager() -> Element {
    let services = use_context::<Arc<UiServices>>();
    let auth = use_auth();
    let phase = auth.read().phase;
    use_future(move || {
        let services = services.clone();
        async move {
            if matches!(phase, AuthPhase::Authenticated)
                && let Err(err) = services.directory().refresh_all().await
            {
                error!("failed to refresh directory snapshot: {err}");
            }
        }
    });
    rsx! { Fragment {} }
}

#[component]
fn RouteObserver() -> Element {
    let services = use_context::<Arc<UiServices>>();
    let route = use_route::<Route>();
    use_effect(move || {
        let services = services.clone();
        let current_route = route.clone();
        spawn(async move {
            info!(target = "ui.nav", route = ?current_route);
            match route_navigation_event(&current_route) {
                Some(RouteNavigationEvent::Entity(key)) => {
                    if let Err(err) = services.navigation().record_entity(key).await {
                        warn!(target = "ui.nav", "failed to record entity visit: {err}");
                    }
                }
                Some(RouteNavigationEvent::Contact(contact_id)) => {
                    if let Err(err) = services.navigation().record_contact(contact_id).await {
                        warn!(target = "ui.nav", "failed to record contact visit: {err}");
                    }
                }
                None => {}
            }
        });
    });
    rsx! { Fragment {} }
}

#[component]
fn LoginRoute() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    let services = use_context::<Arc<UiServices>>();
    let auth_service = services.auth();
    if auth.read().is_authenticated() {
        navigator.replace(Route::DashboardRoute {});
    }

    let mut four_words = use_signal(String::new);
    let mut password = use_signal(String::new);

    let login_action = {
        let mut auth_signal = auth;
        let auth_service = auth_service.clone();
        use_coroutine(move |mut rx: UnboundedReceiver<LoginRequest>| {
            let auth_service = auth_service.clone();
            async move {
                while let Some(payload) = rx.next().await {
                    if let Err(err) =
                        process_login(auth_signal, auth_service.clone(), payload).await
                    {
                        auth_signal.with_mut(|state| {
                            state.error = Some(err);
                            state.phase = AuthPhase::LoggedOut;
                        });
                    } else {
                        navigator.replace(Route::DashboardRoute {});
                    }
                }
            }
        })
    };

    let busy = matches!(auth.read().phase, AuthPhase::Authenticating);
    let error_msg = auth.read().error.clone();
    let mut auth_for_validation = auth;

    rsx! {
        AuthLayout {
            title: "Welcome back",
            subtitle: "Unlock your Communitas vault with your four-word identity and passphrase.",
            error: error_msg,
            footer: Some(rsx! {
                div { class: "flex flex-col gap-2 text-sm text-slate-400",
                    span {
                        "Need a vault? "
                        Link { to: Route::CreateIdentityRoute {}, class: "text-emerald-400 hover:underline", "Create one" }
                        " or "
                        Link { to: Route::RecoverIdentityRoute {}, class: "text-emerald-400 hover:underline", "recover" }
                    }
                }
            }),
            form {
                class: "flex flex-col gap-4",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    // Validate empty fields
                    if four_words().trim().is_empty() || password().is_empty() {
                        auth_for_validation.with_mut(|state| {
                            state.error = Some("Please enter your four words and password".into());
                        });
                        return;
                    }
                    login_action.send(LoginRequest {
                        four_words: four_words().trim().to_string(),
                        password: password().clone(),
                    });
                },
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Four words" }
                    input {
                        r#type: "text",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        placeholder: "forest-ocean-light-house",
                        disabled: busy,
                        value: "{four_words}",
                        oninput: move |evt| four_words.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Vault password" }
                    input {
                        r#type: "password",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        placeholder: "********",
                        disabled: busy,
                        value: "{password}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                }
                button {
                    class: "rounded-lg bg-emerald-500 px-4 py-3 font-semibold text-slate-900 shadow-lg shadow-emerald-500/30 transition hover:bg-emerald-400 disabled:opacity-50",
                    r#type: "submit",
                    disabled: busy,
                    if busy { "Signing in..." } else { "Sign in" }
                }
            }
        }
    }
}

#[component]
fn CreateIdentityRoute() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    let services = use_context::<Arc<UiServices>>();
    let auth_service = services.auth();
    if auth.read().is_authenticated() {
        navigator.replace(Route::DashboardRoute {});
    }

    let mut display_name = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut preview_words = use_signal(sample_words_from_core);

    let create_action = {
        let mut auth = auth;
        let auth_service = auth_service.clone();
        use_coroutine(move |mut rx: UnboundedReceiver<CreateRequest>| {
            let auth_service = auth_service.clone();
            async move {
                while let Some(payload) = rx.next().await {
                    if let Err(err) = process_create(auth, auth_service.clone(), payload).await {
                        auth.with_mut(|state| {
                            state.error = Some(err);
                            state.phase = AuthPhase::LoggedOut;
                        });
                    } else {
                        navigator.replace(Route::DashboardRoute {});
                    }
                }
            }
        })
    };

    let busy = matches!(auth.read().phase, AuthPhase::Authenticating);
    let error_msg = auth.read().error.clone();
    let mut auth_for_submit = auth;

    rsx! {
        AuthLayout {
            title: "Create identity",
            subtitle: "Choose a display name and passphrase. Four words are auto-generated from the Rust core.",
            error: error_msg,
            footer: Some(rsx! {
                div { class: "flex flex-col gap-2 text-sm text-slate-400",
                    span {
                        "Already have a vault? "
                        Link { to: Route::LoginRoute {}, class: "text-emerald-400 hover:underline", "Sign in" }
                    }
                }
            }),
            form {
                class: "flex flex-col gap-4",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    if password() != confirm() {
                        auth_for_submit.with_mut(|state| {
                            state.error = Some("Passwords do not match".into())
                        });
                        return;
                    }
                    create_action.send(CreateRequest {
                        display_name: display_name().trim().to_string(),
                        password: password().clone(),
                        four_words: preview_words().as_str().to_string(),
                    });
                },
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Display name" }
                    input {
                        r#type: "text",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        placeholder: "Aurora Station",
                        disabled: busy,
                        value: "{display_name}",
                        oninput: move |evt| display_name.set(evt.value()),
                    }
                }
                div { class: "rounded-lg border border-slate-700 bg-slate-900/60 p-4",
                    span { class: "text-xs uppercase tracking-[0.4em] text-slate-500", "four words" }
                    p { class: "mt-2 font-mono text-lg text-emerald-300 break-words", "{preview_words().as_str()}" }
                    button {
                        class: "mt-3 text-sm font-semibold text-emerald-400 hover:text-emerald-300",
                        r#type: "button",
                        disabled: busy,
                        onclick: move |_| preview_words.set(sample_words_from_core()),
                        "Refresh words"
                    }
                }
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Password" }
                    input {
                        r#type: "password",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        disabled: busy,
                        value: "{password}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Confirm password" }
                    input {
                        r#type: "password",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        disabled: busy,
                        value: "{confirm}",
                        oninput: move |evt| confirm.set(evt.value()),
                    }
                }
                button {
                    class: "rounded-lg bg-emerald-500 px-4 py-3 font-semibold text-slate-900 shadow-lg shadow-emerald-500/30 transition hover:bg-emerald-400 disabled:opacity-50",
                    r#type: "submit",
                    disabled: busy,
                    if busy { "Creating..." } else { "Create identity" }
                }
            }
        }
    }
}

#[component]
fn RecoverIdentityRoute() -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    let services = use_context::<Arc<UiServices>>();
    let auth_service = services.auth();
    if auth.read().is_authenticated() {
        navigator.replace(Route::DashboardRoute {});
    }

    let mut mnemonic = use_signal(String::new);
    let mut passphrase = use_signal(String::new);
    let mut display_name = use_signal(String::new);
    let mut password = use_signal(String::new);

    let recover_action = {
        let mut auth = auth;
        let auth_service = auth_service.clone();
        use_coroutine(move |mut rx: UnboundedReceiver<RecoverRequest>| {
            let auth_service = auth_service.clone();
            async move {
                while let Some(payload) = rx.next().await {
                    if let Err(err) = process_recover(auth, auth_service.clone(), payload).await {
                        auth.with_mut(|state| {
                            state.error = Some(err);
                            state.phase = AuthPhase::LoggedOut;
                        });
                    } else {
                        navigator.replace(Route::DashboardRoute {});
                    }
                }
            }
        })
    };

    let busy = matches!(auth.read().phase, AuthPhase::Authenticating);
    let error_msg = auth.read().error.clone();

    rsx! {
        AuthLayout {
            title: "Recover identity",
            subtitle: "Paste your BIP39 mnemonic and optional passphrase to recreate your Communitas vault.",
            error: error_msg,
            footer: Some(rsx! {
                div { class: "flex flex-col gap-2 text-sm text-slate-400",
                    span {
                        "Remembered your password? "
                        Link { to: Route::LoginRoute {}, class: "text-emerald-400 hover:underline", "Sign in" }
                    }
                }
            }),
            form {
                class: "flex flex-col gap-4",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    let pass = passphrase().trim().to_string();
                    recover_action.send(RecoverRequest {
                        mnemonic: mnemonic().trim().to_string(),
                        passphrase: if pass.is_empty() { None } else { Some(pass) },
                        display_name: display_name().trim().to_string(),
                        password: password().clone(),
                    });
                },
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Mnemonic phrase" }
                    textarea {
                        class: "h-32 rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        placeholder: "abandon ability able about...",
                        disabled: busy,
                        value: "{mnemonic}",
                        oninput: move |evt| mnemonic.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Passphrase (optional)" }
                    input {
                        r#type: "text",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        disabled: busy,
                        value: "{passphrase}",
                        oninput: move |evt| passphrase.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Display name" }
                    input {
                        r#type: "text",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        disabled: busy,
                        value: "{display_name}",
                        oninput: move |evt| display_name.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span { class: "text-sm font-medium text-slate-200", "Vault password" }
                    input {
                        r#type: "password",
                        class: "rounded-lg border border-slate-700 bg-slate-900/60 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                        disabled: busy,
                        value: "{password}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                }
                button {
                    class: "rounded-lg bg-emerald-500 px-4 py-3 font-semibold text-slate-900 shadow-lg shadow-emerald-500/30 transition hover:bg-emerald-400 disabled:opacity-50",
                    r#type: "submit",
                    disabled: busy,
                    if busy { "Recovering..." } else { "Recover identity" }
                }
            }
        }
    }
}

#[component]
fn DashboardRoute() -> Element {
    render_authenticated_page(
        "Home",
        rsx! {
            HomeOverview {}
        },
    )
}

#[component]
fn HomeOverview() -> Element {
    let directory = use_directory_snapshot();
    let snapshot = directory();

    // Show skeleton while directory is loading (no identity yet)
    if snapshot.identity.is_none() {
        return rsx! {
            div { class: "flex flex-col gap-8",
                SkeletonWelcomeCard {}
                SkeletonStatsGrid {}
                SkeletonSpacesSection {}
            }
        };
    }

    let display_name = snapshot
        .identity
        .as_ref()
        .map(|identity| identity.display_name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Explorer".to_string());

    let organizations: Vec<_> = snapshot
        .entities
        .iter()
        .filter(|entity| {
            matches!(entity.entity_type, UnifiedEntityType::Organization)
                && entity.category == Some(OrganizationCategory::Organization)
        })
        .cloned()
        .collect();
    let communities: Vec<_> = snapshot
        .entities
        .iter()
        .filter(|entity| {
            matches!(entity.entity_type, UnifiedEntityType::Organization)
                && entity.category == Some(OrganizationCategory::Community)
        })
        .cloned()
        .collect();
    let projects: Vec<_> = snapshot
        .entities
        .iter()
        .filter(|entity| matches!(entity.entity_type, UnifiedEntityType::Project))
        .cloned()
        .collect();
    let groups: Vec<_> = snapshot
        .entities
        .iter()
        .filter(|entity| matches!(entity.entity_type, UnifiedEntityType::Group))
        .cloned()
        .collect();
    let channels: Vec<_> = snapshot
        .entities
        .iter()
        .filter(|entity| matches!(entity.entity_type, UnifiedEntityType::Channel))
        .cloned()
        .collect();
    let personal_groups: Vec<_> = groups
        .iter()
        .filter(|group| group.parent_id.is_none())
        .cloned()
        .collect();

    let stats = vec![
        StatItem {
            label: "Organizations",
            value: organizations.len(),
        },
        StatItem {
            label: "Communities",
            value: communities.len(),
        },
        StatItem {
            label: "Projects",
            value: projects.len(),
        },
        StatItem {
            label: "Groups",
            value: groups.len(),
        },
        StatItem {
            label: "Channels",
            value: channels.len(),
        },
        StatItem {
            label: "Contacts",
            value: snapshot.contacts.len(),
        },
    ];

    rsx! {
        div { class: "flex flex-col gap-8",
            WelcomeCard { display_name }
            StatsGrid { stats }
            SpacesSection {
                personal: personal_groups,
                communities,
                organizations,
                projects,
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct WelcomeCardProps {
    display_name: String,
}

#[component]
fn WelcomeCard(props: WelcomeCardProps) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-slate-900 bg-gradient-to-r from-emerald-800/40 to-emerald-500/20 p-6 shadow-lg",
            span { class: "text-xs uppercase tracking-[0.4em] text-emerald-200", "Communitas" }
            h2 { class: "mt-2 text-3xl font-semibold text-white", "Welcome back, {props.display_name}" }
            p { class: "text-slate-300", "Your local-first collaboration hub" }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct StatItem {
    label: &'static str,
    value: usize,
}

#[derive(Props, PartialEq, Clone)]
struct StatsGridProps {
    stats: Vec<StatItem>,
}

#[component]
fn StatsGrid(props: StatsGridProps) -> Element {
    rsx! {
        div { class: "grid gap-4 md:grid-cols-3",
            {props.stats.iter().map(|item| {
                rsx! {
                    div { class: "rounded-2xl border border-slate-900 bg-slate-950/80 p-4",
                        span { class: "text-sm uppercase tracking-[0.3em] text-slate-500", "{item.label}" }
                        h3 { class: "text-2xl font-semibold text-white", "{item.value}" }
                    }
                }
            })}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SpacesSectionProps {
    personal: Vec<UnifiedEntity>,
    communities: Vec<UnifiedEntity>,
    organizations: Vec<UnifiedEntity>,
    projects: Vec<UnifiedEntity>,
}

#[component]
fn SpacesSection(props: SpacesSectionProps) -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            h3 { class: "text-2xl font-semibold text-white", "Your Spaces" }
            div { class: "grid gap-4 lg:grid-cols-2",
                EntityListPanel {
                    title: "Personal",
                    entities: props.personal.clone(),
                    empty_label: "No personal groups yet",
                    accent_class: "border-emerald-500/40",
                }
                EntityListPanel {
                    title: "Communities",
                    entities: props.communities.clone(),
                    empty_label: "No communities yet",
                    accent_class: "border-emerald-500/20",
                }
                EntityListPanel {
                    title: "Organizations",
                    entities: props.organizations.clone(),
                    empty_label: "No organizations yet",
                    accent_class: "border-emerald-300/20",
                }
                EntityListPanel {
                    title: "Projects",
                    entities: props.projects.clone(),
                    empty_label: "No projects yet",
                    accent_class: "border-emerald-200/30",
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct EntityListPanelProps {
    title: &'static str,
    entities: Vec<UnifiedEntity>,
    empty_label: &'static str,
    accent_class: &'static str,
}

#[component]
fn EntityListPanel(props: EntityListPanelProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    rsx! {
        div { class: format!("rounded-2xl border bg-slate-950/80 p-4 {}", props.accent_class),
            h4 { class: "text-lg font-semibold text-white", "{props.title}" }
            if props.entities.is_empty() {
                p { class: "text-sm text-slate-500", "{props.empty_label}" }
            } else {
                div { class: "mt-3 flex flex-col gap-2",
                    {props.entities.iter().take(5).map(|entity| {
                        let services = services.clone();
                        let nav_key = nav_key_for(entity);
                        let route = entity_route(entity);
                        let entity_name = entity.name.clone();
                        let member_count = entity.member_count;
                        rsx! {
                            Link {
                                to: route.clone(),
                                class: "flex flex-col rounded-xl border border-slate-900/80 bg-slate-950/60 px-3 py-2 text-left hover:border-emerald-400",
                                onclick: move |_| {
                                    record_entity_visit(services.clone(), nav_key.clone());
                                },
                                span { class: "text-sm font-semibold text-white", "{entity_name}" }
                                span { class: "text-xs text-slate-500", "{member_count} members" }
                            }
                        }
                    })}
                }
            }
        }
    }
}
#[component]
fn MessagesRoute() -> Element {
    render_authenticated_page(
        "Messages",
        rsx! {
            PlaceholderPanel { title: "Messages".into(), body: "Stream and compose entity chats, DMs, and reactions.".into() }
        },
    )
}

#[component]
fn ProjectsRoute() -> Element {
    render_authenticated_page(
        "Projects",
        rsx! {
            PlaceholderPanel { title: "Projects & Kanban".into(), body: "Kanban boards with drag-and-drop CRDT synchronization land here.".into() }
        },
    )
}

#[component]
fn ContactsRoute() -> Element {
    render_authenticated_page(
        "Contacts",
        rsx! {
            PlaceholderPanel { title: "Contacts".into(), body: "Presence indicators, favorites, and quick actions.".into() }
        },
    )
}

#[component]
fn NetworkRoute() -> Element {
    render_authenticated_page(
        "Network",
        rsx! {
            PlaceholderPanel { title: "Network diagnostics".into(), body: "Gossip peers, bootstrap connections, and MCP wiring.".into() }
        },
    )
}

#[component]
fn MoreRoute() -> Element {
    render_authenticated_page(
        "More",
        rsx! {
            PlaceholderPanel { title: "More".into(), body: "Settings, MCP tools, and advanced utilities.".into() }
        },
    )
}

#[component]
fn EntityDetailRoute(entity_type: String, entity_id: String) -> Element {
    render_authenticated_page(
        "Entity",
        rsx! {
            EntityDetailsView { entity_type, entity_id }
        },
    )
}

#[component]
fn EntityChatRoute(entity_type: String, entity_id: String) -> Element {
    render_authenticated_page(
        "Entity Chat",
        rsx! {
            PlaceholderPanel { title: "Entity Chat".into(), body: format!("Chat for {entity_type} {entity_id}").into() }
        },
    )
}

#[component]
fn EntityDriveRoute(entity_type: String, entity_id: String) -> Element {
    render_authenticated_page(
        "Entity Drive",
        rsx! {
            PlaceholderPanel { title: "Entity Drive".into(), body: format!("Drive for {entity_type} {entity_id}").into() }
        },
    )
}

#[component]
fn ProjectBoardRoute(project_id: String) -> Element {
    render_authenticated_page(
        "Project Board",
        rsx! {
            PlaceholderPanel { title: "Kanban Board".into(), body: format!("Project board for {project_id}").into() }
        },
    )
}

#[component]
fn ContactChatRoute(contact_id: String) -> Element {
    render_authenticated_page(
        "Contact Chat",
        rsx! {
            PlaceholderPanel { title: "Contact Chat".into(), body: format!("Chat with {contact_id}").into() }
        },
    )
}

#[derive(Props, PartialEq, Clone)]
struct EntityDetailsViewProps {
    entity_type: String,
    entity_id: String,
}

#[component]
fn EntityDetailsView(props: EntityDetailsViewProps) -> Element {
    let directory = use_directory_snapshot();
    let snapshot = directory();
    let entity = snapshot
        .entities
        .iter()
        .find(|entity| entity.id == props.entity_id)
        .cloned();

    if let Some(entity) = entity {
        rsx! {
            div { class: "flex flex-col gap-4",
                h2 { class: "text-2xl font-semibold text-white", "{entity.name}" }
                p { class: "text-sm text-slate-400", "{entity.description}" }
                div { class: "flex flex-wrap gap-4 text-sm text-slate-400",
                    span { "{entity.member_count} members" }
                    if matches!(entity.entity_type, UnifiedEntityType::Organization) {
                        span {
                            "Category: ",
                            match entity.category {
                                Some(OrganizationCategory::Community) => "Community",
                                Some(OrganizationCategory::Organization) => "Organization",
                                None => "Organization",
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            PlaceholderPanel { title: "Entity".into(), body: format!("Entity {} not found", props.entity_id).into() }
        }
    }
}

fn render_authenticated_page(title: &'static str, body: Element) -> Element {
    let auth = use_auth();
    let navigator = use_navigator();
    if !auth.read().is_authenticated() {
        navigator.replace(Route::LoginRoute {});
        return rsx! {
            div { class: "min-h-screen bg-slate-950 text-slate-100 flex items-center justify-center", "Redirecting..." }
        };
    }
    rsx! {
        AppShell { title, body }
    }
}

fn nav_key_for(entity: &UnifiedEntity) -> EntityNavigationKey {
    EntityNavigationKey::new(entity_type_segment(entity.entity_type), entity.id.clone())
}

fn entity_route(entity: &UnifiedEntity) -> Route {
    Route::EntityDetailRoute {
        entity_type: entity_type_segment(entity.entity_type).to_string(),
        entity_id: entity.id.clone(),
    }
}

fn entity_type_segment(entity_type: UnifiedEntityType) -> &'static str {
    match entity_type {
        UnifiedEntityType::Organization => "organisation",
        UnifiedEntityType::Project => "project",
        UnifiedEntityType::Group => "group",
        UnifiedEntityType::Channel => "channel",
        UnifiedEntityType::Person => "person",
    }
}

fn record_entity_visit(services: Arc<UiServices>, key: EntityNavigationKey) {
    let nav = services.navigation();
    spawn(async move {
        let _ = nav.record_entity(key).await;
    });
}

fn find_entity_by_id(snapshot: &DirectorySnapshot, entity_id: &str) -> Option<UnifiedEntity> {
    snapshot
        .entities
        .iter()
        .find(|entity| entity.id == entity_id)
        .cloned()
}

#[derive(Props, PartialEq, Clone)]
struct AppShellProps {
    title: &'static str,
    body: Element,
}

#[component]
fn AppShell(props: AppShellProps) -> Element {
    let auth = use_auth();
    let session = auth.read().session.clone();
    let navigator = use_navigator();
    let services = use_context::<Arc<UiServices>>();
    let auth_service = services.auth();
    let directory_signal = use_directory_snapshot();
    let directory_snapshot = directory_signal();
    let navigation_signal = use_navigation_snapshot();
    let navigation_snapshot = navigation_signal();
    let mut sidebar_collapsed = use_signal(|| false);

    let logout_action = {
        let mut auth = auth;
        let auth_service = auth_service.clone();
        use_coroutine(move |mut rx: UnboundedReceiver<()>| {
            let auth_service = auth_service.clone();
            async move {
                while rx.next().await.is_some() {
                    if let Err(err) = process_logout(auth, auth_service.clone()).await {
                        auth.with_mut(|state| state.error = Some(err));
                    } else {
                        navigator.replace(Route::LoginRoute {});
                    }
                }
            }
        })
    };

    let current_route = use_route::<Route>();
    let nav_links = nav_items();
    let recent_entities = navigation_snapshot
        .recent_entities
        .iter()
        .filter_map(|key| find_entity_by_id(&directory_snapshot, &key.entity_id))
        .collect::<Vec<_>>();
    let starred_entities = navigation_snapshot
        .starred_entities
        .iter()
        .filter_map(|key| find_entity_by_id(&directory_snapshot, &key.entity_id))
        .collect::<Vec<_>>();

    rsx! {
        main { class: "min-h-screen bg-slate-950 text-slate-100",
            header { class: "flex flex-col gap-3 border-b border-slate-900 bg-slate-950/70 px-8 py-6 md:flex-row md:items-center md:justify-between",
                div {
                    span { class: "text-xs uppercase tracking-[0.5em] text-emerald-400", "Communitas" }
                    h1 { class: "text-3xl font-semibold tracking-tight", "{props.title}" }
                }
                if let Some(session) = session.clone() {
                    div { class: "flex items-center gap-4 text-sm text-slate-300",
                        div {
                            span { class: "font-semibold text-slate-100", "{session.display_name}" }
                            p { class: "text-xs text-slate-500", "{session.four_words}" }
                        }
                        button {
                            class: "rounded-lg border border-slate-700 px-4 py-2 text-xs font-semibold text-slate-200 hover:border-emerald-400",
                            onclick: move |_| logout_action.send(()),
                            "Logout"
                        }
                    }
                }
                button {
                    class: "inline-flex items-center justify-center rounded-lg border border-slate-800 px-3 py-2 text-xs font-semibold text-slate-300 hover:border-emerald-400 lg:hidden",
                    onclick: move |_| sidebar_collapsed.set(!sidebar_collapsed()),
                    if sidebar_collapsed() { "Show Navigation" } else { "Hide Navigation" }
                }
            }
            div { class: "flex flex-col gap-8 px-8 py-8 lg:flex-row",
                if sidebar_collapsed() {
                    div { class: "rounded-2xl border border-slate-900 bg-slate-950/60 p-4 text-sm text-slate-400 lg:hidden",
                        span { "Navigation hidden. " }
                        button {
                            class: "font-semibold text-emerald-400 hover:text-emerald-300",
                            onclick: move |_| sidebar_collapsed.set(false),
                            "Show menu"
                        }
                    }
                } else {
                    nav { class: "w-full max-w-sm space-y-4 rounded-2xl border border-slate-900 bg-slate-950/80 p-4 lg:w-80",
                        span { class: "text-xs uppercase tracking-[0.4em] text-slate-500", "Navigation" }
                        {nav_links.into_iter().map(|item| {
                            let route = (item.to)();
                            let active = current_route == route;
                            let link_route = route.clone();
                            let label = item.label;
                            rsx! {
                                Link {
                                    to: link_route,
                                    class: format!(
                                        "block rounded-xl border px-4 py-3 transition {}",
                                        if active {
                                            "border-emerald-400 bg-emerald-400/10 text-emerald-200"
                                        } else {
                                            "border-transparent text-slate-400 hover:text-emerald-200 hover:border-slate-800"
                                        }
                                    ),
                                    onclick: move |_| {
                                        info!(target = "ui.nav", event = "navigate_click", destination = label);
                                    },
                                    span { class: "text-base font-semibold", "{item.label}" }
                                    p { class: "text-sm text-slate-500", "{item.description}" }
                                }
                            }
                        })}
                    if !recent_entities.is_empty() {
                        SidebarEntityList { title: "Recents", entities: recent_entities }
                    }
                        if !starred_entities.is_empty() {
                            SidebarEntityList { title: "Starred", entities: starred_entities }
                        }
                    }
                }
                section { class: "flex-1 rounded-2xl border border-slate-900 bg-slate-950/80 p-6",
                    {props.body}
                }
            }
        }
    }
}

enum RouteNavigationEvent {
    Entity(EntityNavigationKey),
    Contact(String),
}

fn route_navigation_event(route: &Route) -> Option<RouteNavigationEvent> {
    match route {
        Route::EntityDetailRoute {
            entity_type,
            entity_id,
        }
        | Route::EntityChatRoute {
            entity_type,
            entity_id,
        }
        | Route::EntityDriveRoute {
            entity_type,
            entity_id,
        } => Some(RouteNavigationEvent::Entity(EntityNavigationKey::new(
            entity_type.clone(),
            entity_id.clone(),
        ))),
        Route::ProjectBoardRoute { project_id } => Some(RouteNavigationEvent::Entity(
            EntityNavigationKey::new("project", project_id.clone()),
        )),
        Route::ContactChatRoute { contact_id } => {
            Some(RouteNavigationEvent::Contact(contact_id.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_service::{UiServices, storage::UiStorage};
    use dioxus::prelude::{Element, VirtualDom, rsx, use_context_provider, use_hook, use_signal};
    use dioxus_history::{MemoryHistory, provide_history_context};
    use dioxus_ssr::render;
    use std::{rc::Rc, sync::Arc};
    use tempfile::TempDir;

    #[test]
    fn route_event_detects_entity_links() {
        let route = Route::EntityDetailRoute {
            entity_type: "project".into(),
            entity_id: "abc".into(),
        };
        match route_navigation_event(&route) {
            Some(RouteNavigationEvent::Entity(key)) => {
                assert_eq!(key.entity_id, "abc");
                assert_eq!(key.entity_type, "project");
            }
            _ => panic!("expected entity event"),
        }
    }

    #[test]
    fn route_event_detects_contacts() {
        let route = Route::ContactChatRoute {
            contact_id: "alice".into(),
        };
        match route_navigation_event(&route) {
            Some(RouteNavigationEvent::Contact(contact)) => assert_eq!(contact, "alice"),
            _ => panic!("expected contact event"),
        }
    }

    #[test]
    fn login_route_renders_copy() {
        let env = TestEnv::new();
        let html = render_route_html("/login", env.services(), AuthState::default());
        assert!(
            html.contains("Unlock your Communitas vault"),
            "login copy missing:\n{html}"
        );
    }

    #[test]
    fn create_identity_route_shows_generated_words() {
        let env = TestEnv::new();
        let html = render_route_html("/create", env.services(), AuthState::default());
        assert!(
            html.contains("Create identity") && html.contains("four words"),
            "create identity layout missing copy:\n{html}"
        );
    }

    #[test]
    fn recover_identity_route_includes_mnemonic_field() {
        let env = TestEnv::new();
        let html = render_route_html("/recover", env.services(), AuthState::default());
        assert!(
            html.contains("Mnemonic phrase") && html.contains("Recover identity"),
            "recover identity layout missing mnemonic instructions:\n{html}"
        );
    }

    #[test]
    fn unauthenticated_routes_redirect_to_login() {
        let env = TestEnv::new();
        let html = render_route_html("/messages", env.services(), AuthState::default());
        assert!(
            html.contains("Redirecting"),
            "expected redirect placeholder when user is logged out:\n{html}"
        );
    }

    #[test]
    fn dashboard_renders_authenticated_session_name() {
        let env = TestEnv::new();
        let mut authed = AuthState::default();
        authed.phase = AuthPhase::Authenticated;
        authed.session = Some(AuthSession {
            pubkey_hex: "deadbeef".into(),
            four_words: "forest-ocean-light-house".into(),
            display_name: "Test Pilot".into(),
            device_name: "Test Device".into(),
        });
        let html = render_route_html("/", env.services(), authed);
        assert!(
            html.contains("Test Pilot") && html.contains("forest-ocean-light-house"),
            "dashboard missing authenticated session info:\n{html}"
        );
    }

    struct TestEnv {
        _temp: TempDir,
        services: Arc<UiServices>,
    }

    impl TestEnv {
        fn new() -> Self {
            let temp = TempDir::new().expect("create temp dir");
            let storage = UiStorage::from_path(temp.path()).expect("storage");
            let services = UiServices::new(storage).expect("ui services");
            Self {
                _temp: temp,
                services: Arc::new(services),
            }
        }

        fn services(&self) -> Arc<UiServices> {
            self.services.clone()
        }
    }

    #[derive(Clone)]
    struct ServicesHandle(Arc<UiServices>);

    impl ServicesHandle {
        fn get(&self) -> Arc<UiServices> {
            self.0.clone()
        }
    }

    impl PartialEq for ServicesHandle {
        fn eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.0, &other.0)
        }
    }

    #[derive(Props, Clone, PartialEq)]
    struct ProviderProps {
        services: ServicesHandle,
        initial_auth: AuthState,
        children: Element,
    }

    #[component]
    fn TestProviders(props: ProviderProps) -> Element {
        let services = props.services.get();
        let initial_auth = props.initial_auth.clone();
        use_context_provider(|| services);
        let auth_state = use_signal(move || initial_auth.clone());
        use_context_provider(|| auth_state);
        rsx! { {props.children} }
    }

    fn render_route_html(path: &str, services: Arc<UiServices>, auth: AuthState) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct RootProps {
            services: ServicesHandle,
            auth: AuthState,
            initial_path: String,
        }

        #[component]
        fn Root(props: RootProps) -> Element {
            let initial_path = props.initial_path.clone();
            use_hook(move || {
                let history = Rc::new(MemoryHistory::default());
                history.replace(initial_path);
                provide_history_context(history);
            });
            rsx! {
                TestProviders {
                    services: props.services.clone(),
                    initial_auth: props.auth.clone(),
                    Router::<Route> {}
                }
            }
        }

        let mut dom = VirtualDom::new_with_props(
            Root,
            RootProps {
                services: ServicesHandle(services),
                auth,
                initial_path: path.to_string(),
            },
        );
        dom.rebuild_in_place();
        render(&dom)
    }
}

struct NavItem {
    to: fn() -> Route,
    label: &'static str,
    description: &'static str,
}

fn nav_items() -> Vec<NavItem> {
    vec![
        NavItem {
            to: || Route::DashboardRoute {},
            label: "Home",
            description: "Unified overview",
        },
        NavItem {
            to: || Route::MessagesRoute {},
            label: "Messages",
            description: "Threads & entities",
        },
        NavItem {
            to: || Route::ProjectsRoute {},
            label: "Projects",
            description: "Kanban + CRDT collab",
        },
        NavItem {
            to: || Route::ContactsRoute {},
            label: "Contacts",
            description: "Presence & invites",
        },
        NavItem {
            to: || Route::NetworkRoute {},
            label: "Network",
            description: "Gossip + MCP",
        },
        NavItem {
            to: || Route::MoreRoute {},
            label: "More",
            description: "Settings & tools",
        },
    ]
}

#[derive(Props, PartialEq, Clone)]
struct DashboardCardProps {
    title: &'static str,
    body: &'static str,
    route: Route,
}

#[component]
fn DashboardCard(props: DashboardCardProps) -> Element {
    rsx! {
        Link {
            to: props.route.clone(),
            class: "flex flex-col gap-2 rounded-2xl border border-slate-900 bg-slate-950/80 p-5 transition hover:border-emerald-400 hover:-translate-y-0.5",
            span { class: "text-sm uppercase tracking-[0.4em] text-slate-500", "Preview" }
            h3 { class: "text-2xl font-semibold text-slate-100", "{props.title}" }
            p { class: "text-sm text-slate-400", "{props.body}" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SidebarEntityListProps {
    title: &'static str,
    entities: Vec<UnifiedEntity>,
}

#[component]
fn SidebarEntityList(props: SidebarEntityListProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    rsx! {
        div { class: "space-y-2",
            h5 { class: "text-xs uppercase tracking-[0.3em] text-slate-500", "{props.title}" }
            {props.entities.iter().take(5).map(|entity| {
                let services = services.clone();
                let nav_key = nav_key_for(entity);
                let route = entity_route(entity);
                let name = entity.name.clone();
                rsx! {
                    Link {
                        to: route.clone(),
                        class: "block rounded-xl border border-slate-900/60 bg-slate-950/40 px-3 py-2 text-sm text-slate-200 hover:border-emerald-400",
                        onclick: move |_| {
                            record_entity_visit(services.clone(), nav_key.clone());
                        },
                        "{name}"
                    }
                }
            })}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct PlaceholderProps {
    title: Cow<'static, str>,
    body: Cow<'static, str>,
}

#[component]
fn PlaceholderPanel(props: PlaceholderProps) -> Element {
    rsx! {
        div { class: "flex flex-col gap-2",
            h2 { class: "text-2xl font-semibold text-slate-100", "{props.title}" }
            p { class: "text-slate-400", "{props.body}" }
            span { class: "text-xs uppercase tracking-[0.4em] text-slate-600", "Coming soon" }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct AuthLayoutProps {
    title: &'static str,
    subtitle: &'static str,
    error: Option<String>,
    #[props(optional)]
    footer: Option<Element>,
    children: Element,
}

#[component]
fn AuthLayout(props: AuthLayoutProps) -> Element {
    rsx! {
        main { class: "min-h-screen bg-slate-950 px-4 py-16 text-slate-100 sm:px-0",
            div { class: "mx-auto flex max-w-xl flex-col gap-6 rounded-3xl border border-slate-900 bg-slate-950/80 p-8",
                div { class: "flex flex-col gap-1",
                    span { class: "text-xs uppercase tracking-[0.5em] text-emerald-400", "Communitas" }
                    h1 { class: "text-3xl font-semibold tracking-tight", "{props.title}" }
                    p { class: "text-sm text-slate-400", "{props.subtitle}" }
                }
                if let Some(err) = props.error {
                    div { class: "rounded-xl border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-200",
                        "{err}"
                    }
                }
                {props.children}
                if let Some(content) = props.footer {
                    div { {content} }
                }
            }
        }
    }
}

fn sample_words_from_core() -> SampleWords {
    match generate_id_words() {
        Ok(words) => SampleWords::new(words),
        Err(err) => {
            error!("failed to generate id words: {err}");
            SampleWords::new(format!("identity error: {err}"))
        }
    }
}

// --- Skeleton loading components ---

#[component]
fn SkeletonPulse(class: &'static str) -> Element {
    rsx! {
        div { class: format!("animate-pulse bg-slate-800 rounded {class}") }
    }
}

#[component]
fn SkeletonWelcomeCard() -> Element {
    rsx! {
        div { class: "rounded-2xl border border-slate-900 bg-gradient-to-r from-slate-800/40 to-slate-700/20 p-6 shadow-lg",
            SkeletonPulse { class: "h-3 w-24 mb-4" }
            SkeletonPulse { class: "h-8 w-64 mb-2" }
            SkeletonPulse { class: "h-4 w-48" }
        }
    }
}

#[component]
fn SkeletonStatsGrid() -> Element {
    rsx! {
        div { class: "grid gap-4 md:grid-cols-3",
            for _ in 0..6 {
                div { class: "rounded-2xl border border-slate-900 bg-slate-950/80 p-4",
                    SkeletonPulse { class: "h-3 w-20 mb-2" }
                    SkeletonPulse { class: "h-7 w-12" }
                }
            }
        }
    }
}

#[component]
fn SkeletonSpacesSection() -> Element {
    rsx! {
        div { class: "flex flex-col gap-4",
            SkeletonPulse { class: "h-7 w-32" }
            div { class: "grid gap-4 lg:grid-cols-2",
                for _ in 0..4 {
                    div { class: "rounded-2xl border border-slate-900 bg-slate-950/80 p-4",
                        SkeletonPulse { class: "h-5 w-24 mb-3" }
                        SkeletonPulse { class: "h-4 w-32" }
                    }
                }
            }
        }
    }
}

struct LoginRequest {
    four_words: String,
    password: String,
}

struct CreateRequest {
    display_name: String,
    password: String,
    four_words: String,
}

struct RecoverRequest {
    mnemonic: String,
    passphrase: Option<String>,
    display_name: String,
    password: String,
}

async fn process_login(
    mut auth: Signal<AuthState>,
    auth_service: Arc<AuthController>,
    payload: LoginRequest,
) -> Result<(), String> {
    let LoginRequest {
        four_words,
        password,
    } = payload;
    auth.with_mut(|state| {
        state.phase = AuthPhase::Authenticating;
        state.error = None;
    });

    let session = auth_service
        .login(four_words.trim(), password.as_str())
        .await
        .map_err(|err| err.to_string())?;

    auth.with_mut(|state| {
        state.phase = AuthPhase::Authenticated;
        state.session = Some(session.clone());
        state.error = None;
    });
    Ok(())
}

async fn process_create(
    mut auth: Signal<AuthState>,
    auth_service: Arc<AuthController>,
    payload: CreateRequest,
) -> Result<(), String> {
    let CreateRequest {
        display_name,
        password,
        four_words,
    } = payload;
    auth.with_mut(|state| {
        state.phase = AuthPhase::Authenticating;
        state.error = None;
    });

    let session = auth_service
        .create_identity(four_words.trim(), display_name.trim(), password.as_str())
        .await
        .map_err(|err| err.to_string())?;

    auth.with_mut(|state| {
        state.phase = AuthPhase::Authenticated;
        state.session = Some(session.clone());
        state.error = None;
    });
    Ok(())
}

async fn process_recover(
    mut auth: Signal<AuthState>,
    auth_service: Arc<AuthController>,
    payload: RecoverRequest,
) -> Result<(), String> {
    let RecoverRequest {
        mnemonic,
        passphrase,
        display_name,
        password,
    } = payload;
    auth.with_mut(|state| {
        state.phase = AuthPhase::Authenticating;
        state.error = None;
    });

    let session = auth_service
        .recover_identity(
            mnemonic.trim(),
            passphrase.as_deref(),
            display_name.trim(),
            password.as_str(),
        )
        .await
        .map_err(|err| err.to_string())?;

    auth.with_mut(|state| {
        state.phase = AuthPhase::Authenticated;
        state.session = Some(session.clone());
        state.error = None;
    });
    Ok(())
}

async fn process_logout(
    mut auth: Signal<AuthState>,
    auth_service: Arc<AuthController>,
) -> Result<(), String> {
    auth_service.logout().await.map_err(|err| err.to_string())?;
    auth.with_mut(|state| {
        state.session = None;
        state.phase = AuthPhase::LoggedOut;
        state.error = None;
    });
    Ok(())
}
