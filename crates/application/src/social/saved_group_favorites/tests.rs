use super::*;
use std::sync::Mutex;

#[derive(Default)]
struct RecordingPort {
    calls: Mutex<Vec<Vec<String>>>,
    fail: std::sync::atomic::AtomicBool,
}
impl RecordingPort {
    fn record(&self, operation: &str, owner: &OwnerId, values: &[&str]) -> Result<i64> {
        let mut call = vec![operation.to_string(), owner.as_str().to_string()];
        call.extend(values.iter().map(|value| value.to_string()));
        self.calls.lock().unwrap().push(call);
        if self.fail.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(Error::Custom("storage failed".into()));
        }
        Ok(0)
    }
}
impl SavedGroupFavoritesPort for RecordingPort {
    fn snapshot(&self, owner: &OwnerId) -> Result<SavedGroupFavoritesSnapshot> {
        self.record("snapshot", owner, &[])?;
        Ok(Default::default())
    }
    fn create_collection(&self, owner: &OwnerId, id: &str, name: &str) -> Result<i64> {
        self.record("create", owner, &[id, name])
    }
    fn delete_collection(&self, owner: &OwnerId, id: &str) -> Result<i64> {
        self.record("delete", owner, &[id])
    }
    fn add_group(&self, owner: &OwnerId, id: &str, group: &str) -> Result<i64> {
        self.record("add", owner, &[id, group])
    }
    fn remove_group(&self, owner: &OwnerId, group: &str) -> Result<i64> {
        self.record("remove", owner, &[group])
    }
    fn favorites_changed(&self) {
        self.calls.lock().unwrap().push(vec!["changed".into()]);
    }
}
fn runtime() -> (
    SavedGroupFavoritesRuntime,
    Arc<RecordingPort>,
    RuntimeAuthScope,
) {
    let port = Arc::new(RecordingPort::default());
    let scope = RuntimeAuthScope::new();
    scope.set("usr_a", "https://api.vrchat.cloud/api/1");
    (
        SavedGroupFavoritesRuntime::new(port.clone(), scope.clone()),
        port,
        scope,
    )
}
const GROUP: &str = "grp_11111111-1111-4111-8111-111111111111";

#[test]
fn creation_normalizes_name_generates_id_and_invalidates_after_success() {
    let (runtime, port, _) = runtime();
    assert_eq!(
        runtime
            .create_collection(SavedGroupCollectionCreateInput {
                name: "  Group A  ".into()
            })
            .unwrap(),
        0
    );
    let calls = port.calls.lock().unwrap();
    assert_eq!(calls[0][0], "create");
    assert_eq!(calls[0][1], "usr_a");
    assert_eq!(
        uuid::Uuid::parse_str(&calls[0][2])
            .unwrap()
            .get_version_num(),
        4
    );
    assert_eq!(calls[0][3], "Group A");
    assert_eq!(calls[1], vec!["changed"]);
}

#[test]
fn validation_precedes_authentication_and_never_calls_storage() {
    let (runtime, port, scope) = runtime();
    scope.set("", "");
    assert_eq!(
        runtime
            .create_collection(SavedGroupCollectionCreateInput { name: "  ".into() })
            .unwrap_err()
            .to_string(),
        "Saved group collection name is required."
    );
    assert_eq!(
        runtime
            .add_group(SavedGroupFavoriteAddInput {
                collection_id: "c".into(),
                group_id: "invalid".into()
            })
            .unwrap_err()
            .to_string(),
        "Saved group favorite requires a canonical group ID."
    );
    assert_eq!(
        runtime.snapshot().unwrap_err().to_string(),
        "Saved group favorites require an authenticated session."
    );
    assert!(port.calls.lock().unwrap().is_empty());
}

#[test]
fn each_operation_captures_current_owner_and_keeps_original_group_input() {
    let (runtime, port, scope) = runtime();
    runtime.snapshot().unwrap();
    scope.set("usr_b", "https://api.vrchat.cloud/api/1");
    let padded = format!(" {GROUP} ");
    runtime
        .add_group(SavedGroupFavoriteAddInput {
            collection_id: " c ".into(),
            group_id: padded.clone(),
        })
        .unwrap();
    let calls = port.calls.lock().unwrap();
    assert_eq!(calls[0], vec!["snapshot", "usr_a"]);
    assert_eq!(calls[1], vec!["add", "usr_b", " c ", padded.as_str()]);
    assert_eq!(calls[2], vec!["changed"]);
}

#[test]
fn failed_mutations_propagate_error_without_invalidating() {
    let (runtime, port, _) = runtime();
    port.fail.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        runtime
            .delete_collection(SavedGroupCollectionDeleteInput {
                collection_id: "c".into()
            })
            .unwrap_err()
            .to_string(),
        "storage failed"
    );
    assert_eq!(
        runtime
            .remove_group(SavedGroupFavoriteRemoveInput {
                group_id: GROUP.into()
            })
            .unwrap_err()
            .to_string(),
        "storage failed"
    );
    assert_eq!(port.calls.lock().unwrap().len(), 2);
}

#[test]
fn successful_no_op_removals_still_invalidate_each_time() {
    let (runtime, port, _) = runtime();
    runtime
        .delete_collection(SavedGroupCollectionDeleteInput {
            collection_id: "c".into(),
        })
        .unwrap();
    runtime
        .remove_group(SavedGroupFavoriteRemoveInput {
            group_id: GROUP.into(),
        })
        .unwrap();
    let calls = port.calls.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|call| call[0].as_str())
            .collect::<Vec<_>>(),
        vec!["delete", "changed", "remove", "changed"]
    );
}
