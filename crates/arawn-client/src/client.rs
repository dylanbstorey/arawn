//! Main client implementation.

use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use url::Url;

use crate::api::{
    AgentsApi, ChatApi, ConfigApi, HealthApi, McpApi, MemoryApi, NotesApi, SessionsApi, TasksApi,
    WorkstreamsApi,
};
use crate::error::{Error, ErrorResponse, Result};

/// Default timeout for requests.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for streaming requests.
const DEFAULT_STREAM_TIMEOUT: Duration = Duration::from_secs(300);

/// Arawn API client.
///
/// Provides typed access to all Arawn server endpoints.
///
/// # Example
///
/// ```no_run
/// use arawn_client::ArawnClient;
///
/// # async fn example() -> arawn_client::Result<()> {
/// let client = ArawnClient::builder()
///     .base_url("http://localhost:8080")
///     .auth_token("secret")
///     .build()?;
///
/// let sessions = client.sessions().list().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ArawnClient {
    /// Inner shared state.
    inner: Arc<ClientInner>,
}

/// Inner client state (shared across clones).
pub(crate) struct ClientInner {
    /// HTTP client.
    pub(crate) http: reqwest::Client,
    /// Base URL for API requests.
    pub(crate) base_url: Url,
    /// Request timeout.
    pub(crate) timeout: Duration,
    /// Streaming timeout.
    pub(crate) stream_timeout: Duration,
}

impl ArawnClient {
    /// Get access to the inner client state (for API implementations).
    pub(crate) fn inner(&self) -> &ClientInner {
        &self.inner
    }
}

impl ArawnClient {
    /// Create a new client builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Create a client with default settings pointing to localhost.
    pub fn localhost() -> Result<Self> {
        Self::builder().base_url("http://127.0.0.1:8080").build()
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &Url {
        &self.inner.base_url
    }

    // ─────────────────────────────────────────────────────────────────────────
    // API accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Access the sessions API.
    pub fn sessions(&self) -> SessionsApi {
        SessionsApi::new(self.clone())
    }

    /// Access the workstreams API.
    pub fn workstreams(&self) -> WorkstreamsApi {
        WorkstreamsApi::new(self.clone())
    }

    /// Access the chat API.
    pub fn chat(&self) -> ChatApi {
        ChatApi::new(self.clone())
    }

    /// Access the config API.
    pub fn config(&self) -> ConfigApi {
        ConfigApi::new(self.clone())
    }

    /// Access the agents API.
    pub fn agents(&self) -> AgentsApi {
        AgentsApi::new(self.clone())
    }

    /// Access the notes API.
    pub fn notes(&self) -> NotesApi {
        NotesApi::new(self.clone())
    }

    /// Access the memory API.
    pub fn memory(&self) -> MemoryApi {
        MemoryApi::new(self.clone())
    }

    /// Access the tasks API.
    pub fn tasks(&self) -> TasksApi {
        TasksApi::new(self.clone())
    }

    /// Access the MCP API.
    pub fn mcp(&self) -> McpApi {
        McpApi::new(self.clone())
    }

    /// Access the health API.
    pub fn health(&self) -> HealthApi {
        HealthApi::new(self.clone())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal HTTP methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Build a URL for an API path.
    pub(crate) fn url(&self, path: &str) -> Result<Url> {
        let path = path.trim_start_matches('/');
        self.inner
            .base_url
            .join(&format!("api/v1/{}", path))
            .map_err(Error::from)
    }

    /// Make a GET request.
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .get(url)
            .timeout(self.inner.timeout)
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a GET request with query parameters.
    pub(crate) async fn get_with_query<T, Q>(&self, path: &str, query: &Q) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        Q: serde::Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .get(url)
            .query(query)
            .timeout(self.inner.timeout)
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a POST request.
    pub(crate) async fn post<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .post(url)
            .json(body)
            .timeout(self.inner.timeout)
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a POST request for streaming (returns the response directly).
    pub(crate) async fn post_stream<B>(&self, path: &str, body: &B) -> Result<reqwest::Response>
    where
        B: serde::Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .post(url)
            .json(body)
            .timeout(self.inner.stream_timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.extract_error(response).await);
        }

        Ok(response)
    }

