//! Context-sensitive detail panel (360px right column).
//!
//! Displays different content based on the current selection:
//! - Agent profile (when clicking a contact/agent)
//! - Space info (when clicking info on a space header)
//! - Delegates to ThreadPanel for thread views

use crate::design_tokens::{
    layout, motion, palette, radius, semantic, shadow, spacing, typography,
};
use communitas_x0x_client::{MachineRecord, TrustLevel, X0xClient};
use dioxus::prelude::*;
use tracing::{error, warn};

/// Copy text to clipboard via JS eval (works in Tauri WebView).
fn copy_text(value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("navigator.clipboard.writeText(\"{escaped}\").catch(()=>{{}});",);
    spawn(async move {
        let _ = dioxus::document::eval(&script);
    });
}

/// Width of the detail panel (matches thread panel for visual consistency).
const DETAIL_PANEL_WIDTH: &str = layout::THREAD_PANEL_WIDTH;

/// What the detail panel is currently showing.
#[derive(Clone, Debug, PartialEq)]
pub enum DetailContent {
    /// Panel is hidden.
    None,
    /// Show an agent/contact profile.
    AgentProfile {
        /// The agent ID to display.
        agent_id: String,
    },
    /// Show space information and invite controls.
    SpaceInfo {
        /// The group ID of the space.
        group_id: String,
    },
}

/// Context-sensitive detail panel.
#[component]
pub fn DetailPanel(
    /// Signal controlling what the panel shows; set to `DetailContent::None` to hide.
    content: Signal<DetailContent>,
) -> Element {
    let current = content();

    if current == DetailContent::None {
        return rsx! {};
    }

    rsx! {
        div {
            style: format!(
                "width: {DETAIL_PANEL_WIDTH}; \
                 min-width: {DETAIL_PANEL_WIDTH}; \
                 height: 100%; \
                 display: flex; \
                 flex-direction: column; \
                 background: {}; \
                 backdrop-filter: blur(12px); \
                 border-left: 1px solid {}; \
                 flex-shrink: 0; \
                 overflow: hidden;",
                semantic::GLASS_BG,
                semantic::BORDER_SUBTLE,
            ),
            role: "complementary",
            aria_label: "Detail panel",

            match current {
                DetailContent::AgentProfile { agent_id } => rsx! {
                    AgentProfileView {
                        agent_id: agent_id,
                        on_close: move |_| content.set(DetailContent::None),
                    }
                },
                DetailContent::SpaceInfo { group_id } => rsx! {
                    SpaceInfoView {
                        group_id: group_id,
                        on_close: move |_| content.set(DetailContent::None),
                    }
                },
                DetailContent::None => rsx! {},
            }
        }
    }
}

// ── Panel header (shared) ────────────────────────────────────────────────────

/// Shared panel header with title and close button.
#[component]
fn PanelHeader(title: String, on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 justify-content: space-between; \
                 padding: {} {}; \
                 border-bottom: 1px solid {}; \
                 flex-shrink: 0;",
                spacing::MD,
                spacing::BASE,
                semantic::BORDER_SUBTLE,
            ),

            span {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {};",
                    typography::SIZE_MD,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY,
                ),
                "{title}"
            }

            button {
                style: format!(
                    "width: 28px; \
                     height: 28px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     background: none; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     cursor: pointer; \
                     color: {}; \
                     font-size: {}; \
                     transition: {};",
                    semantic::BORDER_SUBTLE,
                    radius::MD,
                    semantic::TEXT_MUTED,
                    typography::SIZE_SM,
                    motion::transition("background, border-color"),
                ),
                aria_label: "Close panel",
                onclick: move |_| on_close.call(()),
                "\u{2715}"
            }
        }
    }
}

// ── Agent profile view ───────────────────────────────────────────────────────

