use crate::container::EngineState;
use communitas_container::{Tip, Op};
use tokio::net::lookup_host;
use ant_quic::crypto::raw_public_keys::RawPublicKeyConfigBuilder;
use saorsa_fec::{FecCodec, FecParams};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{oneshot, RwLock};
use tokio::time::{sleep, Duration};

#[derive(Default)]
pub struct TipWatcherState {
    handle: Option<tokio::task::JoinHandle<()>>,
    cancel_tx: Option<oneshot::Sender<()>>,
    last_tip: Option<Tip>,
}

#[tauri::command]
pub async fn sync_start_tip_watcher(
    app: AppHandle,
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    watcher: State<'_, Arc<RwLock<TipWatcherState>>>,
    interval_ms: Option<u64>,
) -> Result<bool, String> {
    // Stop any existing watcher first
    {
        let mut w = watcher.write().await;
        if let Some(tx) = w.cancel_tx.take() {
            let _ = tx.send(());
        }
        if let Some(h) = w.handle.take() {
            // Detach; task will observe cancel via channel
            let _ = h;
        }
        w.last_tip = None;
    }

    let poll = Duration::from_millis(interval_ms.unwrap_or(1000));
    let (tx, rx) = oneshot::channel::<()>();
    let app_clone = app.clone();
    let container_clone = container.inner().clone();

    let handle = tokio::spawn(async move {
        let mut cancel = rx;
        loop {
            // Check cancellation
            if let Ok(_) | Err(oneshot::error::TryRecvError::Closed) = cancel.try_recv() {
                break;
            }

            // Compute current tip (ignore errors; just continue)
            let tip_opt = {
                let guard = container_clone.read().await;
                if let Some(state) = guard.as_ref() {
                    match state.current_tip() {
                        Ok(t) => Some(t),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            };

            if let Some(tip) = tip_opt {
                let payload = serde_json::json!({
                    "root": hex::encode(tip.root),
                    "count": tip.count,
                });
                let _ = app_clone.emit("container-tip", payload);
            }

            sleep(poll).await;
        }
    });

    let mut w = watcher.write().await;
    w.handle = Some(handle);
    w.cancel_tx = Some(tx);
    Ok(true)
}

#[tauri::command]
pub async fn sync_stop_tip_watcher(watcher: State<'_, Arc<RwLock<TipWatcherState>>>) -> Result<bool, String> {
    let mut w = watcher.write().await;
    if let Some(tx) = w.cancel_tx.take() { let _ = tx.send(()); }
    w.handle.take();
    w.last_tip = None;
    Ok(true)
}

/// Attempt a FEC repair given k/m and provided shares. `shares` is a vector where
/// some entries may be `None` for missing pieces.
#[tauri::command]
pub async fn sync_repair_fec(
    data_shards: u16,
    parity_shards: u16,
    shares: Vec<Option<Vec<u8>>>,
) -> Result<Vec<u8>, String> {
    if data_shards == 0 {
        return Err("data_shards must be > 0".into());
    }
    // Create codec and attempt decode
    let params = FecParams::new(data_shards, parity_shards)
        .map_err(|e| format!("fec params: {e:?}"))?;
    let codec = FecCodec::new(params).map_err(|e| format!("fec codec: {e:?}"))?;
    codec.decode(&shares).map_err(|e| format!("fec decode: {e:?}"))
}

/// Delta fetcher over QUIC (IPv4-first). Returns number of ops fetched.
#[tauri::command]
pub async fn sync_fetch_deltas(
    app: AppHandle,
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    rpk: State<'_, Arc<RwLock<crate::security::raw_spki::RawSpkiState>>>,
    peer_addr: String,
) -> Result<u64, String> {
    // PQC + QUIC only. No TCP fallback.
    let pinned_key = rpk
        .read()
        .await
        .get()
        .or_else(crate::security::raw_spki::parse_env_pinned_spki);
    // Derive request hints from current tip if available
    let (from_root_hex, since_count) = {
        let g = container.read().await;
        if let Some(state) = g.as_ref() {
            match state.current_tip() {
                Ok(tip) => (Some(hex::encode(tip.root)), Some(tip.count)),
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        }
    };

    let _ = app.emit("sync-progress", serde_json::json!({
        "phase": "request",
        "peer": peer_addr,
        "since": since_count,
    }));

    let ops = fetch_deltas_over_ant_quic(&peer_addr, from_root_hex, since_count, pinned_key).await?;
    let received = ops.len() as u64;
    let _ = app.emit("sync-progress", serde_json::json!({
        "phase": "received",
        "peer": peer_addr,
        "ops": received,
    }));
    let count = ops.len() as u64;
    if count == 0 { return Ok(0); }
    if let Some(state) = container.read().await.as_ref() {
        let tip = state.apply_ops(&ops)?;
        let _ = app.emit("sync-progress", serde_json::json!({
            "phase": "applied",
            "peer": peer_addr,
            "new_count": tip.count,
            "root": hex::encode(tip.root),
        }));
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fec_roundtrip_recovers_with_missing_shares() {
        let data = b"hello sync repair fec".to_vec();
        let k: u16 = 4;
        let m: u16 = 2;
        let params = FecParams::new(k, m).unwrap();
        let codec = FecCodec::new(params).unwrap();
        let shares = codec.encode(&data).unwrap();
        assert_eq!(shares.len() as u16, k + m);

        // Drop two shares, keep k shares total
        let mut provided: Vec<Option<Vec<u8>>> = shares.into_iter().map(Some).collect();
        provided[1] = None;
        provided[4] = None;

        let decoded = codec.decode(&provided).unwrap();
        assert_eq!(decoded, data);
    }
}

// Request/response payloads for QUIC delta fetch
#[derive(serde::Serialize)]
struct DeltaRequest<'a> {
    from_root_hex: Option<&'a str>,
    want_since_count: Option<u64>,
}

#[derive(serde::Deserialize)]
struct DeltaResponse { ops: Vec<Op> }

// ---------------- ant-quic client (QUIC, IPv4-first) -----------------
async fn fetch_deltas_over_ant_quic(
    peer_addr: &str,
    from_root_hex: Option<String>,
    want_since_count: Option<u64>,
    pinned_key: Option<[u8; 32]>,
) -> Result<Vec<Op>, String> {
    use ant_quic::high_level::Endpoint as QuicEndpoint;
    use ant_quic::config::ClientConfig as QuicClientConfig;
    use std::sync::Arc as StdArc;
    // write_all is available on send stream without extra trait import

    // Bind dual-stack client endpoint ([::]:0) to allow IPv4+IPv6 when possible
    let bind_addr: std::net::SocketAddr = (std::net::Ipv6Addr::UNSPECIFIED, 0).into();
    let endpoint = QuicEndpoint::client(bind_addr).map_err(|e| format!("quic client bind: {e}"))?;

    // Build client config for Raw Public Keys (RFC 7250 style)
    let client_cfg = {
        let mut builder = RawPublicKeyConfigBuilder::new().enable_certificate_type_extensions();
        if let Some(pk) = pinned_key {
            builder = builder.add_trusted_key(pk);
        } else if std::env::var("COMMUNITAS_RPK_ALLOW_ANY").is_ok() {
            builder = builder.allow_any_key();
        } else {
            return Err("no pinned raw key available; set via Presence, Tauri command, or COMMUNITAS_RPK_ALLOW_ANY for dev".into());
        }
        let rustls_cfg = builder
            .build_client_config()
            .map_err(|e| format!("raw public key client config: {e}"))?;
        let quic_tls: ant_quic::crypto::rustls::QuicClientConfig = StdArc::new(rustls_cfg)
            .try_into()
            .map_err(|e| format!("convert rustls cfg to quic tls cfg: {e}"))?;
        let client = QuicClientConfig::new(StdArc::new(quic_tls));
        // Enforce PQC support on client config (placeholder in ant-quic, for intent signaling)
        ant_quic::crypto::pqc::rustls_provider::with_pqc_support(client)
            .map_err(|e| format!("enable PQC on client: {e:?}"))?
    };
    let mut ep = endpoint.clone();
    ep.set_default_client_config(client_cfg);

    // Resolve and prefer IPv4 addresses for connect attempts
    let (host_for_sni, addrs) = resolve_host_ipv4_first(peer_addr).await?;

    let mut last_err: Option<String> = None;
    for addr in addrs {
        // With raw public keys we can use any SNI; use host if available or 'peer'.
        let sni = host_for_sni.as_deref().unwrap_or("peer");
        match ep.connect(addr, sni) {
            Ok(connecting) => match connecting.await {
                Ok(conn) => {
                    // Open bi-directional stream
                    let (mut send, mut recv) = conn
                        .open_bi()
                        .await
                        .map_err(|e| format!("open_bi: {e}"))?;
                    // Send JSON line
                    let req = serde_json::to_string(&DeltaRequest {
                        from_root_hex: from_root_hex.as_deref(),
                        want_since_count,
                    })
                    .map_err(|e| format!("encode request: {e}"))?
                        + "\n";
                    send.write_all(req.as_bytes())
                        .await
                        .map_err(|e| format!("send request: {e}"))?;
                    send.finish().map_err(|e| format!("finish stream: {e}"))?;

                    // Read full response (single line JSON)
                    let bytes = match recv.read_to_end(1024 * 1024).await {
                        Ok(v) => v,
                        Err(e) => return Err(format!("read deltas: {e}")),
                    };
                    let text = String::from_utf8(bytes).map_err(|e| format!("utf8: {e}"))?;
                    let resp: DeltaResponse = serde_json::from_str(text.trim_end())
                        .map_err(|e| format!("decode deltas: {e}"))?;
                    return Ok(resp.ops);
                }
                Err(e) => last_err = Some(format!("handshake failed to {addr}: {e}")),
            },
            Err(e) => last_err = Some(format!("connect setup failed to {addr}: {e}")),
        }
    }
    Err(last_err.unwrap_or_else(|| "no QUIC addresses reachable".into()))
}

async fn resolve_host_ipv4_first(peer_addr: &str) -> Result<(Option<String>, Vec<std::net::SocketAddr>), String> {
    // Extract host for SNI if present
    let sni = peer_addr
        .rsplit_once(':')
        .map(|(h, _)| h)
        .and_then(|h| if h.parse::<std::net::IpAddr>().is_ok() { None } else { Some(h.to_string()) });
    let mut v4 = Vec::new();
    let mut v6 = Vec::new();
    let it = lookup_host(peer_addr)
        .await
        .map_err(|e| format!("resolve {peer_addr}: {e}"))?;
    for sa in it {
        match sa.ip() {
            std::net::IpAddr::V4(_) => v4.push(sa),
            std::net::IpAddr::V6(_) => v6.push(sa),
        }
    }
    let addrs = v4.into_iter().chain(v6.into_iter()).collect();
    Ok((sni, addrs))
}
// no helper needed for TLS pinning
