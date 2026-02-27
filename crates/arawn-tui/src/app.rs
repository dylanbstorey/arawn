//! Application state and main loop.

use crate::bounded::BoundedVec;
use crate::client::{ConnectionStatus, WsClient};
use crate::focus::{FocusManager, FocusTarget};

/// Maximum number of chat messages to retain (prevents unbounded memory growth).
const MAX_MESSAGES: usize = 10_000;

/// Maximum number of tool executions to retain per response.
const MAX_TOOLS: usize = 1_000;

use crate::Tui;
use crate::events::{Event, EventHandler};
use crate::input::InputState;
use crate::logs::LogBuffer;
use crate::palette::{ActionId, CommandPalette};
use crate::protocol::ServerMessage;
use crate::sessions::{SessionList, SessionSummary};
use crate::sidebar::{Sidebar, SidebarSection, WorkstreamEntry};
use crate::ui;
use crate::ui::CommandPopup;
use anyhow::Result;
use arawn_client::{ArawnClient, CreateWorkstreamRequest, UpdateWorkstreamRequest};
use chrono::{DateTime, Utc};

/// Pending async actions to be executed in the main loop.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PendingAction {
    /// Create a new workstream with the given title.
    CreateWorkstream(String),
    /// Rename a workstream (id, new_title).
    RenameWorkstream(String, String),
    /// Delete a session by ID.
    DeleteSession(String),
    /// Delete a workstream by ID.
    DeleteWorkstream(String),
    /// Refresh sidebar data.
    RefreshSidebar,
    /// Fetch sessions for a workstream by ID.
    FetchWorkstreamSessions(String),
    /// Fetch message history for a session by ID.
    FetchSessionMessages(String),
    /// Move a session to a different workstream (session_id, new_workstream_id).
    MoveSessionToWorkstream(String, String),
}

/// Input mode determines what the input field is being used for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal chat input.
    #[default]
    Chat,
    /// Creating a new workstream - input is the name.
    NewWorkstream,
    /// Renaming a workstream - stores the workstream ID.
    RenameWorkstream(String),
}
use crossterm::event::{KeyCode, KeyModifiers};

/// A chat message for display.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Whether this is from the user (true) or assistant (false).
    pub is_user: bool,
    /// Message content.
    pub content: String,
    /// Whether the message is still streaming.
    pub streaming: bool,
}

/// A tool execution for display.
#[derive(Debug, Clone)]
pub struct ToolExecution {
    /// Tool call ID.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Tool arguments (truncated for display).
    pub args: String,
    /// Accumulated output.
    pub output: String,
    /// Whether the tool is still running.
    pub running: bool,
    /// Whether the tool succeeded (None if still running).
    pub success: Option<bool>,
    /// When the tool started (for duration calculation).
    pub started_at: std::time::Instant,
    /// Duration in milliseconds (calculated when tool ends).
    pub duration_ms: Option<u64>,
}

/// Main application state.
pub struct App {
    /// Server URL to connect to.
    pub server_url: String,
    /// WebSocket client for real-time chat.
    pub ws_client: WsClient,
    /// HTTP API client for REST endpoints.
    pub api: ArawnClient,
    /// Current connection status.
    pub connection_status: ConnectionStatus,
    /// Focus manager for panel/overlay navigation.
    pub focus: FocusManager,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Input state with history.
    pub input: InputState,
    /// Current input mode (chat, new workstream, rename, etc.)
    pub input_mode: InputMode,
    /// Status bar message.
    pub status_message: Option<String>,
    /// Current workstream name.
    pub workstream: String,
    /// Current workstream ID (for API calls).
    pub workstream_id: Option<String>,
    /// Current session ID.
    pub session_id: Option<String>,
    /// Chat messages (bounded to prevent unbounded growth).
    pub messages: BoundedVec<ChatMessage>,
    /// Tool executions in current response (bounded).
    pub tools: BoundedVec<ToolExecution>,
    /// Whether we're waiting for a response.
    pub waiting: bool,
    /// Chat scroll offset (lines from top).
    pub chat_scroll: usize,
    /// Whether to auto-scroll to bottom during streaming.
    pub chat_auto_scroll: bool,
    /// Session list state.
    pub sessions: SessionList,
    /// Command palette state.
    pub palette: CommandPalette,
    /// Context name (for display in header).
    pub context_name: Option<String>,
    /// Log buffer for capturing and displaying logs.
    pub log_buffer: LogBuffer,
    /// Log scroll offset.
    pub log_scroll: usize,
    /// Whether logs panel is visible.
    pub show_logs: bool,
    /// Sidebar state for workstreams and sessions navigation.
    pub sidebar: Sidebar,
    /// Whether sidebar is in "move session to workstream" mode.
    pub moving_session_to_workstream: bool,
    /// Tool pane scroll offset.
    pub tool_scroll: usize,
    /// Currently selected tool index (for tool pane navigation).
    pub selected_tool_index: Option<usize>,
    /// Whether the tool pane (split view) is visible.
    pub show_tool_pane: bool,
    /// Pending async actions to process.
    pending_actions: Vec<PendingAction>,
    /// Command autocomplete popup.
    pub command_popup: CommandPopup,
    /// Whether a command is currently executing.
    pub command_executing: bool,
    /// Current command execution progress message.
    pub command_progress: Option<String>,
    /// Context usage information for current session.
    pub context_info: Option<ContextState>,
    /// Disk usage stats for current workstream.
    pub workstream_usage: Option<UsageStats>,
    /// Active disk warnings.
    pub disk_warnings: Vec<DiskWarning>,
    /// Whether to show usage popup (Ctrl+U).
    pub show_usage_popup: bool,
    /// Reconnect tokens for session ownership recovery after disconnect.
    /// Maps session_id -> reconnect_token.
    pub reconnect_tokens: std::collections::HashMap<String, String>,
    /// Whether the current session is owned by this client (can send Chat).
    /// When false, the client is in read-only mode.
    pub is_session_owner: bool,
    /// Pending delete confirmation for workstream (id, name).
    /// Set on first 'd' press, cleared on second 'd' (executes delete) or any other action.
    pub pending_delete_workstream: Option<(String, String)>,
    /// Pending delete confirmation for session (id).
    /// Set on first 'd' press, cleared on second 'd' (executes delete) or any other action.
    pub pending_delete_session: Option<String>,
}

/// Context usage state for display in status bar.
#[derive(Debug, Clone)]
pub struct ContextState {
    /// Current token count.
    pub current_tokens: usize,
    /// Maximum tokens.
    pub max_tokens: usize,
    /// Usage percentage (0-100).
    pub percent: u8,
    /// Status: "ok", "warning", or "critical".
    pub status: String,
}

/// Disk usage statistics for a workstream.
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    /// Workstream ID.
    pub workstream_id: String,
    /// Workstream name.
    pub workstream_name: String,
    /// Whether this is a scratch workstream.
    pub is_scratch: bool,
    /// Size of production directory in bytes.
    pub production_bytes: u64,
    /// Size of work directory in bytes.
    pub work_bytes: u64,
    /// Total size in bytes.
    pub total_bytes: u64,
    /// Configured limit in bytes (0 = no limit).
    pub limit_bytes: u64,
    /// Usage percentage (0-100).
    pub percent: u8,
}

