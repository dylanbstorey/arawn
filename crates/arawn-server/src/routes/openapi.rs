//! OpenAPI documentation configuration.

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use super::{agents, chat, commands, config, health, mcp, memory, sessions, tasks, workstreams};

/// OpenAPI documentation for the Arawn API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Arawn API",
        description = "HTTP API for the Arawn personal research agent",
        version = "1.0.0",
        license(name = "MIT"),
    ),
    servers(
        (url = "/", description = "Local server"),
    ),
    paths(
        // Health
        health::health,
        health::health_detailed,
        // Config
        config::get_config_handler,
        // Sessions
        sessions::create_session_handler,
        sessions::list_sessions_handler,
        sessions::get_session_handler,
        sessions::delete_session_handler,
        sessions::update_session_handler,
        sessions::get_session_messages_handler,
        // Workstreams
        workstreams::create_workstream_handler,
        workstreams::list_workstreams_handler,
        workstreams::get_workstream_handler,
        workstreams::delete_workstream_handler,
        workstreams::update_workstream_handler,
        workstreams::list_workstream_sessions_handler,
        workstreams::send_message_handler,
        workstreams::list_messages_handler,
        workstreams::promote_handler,
        workstreams::promote_file_handler,
        workstreams::export_file_handler,
        workstreams::clone_repo_handler,
        workstreams::get_usage_handler,
        workstreams::cleanup_handler,
        // Memory
        memory::create_note_handler,
        memory::list_notes_handler,
        memory::get_note_handler,
        memory::update_note_handler,
        memory::delete_note_handler,
        memory::memory_search_handler,
        memory::store_memory_handler,
        memory::delete_memory_handler,
        // Agents
        agents::list_agents_handler,
        agents::get_agent_handler,
        // Chat
        chat::chat_handler,
        chat::chat_stream_handler,
        // Tasks
        tasks::list_tasks_handler,
        tasks::get_task_handler,
        tasks::cancel_task_handler,
        // MCP
        mcp::add_server_handler,
        mcp::remove_server_handler,
        mcp::list_servers_handler,
        mcp::list_server_tools_handler,
        mcp::connect_server_handler,
        mcp::disconnect_server_handler,
        // Commands
        commands::list_commands_handler,
        commands::compact_command_handler,
        commands::compact_command_stream_handler,
    ),
    components(
        schemas(
            // Health
            health::HealthResponse,
            health::DetailedHealthResponse,
            health::AgentHealth,
            // Config
            config::ConfigResponse,
            config::ConfigFeatures,
            config::ConfigLimits,
            // Sessions
            sessions::CreateSessionRequest,
            sessions::UpdateSessionRequest,
            sessions::SessionSummary,
            sessions::SessionDetail,
            sessions::TurnInfo,
            sessions::ListSessionsResponse,
            sessions::MessageInfo,
            sessions::SessionMessagesResponse,
            // Workstreams
            workstreams::CreateWorkstreamRequest,
            workstreams::UpdateWorkstreamRequest,
            workstreams::WorkstreamResponse,
            workstreams::WorkstreamListResponse,
            workstreams::SendMessageRequest,
            workstreams::MessageResponse,
            workstreams::MessageListResponse,
            workstreams::SessionResponse,
            workstreams::SessionListResponse,
            workstreams::PromoteRequest,
            workstreams::PromoteFileRequest,
            workstreams::PromoteFileResponse,
            workstreams::ExportFileRequest,
            workstreams::ExportFileResponse,
            workstreams::CloneRepoRequest,
            workstreams::CloneRepoResponse,
            workstreams::UsageResponse,
            workstreams::SessionUsageResponse,
            workstreams::CleanupRequest,
            workstreams::CleanupResponse,
            // Memory
            memory::Note,
            memory::CreateNoteRequest,
            memory::UpdateNoteRequest,
            memory::ListNotesResponse,
            memory::MemorySearchResult,
            memory::MemorySearchResponse,
            memory::StoreMemoryRequest,
            memory::StoreMemoryResponse,
            // Agents
            agents::AgentToolInfo,
            agents::AgentSummary,
            agents::AgentDetail,
            agents::AgentCapabilities,
            agents::ListAgentsResponse,
            // Chat
            chat::ChatRequest,
            chat::ChatResponse,
            chat::ToolCallInfo,
            chat::UsageInfo,
            // Tasks
            tasks::TaskSummary,
            tasks::TaskDetail,
            tasks::ListTasksResponse,
            // MCP
            mcp::AddServerRequest,
            mcp::AddServerResponse,
            mcp::ServerInfo,
            mcp::ListServersResponse,
            mcp::ToolInfo,
            mcp::ListToolsResponse,
            mcp::RemoveServerResponse,
            // Commands
            commands::CommandInfo,
            commands::ListCommandsResponse,
            commands::CompactRequest,
            commands::CompactResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "config", description = "Server configuration"),
        (name = "sessions", description = "Session management"),
        (name = "workstreams", description = "Workstream management"),
        (name = "memory", description = "Memory and notes"),
        (name = "chat", description = "Chat endpoints"),
        (name = "agents", description = "Agent information"),
        (name = "tasks", description = "Background tasks"),
        (name = "mcp", description = "MCP server management"),
    )
)]
pub struct ApiDoc;

/// Add bearer token security scheme.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

/// Create the Swagger UI router.
pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/api/docs")
        .url("/api/openapi.json", ApiDoc::openapi())
}
