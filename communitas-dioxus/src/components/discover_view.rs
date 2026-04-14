// SPDX-License-Identifier: MIT OR Apache-2.0

//! Named-group discovery view.
//!
//! Provides the client-side surface for the Phase C + C.2 distributed
//! discovery index: tag/name query search, shard-only "nearby" witness,
//! and the inline request-access flow. Mirrors `x0x group discover*`
//! plus `x0x group request-access`.
//!
//! Tiers exposed here correspond to the spec in
//! `docs/design/named-groups-full-model.md`:
//!
//! - Tier 2 (shard-based): handled by the "All" mode, which merges
//!   locally-owned cards, bridge cache, and shard cache via
//!   `GET /groups/discover?q=`.
//! - Tier 3 (presence-social): the "Nearby" mode reads the C.2
//!   shard-only witness via `GET /groups/discover/nearby`.
//!
//! `Hidden` cards cannot reach this view; `ListedToContacts` cards
//! never show in Nearby (privacy invariant enforced by the daemon).

use communitas_x0x_client::{
    GroupAdmission, GroupCard, GroupConfidentiality, GroupDiscoverability, GroupReadAccess,
    GroupWriteAccess, X0xClient,
};
use dioxus::prelude::*;
use tracing::{info, warn};

use crate::design_tokens::{radius, semantic, spacing, typography};

/// How often to refresh the active card list, in seconds. Discovery
/// converges over gossip; polling every 10s matches the daemon's
/// digest cadence without being spammy.
const REFRESH_INTERVAL_SECS: u64 = 10;

/// Which discovery mode the user is looking at.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DiscoverMode {
    /// `GET /groups/discover?q=<query>` — merged local + shard cache.
    All,
    /// `GET /groups/discover/nearby` — shard-only PublicDirectory witness.
    Nearby,
}

/// Outcome of the most-recent request-access submission for a given card.
#[derive(Clone, PartialEq, Eq)]
enum RequestStatus {
    /// No attempt yet.
    Idle,
    /// Request in flight.
    Pending,
    /// Daemon accepted the request.
    Submitted,
    /// Daemon rejected with an error message.
    Failed(String),
}

