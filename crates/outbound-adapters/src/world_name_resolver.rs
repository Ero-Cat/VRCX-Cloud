use std::sync::Arc;

use vrcx_0_application::social::{WorldNameFuture, WorldNameResolver};
use vrcx_0_application_core::{WebClient, WorldCache};

pub struct CachedWorldNameResolver {
    cache: Arc<WorldCache>,
    web: Arc<WebClient>,
}

impl CachedWorldNameResolver {
    pub fn new(cache: Arc<WorldCache>, web: Arc<WebClient>) -> Self {
        Self { cache, web }
    }
}

impl WorldNameResolver for CachedWorldNameResolver {
    fn resolve<'a>(&'a self, endpoint: &'a str, world_id: &'a str) -> WorldNameFuture<'a> {
        Box::pin(async move { self.cache.resolve_name(&self.web, endpoint, world_id).await })
    }
}
