use communitas_core::keystore::Keystore;
use cc::{AeadConfig, ContainerEngine, FecConfig, Op, Tip};
use communitas_container as cc;
use saorsa_core::quantum_crypto::{MlDsaPublicKey, MlDsaSecretKey};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub struct EngineState {
    engine: Arc<ContainerEngine>,
    id_hex: String,
}

impl EngineState {
    pub fn current_tip(&self) -> Result<Tip, String> {
        self.engine.current_tip().map_err(|e| e.to_string())
    }
    pub fn apply_ops(&self, ops: &[Op]) -> Result<Tip, String> {
        self.engine.apply_ops(ops).map_err(|e| e.to_string())
    }
}

fn data_root() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("COMMUNITAS_DATA_DIR") {
        std::path::PathBuf::from(p)
    } else {
        std::path::PathBuf::from("src-tauri/.communitas-data")
    }
}

async fn get_or_init_engine(
    state: &State<'_, Arc<RwLock<Option<EngineState>>>>,
) -> Result<Arc<ContainerEngine>, String> {
    // Fast path
    if let Some(e) = state.read().await.as_ref().map(|s| s.engine.clone()) {
        return Ok(e);
    }

    // Build from keystore
    let ks = Keystore::new();
    let id_hex = ks.load_current_identity()?;
    let (pk_bytes, sk_bytes) = ks.load_mldsa_keys(&id_hex)?;
    let pk =
        MlDsaPublicKey::from_bytes(&pk_bytes).map_err(|e| format!("invalid ML-DSA pk: {:?}", e))?;
    let sk =
        MlDsaSecretKey::from_bytes(&sk_bytes).map_err(|e| format!("invalid ML-DSA sk: {:?}", e))?;
    let engine = Arc::new(ContainerEngine::new(
        pk,
        sk,
        AeadConfig::default(),
        FecConfig::default(),
    ));
    let mut w = state.write().await;
    *w = Some(EngineState {
        engine: engine.clone(),
        id_hex,
    });
    Ok(engine)
}

#[tauri::command]
pub async fn container_init(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
) -> Result<bool, String> {
    let _ = get_or_init_engine(&container).await?;
    Ok(true)
}

#[tauri::command]
pub async fn container_put_object(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let engine = get_or_init_engine(&container).await?;
    let id_hex = {
        let g = container.read().await;
        g.as_ref()
            .map(|s| s.id_hex.clone())
            .ok_or_else(|| "engine not initialized".to_string())?
    };
    let oid = engine.put_object(&bytes).map_err(|e| e.to_string())?;
    // also place opaque blob in per-identity personal area for offline access
    let handle = hex::encode(oid);
    let path = data_root()
        .join("personal")
        .join(&id_hex)
        .join(format!("{}.data", handle));
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("mkdirs failed: {}", e))?;
    }
    tokio::fs::write(&path, &bytes)
        .await
        .map_err(|e| format!("write object failed: {}", e))?;
    Ok(handle)
}

#[tauri::command]
pub async fn container_get_object(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    oid_hex: String,
) -> Result<Vec<u8>, String> {
    if oid_hex.len() != 64 {
        return Err("invalid oid length".into());
    }
    let raw = hex::decode(&oid_hex).map_err(|e| format!("oid hex decode: {e}"))?;
    let mut oid = [0u8; 32];
    if raw.len() != 32 {
        return Err("invalid oid bytes".into());
    }
    oid.copy_from_slice(&raw);
    let engine = get_or_init_engine(&container).await?;
    match engine.get_object(&oid) {
        Ok(v) => Ok(v),
        Err(_) => {
            // Fallback to per-identity personal store for offline access
            let id_hex = {
                let g = container.read().await;
                g.as_ref()
                    .map(|s| s.id_hex.clone())
                    .ok_or_else(|| "engine not initialized".to_string())?
            };
            let path = data_root()
                .join("personal")
                .join(&id_hex)
                .join(format!("{}.data", oid_hex));
            tokio::fs::read(&path)
                .await
                .map_err(|e| format!("object not found/read failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn container_apply_ops(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    ops: Vec<Op>,
) -> Result<Tip, String> {
    let engine = get_or_init_engine(&container).await?;
    engine.apply_ops(&ops).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn container_current_tip(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
) -> Result<Tip, String> {
    let engine = get_or_init_engine(&container).await?;
    engine.current_tip().map_err(|e| e.to_string())
}
