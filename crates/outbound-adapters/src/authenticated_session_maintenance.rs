use std::sync::Arc;

use chrono::{DateTime, Utc};
use vrcx_0_application::auth::AuthenticatedSessionMaintenance;
use vrcx_0_application_core::Result;
use vrcx_0_persistence::{maintenance::avatar_auto_cleanup_run, DatabaseService};

pub struct LocalAuthenticatedSessionMaintenance {
    db: Arc<DatabaseService>,
}

impl LocalAuthenticatedSessionMaintenance {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}

impl AuthenticatedSessionMaintenance for LocalAuthenticatedSessionMaintenance {
    fn run_avatar_cleanup(&self, user_id: &str, now: DateTime<Utc>) -> Result<()> {
        avatar_auto_cleanup_run(self.db.as_ref(), user_id, now).map_err(Into::into)
    }
}
