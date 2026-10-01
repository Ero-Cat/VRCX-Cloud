use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use vrcx_0_application_core::Result;
use vrcx_0_contracts::ProfileRestoreResult;

use super::types::{
    ProfileBackupActionOutcome, ProfileBackupSettings, ProfileBackupStatus,
    ProfileRestoreRollbackCleanupOutcome, ProfileRestoreRollbackState,
    ProfileRestoreValidationOutcome,
};

pub trait ProfileBackupPort: Send + Sync {
    fn current_status(&self) -> ProfileBackupStatus;
    fn operation_gate(&self) -> ProfileOperationGate;
    fn discard_pending(&self) -> ProfileBackupActionOutcome;
    fn dismiss_error(&self) -> ProfileBackupStatus;
    fn run_manual(&self, target_path: PathBuf) -> ProfileBackupActionOutcome;
    fn retry_delivery(&self) -> ProfileBackupActionOutcome;
    fn validate_restore(&self, source: &Path) -> ProfileRestoreValidationOutcome;
    fn request_restore(&self, expected_sha256: &str) -> ProfileRestoreValidationOutcome;
    fn discard_staged_restore(&self) -> Result<()>;
    fn take_last_restore_result(&self) -> Result<Option<ProfileRestoreResult>>;
    fn cleanup_startup_artifacts(&self) -> Result<()>;
    fn restore_rollback_state(&self) -> Result<ProfileRestoreRollbackState>;
    fn clear_restore_rollback(&self) -> ProfileRestoreRollbackCleanupOutcome;
    fn settings(&self) -> ProfileBackupSettings;
    fn target_dir_requiring_grant(&self, requested: &ProfileBackupSettings) -> Option<String>;
    fn set_settings(&self, settings: ProfileBackupSettings) -> ProfileBackupSettings;
    fn start_scheduler(&self);
}

#[derive(Clone)]
pub struct ProfileBackupRuntime {
    inner: Arc<dyn ProfileBackupPort>,
}

#[derive(Clone, Default)]
pub struct ProfileOperationGate {
    flag: Arc<AtomicBool>,
}

pub struct OperationGuard {
    flag: Arc<AtomicBool>,
}

impl ProfileOperationGate {
    pub fn is_acquired(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }
}

impl OperationGuard {
    pub fn try_acquire(gate: &ProfileOperationGate) -> Option<Self> {
        gate.flag
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self {
                flag: Arc::clone(&gate.flag),
            })
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Release);
    }
}

impl ProfileBackupRuntime {
    pub fn new(port: Arc<dyn ProfileBackupPort>) -> Self {
        Self { inner: port }
    }

    pub fn current_status(&self) -> ProfileBackupStatus {
        self.inner.current_status()
    }
    pub fn operation_gate(&self) -> ProfileOperationGate {
        self.inner.operation_gate()
    }
    pub fn discard_pending(&self) -> ProfileBackupActionOutcome {
        self.inner.discard_pending()
    }
    pub fn dismiss_error(&self) -> ProfileBackupStatus {
        self.inner.dismiss_error()
    }
    pub fn run_manual(&self, target_path: impl Into<PathBuf>) -> ProfileBackupActionOutcome {
        self.inner.run_manual(target_path.into())
    }
    pub fn retry_delivery(&self) -> ProfileBackupActionOutcome {
        self.inner.retry_delivery()
    }
    pub fn validate_restore(&self, source: &Path) -> ProfileRestoreValidationOutcome {
        self.inner.validate_restore(source)
    }
    pub fn request_restore(&self, expected_sha256: &str) -> ProfileRestoreValidationOutcome {
        self.inner.request_restore(expected_sha256)
    }
    pub fn discard_staged_restore(&self) -> Result<()> {
        self.inner.discard_staged_restore()
    }
    pub fn take_last_restore_result(&self) -> Result<Option<ProfileRestoreResult>> {
        self.inner.take_last_restore_result()
    }
    pub fn cleanup_startup_artifacts(&self) -> Result<()> {
        self.inner.cleanup_startup_artifacts()
    }
    pub fn restore_rollback_state(&self) -> Result<ProfileRestoreRollbackState> {
        self.inner.restore_rollback_state()
    }
    pub fn clear_restore_rollback(&self) -> ProfileRestoreRollbackCleanupOutcome {
        self.inner.clear_restore_rollback()
    }
    pub fn settings(&self) -> ProfileBackupSettings {
        self.inner.settings()
    }
    pub fn target_dir_requiring_grant(&self, requested: &ProfileBackupSettings) -> Option<String> {
        self.inner.target_dir_requiring_grant(requested)
    }
    pub fn set_settings(&self, settings: ProfileBackupSettings) -> ProfileBackupSettings {
        self.inner.set_settings(settings)
    }
    pub fn start_scheduler(&self) {
        self.inner.start_scheduler();
    }
}