impl UsageStats {
    /// Format size as human-readable string.
    pub fn format_size(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.0} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.0} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Get formatted production size.
    pub fn production_size(&self) -> String {
        Self::format_size(self.production_bytes)
    }

    /// Get formatted work size.
    pub fn work_size(&self) -> String {
        Self::format_size(self.work_bytes)
    }

    /// Get formatted total size.
    pub fn total_size(&self) -> String {
        Self::format_size(self.total_bytes)
    }

    /// Get formatted limit.
    pub fn limit_size(&self) -> String {
        if self.limit_bytes == 0 {
            "∞".to_string()
        } else {
            Self::format_size(self.limit_bytes)
        }
    }
}

/// A disk usage warning.
#[derive(Debug, Clone)]
pub struct DiskWarning {
    /// Workstream that triggered the warning.
    pub workstream: String,
    /// Warning level: "warning" or "critical".
    pub level: String,
    /// Current usage in bytes.
    pub usage_bytes: u64,
    /// Limit in bytes.
    pub limit_bytes: u64,
    /// Usage percentage.
    pub percent: u8,
    /// When the warning was received.
    pub timestamp: std::time::Instant,
}

impl App {
    /// Create a new App instance.
    ///
    /// Returns an error if the HTTP API client cannot be constructed
    /// (e.g., invalid server URL).
    pub fn new(server_url: String, log_buffer: LogBuffer) -> Result<Self> {
        let ws_client = WsClient::new(&server_url);

        // Build HTTP API client, reading auth token from environment
        let mut builder = ArawnClient::builder().base_url(&server_url);
        if let Ok(token) = std::env::var("ARAWN_API_TOKEN") {
            builder = builder.auth_token(token);
        }
        let api = builder.build()?;

        let sidebar = Sidebar::new();

        Ok(Self {
            server_url: server_url.clone(),
            ws_client,
            api,
            connection_status: ConnectionStatus::Connecting,
            focus: FocusManager::new(),
            should_quit: false,
            input: InputState::new(),
            input_mode: InputMode::default(),
            status_message: None,
            workstream: "scratch".to_string(),
            workstream_id: None, // Will be set when workstreams load
            session_id: None,
            messages: BoundedVec::with_capacity(MAX_MESSAGES, 1024),
            tools: BoundedVec::with_capacity(MAX_TOOLS, 64),
            waiting: false,
            chat_scroll: 0,
            chat_auto_scroll: true,
            sessions: SessionList::new(),
            palette: CommandPalette::new(),
            context_name: None,
            log_buffer,
            log_scroll: 0,
            show_logs: false,
            sidebar,
            moving_session_to_workstream: false,
            tool_scroll: 0,
            selected_tool_index: None,
            show_tool_pane: false,
            pending_actions: Vec::new(),
            command_popup: CommandPopup::new(),
            command_executing: false,
            command_progress: None,
            context_info: None,
            workstream_usage: None,
            disk_warnings: Vec::new(),
            show_usage_popup: false,
            reconnect_tokens: std::collections::HashMap::new(),
            is_session_owner: true, // Default to owner until told otherwise
            pending_delete_workstream: None,
            pending_delete_session: None,
        })
    }

    /// Push a message (BoundedVec handles eviction automatically).
    fn push_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    /// Push a tool execution (BoundedVec handles eviction automatically).
    fn push_tool(&mut self, tool: ToolExecution) {
        self.tools.push(tool);
    }

    /// Run the main application loop.
    pub async fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        let mut events = EventHandler::new();
        let mut data_loaded = false;

        while !self.should_quit {
            // Render the UI
            terminal.draw(|frame| ui::render(self, frame))?;

            // Handle events
            tokio::select! {
                // Terminal events
                event = events.next() => {
                    match event? {
                        Event::Key(key) => self.handle_key(key),
                        Event::Tick => {
                            // Poll for connection status updates
                            if let Some(status) = self.ws_client.poll_status() {
                                let was_connected = self.connection_status == ConnectionStatus::Connected;
                                self.connection_status = status;

                                // Load data when we first connect
                                if !was_connected && status == ConnectionStatus::Connected && !data_loaded {
                                    data_loaded = true;
                                    self.refresh_sidebar_data().await;
                                }
                            }
                        }
                        Event::Resize(_, _) => {
                            // Terminal resized, will re-render on next iteration
                        }
                    }
                }

                // WebSocket messages
                msg = self.ws_client.recv() => {
                    if let Some(msg) = msg {
                        self.handle_server_message(msg);
                    }
                }
            }

            // Process any pending async actions
            self.process_pending_actions().await;
        }

