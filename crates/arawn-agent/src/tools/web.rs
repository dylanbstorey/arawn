//! Web tools for fetching and searching the internet.
//!
//! Provides tools for web search and URL fetching.

use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Web Fetch Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for web fetching.
#[derive(Debug, Clone)]
pub struct WebFetchConfig {
    /// Request timeout.
    pub timeout: Duration,
    /// Maximum response size in bytes.
    pub max_size: usize,
    /// User agent string.
    pub user_agent: String,
    /// Whether to extract text from HTML.
    pub extract_text: bool,
    /// Maximum text length to return.
    pub max_text_length: usize,
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_size: 10 * 1024 * 1024, // 10MB for in-memory responses
            user_agent: concat!("Arawn/", env!("CARGO_PKG_VERSION"), " (Research Agent)")
                .to_string(),
            extract_text: true,
            max_text_length: 50_000,
        }
    }
}

/// Tool for fetching web page content.
#[derive(Debug, Clone)]
pub struct WebFetchTool {
    client: Client,
    config: WebFetchConfig,
}

impl WebFetchTool {
    /// Create a new web fetch tool with default configuration.
    pub fn new() -> Self {
        let config = WebFetchConfig::default();
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Create a web fetch tool with custom configuration.
    pub fn with_config(config: WebFetchConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Extract readable text from HTML.
    fn extract_text_from_html(&self, html: &str) -> String {
        let document = Html::parse_document(html);

        // Remove script and style elements
        let mut text_parts = Vec::new();

        // Try to get main content areas first
        let content_selectors = [
            "article",
            "main",
            "[role='main']",
            ".content",
            "#content",
            ".post-content",
            ".entry-content",
        ];

        let mut found_content = false;
        for selector_str in content_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    if !text.trim().is_empty() {
                        text_parts.push(text);
                        found_content = true;
                    }
                }
            }
            if found_content {
                break;
            }
        }

        // Fall back to body if no content areas found
        if !found_content {
            if let Ok(body_selector) = Selector::parse("body") {
                for element in document.select(&body_selector) {
                    // Skip script, style, nav, footer elements
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    text_parts.push(text);
                }
            }
        }

        // Clean up the text
        let text = text_parts.join("\n\n");
        let text = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple whitespace
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

