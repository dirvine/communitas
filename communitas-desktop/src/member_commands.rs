// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Member Management Commands - Thin wrappers around CoreContext EntityService
//!
//! These commands now delegate to the unified EntityService in communitas-core,
//! eliminating code duplication between desktop and TUI applications.

use communitas_core::crdt::EntityType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

// === Member Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub member_id: String,
    pub role: String,
}

#[tauri::command]
pub async fn member_add(
    request: AddMemberRequest,
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
) -> Result<(), String> {
    let core_guard = core_state.read().await;
    let core_ctx = core_guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    core_ctx.entity_service
        .add_member(
            request.entity_type,
            &request.entity_id,
            &request.member_id,
            &request.role,
        )
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberInfo {
    pub member_id: String,
    pub role: String,
    pub joined_at: i64,
    pub deleted: bool,
}

#[tauri::command]
pub async fn member_list(
    entity_type: EntityType,
    entity_id: String,
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
) -> Result<Vec<MemberInfo>, String> {
    let core_guard = core_state.read().await;
    let core_ctx = core_guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    let members = core_ctx.entity_service
        .list_members(entity_type, &entity_id)
        .await
        .map_err(|e| e.to_string())?;

    // Convert core MemberInfo to desktop MemberInfo
    let result = members.into_iter()
        .map(|m| MemberInfo {
            member_id: m.member_id,
            role: m.role,
            joined_at: m.joined_at,
            deleted: m.deleted,
        })
        .collect();

    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveMemberRequest {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub member_id: String,
    pub deleted_by: String,
}

#[tauri::command]
pub async fn member_remove(
    request: RemoveMemberRequest,
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
) -> Result<(), String> {
    let core_guard = core_state.read().await;
    let core_ctx = core_guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    if request.entity_type == communitas_core::crdt::EntityType::Organisation {
        // Use cascading removal for organizations
        core_ctx.entity_service
            .remove_organization_member(
                &request.entity_id,
                &request.member_id,
                &request.deleted_by,
            )
            .await
            .map(|_result| ()) // Ignore summary for now, just succeed/fail
            .map_err(|e| e.to_string())
    } else {
        // Regular removal for non-org entities
        core_ctx.entity_service
            .remove_member(
                request.entity_type,
                &request.entity_id,
                &request.member_id,
                &request.deleted_by,
            )
            .await
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub member_id: String,
    pub new_role: String,
}

#[tauri::command]
pub async fn member_update_role(
    request: UpdateRoleRequest,
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
) -> Result<(), String> {
    let core_guard = core_state.read().await;
    let core_ctx = core_guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    // Update role by re-adding the member with new role
    // (EntityService doesn't have a direct update_role method)
    core_ctx.entity_service
        .add_member(
            request.entity_type,
            &request.entity_id,
            &request.member_id,
            &request.new_role,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn member_prune_tombstones(
    _entity_type: EntityType,
    _entity_id: String,
    _core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
) -> Result<usize, String> {
    // Tombstone pruning is handled automatically by the EntityService
    // Return 0 as we don't track the number of pruned tombstones
    Ok(0)
}
