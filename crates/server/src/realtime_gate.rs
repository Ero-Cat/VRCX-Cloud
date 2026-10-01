//! Realtime session gating: when the user's desktop VRCX-0 is actively
//! syncing (i.e. it is the device currently talking to VRChat), the
//! server holds its own realtime websocket off instead of doubling the
//! session. The moment the desktop goes quiet, the server takes over
//! again. Implemented as a transparent wrapper around the realtime
//! transport, so the orchestrator's normal retry semantics drive the
//! reconnects.

use std::sync::Arc;

use tokio::sync::watch;
use vrcx_0_application_realtime::{
    RealtimeMessageSink, RealtimeSessionContext, RealtimeTransport, RealtimeTransportFuture,
    RealtimeTransportTermination,
};

/// Shared pause flag; `watch` gives both directions change notification.
#[derive(Clone)]
pub struct PauseGate {
    tx: Arc<watch::Sender<bool>>,
    rx: watch::Receiver<bool>,
}

impl PauseGate {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(false);
        Self {
            tx: Arc::new(tx),
            rx,
        }
    }

    pub fn paused(&self) -> bool {
        *self.rx.borrow()
    }

    /// Flip the pause state; wakes waiters in either direction.
    pub fn set_paused(&self, paused: bool) -> bool {
        self.tx.send_if_modified(|current| {
            if *current != paused {
                *current = paused;
                true
            } else {
                false
            }
        })
    }

    async fn wait_until(
        &self,
        paused: bool,
        mut cancel: watch::Receiver<u64>,
        generation: u64,
    ) -> WaitOutcome {
        let mut rx = self.rx.clone();
        loop {
            if *rx.borrow() != paused {
                return WaitOutcome::State;
            }
            if is_cancelled(&cancel, generation) {
                return WaitOutcome::Cancelled;
            }
            tokio::select! {
                changed = rx.changed() => {
                    if changed.is_err() {
                        return WaitOutcome::State;
                    }
                }
                _ = cancel.changed() => {
                    if is_cancelled(&cancel, generation) {
                        return WaitOutcome::Cancelled;
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
                    // Periodic re-check safety net.
                }
            }
        }
    }
}

enum WaitOutcome {
    State,
    Cancelled,
}

fn is_cancelled(cancel_rx: &watch::Receiver<u64>, generation: u64) -> bool {
    *cancel_rx.borrow() != generation
}

const PAUSE_REASON: &str = "held: a desktop device is actively syncing";

/// Wraps the VRChat realtime transport with the pause gate:
/// - paused before connect: hold without any websocket or API traffic;
/// - pause activates mid-connection: drop the socket and report an
///   unexpected exit so the supervisor retries (and the retry then holds);
/// - resumed: connect (or keep) the real transport.
pub struct GatedRealtimeTransport {
    inner: Arc<dyn RealtimeTransport>,
    gate: PauseGate,
}

impl GatedRealtimeTransport {
    pub fn new(inner: Arc<dyn RealtimeTransport>, gate: PauseGate) -> Self {
        Self { inner, gate }
    }
}

impl RealtimeTransport for GatedRealtimeTransport {
    fn run(
        &self,
        message_sink: Arc<dyn RealtimeMessageSink>,
        client_run_id: u64,
        generation: u64,
        session_generation: u64,
        session: RealtimeSessionContext,
        cancel_rx: watch::Receiver<u64>,
    ) -> RealtimeTransportFuture {
        let inner = Arc::clone(&self.inner);
        let gate = self.gate.clone();
        Box::pin(async move {
            // Hold while the desktop device owns data collection.
            if gate.paused() {
                tracing::info!(
                    "realtime transport held: desktop device is active, server websocket stays off"
                );
                match gate.wait_until(false, cancel_rx.clone(), generation).await {
                    WaitOutcome::Cancelled => {
                        return RealtimeTransportTermination::Stopped;
                    }
                    WaitOutcome::State => {}
                }
            }

            let mut inner_cancel = cancel_rx.clone();
            let mut pause_watch = gate.rx.clone();
            let mut inner_future = inner.run(
                message_sink,
                client_run_id,
                generation,
                session_generation,
                session,
                cancel_rx,
            );
            tokio::select! {
                termination = &mut inner_future => termination,
                _ = pause_watch.wait_for(|paused| *paused) => {
                    // Desktop took over: close the websocket (dropping the
                    // inner future closes the stream) and exit so the
                    // supervisor retry loop holds the next attempt.
                    RealtimeTransportTermination::UnexpectedExit {
                        reason: PAUSE_REASON.to_string(),
                        connected_secs: None,
                    }
                }
                _ = inner_cancel.wait_for(|_| gate.paused()) => {
                    RealtimeTransportTermination::UnexpectedExit {
                        reason: PAUSE_REASON.to_string(),
                        connected_secs: None,
                    }
                }
            }
        })
    }
}
