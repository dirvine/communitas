#![allow(non_snake_case)]
#![allow(clippy::print_stderr)] // eprintln! used for bootstrap errors before logger init

mod components;
pub mod contrast;
pub mod hooks;
pub mod onboarding;
mod platform;
pub mod styles;
pub mod tokens;
pub mod version;

use communitas_core::generate_id_words;

use communitas_ui_api::{
    OrganizationCategory, PresenceStatus, SampleWords, UnifiedContact, UnifiedEntity,
    UnifiedEntityType,
};
use communitas_ui_service::{
    UiServices,
    auth::{AuthController, AuthService, AuthSession},
    directory::DirectorySnapshot,
    navigation::{EntityNavigationKey, NavigationService, NavigationStateSnapshot},
};
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use futures::StreamExt;
use native_dialog::{MessageDialog, MessageType};
use onboarding::{TOUR_STEPS, TourOverlay, TourState, use_tour_state};
use std::{
    borrow::Cow,
    sync::{Arc, OnceLock},
    time::Instant,
};
use styles::{button, input};
use tokens::colors;
use tracing::{error, info};

static UI_SERVICES: OnceLock<Arc<UiServices>> = OnceLock::new();
static STARTUP_TIME: OnceLock<Instant> = OnceLock::new();

#[allow(clippy::expect_used)] // OnceLock guaranteed initialized in main() before UI renders
fn ui_services() -> Arc<UiServices> {
    UI_SERVICES
        .get()
        .expect("UI services not initialized")
        .clone()
}

