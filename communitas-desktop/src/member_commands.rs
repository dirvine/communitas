// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

use crate::member_manager::{EntityType, MemberInfo, MemberManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

// State container for member manager
pub struct MemberState {
    pub member_manager: Arc<MemberManager>,
}

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
    state: State<'_, MemberState>,
) -> Result<(), String> {
    state
        .member_manager
        .add_member(
            request.entity_type,
            &request.entity_id,
            &request.member_id,
            &request.role,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn member_list(
    entity_type: EntityType,
    entity_id: String,
    state: State<'_, MemberState>,
) -> Result<Vec<MemberInfo>, String> {
    state
        .member_manager
        .list_members(entity_type, &entity_id)
        .await
        .map_err(|e| e.to_string())
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
    state: State<'_, MemberState>,
) -> Result<(), String> {
    state
        .member_manager
        .remove_member(
            request.entity_type,
            &request.entity_id,
            &request.member_id,
            &request.deleted_by,
        )
        .await
        .map_err(|e| e.to_string())
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
    state: State<'_, MemberState>,
) -> Result<(), String> {
    state
        .member_manager
        .update_role(
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
    entity_type: EntityType,
    entity_id: String,
    state: State<'_, MemberState>,
) -> Result<usize, String> {
    state
        .member_manager
        .prune_tombstones(entity_type, &entity_id)
        .await
        .map_err(|e| e.to_string())
}
