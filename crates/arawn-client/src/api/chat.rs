//! Chat API.

use eventsource_stream::Eventsource;
use futures::StreamExt;
use tokio_stream::Stream;

use crate::client::ArawnClient;
use crate::error::{Error, Result};
use crate::types::{ChatRequest, ChatResponse, StreamEvent};

/// Chat API client.
pub struct ChatApi {
    client: ArawnClient,
}

impl ChatApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// Send a chat message and get a response.
    pub async fn send(&self, request: ChatRequest) -> Result<ChatResponse> {
        self.client.post("chat", &request).await
    }

    /// Send a message with just text (convenience method).
    pub async fn message(&self, text: impl Into<String>) -> Result<ChatResponse> {
        self.send(ChatRequest::new(text)).await
    }

    /// Send a message in an existing session.
    pub async fn message_in_session(
        &self,
        session_id: &str,
        text: impl Into<String>,
    ) -> Result<ChatResponse> {
        self.send(ChatRequest::new(text).with_session(session_id))
            .await
    }

    /// Stream a chat response.
    ///
    /// Returns a stream of events that can be processed as they arrive.
    pub async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<impl Stream<Item = Result<StreamEvent>>> {
        let response = self.client.post_stream("chat/stream", &request).await?;

        // Parse SSE stream using the Eventsource extension trait
        let stream = response.bytes_stream().eventsource();

        Ok(stream.filter_map(|result| async move {
            match result {
                Ok(event) => {
                    // Skip empty events
                    if event.data.is_empty() {
                        return None;
                    }

                    // Parse the event data as JSON
                    match serde_json::from_str::<StreamEvent>(&event.data) {
                        Ok(stream_event) => Some(Ok(stream_event)),
                        Err(e) => {
                            tracing::warn!(data = %event.data, error = %e, "Failed to parse stream event");
                            Some(Err(Error::Json(e)))
                        }
                    }
                }
                Err(e) => Some(Err(Error::Stream(e.to_string()))),
            }
        }))
    }

    /// Stream a message with just text (convenience method).
    pub async fn stream_message(
        &self,
        text: impl Into<String>,
    ) -> Result<impl Stream<Item = Result<StreamEvent>>> {
        self.stream(ChatRequest::new(text)).await
    }
}
