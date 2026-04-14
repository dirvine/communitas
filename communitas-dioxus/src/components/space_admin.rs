// SPDX-License-Identifier: MIT OR Apache-2.0

//! Space admin panel — policy, roster, role/ban controls, and join requests.
//!
//! Surfaces the owner/admin-only operations from the named-groups REST
//! API inside the space's tab bar. Mirrors the CLI commands
//! `x0x group policy`, `x0x group set-role`, `x0x group ban/unban`,
//! `x0x group requests`, `x0x group approve-request`,
//! `x0x group reject-request`, `x0x group state`,
//! `x0x group state-seal`, and `x0x group state-withdraw`.
//!
//! All controls degrade gracefully: a non-admin viewing the panel sees
//! the policy + roster read-only and gets a clear message when the
//! daemon returns 403 on admin actions.

use communitas_x0x_client::{
    GroupAdmission, GroupConfidentiality, GroupDiscoverability, GroupInfo, GroupPolicyPreset,
    GroupReadAccess, GroupRole, GroupStateResponse, GroupWriteAccess, JoinRequest,
    JoinRequestStatus, NamedGroupMember, UpdateGroupPolicyRequest, X0xClient,
};
use dioxus::prelude::*;
use tracing::{info, warn};

use crate::design_tokens::{radius, semantic, spacing, typography};

/// Polling cadence for roster + requests.
const REFRESH_INTERVAL_SECS: u64 = 10;

/// Key-value action feedback used by each sub-panel.
#[derive(Clone, PartialEq, Eq)]
enum ActionStatus {
    Idle,
    Pending,
    Ok(String),
    Err(String),
}

#[derive(Props, Clone, PartialEq)]
pub struct SpaceAdminProps {
    /// Stable group_id of the space.
    pub space_id: String,
}

/// Top-level admin panel rendered under the space's Manage tab.
#[component]
pub fn SpaceAdminPanel(props: SpaceAdminProps) -> Element {
    let space_id = props.space_id.clone();
    let space_id_for_info = space_id.clone();
    let space_id_for_members = space_id.clone();
    let space_id_for_requests = space_id.clone();

    let mut group_info = use_signal(|| None::<GroupInfo>);
    let mut members = use_signal(Vec::<NamedGroupMember>::new);
    let mut requests = use_signal(Vec::<JoinRequest>::new);
    let mut caller_agent_id = use_signal(|| None::<String>);
    let mut last_error = use_signal(|| None::<String>);

    // Poll daemon so the panel stays fresh. Separate from the main
    // group list poll to keep the refresh cadence tight while the admin
    // is working.
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let space_id = space_id_for_info.clone();
        async move {
            let client = X0xClient::new();
            loop {
                match client.agent().await {
                    Ok(a) => caller_agent_id.set(Some(a.agent_id)),
                    Err(e) => warn!(target: "ui.space_admin", "agent fetch failed: {e}"),
                }
                match client.get_group(&space_id).await {
                    Ok(info) => {
                        group_info.set(Some(info));
                        last_error.set(None);
                    }
                    Err(e) => {
                        warn!(target: "ui.space_admin", "get_group failed: {e}");
                        last_error.set(Some(format!("Failed to load group: {e}")));
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
            }
        }
    });

    use_coroutine(move |_: UnboundedReceiver<()>| {
        let space_id = space_id_for_members.clone();
        async move {
            let client = X0xClient::new();
            loop {
                if let Ok(list) = client.list_named_group_members(&space_id).await {
                    members.set(list);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
            }
        }
    });

    use_coroutine(move |_: UnboundedReceiver<()>| {
        let space_id = space_id_for_requests.clone();
        async move {
            let client = X0xClient::new();
            loop {
                // 403 on non-admin is expected; swallow it.
                match client.list_join_requests(&space_id).await {
                    Ok(list) => requests.set(list),
                    Err(_) => requests.set(Vec::new()),
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
            }
        }
    });

    // Derive caller role (if any) from roster snapshot.
    let caller_role = {
        let members_view = members.read();
        let caller = caller_agent_id.read();
        caller
            .as_ref()
            .and_then(|aid| members_view.iter().find(|m| m.agent_id == *aid))
            .map(|m| m.role)
    };
    let is_owner = caller_role == Some(GroupRole::Owner);
    let is_admin_or_above = matches!(caller_role, Some(r) if role_at_least(r, GroupRole::Admin));

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
            aria_label: "Space admin",

            if let Some(err) = last_error() {
                ErrorBanner { message: err }
            }

            if group_info().is_some() {
                PolicyPanel {
                    space_id: space_id.clone(),
                    caller_is_owner: is_owner,
                }

                StatePanel {
                    space_id: space_id.clone(),
                    caller_is_owner: is_owner,
                }
            }

            RosterPanel {
                space_id: space_id.clone(),
                members: members.read().clone(),
                caller_role,
            }

            if is_admin_or_above {
                RequestsPanel {
                    space_id: space_id.clone(),
                    requests: requests.read().clone(),
                }
            }
        }
    }
}

// ── Policy panel ─────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct PolicyPanelProps {
    space_id: String,
    caller_is_owner: bool,
}

