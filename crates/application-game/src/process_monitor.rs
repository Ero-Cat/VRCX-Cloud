use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::game_log_watcher::LogWatcher;
use vrcx_0_application_core::{GameProcessEvent, GameProcessEventSink};

const GAME_STOP_CONFIRMATION_POLLS: u8 = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct GameProcessStatus {
    pub is_game_running: bool,
    pub is_steamvr_running: bool,
}

#[derive(Clone, Copy, Debug)]
enum ProcessMonitorPoll {
    Initial {
        previous: GameProcessStatus,
        current: GameProcessStatus,
    },
    Subsequent {
        previous: GameProcessStatus,
        current: GameProcessStatus,
    },
}

pub trait GameProcessMonitorActions: Send + 'static {
    fn detect(&mut self) -> GameProcessStatus;
    fn on_game_started(&mut self, steamvr_running: bool);
    fn on_game_stopped(&mut self);
    fn on_steamvr_changed(&mut self, _steamvr_running: bool) {}
}

struct ProcessMonitorShared {
    game_running: AtomicBool,
    observed_game_running: AtomicBool,
    steamvr_running: AtomicBool,
    started: AtomicBool,
    stop_requested: AtomicBool,
    generation: AtomicU64,
}

pub struct ProcessMonitor {
    shared: Arc<ProcessMonitorShared>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(ProcessMonitorShared {
                game_running: AtomicBool::new(false),
                observed_game_running: AtomicBool::new(false),
                steamvr_running: AtomicBool::new(false),
                started: AtomicBool::new(false),
                stop_requested: AtomicBool::new(false),
                generation: AtomicU64::new(0),
            }),
            handle: Mutex::new(None),
        }
    }

    pub fn start(
        &self,
        actions: impl GameProcessMonitorActions,
        log_watcher: LogWatcher,
        game_process_sinks: Vec<Arc<dyn GameProcessEventSink>>,
    ) {
        if self
            .shared
            .started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
            && !self.shared.stop_requested.load(Ordering::Acquire)
        {
            tracing::debug!("process monitor is already active");
            return;
        }
        let generation = self.shared.generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.shared.stop_requested.store(false, Ordering::Release);
        let shared = Arc::clone(&self.shared);

        let handle = std::thread::spawn(move || {
            let mut actions = actions;
            let mut first_poll = true;
            let mut consecutive_game_misses = 0;

            while !shared.stop_requested.load(Ordering::Acquire)
                && shared.generation.load(Ordering::Acquire) == generation
            {
                let status = actions.detect();
                let prev_game = shared.observed_game_running.load(Ordering::Relaxed);
                let game_found = if first_poll {
                    status.is_game_running
                } else {
                    resolve_debounced_game_running(
                        status.is_game_running,
                        prev_game,
                        &mut consecutive_game_misses,
                    )
                };
                let steamvr_found = status.is_steamvr_running;

                shared
                    .observed_game_running
                    .store(game_found, Ordering::Relaxed);
                shared.game_running.store(game_found, Ordering::Relaxed);
                let prev_steamvr = shared
                    .steamvr_running
                    .swap(steamvr_found, Ordering::Relaxed);
                let previous = GameProcessStatus {
                    is_game_running: prev_game,
                    is_steamvr_running: prev_steamvr,
                };
                let current = GameProcessStatus {
                    is_game_running: game_found,
                    is_steamvr_running: steamvr_found,
                };
                let poll = if first_poll {
                    ProcessMonitorPoll::Initial { previous, current }
                } else {
                    ProcessMonitorPoll::Subsequent { previous, current }
                };
                let game_changed = prev_game != game_found;
                let steamvr_changed = prev_steamvr != steamvr_found;

                if first_poll || game_changed {
                    log_watcher.set_game_running(game_found);
                }

                if first_poll || game_changed || steamvr_changed {
                    for sink in &game_process_sinks {
                        if let Err(error) = sink.on_game_process_event(GameProcessEvent {
                            is_game_running: game_found,
                            is_steamvr_running: steamvr_found,
                            game_changed,
                        }) {
                            tracing::warn!("failed to handle game process event: {error}");
                        }
                    }
                }

                dispatch_process_monitor_actions(&mut actions, poll);
                if first_poll {
                    first_poll = false;
                }

                crate::sleep_interruptibly(Duration::from_secs(1), || {
                    !shared.stop_requested.load(Ordering::Acquire)
                        && shared.generation.load(Ordering::Acquire) == generation
                });
            }

            if shared.generation.load(Ordering::Acquire) == generation {
                shared.started.store(false, Ordering::Release);
            }
        });
        if let Ok(mut current) = self.handle.lock() {
            if let Some(previous) = current.take() {
                if previous.is_finished() {
                    let _ = previous.join();
                }
            }
            *current = Some(handle);
        }
    }

    pub fn stop(&self) {
        self.shared.generation.fetch_add(1, Ordering::AcqRel);
        self.shared.stop_requested.store(true, Ordering::Release);
        self.shared.started.store(false, Ordering::Release);
        if let Ok(mut handle) = self.handle.lock() {
            if let Some(handle) = handle.take() {
                let _ = handle.join();
            }
        }
        self.shared.game_running.store(false, Ordering::Release);
        self.shared.steamvr_running.store(false, Ordering::Release);
    }

    pub fn is_game_running(&self) -> bool {
        self.shared.game_running.load(Ordering::Relaxed)
    }

    pub fn is_steamvr_running(&self) -> bool {
        self.shared.steamvr_running.load(Ordering::Relaxed)
    }
}

fn resolve_debounced_game_running(
    detected_running: bool,
    committed_running: bool,
    consecutive_misses: &mut u8,
) -> bool {
    if detected_running {
        *consecutive_misses = 0;
        return true;
    }
    if !committed_running {
        *consecutive_misses = 0;
        return false;
    }

    *consecutive_misses = consecutive_misses.saturating_add(1);
    *consecutive_misses < GAME_STOP_CONFIRMATION_POLLS
}

fn dispatch_process_monitor_actions(
    actions: &mut impl GameProcessMonitorActions,
    poll: ProcessMonitorPoll,
) {
    match poll {
        ProcessMonitorPoll::Initial { previous, current } => {
            if current.is_game_running {
                actions.on_game_started(current.is_steamvr_running);
            } else if previous.is_game_running {
                actions.on_game_stopped();
            }
        }
        ProcessMonitorPoll::Subsequent { previous, current } => {
            if previous.is_game_running != current.is_game_running {
                if current.is_game_running {
                    actions.on_game_started(current.is_steamvr_running);
                } else {
                    actions.on_game_stopped();
                }
            } else if current.is_game_running
                && previous.is_steamvr_running != current.is_steamvr_running
            {
                actions.on_steamvr_changed(current.is_steamvr_running);
            }
        }
    }
}

impl Default for ProcessMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
