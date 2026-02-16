//! HTTP client SDK for the Arawn AI agent platform.
//!
//! This crate provides a typed client for interacting with the Arawn server API.
//!
//! # Example
//!
//! ```no_run
//! use arawn_client::{ArawnClient, Result};
//!
//! # async fn example() -> Result<()> {
//! // Create a client
//! let client = ArawnClient::builder()
//!     .base_url("http://localhost:8080")
//!     .auth_token("secret")
//!     .build()?;
//!
//! // Check server health
//! if client.health().is_healthy().await {
//!     println!("Server is healthy!");
//! }
//!
//! // Create a session
//! let session = client.sessions().create(Default::default()).await?;
//! println!("Created session: {}", session.id);
//!
//! // Send a chat message
//! let response = client.chat().message_in_session(&session.id, "Hello!").await?;
//! println!("Response: {}", response.response);
//!
//! // Stream a response
//! use futures::StreamExt;
//! use tokio::pin;
//! let stream = client.chat().stream_message("Tell me a story").await?;
//! pin!(stream);
//! while let Some(event) = stream.next().await {
//!     match event? {
//!         arawn_client::StreamEvent::Content { text } => print!("{}", text),
//!         arawn_client::StreamEvent::Done { .. } => println!("\n[Done]"),
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # API Coverage
//!
//! The client provides access to all server endpoints:
//!
//! - **Sessions**: Create, list, update, delete sessions
//! - **Chat**: Send messages, stream responses
//! - **Workstreams**: Manage workstreams and messages
//! - **Config**: Get server configuration
//! - **Agents**: List agents and their tools
//! - **Notes**: CRUD operations for notes
//! - **Memory**: Search and store memories
//! - **Tasks**: List and cancel background tasks
//! - **MCP**: Manage Model Context Protocol servers
//! - **Health**: Server health checks

pub mod api;
pub mod client;
pub mod error;
pub mod types;

pub use client::{ArawnClient, ClientBuilder};
pub use error::{Error, Result};
pub use types::*;

// Re-export API types that are commonly used with query methods
pub use api::{ListMessagesQuery, ListNotesQuery, ListTasksQuery, MemorySearchQuery};
