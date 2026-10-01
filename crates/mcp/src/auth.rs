use vrcx_0_local_server::{generate_token, is_allowed_authority, tokens_match};

use crate::error::McpError;
use crate::types::ClientConfigSnippets;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpAuthPolicy {
    pub port: u16,
    pub token: String,
    pub allow_lan_connections: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum McpAuthError {
    #[error("MCP requests used an invalid host header")]
    InvalidHost,
    #[error("MCP requests used an invalid origin")]
    InvalidOrigin,
    #[error("MCP requests require an Authorization bearer token")]
    MissingBearerToken,
    #[error("MCP bearer token did not match")]
    InvalidBearerToken,
}

pub fn generate_mcp_token() -> Result<String, McpError> {
    generate_token().map_err(McpError::from)
}

pub fn authorize_mcp_request(
    policy: &McpAuthPolicy,
    authorization: Option<&str>,
    host: Option<&str>,
    origin: Option<&str>,
) -> Result<(), McpAuthError> {
    if !is_allowed_authority(host, policy.port, policy.allow_lan_connections) {
        return Err(McpAuthError::InvalidHost);
    }

    if let Some(origin) = origin {
        if !is_allowed_loopback_origin(origin, policy.port) {
            return Err(McpAuthError::InvalidOrigin);
        }
    }

    let bearer = authorization
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(McpAuthError::MissingBearerToken)?;

    if tokens_match(bearer, &policy.token) {
        Ok(())
    } else {
        Err(McpAuthError::InvalidBearerToken)
    }
}

pub fn client_config_snippets(
    port: u16,
    token: &str,
    allow_lan_connections: bool,
) -> ClientConfigSnippets {
    let host = if allow_lan_connections {
        "YOUR-LAN-IP"
    } else {
        "127.0.0.1"
    };
    let url = format!("http://{host}:{port}/mcp");
    let auth_header = format!("Authorization: Bearer {token}");
    // On Windows `npx` is the `npx.cmd` shim; launching it as `command: "npx"` makes the
    // client wrap it in `cmd /C <resolved npx.cmd path>`, whose quote-stripping mangles a
    // Node install path containing spaces (e.g. `C:\Program Files\nodejs`). Spawning `cmd`
    // with the bare `npx` lets cmd resolve it via PATHEXT, so no spaced path is embedded.
    let mcp_remote_command = if cfg!(windows) {
        format!(
            "\"command\": \"cmd\",\n      \"args\": [\"/c\", \"npx\", \"-y\", \"mcp-remote\", \"{url}\", \"--header\", \"{auth_header}\"]"
        )
    } else {
        format!(
            "\"command\": \"npx\",\n      \"args\": [\"-y\", \"mcp-remote\", \"{url}\", \"--header\", \"{auth_header}\"]"
        )
    };
    ClientConfigSnippets {
        claude_code_command: format!(
            "claude mcp add --transport http vrcx-0 {url} --header \"{auth_header}\""
        ),
        mcp_remote_json: format!(
            "{{\n  \"mcpServers\": {{\n    \"vrcx-0\": {{\n      {mcp_remote_command}\n    }}\n  }}\n}}"
        ),
        generic_json: format!(
            "{{\n  \"mcpServers\": {{\n    \"vrcx-0\": {{\n      \"url\": \"{url}\",\n      \"headers\": {{\n        \"Authorization\": \"Bearer {token}\"\n      }}\n    }}\n  }}\n}}"
        ),
    }
}

fn is_allowed_loopback_origin(origin: &str, port: u16) -> bool {
    let origin = origin.to_ascii_lowercase();
    origin == format!("http://127.0.0.1:{port}") || origin == format!("http://localhost:{port}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_auth_requires_bearer_token_and_loopback_host() {
        let policy = McpAuthPolicy {
            port: 8798,
            token: "secret-token".into(),
            allow_lan_connections: false,
        };

        assert_eq!(
            authorize_mcp_request(
                &policy,
                Some("Bearer secret-token"),
                Some("127.0.0.1:8798"),
                None,
            ),
            Ok(())
        );
        assert_eq!(
            authorize_mcp_request(
                &policy,
                Some("Bearer secret-token"),
                Some("localhost:8798"),
                None,
            ),
            Ok(())
        );
        assert!(authorize_mcp_request(
            &policy,
            Some("Bearer secret-token"),
            Some("evil.test:8798"),
            None,
        )
        .is_err());
        assert!(
            authorize_mcp_request(&policy, Some("Bearer wrong"), Some("127.0.0.1:8798"), None,)
                .is_err()
        );
        assert!(authorize_mcp_request(&policy, None, Some("127.0.0.1:8798"), None).is_err());
    }

    #[test]
    fn request_auth_allows_lan_authority_only_when_enabled() {
        let mut policy = McpAuthPolicy {
            port: 8798,
            token: "secret-token".into(),
            allow_lan_connections: false,
        };

        assert_eq!(
            authorize_mcp_request(
                &policy,
                Some("Bearer secret-token"),
                Some("192.168.1.20:8798"),
                None,
            ),
            Err(McpAuthError::InvalidHost)
        );

        policy.allow_lan_connections = true;
        assert_eq!(
            authorize_mcp_request(
                &policy,
                Some("Bearer secret-token"),
                Some("192.168.1.20:8798"),
                None,
            ),
            Ok(())
        );
        assert_eq!(
            authorize_mcp_request(
                &policy,
                Some("Bearer secret-token"),
                Some("desktop.local:8798"),
                Some("http://evil.test"),
            ),
            Err(McpAuthError::InvalidOrigin)
        );
        assert_eq!(
            authorize_mcp_request(
                &policy,
                Some("Bearer secret-token"),
                Some("192.168.1.20:9876"),
                None,
            ),
            Err(McpAuthError::InvalidHost)
        );
    }

    #[test]
    fn client_help_snippets_include_real_url_and_token_warning() {
        let snippets = client_config_snippets(7654, "tok_secret", false);

        assert!(snippets
            .claude_code_command
            .contains("http://127.0.0.1:7654/mcp"));
        assert!(snippets
            .claude_code_command
            .contains("Authorization: Bearer tok_secret"));
        assert!(snippets.generic_json.contains("\"mcpServers\""));
        assert!(snippets.generic_json.contains("http://127.0.0.1:7654/mcp"));
        assert!(snippets.mcp_remote_json.contains("mcp-remote"));
        assert!(snippets
            .mcp_remote_json
            .contains("http://127.0.0.1:7654/mcp"));

        if cfg!(windows) {
            assert!(snippets.mcp_remote_json.contains("\"command\": \"cmd\""));
            assert!(snippets.mcp_remote_json.contains("\"/c\""));
            assert!(snippets.mcp_remote_json.contains("\"npx\""));
        } else {
            assert!(snippets.mcp_remote_json.contains("\"command\": \"npx\""));
        }
    }

    #[test]
    fn client_help_snippets_use_lan_placeholder_when_lan_connections_are_enabled() {
        let snippets = client_config_snippets(7654, "tok_secret", true);

        assert!(snippets
            .claude_code_command
            .contains("http://YOUR-LAN-IP:7654/mcp"));
        assert!(snippets
            .generic_json
            .contains("http://YOUR-LAN-IP:7654/mcp"));
        assert!(snippets
            .mcp_remote_json
            .contains("http://YOUR-LAN-IP:7654/mcp"));
        assert!(!snippets.claude_code_command.contains("127.0.0.1"));
        assert!(!snippets.generic_json.contains("127.0.0.1"));
        assert!(!snippets.mcp_remote_json.contains("127.0.0.1"));
    }
}
