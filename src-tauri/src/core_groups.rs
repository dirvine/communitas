use saorsa_core::api::{group_identity_create, group_identity_publish, group_identity_fetch, identity_fetch, group_identity_canonical_sign_bytes, group_identity_update_members_signed, GroupIdentityPacketV1, MemberRef};
use saorsa_core::fwid::fw_to_key;
use saorsa_core::dht::Key as DhtKey;
use saorsa_core::quantum_crypto::{MlDsa65, MlDsaOperations};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::core_context::CoreContext;

#[derive(serde::Serialize)]
pub struct GroupCreateResult {
    pub id_hex: String,
    pub words: [String; 4],
}

/// Create a group identity with current user as initial member and publish it
#[tauri::command]
pub async fn core_group_create(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    words: [String; 4],
) -> Result<GroupCreateResult, String> {
    // Validate four-word format
    for word in &words {
        if word.is_empty() {
            return Err("Group words cannot be empty".to_string());
        }
        if word.len() > 20 {
            return Err("Group words too long (max 20 characters)".to_string());
        }
        // Basic alphanumeric check
        if !word.chars().all(|c| c.is_alphanumeric()) {
            return Err(format!("Group word '{}' contains invalid characters", word));
        }
    }
    let guard = shared.read().await;
    let ctx = guard.as_ref().ok_or_else(|| "Core not initialized".to_string())?;

    // Build initial members with self
    let my_words: Vec<String> = ctx.four_words.split('-').map(|s| s.to_string()).collect();
    if my_words.len() != 4 {
        return Err("Invalid local identity four-word address".to_string());
    }
    let my_key = fw_to_key([
        my_words[0].clone(),
        my_words[1].clone(),
        my_words[2].clone(),
        my_words[3].clone(),
    ])
    .map_err(|e| format!("Failed to derive member key: {}", e))?;
    let member = MemberRef {
        member_id: my_key,
        member_pk: ctx.identity.quantum_identity.ml_dsa_public_key.clone(),
    };

    let (packet, kp) = group_identity_create(words.clone(), vec![member])
        .map_err(|e| format!("group_identity_create failed: {}", e))?;

    group_identity_publish(packet.clone())
        .await
        .map_err(|e| format!("group_identity_publish failed: {}", e))?;

    // Store group signing key for future membership updates
    {
        let mut w = (shared.inner()).write().await;
        if let Some(ref mut ctx_mut) = *w {
            ctx_mut.group_keys.insert(hex::encode(packet.id.as_bytes()), kp);
        }
    }

    Ok(GroupCreateResult {
        id_hex: hex::encode(packet.id.as_bytes()),
        words,
    })
}

#[tauri::command]
pub async fn core_group_add_member(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    group_words: [String; 4],
    member_words: [String; 4],
) -> Result<bool, String> {
    let mut guard = shared.write().await;
    let ctx = guard.as_mut().ok_or_else(|| "Core not initialized".to_string())?;

    let group_id = fw_to_key(group_words.clone()).map_err(|e| format!("fw_to_key group: {}", e))?;
    let member_id = fw_to_key(member_words.clone()).map_err(|e| format!("fw_to_key member: {}", e))?;

    // Fetch member identity to get pk
    let member_pkt = identity_fetch(member_id.clone())
        .await
        .map_err(|e| format!("identity_fetch failed: {}", e))?;

    // Fetch group identity packet
    let mut gip = group_identity_fetch(group_id.clone())
        .await
        .map_err(|e| format!("group_identity_fetch failed: {}", e))?;

    // Update members if not present
    if !gip.members.iter().any(|m| m.member_id == member_id) {
        gip.members.push(MemberRef { member_id: member_id.clone(), member_pk: member_pkt.pk.clone() });
    }

    // Recompute root and re-sign with stored group key (using saorsa-core helper)
    let new_root = {
        // Recompute locally for canonical signing; identical to core's function
        let mut ids: Vec<[u8; 32]> = gip.members.iter().map(|m| *m.member_id.as_bytes()).collect();
        ids.sort_unstable();
        let mut hasher = blake3::Hasher::new();
        for id in ids { hasher.update(&id); }
        let out = hasher.finalize();
        DhtKey::from(*out.as_bytes())
    };
    let msg = group_identity_canonical_sign_bytes(&gip.id.clone().into(), &new_root.into());

    let id_hex = hex::encode(gip.id.as_bytes());
    let kp = ctx
        .group_keys
        .get(&id_hex)
        .ok_or_else(|| "Group signing key not available".to_string())?;

    let ml = MlDsa65::new();
    let sig = ml.sign(&kp.group_sk, &msg).map_err(|e| format!("group sign failed: {:?}", e))?;

    // Call core update to store
    group_identity_update_members_signed(
        gip.id.clone(),
        gip.members.clone(),
        kp.group_pk.as_bytes().to_vec(),
        saorsa_core::auth::Sig::new(sig.0.to_vec()),
    )
    .await
    .map_err(|e| format!("group update failed: {}", e))?;

    Ok(true)
}

#[tauri::command]
pub async fn core_group_remove_member(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    group_words: [String; 4],
    member_words: [String; 4],
) -> Result<bool, String> {
    let mut guard = shared.write().await;
    let ctx = guard.as_mut().ok_or_else(|| "Core not initialized".to_string())?;

    let group_id = fw_to_key(group_words.clone()).map_err(|e| format!("fw_to_key group: {}", e))?;
    let member_id = fw_to_key(member_words.clone()).map_err(|e| format!("fw_to_key member: {}", e))?;

    // Fetch group identity packet
    let mut gip = group_identity_fetch(group_id.clone())
        .await
        .map_err(|e| format!("group_identity_fetch failed: {}", e))?;

    let before = gip.members.len();
    gip.members.retain(|m| m.member_id != member_id);
    if gip.members.len() == before {
        return Ok(true); // no-op
    }

    // Recompute root and re-sign using core helper
    let new_root = {
        let mut ids: Vec<[u8; 32]> = gip.members.iter().map(|m| *m.member_id.as_bytes()).collect();
        ids.sort_unstable();
        let mut hasher = blake3::Hasher::new();
        for id in ids { hasher.update(&id); }
        let out = hasher.finalize();
        DhtKey::from(*out.as_bytes())
    };
    let msg = group_identity_canonical_sign_bytes(&gip.id.clone().into(), &new_root.into());
    let id_hex = hex::encode(gip.id.as_bytes());
    let kp = ctx
        .group_keys
        .get(&id_hex)
        .ok_or_else(|| "Group signing key not available".to_string())?;

    let ml = MlDsa65::new();
    let sig = ml.sign(&kp.group_sk, &msg).map_err(|e| format!("group sign failed: {:?}", e))?;

    group_identity_update_members_signed(
        gip.id.clone(),
        gip.members.clone(),
        kp.group_pk.as_bytes().to_vec(),
        saorsa_core::auth::Sig::new(sig.0.to_vec()),
    )
    .await
    .map_err(|e| format!("group update failed: {}", e))?;

    Ok(true)
}