        Ok(())
    }

    /// Process pending async actions.
    async fn process_pending_actions(&mut self) {
        // Take all pending actions and deduplicate to avoid redundant work
        let mut actions: Vec<_> = self.pending_actions.drain(..).collect();

        // Deduplicate while preserving order (keep first occurrence)
        let mut seen = std::collections::HashSet::new();
        actions.retain(|action| seen.insert(action.clone()));

        for action in actions {
            match action {
                PendingAction::CreateWorkstream(title) => {
                    self.do_create_workstream(&title).await;
                }
                PendingAction::RenameWorkstream(id, new_title) => {
                    self.do_rename_workstream(&id, &new_title).await;
                }
                PendingAction::DeleteSession(id) => {
                    self.do_delete_session(&id).await;
                }
                PendingAction::DeleteWorkstream(id) => {
                    self.do_delete_workstream(&id).await;
                }
                PendingAction::RefreshSidebar => {
                    self.refresh_sidebar_data().await;
                }
                PendingAction::FetchWorkstreamSessions(workstream_id) => {
                    self.do_fetch_workstream_sessions(&workstream_id).await;
                }
                PendingAction::FetchSessionMessages(session_id) => {
                    self.do_fetch_session_messages(&session_id).await;
                }
                PendingAction::MoveSessionToWorkstream(session_id, workstream_id) => {
                    tracing::info!(
                        "Processing pending action: MoveSessionToWorkstream({}, {})",
                        session_id,
                        workstream_id
                    );
                    self.do_move_session_to_workstream(&session_id, &workstream_id)
                        .await;
                }
            }
        }
    }

    /// Create a workstream via API.
    async fn do_create_workstream(&mut self, title: &str) {
        let request = CreateWorkstreamRequest {
            title: title.to_string(),
            default_model: None,
            tags: vec![],
        };

        match self.api.workstreams().create(request).await {
            Ok(workstream) => {
                tracing::info!(
                    "Created workstream: {} ({})",
                    workstream.title,
                    workstream.id
                );
                self.status_message = Some(format!("Created workstream: {}", workstream.title));

                // Add to sidebar and switch to it
                self.sidebar.workstreams.push(WorkstreamEntry {
                    id: workstream.id.clone(),
                    name: workstream.title.clone(),
                    session_count: 0,
                    is_current: false,
                    is_scratch: false,
                    usage_bytes: None,
                    limit_bytes: None,
                    state: "active".to_string(),
                });

                // Switch to the new workstream
                self.switch_to_workstream(&workstream.title);
            }
            Err(e) => {
                tracing::error!("Failed to create workstream: {}", e);
                self.status_message = Some(format!("Failed to create workstream: {}", e));
            }
        }
    }

    /// Rename a workstream via API.
    async fn do_rename_workstream(&mut self, id: &str, new_title: &str) {
        let request = UpdateWorkstreamRequest {
            title: Some(new_title.to_string()),
            summary: None,
            default_model: None,
            tags: None,
        };

        match self.api.workstreams().update(id, request).await {
            Ok(workstream) => {
                tracing::info!("Renamed workstream to: {}", workstream.title);
                self.status_message = Some(format!("Renamed to: {}", workstream.title));

                // Update sidebar entry - lookup by ID, not name
                if let Some(entry) = self.sidebar.workstreams.iter_mut().find(|ws| ws.id == id) {
                    entry.name = workstream.title.clone();
                }

                // Update current workstream name if it was the renamed one
                if self.workstream == id {
                    self.workstream = workstream.title;
                }
            }
            Err(e) => {
                tracing::error!("Failed to rename workstream: {}", e);
                self.status_message = Some(format!("Failed to rename: {}", e));
            }
        }
    }

    /// Delete a session via API.
    async fn do_delete_session(&mut self, id: &str) {
        match self.api.sessions().delete(id).await {
            Ok(()) => {
                tracing::info!("Deleted session: {}", id);
                self.status_message = Some("Session deleted".to_string());

                // Remove from sidebar
                self.sidebar.sessions.retain(|s| s.id != id);

                // If we deleted the current session, clear it
                if self.session_id.as_deref() == Some(id) {
                    self.session_id = None;
                    self.messages.clear();
                    self.tools.clear();
                }
            }
            Err(e) => {
                tracing::error!("Failed to delete session: {}", e);
                self.status_message = Some(format!("Failed to delete session: {}", e));
            }
        }
    }

    /// Delete a workstream via API.
    async fn do_delete_workstream(&mut self, id: &str) {
        match self.api.workstreams().delete(id).await {
            Ok(()) => {
                tracing::info!("Deleted workstream: {}", id);
                self.status_message = Some("Workstream deleted".to_string());

                // Remove from sidebar
                self.sidebar.workstreams.retain(|ws| ws.name != id);

                // If we deleted the current workstream, switch to scratch
                if self.workstream == id {
                    self.workstream = "scratch".to_string();
                    self.messages.clear();
                    self.tools.clear();
                    self.session_id = None;
                }
            }
            Err(e) => {
                tracing::error!("Failed to delete workstream: {}", e);
                self.status_message = Some(format!("Failed to delete workstream: {}", e));
            }
        }
    }

    /// Fetch sessions for a specific workstream.
    async fn do_fetch_workstream_sessions(&mut self, workstream_id: &str) {
        match self.api.workstreams().sessions(workstream_id).await {
            Ok(response) => {
                self.sidebar.sessions = response
                    .sessions
                    .iter()
                    .map(|s| {
                        // Parse the started_at timestamp
                        let last_active = DateTime::parse_from_rfc3339(&s.started_at)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now());

                        // Generate a title from the date or use a default
                        let title = if s.is_active {
                            "Active session".to_string()
                        } else {
                            format!("Session {}", last_active.format("%b %d %H:%M"))
                        };

                        SessionSummary {
                            id: s.id.clone(),
                            title,
                            last_active,
                            message_count: 0, // Not available from this API
                            is_current: self.session_id.as_ref() == Some(&s.id),
                        }
                    })
                    .collect();

                // Reset to "+ New Session" selected
                self.sidebar.session_index = 0;

                // Update the session count for this workstream in the sidebar
                let session_count = self.sidebar.sessions.len();
                if let Some(ws_entry) = self
                    .sidebar
                    .workstreams
                    .iter_mut()
                    .find(|ws| ws.id == workstream_id)
                {
                    ws_entry.session_count = session_count;
                }

                tracing::info!(
                    "Loaded {} sessions for workstream {}",
                    session_count,
                    workstream_id
                );
            }
            Err(e) => {
                tracing::warn!("Failed to load sessions for workstream: {}", e);
                // Clear sessions on error
                self.sidebar.sessions.clear();
                self.sidebar.session_index = 0;
            }
        }
    }

    /// Fetch message history for a session.
    async fn do_fetch_session_messages(&mut self, session_id: &str) {
        match self.api.sessions().messages(session_id).await {
            Ok(response) => {
                // Convert API messages to ChatMessages
                let chat_messages: Vec<_> = response
                    .messages
                    .iter()
                    .map(|m| ChatMessage {
                        is_user: m.role == "user",
                        content: m.content.clone(),
                        streaming: false,
                    })
                    .collect();
                self.messages.replace_from_vec(chat_messages);

                // Scroll to bottom to show latest messages
                self.chat_auto_scroll = true;

                tracing::info!(
                    "Loaded {} messages for session {}",
                    self.messages.len(),
                    session_id
                );
            }
            Err(e) => {
                tracing::warn!("Failed to load session messages: {}", e);
                self.status_message = Some(format!("Failed to load messages: {}", e));
                // Keep messages cleared
            }
        }
    }

    /// Move a session to a different workstream via API.
    async fn do_move_session_to_workstream(&mut self, session_id: &str, workstream_id: &str) {
        use arawn_client::UpdateSessionRequest;

        tracing::info!(
            "Moving session {} to workstream {}",
            session_id,
            workstream_id
        );

        let request = UpdateSessionRequest {
            workstream_id: Some(workstream_id.to_string()),
            ..Default::default()
        };

        tracing::info!("Sending PATCH request to server...");
        match self.api.sessions().update(session_id, request).await {
            Ok(_) => {
                // Find workstream name for display
                let ws_name = self
                    .sidebar
                    .workstreams
                    .iter()
                    .find(|ws| ws.id == workstream_id)
                    .map(|ws| ws.name.clone())
                    .unwrap_or_else(|| workstream_id.to_string());

                tracing::info!("Moved session {} to workstream {}", session_id, ws_name);
                self.status_message = Some(format!("Moved session to {}", ws_name));

                // Refresh sidebar to reflect the change
                self.pending_actions.push(PendingAction::RefreshSidebar);
            }
            Err(e) => {
                tracing::error!("Failed to move session: {}", e);
                self.status_message = Some(format!("Failed to move session: {}", e));
            }
        }
    }

    /// Refresh sidebar data from the server API.
    async fn refresh_sidebar_data(&mut self) {
        // Fetch workstreams (including archived)
        match self.api.workstreams().list_all().await {
            Ok(response) => {
                self.sidebar.workstreams = response
                    .workstreams
                    .iter()
                    .map(|ws| WorkstreamEntry {
                        id: ws.id.clone(),
                        name: ws.title.clone(),
                        session_count: 0, // Updated below when loading sessions
                        is_current: ws.title == self.workstream
                            || (ws.is_scratch && self.workstream == "scratch"),
                        is_scratch: ws.is_scratch,
                        usage_bytes: None, // Updated via WebSocket events
                        limit_bytes: None,
                        state: ws.state.clone(),
                    })
                    .collect();

                // Set initial selection to current workstream and store the ID
                // Only consider active workstreams for selection
                if let Some(pos) = self
                    .sidebar
                    .workstreams
                    .iter()
                    .position(|ws| ws.is_current && !ws.is_archived())
                {
                    self.sidebar.workstream_index = pos;
                    self.workstream_id = Some(self.sidebar.workstreams[pos].id.clone());
                    self.workstream = self.sidebar.workstreams[pos].name.clone();
                }

                tracing::info!("Loaded {} workstreams", self.sidebar.workstreams.len());
            }
            Err(e) => {
                tracing::warn!("Failed to load workstreams: {}", e);
                self.status_message = Some(format!("Failed to load workstreams: {}", e));
            }
        }

        // Fetch sessions for current workstream
        if let Some(ws_id) = self.workstream_id.clone() {
            self.do_fetch_workstream_sessions(&ws_id).await;
        } else {
            // No workstream selected, clear sessions
            self.sidebar.sessions.clear();
            self.sidebar.session_index = 0;
        }
    }

    /// Handle a message from the server.
    fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::SessionCreated { session_id } => {
                self.session_id = Some(session_id);
            }

            ServerMessage::ChatChunk { chunk, done, .. } => {
                if done {
                    // Mark last message as not streaming
                    if let Some(last) = self.messages.last_mut() {
                        last.streaming = false;
                    }
                    self.waiting = false;
                } else if !chunk.is_empty() {
                    // Append to last message or create new one
                    if let Some(last) = self.messages.last_mut() {
                        if !last.is_user && last.streaming {
                            last.content.push_str(&chunk);
                            return;
                        }
                    }
                    // Create new assistant message
                    self.push_message(ChatMessage {
                        is_user: false,
                        content: chunk,
                        streaming: true,
                    });
                }
            }

            ServerMessage::ToolStart {
                tool_id, tool_name, ..
            } => {
                self.push_tool(ToolExecution {
                    id: tool_id,
                    name: tool_name,
                    args: String::new(), // Args not provided by protocol yet
                    output: String::new(),
                    running: true,
                    success: None,
                    started_at: std::time::Instant::now(),
                    duration_ms: None,
                });
                // Auto-select the new tool if tool pane is visible
                if self.show_tool_pane {
                    self.selected_tool_index = Some(self.tools.len().saturating_sub(1));
                }
            }

            ServerMessage::ToolOutput {
                tool_id, content, ..
            } => {
                if let Some(tool) = self.tools.iter_mut().find(|t| t.id == tool_id) {
                    tool.output.push_str(&content);
                }
            }

            ServerMessage::ToolEnd {
                tool_id, success, ..
            } => {
                if let Some(tool) = self.tools.iter_mut().find(|t| t.id == tool_id) {
                    tool.running = false;
                    tool.success = Some(success);
                    tool.duration_ms = Some(tool.started_at.elapsed().as_millis() as u64);
                }
            }

            ServerMessage::Error { code, message } => {
                // Handle specific error codes
                if code == "session_not_owned" {
                    // We tried to send a message but aren't the owner
                    self.is_session_owner = false;
                    self.status_message =
                        Some("Read-only mode: session owned by another client".to_string());
                } else {
                    self.status_message = Some(format!("Error: {}", message));
                }
                self.waiting = false;
            }

            ServerMessage::AuthResult { success, error } => {
                if success {
                    self.status_message = Some("Authenticated".to_string());
                } else {
                    self.status_message =
                        Some(format!("Auth failed: {}", error.unwrap_or_default()));
                }
            }

            ServerMessage::Pong => {
                // Ignore pongs
            }

            ServerMessage::CommandProgress {
                command,
                message,
                percent,
            } => {
                self.command_executing = true;
                let progress_str = match percent {
                    Some(p) => format!("/{}: {} ({}%)", command, message, p),
                    None => format!("/{}: {}", command, message),
                };
                self.command_progress = Some(progress_str.clone());
                self.status_message = Some(progress_str);
            }

            ServerMessage::CommandResult {
                command,
                success,
                result,
            } => {
                self.command_executing = false;
                self.command_progress = None;

                if success {
                    // Format the result as a system message
                    let result_str =
                        if let Some(msg) = result.get("message").and_then(|v| v.as_str()) {
                            msg.to_string()
                        } else {
                            serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "Success".to_string())
                        };
                    self.status_message = Some(format!("/{}: {}", command, result_str));

                    // Add as system message in chat
                    self.push_message(ChatMessage {
                        is_user: false,
                        content: format!("[/{}] {}", command, result_str),
                        streaming: false,
                    });
                } else {
                    let error_str = result
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    self.status_message = Some(format!("/{} failed: {}", command, error_str));
                }
            }

            ServerMessage::ContextInfo {
                current_tokens,
                max_tokens,
                percent,
                status,
                ..
            } => {
                self.context_info = Some(ContextState {
                    current_tokens,
                    max_tokens,
                    percent,
                    status,
                });
            }

            ServerMessage::DiskPressure {
                workstream_id,
                workstream_name,
                level,
                usage_bytes,
                limit_bytes,
                percent,
            } => {
                // Add warning, replacing any existing warning for same workstream
                self.disk_warnings.retain(|w| w.workstream != workstream_id);
                self.disk_warnings.push(DiskWarning {
                    workstream: workstream_name.clone(),
                    level,
                    usage_bytes,
                    limit_bytes,
                    percent,
                    timestamp: std::time::Instant::now(),
                });

                // Show status message for critical warnings
                if self.disk_warnings.last().map(|w| w.level.as_str()) == Some("critical") {
                    self.status_message = Some(format!(
                        "⚠ Disk critical: {} at {}% of limit",
                        workstream_name, percent
                    ));
                }
            }

            ServerMessage::WorkstreamUsage {
                workstream_id,
                workstream_name,
                is_scratch,
                production_bytes,
                work_bytes,
                total_bytes,
                limit_bytes,
                percent,
            } => {
                // Update usage stats if it's for the current workstream
                if self.workstream_id.as_deref() == Some(&workstream_id) {
                    self.workstream_usage = Some(UsageStats {
                        workstream_id,
                        workstream_name,
                        is_scratch,
                        production_bytes,
                        work_bytes,
                        total_bytes,
                        limit_bytes,
                        percent,
                    });
                }
            }

            ServerMessage::SubscribeAck {
                session_id,
                owner,
                reconnect_token,
            } => {
                // Update ownership state
                self.is_session_owner = owner;

                // Store reconnect token if we're the owner
                if let Some(token) = reconnect_token {
                    self.reconnect_tokens.insert(session_id.clone(), token);
                }

                if owner {
                    tracing::info!(session_id = %session_id, "Subscribed as owner");
                } else {
                    tracing::info!(session_id = %session_id, "Subscribed as reader (read-only)");
                    self.status_message = Some("Read-only mode".to_string());
                }
            }
        }
    }

    /// Handle keyboard input.
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        // Global shortcuts first
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('c') => {
                    // Cancel current operation or quit if nothing running
                    if self.waiting {
                        // Send cancel to server if we have a session
                        if let Some(ref session_id) = self.session_id {
                            if let Err(e) = self.ws_client.cancel(session_id.clone()) {
                                tracing::warn!(error = %e, "Failed to send cancel to server");
                            }
                        }
                        self.waiting = false;
                        self.status_message = Some("Cancelled".to_string());
                    } else {
                        self.should_quit = true;
                    }
                    return;
                }
                KeyCode::Char('k') => {
                    self.palette.reset();
                    self.focus.push_overlay(FocusTarget::CommandPalette);
                    return;
                }
                KeyCode::Char('s') => {
                    self.open_sessions_panel();
                    return;
                }
                KeyCode::Char('w') => {
                    // Toggle sidebar open/closed
                    if self.sidebar.is_open() {
                        // Close sidebar, return focus to input
                        self.sidebar.close();
                        self.focus.return_to_input();
                    } else {
                        // Open sidebar and focus it
                        self.sidebar.open();
                        self.focus.focus(FocusTarget::Sidebar);
                    }
                    return;
                }
                KeyCode::Char('e') => {
                    self.show_tool_pane = !self.show_tool_pane;
                    if self.show_tool_pane {
                        // Select first tool if none selected
                        if self.selected_tool_index.is_none() && !self.tools.is_empty() {
                            self.selected_tool_index = Some(0);
                        }
                        self.focus.focus(FocusTarget::ToolPane);
                    } else {
                        self.selected_tool_index = None;
                        self.focus.return_to_input();
                    }
                    return;
                }
                KeyCode::Char('o') if self.show_tool_pane => {
                    // Open selected tool output in external editor
                    self.open_tool_in_editor();
                    return;
                }
                KeyCode::Char('l') => {
                    // Toggle logs panel
                    self.show_logs = !self.show_logs;
                    if self.show_logs {
                        self.focus.focus(FocusTarget::Logs);
                    } else {
                        self.focus.return_to_input();
                    }
                    return;
                }
                KeyCode::Char('u') => {
                    // Toggle usage stats popup
                    self.show_usage_popup = !self.show_usage_popup;
                    return;
                }
                _ => {}
            }
        }

        // Delegate to focused component
        match self.focus.current() {
            FocusTarget::Input => self.handle_input_key(key),
            FocusTarget::Sidebar => self.handle_sidebar_key(key),
            FocusTarget::Sessions => self.handle_sessions_key(key),
            FocusTarget::CommandPalette => self.handle_palette_key(key),
            FocusTarget::Workstreams => self.handle_overlay_key(key),
            FocusTarget::ToolPane => self.handle_tool_pane_key(key),
            FocusTarget::Logs => self.handle_logs_key(key),
        }
    }

    /// Handle input-focused key events.
    fn handle_input_key(&mut self, key: crossterm::event::KeyEvent) {
        let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Check if command popup is visible and handle its keys
        if self.command_popup.is_visible() {
            match key.code {
                KeyCode::Esc => {
                    self.command_popup.hide();
                    return;
                }
                KeyCode::Up => {
                    self.command_popup.select_prev();
                    return;
                }
                KeyCode::Down => {
                    self.command_popup.select_next();
                    return;
                }
                KeyCode::Tab | KeyCode::Enter => {
                    // Complete the selected command
                    if let Some(cmd) = self.command_popup.selected_command() {
                        let cmd_name = cmd.name.clone();
                        self.input.set_text(&format!("/{} ", cmd_name));
                        self.command_popup.hide();
                    }
                    return;
                }
                KeyCode::Char(c) => {
                    // Continue typing and update filter
                    self.input.insert_char(c);
                    self.update_command_popup();
                    return;
                }
                KeyCode::Backspace => {
                    self.input.delete_char_before();
                    self.update_command_popup();
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char(c) => {
                self.input.insert_char(c);
                // Show command popup when typing '/'
                if c == '/' && self.input.content().trim() == "/" {
                    self.command_popup.show("");
                } else {
                    self.update_command_popup();
                }
            }
            KeyCode::Backspace => {
                self.input.delete_char_before();
                self.update_command_popup();
            }
            KeyCode::Delete => {
                self.input.delete_char_at();
            }
            KeyCode::Left => {
                self.input.move_left();
            }
            KeyCode::Right => {
                self.input.move_right();
            }
            KeyCode::Home => {
                if has_ctrl {
                    // Ctrl+Home: scroll to top of chat
                    self.chat_scroll = 0;
                    self.chat_auto_scroll = false;
                } else {
                    self.input.move_to_line_start();
                }
            }
            KeyCode::End => {
                if has_ctrl {
                    // Ctrl+End: scroll to bottom and enable auto-scroll
                    self.chat_auto_scroll = true;
                } else {
                    self.input.move_to_line_end();
                }
            }
            KeyCode::Enter => {
                if has_shift {
                    // Shift+Enter: insert newline
                    self.input.insert_newline();
                } else if !self.input.is_empty() {
                    // Hide command popup
                    self.command_popup.hide();

                    // Handle based on input mode
                    match &self.input_mode {
                        InputMode::Chat => {
                            if !self.waiting && !self.command_executing {
                                // Check if this is a command
                                if self.input.is_command() {
                                    self.send_command();
                                } else {
                                    self.send_message();
                                }
                            }
                        }
                        InputMode::NewWorkstream => {
                            let title = self.input.content().trim().to_string();

                            // Validation
                            if title.is_empty() {
                                self.status_message =
                                    Some("Workstream name cannot be empty".to_string());
                                return;
                            }
                            if title.len() > 100 {
                                self.status_message =
                                    Some("Workstream name too long (max 100 chars)".to_string());
                                return;
                            }
                            // Check for duplicate names
                            let name_exists = self
                                .sidebar
                                .workstreams
                                .iter()
                                .any(|ws| ws.name.eq_ignore_ascii_case(&title));
                            if name_exists {
                                self.status_message =
                                    Some(format!("Workstream '{}' already exists", title));
                                return;
                            }

                            self.pending_actions
                                .push(PendingAction::CreateWorkstream(title));
                            self.input.clear();
                            self.input_mode = InputMode::Chat;
                            self.status_message = None;
                        }
                        InputMode::RenameWorkstream(id) => {
                            let new_title = self.input.content().to_string();
                            let id = id.clone();
                            self.pending_actions
                                .push(PendingAction::RenameWorkstream(id, new_title));
                            self.input.clear();
                            self.input_mode = InputMode::Chat;
                            self.status_message = None;
                        }
                    }
                }
            }
            KeyCode::Esc => {
                // Close usage popup if open
                if self.show_usage_popup {
                    self.show_usage_popup = false;
                    return;
                }
                // Cancel special mode or clear input
                if self.input_mode != InputMode::Chat {
                    self.input_mode = InputMode::Chat;
                    self.input.clear();
                    self.status_message = None;
                } else if !self.input.is_empty() {
                    self.input.clear();
                }
            }
            // History navigation with Up/Down (when on single line or at boundaries)
            KeyCode::Up => {
                let (line, _) = self.input.cursor_position();
                if self.input.is_empty() {
                    // Empty input: scroll chat
                    self.scroll_chat_up(1);
                } else if line == 0 {
                    // At first line: navigate history
                    self.input.history_prev();
                } else {
                    // Multi-line: move cursor up
                    self.input.move_up();
                }
            }
            KeyCode::Down => {
                let (line, _) = self.input.cursor_position();
                let last_line = self.input.line_count().saturating_sub(1);
                if self.input.is_empty() {
                    // Empty input: scroll chat
                    self.scroll_chat_down(1);
                } else if line >= last_line {
                    // At last line: navigate history
                    self.input.history_next();
                } else {
                    // Multi-line: move cursor down
                    self.input.move_down();
                }
            }
            KeyCode::PageUp => {
                self.scroll_chat_up(10);
            }
            KeyCode::PageDown => {
                self.scroll_chat_down(10);
            }
            _ => {}
        }
    }

    /// Scroll chat up by the given number of lines.
    ///
    /// Disables auto-scroll so the user can read history without
    /// being snapped back to the bottom during streaming. Auto-scroll
    /// is only re-enabled when the user sends a new message.
    fn scroll_chat_up(&mut self, lines: usize) {
        self.chat_auto_scroll = false;
        self.chat_scroll = self.chat_scroll.saturating_sub(lines);
    }

    /// Scroll chat down by the given number of lines.
    ///
    /// Disables auto-scroll to preserve manual scroll position.
    /// Use Ctrl+End or send a message to re-enable auto-scroll.
    fn scroll_chat_down(&mut self, lines: usize) {
        self.chat_auto_scroll = false;
        self.chat_scroll = self.chat_scroll.saturating_add(lines);
        // Note: actual clamping happens in render_chat based on content height
    }

    /// Update the command popup based on current input.
    fn update_command_popup(&mut self) {
        if let Some(prefix) = self.input.command_prefix() {
            if !self.command_popup.is_visible() {
                self.command_popup.show(prefix);
            } else {
                self.command_popup.filter(prefix);
            }
        } else {
            self.command_popup.hide();
        }
    }

    /// Send the current input as a command.
    fn send_command(&mut self) {
        let input = self.input.submit();

        // Parse the command
        if let Some(cmd) = crate::input::ParsedCommand::parse(&input) {
            // Handle built-in commands
            if cmd.name.eq_ignore_ascii_case("help") {
                // Show available commands in chat
                let help_text = self.get_help_text();
                self.push_message(ChatMessage {
                    is_user: false,
                    content: help_text,
                    streaming: false,
                });
                return;
            }

            // Check read-only mode for server commands
            if !self.is_session_owner {
                self.status_message = Some("Read-only mode: cannot run commands".to_string());
                return;
            }

            // Build args JSON
            let args = self.build_command_args(&cmd);

            // Send command via WebSocket
            if let Err(e) = self.ws_client.send_command(cmd.name.clone(), args) {
                self.status_message = Some(format!("Failed to send command: {}", e));
                return;
            }

            self.command_executing = true;
            self.status_message = Some(format!("Executing /{}", cmd.name));
        } else {
            self.status_message = Some("Invalid command".to_string());
        }
    }

    /// Build command arguments JSON from parsed command.
    fn build_command_args(&self, cmd: &crate::input::ParsedCommand) -> serde_json::Value {
        let mut args = serde_json::json!({});

        // Always include session_id if available
        if let Some(ref sid) = self.session_id {
            args["session_id"] = serde_json::Value::String(sid.clone());
        }

        // Parse additional args (simple key=value or flags)
        for part in cmd.args.split_whitespace() {
            if let Some(flag) = part.strip_prefix("--") {
                if let Some((key, value)) = flag.split_once('=') {
                    // --key=value
                    args[key] = serde_json::Value::String(value.to_string());
                } else {
                    // --flag (boolean true)
                    args[flag] = serde_json::Value::Bool(true);
                }
            } else if part == "-f" || part == "--force" {
                args["force"] = serde_json::Value::Bool(true);
            }
        }

        args
    }

    /// Get help text for available commands.
    fn get_help_text(&self) -> String {
        let mut text = String::from("**Available Commands:**\n\n");
        text.push_str("/compact - Compact session history by summarizing older turns\n");
        text.push_str("  Options: --force, -f (force compaction even if not needed)\n\n");
        text.push_str("/help - Show this help message\n");
        text
    }

    /// Send the current input as a chat message.
    fn send_message(&mut self) {
        // Check read-only mode
        if !self.is_session_owner {
            self.status_message = Some("Read-only mode: cannot send messages".to_string());
            return;
        }

        // Submit and get the message (also adds to history)
        let message = self.input.submit();

        // Add user message to display
        self.push_message(ChatMessage {
            is_user: true,
            content: message.clone(),
            streaming: false,
        });

        // Clear tools from previous response
        self.tools.clear();

        // Enable auto-scroll when sending
        self.chat_auto_scroll = true;

        // Send via WebSocket with workstream context
        if let Err(e) =
            self.ws_client
                .send_chat(message, self.session_id.clone(), self.workstream_id.clone())
        {
            self.status_message = Some(format!("Failed to send: {}", e));
            return;
        }

        self.waiting = true;
        self.status_message = None;
    }

    /// Handle sessions overlay key events.
    fn handle_sessions_key(&mut self, key: crossterm::event::KeyEvent) {
        let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Esc => {
                self.sessions.reset();
                self.focus.pop_overlay();
            }
            KeyCode::Enter => {
                // Select the current session
                if let Some(session) = self.sessions.selected_session() {
                    let session_id = session.id.clone();
                    self.switch_to_session(&session_id);
                }
                self.sessions.reset();
                self.focus.pop_overlay();
            }
            KeyCode::Up => {
                self.sessions.select_prev();
            }
            KeyCode::Down => {
                self.sessions.select_next();
            }
            KeyCode::Home => {
                self.sessions.select_first();
            }
            KeyCode::End => {
                self.sessions.select_last();
            }
            KeyCode::Char('n') if has_ctrl => {
                // Create new session
                self.create_new_session();
                self.sessions.reset();
                self.focus.pop_overlay();
            }
            KeyCode::Char(c) => {
                // Add to filter
                self.sessions.filter_push(c);
            }
            KeyCode::Backspace => {
                self.sessions.filter_pop();
            }
            _ => {}
        }
    }

    /// Handle command palette key events.
    fn handle_palette_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.palette.reset();
                self.focus.pop_overlay();
            }
            KeyCode::Enter => {
                // Execute selected action
                if let Some(action) = self.palette.selected_action() {
                    let action_id = action.id;
                    self.palette.reset();
                    self.focus.pop_overlay();
                    self.execute_action(action_id);
                } else {
                    self.palette.reset();
                    self.focus.pop_overlay();
                }
            }
            KeyCode::Up => {
                self.palette.select_prev();
            }
            KeyCode::Down => {
                self.palette.select_next();
            }
            KeyCode::Home => {
                self.palette.select_first();
            }
            KeyCode::End => {
                self.palette.select_last();
            }
            KeyCode::Char(c) => {
                self.palette.filter_push(c);
            }
            KeyCode::Backspace => {
                self.palette.filter_pop();
            }
            _ => {}
        }
    }

    /// Execute a palette action.
    fn execute_action(&mut self, action_id: ActionId) {
        match action_id {
            ActionId::SessionsSwitch => {
                self.open_sessions_panel();
            }
            ActionId::SessionsNew => {
                self.create_new_session();
            }
            ActionId::SessionsDelete => {
                self.sidebar.open();
                self.sidebar.section = SidebarSection::Sessions;
                self.focus.focus(FocusTarget::Sidebar);
                self.status_message = Some("Select a session and press 'd' to delete".to_string());
            }
            ActionId::SessionsMoveToWorkstream => {
                if let Some(ref sid) = self.session_id {
                    tracing::info!("Session move initiated for session: {}", sid);
                    // Open sidebar in workstreams section for selection
                    self.sidebar.open();
                    self.sidebar.section = SidebarSection::Workstreams;
                    self.moving_session_to_workstream = true;
                    self.focus.focus(FocusTarget::Sidebar);
                    self.status_message =
                        Some("Select target workstream (Enter to move, Esc to cancel)".to_string());
                } else {
                    tracing::warn!("Session move attempted with no active session");
                    self.status_message = Some("No session to move".to_string());
                }
            }
            ActionId::WorkstreamsSwitch => {
                self.focus.push_overlay(FocusTarget::Workstreams);
            }
            ActionId::WorkstreamsCreate => {
                // Enter new workstream name mode
                self.input_mode = InputMode::NewWorkstream;
                self.input.clear();
                self.focus.return_to_input();
                self.status_message =
                    Some("New workstream: Enter name (Esc to cancel)".to_string());
            }
            ActionId::ViewToggleToolPane => {
                self.focus.toggle(FocusTarget::ToolPane);
            }
            ActionId::AppQuit => {
                self.should_quit = true;
            }
        }
    }

    /// Switch to a different session.
    fn switch_to_session(&mut self, session_id: &str) {
        // Subscribe to the new session FIRST to avoid missing messages
        // that might arrive between subscribe and fetch
        // Use reconnect token if we have one to reclaim ownership
        let reconnect_token = self.reconnect_tokens.get(session_id).cloned();
        if let Err(e) = self
            .ws_client
            .subscribe(session_id.to_string(), reconnect_token)
        {
            self.status_message = Some(format!("Failed to switch session: {}", e));
            return; // Don't clear state if we failed to subscribe
        }

        // Reset ownership state - will be updated by SubscribeAck
        self.is_session_owner = false;

        // Now clear current messages and tools
        self.messages.clear();
        self.tools.clear();
        self.session_id = Some(session_id.to_string());
        self.sessions.set_current(session_id);
        self.sidebar.set_current_session(session_id);
        self.chat_scroll = 0;
        self.chat_auto_scroll = true;

        self.status_message = Some("Loading session...".to_string());

        // Queue fetch of message history
        self.pending_actions
            .push(PendingAction::FetchSessionMessages(session_id.to_string()));
    }

    /// Create a new session.
    fn create_new_session(&mut self) {
        self.messages.clear();
        self.tools.clear();
        self.session_id = None; // Will be assigned by server on first message
        self.chat_scroll = 0;
        self.chat_auto_scroll = true;
        self.status_message = Some("New session created".to_string());
    }

    /// Open the sessions panel.
    fn open_sessions_panel(&mut self) {
        self.sessions.reset();
        self.focus.push_overlay(FocusTarget::Sessions);

        // Use sessions from sidebar (already loaded from API)
        self.sessions.set_items(self.sidebar.sessions.clone());
    }

    /// Handle overlay (workstreams/palette) key events.
    fn handle_overlay_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.focus.pop_overlay();
            }
            KeyCode::Enter => {
                // TODO: Select item
                self.focus.pop_overlay();
            }
            KeyCode::Up | KeyCode::Down => {
                // TODO: Navigate list
            }
            KeyCode::Char(c) => {
                // TODO: Filter list
                let _ = c;
            }
            _ => {}
        }
    }

    /// Handle tool pane key events.
    fn handle_tool_pane_key(&mut self, key: crossterm::event::KeyEvent) {
        let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Esc => {
                self.show_tool_pane = false;
                self.selected_tool_index = None;
                self.focus.return_to_input();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // Navigate to previous tool
                if !self.tools.is_empty() {
                    let current = self.selected_tool_index.unwrap_or(0);
                    self.selected_tool_index = Some(current.saturating_sub(1));
                    self.tool_scroll = 0; // Reset scroll when changing tools
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                // Navigate to next tool
                if !self.tools.is_empty() {
                    let current = self.selected_tool_index.unwrap_or(0);
                    let max_idx = self.tools.len().saturating_sub(1);
                    self.selected_tool_index = Some((current + 1).min(max_idx));
                    self.tool_scroll = 0; // Reset scroll when changing tools
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Scroll output up
                self.tool_scroll = self.tool_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Scroll output down
                self.tool_scroll = self.tool_scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.tool_scroll = self.tool_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.tool_scroll = self.tool_scroll.saturating_add(10);
            }
            KeyCode::Home => {
                if has_ctrl {
                    // Ctrl+Home: go to first tool
                    if !self.tools.is_empty() {
                        self.selected_tool_index = Some(0);
                        self.tool_scroll = 0;
                    }
                } else {
                    self.tool_scroll = 0;
                }
            }
            KeyCode::End => {
                if has_ctrl {
                    // Ctrl+End: go to last tool
                    if !self.tools.is_empty() {
                        self.selected_tool_index = Some(self.tools.len() - 1);
                        self.tool_scroll = 0;
                    }
                } else {
                    // Scroll to end - will be clamped in render
                    self.tool_scroll = usize::MAX;
                }
            }
            KeyCode::Char('o') if has_ctrl => {
                // Ctrl+O: open in external editor
                self.open_tool_in_editor();
            }
            _ => {}
        }
    }

    /// Open the selected tool's output in an external pager.
    ///
    /// This suspends the TUI, runs the pager (e.g., `less`), and restores
    /// the TUI when the user exits the pager.
    fn open_tool_in_editor(&mut self) {
        let Some(idx) = self.selected_tool_index else {
            self.status_message = Some("No tool selected".to_string());
            return;
        };
        let Some(tool) = self.tools.get(idx) else {
            self.status_message = Some("Tool not found".to_string());
            return;
        };

        if tool.output.is_empty() {
            self.status_message = Some("Tool has no output".to_string());
            return;
        }

        // Get pager from environment (prefer PAGER for viewing)
        let pager = std::env::var("PAGER")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| "less".to_string());

        let output = tool.output.clone();
        let tool_name = tool.name.clone();

        // Run pager synchronously, suspending the TUI
        if let Err(e) = self.run_pager(&pager, &output) {
            self.status_message = Some(format!("Failed to open pager: {}", e));
        } else {
            self.status_message = Some(format!("Viewed {} output in {}", tool_name, pager));
        }
    }

    /// Run a pager with the given content, suspending and restoring the TUI.
    fn run_pager(&self, pager: &str, content: &str) -> std::io::Result<()> {
        use crossterm::{
            execute,
            terminal::{
                EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
            },
        };
        use std::io::Write;

        // Write content to temp file
        let mut tmp = tempfile::NamedTempFile::new()?;
        tmp.write_all(content.as_bytes())?;
        tmp.flush()?;

        // Suspend TUI
        disable_raw_mode()?;
        execute!(std::io::stdout(), LeaveAlternateScreen)?;

        // Run pager (blocks until user quits)
        let status = std::process::Command::new(pager).arg(tmp.path()).status();

        // Restore TUI (even if pager failed)
        execute!(std::io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        // Check pager result
        match status {
            Ok(exit) if exit.success() => Ok(()),
            Ok(exit) => Err(std::io::Error::other(format!(
                "Pager exited with status: {}",
                exit
            ))),
            Err(e) => Err(e),
        }
    }

    /// Handle logs panel key events.
    fn handle_logs_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.show_logs = false;
                self.focus.return_to_input();
            }
            KeyCode::Up => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.log_scroll = self.log_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.log_scroll = self.log_scroll.saturating_add(10);
            }
            KeyCode::Home => {
                self.log_scroll = 0;
            }
            KeyCode::End => {
                // Scroll to end - will be clamped in render
                self.log_scroll = usize::MAX;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Clear logs
                self.log_buffer.clear();
                self.log_scroll = 0;
            }
            _ => {}
        }
    }

    /// Clear any pending delete confirmations.
    fn clear_pending_deletes(&mut self) {
        self.pending_delete_workstream = None;
        self.pending_delete_session = None;
    }

    /// Handle sidebar key events.
    fn handle_sidebar_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Right => {
                // Close sidebar and return focus to input
                self.sidebar.close();
                self.moving_session_to_workstream = false;
                self.clear_pending_deletes();
                self.focus.return_to_input();
            }
            KeyCode::Tab => {
                // Switch between workstreams and sessions sections
                self.sidebar.toggle_section();
                self.clear_pending_deletes();
            }
            KeyCode::Up => {
                self.clear_pending_deletes();
                if let Some(ws_id) = self.sidebar.select_prev() {
                    // Workstream selection changed, fetch sessions from API
                    self.pending_actions
                        .push(PendingAction::FetchWorkstreamSessions(ws_id));
                }
            }
            KeyCode::Down => {
                self.clear_pending_deletes();
                if let Some(ws_id) = self.sidebar.select_next() {
                    // Workstream selection changed, fetch sessions from API
                    self.pending_actions
                        .push(PendingAction::FetchWorkstreamSessions(ws_id));
                }
            }
            KeyCode::Enter => {
                self.clear_pending_deletes();
                // Select current item
                match self.sidebar.section {
                    SidebarSection::Workstreams => {
                        if self.moving_session_to_workstream {
                            // Move current session to selected workstream
                            if let (Some(session_id), Some(ws)) =
                                (self.session_id.clone(), self.sidebar.selected_workstream())
                            {
                                tracing::info!(
                                    "User confirmed move: session {} -> workstream {} ({})",
                                    session_id,
                                    ws.name,
                                    ws.id
                                );
                                let ws_id = ws.id.clone();
                                self.pending_actions
                                    .push(PendingAction::MoveSessionToWorkstream(
                                        session_id, ws_id,
                                    ));
                            } else {
                                tracing::warn!(
                                    "Move confirmed but session_id or workstream not available"
                                );
                            }
                            self.moving_session_to_workstream = false;
                            self.sidebar.close();
                            self.focus.return_to_input();
                        } else {
                            // Switch to selected workstream
                            if let Some(ws) = self.sidebar.selected_workstream() {
                                let ws_name = ws.name.clone();
                                self.switch_to_workstream(&ws_name);
                            }
                        }
                    }
                    SidebarSection::Sessions => {
                        // First, ensure we're in the selected workstream
                        if let Some(ws) = self.sidebar.selected_workstream() {
                            if !ws.is_current {
                                let ws_name = ws.name.clone();
                                self.switch_to_workstream(&ws_name);
                            }
                        }

                        if self.sidebar.is_new_session_selected() {
                            // Create new session in the (now current) workstream
                            self.create_new_session();
                            self.sidebar.close();
                            self.focus.return_to_input();
                        } else if let Some(session) = self.sidebar.selected_session() {
                            // Switch to selected session
                            let session_id = session.id.clone();
                            self.switch_to_session(&session_id);
                            self.sidebar.close();
                            self.focus.return_to_input();
                        }
                    }
                }
            }
            KeyCode::Char('n') => {
                self.clear_pending_deletes();
                // Create new item in current section
                match self.sidebar.section {
                    SidebarSection::Workstreams => {
                        // Enter new workstream name mode
                        self.input_mode = InputMode::NewWorkstream;
                        self.input.clear();
                        self.sidebar.close();
                        self.focus.return_to_input();
                        self.status_message =
                            Some("New workstream: Enter name (Esc to cancel)".to_string());
                    }
                    SidebarSection::Sessions => {
                        // Switch to selected workstream if different
                        if let Some(ws) = self.sidebar.selected_workstream() {
                            if !ws.is_current {
                                let ws_name = ws.name.clone();
                                self.switch_to_workstream(&ws_name);
                            }
                        }
                        // Create new session in the (now current) workstream
                        self.create_new_session();
                        self.sidebar.close();
                        self.focus.return_to_input();
                    }
                }
            }
            KeyCode::Char('N') => {
                self.clear_pending_deletes();
                // Create new workstream regardless of current section
                self.input_mode = InputMode::NewWorkstream;
                self.input.clear();
                self.sidebar.close();
                self.focus.return_to_input();
                self.status_message =
                    Some("New workstream: Enter name (Esc to cancel)".to_string());
            }
            KeyCode::Char('r') => {
                self.clear_pending_deletes();
                // Rename selected workstream
                if self.sidebar.section == SidebarSection::Workstreams {
                    if let Some(ws) = self.sidebar.selected_workstream() {
                        let name = ws.name.clone();
                        self.input_mode = InputMode::RenameWorkstream(name.clone());
                        self.input.clear();
                        self.input.set_text(&name); // Pre-fill with current name
                        self.sidebar.close();
                        self.focus.return_to_input();
                        self.status_message = Some("Rename workstream (Esc to cancel)".to_string());
                    }
                } else {
                    self.status_message = Some("Select a workstream to rename".to_string());
                }
            }
            KeyCode::Char('d') => {
                // Delete current item (requires confirmation - press 'd' twice)
                match self.sidebar.section {
                    SidebarSection::Workstreams => {
                        if let Some(ws) = self.sidebar.selected_workstream() {
                            // Check if this is a confirmation (second 'd' press)
                            if let Some((pending_id, _)) = &self.pending_delete_workstream {
                                if pending_id == &ws.id {
                                    // Confirmed - execute delete
                                    let id = ws.id.clone();
                                    self.pending_actions
                                        .push(PendingAction::DeleteWorkstream(id));
                                    self.clear_pending_deletes();
                                    return;
                                }
                            }

                            // First 'd' press - check if deletable and show confirmation
                            if ws.is_scratch {
                                self.status_message =
                                    Some("Cannot delete scratch workstream".to_string());
                            } else if ws.is_current {
                                self.status_message =
                                    Some("Cannot delete current workstream".to_string());
                            } else {
                                // Set pending and show confirmation message
                                let name = ws.name.clone();
                                let id = ws.id.clone();
                                self.pending_delete_workstream = Some((id, name.clone()));
                                self.status_message = Some(format!(
                                    "Delete '{}'? Press 'd' again to confirm, Esc to cancel",
                                    name
                                ));
                            }
                        }
                    }
                    SidebarSection::Sessions => {
                        if let Some(session) = self.sidebar.selected_session() {
                            // Check if this is a confirmation (second 'd' press)
                            if let Some(pending_id) = &self.pending_delete_session {
                                if pending_id == &session.id {
                                    // Confirmed - execute delete
                                    let id = session.id.clone();
                                    self.pending_actions.push(PendingAction::DeleteSession(id));
                                    self.clear_pending_deletes();
                                    return;
                                }
                            }

                            // First 'd' press - check if deletable and show confirmation
                            if session.is_current {
                                self.status_message =
                                    Some("Cannot delete current session".to_string());
                            } else {
                                // Set pending and show confirmation message
                                let id = session.id.clone();
                                self.pending_delete_session = Some(id);
                                self.status_message = Some(
                                    "Delete session? Press 'd' again to confirm, Esc to cancel"
                                        .to_string(),
                                );
                            }
                        }
                    }
                }
            }
            KeyCode::Char('/') => {
                // Start filtering - clear existing filter and start fresh
                self.sidebar.filter_clear();
                self.status_message =
                    Some("Filter: type to search (Backspace to clear)".to_string());
            }
            KeyCode::Char(c) => {
                // Add to filter for incremental search
                self.sidebar.filter_push(c);
                if !self.sidebar.filter.is_empty() {
                    self.status_message = Some(format!("Filter: {}", self.sidebar.filter));
                }
            }
            KeyCode::Backspace => {
                self.sidebar.filter_pop();
                if self.sidebar.filter.is_empty() {
                    self.status_message = None;
                } else {
                    self.status_message = Some(format!("Filter: {}", self.sidebar.filter));
                }
            }
            _ => {}
        }
    }

    /// Switch to a different workstream.
    fn switch_to_workstream(&mut self, workstream_name: &str) {
        self.workstream = workstream_name.to_string();

        // Mark the new workstream as current in sidebar and get the ID
        let mut new_workstream_id = None;
        for ws in &mut self.sidebar.workstreams {
            ws.is_current = ws.name == workstream_name;
            if ws.is_current {
                new_workstream_id = Some(ws.id.clone());
            }
        }
        self.workstream_id = new_workstream_id;

        // Clear current session since we're switching workstreams
        self.messages.clear();
        self.tools.clear();
        self.session_id = None;
        self.chat_scroll = 0;
        self.chat_auto_scroll = true;

        // Clear usage stats (will be updated via WebSocket)
        self.workstream_usage = None;

        // Queue fetch of sessions for this workstream
        if let Some(ref ws_id) = self.workstream_id {
            self.pending_actions
                .push(PendingAction::FetchWorkstreamSessions(ws_id.clone()));
        } else {
            // No workstream ID, clear sessions
            self.sidebar.sessions.clear();
            self.sidebar.session_index = 0;
        }

        self.status_message = Some(format!("Switched to workstream: {}", workstream_name));
    }
}
