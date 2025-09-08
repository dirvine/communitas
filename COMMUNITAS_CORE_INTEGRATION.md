# Communitas ↔ saorsa‑core Integration

## Mapping
| Communitas feature | saorsa‑core API |
| --- | --- |
| Claim identity | `register_identity(words, &MlDsaKeyPair)` |
| Presence | `register_presence(handle, devices, active_device)` |
| Write message/file | `store_data(handle, bytes, group_size)` or `store_dyad` or `store_with_fec` |
| Read message/file | `get_data(&StorageHandle)` |
| Publish tip | Provided by core when container commit occurs |
| Transport | ant‑quic client via core transport adapter |

## Tauri commands
Create `src-tauri/src/core_cmds.rs`:
```rust
use tauri::State;
use serde::{Deserialize, Serialize};
use saorsa_core::{register_identity, register_presence, store_data, store_dyad, store_with_fec, get_data, MlDsaKeyPair, Device, DeviceType, DeviceId, Endpoint};

#[derive(Default)]
pub struct CoreCtx;

#[tauri::command]
pub async fn core_claim(words: [String;4]) -> Result<String, String> {
    let kp = MlDsaKeyPair::generate().map_err(|e| e.to_string())?;
    let arr = [words[0].as_str(), words[1].as_str(), words[2].as_str(), words[3].as_str()];
    let handle = register_identity(arr, &kp).await.map_err(|e| e.to_string())?;
    Ok(handle.key())
}

#[tauri::command]
pub async fn core_advertise(addr: String, storage_gb: u32) -> Result<(), String> {
    let kp = MlDsaKeyPair::generate().map_err(|e| e.to_string())?; // replace with persisted key
    let handle = register_identity(["temp","temp","temp","temp"], &kp).await.map_err(|e| e.to_string())?; // load existing
    let device = Device {
        id: DeviceId::generate(),
        device_type: DeviceType::Active,
        storage_gb,
        endpoint: Endpoint{protocol: "quic".into(), address: addr},
        capabilities: Default::default(),
    };
    register_presence(&handle, vec![device.clone()], device.id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn container_put(bytes: Vec<u8>, group_size: usize) -> Result<String, String> {
    let kp = MlDsaKeyPair::generate().map_err(|e| e.to_string())?; // load existing
    let handle = register_identity(["temp","temp","temp","temp"], &kp).await.map_err(|e| e.to_string())?; // load existing
    let sh = store_data(&handle, bytes, group_size).await.map_err(|e| e.to_string())?;
    Ok(sh.to_string())
}

#[tauri::command]
pub async fn container_get(handle: String) -> Result<Vec<u8>, String> {
    let sh = handle.parse().map_err(|_| "bad handle".to_string())?;
    get_data(&sh).await.map_err(|e| e.to_string())
}
```

Wire in `src-tauri/src/main.rs`:
```rust
mod core_cmds;
fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      core_cmds::core_claim,
      core_cmds::core_advertise,
      core_cmds::container_put,
      core_cmds::container_get
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri app");
}
```

## Frontend calls (TypeScript)
```ts
import { invoke } from "@tauri-apps/api/tauri";

export async function claim(words: string[]) {
  return invoke<string>("core_claim", { words });
}
export async function advertise(addr: string, storageGb: number) {
  return invoke<void>("core_advertise", { addr, storageGb });
}
export async function put(bytes: Uint8Array, groupSize: number) {
  return invoke<string>("container_put", { bytes: Array.from(bytes), groupSize });
}
export async function get(handle: string) {
  const v = await invoke<number[]>("container_get", { handle });
  return new Uint8Array(v);
}
```

## Migration notes
- Remove any direct DHT blob paths. Only publish signed tips.
- Ensure IPv4 first in ant‑quic dialer.
- Persist ML‑DSA keys and device id in encrypted key store.
- Show four‑word IDs in invite and peer cards.
- Store only pointer + hash for external assets by default.

## Telemetry and diagnostics
- Commands: `communitas tips`, `communitas peers`, `communitas repair`.
- Log connect, fetch, verify, repair. Keep user content out of logs.