    /// Make a PATCH request.
    pub(crate) async fn patch<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .patch(url)
            .json(body)
            .timeout(self.inner.timeout)
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a PUT request.
    pub(crate) async fn put<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .put(url)
            .json(body)
            .timeout(self.inner.timeout)
            .send()
            .await?;
        self.handle_response(response).await
    }

    /// Make a DELETE request.
    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        let url = self.url(path)?;
        let response = self
            .inner
            .http
            .delete(url)
            .timeout(self.inner.timeout)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.extract_error(response).await);
        }

        Ok(())
    }

    /// Handle a response, extracting the body or error.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(self.extract_error(response).await)
        }
    }

    /// Extract an error from a failed response.
    async fn extract_error(&self, response: reqwest::Response) -> Error {
        let status = response.status().as_u16();

        // Try to parse error response
        match response.json::<ErrorResponse>().await {
            Ok(err) => {
                if status == 404 {
                    Error::NotFound(err.message)
                } else if status == 401 {
                    Error::Auth(err.message)
                } else {
                    Error::Api {
                        status,
                        code: err.code,
                        message: err.message,
                    }
                }
            }
            Err(_) => Error::Api {
                status,
                code: "unknown".to_string(),
                message: format!("HTTP {}", status),
            },
        }
    }
}

/// Builder for creating an ArawnClient.
#[derive(Debug)]
pub struct ClientBuilder {
    base_url: Option<String>,
    auth_token: Option<String>,
    timeout: Duration,
    stream_timeout: Duration,
    user_agent: Option<String>,
}

impl ClientBuilder {
    /// Create a new builder with defaults.
    pub fn new() -> Self {
        Self {
            base_url: None,
            auth_token: None,
            timeout: DEFAULT_TIMEOUT,
            stream_timeout: DEFAULT_STREAM_TIMEOUT,
            user_agent: None,
        }
    }

    /// Set the base URL for the server.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the authentication token.
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the streaming request timeout.
    pub fn stream_timeout(mut self, timeout: Duration) -> Self {
        self.stream_timeout = timeout;
        self
    }

    /// Set a custom user agent.
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<ArawnClient> {
        let base_url = self
            .base_url
            .ok_or_else(|| Error::Config("base_url is required".to_string()))?;

        // Parse and normalize base URL
        let mut base_url = Url::parse(&base_url)?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }

        // Build default headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(token) = &self.auth_token {
            let value = HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|_| Error::Config("Invalid auth token".to_string()))?;
            headers.insert(AUTHORIZATION, value);
        }

        // Build HTTP client
        let user_agent = self
            .user_agent
            .unwrap_or_else(|| format!("arawn-client/{}", env!("CARGO_PKG_VERSION")));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(user_agent)
            .build()?;

        Ok(ArawnClient {
            inner: Arc::new(ClientInner {
                http,
                base_url,
                timeout: self.timeout,
                stream_timeout: self.stream_timeout,
            }),
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_base_url() {
        let result = ClientBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_base_url() {
        let client = ClientBuilder::new()
            .base_url("http://localhost:8080")
            .build()
            .unwrap();

        assert_eq!(client.base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn test_builder_normalizes_trailing_slash() {
        let client = ClientBuilder::new()
            .base_url("http://localhost:8080/")
            .build()
            .unwrap();

        assert_eq!(client.base_url().as_str(), "http://localhost:8080/");
    }

    #[test]
    fn test_url_building() {
        let client = ClientBuilder::new()
            .base_url("http://localhost:8080")
            .build()
            .unwrap();

        let url = client.url("sessions").unwrap();
        assert_eq!(url.as_str(), "http://localhost:8080/api/v1/sessions");

        let url = client.url("/sessions").unwrap();
        assert_eq!(url.as_str(), "http://localhost:8080/api/v1/sessions");
    }
}
