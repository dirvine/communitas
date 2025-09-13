use std::sync::Arc;
use tokio::sync::RwLock;

pub type RootStore = rustls::RootCertStore;

#[derive(Default)]
pub struct TlsPinningState {
    pub roots: Option<Arc<RootStore>>, // pinned root store; if None, use platform verifier
}

fn parse_pem_to_root_store(pem: &str) -> Result<Arc<RootStore>, String> {
    use rustls::pki_types::CertificateDer;
    let mut reader = pem.as_bytes();
    let mut store = rustls::RootCertStore::empty();
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<CertificateDer<'_>>, _>>()
        .map_err(|e| format!("parse pem: {e}"))?;
    if certs.is_empty() {
        return Err("no certificates found in PEM".into());
    }
    for cert in certs {
        store.add(cert).map_err(|e| format!("add root cert: {e}"))?;
    }
    Ok(Arc::new(store))
}

#[tauri::command]
pub async fn sync_set_quic_root_cert_pem(
    state: tauri::State<'_, Arc<RwLock<TlsPinningState>>>,
    pem: String,
) -> Result<bool, String> {
    let roots = parse_pem_to_root_store(&pem)?;
    let mut w = state.write().await;
    w.roots = Some(roots);
    Ok(true)
}

#[tauri::command]
pub async fn sync_clear_quic_root_cert(
    state: tauri::State<'_, Arc<RwLock<TlsPinningState>>>,
) -> Result<bool, String> {
    let mut w = state.write().await;
    w.roots = None;
    Ok(true)
}

// Helper used by non-tauri contexts (e.g., env var path in headless)
pub fn sync_set_quic_root_cert_pem_parse_only(pem: &str) -> Result<Arc<RootStore>, String> {
    parse_pem_to_root_store(pem)
}
