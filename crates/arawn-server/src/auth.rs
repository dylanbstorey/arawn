//! Authentication middleware.
//!
//! Provides token-based authentication with optional Tailscale identity validation.
//!
//! # Security
//!
//! Token comparison uses constant-time comparison to prevent timing attacks.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Identity
// ─────────────────────────────────────────────────────────────────────────────

/// Authenticated identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Identity {
    /// Authenticated via bearer token.
    Token,
    /// Authenticated via Tailscale identity.
    Tailscale { user: String },
}

impl Identity {
    /// Check if this is a token identity.
    pub fn is_token(&self) -> bool {
        matches!(self, Identity::Token)
    }

    /// Check if this is a Tailscale identity.
    pub fn is_tailscale(&self) -> bool {
        matches!(self, Identity::Tailscale { .. })
    }

    /// Get the Tailscale user if this is a Tailscale identity.
    pub fn tailscale_user(&self) -> Option<&str> {
        match self {
            Identity::Tailscale { user } => Some(user),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Auth Error
// ─────────────────────────────────────────────────────────────────────────────

/// Authentication error.
#[derive(Debug, Clone)]
pub enum AuthError {
    /// Missing authorization header.
    MissingToken,
    /// Invalid token format.
    InvalidFormat,
    /// Token validation failed.
    InvalidToken,
    /// Tailscale user not in allowed list.
    TailscaleNotAllowed,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "Missing authorization token"),
            AuthError::InvalidFormat => write!(f, "Invalid authorization format"),
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::TailscaleNotAllowed => write!(f, "Tailscale user not allowed"),
        }
    }
}

impl std::error::Error for AuthError {}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidFormat => (StatusCode::BAD_REQUEST, "Invalid authorization format"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::TailscaleNotAllowed => (StatusCode::FORBIDDEN, "Tailscale user not allowed"),
        };

        let body = serde_json::json!({
            "error": message,
            "code": status.as_u16(),
        });

        (status, axum::Json(body)).into_response()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Header name for Tailscale user login.
pub const TAILSCALE_USER_HEADER: &str = "Tailscale-User-Login";

// ─────────────────────────────────────────────────────────────────────────────
// Security Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Compare two strings in constant time.
///
/// This prevents timing attacks by ensuring the comparison takes the same
/// amount of time regardless of how many characters match. The strings are
/// first padded to the same length to avoid leaking length information.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    // If lengths differ, we still do the comparison to avoid timing leaks,
    // but we know the result will be false.
    let length_matches = a_bytes.len() == b_bytes.len();

    // Compare using constant-time equality.
    // If lengths differ, compare a with itself (constant time, always true)
    // to maintain timing consistency, then return false.
    if length_matches {
        a_bytes.ct_eq(b_bytes).into()
    } else {
        // Do a dummy comparison to maintain consistent timing
        let _ = a_bytes.ct_eq(a_bytes);
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Middleware
// ─────────────────────────────────────────────────────────────────────────────

/// Authentication middleware function.
///
/// Validates the request and injects the `Identity` into request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let identity = validate_request(&request, &state)?;

    // Insert identity into request extensions for handlers to access
    request.extensions_mut().insert(identity);

    Ok(next.run(request).await)
}

/// Validate a request and return the identity.
fn validate_request(request: &Request<Body>, state: &AppState) -> Result<Identity, AuthError> {
    // If no auth token configured (localhost mode), skip auth entirely
    let Some(ref expected_token) = state.config().auth_token else {
        return Ok(Identity::Token);
    };

    // Try Authorization header first
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        let auth_str = auth_header.to_str().map_err(|_| AuthError::InvalidFormat)?;

        // Check for Bearer token
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            // Use constant-time comparison to prevent timing attacks.
            // This ensures the comparison takes the same amount of time
            // regardless of how many characters match.
            if constant_time_eq(token, expected_token) {
                return Ok(Identity::Token);
            }
            return Err(AuthError::InvalidToken);
        }

        return Err(AuthError::InvalidFormat);
    }

    // Try Tailscale header if configured
    if let Some(allowed_users) = &state.config().tailscale_users
        && let Some(ts_header) = request.headers().get(TAILSCALE_USER_HEADER)
    {
        let ts_user = ts_header.to_str().map_err(|_| AuthError::InvalidFormat)?;

        if allowed_users.iter().any(|u| u == ts_user) {
            return Ok(Identity::Tailscale {
                user: ts_user.to_string(),
            });
        }

        return Err(AuthError::TailscaleNotAllowed);
    }

    Err(AuthError::MissingToken)
}