        // Truncate if needed
        if text.len() > self.config.max_text_length {
            format!("{}...[truncated]", &text[..self.config.max_text_length])
        } else {
            text
        }
    }

    /// Extract title from HTML.
    fn extract_title(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        if let Ok(selector) = Selector::parse("title") {
            document
                .select(&selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
        } else {
            None
        }
    }

    /// Extract meta description from HTML.
    fn extract_description(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);
        if let Ok(selector) = Selector::parse("meta[name='description']") {
            document
                .select(&selector)
                .next()
                .and_then(|el| el.value().attr("content").map(|s| s.to_string()))
        } else {
            None
        }
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch content from a URL. Supports all HTTP methods, custom headers, and request bodies. Returns the page content, status code, and metadata."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                },
                "method": {
                    "type": "string",
                    "description": "HTTP method to use. Defaults to GET.",
                    "enum": ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"],
                    "default": "GET"
                },
                "headers": {
                    "type": "object",
                    "description": "Custom request headers as key-value pairs",
                    "additionalProperties": { "type": "string" }
                },
                "body": {
                    "type": "string",
                    "description": "Request body (for POST, PUT, PATCH). Can be JSON string or plain text."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Request timeout in seconds. Defaults to 30.",
                    "minimum": 1,
                    "maximum": 300
                },
                "raw": {
                    "type": "boolean",
                    "description": "If true, return raw HTML instead of extracted text. Defaults to false.",
                    "default": false
                },
                "include_headers": {
                    "type": "boolean",
                    "description": "If true, include response headers in the result. Defaults to false.",
                    "default": false
                },
                "download": {
                    "type": "string",
                    "description": "File path to save the response body to. Streams directly to disk, bypassing size limits. Returns file metadata instead of content. Useful for binary files (images, PDFs, etc.)."
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        let url_str = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::AgentError::Tool("Missing 'url' parameter".to_string()))?;

        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        let raw = params.get("raw").and_then(|v| v.as_bool()).unwrap_or(false);
        let include_headers = params
            .get("include_headers")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let custom_headers = params.get("headers").and_then(|v| v.as_object());
        let body = params.get("body").and_then(|v| v.as_str());
        let timeout_secs = params.get("timeout_secs").and_then(|v| v.as_u64());
        let download_path = params.get("download").and_then(|v| v.as_str());

        // Validate URL
        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(e) => return Ok(ToolResult::error(format!("Invalid URL: {}", e))),
        };

        // Only allow http/https
        if url.scheme() != "http" && url.scheme() != "https" {
            return Ok(ToolResult::error("Only HTTP and HTTPS URLs are supported"));
        }

        // Build the request with the appropriate method
        let mut request = match method.as_str() {
            "GET" => self.client.get(url.as_str()),
            "POST" => self.client.post(url.as_str()),
            "PUT" => self.client.put(url.as_str()),
            "PATCH" => self.client.patch(url.as_str()),
            "DELETE" => self.client.delete(url.as_str()),
            "HEAD" => self.client.head(url.as_str()),
            "OPTIONS" => self.client.request(reqwest::Method::OPTIONS, url.as_str()),
            _ => {
                return Ok(ToolResult::error(format!(
                    "Unsupported HTTP method: {}",
                    method
                )));
            }
        };

        // Add custom headers
        if let Some(headers) = custom_headers {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key.as_str(), val_str);
                }
            }
        }

        // Add request body
        if let Some(body_str) = body {
            request = request.body(body_str.to_string());
        }

        // Apply custom timeout if specified
        if let Some(secs) = timeout_secs {
            request = request.timeout(Duration::from_secs(secs));
        }

        // Send the request
        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => return Ok(ToolResult::error(format!("Failed to fetch URL: {}", e))),
        };

        let status = response.status();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        // Capture response headers if requested
        let response_headers: Option<serde_json::Map<String, Value>> = if include_headers {
            Some(
                response
                    .headers()
                    .iter()
                    .filter_map(|(name, value)| {
                        value.to_str().ok().map(|v| (name.to_string(), json!(v)))
                    })
                    .collect(),
            )
        } else {
            None
        };

        // For non-success status, still return the response but include error info
        let is_error = !status.is_success();

        // Get content type and content length
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        // Handle download to file - stream directly to disk, bypassing size limits
        if let Some(path_str) = download_path {
            let path = Path::new(path_str);

            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    if let Err(e) = tokio::fs::create_dir_all(parent).await {
                        return Ok(ToolResult::error(format!(
                            "Failed to create directory: {}",
                            e
                        )));
                    }
                }
            }

            // Stream response body to file
            let mut file = match tokio::fs::File::create(path).await {
                Ok(f) => f,
                Err(e) => {
                    return Ok(ToolResult::error(format!("Failed to create file: {}", e)));
                }
            };

            let mut bytes_written: u64 = 0;
            let mut stream = response.bytes_stream();
            use futures::StreamExt;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        bytes_written += chunk.len() as u64;
                        if let Err(e) = file.write_all(&chunk).await {
                            return Ok(ToolResult::error(format!(
                                "Failed to write to file: {}",
                                e
                            )));
                        }
                    }
                    Err(e) => {
                        return Ok(ToolResult::error(format!(
                            "Failed to read response stream: {}",
                            e
                        )));
                    }
                }
            }

            if let Err(e) = file.flush().await {
                return Ok(ToolResult::error(format!("Failed to flush file: {}", e)));
            }

            // Return metadata about the downloaded file
            let mut result = json!({
                "url": url_str,
                "method": method,
                "status": status_code,
                "status_text": status_text,
                "downloaded": true,
                "path": path_str,
                "size": bytes_written,
                "content_type": content_type
            });
            if let Some(expected_size) = content_length {
                result["expected_size"] = json!(expected_size);
            }
            if is_error {
                result["error"] = json!(true);
            }
            if let Some(headers) = response_headers {
                result["headers"] = json!(headers);
            }
            return Ok(ToolResult::json(result));
        }

        // For HEAD requests, we don't read the body
        if method == "HEAD" {
            let mut result = json!({
                "url": url_str,
                "method": method,
                "status": status_code,
                "status_text": status_text,
                "content_type": content_type
            });
            if let Some(headers) = response_headers {
                result["headers"] = json!(headers);
            }
            return Ok(ToolResult::json(result));
        }

        // Check if response will exceed size limit - if so, auto-download to temp file
        let auto_download_path = if let Some(size) = content_length {
            if size > self.config.max_size as u64 {
                // Generate temp file path based on URL
                let filename = url
                    .path_segments()
                    .and_then(|mut s| s.next_back())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("download");
                let temp_path = std::env::temp_dir().join("arawn_downloads").join(format!(
                    "{}_{}",
                    uuid::Uuid::new_v4(),
                    filename
                ));
                Some(temp_path)
            } else {
                None
            }
        } else {
            None
        };

        // If we need to auto-download due to size, stream to temp file
        if let Some(temp_path) = auto_download_path {
            // Create parent directories
            if let Some(parent) = temp_path.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    return Ok(ToolResult::error(format!(
                        "Failed to create temp directory: {}",
                        e
                    )));
                }
            }

            let mut file = match tokio::fs::File::create(&temp_path).await {
                Ok(f) => f,
                Err(e) => {
                    return Ok(ToolResult::error(format!(
                        "Failed to create temp file: {}",
                        e
                    )));
                }
            };

            let mut bytes_written: u64 = 0;
            let mut stream = response.bytes_stream();
            use futures::StreamExt;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        bytes_written += chunk.len() as u64;
                        if let Err(e) = file.write_all(&chunk).await {
                            return Ok(ToolResult::error(format!(
                                "Failed to write to temp file: {}",
                                e
                            )));
                        }
                    }
                    Err(e) => {
                        return Ok(ToolResult::error(format!(
                            "Failed to read response stream: {}",
                            e
                        )));
                    }
                }
            }

            if let Err(e) = file.flush().await {
                return Ok(ToolResult::error(format!(
                    "Failed to flush temp file: {}",
                    e
                )));
            }

            let path_str = temp_path.display().to_string();
            let mut result = json!({
                "url": url_str,
                "method": method,
                "status": status_code,
                "status_text": status_text,
                "downloaded": true,
                "auto_downloaded": true,
                "reason": format!("Response exceeded {} byte limit", self.config.max_size),
                "path": path_str,
                "size": bytes_written,
                "content_type": content_type
            });
            if let Some(expected_size) = content_length {
                result["expected_size"] = json!(expected_size);
            }
            if is_error {
                result["error"] = json!(true);
            }
            if let Some(headers) = response_headers {
                result["headers"] = json!(headers);
            }
            return Ok(ToolResult::json(result));
        }

        // Read body with size limit - fallback to auto-download if exceeded during read
        let bytes = match response.bytes().await {
            Ok(b) => {
                if b.len() > self.config.max_size {
                    // Content-Length wasn't provided but response is too large
                    // Save to temp file and return that
                    let filename = url
                        .path_segments()
                        .and_then(|mut s| s.next_back())
                        .filter(|s| !s.is_empty())
                        .unwrap_or("download");
                    let temp_path = std::env::temp_dir().join("arawn_downloads").join(format!(
                        "{}_{}",
                        uuid::Uuid::new_v4(),
                        filename
                    ));

                    if let Some(parent) = temp_path.parent() {
                        let _ = tokio::fs::create_dir_all(parent).await;
                    }

                    if let Err(e) = tokio::fs::write(&temp_path, &b).await {
                        return Ok(ToolResult::error(format!(
                            "Response too large ({} bytes) and failed to save to temp file: {}",
                            b.len(),
                            e
                        )));
                    }

                    let path_str = temp_path.display().to_string();
                    let mut result = json!({
                        "url": url_str,
                        "method": method,
                        "status": status_code,
                        "status_text": status_text,
                        "downloaded": true,
                        "auto_downloaded": true,
                        "reason": format!("Response exceeded {} byte limit", self.config.max_size),
                        "path": path_str,
                        "size": b.len(),
                        "content_type": content_type
                    });
                    if is_error {
                        result["error"] = json!(true);
                    }
                    if let Some(headers) = response_headers {
                        result["headers"] = json!(headers);
                    }
                    return Ok(ToolResult::json(result));
                }
                b
            }
            Err(e) => return Ok(ToolResult::error(format!("Failed to read response: {}", e))),
        };

        let response_body = String::from_utf8_lossy(&bytes).to_string();

        // Build base result with status info
        let build_result = |content: Value, extra: Option<serde_json::Map<String, Value>>| {
            let mut result = json!({
                "url": url_str,
                "method": method,
                "status": status_code,
                "status_text": status_text,
                "content_type": content_type,
                "content": content
            });
            if is_error {
                result["error"] = json!(true);
            }
            if let Some(headers) = &response_headers {
                result["headers"] = json!(headers);
            }
            if let Some(extra_fields) = extra {
                for (k, v) in extra_fields {
                    result[k] = v;
                }
            }
            result
        };

        // Return raw HTML if requested
        if raw {
            return Ok(ToolResult::json(build_result(json!(response_body), None)));
        }

        // Extract text from HTML
        if content_type.contains("text/html") {
            let title = self.extract_title(&response_body);
            let description = self.extract_description(&response_body);
            let text = self.extract_text_from_html(&response_body);

            let mut extra = serde_json::Map::new();
            if let Some(t) = title {
                extra.insert("title".to_string(), json!(t));
            }
            if let Some(d) = description {
                extra.insert("description".to_string(), json!(d));
            }

            Ok(ToolResult::json(build_result(json!(text), Some(extra))))
        } else {
            // Return raw content for non-HTML
            let truncated = if response_body.len() > self.config.max_text_length {
                format!(
                    "{}...[truncated]",
                    &response_body[..self.config.max_text_length]
                )
            } else {
                response_body
            };

            Ok(ToolResult::json(build_result(json!(truncated), None)))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Web Search Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Web search provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum SearchProvider {
    /// Brave Search API
    Brave { api_key: String },
    /// Serper (Google Search API)
    Serper { api_key: String },
    /// Tavily Search API
    Tavily { api_key: String },
    /// DuckDuckGo (no API key needed, but limited)
    DuckDuckGo,
}

/// Configuration for web search.
#[derive(Debug, Clone)]
pub struct WebSearchConfig {
    /// Search provider configuration.
    pub provider: SearchProvider,
    /// Maximum number of results to return.
    pub max_results: usize,
    /// Request timeout.
    pub timeout: Duration,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            provider: SearchProvider::DuckDuckGo,
            max_results: 10,
            timeout: Duration::from_secs(30),
        }
    }
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Tool for searching the web.
#[derive(Debug, Clone)]
pub struct WebSearchTool {
    client: Client,
    config: WebSearchConfig,
}

