use std::path::PathBuf;
use std::sync::Arc;

use vrcx_0_application_core::{ImageCache, WebClient};
use vrcx_0_composition::RuntimeHostServerAssemblyDeps;
use vrcx_0_persistence::{storage::StorageService, DatabaseService};

use super::*;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vrcx-0-runtime-host-server-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn test_services(name: &str) -> (TestDir, ServerRuntimeServices) {
    let dir = TestDir::new(name);
    let db = Arc::new(DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap());
    let storage = StorageService::new(&dir.path.join("storage.json")).unwrap();
    let web = Arc::new(WebClient::new(
        vrcx_0_outbound_adapters::LocalWebClientAdapter::new(
            &storage,
            Arc::clone(&db),
            "wss://pipeline.vrchat.cloud".to_string(),
            env!("CARGO_PKG_VERSION"),
        )
        .unwrap(),
    ));
    let image_cache = Arc::new(ImageCache::new(Arc::new(
        vrcx_0_outbound_adapters::LocalImageCacheAdapter::new(
            dir.path.join("ImageCache"),
            Arc::clone(&web),
        )
        .unwrap(),
    )));
    let context = RuntimeHostServerAssemblyDeps::new(db, web, image_cache);
    let services =
        ServerRuntimeServices::new(crate::state::build_server_runtime_services_deps(&context))
            .unwrap();
    (dir, services)
}

#[test]
fn services_construct_with_server_side_deps() {
    // The reduced server services bundle (privacy lock, webhook overlay
    // activity fan-out, realtime image resolver slot) must construct from
    // the same assembly deps the composition root provides.
    let (_dir, _services) = test_services("construction");
}