/// Agent profile panel showing identity info, trust controls, and machine records.
#[component]
fn AgentProfileView(agent_id: String, on_close: EventHandler<()>) -> Element {
    let mut display_name = use_signal(|| None::<String>);
    let mut current_trust = use_signal(|| None::<TrustLevel>);
    let mut machines = use_signal(Vec::<MachineRecord>::new);
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| None::<String>);
    let mut removing = use_signal(|| false);

    let agent_id_fetch = agent_id.clone();
    use_future(move || {
        let aid = agent_id_fetch.clone();
        async move {
            let client = X0xClient::new();

            // Load contact info
            match client.list_contacts().await {
                Ok(contacts) => {
                    if let Some(contact) = contacts.iter().find(|c| c.agent_id == aid) {
                        current_trust.set(Some(contact.trust_level));
                        display_name.set(contact.label.clone());
                    }
                }
                Err(e) => warn!(target: "ui.detail_panel", "Failed to load contacts: {e}"),
            }

            // Load machine records
            match client.list_machines(&aid).await {
                Ok(m) => machines.set(m),
                Err(e) => {
                    warn!(target: "ui.detail_panel", "Failed to load machines for {aid}: {e}")
                }
            }

            loading.set(false);
        }
    });

    let initials = agent_id.chars().take(2).collect::<String>().to_uppercase();

    let short_id = if agent_id.len() > 16 {
        format!("{}...{}", &agent_id[..8], &agent_id[agent_id.len() - 6..])
    } else {
        agent_id.clone()
    };

    rsx! {
        PanelHeader { title: "Agent Profile", on_close: on_close }

        // Scrollable body
        div {
            style: format!(
                "flex: 1; \
                 overflow-y: auto; \
                 padding: {}; \
                 display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 scrollbar-width: thin; \
                 scrollbar-color: {} transparent;",
                spacing::BASE,
                spacing::BASE,
                semantic::BORDER_DEFAULT,
            ),

            // Avatar + name section
            div {
                style: format!(
                    "display: flex; \
                     flex-direction: column; \
                     align-items: center; \
                     gap: {}; \
                     padding: {} 0;",
                    spacing::SM,
                    spacing::SM,
                ),

                // Avatar circle
                div {
                    style: format!(
                        "width: 64px; \
                         height: 64px; \
                         border-radius: {}; \
                         background: {}; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         box-shadow: {};",
                        radius::FULL,
                        semantic::BG_ELEVATED,
                        typography::SIZE_XL,
                        typography::WEIGHT_BOLD,
                        semantic::TEXT_PRIMARY,
                        shadow::GLOW_SM,
                    ),
                    "{initials}"
                }

                // Display name
                if let Some(ref name) = display_name() {
                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_LG,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY,
                        ),
                        "{name}"
                    }
                }

                // Agent ID (copyable)
                button {
                    style: format!(
                        "font-family: {}; \
                         font-size: {}; \
                         color: {}; \
                         background: {}; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         padding: {} {}; \
                         cursor: pointer; \
                         word-break: break-all; \
                         text-align: center; \
                         transition: {};",
                        typography::FONT_MONO,
                        typography::SIZE_XS,
                        palette::JADE_400,
                        semantic::BG_TERTIARY,
                        semantic::BORDER_SUBTLE,
                        radius::MD,
                        spacing::XS,
                        spacing::SM,
                        motion::transition("border-color"),
                    ),
                    title: "Click to copy full Agent ID",
                    onclick: {
                        let full_id = agent_id.clone();
                        move |_| {
                            copy_text(&full_id);
                        }
                    },
                    "{short_id}"
                }
            }

            // Divider
            div {
                style: format!(
                    "height: 1px; background: {}; margin: {} 0;",
                    semantic::BORDER_SUBTLE,
                    spacing::XS,
                ),
            }

            // Trust level section
            div {
                style: format!(
                    "display: flex; \
                     flex-direction: column; \
                     gap: {};",
                    spacing::SM,
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         text-transform: uppercase; \
                         letter-spacing: {};",
                        typography::SIZE_XS,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_MUTED,
                        typography::TRACKING_WIDER,
                    ),
                    "Trust Level"
                }

                div {
                    style: format!(
                        "display: flex; \
                         gap: {};",
                        spacing::XS,
                    ),

                    for (level, label) in [
                        (TrustLevel::Blocked, "Blocked"),
                        (TrustLevel::Unknown, "Unknown"),
                        (TrustLevel::Known, "Known"),
                        (TrustLevel::Trusted, "Trusted"),
                    ] {
                        {
                            let is_active = current_trust() == Some(level);
                            let aid = agent_id.clone();
                            rsx! {
                                TrustButton {
                                    key: "{label}",
                                    label: label,
                                    active: is_active,
                                    level: level,
                                    on_click: move |new_level: TrustLevel| {
                                        let aid = aid.clone();
                                        current_trust.set(Some(new_level));
                                        spawn(async move {
                                            let client = X0xClient::new();
                                            if let Err(e) = client.set_trust(&aid, new_level).await {
                                                error!(target: "ui.detail_panel", "Failed to set trust: {e}");
                                            }
                                        });
                                    },
                                }
                            }
                        }
                    }
                }
            }

            // Divider
            div {
                style: format!(
                    "height: 1px; background: {}; margin: {} 0;",
                    semantic::BORDER_SUBTLE,
                    spacing::XS,
                ),
            }

            // Machine records section
            div {
                style: format!(
                    "display: flex; \
                     flex-direction: column; \
                     gap: {};",
                    spacing::SM,
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         text-transform: uppercase; \
                         letter-spacing: {};",
                        typography::SIZE_XS,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_MUTED,
                        typography::TRACKING_WIDER,
                    ),
                    "Machines"
                }

                if loading() {
                    div {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_SM,
                            semantic::TEXT_MUTED,
                        ),
                        "Loading..."
                    }
                } else if machines().is_empty() {
                    div {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_SM,
                            semantic::TEXT_MUTED,
                        ),
                        "No machine records."
                    }
                } else {
                    for machine in machines() {
                        {
                            let aid = agent_id.clone();
                            let mid = machine.machine_id.clone();
                            let m_label = machine.label.clone();
                            let m_first_seen = machine.first_seen;
                            let m_last_seen = machine.last_seen;
                            let m_pinned = machine.pinned;
                            rsx! {
                                MachineRow {
                                    key: "{mid}",
                                    machine_id: mid.clone(),
                                    label: m_label,
                                    first_seen: m_first_seen,
                                    last_seen: m_last_seen,
                                    pinned: m_pinned,
                                    on_toggle_pin: move |pinned: bool| {
                                        let aid = aid.clone();
                                        let mid = mid.clone();
                                        spawn(async move {
                                            let client = X0xClient::new();
                                            let result = if pinned {
                                                client.pin_machine(&aid, &mid).await
                                            } else {
                                                client.unpin_machine(&aid, &mid).await
                                            };
                                            if let Err(e) = result {
                                                error!(target: "ui.detail_panel", "Failed to toggle pin: {e}");
                                            }
                                        });
                                    },
                                }
                            }
                        }
                    }
                }
            }

            // Error display
            if let Some(ref err) = error_msg() {
                div {
                    style: format!(
                        "font-size: {}; color: {}; padding: {};",
                        typography::SIZE_SM,
                        semantic::ERROR,
                        spacing::SM,
                    ),
                    "{err}"
                }
            }

            // Spacer
            div { style: "flex: 1;" }

            // Remove Contact button
            button {
                style: format!(
                    "width: 100%; \
                     padding: {} {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     background: transparent; \
                     color: {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     cursor: {}; \
                     opacity: {}; \
                     transition: {};",
                    spacing::SM,
                    spacing::BASE,
                    palette::ROSE_500,
                    radius::LG,
                    palette::ROSE_400,
                    typography::SIZE_SM,
                    typography::WEIGHT_MEDIUM,
                    if removing() { "not-allowed" } else { "pointer" },
                    if removing() { "0.6" } else { "1" },
                    motion::transition("background, opacity"),
                ),
                disabled: removing(),
                onclick: {
                    let aid = agent_id.clone();
                    move |_| {
                        let aid = aid.clone();
                        removing.set(true);
                        spawn(async move {
                            let client = X0xClient::new();
                            match client.remove_contact(&aid).await {
                                Ok(()) => {
                                    // Panel will be closed by parent after refresh
                                }
                                Err(e) => {
                                    error_msg.set(Some(format!("Failed to remove contact: {e}")));
                                }
                            }
                            removing.set(false);
                        });
                    }
                },
                if removing() { "Removing..." } else { "Remove Contact" }
            }
        }
    }
}

