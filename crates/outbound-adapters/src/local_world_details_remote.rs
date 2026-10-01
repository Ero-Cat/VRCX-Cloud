use vrcx_0_application::favorites::{FavoriteRemoteFuture, LocalWorldDetailsRemote};
use vrcx_0_application_core::{WebClient, WorldCache};

pub struct CachedLocalWorldDetailsRemote<'a> {
    world_cache: &'a WorldCache,
    web: &'a WebClient,
}

impl<'a> CachedLocalWorldDetailsRemote<'a> {
    pub fn new(world_cache: &'a WorldCache, web: &'a WebClient) -> Self {
        Self { world_cache, web }
    }
}

impl LocalWorldDetailsRemote for CachedLocalWorldDetailsRemote<'_> {
    fn refresh<'a>(
        &'a self,
        endpoint: &'a str,
        world_id: &'a str,
    ) -> FavoriteRemoteFuture<'a, i32> {
        Box::pin(async move {
            self.world_cache
                .get(self.web, endpoint, world_id, true, false)
                .await
                .map(|response| response.status)
        })
    }
}
