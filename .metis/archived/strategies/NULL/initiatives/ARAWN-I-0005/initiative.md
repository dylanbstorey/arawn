---
id: server-http-api-and-websocket
level: initiative
title: "Server: HTTP API and WebSocket Transport"
short_code: "ARAWN-I-0005"
created_at: 2026-01-28T01:37:29.814265+00:00
updated_at: 2026-01-28T13:21:44.280682+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: server-http-api-and-websocket
---

# Server: HTTP API and WebSocket Transport

## Context

The transport layer that exposes the agent to clients (CLI, mobile apps, external tools). Built with axum for both REST API and WebSocket support.

Key security requirement: authentication required for all endpoints (lesson from clawdbot incident).

## Goals & Non-Goals

**Goals:**
- REST API for request/response interactions
- WebSocket for real-time bidirectional communication
- SSE for streaming responses
- Token-based authentication (no localhost bypass)
- Rate limiting
- Audit logging
- Bind to localhost by default (explicit opt-in for external)

**Non-Goals:**
- Web UI (v1 is CLI + mobile only)
- TLS termination (handled by Tailscale or reverse proxy)

## Detailed Design

### API Routes

```
POST   /api/v1/chat              # Send message, get response
POST   /api/v1/chat/stream       # Send message, stream response (SSE)
GET    /api/v1/sessions          # List sessions
GET    /api/v1/sessions/:id      # Get session details
DELETE /api/v1/sessions/:id      # Delete session

POST   /api/v1/tasks             # Submit long-running task
GET    /api/v1/tasks             # List tasks
GET    /api/v1/tasks/:id         # Get task status
DELETE /api/v1/tasks/:id         # Cancel task

GET    /api/v1/memory/search     # Search memories
POST   /api/v1/notes             # Create note
GET    /api/v1/notes             # List notes

GET    /api/v1/health            # Health check (no auth)
GET    /api/v1/config            # Get config (auth required)

# LLM Proxy (OpenAI-compatible)
POST   /v1/chat/completions      # Proxy to LLM
POST   /v1/embeddings            # Proxy embeddings

# WebSocket
WS     /ws                       # Bidirectional real-time
```

### WebSocket Protocol

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Chat { session_id: Option<String>, message: String },
    Subscribe { session_id: String },
    Unsubscribe { session_id: String },
    Ping,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    ChatChunk { session_id: String, chunk: String, done: bool },
    TaskUpdate { task_id: String, status: TaskStatus },
    Error { code: String, message: String },
    Pong,
}
```

### Authentication

```rust
pub struct AuthConfig {
    /// Token for API access (generated on first run)
    pub token: String,
    /// Optional: Tailscale identity validation
    pub tailscale_users: Option<Vec<String>>,
}

pub struct AuthMiddleware {
    config: AuthConfig,
}

impl AuthMiddleware {
    fn validate(&self, req: &Request) -> Result<Identity, AuthError> {
        // Check Authorization header
        let token = req.header("Authorization")
            .and_then(|h| h.strip_prefix("Bearer "))?;
        
        if token == self.config.token {
            return Ok(Identity::Token);
        }
        
        // Optional: Check Tailscale identity header
        if let Some(allowed) = &self.config.tailscale_users {
            if let Some(ts_user) = req.header("Tailscale-User-Login") {
                if allowed.contains(&ts_user.to_string()) {
                    return Ok(Identity::Tailscale(ts_user.to_string()));
                }
            }
        }
        
        Err(AuthError::Unauthorized)
    }
}
```

### Rate Limiting

```rust
use governor::{Quota, RateLimiter};

pub struct RateLimits {
    chat: RateLimiter<IpAddr>,      // 60/min per IP
    tasks: RateLimiter<IpAddr>,     // 10/min per IP
    proxy: RateLimiter<IpAddr>,     // 100/min per IP
}
```

### Server Setup

```rust
pub async fn run_server(config: ServerConfig, agent: Arc<Agent>) -> Result<()> {
    let app = Router::new()
        // API routes
        .route("/api/v1/chat", post(chat_handler))
        .route("/api/v1/chat/stream", post(chat_stream_handler))
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/tasks", post(submit_task).get(list_tasks))
        // ...
        
        // LLM Proxy
        .route("/v1/chat/completions", post(proxy_completions))
        .route("/v1/embeddings", post(proxy_embeddings))
        
        // WebSocket
        .route("/ws", get(ws_handler))
        
        // Middleware
        .layer(AuthLayer::new(config.auth))
        .layer(RateLimitLayer::new(config.rate_limits))
        .layer(AuditLogLayer::new(config.audit))
        
        .with_state(AppState { agent });
    
    // Bind to localhost by default
    let addr = config.bind_address; // Default: 127.0.0.1:8080
    
    axum::serve(TcpListener::bind(addr).await?, app).await
}
```

### Dependencies

```toml
[dependencies]
axum = { version = "0.8", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
governor = "0.6"
tracing = "0.1"
```

## Alternatives Considered

- **gRPC**: Rejected - HTTP/WS simpler for mobile clients
- **actix-web**: axum preferred for tower ecosystem compatibility
- **No WebSocket**: Rejected - needed for real-time streaming

## Implementation Plan

1. Basic axum server with health endpoint
2. Authentication middleware
3. Chat endpoints (sync and streaming)
4. Session management endpoints
5. Task endpoints
6. WebSocket handler
7. LLM proxy endpoints
8. Rate limiting
9. Audit logging