/// A single trust level button.
#[component]
fn TrustButton(
    label: &'static str,
    active: bool,
    level: TrustLevel,
    on_click: EventHandler<TrustLevel>,
) -> Element {
    let (bg, border, fg) = if active {
        match level {
            TrustLevel::Blocked => (palette::ROSE_500, palette::ROSE_500, semantic::TEXT_INVERSE),
            TrustLevel::Unknown => (
                semantic::BG_ELEVATED,
                semantic::BORDER_STRONG,
                semantic::TEXT_PRIMARY,
            ),
            TrustLevel::Known => (palette::JADE_700, palette::JADE_600, semantic::TEXT_PRIMARY),
            TrustLevel::Trusted => (palette::JADE_500, palette::JADE_400, semantic::TEXT_INVERSE),
        }
    } else {
        (
            semantic::BG_TERTIARY,
            semantic::BORDER_SUBTLE,
            semantic::TEXT_MUTED,
        )
    };

    rsx! {
        button {
            style: format!(
                "flex: 1; \
                 padding: {} {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 background: {}; \
                 color: {}; \
                 font-size: {}; \
                 font-weight: {}; \
                 cursor: pointer; \
                 white-space: nowrap; \
                 transition: {};",
                spacing::XS,
                spacing::XS,
                border,
                radius::MD,
                bg,
                fg,
                typography::SIZE_XXS,
                if active { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_NORMAL },
                motion::transition("background, border-color, color"),
            ),
            onclick: move |_| on_click.call(level),
            "{label}"
        }
    }
}