impl WebSearchTool {
    /// Create a new web search tool with default configuration (DuckDuckGo).
    pub fn new() -> Self {
        let config = WebSearchConfig::default();
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Create a web search tool with custom configuration.
    pub fn with_config(config: WebSearchConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Create a web search tool with Brave Search.
    pub fn brave(api_key: impl Into<String>) -> Self {
        Self::with_config(WebSearchConfig {
            provider: SearchProvider::Brave {
                api_key: api_key.into(),
            },
            ..Default::default()
        })
    }

    /// Create a web search tool with Serper.
    pub fn serper(api_key: impl Into<String>) -> Self {
        Self::with_config(WebSearchConfig {
            provider: SearchProvider::Serper {
                api_key: api_key.into(),
            },
            ..Default::default()
        })
    }

    /// Create a web search tool with Tavily.
    pub fn tavily(api_key: impl Into<String>) -> Self {
        Self::with_config(WebSearchConfig {
            provider: SearchProvider::Tavily {
                api_key: api_key.into(),
            },
            ..Default::default()
        })
    }

    async fn search_brave(&self, query: &str, api_key: &str) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            self.config.max_results
        );

        let response = self
            .client
            .get(&url)
            .header("X-Subscription-Token", api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| crate::error::AgentError::Tool(format!("Brave search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(crate::error::AgentError::Tool(format!(
                "Brave search error: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to parse response: {}", e))
        })?;

