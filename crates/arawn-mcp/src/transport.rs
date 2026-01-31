//! Transport layer for MCP communication.
//!
//! MCP uses a Content-Length framed protocol over stdio for local servers,
//! or HTTP POST for remote servers.

use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use crate::error::{McpError, Result};
use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

/// Configuration for HTTP transport.
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Base URL of the MCP server.
    pub url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Number of retries for failed requests.
    pub retries: u32,
    /// Optional authentication headers.
    pub headers: Vec<(String, String)>,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            timeout: Duration::from_secs(30),
            retries: 3,
            headers: Vec::new(),
        }
    }
}

impl HttpTransportConfig {
    /// Create a new HTTP transport config with the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the number of retries.
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Add a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

/// Transport for communicating with an MCP server.
pub enum McpTransport {
    /// Stdio transport - communicates with a child process via stdin/stdout.
    Stdio {
        /// The child process.
        child: Child,
        /// Buffered writer to stdin.
        stdin: BufWriter<ChildStdin>,
        /// Buffered reader from stdout.
        stdout: BufReader<ChildStdout>,
    },
    /// HTTP transport - communicates via HTTP POST requests.
    Http {
        /// HTTP client (shared for connection pooling).
        client: Arc<reqwest::blocking::Client>,
        /// Transport configuration.
        config: HttpTransportConfig,
    },
}

impl McpTransport {
    /// Create a new HTTP transport.
    ///
    /// # Arguments
    /// * `config` - HTTP transport configuration
    pub fn connect_http(config: HttpTransportConfig) -> Result<Self> {
        // Validate URL
        let _parsed = url::Url::parse(&config.url)
            .map_err(|e| McpError::transport(format!("invalid URL: {}", e)))?;

        // Build HTTP client with connection pooling
        let client_builder = reqwest::blocking::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(5)
            .tcp_keepalive(Duration::from_secs(30));

        // Configure TLS (reqwest uses native-tls or rustls based on features)
        // TLS is enabled by default for HTTPS URLs

        let client = client_builder
            .build()
            .map_err(|e| McpError::transport(format!("failed to build HTTP client: {}", e)))?;

        tracing::info!(
            url = %config.url,
            timeout_secs = config.timeout.as_secs(),
            "created HTTP transport"
        );

        Ok(Self::Http {
            client: Arc::new(client),
            config,
        })
    }

    /// Spawn a new stdio transport.
    ///
    /// # Arguments
    /// * `command` - The command to spawn (e.g., "mcp-server-sqlite")
    /// * `args` - Arguments to pass to the command
    /// * `env` - Optional environment variables to set
    pub fn spawn_stdio(
        command: &str,
        args: &[String],
        env: Option<&[(String, String)]>,
    ) -> Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()); // Let stderr pass through for debugging

