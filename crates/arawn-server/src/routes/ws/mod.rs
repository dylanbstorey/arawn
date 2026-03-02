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

    // Validate Origin header if allowed origins are configured
    if !config.ws_allowed_origins.is_empty() {
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
        // Also support wildcard matching for subdomains (e.g., "*.example.com")
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
}
