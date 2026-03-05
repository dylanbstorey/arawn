//! CLI command handlers.

pub mod agent;
pub mod ask;
pub mod auth;
pub mod chat;
pub mod config;
pub mod mcp;
pub mod memory;
pub mod notes;
pub mod output;
pub mod plugin;
pub mod repl;
pub mod secrets;
pub mod start;
pub mod status;
pub mod tui;

use console::Style;

/// Shared context for all commands.
#[derive(Debug, Clone)]
pub struct Context {
    /// Server URL to connect to.
    pub server_url: String,
    /// Output as JSON for scripting.
    pub json_output: bool,
    /// Verbose output enabled.
    pub verbose: bool,
}

/// Format an error into a user-friendly message with actionable suggestions.
///
/// Detects common failure modes (connection refused, auth failures, etc.)
/// and returns a message explaining what went wrong and how to fix it.
pub fn format_user_error(error: &anyhow::Error, server_url: &str) -> String {
    let error_chain = format!("{:#}", error);
    let lower = error_chain.to_lowercase();

    // Connection refused — server not running
    if lower.contains("connection refused")
        || lower.contains("connect error")
        || lower.contains("tcp connect error")
    {
        return format!(
            "Could not connect to server at {server_url}\n\n  \
             Is the server running? Start it with:\n    \
             arawn start"
        );
    }

    // DNS / network resolution errors
    if lower.contains("dns error")
        || lower.contains("resolve")
        || lower.contains("name or service not known")
        || lower.contains("no such host")
        || lower.contains("getaddrinfo")
    {
        return format!(
            "Could not resolve server address: {server_url}\n\n  \
             Check the server URL is correct. You can set it with:\n    \
             arawn --server <URL> ...\n    \
             or set ARAWN_SERVER_URL"
        );
    }

    // Authentication failures
    if lower.contains("authentication failed") || lower.contains("401 unauthorized") {
        return "Authentication failed.\n\n  \
                Check your API token or authenticate with:\n    \
                arawn auth login\n\n  \
                You can also set the ARAWN_API_TOKEN environment variable."
            .to_string();
    }

    // HTTP 403
    if lower.contains("403 forbidden") {
        return "Access denied by server.\n\n  \
                You may need to authenticate:\n    \
                arawn auth login"
            .to_string();
    }

    // HTTP 404
    if lower.contains("404 not found") || lower.contains("note not found") {
        return format!(
            "Resource not found on server at {server_url}\n\n  \
             The resource may have been deleted, or the ID may be incorrect."
        );
    }

    // HTTP 5xx
    if lower.contains("500 internal server error")
        || lower.contains("502 bad gateway")
        || lower.contains("503 service unavailable")
    {
        return format!(
            "Server error at {server_url}\n\n  \
             Check server logs for details. If the issue persists,\n  \
             try restarting the server:\n    \
             arawn start"
        );
    }

    // Timeout
    if lower.contains("timed out") || lower.contains("timeout") {
        return format!(
            "Request to {server_url} timed out.\n\n  \
             The server may be overloaded or unreachable.\n  \
             Check your network connection and try again."
        );
    }

    // Config file parsing errors
    if lower.contains("toml") && (lower.contains("parse") || lower.contains("deserialize")) {
        return "Configuration file has syntax errors.\n\n  \
                Check your config with:\n    \
                arawn config show\n\n  \
                Edit the config file to fix any issues."
            .to_string();
    }

    // WebSocket protocol errors
    if lower.contains("websocket") && lower.contains("handshake") {
        return format!(
            "WebSocket connection to {server_url} failed.\n\n  \
             The server may not support WebSocket connections.\n  \
             Check that the server URL is correct."
        );
    }

    // Fallback — return the original error message
    format!("{error}")
}

/// Print a CLI error with optional verbose details.
///
/// Uses the shared `output::error()` for consistent formatting,
/// then optionally shows the full error chain in verbose mode.
pub fn print_cli_error(error: &anyhow::Error, server_url: &str, verbose: bool) {
    let friendly = format_user_error(error, server_url);

    eprintln!();
    output::error(&friendly);

    if verbose {
        // Show the full error chain for debugging
        let full = format!("{:#}", error);
        if full != friendly && full != format!("{error}") {
            eprintln!();
            eprintln!(
                "{}",
                Style::new().dim().apply_to(format!("Details: {}", full))
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_error(msg: &str) -> anyhow::Error {
        anyhow::anyhow!("{}", msg)
    }

    const URL: &str = "http://localhost:8080";

    #[test]
    fn test_connection_refused() {
        let err = make_error("error sending request: connection refused");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Could not connect to server"));
        assert!(msg.contains("arawn start"));
        assert!(msg.contains(URL));
    }

    #[test]
    fn test_tcp_connect_error() {
        let err = make_error("tcp connect error: Connection refused (os error 61)");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Could not connect to server"));
    }

    #[test]
    fn test_dns_error() {
        let err = make_error("dns error: no such host is known");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Could not resolve server address"));
        assert!(msg.contains("ARAWN_SERVER_URL"));
    }

    #[test]
    fn test_auth_failed() {
        let err = make_error("Authentication failed: invalid token");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Authentication failed"));
        assert!(msg.contains("arawn auth login"));
        assert!(msg.contains("ARAWN_API_TOKEN"));
    }

    #[test]
    fn test_401() {
        let err = make_error("Server returned error: 401 Unauthorized");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Authentication failed"));
    }

    #[test]
    fn test_403() {
        let err = make_error("Server returned error: 403 Forbidden");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Access denied"));
        assert!(msg.contains("arawn auth login"));
    }

    #[test]
    fn test_404() {
        let err = make_error("Server returned error: 404 Not Found");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Resource not found"));
    }

    #[test]
    fn test_note_not_found() {
        let err = make_error("Note not found: abc123");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Resource not found"));
        assert!(msg.contains("may have been deleted"));
    }

    #[test]
    fn test_500() {
        let err = make_error("Server returned error: 500 Internal Server Error");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Server error"));
        assert!(msg.contains("Check server logs"));
    }

    #[test]
    fn test_timeout() {
        let err = make_error("operation timed out");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("timed out"));
        assert!(msg.contains("try again"));
    }

    #[test]
    fn test_toml_parse_error() {
        let err = make_error("TOML parse error: expected `=`, found newline");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("Configuration file has syntax errors"));
        assert!(msg.contains("arawn config show"));
    }

    #[test]
    fn test_websocket_handshake() {
        let err = make_error("WebSocket handshake failed: unexpected status code 404");
        let msg = format_user_error(&err, URL);
        assert!(msg.contains("WebSocket connection"));
        assert!(msg.contains("failed"));
    }

    #[test]
    fn test_unknown_error_passes_through() {
        let err = make_error("something completely unexpected happened");
        let msg = format_user_error(&err, URL);
        assert_eq!(msg, "something completely unexpected happened");
    }

    #[test]
    fn test_server_url_included_in_connection_error() {
        let err = make_error("connection refused");
        let msg = format_user_error(&err, "https://my-server.example.com:9090");
        assert!(msg.contains("https://my-server.example.com:9090"));
    }
}
