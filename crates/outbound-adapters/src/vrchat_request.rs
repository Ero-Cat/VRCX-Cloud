use std::sync::Arc;

use vrcx_0_application::remote::{VrchatRequestFuture, VrchatRequestPort};
use vrcx_0_application_core::vrchat_api::{VrchatApiRequest, VrchatScope};
use vrcx_0_application_core::WebClient;

#[derive(Clone)]
pub struct VrchatRequestAdapter {
    web: Arc<WebClient>,
}

impl VrchatRequestAdapter {
    pub fn new(web: Arc<WebClient>) -> Self {
        Self { web }
    }
}

impl VrchatRequestPort for VrchatRequestAdapter {
    fn send(&self, input: VrchatApiRequest, scope: VrchatScope) -> VrchatRequestFuture<'_> {
        Box::pin(async move { self.web.execute_api(input, scope).await })
    }
}