#[component]
fn PolicyPanel(props: PolicyPanelProps) -> Element {
    // NOTE: GroupInfo from the minimal API shape may not include full
    // policy axes. We render what we can derive and expose preset +
    // axis controls for owner-only edits via `PATCH /groups/:id/policy`.
    let mut status = use_signal(|| ActionStatus::Idle);
    let mut preset_choice = use_signal(|| None::<GroupPolicyPreset>);
    let mut disc_choice = use_signal(|| None::<GroupDiscoverability>);
    let mut adm_choice = use_signal(|| None::<GroupAdmission>);
    let mut conf_choice = use_signal(|| None::<GroupConfidentiality>);
    let mut read_choice = use_signal(|| None::<GroupReadAccess>);
    let mut write_choice = use_signal(|| None::<GroupWriteAccess>);

    let space_id = props.space_id.clone();

    rsx! {
        PanelShell {
            title: "Policy",
            subtitle: format!("Group ID {}", shorten(&props.space_id)),
            children: rsx! {
                if !props.caller_is_owner {
                    HintLine {
                        message: "Only the owner can change policy. Contact an admin to request changes.",
                    }
                }

                div {
                    style: format!(
                        "display: grid; \
                         grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); \
                         gap: {};",
                        spacing::SM,
                    ),

                    PresetDropdown {
                        disabled: !props.caller_is_owner,
                        value: preset_choice(),
                        on_change: move |v: Option<GroupPolicyPreset>| preset_choice.set(v),
                    }

                    DiscoverabilityDropdown {
                        disabled: !props.caller_is_owner,
                        value: disc_choice(),
                        on_change: move |v: Option<GroupDiscoverability>| disc_choice.set(v),
                    }

                    AdmissionDropdown {
                        disabled: !props.caller_is_owner,
                        value: adm_choice(),
                        on_change: move |v: Option<GroupAdmission>| adm_choice.set(v),
                    }

                    ConfidentialityDropdown {
                        disabled: !props.caller_is_owner,
                        value: conf_choice(),
                        on_change: move |v: Option<GroupConfidentiality>| conf_choice.set(v),
                    }

                    ReadAccessDropdown {
                        disabled: !props.caller_is_owner,
                        value: read_choice(),
                        on_change: move |v: Option<GroupReadAccess>| read_choice.set(v),
                    }

                    WriteAccessDropdown {
                        disabled: !props.caller_is_owner,
                        value: write_choice(),
                        on_change: move |v: Option<GroupWriteAccess>| write_choice.set(v),
                    }
                }

                StatusLine { status: status() }

                if props.caller_is_owner {
                    div {
                        style: format!(
                            "display: flex; justify-content: flex-end; gap: {};",
                            spacing::SM,
                        ),
                        button {
                            style: primary_button_style(matches!(status(), ActionStatus::Pending)),
                            disabled: matches!(status(), ActionStatus::Pending),
                            onclick: move |_| {
                                let sid = space_id.clone();
                                let patch = UpdateGroupPolicyRequest {
                                    preset: preset_choice().map(|p| preset_to_wire(p).to_owned()),
                                    discoverability: disc_choice(),
                                    admission: adm_choice(),
                                    confidentiality: conf_choice(),
                                    read_access: read_choice(),
                                    write_access: write_choice(),
                                };
                                if patch.preset.is_none()
                                    && patch.discoverability.is_none()
                                    && patch.admission.is_none()
                                    && patch.confidentiality.is_none()
                                    && patch.read_access.is_none()
                                    && patch.write_access.is_none()
                                {
                                    status.set(ActionStatus::Err(
                                        "Pick at least one field to change.".into(),
                                    ));
                                    return;
                                }
                                status.set(ActionStatus::Pending);
                                spawn(async move {
                                    let client = X0xClient::new();
                                    match client.update_group_policy(&sid, &patch).await {
                                        Ok(()) => {
                                            info!(target: "ui.space_admin", "policy updated for {sid}");
                                            status.set(ActionStatus::Ok("Policy updated.".into()));
                                        }
                                        Err(e) => {
                                            warn!(target: "ui.space_admin", "policy update failed: {e}");
                                            status.set(ActionStatus::Err(format!("{e}")));
                                        }
                                    }
                                });
                            },
                            "Apply policy change"
                        }
                    }
                }
            }
        }
    }
}

// ── State-commit panel ───────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct StatePanelProps {
    space_id: String,
    caller_is_owner: bool,
}