/// A single machine record row with pin toggle.
#[component]
fn MachineRow(
    machine_id: String,
    label: Option<String>,
    first_seen: Option<u64>,
    last_seen: Option<u64>,
    pinned: bool,
    on_toggle_pin: EventHandler<bool>,
) -> Element {
    let short_mid = if machine_id.len() > 12 {
        format!(
            "{}..{}",
            &machine_id[..6],
            &machine_id[machine_id.len() - 4..]
        )
    } else {
        machine_id.clone()
    };

    let label_text = label
        .clone()
        .unwrap_or_else(|| "Unnamed machine".to_string());

    let first_seen_text = first_seen.map(format_timestamp).unwrap_or_default();
    let last_seen_text = last_seen.map(format_timestamp).unwrap_or_default();

    let is_pinned = pinned;

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border-radius: {}; \
                 border: 1px solid {};",
                spacing::SM,
                spacing::SM,
                semantic::BG_SECONDARY,
                radius::MD,
                semantic::BORDER_SUBTLE,
            ),

            // Machine info
            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY,
                    ),
                    "{label_text}"
                }

                div {
                    style: format!(
                        "font-family: {}; \
                         font-size: {}; \
                         color: {};",
                        typography::FONT_MONO,
                        typography::SIZE_XXS,
                        semantic::TEXT_MUTED,
                    ),
                    "{short_mid}"
                }

                if !first_seen_text.is_empty() || !last_seen_text.is_empty() {
                    div {
                        style: format!(
                            "font-size: {}; \
                             color: {}; \
                             margin-top: {};",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED,
                            spacing::XXS,
                        ),
                        if !first_seen_text.is_empty() {
                            span { "First: {first_seen_text}" }
                        }
                        if !first_seen_text.is_empty() && !last_seen_text.is_empty() {
                            span { " \u{2022} " }
                        }
                        if !last_seen_text.is_empty() {
                            span { "Last: {last_seen_text}" }
                        }
                    }
                }
            }

            // Pin toggle
            button {
                style: format!(
                    "width: 28px; \
                     height: 28px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     cursor: pointer; \
                     color: {}; \
                     font-size: {}; \
                     transition: {};",
                    if is_pinned { semantic::PRIMARY } else { "transparent" },
                    if is_pinned { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                    radius::MD,
                    if is_pinned { semantic::TEXT_INVERSE } else { semantic::TEXT_MUTED },
                    typography::SIZE_XS,
                    motion::transition("background, border-color, color"),
                ),
                title: if is_pinned { "Unpin machine" } else { "Pin machine" },
                onclick: move |_| on_toggle_pin.call(!is_pinned),
                "\u{1F4CC}" // pin emoji
            }
        }
    }
}

