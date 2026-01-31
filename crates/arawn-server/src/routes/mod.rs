//! API routes.

pub mod chat;
pub mod health;
pub mod mcp;
pub mod memory;
pub mod sessions;
pub mod workstreams;
pub mod ws;

pub use chat::{ChatRequest, ChatResponse, chat_handler, chat_stream_handler};
pub use health::health_routes;
pub use mcp::{
    AddServerRequest, AddServerResponse, ListServersResponse, ListToolsResponse,
    RemoveServerResponse, ServerInfo, ToolInfo, add_server_handler, connect_server_handler,
    disconnect_server_handler, list_server_tools_handler, list_servers_handler,
    remove_server_handler,
};
pub use memory::{
    CreateNoteRequest, ListNotesResponse, MemorySearchResponse, Note, create_note_handler,
    list_notes_handler, memory_search_handler,
};
pub use sessions::{
    ListSessionsResponse, SessionDetail, SessionSummary, delete_session_handler,
    get_session_handler, list_sessions_handler,
};
pub use workstreams::{
    create_workstream_handler, delete_workstream_handler, get_workstream_handler,
    list_messages_handler, list_workstreams_handler, promote_handler, send_message_handler,
};
pub use ws::{ClientMessage, ServerMessage, ws_handler};
