// SPDX-License-Identifier: MIT OR Apache-2.0

//! Presence polling hook.
//!
//! Polls `GET /presence` every 60 seconds to maintain a set of online agent IDs.
//! Best-effort: silently ignores failures (presence is non-critical).

use std::collections::HashSet;

use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use tracing::warn;

/// How often to poll presence (seconds). Matches Swift AppState behavior.
const PRESENCE_POLL_SECS: u64 = 60;

/// Hook that polls presence and returns a signal of online agent IDs.
///
/// # Usage
///
/// ```ignore
/// let online = use_presence();
/// if online().contains("agent_id_hex") { /* show green dot */ }
/// ```
pub fn use_presence() -> Signal<HashSet<String>> {
    let mut online = use_signal(HashSet::<String>::new);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();
        loop {
            match client.presence().await {
                Ok(agents) => {
                    let next_online = agents.into_iter().collect();
                    if *online.peek() != next_online {
                        online.set(next_online);
                    }
                }
                Err(e) => {
                    // Best-effort — silently ignore like Swift
                    warn!(target: "ui.presence", "presence poll failed: {e}");
                }
            }
            crate::poll_sleep(tokio::time::Duration::from_secs(PRESENCE_POLL_SECS)).await;
        }
    });

    online
}
