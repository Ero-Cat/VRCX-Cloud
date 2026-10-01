use std::sync::Arc;

use vrcx_0_application_core::Result;

use super::{
    join_instance_launch, InstanceLaunchDeps, InstanceLaunchHttpClient, InstanceLaunchInput,
    InstanceLaunchOutcome, InstanceLaunchPipe,
};

#[derive(Clone)]
pub struct InstanceLaunchRuntime {
    api: Arc<dyn InstanceLaunchHttpClient>,
    launch_pipe: Arc<dyn InstanceLaunchPipe>,
}

impl InstanceLaunchRuntime {
    pub fn new(
        api: Arc<dyn InstanceLaunchHttpClient>,
        launch_pipe: Arc<dyn InstanceLaunchPipe>,
    ) -> Self {
        Self { api, launch_pipe }
    }

    pub async fn join(&self, input: InstanceLaunchInput) -> Result<InstanceLaunchOutcome> {
        join_instance_launch(
            &InstanceLaunchDeps {
                api: self.api.as_ref(),
                launch_pipe: self.launch_pipe.as_ref(),
            },
            input,
        )
        .await
    }
}
