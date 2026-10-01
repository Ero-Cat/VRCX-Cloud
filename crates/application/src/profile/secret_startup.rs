use vrcx_0_application_core::Result;

pub trait SecretStartupActions {
    fn initialize(&mut self);
    fn is_encrypting_writes(&mut self) -> bool;
    fn migrate_cookies(&mut self) -> Result<()>;
    fn migrate_saved_credentials(&mut self) -> Result<()>;
    fn migrate_sensitive_config_values(&mut self) -> Result<()>;
    fn read_cleanup_completed(&mut self) -> Result<bool>;
    fn cleanup(&mut self) -> Result<()>;
    fn record_cleanup_completed(&mut self) -> Result<()>;
}

pub fn run_secret_startup(actions: &mut dyn SecretStartupActions) {
    actions.initialize();
    let mut migrations_succeeded = true;
    if let Err(error) = actions.migrate_cookies() {
        migrations_succeeded = false;
        tracing::warn!(error = %error, "failed to migrate stored cookies to encrypted form");
    }
    if let Err(error) = actions.migrate_saved_credentials() {
        migrations_succeeded = false;
        tracing::warn!(error = %error, "failed to migrate saved credentials to encrypted form");
    }
    if let Err(error) = actions.migrate_sensitive_config_values() {
        migrations_succeeded = false;
        tracing::warn!(error = %error, "failed to migrate sensitive config values to obfuscated form");
    }
    let cleanup_completed = match actions.read_cleanup_completed() {
        Ok(completed) => completed,
        Err(error) => {
            tracing::warn!(error = %error, "failed to read secret migration cleanup state");
            false
        }
    };
    if !actions.is_encrypting_writes() || !migrations_succeeded || cleanup_completed {
        return;
    }
    if let Err(error) = actions.cleanup() {
        tracing::warn!(error = %error, "failed to remove plaintext remnants after secret migration");
        return;
    }
    if let Err(error) = actions.record_cleanup_completed() {
        tracing::warn!(error = %error, "failed to record completed secret migration cleanup state");
    }
}

#[cfg(test)]
mod tests {
    use super::{run_secret_startup, SecretStartupActions};
    use vrcx_0_application_core::{Error, Result};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Step {
        Initialize,
        MigrateCookies,
        MigrateSavedCredentials,
        MigrateSensitiveConfigValues,
        ReadCleanupCompleted,
        IsEncryptingWrites,
        Cleanup,
        RecordCleanupCompleted,
    }

    struct TestSecretStartup {
        events: Vec<Step>,
        fail_at: Option<Step>,
        encrypting_writes: bool,
        cleanup_completed: bool,
        cleanup_recorded: bool,
    }

    impl TestSecretStartup {
        fn step(&mut self, current: Step) -> Result<()> {
            self.events.push(current);
            if self.fail_at == Some(current) {
                return Err(Error::Custom(format!("{current:?} failed")));
            }
            Ok(())
        }
    }

    impl SecretStartupActions for TestSecretStartup {
        fn initialize(&mut self) {
            self.events.push(Step::Initialize);
        }

        fn is_encrypting_writes(&mut self) -> bool {
            self.events.push(Step::IsEncryptingWrites);
            self.encrypting_writes
        }

        fn migrate_cookies(&mut self) -> Result<()> {
            self.step(Step::MigrateCookies)
        }

        fn migrate_saved_credentials(&mut self) -> Result<()> {
            self.step(Step::MigrateSavedCredentials)
        }

        fn migrate_sensitive_config_values(&mut self) -> Result<()> {
            self.step(Step::MigrateSensitiveConfigValues)
        }

        fn read_cleanup_completed(&mut self) -> Result<bool> {
            self.step(Step::ReadCleanupCompleted)?;
            Ok(self.cleanup_completed)
        }

        fn cleanup(&mut self) -> Result<()> {
            self.step(Step::Cleanup)
        }

        fn record_cleanup_completed(&mut self) -> Result<()> {
            self.step(Step::RecordCleanupCompleted)?;
            self.cleanup_recorded = true;
            Ok(())
        }
    }

    fn run(
        fail_at: Option<Step>,
        encrypting_writes: bool,
        cleanup_completed: bool,
    ) -> (Vec<Step>, bool) {
        let mut startup = TestSecretStartup {
            events: Vec::new(),
            fail_at,
            encrypting_writes,
            cleanup_completed,
            cleanup_recorded: false,
        };
        run_secret_startup(&mut startup);
        (startup.events, startup.cleanup_recorded)
    }

    #[test]
    fn secret_startup_runs_all_steps_in_order() {
        let (events, cleanup_recorded) = run(None, true, false);
        assert_eq!(
            events,
            vec![
                Step::Initialize,
                Step::MigrateCookies,
                Step::MigrateSavedCredentials,
                Step::MigrateSensitiveConfigValues,
                Step::ReadCleanupCompleted,
                Step::IsEncryptingWrites,
                Step::Cleanup,
                Step::RecordCleanupCompleted,
            ]
        );
        assert!(cleanup_recorded);
    }

    #[test]
    fn secret_startup_requires_all_migrations_before_cleanup() {
        for failed_step in [
            Step::MigrateCookies,
            Step::MigrateSavedCredentials,
            Step::MigrateSensitiveConfigValues,
        ] {
            let (events, cleanup_recorded) = run(Some(failed_step), true, false);
            assert_eq!(
                events,
                vec![
                    Step::Initialize,
                    Step::MigrateCookies,
                    Step::MigrateSavedCredentials,
                    Step::MigrateSensitiveConfigValues,
                    Step::ReadCleanupCompleted,
                    Step::IsEncryptingWrites,
                ]
            );
            assert!(!cleanup_recorded);
        }
    }

    #[test]
    fn secret_startup_skips_cleanup_when_disabled_or_already_completed() {
        for (encrypting_writes, cleanup_completed) in [(false, false), (true, true)] {
            let (events, cleanup_recorded) = run(None, encrypting_writes, cleanup_completed);
            assert!(!events.contains(&Step::Cleanup));
            assert!(!cleanup_recorded);
        }
    }

    #[test]
    fn secret_startup_does_not_record_failed_cleanup() {
        let (events, cleanup_recorded) = run(Some(Step::Cleanup), true, false);
        assert!(events.contains(&Step::Cleanup));
        assert!(!events.contains(&Step::RecordCleanupCompleted));
        assert!(!cleanup_recorded);
    }

    #[test]
    fn secret_startup_retries_when_cleanup_state_cannot_be_read() {
        let (events, cleanup_recorded) = run(Some(Step::ReadCleanupCompleted), true, false);
        assert!(events.contains(&Step::Cleanup));
        assert!(cleanup_recorded);
    }

    #[test]
    fn secret_startup_keeps_cleanup_retryable_when_recording_fails() {
        let (events, cleanup_recorded) = run(Some(Step::RecordCleanupCompleted), true, false);
        assert!(events.contains(&Step::Cleanup));
        assert!(events.contains(&Step::RecordCleanupCompleted));
        assert!(!cleanup_recorded);
    }
}