#[component]
fn StatePanel(props: StatePanelProps) -> Element {
    let mut status = use_signal(|| ActionStatus::Idle);
    let mut chain = use_signal(|| None::<GroupStateResponse>);
    let mut chain_error = use_signal(|| None::<String>);
    // Revision bumps after seal / withdraw so the reader re-fetches
    // immediately rather than waiting for the next poll.
    let mut chain_rev = use_signal(|| 0u64);
    let space_id = props.space_id.clone();
    let space_id_seal = space_id.clone();
    let space_id_withdraw = space_id.clone();

    // Inspection loop — polls GET /groups/:id/state so the reader stays
    // in sync with the daemon. Also refires whenever a local action
    // bumps `chain_rev`.
    use_coroutine({
        let space_id = space_id.clone();
        move |_: UnboundedReceiver<()>| {
            let space_id = space_id.clone();
            async move {
                let client = X0xClient::new();
                loop {
                    // chain_rev() is read to participate in reactivity;
                    // a bump from seal/withdraw triggers the next poll
                    // sooner via the inline re-fetch in those handlers.
                    let _ = chain_rev();
                    match client.get_group_state(&space_id).await {
                        Ok(state) => {
                            chain.set(Some(state));
                            chain_error.set(None);
                        }
                        Err(e) => {
                            let msg = format!("{e}");
                            // 403 is normal for non-members; silence it.
                            if !msg.contains("not a member") {
                                warn!(target: "ui.space_admin", "state fetch failed: {msg}");
                                chain_error.set(Some(msg));
                            }
                            chain.set(None);
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(REFRESH_INTERVAL_SECS))
                        .await;
                }
            }
        }
    });

    rsx! {
        PanelShell {
            title: "State chain (Phase D.3)",
            subtitle: "Signed state-commit chain binds roster, policy, public metadata, and MLS epoch.".to_owned(),
            children: rsx! {
                if let Some(err) = chain_error() {
                    ErrorBanner { message: format!("Could not read state: {err}") }
                }

                match chain() {
                    Some(state) => rsx! {
                        StateReadout { state: state.clone() }
                    },
                    None => rsx! {
                        HintLine { message: "Loading state chain…" }
                    },
                }

                StatusLine { status: status() }

                if props.caller_is_owner {
                    div {
                        style: format!(
                            "display: flex; gap: {}; flex-wrap: wrap;",
                            spacing::SM,
                        ),

                        button {
                            style: primary_button_style(matches!(status(), ActionStatus::Pending)),
                            disabled: matches!(status(), ActionStatus::Pending),
                            onclick: move |_| {
                                let sid = space_id_seal.clone();
                                status.set(ActionStatus::Pending);
                                spawn(async move {
                                    let client = X0xClient::new();
                                    match client.seal_group_state(&sid).await {
                                        Ok(()) => {
                                            status.set(ActionStatus::Ok(
                                                "State sealed — new revision published.".into(),
                                            ));
                                            // Re-read the chain immediately — the
                                            // poll loop would otherwise take up to
                                            // REFRESH_INTERVAL_SECS to show it.
                                            if let Ok(state) = client.get_group_state(&sid).await {
                                                chain.set(Some(state));
                                            }
                                            chain_rev.with_mut(|r| *r += 1);
                                        }
                                        Err(e) => status.set(ActionStatus::Err(format!("{e}"))),
                                    }
                                });
                            },
                            "Seal state"
                        }

                        button {
                            style: danger_button_style(matches!(status(), ActionStatus::Pending)),
                            disabled: matches!(status(), ActionStatus::Pending),
                            onclick: move |_| {
                                let sid = space_id_withdraw.clone();
                                status.set(ActionStatus::Pending);
                                spawn(async move {
                                    let client = X0xClient::new();
                                    match client.withdraw_group_state(&sid).await {
                                        Ok(()) => {
                                            status.set(ActionStatus::Ok(
                                                "Withdrawal sealed — public card superseded.".into(),
                                            ));
                                            if let Ok(state) = client.get_group_state(&sid).await {
                                                chain.set(Some(state));
                                            }
                                            chain_rev.with_mut(|r| *r += 1);
                                        }
                                        Err(e) => status.set(ActionStatus::Err(format!("{e}"))),
                                    }
                                });
                            },
                            "Withdraw (hide publicly)"
                        }
                    }
                } else {
                    HintLine { message: "Only the owner can seal or withdraw the state chain." }
                }
            }
        }
    }
}

/// Read-only rendering of the current `GroupStateResponse`.
#[derive(Props, Clone, PartialEq)]
struct StateReadoutProps {
    state: GroupStateResponse,
}

#[component]
fn StateReadout(props: StateReadoutProps) -> Element {
    let s = &props.state;
    let prev = s
        .prev_state_hash
        .clone()
        .unwrap_or_else(|| "(genesis — no parent)".to_owned());
    let binding = s
        .security_binding
        .clone()
        .unwrap_or_else(|| "(none)".to_owned());
    let genesis_display = match &s.genesis {
        Some(g) => format!(
            "{} · nonce {} · created {}",
            shorten(&g.creator_agent_id),
            shorten(&g.creation_nonce),
            g.created_at
        ),
        None => "(legacy — no genesis record)".to_owned(),
    };
    let withdrawn_display = if s.withdrawn {
        "withdrawn (public card superseded)"
    } else {
        "active"
    };
    let withdrawn_tone = if s.withdrawn {
        semantic::ERROR
    } else {
        semantic::SUCCESS
    };

    rsx! {
        div {
            style: format!(
                "display: grid; \
                 grid-template-columns: max-content 1fr; \
                 row-gap: {}; \
                 column-gap: {}; \
                 font-family: {}; \
                 font-size: {}; \
                 color: {};",
                spacing::XXS,
                spacing::SM,
                typography::FONT_MONO,
                typography::SIZE_XS,
                semantic::TEXT_SECONDARY,
            ),

            StateFieldLabel { label: "revision" }
            span { "{s.state_revision}" }

            StateFieldLabel { label: "status" }
            span {
                style: format!("color: {};", withdrawn_tone),
                "{withdrawn_display}"
            }

            StateFieldLabel { label: "state_hash" }
            HashCell { value: s.state_hash.clone() }

            StateFieldLabel { label: "prev_hash" }
            HashCell { value: prev }

            StateFieldLabel { label: "roster_root" }
            HashCell { value: s.roster_root.clone() }

            StateFieldLabel { label: "policy_hash" }
            HashCell { value: s.policy_hash.clone() }

            StateFieldLabel { label: "public_meta" }
            HashCell { value: s.public_meta_hash.clone() }

            StateFieldLabel { label: "security_binding" }
            span { "{binding}" }

            StateFieldLabel { label: "genesis" }
            span { "{genesis_display}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StateFieldLabelProps {
    label: &'static str,
}

#[component]
fn StateFieldLabel(props: StateFieldLabelProps) -> Element {
    rsx! {
        span {
            style: format!(
                "color: {}; font-weight: {};",
                semantic::TEXT_MUTED,
                typography::WEIGHT_MEDIUM,
            ),
            "{props.label}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct HashCellProps {
    value: String,
}

#[component]
fn HashCell(props: HashCellProps) -> Element {
    // Hashes are long — render them with `word-break: break-all` so
    // the grid stays readable on narrow viewports.
    rsx! {
        span {
            style: "word-break: break-all; overflow-wrap: anywhere;",
            "{props.value}"
        }
    }
}

// ── Roster panel ─────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct RosterPanelProps {
    space_id: String,
    members: Vec<NamedGroupMember>,
    caller_role: Option<GroupRole>,
}

#[component]
fn RosterPanel(props: RosterPanelProps) -> Element {
    let is_admin_or_above =
        matches!(props.caller_role, Some(r) if role_at_least(r, GroupRole::Admin));
    let is_owner = props.caller_role == Some(GroupRole::Owner);
    let mut row_status = use_signal(std::collections::HashMap::<String, ActionStatus>::new);

    rsx! {
        PanelShell {
            title: format!("Members ({})", props.members.len()),
            subtitle: "Add, remove, ban, or change member roles.".to_owned(),
            children: rsx! {
                if props.members.is_empty() {
                    HintLine { message: "No members yet." }
                } else {
                    ul {
                        style: "list-style: none; margin: 0; padding: 0;",

                        for member in props.members.iter().cloned() {
                            {
                                let agent_id = member.agent_id.clone();
                                let space_id = props.space_id.clone();
                                let current_status = row_status
                                    .read()
                                    .get(&agent_id)
                                    .cloned()
                                    .unwrap_or(ActionStatus::Idle);
                                rsx! {
                                    li {
                                        key: "{member.agent_id}",
                                        MemberRow {
                                            member: member.clone(),
                                            space_id: space_id.clone(),
                                            status: current_status,
                                            caller_is_admin: is_admin_or_above,
                                            caller_is_owner: is_owner,
                                            on_status_change: move |(aid, new_status): (String, ActionStatus)| {
                                                row_status.write().insert(aid, new_status);
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
}

#[derive(Props, Clone, PartialEq)]
struct MemberRowProps {
    member: NamedGroupMember,
    space_id: String,
    status: ActionStatus,
    caller_is_admin: bool,
    caller_is_owner: bool,
    on_status_change: EventHandler<(String, ActionStatus)>,
}

#[component]
fn MemberRow(props: MemberRowProps) -> Element {
    let display = props
        .member
        .display_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| shorten(&props.member.agent_id));
    let is_target_owner = props.member.role == GroupRole::Owner;
    let can_manage = props.caller_is_admin && !is_target_owner;
    let state_tone = member_state_tone(props.member.state);
    let busy = matches!(props.status, ActionStatus::Pending);

    let m_for_promote = props.member.clone();
    let m_for_ban = props.member.clone();

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 background: {}; \
                 margin-bottom: {};",
                spacing::SM,
                spacing::SM,
                spacing::BASE,
                semantic::BORDER_DEFAULT,
                radius::LG,
                semantic::BG_SECONDARY,
                spacing::XS,
            ),

            div {
                style: format!(
                    "flex: 1; \
                     display: flex; \
                     flex-direction: column; \
                     gap: {};",
                    spacing::XXS,
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY,
                    ),
                    "{display}"
                }

                span {
                    style: format!(
                        "font-size: {}; color: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED,
                    ),
                    "{shorten(&props.member.agent_id)} · role {role_label(props.member.role)} · state "
                    span {
                        style: format!("color: {};", state_tone),
                        "{member_state_label(props.member.state)}"
                    }
                }
            }

            if can_manage {
                div {
                    style: format!(
                        "display: flex; gap: {};",
                        spacing::XS,
                    ),

                    if props.caller_is_owner && props.member.role != GroupRole::Admin {
                        button {
                            style: small_button_style(busy),
                            disabled: busy,
                            onclick: {
                                let space_id = props.space_id.clone();
                                let agent_id = m_for_promote.agent_id.clone();
                                let on_status_change = props.on_status_change;
                                move |_| {
                                    let sid = space_id.clone();
                                    let aid = agent_id.clone();
                                    on_status_change.call((aid.clone(), ActionStatus::Pending));
                                    spawn(async move {
                                        let client = X0xClient::new();
                                        match client
                                            .set_named_group_member_role(&sid, &aid, GroupRole::Admin)
                                            .await
                                        {
                                            Ok(()) => on_status_change.call((
                                                aid,
                                                ActionStatus::Ok("Promoted to admin.".into()),
                                            )),
                                            Err(e) => on_status_change.call((
                                                aid,
                                                ActionStatus::Err(format!("{e}")),
                                            )),
                                        }
                                    });
                                }
                            },
                            "Promote to admin"
                        }
                    }

                    if props.member.state != communitas_x0x_client::GroupMemberState::Banned {
                        button {
                            style: danger_small_button_style(busy),
                            disabled: busy,
                            onclick: {
                                let space_id = props.space_id.clone();
                                let agent_id = m_for_ban.agent_id.clone();
                                let on_status_change = props.on_status_change;
                                move |_| {
                                    let sid = space_id.clone();
                                    let aid = agent_id.clone();
                                    on_status_change.call((aid.clone(), ActionStatus::Pending));
                                    spawn(async move {
                                        let client = X0xClient::new();
                                        match client.ban_group_member(&sid, &aid).await {
                                            Ok(()) => on_status_change.call((
                                                aid,
                                                ActionStatus::Ok("Banned.".into()),
                                            )),
                                            Err(e) => on_status_change.call((
                                                aid,
                                                ActionStatus::Err(format!("{e}")),
                                            )),
                                        }
                                    });
                                }
                            },
                            "Ban"
                        }
                    } else {
                        button {
                            style: small_button_style(busy),
                            disabled: busy,
                            onclick: {
                                let space_id = props.space_id.clone();
                                let agent_id = m_for_ban.agent_id.clone();
                                let on_status_change = props.on_status_change;
                                move |_| {
                                    let sid = space_id.clone();
                                    let aid = agent_id.clone();
                                    on_status_change.call((aid.clone(), ActionStatus::Pending));
                                    spawn(async move {
                                        let client = X0xClient::new();
                                        match client.unban_group_member(&sid, &aid).await {
                                            Ok(()) => on_status_change.call((
                                                aid,
                                                ActionStatus::Ok("Unbanned.".into()),
                                            )),
                                            Err(e) => on_status_change.call((
                                                aid,
                                                ActionStatus::Err(format!("{e}")),
                                            )),
                                        }
                                    });
                                }
                            },
                            "Unban"
                        }
                    }
                }
            }
        }

        InlineStatus { status: props.status.clone() }
    }
}

// ── Join-requests panel ──────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct RequestsPanelProps {
    space_id: String,
    requests: Vec<JoinRequest>,
}

#[component]
fn RequestsPanel(props: RequestsPanelProps) -> Element {
    let mut row_status = use_signal(std::collections::HashMap::<String, ActionStatus>::new);
    let pending: Vec<JoinRequest> = props
        .requests
        .iter()
        .filter(|r| r.status == JoinRequestStatus::Pending)
        .cloned()
        .collect();

    rsx! {
        PanelShell {
            title: format!("Join requests ({})", pending.len()),
            subtitle: "Review pending access requests and approve or reject.".to_owned(),
            children: rsx! {
                if pending.is_empty() {
                    HintLine { message: "No pending requests." }
                } else {
                    ul {
                        style: "list-style: none; margin: 0; padding: 0;",
                        for req in pending {
                            {
                                let rid = req.request_id.clone();
                                let status = row_status
                                    .read()
                                    .get(&rid)
                                    .cloned()
                                    .unwrap_or(ActionStatus::Idle);
                                let space_id = props.space_id.clone();
                                rsx! {
                                    li {
                                        key: "{req.request_id}",
                                        RequestRow {
                                            request: req.clone(),
                                            space_id,
                                            status,
                                            on_status_change: move |(id, new): (String, ActionStatus)| {
                                                row_status.write().insert(id, new);
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
}

#[derive(Props, Clone, PartialEq)]
struct RequestRowProps {
    request: JoinRequest,
    space_id: String,
    status: ActionStatus,
    on_status_change: EventHandler<(String, ActionStatus)>,
}

#[component]
fn RequestRow(props: RequestRowProps) -> Element {
    let request = props.request.clone();
    let requester = shorten(&request.requester_agent_id);
    let message = request
        .message
        .clone()
        .unwrap_or_else(|| "(no message)".to_owned());
    let busy = matches!(props.status, ActionStatus::Pending);

    let sid_approve = props.space_id.clone();
    let sid_reject = props.space_id.clone();
    let rid_approve = request.request_id.clone();
    let rid_reject = request.request_id.clone();
    let on_status_approve = props.on_status_change;
    let on_status_reject = props.on_status_change;

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: flex-start; \
                 gap: {}; \
                 padding: {} {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 background: {}; \
                 margin-bottom: {};",
                spacing::SM,
                spacing::SM,
                spacing::BASE,
                semantic::BORDER_DEFAULT,
                radius::LG,
                semantic::BG_SECONDARY,
                spacing::XS,
            ),

            div {
                style: format!(
                    "flex: 1; \
                     display: flex; \
                     flex-direction: column; \
                     gap: {};",
                    spacing::XXS,
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY,
                    ),
                    "{requester}"
                }

                span {
                    style: format!(
                        "font-size: {}; color: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_SECONDARY,
                    ),
                    "{message}"
                }
            }

            div {
                style: format!("display: flex; gap: {};", spacing::XS),

                button {
                    style: small_button_style(busy),
                    disabled: busy,
                    onclick: move |_| {
                        let sid = sid_approve.clone();
                        let rid = rid_approve.clone();
                        on_status_approve.call((rid.clone(), ActionStatus::Pending));
                        spawn(async move {
                            let client = X0xClient::new();
                            match client.approve_join_request(&sid, &rid).await {
                                Ok(()) => on_status_approve.call((
                                    rid,
                                    ActionStatus::Ok("Approved.".into()),
                                )),
                                Err(e) => on_status_approve
                                    .call((rid, ActionStatus::Err(format!("{e}")))),
                            }
                        });
                    },
                    "Approve"
                }

                button {
                    style: danger_small_button_style(busy),
                    disabled: busy,
                    onclick: move |_| {
                        let sid = sid_reject.clone();
                        let rid = rid_reject.clone();
                        on_status_reject.call((rid.clone(), ActionStatus::Pending));
                        spawn(async move {
                            let client = X0xClient::new();
                            match client.reject_join_request(&sid, &rid).await {
                                Ok(()) => on_status_reject
                                    .call((rid, ActionStatus::Ok("Rejected.".into()))),
                                Err(e) => on_status_reject
                                    .call((rid, ActionStatus::Err(format!("{e}")))),
                            }
                        });
                    },
                    "Reject"
                }
            }
        }

        InlineStatus { status: props.status.clone() }
    }
}

// ── Shared presentational helpers ────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct PanelShellProps {
    title: String,
    subtitle: String,
    children: Element,
}

#[component]
fn PanelShell(props: PanelShellProps) -> Element {
    rsx! {
        div {
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
                semantic::BG_PRIMARY,
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
                "{props.title}"
            }

            span {
                style: format!(
                    "font-size: {}; color: {};",
                    typography::SIZE_XS,
                    semantic::TEXT_MUTED,
                ),
                "{props.subtitle}"
            }

            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct HintProps {
    message: &'static str,
}

#[component]
fn HintLine(props: HintProps) -> Element {
    rsx! {
        span {
            style: format!(
                "font-size: {}; color: {};",
                typography::SIZE_XS,
                semantic::TEXT_MUTED,
            ),
            "{props.message}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StatusProps {
    status: ActionStatus,
}

#[component]
fn StatusLine(props: StatusProps) -> Element {
    match props.status {
        ActionStatus::Idle => rsx! {},
        ActionStatus::Pending => rsx! {
            span {
                style: format!("font-size: {}; color: {};", typography::SIZE_XS, semantic::TEXT_MUTED),
                "Working…"
            }
        },
        ActionStatus::Ok(msg) => rsx! {
            span {
                style: format!("font-size: {}; color: {};", typography::SIZE_XS, semantic::SUCCESS),
                "{msg}"
            }
        },
        ActionStatus::Err(msg) => rsx! {
            span {
                style: format!("font-size: {}; color: {};", typography::SIZE_XS, semantic::ERROR),
                "{msg}"
            }
        },
    }
}

#[component]
fn InlineStatus(props: StatusProps) -> Element {
    match props.status {
        ActionStatus::Idle | ActionStatus::Pending => rsx! {},
        _ => rsx! { StatusLine { status: props.status } },
    }
}

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
                 background: rgba(239, 68, 68, 0.08); \
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

// ── Policy-axis dropdowns ────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct PresetDropdownProps {
    disabled: bool,
    value: Option<GroupPolicyPreset>,
    on_change: EventHandler<Option<GroupPolicyPreset>>,
}

#[component]
fn PresetDropdown(props: PresetDropdownProps) -> Element {
    let current = props.value.map(preset_to_wire).unwrap_or("");
    rsx! {
        label {
            style: dropdown_label_style(),
            span { style: dropdown_caption_style(), "Preset" }
            select {
                style: dropdown_style(props.disabled),
                disabled: props.disabled,
                value: "{current}",
                onchange: move |evt: Event<FormData>| {
                    let v = evt.value();
                    let choice = match v.as_str() {
                        "" => None,
                        "private_secure" => Some(GroupPolicyPreset::PrivateSecure),
                        "public_request_secure" => Some(GroupPolicyPreset::PublicRequestSecure),
                        "public_open" => Some(GroupPolicyPreset::PublicOpen),
                        "public_announce" => Some(GroupPolicyPreset::PublicAnnounce),
                        _ => None,
                    };
                    props.on_change.call(choice);
                },
                option { value: "", "(leave unchanged)" }
                option { value: "private_secure", "private_secure" }
                option { value: "public_request_secure", "public_request_secure" }
                option { value: "public_open", "public_open" }
                option { value: "public_announce", "public_announce" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DiscoverabilityDropdownProps {
    disabled: bool,
    value: Option<GroupDiscoverability>,
    on_change: EventHandler<Option<GroupDiscoverability>>,
}

#[component]
fn DiscoverabilityDropdown(props: DiscoverabilityDropdownProps) -> Element {
    let current = match props.value {
        None => "",
        Some(GroupDiscoverability::Hidden) => "hidden",
        Some(GroupDiscoverability::ListedToContacts) => "listed_to_contacts",
        Some(GroupDiscoverability::PublicDirectory) => "public_directory",
    };
    rsx! {
        label {
            style: dropdown_label_style(),
            span { style: dropdown_caption_style(), "Discoverability" }
            select {
                style: dropdown_style(props.disabled),
                disabled: props.disabled,
                value: "{current}",
                onchange: move |evt: Event<FormData>| {
                    let v = evt.value();
                    let choice = match v.as_str() {
                        "hidden" => Some(GroupDiscoverability::Hidden),
                        "listed_to_contacts" => Some(GroupDiscoverability::ListedToContacts),
                        "public_directory" => Some(GroupDiscoverability::PublicDirectory),
                        _ => None,
                    };
                    props.on_change.call(choice);
                },
                option { value: "", "(leave unchanged)" }
                option { value: "hidden", "hidden" }
                option { value: "listed_to_contacts", "listed_to_contacts" }
                option { value: "public_directory", "public_directory" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct AdmissionDropdownProps {
    disabled: bool,
    value: Option<GroupAdmission>,
    on_change: EventHandler<Option<GroupAdmission>>,
}

#[component]
fn AdmissionDropdown(props: AdmissionDropdownProps) -> Element {
    let current = match props.value {
        None => "",
        Some(GroupAdmission::InviteOnly) => "invite_only",
        Some(GroupAdmission::RequestAccess) => "request_access",
        Some(GroupAdmission::OpenJoin) => "open_join",
    };
    rsx! {
        label {
            style: dropdown_label_style(),
            span { style: dropdown_caption_style(), "Admission" }
            select {
                style: dropdown_style(props.disabled),
                disabled: props.disabled,
                value: "{current}",
                onchange: move |evt: Event<FormData>| {
                    let v = evt.value();
                    let choice = match v.as_str() {
                        "invite_only" => Some(GroupAdmission::InviteOnly),
                        "request_access" => Some(GroupAdmission::RequestAccess),
                        "open_join" => Some(GroupAdmission::OpenJoin),
                        _ => None,
                    };
                    props.on_change.call(choice);
                },
                option { value: "", "(leave unchanged)" }
                option { value: "invite_only", "invite_only" }
                option { value: "request_access", "request_access" }
                option { value: "open_join", "open_join" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ConfidentialityDropdownProps {
    disabled: bool,
    value: Option<GroupConfidentiality>,
    on_change: EventHandler<Option<GroupConfidentiality>>,
}

#[component]
fn ConfidentialityDropdown(props: ConfidentialityDropdownProps) -> Element {
    let current = match props.value {
        None => "",
        Some(GroupConfidentiality::MlsEncrypted) => "mls_encrypted",
        Some(GroupConfidentiality::SignedPublic) => "signed_public",
    };
    rsx! {
        label {
            style: dropdown_label_style(),
            span { style: dropdown_caption_style(), "Confidentiality" }
            select {
                style: dropdown_style(props.disabled),
                disabled: props.disabled,
                value: "{current}",
                onchange: move |evt: Event<FormData>| {
                    let v = evt.value();
                    let choice = match v.as_str() {
                        "mls_encrypted" => Some(GroupConfidentiality::MlsEncrypted),
                        "signed_public" => Some(GroupConfidentiality::SignedPublic),
                        _ => None,
                    };
                    props.on_change.call(choice);
                },
                option { value: "", "(leave unchanged)" }
                option { value: "mls_encrypted", "mls_encrypted" }
                option { value: "signed_public", "signed_public" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ReadAccessDropdownProps {
    disabled: bool,
    value: Option<GroupReadAccess>,
    on_change: EventHandler<Option<GroupReadAccess>>,
}

#[component]
fn ReadAccessDropdown(props: ReadAccessDropdownProps) -> Element {
    let current = match props.value {
        None => "",
        Some(GroupReadAccess::MembersOnly) => "members_only",
        Some(GroupReadAccess::Public) => "public",
    };
    rsx! {
        label {
            style: dropdown_label_style(),
            span { style: dropdown_caption_style(), "Read access" }
            select {
                style: dropdown_style(props.disabled),
                disabled: props.disabled,
                value: "{current}",
                onchange: move |evt: Event<FormData>| {
                    let v = evt.value();
                    let choice = match v.as_str() {
                        "members_only" => Some(GroupReadAccess::MembersOnly),
                        "public" => Some(GroupReadAccess::Public),
                        _ => None,
                    };
                    props.on_change.call(choice);
                },
                option { value: "", "(leave unchanged)" }
                option { value: "members_only", "members_only" }
                option { value: "public", "public" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct WriteAccessDropdownProps {
    disabled: bool,
    value: Option<GroupWriteAccess>,
    on_change: EventHandler<Option<GroupWriteAccess>>,
}

#[component]
fn WriteAccessDropdown(props: WriteAccessDropdownProps) -> Element {
    let current = match props.value {
        None => "",
        Some(GroupWriteAccess::MembersOnly) => "members_only",
        Some(GroupWriteAccess::ModeratedPublic) => "moderated_public",
        Some(GroupWriteAccess::AdminOnly) => "admin_only",
    };
    rsx! {
        label {
            style: dropdown_label_style(),
            span { style: dropdown_caption_style(), "Write access" }
            select {
                style: dropdown_style(props.disabled),
                disabled: props.disabled,
                value: "{current}",
                onchange: move |evt: Event<FormData>| {
                    let v = evt.value();
                    let choice = match v.as_str() {
                        "members_only" => Some(GroupWriteAccess::MembersOnly),
                        "moderated_public" => Some(GroupWriteAccess::ModeratedPublic),
                        "admin_only" => Some(GroupWriteAccess::AdminOnly),
                        _ => None,
                    };
                    props.on_change.call(choice);
                },
                option { value: "", "(leave unchanged)" }
                option { value: "members_only", "members_only" }
                option { value: "moderated_public", "moderated_public" }
                option { value: "admin_only", "admin_only" }
            }
        }
    }
}

fn dropdown_label_style() -> String {
    format!(
        "display: flex; flex-direction: column; gap: {};",
        spacing::XXS,
    )
}

fn dropdown_caption_style() -> String {
    format!(
        "font-size: {}; font-weight: {}; color: {};",
        typography::SIZE_XS,
        typography::WEIGHT_MEDIUM,
        semantic::TEXT_SECONDARY,
    )
}

fn dropdown_style(disabled: bool) -> String {
    format!(
        "padding: {} {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         background: {}; \
         color: {}; \
         font-size: {}; \
         cursor: {};",
        spacing::XS,
        spacing::SM,
        semantic::BORDER_DEFAULT,
        radius::MD,
        semantic::BG_TERTIARY,
        semantic::TEXT_PRIMARY,
        typography::SIZE_SM,
        if disabled { "not-allowed" } else { "pointer" },
    )
}

// ── Misc helpers ─────────────────────────────────────────────────────────

fn role_at_least(role: GroupRole, minimum: GroupRole) -> bool {
    role_rank(role) >= role_rank(minimum)
}

fn role_rank(role: GroupRole) -> u8 {
    match role {
        GroupRole::Owner => 4,
        GroupRole::Admin => 3,
        GroupRole::Moderator => 2,
        GroupRole::Member => 1,
        GroupRole::Guest => 0,
    }
}

fn role_label(role: GroupRole) -> &'static str {
    match role {
        GroupRole::Owner => "owner",
        GroupRole::Admin => "admin",
        GroupRole::Moderator => "moderator",
        GroupRole::Member => "member",
        GroupRole::Guest => "guest",
    }
}

fn member_state_label(state: communitas_x0x_client::GroupMemberState) -> &'static str {
    match state {
        communitas_x0x_client::GroupMemberState::Active => "active",
        communitas_x0x_client::GroupMemberState::Pending => "pending",
        communitas_x0x_client::GroupMemberState::Removed => "removed",
        communitas_x0x_client::GroupMemberState::Banned => "banned",
    }
}

fn member_state_tone(state: communitas_x0x_client::GroupMemberState) -> &'static str {
    match state {
        communitas_x0x_client::GroupMemberState::Active => semantic::SUCCESS,
        communitas_x0x_client::GroupMemberState::Pending => semantic::WARNING,
        communitas_x0x_client::GroupMemberState::Removed => semantic::TEXT_MUTED,
        communitas_x0x_client::GroupMemberState::Banned => semantic::ERROR,
    }
}

fn preset_to_wire(p: GroupPolicyPreset) -> &'static str {
    match p {
        GroupPolicyPreset::PrivateSecure => "private_secure",
        GroupPolicyPreset::PublicRequestSecure => "public_request_secure",
        GroupPolicyPreset::PublicOpen => "public_open",
        GroupPolicyPreset::PublicAnnounce => "public_announce",
    }
}

fn shorten(id: &str) -> String {
    if id.len() <= 16 {
        id.to_owned()
    } else {
        format!("{}…{}", &id[..8], &id[id.len() - 6..])
    }
}

fn small_button_style(busy: bool) -> String {
    format!(
        "padding: {} {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         background: transparent; \
         color: {}; \
         font-size: {}; \
         cursor: {};",
        spacing::XXS,
        spacing::SM,
        semantic::BORDER_DEFAULT,
        radius::MD,
        semantic::TEXT_PRIMARY,
        typography::SIZE_XS,
        if busy { "not-allowed" } else { "pointer" },
    )
}

fn danger_small_button_style(busy: bool) -> String {
    format!(
        "padding: {} {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         background: transparent; \
         color: {}; \
         font-size: {}; \
         cursor: {};",
        spacing::XXS,
        spacing::SM,
        semantic::ERROR,
        radius::MD,
        semantic::ERROR,
        typography::SIZE_XS,
        if busy { "not-allowed" } else { "pointer" },
    )
}

fn primary_button_style(busy: bool) -> String {
    format!(
        "padding: {} {}; \
         border: none; \
         border-radius: {}; \
         background: {}; \
         color: {}; \
         font-size: {}; \
         font-weight: {}; \
         cursor: {};",
        spacing::XS,
        spacing::BASE,
        radius::LG,
        semantic::PRIMARY,
        semantic::TEXT_INVERSE,
        typography::SIZE_SM,
        typography::WEIGHT_SEMIBOLD,
        if busy { "not-allowed" } else { "pointer" },
    )
}

fn danger_button_style(busy: bool) -> String {
    format!(
        "padding: {} {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         background: transparent; \
         color: {}; \
         font-size: {}; \
         font-weight: {}; \
         cursor: {};",
        spacing::XS,
        spacing::BASE,
        semantic::ERROR,
        radius::LG,
        semantic::ERROR,
        typography::SIZE_SM,
        typography::WEIGHT_SEMIBOLD,
        if busy { "not-allowed" } else { "pointer" },
    )
}
