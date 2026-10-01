use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;
use vrcx_0_application_core::RuntimeAuthScopeSnapshot;
use vrcx_0_core::vrchat_endpoints::normalize_vrchat_api_endpoint;

use super::ModerationSyncRefreshOutput;
use vrcx_0_application_core::{Error, Result};

const MODERATION_SYNC_CACHE_CAPACITY: u64 = 4;
const MODERATION_SYNC_CACHE_TTL: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) struct ModerationSyncCacheKey {
    auth_generation: u64,
    current_user_id: String,
    requested_user_id: String,
    endpoint: String,
    revision: u64,
}

#[derive(Clone)]
pub struct ModerationSyncRuntime {
    cache: Cache<ModerationSyncCacheKey, Arc<ModerationSyncRefreshOutput>>,
    revision: Arc<AtomicU64>,
}

impl ModerationSyncRuntime {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(MODERATION_SYNC_CACHE_CAPACITY)
                .time_to_live(MODERATION_SYNC_CACHE_TTL)
                .build(),
            revision: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(super) fn cache_key(
        &self,
        scope: &RuntimeAuthScopeSnapshot,
        requested_user_id: &str,
        endpoint: &str,
    ) -> ModerationSyncCacheKey {
        ModerationSyncCacheKey {
            auth_generation: scope.generation,
            current_user_id: scope.current_user_id.clone(),
            requested_user_id: requested_user_id.to_string(),
            endpoint: normalize_vrchat_api_endpoint(Some(endpoint)),
            revision: self.revision.load(Ordering::SeqCst),
        }
    }

    pub(super) async fn resolve<F, Fut>(
        &self,
        key: ModerationSyncCacheKey,
        force: bool,
        load: F,
    ) -> Result<ModerationSyncRefreshOutput>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<ModerationSyncRefreshOutput>>,
    {
        if force {
            self.cache.invalidate(&key).await;
        }
        let output = self
            .cache
            .try_get_with(key.clone(), async move { load().await.map(Arc::new) })
            .await
            .map_err(|error| Error::Custom(error.to_string()))?;
        if !output.accepted {
            self.cache.invalidate(&key).await;
        }
        Ok((*output).clone())
    }

    pub fn invalidate(&self) {
        self.revision.fetch_add(1, Ordering::SeqCst);
        self.cache.invalidate_all();
    }
}

impl Default for ModerationSyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use tokio::sync::Notify;

    fn scope() -> RuntimeAuthScopeSnapshot {
        RuntimeAuthScopeSnapshot {
            current_user_id: "usr_self".into(),
            endpoint: "https://api.example.test/api/1".into(),
            generation: 7,
            active: true,
        }
    }

    fn output(remote_count: usize) -> ModerationSyncRefreshOutput {
        ModerationSyncRefreshOutput {
            accepted: true,
            user_id: "usr_self".into(),
            remote_count: crate::wire_count(remote_count),
            local_count: crate::wire_count(remote_count),
            rows: Vec::new(),
        }
    }

    #[tokio::test]
    async fn reuses_cached_refresh_until_force_or_mutation_invalidation() {
        let runtime = ModerationSyncRuntime::new();
        let calls = Arc::new(AtomicUsize::new(0));

        for (force, expected) in [(false, 1), (false, 1), (true, 2)] {
            let calls = Arc::clone(&calls);
            let key = runtime.cache_key(&scope(), "usr_self", "https://api.example.test/api/1/");
            let actual = runtime
                .resolve(key, force, move || async move {
                    let count = calls.fetch_add(1, Ordering::SeqCst) + 1;
                    Ok(output(count))
                })
                .await
                .unwrap();
            assert_eq!(actual.remote_count, expected);
        }

        runtime.invalidate();
        let calls_after_invalidation = Arc::clone(&calls);
        let actual = runtime
            .resolve(
                runtime.cache_key(&scope(), "usr_self", "https://api.example.test/api/1"),
                false,
                move || async move {
                    let count = calls_after_invalidation.fetch_add(1, Ordering::SeqCst) + 1;
                    Ok(output(count))
                },
            )
            .await
            .unwrap();

        assert_eq!(actual.remote_count, 3);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn does_not_cache_rejected_stale_scope_results() {
        let runtime = ModerationSyncRuntime::new();
        let calls = Arc::new(AtomicUsize::new(0));

        for _ in 0..2 {
            let calls = Arc::clone(&calls);
            let mut rejected = output(1);
            rejected.accepted = false;
            runtime
                .resolve(
                    runtime.cache_key(&scope(), "usr_self", "https://api.example.test/api/1"),
                    false,
                    move || async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(rejected)
                    },
                )
                .await
                .unwrap();
        }

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_refreshes_share_one_loader() {
        let runtime = ModerationSyncRuntime::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let release = Arc::new(Notify::new());
        let first_runtime = runtime.clone();
        let first_calls = Arc::clone(&calls);
        let first_release = Arc::clone(&release);
        let first = tokio::spawn(async move {
            first_runtime
                .resolve(
                    first_runtime.cache_key(&scope(), "usr_self", "https://api.example.test/api/1"),
                    false,
                    move || async move {
                        first_calls.fetch_add(1, Ordering::SeqCst);
                        first_release.notified().await;
                        Ok(output(1))
                    },
                )
                .await
        });

        while calls.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
        let second_calls = Arc::clone(&calls);
        let second_runtime = runtime.clone();
        let second = tokio::spawn(async move {
            second_runtime
                .resolve(
                    second_runtime.cache_key(
                        &scope(),
                        "usr_self",
                        "https://api.example.test/api/1",
                    ),
                    false,
                    move || async move {
                        second_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(output(2))
                    },
                )
                .await
        });
        tokio::task::yield_now().await;
        release.notify_waiters();

        assert_eq!(first.await.unwrap().unwrap().remote_count, 1);
        assert_eq!(second.await.unwrap().unwrap().remote_count, 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
