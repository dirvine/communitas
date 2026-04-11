// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact SSE client for real-time x0xd event streams.
//!
//! Supports the daemon's `/events`, `/direct/events`, and `/presence/events`
//! endpoints. Authentication is handled with the same Bearer token used for the
//! REST API, which works for all SSE routes.

use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::config::X0xConfig;
use crate::error::{Result, X0xError};

/// One parsed server-sent-event frame from x0xd.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseFrame {
    /// Optional `event:` field from the SSE frame.
    pub event: Option<String>,
    /// Optional `id:` field from the SSE frame.
    pub id: Option<String>,
    /// Concatenated `data:` payload from the SSE frame.
    pub data: String,
}

impl SseFrame {
    /// Decode the `data:` payload as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_str(&self.data).map_err(X0xError::Json)
    }
}

/// An active SSE connection to x0xd.
pub struct X0xSseStream {
    rx: mpsc::UnboundedReceiver<SseFrame>,
}

impl X0xSseStream {
    /// Connect to the daemon's general SSE endpoint (`/events`).
    pub async fn connect() -> Result<Self> {
        match crate::config::discover() {
            Ok(cfg) => Self::connect_with_config(&cfg, "/events").await,
            Err(e) => {
                tracing::debug!(
                    "x0x config discovery failed ({e}); falling back to http://127.0.0.1:12700/events with no auth"
                );
                Self::connect_to("http://127.0.0.1:12700/events").await
            }
        }
    }

    /// Connect to the daemon's direct-message SSE endpoint (`/direct/events`).
    pub async fn connect_direct() -> Result<Self> {
        match crate::config::discover() {
            Ok(cfg) => Self::connect_with_config(&cfg, "/direct/events").await,
            Err(e) => {
                tracing::debug!(
                    "x0x config discovery failed ({e}); falling back to http://127.0.0.1:12700/direct/events with no auth"
                );
                Self::connect_to("http://127.0.0.1:12700/direct/events").await
            }
        }
    }

    /// Connect to the daemon's presence SSE endpoint (`/presence/events`).
    pub async fn connect_presence() -> Result<Self> {
        match crate::config::discover() {
            Ok(cfg) => Self::connect_with_config(&cfg, "/presence/events").await,
            Err(e) => {
                tracing::debug!(
                    "x0x config discovery failed ({e}); falling back to http://127.0.0.1:12700/presence/events with no auth"
                );
                Self::connect_to("http://127.0.0.1:12700/presence/events").await
            }
        }
    }

    /// Connect to a custom SSE URL with no authentication.
    pub async fn connect_to(url: &str) -> Result<Self> {
        Self::connect_with_client(reqwest::Client::new(), url).await
    }

    /// Connect to a custom SSE URL with an explicit Bearer token.
    pub async fn connect_with_token(address: &str, token: &str, path: &str) -> Result<Self> {
        let url = format!("http://{}{}", address.trim_end_matches('/'), path);
        let client = authenticated_client(token)?;
        Self::connect_with_client(client, &url).await
    }

    async fn connect_with_config(cfg: &X0xConfig, path: &str) -> Result<Self> {
        let url = format!("http://{}{}", cfg.address.trim_end_matches('/'), path);
        let client = authenticated_client(&cfg.token)?;
        Self::connect_with_client(client, &url).await
    }

    async fn connect_with_client(client: reqwest::Client, url: &str) -> Result<Self> {
        let resp = client
            .get(url)
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|_| X0xError::NotReachable(url.to_owned()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = match resp.text().await {
                Ok(body) => body,
                Err(err) => format!("failed to read error body: {err}"),
            };
            return Err(X0xError::Daemon(format!("HTTP {status}: {body}")));
        }

        let mut stream = resp.bytes_stream();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut buffer = String::new();
            while let Some(chunk) = stream.next().await {
                let Ok(chunk) = chunk else {
                    break;
                };
                let chunk = String::from_utf8_lossy(&chunk).replace("\r\n", "\n");
                buffer.push_str(&chunk);

                while let Some(frame) = take_next_frame(&mut buffer) {
                    if let Some(parsed) = parse_frame(&frame)
                        && tx.send(parsed).is_err()
                    {
                        return;
                    }
                }
            }
        });

        Ok(Self { rx })
    }

    /// Receive the next SSE frame. Returns `None` when the stream closes.
    pub async fn recv(&mut self) -> Option<SseFrame> {
        self.rx.recv().await
    }
}

fn authenticated_client(token: &str) -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    let value = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|err| X0xError::Config(format!("invalid api token header: {err}")))?;
    headers.insert(reqwest::header::AUTHORIZATION, value);
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(X0xError::Http)
}

fn take_next_frame(buffer: &mut String) -> Option<String> {
    let pos = buffer.find("\n\n")?;
    let frame = buffer[..pos].to_owned();
    let rest = buffer[pos + 2..].to_owned();
    *buffer = rest;
    Some(frame)
}

fn parse_frame(frame: &str) -> Option<SseFrame> {
    let mut event = None;
    let mut id = None;
    let mut data_lines = Vec::new();

    for line in frame.lines() {
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("event:") {
            event = Some(rest.trim_start().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("id:") {
            id = Some(rest.trim_start().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_owned());
        }
    }

    if event.is_none() && id.is_none() && data_lines.is_empty() {
        return None;
    }

    Some(SseFrame {
        event,
        id,
        data: data_lines.join("\n"),
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_frame, take_next_frame};

    #[test]
    fn parses_single_sse_frame() {
        let frame =
            parse_frame("event: message\ndata: {\"ok\":true}\n").expect("frame should parse");
        assert_eq!(frame.event.as_deref(), Some("message"));
        assert_eq!(frame.data, "{\"ok\":true}");
    }

    #[test]
    fn joins_multiline_data() {
        let frame = parse_frame("event: message\ndata: line-1\ndata: line-2\n")
            .expect("frame should parse");
        assert_eq!(frame.data, "line-1\nline-2");
    }

    #[test]
    fn extracts_complete_frames_from_buffer() {
        let mut buffer = "event: ping\ndata: one\n\nevent: pong\ndata: two\n\n".to_string();
        let first = take_next_frame(&mut buffer).expect("first frame");
        let second = take_next_frame(&mut buffer).expect("second frame");
        assert!(buffer.is_empty());
        assert!(first.contains("one"));
        assert!(second.contains("two"));
    }
}
