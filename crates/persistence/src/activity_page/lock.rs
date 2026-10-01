use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use crate::database::DatabaseService;

type BuildLockMap = HashMap<(PathBuf, String), Weak<Mutex<()>>>;

pub fn with_activity_page_build_lock<T>(
    db: &DatabaseService,
    user_id: &str,
    operation: impl FnOnce() -> T,
) -> T {
    static LOCKS: OnceLock<Mutex<BuildLockMap>> = OnceLock::new();
    let key = (db.db_path().to_path_buf(), user_id.to_string());
    let lock = {
        let mut locks = LOCKS
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        locks.retain(|_, lock| lock.strong_count() > 0);
        locks.get(&key).and_then(Weak::upgrade).unwrap_or_else(|| {
            let lock = Arc::new(Mutex::new(()));
            locks.insert(key, Arc::downgrade(&lock));
            lock
        })
    };
    let _guard = lock.lock().unwrap_or_else(|error| error.into_inner());
    operation()
}
