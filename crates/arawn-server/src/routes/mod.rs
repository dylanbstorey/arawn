//! API routes.

pub mod agents;
pub mod chat;
pub mod commands;
pub mod config;
pub mod health;
pub mod mcp;
pub mod memory;
pub mod openapi;
pub mod pagination;
pub mod sessions;
pub mod tasks;
pub mod workstreams;
pub mod ws;

pub use agents::{
    AgentCapabilities, AgentDetail, AgentSummary, AgentToolInfo, ListAgentsResponse,
    get_agent_handler, list_agents_handler,
};
pub use chat::{ChatRequest, ChatResponse, chat_handler, chat_stream_handler};
pub use config::{ConfigFeatures, ConfigLimits, ConfigResponse, get_config_handler};
pub use health::health_routes;
pub use mcp::{
    AddServerRequest, AddServerResponse, ListServersResponse, ListToolsResponse,
    RemoveServerResponse, ServerInfo, ToolInfo, add_server_handler, connect_server_handler,
    disconnect_server_handler, list_server_tools_handler, list_servers_handler,
    remove_server_handler,
};
pub use memory::{
    CreateNoteRequest, ListNotesResponse,
    MemorySearchResponse, MemorySearchResult, Note, StoreMemoryRequest, StoreMemoryResponse,
    UpdateNoteRequest, create_note_handler, delete_memory_handler, delete_note_handler,
    get_note_handler, list_notes_handler, memory_search_handler, store_memory_handler,
    update_note_handler,
};
pub use sessions::{
    CreateSessionRequest, ListSessionsResponse, MessageInfo, SessionDetail,
    SessionMessagesResponse, SessionSummary, UpdateSessionRequest, create_session_handler,
    delete_session_handler, get_session_handler, get_session_messages_handler,
    list_sessions_handler, update_session_handler,
};
pub use tasks::{
    ListTasksResponse, TaskDetail, TaskSummary, cancel_task_handler, get_task_handler,
    list_tasks_handler,
};
pub use workstreams::{
    cleanup_handler, clone_repo_handler, create_workstream_handler, delete_workstream_handler,
    export_file_handler, get_usage_handler, get_workstream_handler, list_messages_handler,
    list_workstream_sessions_handler, list_workstreams_handler, promote_file_handler, promote_handler,
    send_message_handler, update_workstream_handler, CleanupRequest, CleanupResponse, CloneRepoRequest,
    CloneRepoResponse, CreateWorkstreamRequest, ExportFileRequest, ExportFileResponse,
    MessageListResponse, MessageResponse, PromoteFileRequest, PromoteFileResponse, PromoteRequest,
    SendMessageRequest, SessionListResponse, SessionResponse, SessionUsageResponse,
    UpdateWorkstreamRequest, UsageResponse, WorkstreamListResponse, WorkstreamResponse,
};
pub use ws::{ClientMessage, ServerMessage, ws_handler};
pub use commands::{
    CommandHandler, CommandInfo, CommandOutput, CommandRegistry, CompactCommand, CompactEvent,
    CompactRequest, CompactResponse, ListCommandsResponse, SharedCommandRegistry,
    compact_command_handler, compact_command_stream_handler, list_commands_handler,
};