/// Root view rendered at `/discover`.
#[component]
pub fn DiscoverView() -> Element {
    let mut query = use_signal(String::new);
    let mut mode = use_signal(|| DiscoverMode::All);
    let mut cards = use_signal(Vec::<GroupCard>::new);
    let mut last_error = use_signal(|| None::<String>);
    let request_state = use_signal(std::collections::HashMap::<String, RequestStatus>::new);

    // Poll the daemon so newly-observed cards appear without a manual refresh.
    use_coroutine({
        move |_: UnboundedReceiver<()>| async move {
            let client = X0xClient::new();
            loop {
                let current_mode = mode();
                let current_query = query().trim().to_string();
                let result = match current_mode {
                    DiscoverMode::All => {
                        let q = if current_query.is_empty() {
                            None
                        } else {
                            Some(current_query.as_str())
                        };
                        client.discover_groups(q).await
                    }
                    DiscoverMode::Nearby => client.discover_groups_nearby().await,
                };
                match result {
                    Ok(mut list) => {
                        list.sort_by(|a, b| {
                            b.revision
                                .cmp(&a.revision)
                                .then_with(|| a.name.cmp(&b.name))
                        });
                        cards.set(list);
                        last_error.set(None);
                    }
                    Err(e) => {
                        warn!(target: "ui.discover", "discover fetch failed: {e}");
                        last_error.set(Some(format!("Discovery failed: {e}")));
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
            }
        }
    });

    let container_style = format!(
        "display: flex; \
         flex-direction: column; \
         gap: {}; \
         padding: {} {}; \
         height: 100%; \
         overflow-y: auto;",
        spacing::BASE,
        spacing::BASE,
        spacing::XL,
    );

    rsx! {
        section {
            style: "{container_style}",
            role: "region",
            aria_label: "Discover groups",

            h1 {
                style: format!(
                    "margin: 0; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {};",
                    typography::SIZE_LG,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY,
                ),
                "Discover groups"
            }

            DiscoverToolbar {
                query: query(),
                mode: mode(),
                on_query: move |val: String| query.set(val),
                on_mode: move |m: DiscoverMode| mode.set(m),
            }

            if let Some(err) = last_error() {
                ErrorBanner { message: err }
            }

            if cards.read().is_empty() {
                EmptyState { mode: mode(), query: query() }
            } else {
                ul {
                    style: format!(
                        "list-style: none; margin: 0; padding: 0; \
                         display: flex; flex-direction: column; gap: {};",
                        spacing::SM,
                    ),

                    for card in cards.read().iter().cloned() {
                        {
                            let group_id = card.group_id.clone();
                            let can_request = card.request_access_enabled
                                && card.policy_summary.admission == GroupAdmission::RequestAccess
                                && !card.withdrawn;
                            let status = request_state
                                .read()
                                .get(&group_id)
                                .cloned()
                                .unwrap_or(RequestStatus::Idle);
                            rsx! {
                                li {
                                    key: "{card.group_id}-{card.revision}",
                                    DiscoverCard {
                                        card: card.clone(),
                                        can_request,
                                        status: status.clone(),
                                        on_request: move |_| {
                                            if !can_request
                                                || matches!(status, RequestStatus::Pending | RequestStatus::Submitted)
                                            {
                                                return;
                                            }
                                            let gid = group_id.clone();
                                            let mut req_state = request_state;
                                            req_state
                                                .write()
                                                .insert(gid.clone(), RequestStatus::Pending);
                                            spawn(async move {
                                                let client = X0xClient::new();
                                                match client.create_join_request(&gid, None).await {
                                                    Ok(_) => {
                                                        info!(
                                                            target: "ui.discover",
                                                            "submitted join request for {gid}"
                                                        );
                                                        req_state
                                                            .write()
                                                            .insert(gid, RequestStatus::Submitted);
                                                    }
                                                    Err(e) => {
                                                        warn!(
                                                            target: "ui.discover",
                                                            "join request failed for {gid}: {e}"
                                                        );
                                                        req_state.write().insert(
                                                            gid,
                                                            RequestStatus::Failed(format!("{e}")),
                                                        );
                                                    }
                                                }
                                            });
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ToolbarProps {
    query: String,
    mode: DiscoverMode,
    on_query: EventHandler<String>,
    on_mode: EventHandler<DiscoverMode>,
}

#[component]
fn DiscoverToolbar(props: ToolbarProps) -> Element {
    let mode = props.mode;
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-wrap: wrap; \
                 align-items: center; \
                 gap: {};",
                spacing::SM,
            ),

            input {
                r#type: "search",
                value: "{props.query}",
                placeholder: "Search by tag, name, or ID…",
                aria_label: "Search groups",
                disabled: mode == DiscoverMode::Nearby,
                style: format!(
                    "flex: 1 1 260px; \
                     padding: {} {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     background: {}; \
                     color: {}; \
                     font-size: {};",
                    spacing::SM,
                    spacing::BASE,
                    semantic::BORDER_DEFAULT,
                    radius::LG,
                    semantic::BG_SECONDARY,
                    semantic::TEXT_PRIMARY,
                    typography::SIZE_SM,
                ),
                oninput: move |evt: Event<FormData>| props.on_query.call(evt.value().to_string()),
            }

            div {
                role: "tablist",
                aria_label: "Discovery mode",
                style: format!(
                    "display: inline-flex; \
                     gap: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     padding: 2px;",
                    spacing::XS,
                    semantic::BORDER_DEFAULT,
                    radius::LG,
                ),

                for (m, label) in [(DiscoverMode::All, "All"), (DiscoverMode::Nearby, "Nearby")] {
                    {
                        let is_active = mode == m;
                        rsx! {
                            button {
                                key: "{label}",
                                role: "tab",
                                aria_selected: if is_active { "true" } else { "false" },
                                style: format!(
                                    "padding: {} {}; \
                                     border: none; \
                                     border-radius: {}; \
                                     background: {}; \
                                     color: {}; \
                                     font-size: {}; \
                                     font-weight: {}; \
                                     cursor: pointer;",
                                    spacing::XS,
                                    spacing::BASE,
                                    radius::MD,
                                    if is_active { semantic::BG_TERTIARY } else { "transparent" },
                                    if is_active { semantic::PRIMARY } else { semantic::TEXT_SECONDARY },
                                    typography::SIZE_SM,
                                    if is_active { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_NORMAL },
                                ),
                                onclick: move |_| props.on_mode.call(m),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CardProps {
    card: GroupCard,
    can_request: bool,
    status: RequestStatus,
    on_request: EventHandler<()>,
}

#[component]
fn DiscoverCard(props: CardProps) -> Element {
    let card = &props.card;
    let policy = &card.policy_summary;

    rsx! {
        article {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 padding: {} {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 background: {};",
                spacing::SM,
                spacing::BASE,
                spacing::BASE,
                semantic::BORDER_DEFAULT,
                radius::LG,
                semantic::BG_SECONDARY,
            ),

            header {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: space-between; \
                     gap: {};",
                    spacing::SM,
                ),

                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {};",
                        spacing::XS,
                    ),
                    h2 {
                        style: format!(
                            "margin: 0; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_BASE,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY,
                        ),
                        "{card.name}"
                    }
                    span {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_XS,
                            semantic::TEXT_MUTED,
                        ),
                        "rev {card.revision} · {card.member_count} members"
                    }
                }

                div {
                    style: format!(
                        "display: flex; \
                         gap: {}; \
                         flex-wrap: wrap; \
                         justify-content: flex-end;",
                        spacing::XS,
                    ),
                    PolicyBadge {
                        label: discoverability_label(policy.discoverability),
                        tone: Tone::Neutral,
                    }
                    PolicyBadge {
                        label: admission_label(policy.admission),
                        tone: admission_tone(policy.admission),
                    }
                    PolicyBadge {
                        label: confidentiality_label(policy.confidentiality),
                        tone: confidentiality_tone(policy.confidentiality),
                    }
                    PolicyBadge {
                        label: read_label(policy.read_access),
                        tone: Tone::Neutral,
                    }
                    PolicyBadge {
                        label: write_label(policy.write_access),
                        tone: Tone::Neutral,
                    }
                }
            }

            if !card.description.is_empty() {
                p {
                    style: format!(
                        "margin: 0; \
                         font-size: {}; \
                         color: {}; \
                         line-height: {};",
                        typography::SIZE_SM,
                        semantic::TEXT_SECONDARY,
                        typography::LEADING_NORMAL,
                    ),
                    "{card.description}"
                }
            }

            if !card.tags.is_empty() {
                div {
                    style: format!(
                        "display: flex; \
                         flex-wrap: wrap; \
                         gap: {};",
                        spacing::XS,
                    ),
                    for tag in card.tags.iter() {
                        span {
                            key: "{tag}",
                            style: format!(
                                "padding: 2px {}; \
                                 border-radius: {}; \
                                 background: {}; \
                                 color: {}; \
                                 font-size: {};",
                                spacing::XS,
                                radius::SM,
                                semantic::BG_ELEVATED,
                                semantic::TEXT_SECONDARY,
                                typography::SIZE_XS,
                            ),
                            "#{tag}"
                        }
                    }
                }
            }

            footer {
                style: format!(
                    "display: flex; \
                     justify-content: flex-end; \
                     align-items: center; \
                     gap: {};",
                    spacing::SM,
                ),

                {
                    match &props.status {
                        RequestStatus::Failed(err) => {
                            let err_display = err.clone();
                            rsx! {
                                span {
                                    style: format!(
                                        "font-size: {}; color: {};",
                                        typography::SIZE_XS,
                                        semantic::ERROR,
                                    ),
                                    "Request failed: {err_display}"
                                }
                            }
                        }
                        RequestStatus::Submitted => rsx! {
                            span {
                                style: format!(
                                    "font-size: {}; color: {};",
                                    typography::SIZE_XS,
                                    semantic::SUCCESS,
                                ),
                                "Request submitted — awaiting admin review."
                            }
                        },
                        _ => rsx! {},
                    }
                }

                if props.can_request {
                    {
                        let disabled = matches!(
                            props.status,
                            RequestStatus::Pending | RequestStatus::Submitted
                        );
                        let label = match props.status {
                            RequestStatus::Pending => "Requesting…",
                            RequestStatus::Submitted => "Requested",
                            _ => "Request access",
                        };
                        rsx! {
                            button {
                                r#type: "button",
                                disabled,
                                style: format!(
                                    "border: none; \
                                     border-radius: {}; \
                                     background: {}; \
                                     color: {}; \
                                     padding: {} {}; \
                                     font-size: {}; \
                                     font-weight: {}; \
                                     cursor: {}; \
                                     opacity: {};",
                                    radius::LG,
                                    semantic::PRIMARY,
                                    semantic::TEXT_INVERSE,
                                    spacing::XS,
                                    spacing::BASE,
                                    typography::SIZE_SM,
                                    typography::WEIGHT_SEMIBOLD,
                                    if disabled { "not-allowed" } else { "pointer" },
                                    if disabled { "0.6" } else { "1" },
                                ),
                                onclick: move |_| props.on_request.call(()),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Policy-axis labels and tone helpers ──────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tone {
    Neutral,
    Info,
    Warning,
    Success,
}

fn tone_color(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => semantic::TEXT_MUTED,
        Tone::Info => semantic::INFO,
        Tone::Warning => semantic::WARNING,
        Tone::Success => semantic::SUCCESS,
    }
}

#[derive(Props, Clone, PartialEq)]
struct BadgeProps {
    label: &'static str,
    tone: Tone,
}

#[component]
fn PolicyBadge(props: BadgeProps) -> Element {
    rsx! {
        span {
            style: format!(
                "font-size: {}; \
                 padding: 2px {}; \
                 border-radius: {}; \
                 border: 1px solid {}; \
                 color: {}; \
                 white-space: nowrap;",
                typography::SIZE_XS,
                spacing::XS,
                radius::SM,
                semantic::BORDER_DEFAULT,
                tone_color(props.tone),
            ),
            "{props.label}"
        }
    }
}

fn discoverability_label(v: GroupDiscoverability) -> &'static str {
    match v {
        GroupDiscoverability::Hidden => "hidden",
        GroupDiscoverability::ListedToContacts => "contacts",
        GroupDiscoverability::PublicDirectory => "public",
    }
}

fn admission_label(v: GroupAdmission) -> &'static str {
    match v {
        GroupAdmission::InviteOnly => "invite only",
        GroupAdmission::RequestAccess => "request access",
        GroupAdmission::OpenJoin => "open join",
    }
}

fn admission_tone(v: GroupAdmission) -> Tone {
    match v {
        GroupAdmission::InviteOnly => Tone::Neutral,
        GroupAdmission::RequestAccess => Tone::Info,
        GroupAdmission::OpenJoin => Tone::Success,
    }
}

fn confidentiality_label(v: GroupConfidentiality) -> &'static str {
    match v {
        GroupConfidentiality::MlsEncrypted => "encrypted",
        GroupConfidentiality::SignedPublic => "signed public",
    }
}

fn confidentiality_tone(v: GroupConfidentiality) -> Tone {
    match v {
        GroupConfidentiality::MlsEncrypted => Tone::Success,
        GroupConfidentiality::SignedPublic => Tone::Warning,
    }
}

fn read_label(v: GroupReadAccess) -> &'static str {
    match v {
        GroupReadAccess::MembersOnly => "members read",
        GroupReadAccess::Public => "public read",
    }
}

fn write_label(v: GroupWriteAccess) -> &'static str {
    match v {
        GroupWriteAccess::MembersOnly => "members post",
        GroupWriteAccess::ModeratedPublic => "moderated post",
        GroupWriteAccess::AdminOnly => "admin post",
    }
}

// ── Low-volume helpers ────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct ErrorProps {
    message: String,
}

#[component]
fn ErrorBanner(props: ErrorProps) -> Element {
    rsx! {
        div {
            style: format!(
                "padding: {} {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 background: rgba(255, 68, 102, 0.08); \
                 color: {}; \
                 font-size: {};",
                spacing::SM,
                spacing::BASE,
                semantic::ERROR,
                radius::LG,
                semantic::ERROR,
                typography::SIZE_SM,
            ),
            role: "alert",
            "{props.message}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct EmptyProps {
    mode: DiscoverMode,
    query: String,
}

#[component]
fn EmptyState(props: EmptyProps) -> Element {
    let message = match (props.mode, props.query.trim().is_empty()) {
        (DiscoverMode::Nearby, _) => {
            "No PublicDirectory groups have arrived on the shard plane yet."
        }
        (DiscoverMode::All, true) => "No discoverable groups observed yet.",
        (DiscoverMode::All, false) => "No matches — try a different tag, name, or ID.",
    };
    rsx! {
        div {
            style: format!(
                "padding: {} {}; \
                 border: 1px dashed {}; \
                 border-radius: {}; \
                 color: {}; \
                 text-align: center; \
                 font-size: {};",
                spacing::XL,
                spacing::BASE,
                semantic::BORDER_DEFAULT,
                radius::LG,
                semantic::TEXT_MUTED,
                typography::SIZE_SM,
            ),
            "{message}"
        }
    }
}
