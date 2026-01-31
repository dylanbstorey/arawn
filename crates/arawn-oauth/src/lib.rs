//! OAuth 2.0 PKCE proxy for Claude MAX authentication.
//!
//! Provides a vendored OAuth proxy that enables Arawn to use Claude MAX
//! subscription credits without a separate API key. Modeled after muninn's
//! proxy implementation.
//!
//! # Components
//!
//! - [`oauth`] — PKCE flow: challenge generation, authorization URL, token exchange/refresh
//! - [`token_manager`] — Token persistence and automatic refresh
//! - [`passthrough`] — Request mangling: system prompt injection, field stripping, auth headers
//! - [`proxy`] — Axum-based localhost proxy server

pub mod error;
pub mod oauth;
pub mod passthrough;
pub mod proxy;
pub mod token_manager;

pub use error::{OAuthError, Result};
pub use oauth::{OAuthConfig, OAuthTokens, PkceChallenge};
pub use passthrough::{AuthMode, Passthrough, PassthroughConfig};
pub use proxy::{ProxyConfig, ProxyServer};
pub use token_manager::{FileTokenManager, SharedTokenManager, TokenManager};
