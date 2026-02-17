//! WebSocket message handlers.

use futures::StreamExt;
use uuid::Uuid;

use arawn_agent::{SessionId, ToolCall, ToolResultRecord, Turn, TurnId};

use crate::routes::commands::{CommandOutput, CommandRegistry};
use crate::state::AppState;
use super::connection::ConnectionState;
use super::protocol::{ClientMessage, ServerMessage};

/// Response from handling a message.
pub enum MessageResponse {
    /// Send a single message.
    Single(ServerMessage),
    /// Send a stream of messages.
    Stream(futures::stream::BoxStream<'static, ServerMessage>),
    /// No response needed.
    None,
}

/// Handle a client message.
pub async fn handle_message(
    msg: ClientMessage,
    conn_state: &mut ConnectionState,
    app_state: &AppState,
) -> MessageResponse {
    match msg {
        ClientMessage::Ping => MessageResponse::Single(ServerMessage::Pong),

        ClientMessage::Auth { token } => handle_auth(token, conn_state, app_state),

        ClientMessage::Subscribe { session_id } => {
            handle_subscribe(session_id, conn_state)
        }

        ClientMessage::Unsubscribe { session_id } => {
            handle_unsubscribe(session_id, conn_state)
        }

        ClientMessage::Cancel { session_id } => {
            handle_cancel(session_id, conn_state)
        }

        ClientMessage::Chat {
            session_id,
            workstream_id,
            message,
        } => {
            handle_chat(session_id, workstream_id, message, conn_state, app_state).await
        }

        ClientMessage::Command { command, args } => {
            handle_command(command, args, conn_state, app_state).await
        }
    }
}

/// Handle authentication.
fn handle_auth(
    token: String,
    conn_state: &mut ConnectionState,
    app_state: &AppState,
) -> MessageResponse {
    let authed = match &app_state.config.auth_token {
        None => true,
        Some(expected) => token == *expected,
    };
    if authed {
        conn_state.authenticated = true;
        MessageResponse::Single(ServerMessage::auth_success())
    } else {
        MessageResponse::Single(ServerMessage::auth_failure("Invalid token"))
    }
}

/// Handle session subscription.
fn handle_subscribe(
    session_id: String,
    conn_state: &mut ConnectionState,
) -> MessageResponse {
    if !conn_state.authenticated {
        return MessageResponse::Single(ServerMessage::error(
            "unauthorized",
            "Authentication required",
        ));
    }

    match Uuid::parse_str(&session_id) {
        Ok(uuid) => {
            conn_state.subscriptions.insert(SessionId::from_uuid(uuid));
            MessageResponse::None
        }
        Err(_) => MessageResponse::Single(ServerMessage::error(
            "invalid_session",
            "Invalid session ID",
        )),
    }
}

/// Handle session unsubscription.
fn handle_unsubscribe(
    session_id: String,
    conn_state: &mut ConnectionState,
) -> MessageResponse {
    if let Ok(uuid) = Uuid::parse_str(&session_id) {
        conn_state.subscriptions.remove(&SessionId::from_uuid(uuid));
    }
    MessageResponse::None
}

/// Handle cancellation request.
fn handle_cancel(
    session_id: String,
    conn_state: &mut ConnectionState,
) -> MessageResponse {
    if !conn_state.authenticated {
        return MessageResponse::Single(ServerMessage::error(
            "unauthorized",
            "Authentication required",
        ));
    }

    // Validate session ID
    if Uuid::parse_str(&session_id).is_err() {
        return MessageResponse::Single(ServerMessage::error(
            "invalid_session",
            "Invalid session ID",
        ));
    }

    // Cancel via the cancellation token
    conn_state.cancellation.cancel();
    // Create a new token for future operations
    conn_state.cancellation = tokio_util::sync::CancellationToken::new();

    MessageResponse::None
}

/// Handle command execution.
async fn handle_command(
    command: String,
    args: serde_json::Value,
    conn_state: &ConnectionState,
    app_state: &AppState,
) -> MessageResponse {
    if !conn_state.authenticated {
        return MessageResponse::Single(ServerMessage::error(
            "unauthorized",
            "Authentication required",
        ));
    }

    // Get the command registry (create with defaults)
    let registry = CommandRegistry::with_defaults();

    // Look up the command handler
    let handler = match registry.get(&command) {
        Some(h) => h,
        None => {
            return MessageResponse::Single(ServerMessage::command_failure(
                &command,
                format!("Unknown command: {}", command),
            ));
        }
    };

    // For commands that need session context, inject the current session
    // if the client didn't provide one and we have a current subscription
    let args = inject_session_context(args, conn_state);

    // Send progress message
    let command_name = command.clone();
    let progress_msg = ServerMessage::command_progress(&command_name, "Starting...", Some(0));

    // Execute the command
    let result = handler.execute(app_state, args).await;

    // Create response stream with progress and result
    let response_stream = async_stream::stream! {
        yield progress_msg;

        match result {
            Ok(output) => {
                match output {
                    CommandOutput::Completed { result } => {
                        yield ServerMessage::command_success(&command_name, result);
                    }
                    CommandOutput::Text { message } => {
                        yield ServerMessage::command_success(
                            &command_name,
                            serde_json::json!({ "message": message }),
                        );
                    }
                    CommandOutput::Json { data } => {
                        yield ServerMessage::command_success(&command_name, data);
                    }
                    CommandOutput::Progress { percent, message } => {
                        yield ServerMessage::command_progress(&command_name, message, Some(percent));
                    }
                    CommandOutput::Error { error } => {
                        yield ServerMessage::command_failure(&command_name, error.message);
                    }
                }
            }
            Err(e) => {
                yield ServerMessage::command_failure(&command_name, e.message);
            }
        }
    };

    MessageResponse::Stream(Box::pin(response_stream))
}

/// Inject session context from the connection state if not provided in args.
fn inject_session_context(
    mut args: serde_json::Value,
    conn_state: &ConnectionState,
) -> serde_json::Value {
    // If args is null, create an empty object
    if args.is_null() {
        args = serde_json::json!({});
    }

    // If args is an object and doesn't have session_id, try to inject from subscriptions
    if let Some(obj) = args.as_object_mut() {
        if !obj.contains_key("session_id") {
            // Use the first subscribed session if available
            if let Some(session_id) = conn_state.subscriptions.iter().next() {
                obj.insert(
                    "session_id".to_string(),
                    serde_json::Value::String(session_id.to_string()),
                );
            }
        }
    }

    args
}

/// Handle chat message.
async fn handle_chat(
    session_id: Option<String>,
    workstream_id: Option<String>,
    message: String,
    conn_state: &mut ConnectionState,
    app_state: &AppState,
) -> MessageResponse {
    if !conn_state.authenticated {
        return MessageResponse::Single(ServerMessage::error(
            "unauthorized",
            "Authentication required",
        ));
    }

    // Parse session ID if provided
    let session_id = session_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(SessionId::from_uuid);

    // Resolve workstream ID (default to "scratch")
    let ws_id = workstream_id.as_deref().unwrap_or("scratch");

    // Get or create session using the session cache
    let session_id = app_state
        .get_or_create_session_in_workstream(session_id, ws_id)
        .await;
    let session_id_str = session_id.to_string();

    // Store user message in workstream (if workstreams enabled)
    if let Some(ref ws_manager) = app_state.workstreams {
        use arawn_workstream::MessageRole;
        if let Err(e) = ws_manager.send_message(
            workstream_id.as_deref(),
            Some(&session_id_str),
            MessageRole::User,
            &message,
            None,
        ) {
            tracing::warn!("Failed to store user message in workstream: {}", e);
        }
    }

    // Get the agent stream
    let stream_result = {
        if let Some(mut session) = app_state.session_cache.get(&session_id).await {
            let cancellation = conn_state.cancellation.clone();
            let stream = app_state.agent.turn_stream(&mut session, &message, cancellation);
            app_state.update_session(session_id, session).await;
            Some(stream)
        } else {
            None
        }
    };

    let stream = match stream_result {
        Some(s) => s,
        None => {
            return MessageResponse::Single(ServerMessage::error(
                "internal",
                "Session disappeared",
            ));
        }
    };

    // Clone references for use in async stream
    let workstream_id_for_stream = workstream_id.clone();
    let session_cache = app_state.session_cache.clone();
    let user_message = message.clone();

    // Create response stream
    let session_id_for_stream = session_id_str.clone();
    let response_stream = async_stream::stream! {
        // First, send session created
        yield ServerMessage::SessionCreated {
            session_id: session_id_for_stream.clone(),
        };

        // Accumulate the full assistant response and tool data for workstream storage
        let mut full_response = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut tool_results: Vec<ToolResultRecord> = Vec::new();
        let mut current_tool_output: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        let mut stream = std::pin::pin!(stream);
        while let Some(chunk) = stream.next().await {
            use arawn_agent::StreamChunk;

            match chunk {
                StreamChunk::Text { content } => {
                    full_response.push_str(&content);
                    yield ServerMessage::ChatChunk {
                        session_id: session_id_for_stream.clone(),
                        chunk: content,
                        done: false,
                    };
                }
                StreamChunk::ToolStart { id, name } => {
                    tool_calls.push(ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        arguments: serde_json::Value::Null,
                    });
                    yield ServerMessage::ToolStart {
                        session_id: session_id_for_stream.clone(),
                        tool_id: id,
                        tool_name: name,
                    };
                }
                StreamChunk::ToolOutput { id, content } => {
                    current_tool_output
                        .entry(id.clone())
                        .or_default()
                        .push_str(&content);
                    yield ServerMessage::ToolOutput {
                        session_id: session_id_for_stream.clone(),
                        tool_id: id,
                        content,
                    };
                }
                StreamChunk::ToolEnd { id, success, .. } => {
                    let output = current_tool_output.remove(&id).unwrap_or_default();
                    tool_results.push(ToolResultRecord {
                        tool_call_id: id.clone(),
                        success,
                        content: output,
                    });
                    yield ServerMessage::ToolEnd {
                        session_id: session_id_for_stream.clone(),
                        tool_id: id,
                        success,
                    };
                }
                StreamChunk::Done { .. } => {
                    // Persist the complete turn to workstream storage
                    let workstream_id_str = workstream_id_for_stream
                        .as_deref()
                        .unwrap_or("scratch")
                        .to_string();

                    let turn = Turn {
                        id: TurnId::new(),
                        user_message: user_message.clone(),
                        assistant_response: if full_response.is_empty() {
                            None
                        } else {
                            Some(full_response.clone())
                        },
                        tool_calls: tool_calls.clone(),
                        tool_results: tool_results.clone(),
                        started_at: chrono::Utc::now(),
                        completed_at: Some(chrono::Utc::now()),
                    };

                    if let Err(e) = session_cache.save_turn(session_id, &turn, &workstream_id_str).await {
                        tracing::warn!("Failed to persist turn to workstream: {}", e);
                    }

                    yield ServerMessage::ChatChunk {
                        session_id: session_id_for_stream.clone(),
                        chunk: String::new(),
                        done: true,
                    };
                }
                StreamChunk::Error { message } => {
                    yield ServerMessage::Error {
                        code: "agent_error".to_string(),
                        message,
                    };
                }
            }
        }
    };

