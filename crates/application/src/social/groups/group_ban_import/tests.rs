use super::*;

const GROUP_ID: &str = "grp_00000000-0000-0000-0000-000000000001";
const USER_1: &str = "usr_00000000-0000-0000-0000-000000000001";
const USER_2: &str = "usr_00000000-0000-0000-0000-000000000002";
const USER_3: &str = "usr_00000000-0000-0000-0000-000000000003";

#[derive(Default)]
struct FakeActions {
    attempts: Arc<Mutex<Vec<String>>>,
    fail_user_id: Option<String>,
    gate: Option<Arc<tokio::sync::Notify>>,
}

impl GroupBanImportActions for FakeActions {
    fn ban_user<'a>(&'a self, _group_id: &'a str, user_id: &'a str) -> GroupBanImportFuture<'a> {
        Box::pin(async move {
            if let Some(gate) = &self.gate {
                gate.notified().await;
            }
            self.attempts.lock().unwrap().push(user_id.to_string());
            if self.fail_user_id.as_deref() == Some(user_id) {
                Err(Error::Custom("ban failed".into()))
            } else {
                Ok(())
            }
        })
    }
}

fn runtime_with(actions: FakeActions) -> (GroupBanImportRuntime, TaskSupervisor, RuntimeAuthScope) {
    let tasks = TaskSupervisor::new();
    let auth_scope = RuntimeAuthScope::new();
    auth_scope.set("usr_current", "https://api.vrchat.cloud/api/1");
    let runtime = GroupBanImportRuntime::new_with_interval(
        Arc::new(actions),
        RuntimeEventBus::new(),
        tasks.clone(),
        auth_scope.clone(),
        Duration::ZERO,
    );
    (runtime, tasks, auth_scope)
}

fn wait_terminal(runtime: &GroupBanImportRuntime) -> GroupBanImportStatus {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while is_active_state(runtime.status().status) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    runtime.status()
}

#[test]
fn prepare_trims_deduplicates_and_rejects_invalid_ids() {
    let prepared = prepare_group_ban_import(GroupBanImportStartInput {
        group_id: format!(" {GROUP_ID} "),
        user_ids: vec![
            format!(" {USER_1} "),
            USER_1.into(),
            "usr_not-a-valid-id".into(),
            USER_2.into(),
        ],
    })
    .unwrap();

    assert_eq!(prepared.group_id, GROUP_ID);
    assert_eq!(prepared.user_ids, vec![USER_1, USER_2]);
}

#[test]
fn prepare_rejects_missing_group_and_empty_id_lists() {
    assert!(prepare_group_ban_import(GroupBanImportStartInput {
        group_id: String::new(),
        user_ids: vec![USER_1.into()],
    })
    .is_err());
    assert!(prepare_group_ban_import(GroupBanImportStartInput {
        group_id: GROUP_ID.into(),
        user_ids: vec!["usr_bad".into()],
    })
    .is_err());
}

#[test]
fn runs_serially_and_a_failed_item_does_not_stop_later_items() {
    let attempts = Arc::new(Mutex::new(Vec::new()));
    let (runtime, tasks, _auth_scope) = runtime_with(FakeActions {
        attempts: Arc::clone(&attempts),
        fail_user_id: Some(USER_2.into()),
        gate: None,
    });

    let running = runtime
        .start(GroupBanImportStartInput {
            group_id: GROUP_ID.into(),
            user_ids: vec![USER_1.into(), USER_2.into(), USER_3.into()],
        })
        .unwrap();
    assert_eq!(running.status, GroupBanImportState::Running);

    let terminal = wait_terminal(&runtime);
    assert_eq!(terminal.status, GroupBanImportState::Completed);
    assert_eq!(
        attempts.lock().unwrap().as_slice(),
        &[USER_1, USER_2, USER_3]
    );
    assert_eq!(terminal.succeeded, 2);
    assert_eq!(terminal.failed, 1);
    assert_eq!(terminal.items[1].state, GroupBanImportItemState::Failed);
    assert_eq!(terminal.items[1].message, "ban failed");
    assert_eq!(terminal.last_error.as_deref(), Some("ban failed"));
    tasks.stop_all();
}

#[test]
fn rejects_start_while_an_import_is_active() {
    let gate = Arc::new(tokio::sync::Notify::new());
    let (runtime, tasks, _auth_scope) = runtime_with(FakeActions {
        attempts: Arc::new(Mutex::new(Vec::new())),
        fail_user_id: None,
        gate: Some(Arc::clone(&gate)),
    });

    runtime
        .start(GroupBanImportStartInput {
            group_id: GROUP_ID.into(),
            user_ids: vec![USER_1.into()],
        })
        .unwrap();
    assert!(runtime
        .start(GroupBanImportStartInput {
            group_id: GROUP_ID.into(),
            user_ids: vec![USER_2.into()],
        })
        .unwrap_err()
        .to_string()
        .contains("already active"));

    gate.notify_one();
    assert_eq!(
        wait_terminal(&runtime).status,
        GroupBanImportState::Completed
    );
    tasks.stop_all();
}

#[test]
fn cancel_stops_before_the_next_item() {
    let attempts = Arc::new(Mutex::new(Vec::new()));
    let gate = Arc::new(tokio::sync::Notify::new());
    let (runtime, tasks, _auth_scope) = runtime_with(FakeActions {
        attempts: Arc::clone(&attempts),
        fail_user_id: None,
        gate: Some(Arc::clone(&gate)),
    });

    runtime
        .start(GroupBanImportStartInput {
            group_id: GROUP_ID.into(),
            user_ids: vec![USER_1.into(), USER_2.into()],
        })
        .unwrap();
    let cancelling = runtime.cancel();
    assert_eq!(cancelling.status, GroupBanImportState::Cancelling);
    assert!(cancelling.cancel_requested);
    gate.notify_waiters();
    gate.notify_one();

    let terminal = wait_terminal(&runtime);
    assert_eq!(terminal.status, GroupBanImportState::Cancelled);
    assert!(attempts.lock().unwrap().len() <= 1);
    tasks.stop_all();
}

#[test]
fn auth_scope_change_invalidates_an_active_run() {
    let attempts = Arc::new(Mutex::new(Vec::new()));
    let gate = Arc::new(tokio::sync::Notify::new());
    let (runtime, tasks, auth_scope) = runtime_with(FakeActions {
        attempts: Arc::clone(&attempts),
        fail_user_id: None,
        gate: Some(Arc::clone(&gate)),
    });

    runtime
        .start(GroupBanImportStartInput {
            group_id: GROUP_ID.into(),
            user_ids: vec![USER_1.into(), USER_2.into()],
        })
        .unwrap();
    auth_scope.set("usr_next", "https://api.vrchat.cloud/api/1");
    gate.notify_waiters();
    gate.notify_one();

    let terminal = wait_terminal(&runtime);
    assert_eq!(terminal.status, GroupBanImportState::Cancelled);
    assert!(attempts.lock().unwrap().len() <= 1);
    tasks.stop_all();
}

#[test]
fn start_requires_an_authenticated_session() {
    let tasks = TaskSupervisor::new();
    let runtime = GroupBanImportRuntime::new(
        Arc::new(FakeActions::default()),
        RuntimeEventBus::new(),
        tasks.clone(),
        RuntimeAuthScope::new(),
    );

    assert!(runtime
        .start(GroupBanImportStartInput {
            group_id: GROUP_ID.into(),
            user_ids: vec![USER_1.into()],
        })
        .unwrap_err()
        .to_string()
        .contains("authenticated session"));
    tasks.stop_all();
}
