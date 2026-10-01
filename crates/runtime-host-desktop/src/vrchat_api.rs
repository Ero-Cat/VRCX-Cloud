use std::sync::Arc;

use vrcx_0_application::remote::{VrchatApiFuture, VrchatApiPort, VrchatApiRuntime};
use vrcx_0_application_core::vrchat_api::{self, VrchatApiRequest, VrchatScope};
use vrcx_0_application_core::{
    RemoteMutationGate, RuntimeAuthScope, RuntimeDiagnostics, RuntimeSyncEngine, WebClient,
};

pub struct VrchatApiRuntimeDeps {
    pub auth_scope: RuntimeAuthScope,
    pub remote_mutations: Arc<RemoteMutationGate>,
    pub web: Arc<WebClient>,
    pub diagnostics: RuntimeDiagnostics,
    pub sync: RuntimeSyncEngine,
}

pub fn build_vrchat_api_runtime(deps: VrchatApiRuntimeDeps) -> VrchatApiRuntime {
    VrchatApiRuntime::new(
        deps.auth_scope,
        deps.remote_mutations,
        Arc::new(DesktopVrchatApiPort {
            web: deps.web,
            diagnostics: deps.diagnostics,
            sync: deps.sync,
        }),
    )
}

struct DesktopVrchatApiPort {
    web: Arc<WebClient>,
    diagnostics: RuntimeDiagnostics,
    sync: RuntimeSyncEngine,
}

impl VrchatApiPort for DesktopVrchatApiPort {
    fn execute(
        &self,
        command: String,
        detail: String,
        input: VrchatApiRequest,
        scope: VrchatScope,
    ) -> VrchatApiFuture<'_> {
        Box::pin(async move {
            vrchat_api::execute_api_command(
                self.web.as_ref(),
                &self.diagnostics,
                &self.sync,
                (&command, detail),
                input,
                scope,
            )
            .await
        })
    }
}
