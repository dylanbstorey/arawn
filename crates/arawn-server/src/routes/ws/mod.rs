//! WebSocket handler for real-time bidirectional communication.
//!
//! This module provides WebSocket support for the Arawn server, enabling:
//! - Real-time chat with streaming responses
//! - Tool execution status updates
//! - Session subscription for multi-client scenarios
//!
//! ## Module Structure
//!
//! - `protocol` - Message types (ClientMessage, ServerMessage)
//! - `connection` - Connection lifecycle and state management
//! - `handlers` - Message processing logic
//!
//! ## Security
//!
//! - Origin header validation prevents CSRF attacks
//! - Message size limits prevent DoS attacks
//! - Connection rate limiting prevents connection floods

mod connection;
mod handlers;
mod protocol;

use axum::{
    extract::{ConnectInfo, State, ws::WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use crate::state::AppState;

// Re-export public types
pub use connection::ConnectionId;
pub use handlers::MessageResponse;
pub use protocol::{ClientMessage, ServerMessage};

/// GET /ws - WebSocket upgrade handler.
///
/// ## Security
///
/// - Validates Origin header against configured allowed origins
/// - Rate limits connections per IP address
/// - Authentication happens via the first message (Auth type) to support browsers
///
/// Note: If `ws_allowed_origins` is empty, all origins are allowed (development mode).
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Response {
    let config = state.config();

    // Validate Origin header if allowed origins are configured.
    // A single "*" entry means "allow all origins" (same as empty).
    let allow_all = config.ws_allowed_origins.is_empty() || config.ws_allowed_origins == ["*"];
    if !allow_all {
        match validate_origin(&headers, &config.ws_allowed_origins) {
            Ok(()) => {}
            Err(response) => return response,
        }
    }

    // Check WebSocket connection rate limit
    if config.rate_limiting
        && let Err(response) = state.check_ws_connection_rate(addr.ip()).await
    {
        return response;
    }

    // Configure WebSocket with message size limit
    let ws = ws.max_message_size(config.max_ws_message_size);

    ws.on_upgrade(move |socket| connection::handle_socket(socket, state, addr))
}

/// Validate the Origin header against allowed origins.
///
/// Returns `Ok(())` if the origin is allowed, or an error response if not.
#[allow(clippy::result_large_err)]
fn validate_origin(headers: &HeaderMap, allowed_origins: &[String]) -> Result<(), Response> {
    // Get the Origin header
    let origin = match headers.get("Origin") {
        Some(origin) => match origin.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err(
                    (StatusCode::BAD_REQUEST, "Invalid Origin header encoding").into_response()
                );
            }
        },
        None => {
            // No Origin header - this could be a same-origin request or a non-browser client.
            // For security, we require an Origin header when origins are configured.
            // Non-browser clients should set an Origin header matching an allowed origin.
            return Err((
                StatusCode::FORBIDDEN,
                "Origin header required for WebSocket connections",
            )
                .into_response());
        }
    };

    // Check if origin matches any allowed origin
    for allowed in allowed_origins {
        if origin == allowed {
            return Ok(());
        }
        // Localhost-class origins match any port (e.g., "http://localhost" matches
        // "http://localhost:3000"). This handles the common case where the allowed
        // list includes bare localhost but the client connects from a dev server port.
        if is_localhost_origin(allowed) && origin_matches_ignoring_port(origin, allowed) {
            return Ok(());
        }
        // Support wildcard matching for subdomains (e.g., "*.example.com")
        if allowed.starts_with("*.") {
            let domain = &allowed[1..]; // ".example.com"
            if origin.ends_with(domain) {
                return Ok(());
            }
        }
    }

    tracing::warn!(
        origin = %origin,
        allowed = ?allowed_origins,
        "WebSocket connection rejected: origin not allowed"
    );

    Err((StatusCode::FORBIDDEN, "Origin not allowed").into_response())
}