    MessageResponse::Stream(Box::pin(response_stream))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_session_context_null_args() {
        let conn_state = ConnectionState::new();
        let args = serde_json::Value::Null;
        let result = inject_session_context(args, &conn_state);

        // Should create empty object when no subscriptions
        assert!(result.is_object());
        assert!(!result.as_object().unwrap().contains_key("session_id"));
    }

    #[test]
    fn test_inject_session_context_with_subscription() {
        let mut conn_state = ConnectionState::new();
        let session_id = SessionId::new();
        conn_state.subscriptions.insert(session_id);

        let args = serde_json::json!({});
        let result = inject_session_context(args, &conn_state);

        // Should inject session_id from subscriptions
        assert!(result.as_object().unwrap().contains_key("session_id"));
        assert_eq!(
            result["session_id"].as_str().unwrap(),
            session_id.to_string()
        );
    }

    #[test]
    fn test_inject_session_context_preserves_existing() {
        let mut conn_state = ConnectionState::new();
        let subscribed_id = SessionId::new();
        conn_state.subscriptions.insert(subscribed_id);

        let explicit_id = "00000000-0000-0000-0000-000000000001";
        let args = serde_json::json!({ "session_id": explicit_id });
        let result = inject_session_context(args, &conn_state);

        // Should preserve explicitly provided session_id
        assert_eq!(result["session_id"].as_str().unwrap(), explicit_id);
    }

    #[test]
    fn test_inject_session_context_preserves_other_args() {
        let conn_state = ConnectionState::new();
        let args = serde_json::json!({
            "force": true,
            "other": "value"
        });
        let result = inject_session_context(args, &conn_state);

        // Should preserve other fields
        assert_eq!(result["force"], true);
        assert_eq!(result["other"], "value");
    }
}