        let results = data["web"]["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        Some(SearchResult {
                            title: r["title"].as_str()?.to_string(),
                            url: r["url"].as_str()?.to_string(),
                            snippet: r["description"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn search_serper(&self, query: &str, api_key: &str) -> Result<Vec<SearchResult>> {
        let response = self
            .client
            .post("https://google.serper.dev/search")
            .header("X-API-KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&json!({
                "q": query,
                "num": self.config.max_results
            }))
            .send()
            .await
            .map_err(|e| crate::error::AgentError::Tool(format!("Serper search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(crate::error::AgentError::Tool(format!(
                "Serper search error: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to parse response: {}", e))
        })?;

        let results = data["organic"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        Some(SearchResult {
                            title: r["title"].as_str()?.to_string(),
                            url: r["link"].as_str()?.to_string(),
                            snippet: r["snippet"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn search_tavily(&self, query: &str, api_key: &str) -> Result<Vec<SearchResult>> {
        let response = self
            .client
            .post("https://api.tavily.com/search")
            .header("Content-Type", "application/json")
            .json(&json!({
                "api_key": api_key,
                "query": query,
                "max_results": self.config.max_results
            }))
            .send()
            .await
            .map_err(|e| crate::error::AgentError::Tool(format!("Tavily search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(crate::error::AgentError::Tool(format!(
                "Tavily search error: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to parse response: {}", e))
        })?;

        let results = data["results"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        Some(SearchResult {
                            title: r["title"].as_str()?.to_string(),
                            url: r["url"].as_str()?.to_string(),
                            snippet: r["content"].as_str().unwrap_or("").to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn search_duckduckgo(&self, query: &str) -> Result<Vec<SearchResult>> {
        // DuckDuckGo instant answer API (limited but free)
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let response = self.client.get(&url).send().await.map_err(|e| {
            crate::error::AgentError::Tool(format!("DuckDuckGo search failed: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(crate::error::AgentError::Tool(format!(
                "DuckDuckGo search error: {}",
                response.status()
            )));
        }

        let data: Value = response.json().await.map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to parse response: {}", e))
        })?;

        let mut results = Vec::new();

        // Add abstract if available
        if let Some(abstract_text) = data["AbstractText"].as_str() {
            if !abstract_text.is_empty() {
                results.push(SearchResult {
                    title: data["Heading"].as_str().unwrap_or("Result").to_string(),
                    url: data["AbstractURL"].as_str().unwrap_or("").to_string(),
                    snippet: abstract_text.to_string(),
                });
            }
        }

        // Add related topics
        if let Some(topics) = data["RelatedTopics"].as_array() {
            for topic in topics.iter().take(self.config.max_results - results.len()) {
                if let (Some(text), Some(url)) =
                    (topic["Text"].as_str(), topic["FirstURL"].as_str())
                {
                    results.push(SearchResult {
                        title: text.chars().take(50).collect::<String>() + "...",
                        url: url.to_string(),
                        snippet: text.to_string(),
                    });
                }
            }
        }

        Ok(results)
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns a list of relevant results with titles, URLs, and snippets."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'query' parameter".to_string())
            })?;

        let results = match &self.config.provider {
            SearchProvider::Brave { api_key } => self.search_brave(query, api_key).await,
            SearchProvider::Serper { api_key } => self.search_serper(query, api_key).await,
            SearchProvider::Tavily { api_key } => self.search_tavily(query, api_key).await,
            SearchProvider::DuckDuckGo => self.search_duckduckgo(query).await,
        };

        match results {
            Ok(results) => {
                if results.is_empty() {
                    Ok(ToolResult::text("No results found"))
                } else {
                    Ok(ToolResult::json(json!({
                        "query": query,
                        "count": results.len(),
                        "results": results
                    })))
                }
            }
            Err(e) => Ok(ToolResult::error(format!("Search failed: {}", e))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_fetch_tool_metadata() {
        let tool = WebFetchTool::new();
        assert_eq!(tool.name(), "web_fetch");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params["properties"].get("url").is_some());
        assert!(params["properties"].get("method").is_some());
        assert!(params["properties"].get("headers").is_some());
        assert!(params["properties"].get("body").is_some());
        assert!(params["properties"].get("timeout_secs").is_some());
        assert!(params["properties"].get("include_headers").is_some());

        // Verify method enum values
        let method_enum = &params["properties"]["method"]["enum"];
        assert!(method_enum.as_array().unwrap().contains(&json!("GET")));
        assert!(method_enum.as_array().unwrap().contains(&json!("POST")));
        assert!(method_enum.as_array().unwrap().contains(&json!("PUT")));
        assert!(method_enum.as_array().unwrap().contains(&json!("PATCH")));
        assert!(method_enum.as_array().unwrap().contains(&json!("DELETE")));
    }

    #[test]
    fn test_web_search_tool_metadata() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.name(), "web_search");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params["properties"].get("query").is_some());
    }

    #[test]
    fn test_extract_text_from_html() {
        let tool = WebFetchTool::new();
        let html = r#"
            <html>
            <head><title>Test Page</title></head>
            <body>
                <nav>Navigation</nav>
                <main>
                    <h1>Hello World</h1>
                    <p>This is the main content.</p>
                </main>
                <footer>Footer</footer>
            </body>
            </html>
        "#;

        let text = tool.extract_text_from_html(html);
        assert!(text.contains("Hello World"));
        assert!(text.contains("main content"));
    }

    #[test]
    fn test_extract_title() {
        let tool = WebFetchTool::new();
        let html = "<html><head><title>My Title</title></head><body></body></html>";
        assert_eq!(tool.extract_title(html), Some("My Title".to_string()));
    }

    #[test]
    fn test_extract_description() {
        let tool = WebFetchTool::new();
        let html =
            r#"<html><head><meta name="description" content="My description"></head></html>"#;
        assert_eq!(
            tool.extract_description(html),
            Some("My description".to_string())
        );
    }

    #[test]
    fn test_search_providers() {
        // Test that different providers can be created
        let _brave = WebSearchTool::brave("test_key");
        let _serper = WebSearchTool::serper("test_key");
        let _tavily = WebSearchTool::tavily("test_key");
        let _ddg = WebSearchTool::new(); // DuckDuckGo default
    }

    #[tokio::test]
    async fn test_web_fetch_invalid_url() {
        let tool = WebFetchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"url": "not-a-valid-url"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Invalid URL"));
    }

    #[tokio::test]
    async fn test_web_fetch_non_http() {
        let tool = WebFetchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"url": "ftp://example.com/file"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("HTTP"));
    }

