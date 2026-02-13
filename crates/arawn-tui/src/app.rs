//! Application state and main loop.

use crate::bounded::BoundedVec;
use crate::client::{ConnectionStatus, WsClient};

/// Maximum number of chat messages to retain (prevents unbounded memory growth).
const MAX_MESSAGES: usize = 10_000;

/// Maximum number of tool executions to retain per response.
const MAX_TOOLS: usize = 1_000;

use crate::events::{Event, EventHandler};
use crate::input::InputState;
use crate::logs::LogBuffer;
use crate::palette::{ActionId, CommandPalette};
use crate::protocol::ServerMessage;
use crate::sessions::{SessionList, SessionSummary};
use crate::sidebar::{Sidebar, SidebarSection, WorkstreamEntry};
use crate::ui;
use crate::Tui;
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

/// Focus determines which component handles input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    /// Default - typing messages
    #[default]
    Input,
    /// Sidebar navigation (workstreams + sessions)
    Sidebar,
    /// Session list overlay (legacy, for command palette)
    Sessions,
    /// Workstream list overlay (legacy, for command palette)
    Workstreams,
    /// Command palette (Ctrl+K)
    CommandPalette,
    /// Tool output pane (Ctrl+E)
    ToolPane,
    /// Logs panel (Ctrl+L)
    Logs,
}

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
    /// Accumulated output.
    pub output: String,
    /// Whether the tool is still running.
    pub running: bool,
    /// Whether the tool succeeded (None if still running).
    pub success: Option<bool>,
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
    /// Current focus state.
    pub focus: Focus,
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
    /// Pending async actions to process.
    pending_actions: Vec<PendingAction>,
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
            focus: Focus::default(),
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
            pending_actions: Vec::new(),
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
                    tracing::info!("Processing pending action: MoveSessionToWorkstream({}, {})", session_id, workstream_id);
                    self.do_move_session_to_workstream(&session_id, &workstream_id).await;
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
                tracing::info!("Created workstream: {} ({})", workstream.title, workstream.id);
                self.status_message = Some(format!("Created workstream: {}", workstream.title));

                // Add to sidebar and switch to it
                self.sidebar.workstreams.push(WorkstreamEntry {
                    id: workstream.id.clone(),
                    name: workstream.title.clone(),
                    session_count: 0,
                    is_current: false,
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
        // Fetch workstreams
        match self.api.workstreams().list().await {
            Ok(response) => {
                self.sidebar.workstreams = response
                    .workstreams
                    .iter()
                    .map(|ws| WorkstreamEntry {
                        id: ws.id.clone(),
                        name: ws.title.clone(),
                        session_count: 0, // Updated below when loading sessions
                        is_current: ws.title == self.workstream || (ws.is_scratch && self.workstream == "scratch"),
                    })
                    .collect();

                // Set initial selection to current workstream and store the ID
                if let Some(pos) = self.sidebar.workstreams.iter().position(|ws| ws.is_current) {
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

            ServerMessage::ChatChunk {
                chunk,
                done,
                ..
            } => {
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
                    output: String::new(),
                    running: true,
                    success: None,
                });
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
                }
            }

            ServerMessage::Error { message, .. } => {
                self.status_message = Some(format!("Error: {}", message));
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
                    self.focus = Focus::CommandPalette;
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
                        self.focus = Focus::Input;
                    } else {
                        // Open sidebar and focus it
                        self.sidebar.open();
                        self.focus = Focus::Sidebar;
                    }
                    return;
                }
                KeyCode::Char('e') => {
                    self.focus = if self.focus == Focus::ToolPane {
                        Focus::Input
                    } else {
                        Focus::ToolPane
                    };
                    return;
                }
                KeyCode::Char('l') => {
                    // Toggle logs panel
                    self.show_logs = !self.show_logs;
                    if self.show_logs {
                        self.focus = Focus::Logs;
                    } else {
                        self.focus = Focus::Input;
                    }
                    return;
                }
                _ => {}
            }
        }

        // Delegate to focused component
        match self.focus {
            Focus::Input => self.handle_input_key(key),
            Focus::Sidebar => self.handle_sidebar_key(key),
            Focus::Sessions => self.handle_sessions_key(key),
            Focus::CommandPalette => self.handle_palette_key(key),
            Focus::Workstreams => self.handle_overlay_key(key),
            Focus::ToolPane => self.handle_tool_pane_key(key),
            Focus::Logs => self.handle_logs_key(key),
        }
    }

    /// Handle input-focused key events.
    fn handle_input_key(&mut self, key: crossterm::event::KeyEvent) {
        let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Char(c) => {
                self.input.insert_char(c);
            }
            KeyCode::Backspace => {
                self.input.delete_char_before();
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
                    // Handle based on input mode
                    match &self.input_mode {
                        InputMode::Chat => {
                            if !self.waiting {
                                self.send_message();
                            }
                        }
                        InputMode::NewWorkstream => {
                            let title = self.input.content().to_string();
                            self.pending_actions.push(PendingAction::CreateWorkstream(title));
                            self.input.clear();
                            self.input_mode = InputMode::Chat;
                            self.status_message = None;
                        }
                        InputMode::RenameWorkstream(id) => {
                            let new_title = self.input.content().to_string();
                            let id = id.clone();
                            self.pending_actions.push(PendingAction::RenameWorkstream(id, new_title));
                            self.input.clear();
                            self.input_mode = InputMode::Chat;
                            self.status_message = None;
                        }
                    }
                }
            }
            KeyCode::Esc => {
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

    /// Send the current input as a chat message.
    fn send_message(&mut self) {
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
        if let Err(e) = self
            .ws_client
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
                self.focus = Focus::Input;
            }
            KeyCode::Enter => {
                // Select the current session
                if let Some(session) = self.sessions.selected_session() {
                    let session_id = session.id.clone();
                    self.switch_to_session(&session_id);
                }
                self.sessions.reset();
                self.focus = Focus::Input;
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
                self.focus = Focus::Input;
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
                self.focus = Focus::Input;
            }
            KeyCode::Enter => {
                // Execute selected action
                if let Some(action) = self.palette.selected_action() {
                    let action_id = action.id;
                    self.palette.reset();
                    self.focus = Focus::Input;
                    self.execute_action(action_id);
                } else {
                    self.palette.reset();
                    self.focus = Focus::Input;
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
                // TODO: Implement session deletion
                self.status_message = Some("Session deletion not implemented yet".to_string());
            }
            ActionId::SessionsMoveToWorkstream => {
                if let Some(ref sid) = self.session_id {
                    tracing::info!("Session move initiated for session: {}", sid);
                    // Open sidebar in workstreams section for selection
                    self.sidebar.open();
                    self.sidebar.section = SidebarSection::Workstreams;
                    self.moving_session_to_workstream = true;
                    self.focus = Focus::Sidebar;
                    self.status_message = Some("Select target workstream (Enter to move, Esc to cancel)".to_string());
                } else {
                    tracing::warn!("Session move attempted with no active session");
                    self.status_message = Some("No session to move".to_string());
                }
            }
            ActionId::WorkstreamsSwitch => {
                self.focus = Focus::Workstreams;
            }
            ActionId::WorkstreamsCreate => {
                // Enter new workstream name mode
                self.input_mode = InputMode::NewWorkstream;
                self.input.clear();
                self.focus = Focus::Input;
                self.status_message = Some("Enter workstream name (Esc to cancel)".to_string());
            }
            ActionId::ViewToggleToolPane => {
                self.focus = if self.focus == Focus::ToolPane {
                    Focus::Input
                } else {
                    Focus::ToolPane
                };
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
        if let Err(e) = self.ws_client.subscribe(session_id.to_string()) {
            self.status_message = Some(format!("Failed to switch session: {}", e));
            return; // Don't clear state if we failed to subscribe
        }

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
        self.focus = Focus::Sessions;

        // Use sessions from sidebar (already loaded from API)
        self.sessions.set_items(self.sidebar.sessions.clone());
    }

    /// Handle overlay (workstreams/palette) key events.
    fn handle_overlay_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.focus = Focus::Input;
            }
            KeyCode::Enter => {
                // TODO: Select item
                self.focus = Focus::Input;
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
        match key.code {
            KeyCode::Esc => {
                self.focus = Focus::Input;
            }
            KeyCode::Up => {
                self.tool_scroll = self.tool_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.tool_scroll = self.tool_scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.tool_scroll = self.tool_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.tool_scroll = self.tool_scroll.saturating_add(10);
            }
            KeyCode::Home => {
                self.tool_scroll = 0;
            }
            KeyCode::End => {
                // Scroll to end - will be clamped in render
                self.tool_scroll = usize::MAX;
            }
            _ => {}
        }
    }

    /// Handle logs panel key events.
    fn handle_logs_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.show_logs = false;
                self.focus = Focus::Input;
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

    /// Handle sidebar key events.
    fn handle_sidebar_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Right => {
                // Close sidebar and return focus to input
                self.sidebar.close();
                self.moving_session_to_workstream = false;
                self.focus = Focus::Input;
            }
            KeyCode::Tab => {
                // Switch between workstreams and sessions sections
                self.sidebar.toggle_section();
            }
            KeyCode::Up => {
                self.sidebar.select_prev();
            }
            KeyCode::Down => {
                self.sidebar.select_next();
            }
            KeyCode::Enter => {
                // Select current item
                match self.sidebar.section {
                    SidebarSection::Workstreams => {
                        if self.moving_session_to_workstream {
                            // Move current session to selected workstream
                            if let (Some(session_id), Some(ws)) = (self.session_id.clone(), self.sidebar.selected_workstream()) {
                                tracing::info!(
                                    "User confirmed move: session {} -> workstream {} ({})",
                                    session_id,
                                    ws.name,
                                    ws.id
                                );
                                let ws_id = ws.id.clone();
                                self.pending_actions.push(PendingAction::MoveSessionToWorkstream(session_id, ws_id));
                            } else {
                                tracing::warn!("Move confirmed but session_id or workstream not available");
                            }
                            self.moving_session_to_workstream = false;
                            self.sidebar.close();
                            self.focus = Focus::Input;
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
                            self.focus = Focus::Input;
                        } else if let Some(session) = self.sidebar.selected_session() {
                            // Switch to selected session
                            let session_id = session.id.clone();
                            self.switch_to_session(&session_id);
                            self.sidebar.close();
                            self.focus = Focus::Input;
                        }
                    }
                }
            }
            KeyCode::Char('n') => {
                // Create new item in current section
                match self.sidebar.section {
                    SidebarSection::Workstreams => {
                        // Enter new workstream name mode
                        self.input_mode = InputMode::NewWorkstream;
                        self.input.clear();
                        self.sidebar.close();
                        self.focus = Focus::Input;
                        self.status_message = Some("Enter workstream name (Esc to cancel)".to_string());
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
                        self.focus = Focus::Input;
                    }
                }
            }
            KeyCode::Char('N') => {
                // Create new workstream regardless of current section
                self.input_mode = InputMode::NewWorkstream;
                self.input.clear();
                self.sidebar.close();
                self.focus = Focus::Input;
                self.status_message = Some("Enter workstream name (Esc to cancel)".to_string());
            }
            KeyCode::Char('r') => {
                // Rename selected workstream
                if self.sidebar.section == SidebarSection::Workstreams {
                    if let Some(ws) = self.sidebar.selected_workstream() {
                        let name = ws.name.clone();
                        self.input_mode = InputMode::RenameWorkstream(name.clone());
                        self.input.clear();
                        self.input.set_text(&name); // Pre-fill with current name
                        self.sidebar.close();
                        self.focus = Focus::Input;
                        self.status_message = Some("Rename workstream (Esc to cancel)".to_string());
                    }
                } else {
                    self.status_message = Some("Select a workstream to rename".to_string());
                }
            }
            KeyCode::Char('d') => {
                // Delete current item
                match self.sidebar.section {
                    SidebarSection::Workstreams => {
                        if let Some(ws) = self.sidebar.selected_workstream() {
                            if ws.is_current {
                                self.status_message = Some("Cannot delete current workstream".to_string());
                            } else {
                                let name = ws.name.clone();
                                self.pending_actions.push(PendingAction::DeleteWorkstream(name));
                            }
                        }
                    }
                    SidebarSection::Sessions => {
                        if let Some(session) = self.sidebar.selected_session() {
                            let id = session.id.clone();
                            self.pending_actions.push(PendingAction::DeleteSession(id));
                        }
                    }
                }
            }
            KeyCode::Char('/') => {
                // Start filtering - clear existing filter and start fresh
                self.sidebar.filter_clear();
                self.status_message = Some("Filter: type to search (Backspace to clear)".to_string());
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