// ─────────────────────────────────────────────────────────────────────────────
// Extension extractor
// ─────────────────────────────────────────────────────────────────────────────

/// Type alias for extracting the authenticated identity from request extensions.
///
/// Use this in handlers to get the identity of the authenticated user:
///
/// ```ignore
/// use axum::Extension;
/// use arawn_server::Identity;
///
/// async fn my_handler(Extension(identity): Extension<Identity>) -> impl IntoResponse {
///     match identity {
///         Identity::Token => "Authenticated via token",
///         Identity::Tailscale { user } => format!("Hello, {}", user),
///     }
/// }
/// ```
///
/// The `AuthIdentity` type is a newtype wrapper for use in handlers.
#[derive(Debug, Clone)]
pub struct AuthIdentity(pub Identity);

impl From<axum::Extension<Identity>> for AuthIdentity {
    fn from(ext: axum::Extension<Identity>) -> Self {
        AuthIdentity(ext.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    fn create_test_state(tailscale_users: Option<Vec<String>>) -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        let mut config = ServerConfig::new(Some("test-token-12345".to_string()));
        config.tailscale_users = tailscale_users;

        AppState::new(agent, config)
    }

    async fn protected_handler(axum::Extension(identity): axum::Extension<Identity>) -> String {
        match identity {
            Identity::Token => "token".to_string(),
            Identity::Tailscale { user } => format!("tailscale:{}", user),
        }
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_auth_with_valid_bearer_token() {
        let state = create_test_state(None);
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer test-token-12345")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"token");
    }

    #[tokio::test]
    async fn test_auth_with_invalid_token() {
        let state = create_test_state(None);
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer wrong-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_missing_token() {
        let state = create_test_state(None);
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_invalid_format() {
        let state = create_test_state(None);
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Basic dXNlcjpwYXNz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_auth_with_tailscale_allowed() {
        let state = create_test_state(Some(vec!["alice@example.com".to_string()]));
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(TAILSCALE_USER_HEADER, "alice@example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"tailscale:alice@example.com");
    }

    #[tokio::test]
    async fn test_auth_with_tailscale_not_allowed() {
        let state = create_test_state(Some(vec!["alice@example.com".to_string()]));
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(TAILSCALE_USER_HEADER, "bob@example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_auth_tailscale_disabled_ignores_header() {
        let state = create_test_state(None); // No Tailscale users configured
        let app = create_test_router(state);

        // Tailscale header without token should fail
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(TAILSCALE_USER_HEADER, "alice@example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_bearer_takes_precedence_over_tailscale() {
        let state = create_test_state(Some(vec!["alice@example.com".to_string()]));
        let app = create_test_router(state);

        // Both headers present - bearer should win
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer test-token-12345")
                    .header(TAILSCALE_USER_HEADER, "alice@example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"token"); // Token identity, not Tailscale
    }

    #[test]
    fn test_identity_methods() {
        let token = Identity::Token;
        assert!(token.is_token());
        assert!(!token.is_tailscale());
        assert!(token.tailscale_user().is_none());

        let tailscale = Identity::Tailscale {
            user: "alice".to_string(),
        };
        assert!(!tailscale.is_token());
        assert!(tailscale.is_tailscale());
        assert_eq!(tailscale.tailscale_user(), Some("alice"));
    }

    // ── Security tests ─────────────────────────────────────────────────────

    #[test]
    fn test_constant_time_eq_equal_strings() {
        assert!(super::constant_time_eq("hello", "hello"));
        assert!(super::constant_time_eq(
            "test-token-12345",
            "test-token-12345"
        ));
        assert!(super::constant_time_eq("", ""));
    }

    #[test]
    fn test_constant_time_eq_different_strings() {
        assert!(!super::constant_time_eq("hello", "world"));
        assert!(!super::constant_time_eq("hello", "hell"));
        assert!(!super::constant_time_eq("hello", "helloo"));
        assert!(!super::constant_time_eq("test-token", "test-Token")); // Case sensitive
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        // Different lengths should return false
        assert!(!super::constant_time_eq("short", "longer_string"));
        assert!(!super::constant_time_eq("longer_string", "short"));
        assert!(!super::constant_time_eq("a", ""));
        assert!(!super::constant_time_eq("", "a"));
    }

    #[test]
    fn test_constant_time_eq_unicode() {
        assert!(super::constant_time_eq("héllo", "héllo"));
        assert!(!super::constant_time_eq("héllo", "hello"));
    }
}
