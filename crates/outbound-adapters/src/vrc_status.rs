use std::sync::Arc;

use vrcx_0_application::profile::{VrcStatusRemote, VrcStatusRemoteFuture};
use vrcx_0_application_core::{Error, Result, WebClient};
use vrcx_0_contracts::external_api::{self, ExternalApiScope};
use vrcx_0_core::json::RawJson;

pub struct VrcStatusRemoteAdapter {
    web: Arc<WebClient>,
}

impl VrcStatusRemoteAdapter {
    pub fn new(web: Arc<WebClient>) -> Self {
        Self { web }
    }

    fn fetch(&self, path: &'static str) -> VrcStatusRemoteFuture<'_> {
        Box::pin(async move {
            let response = self
                .web
                .execute_external_api(
                    external_api::vrc_status_json_get_input(path),
                    ExternalApiScope::VrcStatus,
                )
                .await?;
            parse_vrc_status_response(response.status, &response.data)
        })
    }
}

impl VrcStatusRemote for VrcStatusRemoteAdapter {
    fn status(&self) -> VrcStatusRemoteFuture<'_> {
        self.fetch("status.json")
    }

    fn summary(&self) -> VrcStatusRemoteFuture<'_> {
        self.fetch("summary.json")
    }
}

fn parse_vrc_status_response(status: i32, data: &str) -> Result<RawJson> {
    if status != 200 {
        return Err(Error::Custom(format!(
            "VRChat status request failed ({status})."
        )));
    }
    let value: serde_json::Value = serde_json::from_str(data)?;
    Ok(RawJson::from(value))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::parse_vrc_status_response;

    #[test]
    fn status_response_maps_successful_json_and_preserves_failures() {
        assert_eq!(
            parse_vrc_status_response(200, r#"{"status":{"indicator":"none"}}"#)
                .unwrap()
                .into_value(),
            json!({"status": {"indicator": "none"}})
        );
        assert_eq!(
            parse_vrc_status_response(503, "unavailable")
                .unwrap_err()
                .to_string(),
            "VRChat status request failed (503)."
        );
        assert!(parse_vrc_status_response(200, "not-json").is_err());
    }
}