// ── Space info view ──────────────────────────────────────────────────────────

/// Space info panel showing group details, invite generation, and leave controls.
#[component]
fn SpaceInfoView(group_id: String, on_close: EventHandler<()>) -> Element {
    let mut space_name = use_signal(String::new);
    let mut space_description = use_signal(|| None::<String>);
    let mut space_creator = use_signal(|| None::<String>);
    let mut member_count = use_signal(|| 0u32);
    let mut loading = use_signal(|| true);

    let mut invite_link = use_signal(|| None::<String>);
    let mut invite_loading = use_signal(|| false);
    let mut invite_error = use_signal(|| None::<String>);

    let mut display_name_input = use_signal(String::new);
    let mut display_name_saving = use_signal(|| false);
    let mut display_name_saved = use_signal(|| false);

    let mut leaving = use_signal(|| false);
    let mut leave_error = use_signal(|| None::<String>);

    let group_id_fetch = group_id.clone();
    use_future(move || {
        let gid = group_id_fetch.clone();
        async move {
            let client = X0xClient::new();
            match client.get_group(&gid).await {
                Ok(group) => {
                    space_name.set(group.name);
                    space_description.set(group.description);
                    space_creator.set(group.creator);
                    member_count.set(group.member_count.unwrap_or(0));
                }
                Err(e) => {
                    warn!(target: "ui.detail_panel", "Failed to load group {gid}: {e}");
                }
            }
            loading.set(false);
        }
    });

    rsx! {
        PanelHeader { title: "Space Info", on_close: on_close }

        div {
            style: format!(
                "flex: 1; \
                 overflow-y: auto; \
                 padding: {}; \
                 display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 scrollbar-width: thin; \
                 scrollbar-color: {} transparent;",
                spacing::BASE,
                spacing::BASE,
                semantic::BORDER_DEFAULT,
            ),

            if loading() {
                div {
                    style: format!(
                        "font-size: {}; color: {};",
                        typography::SIZE_SM,
                        semantic::TEXT_MUTED,
                    ),
                    "Loading space info..."
                }
            } else {
                // Space name
                div {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_LG,
                        typography::WEIGHT_BOLD,
                        semantic::TEXT_PRIMARY,
                    ),
                    "{space_name}"
                }

                // Description
                if let Some(ref desc) = space_description() {
                    if !desc.is_empty() {
                        div {
                            style: format!(
                                "font-size: {}; \
                                 color: {}; \
                                 line-height: {};",
                                typography::SIZE_SM,
                                semantic::TEXT_SECONDARY,
                                typography::LEADING_NORMAL,
                            ),
                            "{desc}"
                        }
                    }
                }

                // Stats row
                div {
                    style: format!(
                        "display: flex; \
                         gap: {}; \
                         font-size: {}; \
                         color: {};",
                        spacing::BASE,
                        typography::SIZE_SM,
                        semantic::TEXT_MUTED,
                    ),

                    span { "{member_count} members" }

                    if let Some(ref creator) = space_creator() {
                        {
                            let short = if creator.len() > 12 {
                                format!("{}..{}", &creator[..6], &creator[creator.len() - 4..])
                            } else {
                                creator.clone()
                            };
                            rsx! { span { "Creator: {short}" } }
                        }
                    }
                }

                // Divider
                div {
                    style: format!(
                        "height: 1px; background: {}; margin: {} 0;",
                        semantic::BORDER_SUBTLE,
                        spacing::XS,
                    ),
                }

                // ── Invite section ───────────────────────────────────────────
                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {};",
                        spacing::SM,
                    ),

                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {}; \
                             text-transform: uppercase; \
                             letter-spacing: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_MUTED,
                            typography::TRACKING_WIDER,
                        ),
                        "Invite"
                    }

                    button {
                        style: format!(
                            "width: 100%; \
                             padding: {} {}; \
                             border: none; \
                             border-radius: {}; \
                             background: {}; \
                             color: {}; \
                             font-size: {}; \
                             font-weight: {}; \
                             cursor: {}; \
                             opacity: {}; \
                             transition: {};",
                            spacing::SM,
                            spacing::BASE,
                            radius::LG,
                            semantic::PRIMARY,
                            semantic::TEXT_INVERSE,
                            typography::SIZE_SM,
                            typography::WEIGHT_SEMIBOLD,
                            if invite_loading() { "not-allowed" } else { "pointer" },
                            if invite_loading() { "0.6" } else { "1" },
                            motion::transition("opacity"),
                        ),
                        disabled: invite_loading(),
                        onclick: {
                            let gid = group_id.clone();
                            move |_| {
                                let gid = gid.clone();
                                invite_loading.set(true);
                                invite_error.set(None);
                                spawn(async move {
                                    let client = X0xClient::new();
                                    // 7 day expiry
                                    match client.invite(&gid, Some(604_800)).await {
                                        Ok(resp) => invite_link.set(Some(resp.invite_link)),
                                        Err(e) => {
                                            invite_error.set(Some(format!("Failed to generate invite: {e}")));
                                        }
                                    }
                                    invite_loading.set(false);
                                });
                            }
                        },
                        if invite_loading() { "Generating..." } else { "Generate Invite Link" }
                    }

                    if let Some(ref link) = invite_link() {
                        div {
                            style: format!(
                                "display: flex; \
                                 flex-direction: column; \
                                 gap: {};",
                                spacing::XS,
                            ),

                            div {
                                style: format!(
                                    "font-family: {}; \
                                     font-size: {}; \
                                     color: {}; \
                                     background: {}; \
                                     border-radius: {}; \
                                     padding: {}; \
                                     word-break: break-all; \
                                     border: 1px solid {};",
                                    typography::FONT_MONO,
                                    typography::SIZE_XS,
                                    palette::JADE_400,
                                    semantic::BG_TERTIARY,
                                    radius::MD,
                                    spacing::SM,
                                    semantic::BORDER_SUBTLE,
                                ),
                                "{link}"
                            }

                            button {
                                style: format!(
                                    "align-self: flex-start; \
                                     padding: {} {}; \
                                     border: 1px solid {}; \
                                     border-radius: {}; \
                                     background: transparent; \
                                     color: {}; \
                                     font-size: {}; \
                                     cursor: pointer; \
                                     transition: {};",
                                    spacing::XS,
                                    spacing::SM,
                                    semantic::BORDER_DEFAULT,
                                    radius::MD,
                                    semantic::TEXT_SECONDARY,
                                    typography::SIZE_XS,
                                    motion::transition("border-color"),
                                ),
                                onclick: {
                                    let link_val = link.clone();
                                    move |_| {
                                        copy_text(&link_val);
                                    }
                                },
                                "Copy"
                            }
                        }
                    }

                    if let Some(ref err) = invite_error() {
                        div {
                            style: format!(
                                "font-size: {}; color: {};",
                                typography::SIZE_SM,
                                semantic::ERROR,
                            ),
                            "{err}"
                        }
                    }
                }

                // Divider
                div {
                    style: format!(
                        "height: 1px; background: {}; margin: {} 0;",
                        semantic::BORDER_SUBTLE,
                        spacing::XS,
                    ),
                }

                // ── Display name in space ────────────────────────────────────
                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {};",
                        spacing::SM,
                    ),

                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {}; \
                             text-transform: uppercase; \
                             letter-spacing: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_MUTED,
                            typography::TRACKING_WIDER,
                        ),
                        "Display Name in Space"
                    }

                    div {
                        style: format!(
                            "display: flex; \
                             gap: {};",
                            spacing::XS,
                        ),

                        input {
                            r#type: "text",
                            placeholder: "Your name in this space",
                            value: "{display_name_input}",
                            disabled: display_name_saving(),
                            style: format!(
                                "flex: 1; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 background: {}; \
                                 color: {}; \
                                 padding: {} {}; \
                                 font-size: {}; \
                                 outline: none;",
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::BG_TERTIARY,
                                semantic::TEXT_PRIMARY,
                                spacing::SM,
                                spacing::SM,
                                typography::SIZE_SM,
                            ),
                            oninput: move |evt: Event<FormData>| {
                                display_name_input.set(evt.value().to_string());
                                display_name_saved.set(false);
                            },
                        }

                        button {
                            style: format!(
                                "padding: {} {}; \
                                 border: none; \
                                 border-radius: {}; \
                                 background: {}; \
                                 color: {}; \
                                 font-size: {}; \
                                 font-weight: {}; \
                                 cursor: {}; \
                                 opacity: {};",
                                spacing::SM,
                                spacing::SM,
                                radius::LG,
                                if display_name_saved() { semantic::SUCCESS } else { semantic::PRIMARY },
                                semantic::TEXT_INVERSE,
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                if display_name_saving() || display_name_input().trim().is_empty() { "not-allowed" } else { "pointer" },
                                if display_name_saving() || display_name_input().trim().is_empty() { "0.6" } else { "1" },
                            ),
                            disabled: display_name_saving() || display_name_input().trim().is_empty(),
                            onclick: {
                                let gid = group_id.clone();
                                move |_| {
                                    let gid = gid.clone();
                                    let name = display_name_input().trim().to_string();
                                    if name.is_empty() {
                                        return;
                                    }
                                    display_name_saving.set(true);
                                    spawn(async move {
                                        let client = X0xClient::new();
                                        match client.set_group_display_name(&gid, &name).await {
                                            Ok(()) => display_name_saved.set(true),
                                            Err(e) => error!(target: "ui.detail_panel", "Failed to set display name: {e}"),
                                        }
                                        display_name_saving.set(false);
                                    });
                                }
                            },
                            if display_name_saving() {
                                "..."
                            } else if display_name_saved() {
                                "Saved"
                            } else {
                                "Set"
                            }
                        }
                    }
                }

                // Spacer
                div { style: "flex: 1;" }

                // Leave Space button
                if let Some(ref err) = leave_error() {
                    div {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_SM,
                            semantic::ERROR,
                        ),
                        "{err}"
                    }
                }

                button {
                    style: format!(
                        "width: 100%; \
                         padding: {} {}; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         background: transparent; \
                         color: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         cursor: {}; \
                         opacity: {}; \
                         transition: {};",
                        spacing::SM,
                        spacing::BASE,
                        palette::ROSE_500,
                        radius::LG,
                        palette::ROSE_400,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        if leaving() { "not-allowed" } else { "pointer" },
                        if leaving() { "0.6" } else { "1" },
                        motion::transition("background, opacity"),
                    ),
                    disabled: leaving(),
                    onclick: {
                        let gid = group_id.clone();
                        move |_| {
                            let gid = gid.clone();
                            leaving.set(true);
                            leave_error.set(None);
                            spawn(async move {
                                let client = X0xClient::new();
                                match client.leave_group(&gid).await {
                                    Ok(()) => {
                                        // Parent should handle navigation after leave
                                    }
                                    Err(e) => {
                                        leave_error.set(Some(format!("Failed to leave: {e}")));
                                    }
                                }
                                leaving.set(false);
                            });
                        }
                    },
                    if leaving() { "Leaving..." } else { "Leave Space" }
                }
            }
        }
    }
}

/// Format a millisecond timestamp to a short date string.
fn format_timestamp(ms: u64) -> String {
    let secs = ms / 1000;
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 365 {
        format!("{}y ago", days / 365)
    } else if days > 30 {
        format!("{}mo ago", days / 30)
    } else if days > 0 {
        format!("{days}d ago")
    } else if hours > 0 {
        format!("{hours}h ago")
    } else {
        format!("{mins}m ago")
    }
}