/// Main entry point for the Dioxus application.
///
/// Uses `#[tokio::main]` to provide an async runtime, allowing the use of
/// `bootstrap_with_device_enumerator_async()` which avoids creating a nested
/// Tokio runtime for better startup performance.
#[tokio::main]
async fn main() {
    // Record startup time for metrics (first thing we do)
    let _ = STARTUP_TIME.set(Instant::now());

    if let Err(err) = dioxus_logger::init(Level::INFO) {
        eprintln!("failed to init logger: {err}");
    }
    // Check WebView availability BEFORE expensive service initialization
    if let Err(err) = platform::check_webview_available() {
        eprintln!("WebView not available: {err}");
        // Show native dialog since Dioxus UI won't work without WebView
        let _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Communitas - Missing Dependency")
            .set_text(&format!(
                "Communitas cannot start because a required component is missing.\n\n{err}"
            ))
            .show_alert();
        std::process::exit(1);
    }

    // Use async bootstrap with lazy device enumeration for faster startup.
    // Device enumerator is initialized lazily when call UI is first accessed,
    // saving 100-300ms on initial render.
    let services = UiServices::bootstrap_async().await.unwrap_or_else(|err| {
        eprintln!("failed to initialize UI services: {err}");
        std::process::exit(1);
    });

    // Start call-to-presence sync so call/screen share status updates presence
    services.start_call_presence_sync();
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
    #[route("/contact/:contact_id")]
    ContactDetailRoute { contact_id: String },
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

/// Entities categorized by type in a single pass.
///
/// Avoids O(n*k) filtering where k is the number of categories.
/// Instead, categorizes all entities in O(n) time.
#[derive(Default, Clone, PartialEq)]
struct CategorizedEntities {
    organizations: Vec<UnifiedEntity>,
    communities: Vec<UnifiedEntity>,
    projects: Vec<UnifiedEntity>,
    groups: Vec<UnifiedEntity>,
    channels: Vec<UnifiedEntity>,
    /// Groups with no parent (personal groups)
    personal_groups: Vec<UnifiedEntity>,
}

impl CategorizedEntities {
    /// Categorize entities from a slice in a single pass.
    fn from_entities(entities: &[UnifiedEntity]) -> Self {
        let mut result = Self::default();

        for entity in entities {
            match entity.entity_type {
                UnifiedEntityType::Organization => {
                    if entity.category == Some(OrganizationCategory::Community) {
                        result.communities.push(entity.clone());
                    } else {
                        result.organizations.push(entity.clone());
                    }
                }
                UnifiedEntityType::Project => {
                    result.projects.push(entity.clone());
                }
                UnifiedEntityType::Group => {
                    result.groups.push(entity.clone());
                    if entity.parent_id.is_none() {
                        result.personal_groups.push(entity.clone());
                    }
                }
                UnifiedEntityType::Channel => {
                    result.channels.push(entity.clone());
                }
                UnifiedEntityType::Person => {
                    // Persons are handled separately via contacts
                }
            }
        }

        result
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

    // Onboarding tour state
    let mut tour_state = use_context_provider(|| Signal::new(TourState::new(TOUR_STEPS.len())));

    // Log startup timing on first render (runs once)
    use_effect(|| {
        if let Some(start) = STARTUP_TIME.get() {
            let elapsed = start.elapsed();
            info!(
                elapsed_ms = elapsed.as_millis(),
                "First render complete, total startup: {:.1}ms",
                elapsed.as_secs_f64() * 1000.0
            );
        }
    });

    // Tour navigation handlers
    let on_next = move |_: ()| {
        tour_state.with_mut(|state| {
            state.next();
        });
    };

    let on_previous = move |_: ()| {
        tour_state.with_mut(|state| {
            state.previous();
        });
    };

    let on_skip = move |_: ()| {
        tour_state.with_mut(|state| {
            state.skip();
        });
        info!(target: "ui.onboarding", "Tour skipped by user");
    };

    let on_finish = move |_: ()| {
        tour_state.with_mut(|state| {
            state.complete();
        });
        info!(target: "ui.onboarding", "Tour completed successfully");
    };

    rsx! {
        AppLifecycleManager {}
        RouteObserver {}
        Router::<Route> {}

        // Onboarding tour overlay (rendered above all content)
        TourOverlay {
            tour_state: tour_state,
            on_next: on_next,
            on_previous: on_previous,
            on_skip: on_skip,
            on_finish: on_finish,
        }
    }
}

#[component]
fn AppLifecycleManager() -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut auth = use_auth();
    let phase = auth.read().phase;
    let mut tour_state = use_tour_state();

    // Track whether auto-login has been attempted to prevent duplicate attempts
    // when auth state flickers or component re-renders
    let mut auto_login_attempted = use_signal(|| false);

    // Track whether tour has been triggered for this session
    let mut tour_triggered = use_signal(|| false);

    // Clone services for each future
    let services_for_autologin = services.clone();
    let services_for_directory = services;

    // Try auto-login on startup (only when logged out and not yet attempted)
    use_future(move || {
        let services = services_for_autologin.clone();
        async move {
            // Guard: Only attempt auto-login if logged out AND not already attempted
            if matches!(phase, AuthPhase::LoggedOut) && !auto_login_attempted() {
                // Set flag immediately to prevent duplicate attempts
                auto_login_attempted.set(true);

                info!(target: "ui.auth", "attempting auto-login");
                match services.auth().try_auto_login().await {
                    Ok(Some(session)) => {
                        info!(target: "ui.auth", "auto-login successful for {}", session.four_words);
                        auth.with_mut(|state| {
                            state.session = Some(session);
                            state.phase = AuthPhase::Authenticated;
                            state.error = None;
                        });
                    }
                    Ok(None) => {
                        info!(target: "ui.auth", "no identity available for auto-login");
                    }
                    Err(err) => {
                        info!(target: "ui.auth", "auto-login failed: {err}");
                    }
                }
            }
        }
    });

    // Refresh directory when authenticated (debounced to prevent network cascades
    // when auth state flickers during startup or reconnection)
    use_future(move || {
        let services = services_for_directory.clone();
        async move {
            if matches!(phase, AuthPhase::Authenticated)
                && let Err(err) = services.directory().refresh_debounced().await
            {
                error!("failed to refresh directory snapshot: {err}");
            }
        }
    });

    // Trigger onboarding tour for new users after first authentication
    use_effect(move || {
        if matches!(phase, AuthPhase::Authenticated) && !tour_triggered() {
            // Mark as triggered to prevent re-triggering
            tour_triggered.set(true);

            // Check if user should see onboarding (first run or version upgrade)
            // For now, start tour for demonstration - in production, check AppConfig
            // TODO: Integrate with AppConfig.should_show_onboarding() when service is available
            let should_show = !tour_state.read().skipped;
            if should_show {
                info!(target: "ui.onboarding", "Starting onboarding tour for authenticated user");
                tour_state.with_mut(|state| {
                    state.start();
                });
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
                div {
                    class: "flex flex-col gap-2 text-sm",
                    style: format!("color: {};", colors::TEXT_SECONDARY),
                    span {
                        "Need a vault? "
                        Link {
                            to: Route::CreateIdentityRoute {},
                            class: "hover:underline",
                            style: format!("color: {};", colors::PRIMARY),
                            "Create one"
                        }
                        " or "
                        Link {
                            to: Route::RecoverIdentityRoute {},
                            class: "hover:underline",
                            style: format!("color: {};", colors::PRIMARY),
                            "recover"
                        }
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
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Four words"
                    }
                    input {
                        r#type: "text",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        placeholder: "forest-ocean-light-house",
                        disabled: busy,
                        value: "{four_words}",
                        oninput: move |evt| four_words.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Vault password"
                    }
                    input {
                        r#type: "password",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        placeholder: "********",
                        disabled: busy,
                        value: "{password}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                }
                button {
                    class: "rounded-lg px-4 py-3 font-semibold shadow-lg transition disabled:opacity-50",
                    style: button::primary(),
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
                div {
                    class: "flex flex-col gap-2 text-sm",
                    style: format!("color: {};", colors::TEXT_SECONDARY),
                    span {
                        "Already have a vault? "
                        Link {
                            to: Route::LoginRoute {},
                            class: "hover:underline",
                            style: format!("color: {};", colors::PRIMARY),
                            "Sign in"
                        }
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
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Display name"
                    }
                    input {
                        r#type: "text",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        placeholder: "Aurora Station",
                        disabled: busy,
                        value: "{display_name}",
                        oninput: move |evt| display_name.set(evt.value()),
                    }
                }
                div {
                    class: "rounded-lg border p-4",
                    style: format!(
                        "background-color: {}; border-color: {};",
                        colors::SURFACE_BG,
                        colors::BORDER_DEFAULT
                    ),
                    span {
                        class: "text-xs uppercase tracking-[0.4em]",
                        style: format!("color: {};", colors::TEXT_MUTED),
                        "four words"
                    }
                    p {
                        class: "mt-2 font-mono text-lg break-words",
                        style: format!("color: {};", colors::PRIMARY),
                        "{preview_words().as_str()}"
                    }
                    button {
                        class: "mt-3 text-sm font-semibold",
                        style: format!("color: {};", colors::PRIMARY),
                        r#type: "button",
                        disabled: busy,
                        onclick: move |_| preview_words.set(sample_words_from_core()),
                        "Refresh words"
                    }
                }
                label { class: "flex flex-col gap-2",
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Password"
                    }
                    input {
                        r#type: "password",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        disabled: busy,
                        value: "{password}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Confirm password"
                    }
                    input {
                        r#type: "password",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        disabled: busy,
                        value: "{confirm}",
                        oninput: move |evt| confirm.set(evt.value()),
                    }
                }
                button {
                    class: "rounded-lg px-4 py-3 font-semibold shadow-lg transition disabled:opacity-50",
                    style: button::primary(),
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
                div {
                    class: "flex flex-col gap-2 text-sm",
                    style: format!("color: {};", colors::TEXT_SECONDARY),
                    span {
                        "Remembered your password? "
                        Link {
                            to: Route::LoginRoute {},
                            class: "hover:underline",
                            style: format!("color: {};", colors::PRIMARY),
                            "Sign in"
                        }
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
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Mnemonic phrase"
                    }
                    textarea {
                        class: "h-32 rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        placeholder: "abandon ability able about...",
                        disabled: busy,
                        value: "{mnemonic}",
                        oninput: move |evt| mnemonic.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Passphrase (optional)"
                    }
                    input {
                        r#type: "text",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        disabled: busy,
                        value: "{passphrase}",
                        oninput: move |evt| passphrase.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Display name"
                    }
                    input {
                        r#type: "text",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        disabled: busy,
                        value: "{display_name}",
                        oninput: move |evt| display_name.set(evt.value()),
                    }
                }
                label { class: "flex flex-col gap-2",
                    span {
                        class: "text-sm font-medium",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "Vault password"
                    }
                    input {
                        r#type: "password",
                        class: "rounded-lg px-4 py-3 focus:outline-none",
                        style: input::default(),
                        disabled: busy,
                        value: "{password}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                }
                button {
                    class: "rounded-lg px-4 py-3 font-semibold shadow-lg transition disabled:opacity-50",
                    style: button::primary(),
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

    // Single-pass entity categorization (O(n) instead of O(n*6))
    let categorized = CategorizedEntities::from_entities(&snapshot.entities);

    let stats = vec![
        StatItem {
            label: "Organizations",
            value: categorized.organizations.len(),
        },
        StatItem {
            label: "Communities",
            value: categorized.communities.len(),
        },
        StatItem {
            label: "Projects",
            value: categorized.projects.len(),
        },
        StatItem {
            label: "Groups",
            value: categorized.groups.len(),
        },
        StatItem {
            label: "Channels",
            value: categorized.channels.len(),
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
                personal: categorized.personal_groups,
                communities: categorized.communities,
                organizations: categorized.organizations,
                projects: categorized.projects,
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
        div {
            class: "rounded-2xl border p-6 shadow-lg",
            style: format!(
                "background: linear-gradient(to right, {}40, {}20); border-color: {};",
                colors::PRIMARY_HOVER,
                colors::PRIMARY,
                colors::BORDER_DEFAULT
            ),
            span {
                class: "text-xs uppercase tracking-[0.4em]",
                style: format!("color: {};", colors::PRIMARY),
                "Communitas"
            }
            h2 {
                class: "mt-2 text-3xl font-semibold",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "Welcome back, {props.display_name}"
            }
            p {
                style: format!("color: {};", colors::TEXT_SECONDARY),
                "Your local-first collaboration hub"
            }
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
                    div {
                        class: "rounded-2xl border p-4",
                        style: format!(
                            "background-color: {}; border-color: {};",
                            colors::SURFACE_CARD,
                            colors::BORDER_DEFAULT
                        ),
                        span {
                            class: "text-sm uppercase tracking-[0.3em]",
                            style: format!("color: {};", colors::TEXT_MUTED),
                            "{item.label}"
                        }
                        h3 {
                            class: "text-2xl font-semibold",
                            style: format!("color: {};", colors::TEXT_PRIMARY),
                            "{item.value}"
                        }
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
            h3 {
                class: "text-2xl font-semibold",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "Your Spaces"
            }
            div { class: "grid gap-4 lg:grid-cols-2",
                EntityListPanel {
                    title: "Personal",
                    entities: props.personal.clone(),
                    empty_label: "No personal groups yet",
                }
                EntityListPanel {
                    title: "Communities",
                    entities: props.communities.clone(),
                    empty_label: "No communities yet",
                }
                EntityListPanel {
                    title: "Organizations",
                    entities: props.organizations.clone(),
                    empty_label: "No organizations yet",
                }
                EntityListPanel {
                    title: "Projects",
                    entities: props.projects.clone(),
                    empty_label: "No projects yet",
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
}

#[component]
fn EntityListPanel(props: EntityListPanelProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    rsx! {
        div {
            class: "rounded-2xl border p-4",
            style: format!(
                "background-color: {}; border-color: {};",
                colors::SURFACE_CARD,
                colors::BORDER_DEFAULT
            ),
            h4 {
                class: "text-lg font-semibold",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "{props.title}"
            }
            if props.entities.is_empty() {
                p {
                    class: "text-sm",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    "{props.empty_label}"
                }
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
                                class: "flex flex-col rounded-xl border px-3 py-2 text-left transition-colors",
                                style: format!(
                                    "background-color: {}; border-color: {};",
                                    colors::SURFACE_ELEVATED,
                                    colors::BORDER_DEFAULT
                                ),
                                onclick: move |_| {
                                    record_entity_visit(services.clone(), nav_key.clone());
                                },
                                span {
                                    class: "text-sm font-semibold",
                                    style: format!("color: {};", colors::TEXT_PRIMARY),
                                    "{entity_name}"
                                }
                                span {
                                    class: "text-xs",
                                    style: format!("color: {};", colors::TEXT_MUTED),
                                    "{member_count} members"
                                }
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
    let mut selected_thread = use_signal(|| Option::<String>::None);
    let mut reply_to = use_signal(|| Option::<communitas_ui_api::Message>::None);

    let handle_thread_select = move |thread_id: String| {
        selected_thread.set(Some(thread_id.clone()));
        reply_to.set(None); // Clear reply when switching threads
        info!(target = "ui.messages", event = "thread_selected", thread_id = %thread_id);
    };

    render_authenticated_page(
        "Messages",
        rsx! {
            div {
                class: "flex gap-6 h-[calc(100vh-16rem)]",
                // Thread list sidebar
                div {
                    class: "w-80 flex-shrink-0 rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden",
                    components::ThreadListSidebar {
                        selected_thread_id: selected_thread(),
                        on_thread_select: handle_thread_select,
                    }
                }
                // Message area
                div {
                    class: "flex-1 rounded-xl border border-slate-800 bg-slate-900/50 flex flex-col overflow-hidden",
                    if let Some(thread_id) = selected_thread() {
                        // Thread selected - show message list and composer
                        div {
                            class: "flex-1 overflow-hidden",
                            components::MessageList {
                                thread_id: thread_id.clone(),
                                on_reply: move |msg: communitas_ui_api::Message| {
                                    reply_to.set(Some(msg));
                                },
                            }
                        }
                        // Composer at bottom
                        components::MessageComposer {
                            thread_id: thread_id.clone(),
                            reply_to: reply_to(),
                            on_send: move |msg: communitas_ui_api::Message| {
                                info!(target = "ui.messages", event = "message_sent", message_id = %msg.id);
                                reply_to.set(None);
                            },
                            on_cancel_reply: Some(EventHandler::new(move |_| {
                                reply_to.set(None);
                            })),
                        }
                    } else {
                        // No thread selected - show empty state
                        div {
                            class: "flex-1 flex items-center justify-center",
                            div {
                                class: "text-center",
                                div {
                                    class: "w-20 h-20 rounded-full bg-slate-800 flex items-center justify-center mb-4 mx-auto",
                                    span { class: "text-3xl", "💬" }
                                }
                                p { class: "text-slate-400", "Select a conversation" }
                                p { class: "text-slate-500 text-sm mt-1", "Choose a thread from the sidebar to start chatting" }
                            }
                        }
                    }
                }
            }
        },
    )
}

#[component]
fn ProjectsRoute() -> Element {
    render_authenticated_page(
        "Projects",
        rsx! {
            components::kanban::BoardListPage {}
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
    let mut tour_state = use_tour_state();

    let start_tour = move |_| {
        tour_state.with_mut(|state| {
            state.start();
        });
        info!(target: "ui.onboarding", "Tour started from Settings");
    };

    render_authenticated_page(
        "More",
        rsx! {
            div { class: "space-y-6",
                // Version info section
                div {
                    class: "rounded-xl border border-slate-800 bg-slate-950/60 p-6",
                    h2 { class: "mb-4 text-lg font-semibold text-slate-100", "About" }
                    div { class: "space-y-2 text-sm text-slate-400",
                        p { "Version: {version::CURRENT.version}" }
                        p { "Build: {version::CURRENT.commit_hash}" }
                        p { "Platform: {version::CURRENT.target}" }
                    }
                }

                // Help section with tour restart
                div {
                    class: "rounded-xl border border-slate-800 bg-slate-950/60 p-6",
                    h2 { class: "mb-4 text-lg font-semibold text-slate-100", "Help" }
                    p { class: "mb-4 text-sm text-slate-400",
                        "New to Communitas? Take a guided tour of the main features."
                    }
                    button {
                        class: "rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500 focus:outline-none focus:ring-2 focus:ring-emerald-400 focus:ring-offset-2 focus:ring-offset-slate-950",
                        r#type: "button",
                        onclick: start_tour,
                        "Start Tour"
                    }
                }

                // Placeholder for additional settings
                PlaceholderPanel { title: "Settings".into(), body: "Additional settings and MCP tools coming soon.".into() }
            }
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
            components::DriveBrowser { entity_id: entity_id.clone() }
        },
    )
}

#[component]
fn ProjectBoardRoute(project_id: String) -> Element {
    render_authenticated_page(
        "Project Board",
        rsx! {
            components::kanban::BoardView { board_id: project_id }
        },
    )
}

#[component]
fn ContactDetailRoute(contact_id: String) -> Element {
    render_authenticated_page(
        "Contact",
        rsx! {
            ContactDetailView { contact_id }
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
    let mut reply_to = use_signal(|| Option::<communitas_ui_api::Message>::None);
    let mut show_chat = use_signal(|| true);

    if let Some(entity) = entity {
        let entity_for_header = entity.clone();
        let thread_id = props.entity_id.clone();
        let thread_id_for_composer = thread_id.clone();
        let entity_id_for_edit = props.entity_id.clone();
        let entity_id_for_leave = props.entity_id.clone();
        let entity_id_for_members = props.entity_id.clone();

        rsx! {
            div {
                class: "entity-detail-view flex flex-col h-[calc(100vh-16rem)]",
                role: "main",
                aria_label: format!("{} details", entity.name),
                // Entity header
                EntityHeader {
                    entity: entity_for_header,
                    on_edit: move |_| {
                        info!(target = "ui.entity", event = "edit_clicked", entity_id = %entity_id_for_edit);
                    },
                    on_leave: move |_| {
                        info!(target = "ui.entity", event = "leave_clicked", entity_id = %entity_id_for_leave);
                    },
                }
                // Content area with tabs
                div {
                    class: "flex border-b border-slate-800 mt-4",
                    button {
                        class: format!(
                            "px-4 py-2 text-sm font-medium transition {}",
                            if show_chat() { "text-emerald-400 border-b-2 border-emerald-400" } else { "text-slate-400 hover:text-slate-200" }
                        ),
                        onclick: move |_| show_chat.set(true),
                        "Chat"
                    }
                    button {
                        class: format!(
                            "px-4 py-2 text-sm font-medium transition {}",
                            if !show_chat() { "text-emerald-400 border-b-2 border-emerald-400" } else { "text-slate-400 hover:text-slate-200" }
                        ),
                        onclick: move |_| show_chat.set(false),
                        "Members"
                    }
                }
                // Tab content
                if show_chat() {
                    // Chat panel
                    div {
                        class: "flex-1 flex flex-col overflow-hidden mt-4",
                        div {
                            class: "flex-1 overflow-hidden rounded-lg border border-slate-800 bg-slate-900/30",
                            components::MessageList {
                                thread_id: thread_id.clone(),
                                on_reply: move |msg: communitas_ui_api::Message| {
                                    reply_to.set(Some(msg));
                                },
                            }
                        }
                        div {
                            class: "mt-2",
                            components::MessageComposer {
                                thread_id: thread_id_for_composer,
                                reply_to: reply_to(),
                                on_send: move |msg: communitas_ui_api::Message| {
                                    info!(target = "ui.entity", event = "message_sent", message_id = %msg.id);
                                    reply_to.set(None);
                                },
                                on_cancel_reply: Some(EventHandler::new(move |_| {
                                    reply_to.set(None);
                                })),
                            }
                        }
                    }
                } else {
                    // Members panel
                    EntityMemberList {
                        entity_id: entity_id_for_members.clone(),
                    }
                }
            }
        }
    } else {
        rsx! {
            EntityNotFound { entity_id: props.entity_id.clone() }
        }
    }
}

/// Entity header showing name, type badge, description, and actions.
#[derive(Props, Clone, PartialEq)]
struct EntityHeaderProps {
    entity: UnifiedEntity,
    on_edit: EventHandler<()>,
    on_leave: EventHandler<()>,
}

#[component]
fn EntityHeader(props: EntityHeaderProps) -> Element {
    let entity = &props.entity;
    let type_badge = match entity.entity_type {
        UnifiedEntityType::Organization => match entity.category {
            Some(OrganizationCategory::Community) => (
                "Community",
                "bg-purple-500/20 text-purple-300 border-purple-500/30",
            ),
            Some(OrganizationCategory::Organization) | None => (
                "Organization",
                "bg-blue-500/20 text-blue-300 border-blue-500/30",
            ),
        },
        UnifiedEntityType::Project => (
            "Project",
            "bg-amber-500/20 text-amber-300 border-amber-500/30",
        ),
        UnifiedEntityType::Group => (
            "Group",
            "bg-emerald-500/20 text-emerald-300 border-emerald-500/30",
        ),
        UnifiedEntityType::Channel => {
            ("Channel", "bg-cyan-500/20 text-cyan-300 border-cyan-500/30")
        }
        UnifiedEntityType::Person => ("Person", "bg-pink-500/20 text-pink-300 border-pink-500/30"),
    };

    rsx! {
        header {
            class: "entity-header rounded-xl border border-slate-800 bg-slate-900/50 p-5",
            role: "banner",
            div {
                class: "flex flex-col gap-3 md:flex-row md:items-start md:justify-between",
                // Entity info
                div {
                    class: "flex-1",
                    // Type badge
                    span {
                        class: format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border {}", type_badge.1),
                        "{type_badge.0}"
                    }
                    // Name
                    h1 {
                        class: "mt-2 text-2xl font-semibold text-white",
                        "{entity.name}"
                    }
                    // Description
                    if !entity.description.is_empty() {
                        p {
                            class: "mt-1 text-sm text-slate-400",
                            "{entity.description}"
                        }
                    }
                    // Member count
                    div {
                        class: "mt-3 flex items-center gap-4 text-sm text-slate-500",
                        span {
                            class: "flex items-center gap-1",
                            span { class: "text-slate-600", "👥" }
                            {format!("{} member{}", entity.member_count, if entity.member_count != 1 { "s" } else { "" })}
                        }
                    }
                }
                // Actions
                div {
                    class: "flex gap-2 mt-3 md:mt-0",
                    button {
                        class: "px-4 py-2 text-sm font-medium text-slate-300 border border-slate-700 rounded-lg hover:border-slate-600 hover:text-slate-200 transition",
                        onclick: move |_| props.on_edit.call(()),
                        "Edit"
                    }
                    button {
                        class: "px-4 py-2 text-sm font-medium text-red-400 border border-red-500/30 rounded-lg hover:border-red-500/50 hover:bg-red-500/10 transition",
                        onclick: move |_| props.on_leave.call(()),
                        "Leave"
                    }
                }
            }
        }
    }
}

/// Member list panel for entity.
#[derive(Props, Clone, PartialEq)]
struct EntityMemberListProps {
    entity_id: String,
}

#[component]
fn EntityMemberList(props: EntityMemberListProps) -> Element {
    // TODO: Load actual members from service based on props.entity_id
    let _entity_id = &props.entity_id;

    rsx! {
        aside {
            class: "entity-member-list flex-1 overflow-y-auto mt-4 rounded-lg border border-slate-800 bg-slate-900/30 p-4",
            role: "complementary",
            aria_label: "Entity members",
            h3 {
                class: "text-sm font-medium text-slate-400 uppercase tracking-wider mb-4",
                "Members"
            }
            // Placeholder members
            div {
                class: "space-y-2",
                for i in 0..5 {
                    div {
                        key: "{i}",
                        class: "flex items-center gap-3 p-2 rounded-lg hover:bg-slate-800/50 transition",
                        // Avatar
                        div {
                            class: "w-8 h-8 rounded-full bg-slate-700 flex items-center justify-center text-sm text-slate-300",
                            "{(b'A' + i as u8) as char}"
                        }
                        // Info
                        div {
                            class: "flex-1 min-w-0",
                            p {
                                class: "text-sm font-medium text-slate-200 truncate",
                                "Member {i + 1}"
                            }
                            p {
                                class: "text-xs text-slate-500",
                                "four-word-identity"
                            }
                        }
                        // Presence dot placeholder
                        span {
                            class: "w-2 h-2 rounded-full bg-emerald-400",
                            title: "Online",
                        }
                    }
                }
            }
            // Load more hint
            p {
                class: "mt-4 text-center text-xs text-slate-600",
                "Member list integration coming soon"
            }
        }
    }
}

/// Not found state for entity.
#[derive(Props, Clone, PartialEq)]
struct EntityNotFoundProps {
    entity_id: String,
}

#[component]
fn EntityNotFound(props: EntityNotFoundProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-16",
            role: "alert",
            div {
                class: "w-16 h-16 rounded-full bg-slate-800 flex items-center justify-center mb-4",
                span { class: "text-2xl", "🔍" }
            }
            h2 {
                class: "text-xl font-semibold text-slate-200",
                "Entity not found"
            }
            p {
                class: "mt-2 text-sm text-slate-500",
                "The entity \"{props.entity_id}\" could not be found."
            }
            Link {
                to: Route::DashboardRoute {},
                class: "mt-4 px-4 py-2 text-sm font-medium text-emerald-400 border border-emerald-500/30 rounded-lg hover:bg-emerald-500/10 transition",
                "Return to Dashboard"
            }
        }
    }
}

// --- Contact Detail View ---

#[derive(Props, PartialEq, Clone)]
struct ContactDetailViewProps {
    contact_id: String,
}

#[component]
fn ContactDetailView(props: ContactDetailViewProps) -> Element {
    let directory = use_directory_snapshot();
    let snapshot = directory();
    let contact = snapshot
        .contacts
        .iter()
        .find(|c| c.id == props.contact_id)
        .cloned();

    let mut reply_to = use_signal(|| Option::<communitas_ui_api::Message>::None);

    // Mock presence for now - will be wired to PresenceService
    let presence = PresenceStatus::Online;
    let last_seen: Option<u64> = None;

    if let Some(contact) = contact {
        let contact_id_for_edit = props.contact_id.clone();
        let contact_id_for_block = props.contact_id.clone();
        let contact_id_for_remove = props.contact_id.clone();
        let thread_id = props.contact_id.clone();
        let thread_id_for_composer = thread_id.clone();

        rsx! {
            div {
                class: "contact-detail-view flex flex-col h-[calc(100vh-16rem)]",
                role: "main",
                aria_label: format!("{} contact details", contact.display_name),
                // Contact header card
                ContactCard {
                    contact: contact.clone(),
                    presence,
                    last_seen,
                    on_edit: move |_| {
                        info!(target = "ui.contact", event = "edit_clicked", contact_id = %contact_id_for_edit);
                    },
                    on_block: move |_| {
                        info!(target = "ui.contact", event = "block_clicked", contact_id = %contact_id_for_block);
                    },
                    on_remove: move |_| {
                        info!(target = "ui.contact", event = "remove_clicked", contact_id = %contact_id_for_remove);
                    },
                }
                // DM chat panel
                div {
                    class: "flex-1 flex flex-col overflow-hidden mt-4",
                    div {
                        class: "flex-1 overflow-hidden rounded-lg border border-slate-800 bg-slate-900/30",
                        components::MessageList {
                            thread_id: thread_id.clone(),
                            on_reply: move |msg: communitas_ui_api::Message| {
                                reply_to.set(Some(msg));
                            },
                        }
                    }
                    div {
                        class: "mt-2",
                        components::MessageComposer {
                            thread_id: thread_id_for_composer,
                            reply_to: reply_to(),
                            on_send: move |msg: communitas_ui_api::Message| {
                                info!(target = "ui.contact", event = "message_sent", message_id = %msg.id);
                                reply_to.set(None);
                            },
                            on_cancel_reply: Some(EventHandler::new(move |_| {
                                reply_to.set(None);
                            })),
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            ContactNotFound { contact_id: props.contact_id.clone() }
        }
    }
}

/// Contact card showing avatar, name, presence badge, and actions.
#[derive(Props, Clone, PartialEq)]
struct ContactCardProps {
    contact: UnifiedContact,
    presence: PresenceStatus,
    last_seen: Option<u64>,
    on_edit: EventHandler<()>,
    on_block: EventHandler<()>,
    on_remove: EventHandler<()>,
}

#[component]
fn ContactCard(props: ContactCardProps) -> Element {
    let contact = &props.contact;
    let first_char = contact
        .display_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    rsx! {
        header {
            class: "contact-card rounded-xl border border-slate-800 bg-slate-900/50 p-5",
            role: "banner",
            div {
                class: "flex flex-col gap-4 md:flex-row md:items-center md:justify-between",
                // Contact info
                div {
                    class: "flex items-center gap-4",
                    // Avatar with presence dot
                    div {
                        class: "relative",
                        div {
                            class: "w-16 h-16 rounded-full bg-slate-700 flex items-center justify-center text-2xl text-slate-200 font-semibold",
                            "{first_char}"
                        }
                        // Presence dot in corner
                        div {
                            class: "absolute -bottom-0.5 -right-0.5",
                            components::PresenceDot {
                                status: props.presence,
                                size: "md",
                            }
                        }
                    }
                    // Name and details
                    div {
                        class: "flex flex-col",
                        h1 {
                            class: "text-xl font-semibold text-white",
                            "{contact.display_name}"
                        }
                        // Four-word ID
                        p {
                            class: "text-sm text-slate-500 font-mono",
                            "{contact.id}"
                        }
                        // Presence badge with text
                        div {
                            class: "mt-2",
                            components::PresenceBadge {
                                status: props.presence,
                                size: "sm",
                            }
                        }
                        // Last seen (if offline)
                        if props.presence == PresenceStatus::Offline {
                            if let Some(last_seen) = props.last_seen {
                                p {
                                    class: "text-xs text-slate-600 mt-1",
                                    "Last seen {format_relative_time(last_seen)}"
                                }
                            }
                        }
                    }
                }
                // Actions
                div {
                    class: "flex gap-2 mt-3 md:mt-0",
                    button {
                        class: "px-4 py-2 text-sm font-medium text-slate-300 border border-slate-700 rounded-lg hover:border-slate-600 hover:text-slate-200 transition",
                        onclick: move |_| props.on_edit.call(()),
                        "Edit"
                    }
                    button {
                        class: "px-4 py-2 text-sm font-medium text-amber-400 border border-amber-500/30 rounded-lg hover:border-amber-500/50 hover:bg-amber-500/10 transition",
                        onclick: move |_| props.on_block.call(()),
                        "Block"
                    }
                    button {
                        class: "px-4 py-2 text-sm font-medium text-red-400 border border-red-500/30 rounded-lg hover:border-red-500/50 hover:bg-red-500/10 transition",
                        onclick: move |_| props.on_remove.call(()),
                        "Remove"
                    }
                }
            }
        }
    }
}

/// Not found state for contact.
#[derive(Props, Clone, PartialEq)]
struct ContactNotFoundProps {
    contact_id: String,
}

#[component]
fn ContactNotFound(props: ContactNotFoundProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-16",
            role: "alert",
            div {
                class: "w-16 h-16 rounded-full bg-slate-800 flex items-center justify-center mb-4",
                span { class: "text-2xl", "👤" }
            }
            h2 {
                class: "text-xl font-semibold text-slate-200",
                "Contact not found"
            }
            p {
                class: "mt-2 text-sm text-slate-500",
                "The contact \"{props.contact_id}\" could not be found."
            }
            Link {
                to: Route::ContactsRoute {},
                class: "mt-4 px-4 py-2 text-sm font-medium text-emerald-400 border border-emerald-500/30 rounded-lg hover:bg-emerald-500/10 transition",
                "Return to Contacts"
            }
        }
    }
}

/// Format timestamp as relative time.
fn format_relative_time(timestamp_ms: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let diff_ms = now.saturating_sub(timestamp_ms);
    let diff_secs = diff_ms / 1000;
    let diff_mins = diff_secs / 60;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_mins < 1 {
        "just now".to_string()
    } else if diff_mins < 60 {
        format!("{} min ago", diff_mins)
    } else if diff_hours < 24 {
        format!(
            "{} hour{} ago",
            diff_hours,
            if diff_hours == 1 { "" } else { "s" }
        )
    } else {
        format!(
            "{} day{} ago",
            diff_days,
            if diff_days == 1 { "" } else { "s" }
        )
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
                // Notification area with missed call badge
                if session.is_some() {
                    div {
                        class: "flex items-center gap-4",
                        // Missed call badge indicator
                        div {
                            class: "relative",
                            title: "Missed calls",
                            components::ReactiveMissedCallBadge {}
                        }
                        // Identity switcher with dropdown for quick switching
                        components::IdentitySwitcher {
                            on_logout: move |_| logout_action.send(()),
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
                    nav {
                        class: "w-full max-w-sm space-y-4 rounded-2xl border p-4 lg:w-80",
                        style: format!(
                            "background-color: {}; border-color: {};",
                            colors::SURFACE_BG,
                            colors::BORDER_DEFAULT
                        ),
                        span {
                            class: "text-xs uppercase tracking-[0.4em]",
                            style: format!("color: {};", colors::TEXT_MUTED),
                            "Navigation"
                        }
                        {nav_links.into_iter().map(|item| {
                            let route = (item.to)();
                            let active = current_route == route;
                            let link_route = route.clone();
                            let label = item.label;
                            rsx! {
                                Link {
                                    to: link_route,
                                    class: "block rounded-xl border px-4 py-3 transition",
                                    style: if active {
                                        format!(
                                            "border-color: {}; background-color: {}; color: {};",
                                            colors::PRIMARY,
                                            "rgba(16, 185, 129, 0.1)", // PRIMARY with 10% opacity
                                            colors::PRIMARY
                                        )
                                    } else {
                                        format!(
                                            "border-color: transparent; color: {};",
                                            colors::TEXT_SECONDARY
                                        )
                                    },
                                    onclick: move |_| {
                                        info!(target = "ui.nav", event = "navigate_click", destination = label);
                                    },
                                    span { class: "text-base font-semibold", "{item.label}" }
                                    p {
                                        class: "text-sm",
                                        style: format!("color: {};", colors::TEXT_MUTED),
                                        "{item.description}"
                                    }
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
        Route::ContactDetailRoute { contact_id } | Route::ContactChatRoute { contact_id } => {
            Some(RouteNavigationEvent::Contact(contact_id.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_service::UiServices;
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
    fn route_event_detects_contact_detail() {
        let route = Route::ContactDetailRoute {
            contact_id: "bob".into(),
        };
        match route_navigation_event(&route) {
            Some(RouteNavigationEvent::Contact(contact)) => assert_eq!(contact, "bob"),
            _ => panic!("expected contact event for detail route"),
        }
    }

    #[test]
    #[ignore = "requires UiServices test infrastructure update"]
    fn login_route_renders_copy() {
        let env = TestEnv::new();
        let html = render_route_html("/login", env.services(), AuthState::default());
        assert!(
            html.contains("Unlock your Communitas vault"),
            "login copy missing:\n{html}"
        );
    }

    #[test]
    #[ignore = "requires UiServices test infrastructure update"]
    fn create_identity_route_shows_generated_words() {
        let env = TestEnv::new();
        let html = render_route_html("/create", env.services(), AuthState::default());
        assert!(
            html.contains("Create identity") && html.contains("four words"),
            "create identity layout missing copy:\n{html}"
        );
    }

    #[test]
    #[ignore = "requires UiServices test infrastructure update"]
    fn recover_identity_route_includes_mnemonic_field() {
        let env = TestEnv::new();
        let html = render_route_html("/recover", env.services(), AuthState::default());
        assert!(
            html.contains("Mnemonic phrase") && html.contains("Recover identity"),
            "recover identity layout missing mnemonic instructions:\n{html}"
        );
    }

    #[test]
    #[ignore = "requires UiServices test infrastructure update"]
    fn unauthenticated_routes_redirect_to_login() {
        let env = TestEnv::new();
        let html = render_route_html("/messages", env.services(), AuthState::default());
        assert!(
            html.contains("Redirecting"),
            "expected redirect placeholder when user is logged out:\n{html}"
        );
    }

    #[test]
    #[ignore = "requires UiServices test infrastructure update"]
    fn dashboard_renders_authenticated_session_name() {
        let env = TestEnv::new();
        let authed = AuthState {
            phase: AuthPhase::Authenticated,
            session: Some(AuthSession {
                pubkey_hex: "deadbeef".into(),
                four_words: "forest-ocean-light-house".into(),
                display_name: "Test Pilot".into(),
                device_name: "Test Device".into(),
                expires_at: u64::MAX, // Test session never expires
            }),
            ..Default::default()
        };
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
            // SAFETY: Tests run in isolation, setting env var for storage path
            unsafe {
                std::env::set_var("COMMUNITAS_STORAGE_PATH", temp.path());
            }
            let services = UiServices::bootstrap().expect("ui services");
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
            h5 {
                class: "text-xs uppercase tracking-[0.3em]",
                style: format!("color: {};", colors::TEXT_MUTED),
                "{props.title}"
            }
            {props.entities.iter().take(5).map(|entity| {
                let services = services.clone();
                let nav_key = nav_key_for(entity);
                let route = entity_route(entity);
                let name = entity.name.clone();
                rsx! {
                    Link {
                        to: route.clone(),
                        class: "block rounded-xl border px-3 py-2 text-sm transition",
                        style: format!(
                            "background-color: {}; border-color: {}; color: {};",
                            colors::SURFACE_BG,
                            colors::BORDER_DEFAULT,
                            colors::TEXT_PRIMARY
                        ),
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
        main {
            class: "min-h-screen px-4 py-16 sm:px-0",
            style: format!("background-color: {}; color: {};", colors::SURFACE_BG, colors::TEXT_PRIMARY),
            div {
                class: "mx-auto flex max-w-xl flex-col gap-6 rounded-3xl border p-8",
                style: format!(
                    "background-color: {}; border-color: {};",
                    colors::SURFACE_CARD,
                    colors::BORDER_DEFAULT
                ),
                div { class: "flex flex-col gap-1",
                    span {
                        class: "text-xs uppercase tracking-[0.5em]",
                        style: format!("color: {};", colors::PRIMARY),
                        "Communitas"
                    }
                    h1 {
                        class: "text-3xl font-semibold tracking-tight",
                        style: format!("color: {};", colors::TEXT_PRIMARY),
                        "{props.title}"
                    }
                    p {
                        class: "text-sm",
                        style: format!("color: {};", colors::TEXT_SECONDARY),
                        "{props.subtitle}"
                    }
                }
                if let Some(err) = props.error {
                    div {
                        class: "rounded-xl border px-4 py-3 text-sm",
                        style: format!(
                            "border-color: {}; background-color: rgba(239, 68, 68, 0.1); color: {};",
                            colors::DANGER,
                            colors::DANGER
                        ),
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
        div {
            class: format!("animate-pulse rounded {class}"),
            style: format!("background-color: {};", colors::SURFACE_ELEVATED),
        }
    }
}

#[component]
fn SkeletonWelcomeCard() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border p-6 shadow-lg",
            style: format!(
                "background: linear-gradient(to right, {}40, {}20); border-color: {};",
                colors::SURFACE_ELEVATED,
                colors::SURFACE_CARD,
                colors::BORDER_DEFAULT
            ),
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
                div {
                    class: "rounded-2xl border p-4",
                    style: format!(
                        "background-color: {}; border-color: {};",
                        colors::SURFACE_CARD,
                        colors::BORDER_DEFAULT
                    ),
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
                    div {
                        class: "rounded-2xl border p-4",
                        style: format!(
                            "background-color: {}; border-color: {};",
                            colors::SURFACE_CARD,
                            colors::BORDER_DEFAULT
                        ),
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
