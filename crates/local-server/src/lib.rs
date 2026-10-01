use std::net::{SocketAddr, SocketAddrV4};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use constant_time_eq::constant_time_eq;
use http::HeaderMap;
use socket2::{Domain, Protocol, Socket, Type as SocketType};
use tokio::net::TcpListener;

pub fn generate_token() -> Result<String, getrandom::Error> {
    let mut token_bytes = [0_u8; 32];
    getrandom::fill(&mut token_bytes)?;
    Ok(URL_SAFE_NO_PAD.encode(token_bytes))
}

pub fn tokens_match(supplied: &str, expected: &str) -> bool {
    constant_time_eq(supplied.as_bytes(), expected.as_bytes())
}

pub fn is_allowed_authority(
    authority: Option<&str>,
    port: u16,
    allow_lan_connections: bool,
) -> bool {
    if is_allowed_loopback_authority(authority, port) {
        return true;
    }
    let Some(authority) = authority else {
        return false;
    };
    allow_lan_connections && authority_has_expected_port(authority, port)
}

pub fn header_to_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

pub fn bind_listener(
    port: u16,
    allow_lan_connections: bool,
) -> Result<TcpListener, std::io::Error> {
    let socket = Socket::new(Domain::IPV4, SocketType::STREAM, Some(Protocol::TCP))?;
    #[cfg(not(windows))]
    socket.set_reuse_address(true)?;
    let address = if allow_lan_connections {
        [0, 0, 0, 0]
    } else {
        [127, 0, 0, 1]
    };
    socket.bind(&SocketAddr::V4(SocketAddrV4::new(address.into(), port)).into())?;
    socket.listen(1024)?;
    socket.set_nonblocking(true)?;
    TcpListener::from_std(socket.into())
}

fn is_allowed_loopback_authority(authority: Option<&str>, port: u16) -> bool {
    matches!(
        authority.map(|value| value.to_ascii_lowercase()),
        Some(value) if value == format!("127.0.0.1:{port}") || value == format!("localhost:{port}")
    )
}

fn authority_has_expected_port(authority: &str, port: u16) -> bool {
    if authority.contains('@') {
        return false;
    }
    authority
        .parse::<http::uri::Authority>()
        .ok()
        .and_then(|value| value.port_u16())
        == Some(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_is_high_entropy_base64url_without_padding() {
        let token = generate_token().unwrap();

        assert!(token.len() >= 43);
        assert!(token
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));
        assert!(!token.contains('='));
    }

    #[test]
    fn token_comparison_accepts_only_the_expected_token() {
        assert!(tokens_match("secret-token", "secret-token"));
        assert!(!tokens_match("wrong", "secret-token"));
    }

    #[test]
    fn authority_requires_loopback_or_an_explicit_lan_port_match() {
        assert!(is_allowed_authority(Some("127.0.0.1:8798"), 8798, false));
        assert!(is_allowed_authority(Some("LOCALHOST:8798"), 8798, false));
        assert!(!is_allowed_authority(
            Some("192.168.1.20:8798"),
            8798,
            false
        ));
        assert!(is_allowed_authority(Some("192.168.1.20:8798"), 8798, true));
        assert!(!is_allowed_authority(Some("192.168.1.20:8799"), 8798, true));
        assert!(!is_allowed_authority(
            Some("user@192.168.1.20:8798"),
            8798,
            true
        ));
        assert!(!is_allowed_authority(None, 8798, true));
    }

    #[test]
    fn header_values_must_be_valid_text() {
        let mut headers = HeaderMap::new();
        headers.insert("x-valid", "value".parse().unwrap());
        headers.insert("x-binary", http::HeaderValue::from_bytes(&[0xff]).unwrap());

        assert_eq!(header_to_str(&headers, "x-valid"), Some("value"));
        assert_eq!(header_to_str(&headers, "x-binary"), None);
        assert_eq!(header_to_str(&headers, "x-missing"), None);
    }

    #[tokio::test]
    async fn listener_uses_the_requested_loopback_binding() {
        let listener = bind_listener(0, false).unwrap();
        let address = listener.local_addr().unwrap();

        assert!(address.ip().is_loopback());
        assert_ne!(address.port(), 0);
    }
}
