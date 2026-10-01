use super::*;

#[derive(Default)]
struct RecordingActions {
    events: Vec<String>,
}

impl GameProcessMonitorActions for RecordingActions {
    fn detect(&mut self) -> GameProcessStatus {
        GameProcessStatus::default()
    }

    fn on_game_started(&mut self, steamvr_running: bool) {
        self.events.push(format!("started:{steamvr_running}"));
    }

    fn on_game_stopped(&mut self) {
        self.events.push("stopped".to_string());
    }

    fn on_steamvr_changed(&mut self, steamvr_running: bool) {
        self.events.push(format!("steamvr:{steamvr_running}"));
    }
}

#[test]
fn first_poll_running_game_starts_actions() {
    let mut actions = RecordingActions::default();

    dispatch_process_monitor_actions(
        &mut actions,
        ProcessMonitorPoll::Initial {
            previous: GameProcessStatus::default(),
            current: GameProcessStatus {
                is_game_running: true,
                is_steamvr_running: true,
            },
        },
    );

    assert_eq!(actions.events, vec!["started:true"]);
}

#[test]
fn first_poll_without_game_dispatches_no_actions() {
    let mut actions = RecordingActions::default();

    dispatch_process_monitor_actions(
        &mut actions,
        ProcessMonitorPoll::Initial {
            previous: GameProcessStatus::default(),
            current: GameProcessStatus {
                is_game_running: false,
                is_steamvr_running: true,
            },
        },
    );

    assert!(actions.events.is_empty());
}

#[test]
fn first_poll_stopped_after_previous_running_game_stops_actions() {
    let mut actions = RecordingActions::default();

    dispatch_process_monitor_actions(
        &mut actions,
        ProcessMonitorPoll::Initial {
            previous: GameProcessStatus {
                is_game_running: true,
                is_steamvr_running: true,
            },
            current: GameProcessStatus::default(),
        },
    );

    assert_eq!(actions.events, vec!["stopped"]);
}

#[test]
fn game_start_after_steamvr_reports_steamvr_running() {
    let mut actions = RecordingActions::default();

    dispatch_process_monitor_actions(
        &mut actions,
        ProcessMonitorPoll::Subsequent {
            previous: GameProcessStatus {
                is_game_running: false,
                is_steamvr_running: true,
            },
            current: GameProcessStatus {
                is_game_running: true,
                is_steamvr_running: true,
            },
        },
    );

    assert_eq!(actions.events, vec!["started:true"]);
}

#[test]
fn running_game_reacts_to_steamvr_changes() {
    let mut actions = RecordingActions::default();

    dispatch_process_monitor_actions(
        &mut actions,
        ProcessMonitorPoll::Subsequent {
            previous: GameProcessStatus {
                is_game_running: true,
                is_steamvr_running: false,
            },
            current: GameProcessStatus {
                is_game_running: true,
                is_steamvr_running: true,
            },
        },
    );

    assert_eq!(actions.events, vec!["steamvr:true"]);
}

#[test]
fn game_stop_requires_consecutive_misses() {
    let mut consecutive_misses = 0;

    for _ in 1..GAME_STOP_CONFIRMATION_POLLS {
        assert!(resolve_debounced_game_running(
            false,
            true,
            &mut consecutive_misses
        ));
    }
    assert!(!resolve_debounced_game_running(
        false,
        true,
        &mut consecutive_misses
    ));
}

#[test]
fn detected_game_resets_pending_stop() {
    let mut consecutive_misses = 0;

    assert!(resolve_debounced_game_running(
        false,
        true,
        &mut consecutive_misses
    ));
    assert!(resolve_debounced_game_running(
        true,
        true,
        &mut consecutive_misses
    ));
    assert_eq!(consecutive_misses, 0);
    assert!(resolve_debounced_game_running(
        false,
        true,
        &mut consecutive_misses
    ));
}

#[test]
fn stopped_game_does_not_delay_start_or_remain_pending() {
    let mut consecutive_misses = GAME_STOP_CONFIRMATION_POLLS - 1;

    assert!(!resolve_debounced_game_running(
        false,
        false,
        &mut consecutive_misses
    ));
    assert_eq!(consecutive_misses, 0);
    assert!(resolve_debounced_game_running(
        true,
        false,
        &mut consecutive_misses
    ));
}

struct ScriptedDetectActions {
    game_running: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}

impl GameProcessMonitorActions for ScriptedDetectActions {
    fn detect(&mut self) -> GameProcessStatus {
        GameProcessStatus {
            is_game_running: self.game_running.load(Ordering::Relaxed),
            is_steamvr_running: false,
        }
    }

    fn on_game_started(&mut self, _steamvr_running: bool) {}

    fn on_game_stopped(&mut self) {
        self.stopped.store(true, Ordering::Release);
    }
}

struct RecordingSink {
    events: Mutex<Vec<GameProcessEvent>>,
}

impl GameProcessEventSink for RecordingSink {
    fn on_game_process_event(
        &self,
        event: GameProcessEvent,
    ) -> vrcx_0_application_core::Result<()> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

fn wait_for_event(sink: &RecordingSink, predicate: impl Fn(&GameProcessEvent) -> bool) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while std::time::Instant::now() < deadline {
        if sink.events.lock().unwrap().iter().any(&predicate) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn game_exit_during_stop_window_emits_stopped_transition_after_restart() {
    let monitor = ProcessMonitor::new();
    let detected = Arc::new(AtomicBool::new(true));
    let stopped = Arc::new(AtomicBool::new(false));
    let sink = Arc::new(RecordingSink {
        events: Mutex::new(Vec::new()),
    });

    monitor.start(
        ScriptedDetectActions {
            game_running: Arc::clone(&detected),
            stopped: Arc::clone(&stopped),
        },
        LogWatcher::new(None),
        vec![Arc::clone(&sink) as Arc<dyn GameProcessEventSink>],
    );
    assert!(wait_for_event(&sink, |event| event.is_game_running));

    monitor.stop();
    assert!(!monitor.is_game_running());
    assert!(!stopped.load(Ordering::Acquire));

    detected.store(false, Ordering::Relaxed);
    sink.events.lock().unwrap().clear();
    monitor.start(
        ScriptedDetectActions {
            game_running: Arc::clone(&detected),
            stopped: Arc::clone(&stopped),
        },
        LogWatcher::new(None),
        vec![Arc::clone(&sink) as Arc<dyn GameProcessEventSink>],
    );

    assert!(wait_for_event(&sink, |_| true));
    {
        let events = sink.events.lock().unwrap();
        let first_event = events.first().expect("restart process event");
        assert!(first_event.game_changed);
        assert!(!first_event.is_game_running);
    }
    monitor.stop();
    assert!(stopped.load(Ordering::Acquire));
}

#[test]
fn stop_clears_process_state_before_a_later_restart() {
    let monitor = ProcessMonitor::new();
    monitor.shared.game_running.store(true, Ordering::Relaxed);
    monitor
        .shared
        .steamvr_running
        .store(true, Ordering::Relaxed);

    monitor.stop();

    assert!(!monitor.is_game_running());
    assert!(!monitor.is_steamvr_running());
}
