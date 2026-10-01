pub const VRCHAT_API_DEFAULT_ENDPOINT: &str = "https://api.vrchat.cloud/api/1";
pub const VRCHAT_API_HOST: &str = "api.vrchat.cloud";
pub const VRCHAT_FILES_HOST: &str = "files.vrchat.cloud";
pub const VRCHAT_FILES_S3_HOST: &str = "files.vrchat.cloud.s3.amazonaws.com";
pub const VRCHAT_FILES_S3_HOST_PREFIX: &str = "files.vrchat.cloud.";
pub const VRCHAT_ASSETS_HOST: &str = "assets.vrchat.com";
pub const VRCHAT_LEGACY_CLOUDFRONT_HOST: &str = "d348imysud55la.cloudfront.net";
pub const VRCHAT_SITE_ORIGIN: &str = "https://vrchat.com";
pub const VRCHAT_SITE_HOST: &str = "vrchat.com";
pub const VRCHAT_CLOUD_ROOT_HOST: &str = "vrchat.cloud";
pub const VRCHAT_WEBSOCKET_DEFAULT_ENDPOINT: &str = "wss://pipeline.vrchat.cloud";

pub fn normalize_vrchat_api_endpoint(endpoint: Option<&str>) -> String {
    let endpoint = endpoint.unwrap_or("").trim().trim_end_matches('/');
    if endpoint.is_empty() {
        VRCHAT_API_DEFAULT_ENDPOINT.to_string()
    } else {
        endpoint.to_string()
    }
}

pub fn normalize_vrchat_websocket_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim().trim_end_matches('/');
    if endpoint.is_empty() {
        VRCHAT_WEBSOCKET_DEFAULT_ENDPOINT.to_string()
    } else {
        endpoint.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_vrchat_api_endpoint, normalize_vrchat_websocket_endpoint,
        VRCHAT_API_DEFAULT_ENDPOINT, VRCHAT_WEBSOCKET_DEFAULT_ENDPOINT,
    };

    #[test]
    fn api_endpoint_normalization_preserves_default_trim_and_trailing_slash_rules() {
        assert_eq!(
            normalize_vrchat_api_endpoint(None),
            VRCHAT_API_DEFAULT_ENDPOINT
        );
        assert_eq!(
            normalize_vrchat_api_endpoint(Some("  ")),
            VRCHAT_API_DEFAULT_ENDPOINT
        );
        assert_eq!(
            normalize_vrchat_api_endpoint(Some(" https://example.test/api/1/// ")),
            "https://example.test/api/1"
        );
    }

    #[test]
    fn websocket_endpoint_normalization_preserves_default_trim_and_trailing_slash_rules() {
        assert_eq!(
            normalize_vrchat_websocket_endpoint(""),
            VRCHAT_WEBSOCKET_DEFAULT_ENDPOINT
        );
        assert_eq!(
            normalize_vrchat_websocket_endpoint("  wss://example.test/// "),
            "wss://example.test"
        );
    }
}