        // Add environment variables if provided
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::spawn_failed(format!("failed to spawn '{}': {}", command, e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::spawn_failed("failed to capture stdin"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::spawn_failed("failed to capture stdout"))?;

        Ok(Self::Stdio {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
        })
    }

    /// Send a JSON-RPC request and wait for the response.
    pub fn send_request(&mut self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        match self {
            Self::Stdio { .. } => {
                self.send_message_stdio(&serde_json::to_value(request)?)?;
                self.receive_response_stdio()
            }
            Self::Http { client, config } => {
                // Clone what we need to avoid borrow issues
                let client = client.clone();
                let config = config.clone();
                Self::send_request_http_impl(&client, &config, request)
            }
        }
    }

    /// Send a JSON-RPC notification (no response expected).
    pub fn send_notification(&mut self, notification: &JsonRpcNotification) -> Result<()> {
        match self {
            Self::Stdio { .. } => self.send_message_stdio(&serde_json::to_value(notification)?),
            Self::Http { client, config } => {
                // For HTTP, notifications are still sent as POST but response is ignored
                let json = serde_json::to_string(notification)?;
                let mut req = client.post(&config.url).body(json);

                // Add headers
                for (key, value) in &config.headers {
                    req = req.header(key, value);
                }
                req = req.header("Content-Type", "application/json");

                // Send and ignore response
                let _ = req.send();
                Ok(())
            }
        }
    }

    /// Send a JSON-RPC request over HTTP and get the response.
    fn send_request_http_impl(
        client: &reqwest::blocking::Client,
        config: &HttpTransportConfig,
        request: &JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
        let json = serde_json::to_string(request)?;

        tracing::trace!(
            url = %config.url,
            json = %json,
            "sending MCP HTTP request"
        );

        let mut retries = config.retries;
        loop {
            let mut req = client.post(&config.url).body(json.clone());

            // Add headers
            for (key, value) in &config.headers {
                req = req.header(key, value);
            }
            req = req.header("Content-Type", "application/json");

            match req.send() {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let body = resp.text().unwrap_or_default();
                        return Err(McpError::transport(format!(
                            "HTTP error {}: {}",
                            status, body
                        )));
                    }

                    let response_text = resp.text().map_err(|e| {
                        McpError::transport(format!("failed to read response body: {}", e))
                    })?;

                    tracing::trace!(
                        json = %response_text,
                        "received MCP HTTP response"
                    );

                    let response: JsonRpcResponse = serde_json::from_str(&response_text)?;
                    return Ok(response);
                }
                Err(e) => {
                    if retries == 0 {
                        return Err(McpError::transport(format!("HTTP request failed: {}", e)));
                    }
                    retries -= 1;
                    tracing::warn!(
                        error = %e,
                        retries_remaining = retries,
                        "HTTP request failed, retrying"
                    );
                    // Small delay before retry
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Send a JSON message with Content-Length framing (stdio only).
    fn send_message_stdio(&mut self, message: &serde_json::Value) -> Result<()> {
        let json = serde_json::to_string(message)?;
        let content_length = json.len();

        match self {
            Self::Stdio { stdin, .. } => {
                // Write Content-Length header
                write!(stdin, "Content-Length: {}\r\n\r\n", content_length)?;
                // Write JSON body
                write!(stdin, "{}", json)?;
                stdin.flush()?;

                tracing::trace!(
                    content_length,
                    json = %json,
                    "sent MCP message"
                );

                Ok(())
            }
            Self::Http { .. } => Err(McpError::protocol(
                "send_message_stdio called on HTTP transport",
            )),
        }
    }

    /// Receive a JSON-RPC response with Content-Length framing (stdio only).
    fn receive_response_stdio(&mut self) -> Result<JsonRpcResponse> {
        match self {
            Self::Stdio { stdout, .. } => {
                // Read headers until we find Content-Length
                let mut content_length: Option<usize> = None;
                let mut line = String::new();

                loop {
                    line.clear();
                    let bytes_read = stdout.read_line(&mut line)?;

                    if bytes_read == 0 {
                        return Err(McpError::ConnectionClosed);
                    }

                    let trimmed = line.trim();

                    // Empty line signals end of headers
                    if trimmed.is_empty() {
                        break;
                    }

                    // Parse Content-Length header
                    if let Some(len_str) = trimmed.strip_prefix("Content-Length:") {
                        content_length = Some(len_str.trim().parse().map_err(|e| {
                            McpError::protocol(format!("invalid Content-Length: {}", e))
                        })?);
                    }
                }

                let content_length = content_length
                    .ok_or_else(|| McpError::protocol("missing Content-Length header"))?;

                // Read the JSON body
                let mut body = vec![0u8; content_length];
                stdout.read_exact(&mut body)?;

                let json_str = String::from_utf8(body)
                    .map_err(|e| McpError::protocol(format!("invalid UTF-8 in response: {}", e)))?;

                tracing::trace!(
                    content_length,
                    json = %json_str,
                    "received MCP message"
                );

                let response: JsonRpcResponse = serde_json::from_str(&json_str)?;
                Ok(response)
            }
            Self::Http { .. } => Err(McpError::protocol(
                "receive_response_stdio called on HTTP transport",
            )),
        }
    }

    /// Shutdown the transport gracefully.
    pub fn shutdown(&mut self) -> Result<()> {
        match self {
            Self::Stdio { child, .. } => {
                // Try to kill the child process
                let _ = child.kill();
                // Wait for it to exit
                let _ = child.wait();
                Ok(())
            }
            Self::Http { .. } => {
                // HTTP transport doesn't require explicit shutdown
                // Connection pooling is handled by reqwest
                Ok(())
            }
        }
    }

    /// Check if the transport is still connected.
    pub fn is_connected(&mut self) -> bool {
        match self {
            Self::Stdio { child, .. } => {
                // Check if child is still running
                matches!(child.try_wait(), Ok(None))
            }
            Self::Http { .. } => {
                // HTTP transport is always "connected" (stateless)
                true
            }
        }
    }

    /// Check if this is an HTTP transport.
    pub fn is_http(&self) -> bool {
        matches!(self, Self::Http { .. })
    }

    /// Check if this is a stdio transport.
    pub fn is_stdio(&self) -> bool {
        matches!(self, Self::Stdio { .. })
    }
}

impl Drop for McpTransport {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_nonexistent_command() {
        let result = McpTransport::spawn_stdio("nonexistent-mcp-server-12345", &[], None);
        match result {
            Ok(_) => panic!("Expected spawn to fail"),
            Err(err) => assert!(matches!(err, McpError::SpawnFailed(_))),
        }
    }

    #[test]
    fn test_spawn_with_args() {
        // Use 'cat' as a simple echo server for testing spawn
        // Note: This test just verifies spawn works, not full protocol
        let result = McpTransport::spawn_stdio("cat", &[], None);

        // cat should spawn successfully on Unix-like systems
        if cfg!(unix) {
            assert!(result.is_ok());
            let mut transport = result.unwrap();
            assert!(transport.is_stdio());
            assert!(!transport.is_http());
            transport.shutdown().unwrap();
        }
    }

    #[test]
    fn test_http_transport_config() {
        let config = HttpTransportConfig::new("http://localhost:8080/mcp")
            .with_timeout(Duration::from_secs(60))
            .with_retries(5)
            .with_header("Authorization", "Bearer token123");

        assert_eq!(config.url, "http://localhost:8080/mcp");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.retries, 5);
        assert_eq!(config.headers.len(), 1);
        assert_eq!(
            config.headers[0],
            ("Authorization".to_string(), "Bearer token123".to_string())
        );
    }

    #[test]
    fn test_http_transport_config_default() {
        let config = HttpTransportConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retries, 3);
        assert!(config.headers.is_empty());
    }

    #[test]
    fn test_http_transport_creation() {
        let config = HttpTransportConfig::new("http://localhost:8080/mcp");
        let result = McpTransport::connect_http(config);

        assert!(result.is_ok());
        let transport = result.unwrap();
        assert!(transport.is_http());
        assert!(!transport.is_stdio());
    }

    #[test]
    fn test_http_transport_invalid_url() {
        let config = HttpTransportConfig::new("not a valid url");
        let result = McpTransport::connect_http(config);

        assert!(result.is_err());
        match result {
            Err(McpError::Transport(msg)) => assert!(msg.contains("invalid URL")),
            _ => panic!("Expected Transport error"),
        }
    }

    #[test]
    fn test_http_transport_is_always_connected() {
        let config = HttpTransportConfig::new("http://localhost:8080/mcp");
        let mut transport = McpTransport::connect_http(config).unwrap();

        // HTTP transport should always report as connected
        assert!(transport.is_connected());

        // Shutdown should succeed
        assert!(transport.shutdown().is_ok());

        // Still connected (stateless)
        assert!(transport.is_connected());
    }
}