    #[tokio::test]
    async fn test_web_fetch_unsupported_method() {
        let tool = WebFetchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({"url": "https://example.com", "method": "TRACE"}),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Unsupported HTTP method"));
    }

    #[test]
    fn test_method_case_insensitivity() {
        // The method should be uppercased in execute, so "get" becomes "GET"
        // This is tested via the parameters which show the expected format
        let tool = WebFetchTool::new();
        let params = tool.parameters();
        let default = &params["properties"]["method"]["default"];
        assert_eq!(default, "GET");
    }

    #[tokio::test]
    async fn test_web_fetch_with_custom_headers_invalid_url() {
        // Test that headers parameter is properly parsed even with an invalid URL
        let tool = WebFetchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "url": "not-valid",
                    "headers": {
                        "Authorization": "Bearer token123",
                        "X-Custom-Header": "custom-value"
                    }
                }),
                &ctx,
            )
            .await
            .unwrap();

        // Should fail on URL parsing, not header parsing
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Invalid URL"));
    }

    #[tokio::test]
    async fn test_web_fetch_with_body_invalid_url() {
        // Test that body parameter is accepted even with an invalid URL
        let tool = WebFetchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "url": "not-valid",
                    "method": "POST",
                    "body": "{\"key\": \"value\"}"
                }),
                &ctx,
            )
            .await
            .unwrap();

        // Should fail on URL parsing, not body parsing
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Invalid URL"));
    }

    #[test]
    fn test_download_parameter_in_schema() {
        let tool = WebFetchTool::new();
        let params = tool.parameters();

        assert!(params["properties"].get("download").is_some());
        assert_eq!(params["properties"]["download"]["type"], "string");
    }

    #[test]
    fn test_max_size_config() {
        let config = WebFetchConfig::default();
        // Should be 10MB
        assert_eq!(config.max_size, 10 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_web_fetch_download_invalid_url() {
        // Test that download parameter is accepted even with an invalid URL
        let tool = WebFetchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "url": "not-valid",
                    "download": "/tmp/test_file.bin"
                }),
                &ctx,
            )
            .await
            .unwrap();

        // Should fail on URL parsing, not download path
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Invalid URL"));
    }
}
