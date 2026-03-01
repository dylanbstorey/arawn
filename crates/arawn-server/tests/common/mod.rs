//! Common test utilities for integration tests.

use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use std::sync::Arc;

use arawn_agent::{Agent, ToolRegistry};
use arawn_llm::{CompletionResponse, ContentBlock, MockBackend, StopReason, Usage};
use arawn_memory::MemoryStore;
use arawn_server::{AppState, Server, ServerConfig};

/// A test server that runs in the background.
pub struct TestServer {
    /// The server's address.
    pub addr: SocketAddr,
    /// The auth token for the server.
    pub token: String,
    /// HTTP client configured for this server.
    pub client: Client,
    /// Handle to the server task.
    _handle: JoinHandle<()>,
    /// Temporary directory for test data.
    pub temp_dir: TempDir,
}

impl TestServer {
    /// Start a new test server with default configuration.
    pub async fn start() -> Result<Self> {
        Self::start_with_responses(vec!["Test response".to_string()]).await
    }

    /// Start a new test server with mock responses.
    pub async fn start_with_responses(responses: Vec<String>) -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let token = "test-token".to_string();

        // Find an available port
        let addr = find_available_port().await?;

        // Create mock backend with proper CompletionResponses
        let completion_responses: Vec<CompletionResponse> = responses
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                CompletionResponse::new(
                    format!("mock_msg_{}", i),
                    "mock-model",
                    vec![ContentBlock::Text {
                        text,
                        cache_control: None,
                    }],
                    StopReason::EndTurn,
                    Usage::new(10, 20),
                )
            })
            .collect();

        let backend = MockBackend::new(completion_responses);

        // Create agent with mock backend
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()?;

        // Create server config
        let config = ServerConfig::new(Some(token.clone()))
            .with_bind_address(addr)
            .with_rate_limiting(false)
            .with_request_logging(false);

        // Create app state with in-memory MemoryStore for notes/memory
        let memory_store =
            Arc::new(MemoryStore::open_in_memory().expect("Failed to open in-memory store"));
        let mut state = AppState::new(agent, config);
        state.services.memory_store = Some(memory_store);

        // Start server in background
        let server = Server::from_state(state);
        let handle = tokio::spawn(async move {
            let _ = server.run_on(addr).await;
        });

        // Wait for server to be ready
        let client = Client::new();
        wait_for_server(&client, addr).await?;

        Ok(Self {
            addr,
            token,
            client,
            _handle: handle,
            temp_dir,
        })
    }

    /// Get the base URL for the server.
    pub fn base_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// Get an authenticated request builder.
    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(format!("{}{}", self.base_url(), path))
            .bearer_auth(&self.token)
    }

    /// Get an authenticated POST request builder.
    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .post(format!("{}{}", self.base_url(), path))
            .bearer_auth(&self.token)
    }

    /// Get an authenticated DELETE request builder.
    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(format!("{}{}", self.base_url(), path))
            .bearer_auth(&self.token)
    }

    /// Check if server is healthy.
    pub async fn health(&self) -> Result<bool> {
        let resp = self
            .client
            .get(format!("{}/health", self.base_url()))
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
}

/// Find an available port for the test server.
async fn find_available_port() -> Result<SocketAddr> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    drop(listener);
    Ok(addr)
}

/// Wait for the server to become ready.
async fn wait_for_server(client: &Client, addr: SocketAddr) -> Result<()> {
    let url = format!("http://{}/health", addr);

    let result = timeout(Duration::from_secs(5), async {
        loop {
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => anyhow::bail!("Timeout waiting for server to start"),
    }
}