/// Check if an origin is a localhost-class origin (no port specified).
fn is_localhost_origin(origin: &str) -> bool {
    // Strip scheme to get host
    let host = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .unwrap_or(origin);

    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

/// Check if an origin matches an allowed origin ignoring port differences.
///
/// For example, `http://localhost:3000` matches `http://localhost`.
fn origin_matches_ignoring_port(origin: &str, allowed: &str) -> bool {
    // The origin must start with the allowed origin
    if !origin.starts_with(allowed) {
        return false;
    }
    // After the allowed prefix, there should either be nothing or a port (`:NNNN`)
    let suffix = &origin[allowed.len()..];
    suffix.is_empty()
        || (suffix.starts_with(':') && suffix[1..].chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_origin_exact_match() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://example.com".parse().unwrap());

        let allowed = vec!["https://example.com".to_string()];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    #[test]
    fn test_validate_origin_not_allowed() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://evil.com".parse().unwrap());

        let allowed = vec!["https://example.com".to_string()];
        assert!(validate_origin(&headers, &allowed).is_err());
    }

    #[test]
    fn test_validate_origin_missing_header() {
        let headers = HeaderMap::new();
        let allowed = vec!["https://example.com".to_string()];

        // Missing Origin header should be rejected
        assert!(validate_origin(&headers, &allowed).is_err());
    }

    #[test]
    fn test_validate_origin_wildcard_subdomain() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://app.example.com".parse().unwrap());

        let allowed = vec!["*.example.com".to_string()];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    #[test]
    fn test_validate_origin_wildcard_no_match() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://evil.com".parse().unwrap());

        let allowed = vec!["*.example.com".to_string()];
        assert!(validate_origin(&headers, &allowed).is_err());
    }

    #[test]
    fn test_validate_origin_multiple_allowed() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://other.com".parse().unwrap());

        let allowed = vec![
            "https://example.com".to_string(),
            "https://other.com".to_string(),
        ];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    // ─── Localhost port matching tests ────────────────────────────────────

    #[test]
    fn test_validate_origin_localhost_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "http://localhost:3000".parse().unwrap());

        let allowed = vec!["http://localhost".to_string()];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    #[test]
    fn test_validate_origin_localhost_any_port() {
        let allowed = vec!["http://localhost".to_string()];

        for port in ["3000", "8080", "5173", "4200"] {
            let mut headers = HeaderMap::new();
            headers.insert(
                "Origin",
                format!("http://localhost:{}", port).parse().unwrap(),
            );
            assert!(
                validate_origin(&headers, &allowed).is_ok(),
                "http://localhost:{} should be allowed",
                port
            );
        }
    }

    #[test]
    fn test_validate_origin_localhost_bare() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "http://localhost".parse().unwrap());

        let allowed = vec!["http://localhost".to_string()];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    #[test]
    fn test_validate_origin_127_0_0_1_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "http://127.0.0.1:8080".parse().unwrap());

        let allowed = vec!["http://127.0.0.1".to_string()];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    #[test]
    fn test_validate_origin_ipv6_localhost_with_port() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "http://[::1]:3000".parse().unwrap());

        let allowed = vec!["http://[::1]".to_string()];
        assert!(validate_origin(&headers, &allowed).is_ok());
    }

    #[test]
    fn test_validate_origin_localhost_wrong_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://localhost:3000".parse().unwrap());

        // Only http://localhost is in allowed, not https://localhost
        let allowed = vec!["http://localhost".to_string()];
        assert!(validate_origin(&headers, &allowed).is_err());
    }

    #[test]
    fn test_validate_origin_non_localhost_no_port_match() {
        let mut headers = HeaderMap::new();
        headers.insert("Origin", "https://example.com:3000".parse().unwrap());

        // Non-localhost origins should NOT get port-ignoring behavior
        let allowed = vec!["https://example.com".to_string()];
        assert!(validate_origin(&headers, &allowed).is_err());
    }

    #[test]
    fn test_validate_origin_default_localhost_variants() {
        // Simulate the default allowed origins when auth is enabled
        let allowed = vec![
            "http://localhost".to_string(),
            "http://127.0.0.1".to_string(),
            "http://[::1]".to_string(),
            "https://localhost".to_string(),
            "https://127.0.0.1".to_string(),
            "https://[::1]".to_string(),
        ];

        // All these should be allowed
        let valid_origins = [
            "http://localhost",
            "http://localhost:3000",
            "http://localhost:8080",
            "http://127.0.0.1",
            "http://127.0.0.1:5173",
            "https://localhost",
            "https://localhost:443",
        ];

        for origin in valid_origins {
            let mut headers = HeaderMap::new();
            headers.insert("Origin", origin.parse().unwrap());
            assert!(
                validate_origin(&headers, &allowed).is_ok(),
                "Origin '{}' should be allowed with default localhost variants",
                origin
            );
        }

        // These should be rejected
        let invalid_origins = [
            "http://evil.com",
            "https://attacker.com:3000",
            "http://192.168.1.100",
        ];

        for origin in invalid_origins {
            let mut headers = HeaderMap::new();
            headers.insert("Origin", origin.parse().unwrap());
            assert!(
                validate_origin(&headers, &allowed).is_err(),
                "Origin '{}' should be rejected with default localhost variants",
                origin
            );
        }
    }

    // ─── Helper function tests ────────────────────────────────────────────

    #[test]
    fn test_is_localhost_origin() {
        assert!(is_localhost_origin("http://localhost"));
        assert!(is_localhost_origin("https://localhost"));
        assert!(is_localhost_origin("http://127.0.0.1"));
        assert!(is_localhost_origin("https://127.0.0.1"));
        assert!(is_localhost_origin("http://[::1]"));
        assert!(is_localhost_origin("https://[::1]"));

        assert!(!is_localhost_origin("http://example.com"));
        assert!(!is_localhost_origin("http://localhost:3000")); // has port, not bare
        assert!(!is_localhost_origin("https://evil.com"));
    }

    #[test]
    fn test_origin_matches_ignoring_port() {
        assert!(origin_matches_ignoring_port(
            "http://localhost:3000",
            "http://localhost"
        ));
        assert!(origin_matches_ignoring_port(
            "http://localhost",
            "http://localhost"
        ));
        assert!(origin_matches_ignoring_port(
            "http://127.0.0.1:8080",
            "http://127.0.0.1"
        ));

        // Invalid port suffix
        assert!(!origin_matches_ignoring_port(
            "http://localhost:abc",
            "http://localhost"
        ));
        // Not a prefix match
        assert!(!origin_matches_ignoring_port(
            "http://localhostevil.com",
            "http://localhost"
        ));
        // Path after origin (shouldn't happen but be safe)
        assert!(!origin_matches_ignoring_port(
            "http://localhost/evil",
            "http://localhost"
        ));
    }
}
