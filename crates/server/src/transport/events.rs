//! Realtime event stream: `GET /api/events` (WebSocket).
//!
//! A broadcast sink installed on the runtime forwards the same
//! `{event, payload}` frames the desktop webview received via Tauri
//! events; clients hydrate after reconnect using the snapshot commands.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use vrcx_0_application_core::RuntimeEventSink;

/// Bridge from the runtime event bus into the web broadcast channel.
#[derive(Clone)]
pub struct WebEventSink {
    tx: tokio::sync::broadcast::Sender<(String, Value)>,
}

impl WebEventSink {
    pub fn new(tx: tokio::sync::broadcast::Sender<(String, Value)>) -> Self {
        Self { tx }
    }
}

impl RuntimeEventSink for WebEventSink {
    fn emit(&self, event: &str, payload: Value) {
        // Slow or disconnected subscribers must never block the runtime;
        // lagging clients re-hydrate from snapshot commands.
        let _ = self.tx.send((event.to_string(), payload));
    }
}

pub async fn events_endpoint(
    State(ctx): State<Arc<super::WebContext>>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if let Some(response) = super::admin_auth::gate(&ctx, &headers) {
        return response;
    }
    ws.on_upgrade(move |socket| event_loop(socket, ctx.events.subscribe()))
}

async fn event_loop(socket: WebSocket, mut rx: tokio::sync::broadcast::Receiver<(String, Value)>) {
    let (mut sender, mut receiver) = socket.split();
    loop {
        tokio::select! {
            received = rx.recv() => {
                match received {
                    Ok((event, payload)) => {
                        let frame = json!({ "event": event, "payload": payload });
                        if sender
                            .send(Message::Text(frame.to_string().into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => break,
                }
            }
        }
    }
}
