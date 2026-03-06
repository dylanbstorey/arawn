# Code Index

> Generated: 2026-03-06T00:05:15Z | 304 files | Rust

## Project Structure

```
├── crates/
│   ├── arawn/
│   │   ├── src/
│   │   │   ├── client/
│   │   │   │   └── mod.rs
│   │   │   ├── commands/
│   │   │   │   ├── agent.rs
│   │   │   │   ├── ask.rs
│   │   │   │   ├── auth.rs
│   │   │   │   ├── chat.rs
│   │   │   │   ├── config.rs
│   │   │   │   ├── mcp.rs
│   │   │   │   ├── memory.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── notes.rs
│   │   │   │   ├── output.rs
│   │   │   │   ├── plugin.rs
│   │   │   │   ├── repl.rs
│   │   │   │   ├── secrets.rs
│   │   │   │   ├── start.rs
│   │   │   │   ├── status.rs
│   │   │   │   └── tui.rs
│   │   │   └── main.rs
│   │   └── tests/
│   │       └── cli_integration.rs
│   ├── arawn-agent/
│   │   └── src/
│   │       ├── agent.rs
│   │       ├── compaction.rs
│   │       ├── context.rs
│   │       ├── error.rs
│   │       ├── indexing/
│   │       │   ├── extraction.rs
│   │       │   ├── gliner.rs
│   │       │   ├── indexer.rs
│   │       │   ├── mod.rs
│   │       │   ├── ner.rs
│   │       │   ├── report.rs
│   │       │   ├── summarization.rs
│   │       │   └── types.rs
│   │       ├── lib.rs
│   │       ├── mcp.rs
│   │       ├── orchestrator.rs
│   │       ├── prompt/
│   │       │   ├── bootstrap.rs
│   │       │   ├── builder.rs
│   │       │   ├── mod.rs
│   │       │   └── mode.rs
│   │       ├── rlm/
│   │       │   ├── integration_tests.rs
│   │       │   ├── mod.rs
│   │       │   ├── prompt.rs
│   │       │   └── types.rs
│   │       ├── stream.rs
│   │       ├── tool/
│   │       │   ├── command_validator.rs
│   │       │   ├── context.rs
│   │       │   ├── execution.rs
│   │       │   ├── gate.rs
│   │       │   ├── mod.rs
│   │       │   ├── output.rs
│   │       │   ├── params.rs
│   │       │   ├── registry.rs
│   │       │   └── validation.rs
│   │       ├── tools/
│   │       │   ├── catalog.rs
│   │       │   ├── delegate.rs
│   │       │   ├── explore.rs
│   │       │   ├── file.rs
│   │       │   ├── memory.rs
│   │       │   ├── mod.rs
│   │       │   ├── note.rs
│   │       │   ├── search.rs
│   │       │   ├── shell.rs
│   │       │   ├── think.rs
│   │       │   ├── web.rs
│   │       │   └── workflow.rs
│   │       └── types.rs
│   ├── arawn-client/
│   │   └── src/
│   │       ├── api/
│   │       │   ├── agents.rs
│   │       │   ├── chat.rs
│   │       │   ├── config.rs
│   │       │   ├── health.rs
│   │       │   ├── mcp.rs
│   │       │   ├── memory.rs
│   │       │   ├── mod.rs
│   │       │   ├── notes.rs
│   │       │   ├── sessions.rs
│   │       │   ├── tasks.rs
│   │       │   └── workstreams.rs
│   │       ├── client.rs
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       └── types.rs
│   ├── arawn-config/
│   │   └── src/
│   │       ├── age_crypto.rs
│   │       ├── client.rs
│   │       ├── discovery.rs
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       ├── paths.rs
│   │       ├── resolver.rs
│   │       ├── secret_store.rs
│   │       ├── secrets.rs
│   │       └── types.rs
│   ├── arawn-domain/
│   │   └── src/
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       └── services/
│   │           ├── chat.rs
│   │           ├── mcp.rs
│   │           ├── memory.rs
│   │           └── mod.rs
│   ├── arawn-llm/
│   │   └── src/
│   │       ├── anthropic.rs
│   │       ├── api_key.rs
│   │       ├── backend.rs
│   │       ├── client.rs
│   │       ├── embeddings.rs
│   │       ├── error.rs
│   │       ├── interaction_log.rs
│   │       ├── lib.rs
│   │       ├── openai.rs
│   │       └── types.rs
│   ├── arawn-mcp/
│   │   ├── src/
│   │   │   ├── client.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── manager.rs
│   │   │   ├── protocol.rs
│   │   │   └── transport.rs
│   │   └── tests/
│   │       ├── integration.rs
│   │       └── mock_server.rs
│   ├── arawn-memory/
│   │   └── src/
│   │       ├── backend.rs
│   │       ├── error.rs
│   │       ├── graph.rs
│   │       ├── lib.rs
│   │       ├── store/
│   │       │   ├── graph_ops.rs
│   │       │   ├── memory_ops.rs
│   │       │   ├── mod.rs
│   │       │   ├── note_ops.rs
│   │       │   ├── query.rs
│   │       │   ├── recall.rs
│   │       │   ├── session_ops.rs
│   │       │   ├── unified_ops.rs
│   │       │   └── vector_ops.rs
│   │       ├── types.rs
│   │       ├── validation.rs
│   │       └── vector.rs
│   ├── arawn-oauth/
│   │   └── src/
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       ├── oauth.rs
│   │       ├── passthrough.rs
│   │       ├── proxy.rs
│   │       └── token_manager.rs
│   ├── arawn-pipeline/
│   │   ├── src/
│   │   │   ├── catalog.rs
│   │   │   ├── context.rs
│   │   │   ├── definition.rs
│   │   │   ├── engine.rs
│   │   │   ├── error.rs
│   │   │   ├── factory.rs
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs
│   │   │   ├── protocol.rs
│   │   │   ├── sandbox.rs
│   │   │   └── task.rs
│   │   └── tests/
│   │       ├── e2e_runtime_test.rs
│   │       └── engine_test.rs
│   ├── arawn-plugin/
│   │   └── src/
│   │       ├── agent_spawner.rs
│   │       ├── hooks.rs
│   │       ├── lib.rs
│   │       ├── manager.rs
│   │       ├── manifest.rs
│   │       ├── skill.rs
│   │       ├── subscription.rs
│   │       ├── types.rs
│   │       ├── validation.rs
│   │       └── watcher.rs
│   ├── arawn-sandbox/
│   │   └── src/
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       ├── manager.rs
│   │       └── platform.rs
│   ├── arawn-script-sdk/
│   │   └── src/
│   │       ├── context.rs
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       └── text.rs
│   ├── arawn-server/
│   │   ├── src/
│   │   │   ├── auth.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── ratelimit.rs
│   │   │   ├── routes/
│   │   │   │   ├── agents.rs
│   │   │   │   ├── chat.rs
│   │   │   │   ├── commands.rs
│   │   │   │   ├── config.rs
│   │   │   │   ├── health.rs
│   │   │   │   ├── mcp.rs
│   │   │   │   ├── memory.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── openapi.rs
│   │   │   │   ├── pagination.rs
│   │   │   │   ├── sessions.rs
│   │   │   │   ├── tasks.rs
│   │   │   │   ├── workstreams.rs
│   │   │   │   └── ws/
│   │   │   │       ├── connection.rs
│   │   │   │       ├── handlers.rs
│   │   │   │       ├── mod.rs
│   │   │   │       └── protocol.rs
│   │   │   ├── session_cache.rs
│   │   │   └── state.rs
│   │   └── tests/
│   │       ├── chat_integration.rs
│   │       ├── common/
│   │       │   └── mod.rs
│   │       ├── context_management.rs
│   │       ├── memory_integration.rs
│   │       ├── server_integration.rs
│   │       └── validation_integration.rs
│   ├── arawn-session/
│   │   └── src/
│   │       ├── cache.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       ├── persistence.rs
│   │       └── ttl.rs
│   ├── arawn-tui/
│   │   └── src/
│   │       ├── app.rs
│   │       ├── bounded.rs
│   │       ├── client.rs
│   │       ├── events.rs
│   │       ├── focus.rs
│   │       ├── input.rs
│   │       ├── lib.rs
│   │       ├── logs.rs
│   │       ├── palette.rs
│   │       ├── protocol.rs
│   │       ├── sessions.rs
│   │       ├── sidebar.rs
│   │       └── ui/
│   │           ├── chat.rs
│   │           ├── command_popup.rs
│   │           ├── input.rs
│   │           ├── layout.rs
│   │           ├── logs.rs
│   │           ├── mod.rs
│   │           ├── palette.rs
│   │           ├── sessions.rs
│   │           ├── sidebar.rs
│   │           └── tools.rs
│   ├── arawn-types/
│   │   └── src/
│   │       ├── config.rs
│   │       ├── delegation.rs
│   │       ├── fs_gate.rs
│   │       ├── hooks.rs
│   │       ├── lib.rs
│   │       └── secret_resolver.rs
│   ├── arawn-workstream/
│   │   └── src/
│   │       ├── cleanup.rs
│   │       ├── compression.rs
│   │       ├── context.rs
│   │       ├── directory/
│   │       │   ├── clone.rs
│   │       │   ├── manager.rs
│   │       │   ├── mod.rs
│   │       │   ├── operations.rs
│   │       │   ├── session.rs
│   │       │   └── usage.rs
│   │       ├── error.rs
│   │       ├── fs_gate.rs
│   │       ├── lib.rs
│   │       ├── manager.rs
│   │       ├── message_store.rs
│   │       ├── path_validator.rs
│   │       ├── scratch.rs
│   │       ├── session.rs
│   │       ├── session_loader.rs
│   │       ├── storage.rs
│   │       ├── store.rs
│   │       ├── types.rs
│   │       └── watcher.rs
│   ├── gline-rs-vendored/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── model/
│   │       │   ├── input/
│   │       │   │   ├── encoded.rs
│   │       │   │   ├── mod.rs
│   │       │   │   ├── prompt.rs
│   │       │   │   ├── relation/
│   │       │   │   │   ├── mod.rs
│   │       │   │   │   └── schema.rs
│   │       │   │   ├── tensors/
│   │       │   │   │   ├── mod.rs
│   │       │   │   │   ├── span.rs
│   │       │   │   │   └── token.rs
│   │       │   │   ├── text.rs
│   │       │   │   └── tokenized.rs
│   │       │   ├── mod.rs
│   │       │   ├── output/
│   │       │   │   ├── decoded/
│   │       │   │   │   ├── greedy.rs
│   │       │   │   │   ├── mod.rs
│   │       │   │   │   ├── sort.rs
│   │       │   │   │   ├── span.rs
│   │       │   │   │   ├── token.rs
│   │       │   │   │   └── token_flat.rs
│   │       │   │   ├── mod.rs
│   │       │   │   ├── relation.rs
│   │       │   │   └── tensors.rs
│   │       │   ├── params.rs
│   │       │   └── pipeline/
│   │       │       ├── context.rs
│   │       │       ├── mod.rs
│   │       │       ├── relation.rs
│   │       │       ├── span.rs
│   │       │       └── token.rs
│   │       ├── text/
│   │       │   ├── mod.rs
│   │       │   ├── prompt.rs
│   │       │   ├── span.rs
│   │       │   ├── splitter.rs
│   │       │   ├── token.rs
│   │       │   └── tokenizer.rs
│   │       └── util/
│   │           ├── error.rs
│   │           ├── math.rs
│   │           ├── memprof.rs
│   │           ├── mod.rs
│   │           └── result.rs
│   └── orp-vendored/
│       └── src/
│           ├── bin/
│           │   └── inspect.rs
│           ├── error.rs
│           ├── lib.rs
│           ├── model.rs
│           ├── params.rs
│           └── pipeline.rs
└── runtimes/
    ├── file_read/
    │   └── src/
    │       └── main.rs
    ├── file_write/
    │   └── src/
    │       └── main.rs
    ├── http/
    │   └── src/
    │       └── main.rs
    ├── passthrough/
    │   └── src/
    │       └── main.rs
    ├── shell/
    │   └── src/
    │       └── main.rs
    └── transform/
        └── src/
            └── main.rs
```

## Modules

### crates/arawn/src/client

> *Semantic summary to be generated by AI agent.*

#### crates/arawn/src/client/mod.rs

- pub `HealthResponse` struct L20-23 — `{ status: String, version: String }` — Health check response from the server.
- pub `MemoryResult` struct L27-36 — `{ id: String, content_type: String, content: String, score: f32, source: String ...` — Memory search result.
- pub `MemorySearchResponse` struct L41-47 — `{ results: Vec<MemoryResult>, query: String, count: usize }` — Memory search response.
- pub `Note` struct L51-60 — `{ id: String, title: Option<String>, content: String, tags: Vec<String>, created...` — Note from the server.
- pub `SessionInfo` struct L71-75 — `{ id: String, created_at: String, message_count: usize }` — Session info.
- pub `SessionListResponse` struct L80-82 — `{ sessions: Vec<SessionInfo> }` — Session list response.
- pub `NotesResponse` struct L87-95 — `{ notes: Vec<Note>, total: usize, limit: usize, offset: usize }` — Notes list response.
- pub `WsServerMessage` enum L120-148 — `AuthResult | SessionCreated | ChatChunk | ToolStart | ToolEnd | Error | Pong` — Messages received from the server via WebSocket.
- pub `ChatEvent` enum L157-168 — `Text | ToolStart | ToolEnd | Done | Error` — Events from streaming chat responses.
- pub `ChatStream` struct L171-173 — `{ receiver: Pin<Box<dyn Stream<Item = Result<ChatEvent>> + Send>> }` — Streaming chat response.
- pub `next` function L178-193 — `(&mut self) -> Option<Result<String>>` — Get the next event from the stream (simplified text-only).
- pub `next_event` function L196-198 — `(&mut self) -> Option<Result<ChatEvent>>` — Get the next raw event from the stream.
- pub `Client` struct L206-210 — `{ base_url: Url, http: reqwest::Client, token: Option<String> }` — HTTP/WebSocket client for the Arawn server.
- pub `new` function L214-223 — `(base_url: &str) -> Result<Self>` — Create a new client for the given server URL.
- pub `health` function L226-237 — `(&self) -> Result<HealthResponse>` — Check server health.
- pub `chat_stream` function L240-343 — `(&self, message: &str, session_id: Option<&str>) -> Result<ChatStream>` — Send a chat message and get a streaming response via WebSocket.
- pub `memory_search` function L346-366 — `(&self, query: &str, limit: usize) -> Result<Vec<MemoryResult>>` — Search memories.
- pub `create_note` function L369-388 — `(&self, content: &str) -> Result<Note>` — Create a note.
- pub `list_notes` function L391-408 — `(&self) -> Result<Vec<Note>>` — List all notes.
- pub `get_note` function L411-432 — `(&self, id: &str) -> Result<Note>` — Get a single note by ID.
- pub `delete_note` function L435-455 — `(&self, id: &str) -> Result<()>` — Delete a note by ID.
- pub `search_notes` function L458-489 — `(&self, query: &str, limit: usize) -> Result<Vec<MemoryResult>>` — Search notes via memory search endpoint, filtering for note results.
- pub `list_sessions` function L493-510 — `(&self) -> Result<Vec<SessionInfo>>` — List sessions.
- pub `delete_session` function L514-532 — `(&self, session_id: &str) -> Result<()>` — Delete a session.
-  `CreateNoteRequest` struct L64-66 — `{ content: String }` — Create note request.
-  `WsClientMessage` enum L105-114 — `Auth | Chat | Ping` — Messages sent to the server via WebSocket.
-  `ChatStream` type L175-199 — `= ChatStream` — REST API and WebSocket endpoints.
-  `Client` type L212-533 — `= Client` — REST API and WebSocket endpoints.
-  `tests` module L540-581 — `-` — REST API and WebSocket endpoints.
-  `test_client_creation` function L544-547 — `()` — REST API and WebSocket endpoints.
-  `test_ws_client_message_serialization` function L550-565 — `()` — REST API and WebSocket endpoints.
-  `test_ws_server_message_deserialization` function L568-580 — `()` — REST API and WebSocket endpoints.

### crates/arawn/src/commands

> *Semantic summary to be generated by AI agent.*

#### crates/arawn/src/commands/agent.rs

- pub `AgentArgs` struct L24-27 — `{ command: AgentCommand }` — - `arawn agent info <name>` - Show details for a specific agent
- pub `AgentCommand` enum L30-36 — `List | Info` — - `arawn agent info <name>` - Show details for a specific agent
- pub `ListArgs` struct L40-44 — `{ plugin: Option<String> }` — Arguments for `arawn agent list`.
- pub `InfoArgs` struct L48-51 — `{ name: String }` — Arguments for `arawn agent info`.
- pub `run` function L54-59 — `(args: AgentArgs, ctx: &Context) -> Result<()>` — Run the agent command.
-  `AgentInfo` struct L63-71 — `{ name: String, description: String, tools: Vec<String>, source_plugin: String, ...` — Information about an agent for display.
-  `load_agents` function L74-141 — `() -> Result<Vec<AgentInfo>>` — Load all plugins and extract agent information.
-  `run_list` function L144-178 — `(args: ListArgs, ctx: &Context) -> Result<()>` — Run `arawn agent list`.
-  `print_list_json` function L181-198 — `(agents: &[AgentInfo]) -> Result<()>` — Print agent list as JSON.
-  `print_list_table` function L201-245 — `(agents: &[AgentInfo], verbose: bool) -> Result<()>` — Print agent list as a table.
-  `run_info` function L248-308 — `(args: InfoArgs, ctx: &Context) -> Result<()>` — Run `arawn agent info`.
-  `print_info_json` function L311-324 — `(agent: &AgentInfo) -> Result<()>` — Print agent info as JSON.
-  `print_info_detail` function L327-357 — `(agent: &AgentInfo) -> Result<()>` — Print detailed agent info.

#### crates/arawn/src/commands/ask.rs

- pub `AskArgs` struct L17-29 — `{ prompt: String, session: Option<String>, no_memory: bool }` — Ask command - one-shot question to the agent.
- pub `run` function L32-95 — `(args: AskArgs, ctx: &Context) -> Result<()>` — Run the ask command.

#### crates/arawn/src/commands/auth.rs

- pub `AuthArgs` struct L16-19 — `{ command: AuthCommand }` — Auth command - authentication management.
- pub `AuthCommand` enum L22-38 — `Login | Status | Logout | Token` — Auth command - authentication management.
- pub `run` function L41-48 — `(args: AuthArgs, ctx: &Context) -> Result<()>` — Run the auth command.
-  `cmd_login` function L50-135 — `(_ctx: &Context) -> Result<()>` — Auth command - authentication management.
-  `cmd_status` function L137-183 — `(_ctx: &Context) -> Result<()>` — Auth command - authentication management.
-  `cmd_logout` function L185-202 — `() -> Result<()>` — Auth command - authentication management.
-  `cmd_token` function L204-219 — `(generate: bool, _ctx: &Context) -> Result<()>` — Auth command - authentication management.
-  `build_oauth_config` function L222-239 — `() -> arawn_oauth::OAuthConfig` — Build an OAuthConfig applying any `[oauth]` overrides from arawn config.
-  `open_url` function L242-258 — `(url: &str) -> std::io::Result<()>` — Try to open a URL in the default browser.

#### crates/arawn/src/commands/chat.rs

- pub `ChatArgs` struct L16-24 — `{ session: Option<String>, new: bool }` — Chat command - interactive REPL mode.
- pub `run` function L27-36 — `(args: ChatArgs, ctx: &Context) -> Result<()>` — Run the chat command (REPL).

#### crates/arawn/src/commands/config.rs

- pub `ConfigArgs` struct L23-26 — `{ command: ConfigCommand }` — Config command - configuration management.
- pub `ConfigCommand` enum L29-97 — `Show | Which | SetSecret | DeleteSecret | Edit | Init | Path | CurrentContext | ...` — Config command - configuration management.
- pub `run` function L100-120 — `(args: ConfigArgs, ctx: &Context) -> Result<()>` — Run the config command.
-  `cmd_show` function L122-197 — `(ctx: &Context) -> Result<()>` — Config command - configuration management.
-  `cmd_which` function L199-222 — `(_ctx: &Context) -> Result<()>` — Config command - configuration management.
-  `cmd_set_secret` function L224-255 — `(backend_str: &str) -> Result<()>` — Config command - configuration management.
-  `cmd_delete_secret` function L257-273 — `(backend_str: &str) -> Result<()>` — Config command - configuration management.
-  `cmd_edit` function L275-295 — `() -> Result<()>` — Config command - configuration management.
-  `cmd_init` function L297-358 — `(local: bool) -> Result<()>` — Config command - configuration management.
-  `cmd_path` function L360-367 — `() -> Result<()>` — Config command - configuration management.
-  `parse_backend` function L369-382 — `(s: &str) -> Result<Backend>` — Config command - configuration management.
-  `key_status_for` function L384-400 — `(backend: &Backend) -> &'static str` — Config command - configuration management.
-  `cmd_current_context` function L406-421 — `() -> Result<()>` — Config command - configuration management.
-  `cmd_get_contexts` function L423-448 — `() -> Result<()>` — Config command - configuration management.
-  `cmd_use_context` function L450-459 — `(name: &str) -> Result<()>` — Config command - configuration management.
-  `cmd_set_context` function L461-513 — `( name: &str, server: Option<String>, workstream: Option<String>, timeout: Optio...` — Config command - configuration management.
-  `cmd_delete_context` function L515-534 — `(name: &str) -> Result<()>` — Config command - configuration management.

#### crates/arawn/src/commands/mcp.rs

- pub `McpArgs` struct L30-33 — `{ command: McpCommand }` — - `arawn mcp test` - Test connection to an MCP server
- pub `McpCommand` enum L36-48 — `List | Add | Remove | Test` — - `arawn mcp test` - Test connection to an MCP server
- pub `ListArgs` struct L52-56 — `{ tools: bool }` — Arguments for `arawn mcp list`.
- pub `AddArgs` struct L60-94 — `{ name: String, target: String, http: bool, args: Vec<String>, env_vars: Vec<Str...` — Arguments for `arawn mcp add`.
- pub `RemoveArgs` struct L98-101 — `{ name: String }` — Arguments for `arawn mcp remove`.
- pub `TestArgs` struct L105-112 — `{ name: String, full: bool }` — Arguments for `arawn mcp test`.
- pub `run` function L115-122 — `(args: McpArgs, ctx: &Context) -> Result<()>` — Run the MCP command.
-  `run_list` function L125-149 — `(args: ListArgs, ctx: &Context) -> Result<()>` — Run `arawn mcp list`.
-  `print_list_json` function L152-194 — `(servers: &[McpServerEntry], show_tools: bool) -> Result<()>` — Print server list as JSON.
-  `print_list_table` function L197-278 — `(servers: &[McpServerEntry], show_tools: bool, verbose: bool) -> Result<()>` — Print server list as a table.
-  `connect_and_list_tools` function L281-296 — `(server: &McpServerEntry) -> Result<Vec<String>>` — Connect to an MCP server and list its tools.
-  `server_entry_to_config` function L299-327 — `(entry: &McpServerEntry) -> Result<McpServerConfig>` — Convert a McpServerEntry to an McpServerConfig.
-  `run_add` function L330-445 — `(args: AddArgs, ctx: &Context) -> Result<()>` — Run `arawn mcp add`.
-  `run_remove` function L448-492 — `(args: RemoveArgs, ctx: &Context) -> Result<()>` — Run `arawn mcp remove`.
-  `run_test` function L495-659 — `(args: TestArgs, ctx: &Context) -> Result<()>` — Run `arawn mcp test`.
-  `textwrap_simple` function L662-684 — `(text: &str, max_width: usize) -> String` — Simple text wrapping helper.

#### crates/arawn/src/commands/memory.rs

- pub `MemoryArgs` struct L20-23 — `{ command: MemoryCommand }` — Memory command - memory operations.
- pub `MemoryCommand` enum L26-64 — `Search | Recent | Stats | Reindex | Export` — Memory command - memory operations.
- pub `run` function L67-75 — `(args: MemoryArgs, ctx: &Context) -> Result<()>` — Run the memory command.
-  `cmd_search` function L77-113 — `(query: &str, limit: usize, ctx: &Context) -> Result<()>` — Memory command - memory operations.
-  `cmd_recent` function L115-149 — `(limit: usize, ctx: &Context) -> Result<()>` — Memory command - memory operations.
-  `cmd_stats` function L151-185 — `(_ctx: &Context) -> Result<()>` — Memory command - memory operations.
-  `cmd_reindex` function L187-272 — `(dry_run: bool, yes: bool, _ctx: &Context) -> Result<()>` — Memory command - memory operations.
-  `cmd_export` function L274-310 — `(output: Option<String>, ctx: &Context) -> Result<()>` — Memory command - memory operations.
-  `open_memory_store` function L313-319 — `() -> Result<arawn_memory::MemoryStore>` — Open the memory store at the default data directory.
-  `build_embedder_spec` function L322-350 — `(config: &arawn_config::EmbeddingConfig) -> arawn_llm::EmbedderSpec` — Build an EmbedderSpec from EmbeddingConfig (same logic as start.rs).

#### crates/arawn/src/commands/mod.rs

- pub `agent` module L3 — `-` — CLI command handlers.
- pub `ask` module L4 — `-` — CLI command handlers.
- pub `auth` module L5 — `-` — CLI command handlers.
- pub `chat` module L6 — `-` — CLI command handlers.
- pub `config` module L7 — `-` — CLI command handlers.
- pub `mcp` module L8 — `-` — CLI command handlers.
- pub `memory` module L9 — `-` — CLI command handlers.
- pub `notes` module L10 — `-` — CLI command handlers.
- pub `output` module L11 — `-` — CLI command handlers.
- pub `plugin` module L12 — `-` — CLI command handlers.
- pub `repl` module L13 — `-` — CLI command handlers.
- pub `secrets` module L14 — `-` — CLI command handlers.
- pub `start` module L15 — `-` — CLI command handlers.
- pub `status` module L16 — `-` — CLI command handlers.
- pub `tui` module L17 — `-` — CLI command handlers.
- pub `Context` struct L23-30 — `{ server_url: String, json_output: bool, verbose: bool }` — Shared context for all commands.
- pub `format_user_error` function L36-134 — `(error: &anyhow::Error, server_url: &str) -> String` — Format an error into a user-friendly message with actionable suggestions.
- pub `print_cli_error` function L140-157 — `(error: &anyhow::Error, server_url: &str, verbose: bool)` — Print a CLI error with optional verbose details.
-  `tests` module L160-277 — `-` — CLI command handlers.
-  `make_error` function L163-165 — `(msg: &str) -> anyhow::Error` — CLI command handlers.
-  `URL` variable L167 — `: &str` — CLI command handlers.
-  `test_connection_refused` function L170-176 — `()` — CLI command handlers.
-  `test_tcp_connect_error` function L179-183 — `()` — CLI command handlers.
-  `test_dns_error` function L186-191 — `()` — CLI command handlers.
-  `test_auth_failed` function L194-200 — `()` — CLI command handlers.
-  `test_401` function L203-207 — `()` — CLI command handlers.
-  `test_403` function L210-215 — `()` — CLI command handlers.
-  `test_404` function L218-222 — `()` — CLI command handlers.
-  `test_note_not_found` function L225-230 — `()` — CLI command handlers.
-  `test_500` function L233-238 — `()` — CLI command handlers.
-  `test_timeout` function L241-246 — `()` — CLI command handlers.
-  `test_toml_parse_error` function L249-254 — `()` — CLI command handlers.
-  `test_websocket_handshake` function L257-262 — `()` — CLI command handlers.
-  `test_unknown_error_passes_through` function L265-269 — `()` — CLI command handlers.
-  `test_server_url_included_in_connection_error` function L272-276 — `()` — CLI command handlers.

#### crates/arawn/src/commands/notes.rs

- pub `NotesArgs` struct L19-22 — `{ command: NotesCommand }` — Notes command - note management.
- pub `NotesCommand` enum L25-60 — `Add | List | Search | Show | Delete` — Notes command - note management.
- pub `run` function L63-178 — `(args: NotesArgs, ctx: &Context) -> Result<()>` — Run the notes command.

#### crates/arawn/src/commands/output.rs

- pub `header` function L16-20 — `(title: &str)` — Print a section header: bold title + dim separator line.
- pub `success` function L23-25 — `(msg: impl Display)` — Print a success message with a green checkmark.
- pub `error` function L28-30 — `(msg: impl Display)` — Print an error message to stderr with red "Error:" prefix.
- pub `kv` function L37-43 — `(label: &str, value: impl Display)` — Print a dim-labeled key-value pair, indented.
- pub `hint` function L46-48 — `(msg: impl Display)` — Print a dim hint/note line.
- pub `truncate` function L51-58 — `(s: &str, max_len: usize) -> String` — Truncate a string to a maximum length, collapsing newlines to spaces.
- pub `truncate_multiline` function L61-71 — `(s: &str, max_len: usize) -> String` — Truncate a multiline string, preserving indentation on continuation.

#### crates/arawn/src/commands/plugin.rs

- pub `PluginArgs` struct L30-33 — `{ command: PluginCommand }` — - `arawn plugin list` - List all plugins
- pub `PluginCommand` enum L36-48 — `Add | Update | Remove | List` — - `arawn plugin list` - List all plugins
- pub `AddArgs` struct L52-63 — `{ source: String, r#ref: Option<String>, project: bool }` — Arguments for `arawn plugin add`.
- pub `UpdateArgs` struct L67-70 — `{ name: Option<String> }` — Arguments for `arawn plugin update`.
- pub `RemoveArgs` struct L74-85 — `{ name: String, project: bool, delete_cache: bool }` — Arguments for `arawn plugin remove`.
- pub `ListArgs` struct L89-97 — `{ subscribed: bool, local: bool }` — Arguments for `arawn plugin list`.
- pub `run` function L100-107 — `(args: PluginArgs, ctx: &Context) -> Result<()>` — Run the plugin command.
-  `parse_source` function L110-127 — `(source: &str, git_ref: Option<String>) -> PluginSubscription` — Parse a source string into a PluginSubscription.
-  `run_add` function L130-187 — `(args: AddArgs, ctx: &Context) -> Result<()>` — Run `arawn plugin add`.
-  `run_update` function L190-278 — `(args: UpdateArgs, ctx: &Context) -> Result<()>` — Run `arawn plugin update`.
-  `run_remove` function L281-348 — `(args: RemoveArgs, ctx: &Context) -> Result<()>` — Run `arawn plugin remove`.
-  `run_list` function L351-382 — `(args: ListArgs, ctx: &Context) -> Result<()>` — Run `arawn plugin list`.
-  `print_list_json` function L385-431 — `( subscriptions: &[PluginSubscription], local_plugins: &[arawn_plugin::LoadedPlu...` — Print plugin list as JSON.
-  `print_list_table` function L434-514 — `( subscriptions: &[PluginSubscription], local_plugins: &[arawn_plugin::LoadedPlu...` — Print plugin list as a table.

#### crates/arawn/src/commands/repl.rs

- pub `Repl` struct L13-20 — `{ client: Client, server_url: String, session_id: Option<String>, editor: Editor...` — REPL state and configuration.
- pub `new` function L24-45 — `( client: Client, server_url: String, session_id: Option<String>, verbose: bool,...` — Create a new REPL instance.
- pub `run` function L48-99 — `(&mut self) -> Result<()>` — Run the REPL loop.
- pub `ControlFlow` enum L322-325 — `Continue | Exit` — Control flow for the REPL.
-  `Repl` type L22-319 — `= Repl` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `send_message` function L102-141 — `(&mut self, message: &str) -> Result<()>` — Send a message and stream the response.
-  `handle_slash_command` function L144-191 — `(&mut self, input: &str) -> Result<ControlFlow>` — Handle a slash command.
-  `print_welcome` function L193-207 — `(&self)` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `print_help` function L209-230 — `(&self)` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `print_status` function L232-252 — `(&self) -> Result<()>` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `search_memory` function L254-279 — `(&self, query: &str) -> Result<()>` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `add_note` function L281-293 — `(&self, content: &str) -> Result<()>` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `format_prompt` function L295-297 — `(&self) -> String` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `print_dim` function L299-302 — `(&self, msg: &str)` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `print_error` function L304-307 — `(&self, msg: &str)` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `print_tool_start` function L309-312 — `(&self, name: &str)` — REPL (Read-Eval-Print Loop) implementation for interactive chat.
-  `print_tool_end` function L314-318 — `(&self, success: bool)` — REPL (Read-Eval-Print Loop) implementation for interactive chat.

#### crates/arawn/src/commands/secrets.rs

- pub `SecretsArgs` struct L14-17 — `{ command: SecretsCommand }` — Secrets command - manage age-encrypted secret store.
- pub `SecretsCommand` enum L20-35 — `Set | List | Delete` — Secrets command - manage age-encrypted secret store.
- pub `run` function L38-44 — `(args: SecretsArgs) -> Result<()>` — Run the secrets command.
-  `cmd_set` function L46-69 — `(name: &str) -> Result<()>` — Secrets command - manage age-encrypted secret store.
-  `cmd_list` function L71-91 — `() -> Result<()>` — Secrets command - manage age-encrypted secret store.
-  `cmd_delete` function L93-104 — `(name: &str) -> Result<()>` — Secrets command - manage age-encrypted secret store.

#### crates/arawn/src/commands/start.rs

- pub `StartArgs` struct L48-100 — `{ daemon: bool, port: Option<u16>, bind: Option<String>, token: Option<String>, ...` — Start command - launches the Arawn server.
- pub `run` function L103-1419 — `(args: StartArgs, ctx: &Context) -> Result<()>` — Run the start command.
-  `resolve_with_cli_overrides` function L1422-1472 — `( config: &arawn_config::ArawnConfig, args: &StartArgs, ) -> Result<ResolvedLlm>` — Resolve LLM config, applying CLI overrides on top of config file values.
-  `make_api_key_provider` function L1478-1483 — `(backend: Backend, config_value: Option<String>) -> ApiKeyProvider` — Build an `ApiKeyProvider` that re-resolves from the secret store on each request.
-  `create_backend` function L1486-1623 — `( resolved: &ResolvedLlm, oauth_overrides: Option<&arawn_config::OAuthConfigOver...` — Create an LLM backend from a resolved config.
-  `parse_backend` function L1625-1638 — `(s: &str) -> Result<Backend>` — Start command - launches the Arawn server.
-  `load_or_generate_server_token` function L1641-1657 — `() -> Result<String>` — Load a persisted server token, or generate and save a new one.
-  `resolve_profile` function L1660-1691 — `(name: &str, llm_config: &LlmConfig) -> Result<ResolvedLlm>` — Resolve a named LLM profile into a ResolvedLlm ready for backend creation.
-  `build_embedder_spec` function L1694-1740 — `(config: &arawn_config::EmbeddingConfig) -> EmbedderSpec` — Build an `EmbedderSpec` from the application's `EmbeddingConfig`.
-  `default_model` function L1742-1750 — `(backend: &Backend) -> String` — Start command - launches the Arawn server.
-  `register_builtin_runtimes` function L1757-1841 — `( runtimes_src_dir: &std::path::Path, executor: &Arc<ScriptExecutor>, catalog: &...` — Compile and register built-in WASM runtimes from source crate directories.
-  `seed_test_data` function L1844-1937 — `(manager: &WorkstreamManager, verbose: bool)` — Seed the database with test workstreams and sessions for development.

#### crates/arawn/src/commands/status.rs

- pub `StatusArgs` struct L18 — `-` — Status command - shows server status and resource usage.
- pub `run` function L29-77 — `(_args: StatusArgs, ctx: &Context) -> Result<()>` — Run the status command.
-  `StatusOutput` struct L22-26 — `{ running: bool, version: Option<String>, server_url: String }` — Status response for JSON output.

#### crates/arawn/src/commands/tui.rs

- pub `TuiArgs` struct L13-17 — `{ workstream: Option<String> }` — TUI command handler.
- pub `run` function L20-50 — `(args: TuiArgs, ctx: &Context) -> Result<()>` — Run the TUI.

### crates/arawn/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn/src/main.rs

- pub `Cli` struct L30-49 — `{ verbose: bool, json: bool, server: Option<String>, context: Option<String>, co...` — Main entry point for the Arawn CLI.
- pub `Commands` enum L52-91 — `Start | Status | Ask | Chat | Memory | Notes | Config | Auth | Plugin | Agent | ...` — Main entry point for the Arawn CLI.
-  `client` module L8 — `-` — Main entry point for the Arawn CLI.
-  `commands` module L9 — `-` — Main entry point for the Arawn CLI.
-  `resolve_server_url` function L105-134 — `(server_flag: Option<&str>, context_flag: Option<&str>) -> String` — Resolve the server URL from various sources.
-  `main` function L141-149 — `()` — Main entry point for the Arawn CLI.
-  `run` function L151-220 — `() -> Result<()>` — Main entry point for the Arawn CLI.

### crates/arawn/tests

> *Semantic summary to be generated by AI agent.*

#### crates/arawn/tests/cli_integration.rs

-  `arawn` function L15-17 — `() -> Command` — Get a command for the arawn binary.
-  `test_help_displays` function L24-31 — `()` — CLI parsing and help output only.
-  `test_version_displays` function L34-40 — `()` — CLI parsing and help output only.
-  `test_help_lists_subcommands` function L43-60 — `()` — CLI parsing and help output only.
-  `test_verbose_flag_accepted` function L67-71 — `()` — CLI parsing and help output only.
-  `test_json_flag_accepted` function L74-76 — `()` — CLI parsing and help output only.
-  `test_server_flag_accepted` function L79-84 — `()` — CLI parsing and help output only.
-  `test_context_flag_accepted` function L87-92 — `()` — CLI parsing and help output only.
-  `test_start_help` function L99-105 — `()` — CLI parsing and help output only.
-  `test_status_help` function L108-114 — `()` — CLI parsing and help output only.
-  `test_ask_help` function L117-123 — `()` — CLI parsing and help output only.
-  `test_chat_help` function L126-132 — `()` — CLI parsing and help output only.
-  `test_memory_help` function L135-141 — `()` — CLI parsing and help output only.
-  `test_notes_help` function L144-150 — `()` — CLI parsing and help output only.
-  `test_config_help` function L153-159 — `()` — CLI parsing and help output only.
-  `test_auth_help` function L162-168 — `()` — CLI parsing and help output only.
-  `test_plugin_help` function L171-177 — `()` — CLI parsing and help output only.
-  `test_agent_help` function L180-186 — `()` — CLI parsing and help output only.
-  `test_mcp_help` function L189-195 — `()` — CLI parsing and help output only.
-  `test_tui_help` function L198-204 — `()` — CLI parsing and help output only.
-  `test_unknown_subcommand_fails` function L211-217 — `()` — CLI parsing and help output only.
-  `test_invalid_flag_fails` function L220-226 — `()` — CLI parsing and help output only.
-  `test_config_subcommands_listed` function L233-235 — `()` — CLI parsing and help output only.
-  `test_auth_subcommands_listed` function L242-244 — `()` — CLI parsing and help output only.
-  `test_plugin_subcommands_listed` function L251-253 — `()` — CLI parsing and help output only.
-  `test_mcp_subcommands_listed` function L260-262 — `()` — CLI parsing and help output only.

### crates/arawn-agent/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-agent/src/agent.rs

- pub `RecallConfig` struct L34-41 — `{ enabled: bool, threshold: f32, limit: usize }` — Configuration for active recall behavior in the agent turn loop.
- pub `Agent` struct L58-79 — `{ backend: SharedBackend, tools: Arc<ToolRegistry>, config: AgentConfig, interac...` — The core agent that orchestrates LLM calls and tool execution.
- pub `new` function L83-96 — `(backend: SharedBackend, tools: ToolRegistry, config: AgentConfig) -> Self` — Create a new agent with the given backend and tools.
- pub `builder` function L99-101 — `() -> AgentBuilder` — Create an agent builder for fluent construction.
- pub `config` function L104-106 — `(&self) -> &AgentConfig` — Get the agent configuration.
- pub `tools` function L109-111 — `(&self) -> &ToolRegistry` — Get the tool registry.
- pub `backend` function L114-116 — `(&self) -> SharedBackend` — Get the LLM backend.
- pub `turn` function L122-365 — `( &self, session: &mut Session, user_message: &str, workstream_id: Option<&str>,...` — Execute a single turn of conversation.
- pub `turn_stream` function L379-411 — `( &self, session: &mut Session, user_message: &str, cancellation: CancellationTo...` — Execute a single turn of conversation with streaming output.
- pub `AgentBuilder` struct L770-784 — `{ backend: Option<SharedBackend>, tools: ToolRegistry, config: AgentConfig, prom...` — Builder for constructing an Agent with fluent API.
- pub `new` function L788-804 — `() -> Self` — Create a new builder with defaults.
- pub `with_backend` function L807-810 — `(mut self, backend: impl LlmBackend + 'static) -> Self` — Set the LLM backend.
- pub `with_shared_backend` function L813-816 — `(mut self, backend: SharedBackend) -> Self` — Set the LLM backend from a shared reference.
- pub `with_tools` function L819-822 — `(mut self, tools: ToolRegistry) -> Self` — Set the tool registry.
- pub `with_tool` function L825-828 — `(mut self, tool: T) -> Self` — Register a single tool.
- pub `with_config` function L831-834 — `(mut self, config: AgentConfig) -> Self` — Set the configuration.
- pub `with_model` function L837-840 — `(mut self, model: impl Into<String>) -> Self` — Set the model.
- pub `with_system_prompt` function L843-846 — `(mut self, prompt: impl Into<String>) -> Self` — Set the system prompt.
- pub `with_max_tokens` function L849-852 — `(mut self, max_tokens: u32) -> Self` — Set max tokens.
- pub `with_max_iterations` function L855-858 — `(mut self, max_iterations: u32) -> Self` — Set max iterations.
- pub `with_max_total_tokens` function L864-867 — `(mut self, max_total_tokens: usize) -> Self` — Set cumulative token budget (input + output).
- pub `with_workspace` function L872-875 — `(mut self, path: impl Into<std::path::PathBuf>) -> Self` — Set the workspace path.
- pub `with_prompt_builder` function L883-886 — `(mut self, builder: SystemPromptBuilder) -> Self` — Set a prompt builder for dynamic system prompt generation.
- pub `with_bootstrap_dir` function L902-928 — `(mut self, path: impl AsRef<std::path::Path>) -> Self` — Load bootstrap context files from a directory.
- pub `with_prompt_file` function L944-971 — `(mut self, path: impl AsRef<std::path::Path>) -> Self` — Load a custom prompt file and add it to the bootstrap context.
- pub `with_memory_store` function L974-977 — `(mut self, store: Arc<MemoryStore>) -> Self` — Set the memory store for active recall.
- pub `with_embedder` function L980-983 — `(mut self, embedder: SharedEmbedder) -> Self` — Set the embedder for active recall.
- pub `with_recall_config` function L986-989 — `(mut self, config: RecallConfig) -> Self` — Set the recall configuration.
- pub `with_interaction_logger` function L992-995 — `(mut self, logger: Arc<InteractionLogger>) -> Self` — Set the interaction logger for structured JSONL capture.
- pub `with_plugin_prompts` function L1001-1004 — `(mut self, prompts: Vec<(String, String)>) -> Self` — Add plugin prompt fragments to the system prompt.
- pub `with_hook_dispatcher` function L1013-1016 — `(mut self, dispatcher: SharedHookDispatcher) -> Self` — Set the hook dispatcher for plugin lifecycle events.
- pub `build` function L1019-1068 — `(mut self) -> Result<Agent>` — Build the agent.
- pub `with_fs_gate_resolver` function L1071-1074 — `(mut self, resolver: FsGateResolver) -> Self` — Set the filesystem gate resolver for workstream sandbox enforcement.
- pub `with_secret_resolver` function L1077-1080 — `(mut self, resolver: SharedSecretResolver) -> Self` — Set the secret resolver for `${{secrets.*}}` handle resolution in tool params.
-  `RecallConfig` type L43-51 — `impl Default for RecallConfig` — conversation loop, handles tool execution, and manages context.
-  `default` function L44-50 — `() -> Self` — conversation loop, handles tool execution, and manages context.
-  `Agent` type L81-748 — `= Agent` — conversation loop, handles tool execution, and manages context.
-  `estimate_messages_tokens` function L414-419 — `(&self, messages: &[Message]) -> usize` — Estimate total tokens for a list of messages.
-  `estimate_message_tokens` function L422-449 — `(&self, message: &Message) -> usize` — Estimate tokens for a single message.
-  `build_messages` function L452-512 — `(&self, session: &Session) -> Vec<Message>` — Build messages from session history.
-  `build_request` function L519-563 — `( &self, messages: &[Message], context_preamble: Option<&str>, ) -> CompletionRe...` — Build a completion request.
-  `execute_tools` function L566-681 — `( &self, response: &CompletionResponse, session_id: crate::types::SessionId, tur...` — Execute tool calls from an LLM response.
-  `perform_recall` function L688-747 — `(&self, user_message: &str) -> Option<Message>` — Perform active recall for a user message.
-  `format_recall_context` function L751-763 — `(matches: &[arawn_memory::store::RecallMatch]) -> String` — Format recall matches into a concise context string for injection.
-  `AgentBuilder` type L786-1081 — `= AgentBuilder` — conversation loop, handles tool execution, and manages context.
-  `AgentBuilder` type L1083-1087 — `impl Default for AgentBuilder` — conversation loop, handles tool execution, and manages context.
-  `default` function L1084-1086 — `() -> Self` — conversation loop, handles tool execution, and manages context.
-  `tests` module L1094-1773 — `-` — conversation loop, handles tool execution, and manages context.
-  `mock_text_response` function L1099-1110 — `(text: &str) -> CompletionResponse` — conversation loop, handles tool execution, and manages context.
-  `mock_tool_use_response` function L1112-1129 — `( tool_id: &str, tool_name: &str, args: serde_json::Value, ) -> CompletionRespon...` — conversation loop, handles tool execution, and manages context.
-  `test_agent_builder_no_backend` function L1132-1135 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_builder_with_backend` function L1138-1152 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_simple_turn_no_tools` function L1155-1167 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_turn_with_tool_use` function L1170-1199 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_turn_max_iterations` function L1202-1230 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_turn_token_budget_exceeded` function L1233-1261 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_turn_no_token_budget` function L1264-1277 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_turn_tool_error_handling` function L1280-1312 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_turn_unknown_tool` function L1315-1332 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_tool_validation_error_retry` function L1335-1357 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_tool_validation_error_exhausts_retries` function L1360-1385 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_multi_turn_conversation` function L1388-1408 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_with_prompt_builder` function L1411-1434 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_prompt_builder_with_static_fallback` function L1437-1451 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_prompt_builder_overrides_static` function L1454-1474 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_with_bootstrap_dir` function L1477-1507 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_bootstrap_dir_creates_builder_if_none` function L1510-1533 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_bootstrap_dir_nonexistent_is_ok` function L1536-1548 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_with_prompt_file` function L1551-1571 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_with_multiple_prompt_files` function L1574-1597 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_agent_combine_bootstrap_dir_and_prompt_file` function L1600-1630 — `()` — conversation loop, handles tool execution, and manages context.
-  `recall_tests` module L1634-1772 — `-` — conversation loop, handles tool execution, and manages context.
-  `FixedEmbedder` struct L1642-1644 — `{ dims: usize }` — Simple mock embedder that returns a fixed vector.
-  `FixedEmbedder` type L1646-1650 — `= FixedEmbedder` — conversation loop, handles tool execution, and manages context.
-  `new` function L1647-1649 — `(dims: usize) -> Self` — conversation loop, handles tool execution, and manages context.
-  `FixedEmbedder` type L1653-1665 — `impl Embedder for FixedEmbedder` — conversation loop, handles tool execution, and manages context.
-  `embed` function L1654-1656 — `(&self, _text: &str) -> arawn_llm::Result<Vec<f32>>` — conversation loop, handles tool execution, and manages context.
-  `dimensions` function L1658-1660 — `(&self) -> usize` — conversation loop, handles tool execution, and manages context.
-  `name` function L1662-1664 — `(&self) -> &str` — conversation loop, handles tool execution, and manages context.
-  `create_recall_store` function L1667-1672 — `(dims: usize) -> Arc<MemoryStore>` — conversation loop, handles tool execution, and manages context.
-  `test_recall_injects_context` function L1676-1708 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_recall_no_results` function L1712-1734 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_recall_disabled_config` function L1737-1752 — `()` — conversation loop, handles tool execution, and manages context.
-  `test_recall_no_embedder` function L1755-1771 — `()` — conversation loop, handles tool execution, and manages context.

#### crates/arawn-agent/src/compaction.rs

- pub `CompactorConfig` struct L38-47 — `{ model: String, max_summary_tokens: u32, preserve_recent: usize, summary_prompt...` — Configuration for session compaction.
- pub `CompactionResult` struct L77-86 — `{ turns_compacted: usize, tokens_before: usize, tokens_after: usize, summary: St...` — Result of a compaction operation.
- pub `tokens_freed` function L90-92 — `(&self) -> usize` — Estimate tokens freed by compaction.
- pub `compression_ratio` function L95-100 — `(&self) -> f32` — Get compression ratio (smaller is better).
- pub `ProgressCallback` type L104 — `= Box<dyn Fn(CompactionProgress) + Send + Sync>` — Progress callback for compaction operations.
- pub `CompactionProgress` enum L108-123 — `Started | Summarizing | Completed | Cancelled` — Progress updates during compaction.
- pub `CancellationToken` struct L127-129 — `{ cancelled: Arc<AtomicBool> }` — Token for cancelling compaction operations.
- pub `new` function L133-137 — `() -> Self` — Create a new cancellation token.
- pub `cancel` function L140-142 — `(&self)` — Signal cancellation.
- pub `is_cancelled` function L145-147 — `(&self) -> bool` — Check if cancellation was requested.
- pub `SessionCompactor` struct L175-178 — `{ backend: SharedBackend, config: CompactorConfig }` — Compacts sessions by summarizing older turns while preserving recent ones.
- pub `new` function L182-184 — `(backend: SharedBackend, config: CompactorConfig) -> Self` — Create a new session compactor.
- pub `with_preserve_recent` function L187-190 — `(mut self, count: usize) -> Self` — Set the number of recent turns to preserve.
- pub `with_summary_prompt` function L197-200 — `(mut self, prompt: impl Into<String>) -> Self` — Set a custom summary prompt for compaction.
- pub `compact` function L205-207 — `(&self, session: &Session) -> Result<Option<CompactionResult>>` — Compact a session, generating a summary of older turns.
- pub `compact_with_progress` function L210-216 — `( &self, session: &Session, progress: Option<&ProgressCallback>, ) -> Result<Opt...` — Compact with progress callback.
- pub `compact_with_options` function L219-295 — `( &self, session: &Session, progress: Option<&ProgressCallback>, cancel: Option<...` — Compact with full options: progress callback and cancellation token.
- pub `needs_compaction` function L300-305 — `(&self, session: &Session, threshold: usize) -> bool` — Check if a session needs compaction based on turn count.
-  `DEFAULT_PRESERVE_RECENT` variable L20 — `: usize` — Default number of recent turns to preserve verbatim.
-  `MID_SESSION_SUMMARY_PROMPT` variable L23-30 — `: &str` — System prompt for mid-session summarization.
-  `CompactorConfig` type L49-58 — `impl Default for CompactorConfig` — recent turns verbatim, enabling context management before hitting hard limits.
-  `default` function L50-57 — `() -> Self` — recent turns verbatim, enabling context management before hitting hard limits.
-  `CompactionResult` type L88-101 — `= CompactionResult` — recent turns verbatim, enabling context management before hitting hard limits.
-  `CancellationToken` type L131-148 — `= CancellationToken` — recent turns verbatim, enabling context management before hitting hard limits.
-  `SessionCompactor` type L180-379 — `= SessionCompactor` — recent turns verbatim, enabling context management before hitting hard limits.
-  `estimate_turns_tokens` function L308-326 — `(&self, turns: &[Turn]) -> usize` — Estimate tokens for a slice of turns.
-  `summarize_turns` function L329-378 — `(&self, turns: &[Turn]) -> Result<String>` — Generate a summary of the given turns.
-  `tests` module L386-654 — `-` — recent turns verbatim, enabling context management before hitting hard limits.
-  `create_test_session` function L391-398 — `(turn_count: usize) -> Session` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_config` function L400-405 — `() -> CompactorConfig` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compactor` function L407-409 — `(backend: SharedBackend) -> SessionCompactor` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compactor_config_defaults` function L412-416 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compaction_result_tokens_freed` function L419-427 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compaction_result_compression_ratio` function L430-438 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compaction_result_zero_tokens_before` function L441-449 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_needs_compaction_below_threshold` function L452-459 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_needs_compaction_at_threshold` function L462-469 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_needs_compaction_above_threshold` function L472-479 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_empty_session` function L482-489 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_insufficient_turns` function L492-500 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_preserves_recent_turns` function L503-515 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_custom_preserve_count` function L518-529 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_custom_summary_prompt` function L532-549 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_result_stats` function L552-563 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_with_progress_callback` function L566-587 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_estimate_turns_tokens` function L590-600 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_cancellation_token` function L603-609 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_cancelled_before_start` function L612-626 — `()` — recent turns verbatim, enabling context management before hitting hard limits.
-  `test_compact_cancelled_reports_progress` function L629-653 — `()` — recent turns verbatim, enabling context management before hitting hard limits.

#### crates/arawn-agent/src/context.rs

- pub `estimate_tokens` function L22-24 — `(text: &str) -> usize` — Estimate token count for a string (rough approximation).
- pub `estimate_tokens_from_bytes` function L27-29 — `(bytes: usize) -> usize` — Estimate tokens for a byte count.
- pub `ContextStatus` enum L40-62 — `Ok | Warning | Critical` — Status of context usage relative to thresholds.
- pub `is_ok` function L66-68 — `(&self) -> bool` — Returns true if status is Ok.
- pub `is_warning` function L71-73 — `(&self) -> bool` — Returns true if status is Warning or Critical.
- pub `is_critical` function L76-78 — `(&self) -> bool` — Returns true if status is Critical.
- pub `current` function L81-87 — `(&self) -> usize` — Get current token count.
- pub `max` function L90-94 — `(&self) -> usize` — Get maximum token capacity.
- pub `percent` function L97-103 — `(&self) -> f32` — Get usage as percentage (0.0 - 1.0).
- pub `remaining` function L106-108 — `(&self) -> usize` — Get remaining tokens.
- pub `ContextTracker` struct L128-137 — `{ max_tokens: usize, current_tokens: usize, warning_threshold: f32, critical_thr...` — Tracks token usage for a session with configurable thresholds.
- pub `DEFAULT_WARNING_THRESHOLD` variable L141 — `: f32` — Default warning threshold (70% of max).
- pub `DEFAULT_CRITICAL_THRESHOLD` variable L143 — `: f32` — Default critical threshold (90% of max).
- pub `for_model` function L146-153 — `(max_tokens: usize) -> Self` — Create a new context tracker for a model with the given max tokens.
- pub `with_warning_threshold` function L156-159 — `(mut self, threshold: f32) -> Self` — Set custom warning threshold (0.0 - 1.0).
- pub `with_critical_threshold` function L162-165 — `(mut self, threshold: f32) -> Self` — Set custom critical threshold (0.0 - 1.0).
- pub `update` function L168-170 — `(&mut self, token_count: usize)` — Update the current token count.
- pub `add` function L173-175 — `(&mut self, tokens: usize)` — Add tokens to the current count.
- pub `status` function L178-190 — `(&self) -> ContextStatus` — Get the current context status based on thresholds.
- pub `usage_percent` function L193-198 — `(&self) -> f32` — Get current usage as a percentage (0.0 - 1.0).
- pub `should_compact` function L201-203 — `(&self) -> bool` — Returns true if compaction should be triggered (critical threshold exceeded).
- pub `current_tokens` function L206-208 — `(&self) -> usize` — Get current token count.
- pub `max_tokens` function L211-213 — `(&self) -> usize` — Get maximum tokens.
- pub `remaining_tokens` function L216-218 — `(&self) -> usize` — Get remaining tokens before hitting max.
- pub `reset` function L221-223 — `(&mut self)` — Reset the tracker to zero usage.
- pub `ContextBuilder` struct L248-255 — `{ max_context_tokens: usize, chars_per_token: usize, system_prompt: Option<Strin...` — Builds LLM completion requests from session context.
- pub `new` function L259-265 — `() -> Self` — Create a new context builder with default settings.
- pub `with_max_tokens` function L268-271 — `(mut self, max_tokens: usize) -> Self` — Set the maximum context tokens.
- pub `with_system_prompt` function L274-277 — `(mut self, prompt: impl Into<String>) -> Self` — Set the system prompt.
- pub `build` function L315-324 — `( &self, session: &Session, user_message: &str, config: &AgentConfig, tools: &To...` — Build a completion request from session and user message.
- pub `build_messages` function L329-372 — `(&self, session: &Session, user_message: &str) -> Vec<Message>` — Build messages from session history.
- pub `count_messages` function L477-483 — `(&self, session: &Session) -> usize` — Get message count for a session (for diagnostics).
- pub `estimate_session_tokens` function L486-493 — `(&self, session: &Session) -> usize` — Estimate total tokens for a session (for diagnostics).
-  `CHARS_PER_TOKEN` variable L13 — `: usize` — Default characters per token ratio (rough estimate for English text).
-  `RESERVED_RESPONSE_TOKENS` variable L16 — `: usize` — Tokens reserved for the LLM response when building context.
-  `ContextStatus` type L64-109 — `= ContextStatus` — handling token budget management and message formatting.
-  `ContextTracker` type L139-224 — `= ContextTracker` — handling token budget management and message formatting.
-  `ContextBuilder` type L257-494 — `= ContextBuilder` — handling token budget management and message formatting.
-  `estimate_tokens` function L280-282 — `(&self, text: &str) -> usize` — Estimate token count for a string (rough approximation).
-  `estimate_message_tokens` function L285-312 — `(&self, message: &Message) -> usize` — Estimate token count for a message.
-  `turn_to_messages` function L375-431 — `(&self, turn: &Turn) -> Vec<Message>` — Convert a single turn to LLM messages.
-  `build_request` function L434-474 — `( &self, messages: Vec<Message>, config: &AgentConfig, tools: &ToolRegistry, con...` — Build a completion request from messages.
-  `ContextBuilder` type L496-500 — `impl Default for ContextBuilder` — handling token budget management and message formatting.
-  `default` function L497-499 — `() -> Self` — handling token budget management and message formatting.
-  `tests` module L507-870 — `-` — handling token budget management and message formatting.
-  `test_context_builder_default` function L512-516 — `()` — handling token budget management and message formatting.
-  `test_context_builder_config` function L519-526 — `()` — handling token budget management and message formatting.
-  `test_build_messages_empty_session` function L529-537 — `()` — handling token budget management and message formatting.
-  `test_build_messages_with_history` function L540-552 — `()` — handling token budget management and message formatting.
-  `test_build_messages_with_tool_calls` function L555-577 — `()` — handling token budget management and message formatting.
-  `test_build_messages_truncation` function L580-607 — `()` — handling token budget management and message formatting.
-  `test_build_request_with_tools` function L610-626 — `()` — handling token budget management and message formatting.
-  `test_estimate_tokens` function L629-635 — `()` — handling token budget management and message formatting.
-  `test_count_messages` function L638-649 — `()` — handling token budget management and message formatting.
-  `test_estimate_session_tokens` function L652-661 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_for_model` function L668-674 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_custom_thresholds` function L677-684 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_threshold_clamping` function L687-694 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_update` function L697-706 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_add` function L709-718 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_usage_percent` function L721-732 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_usage_percent_zero_max` function L735-738 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_status_ok` function L741-759 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_status_warning` function L762-780 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_status_critical` function L783-801 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_should_compact` function L804-815 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_remaining_tokens` function L818-828 — `()` — handling token budget management and message formatting.
-  `test_context_tracker_reset` function L831-840 — `()` — handling token budget management and message formatting.
-  `test_context_status_at_exact_thresholds` function L843-853 — `()` — handling token budget management and message formatting.
-  `test_context_status_remaining` function L856-862 — `()` — handling token budget management and message formatting.
-  `test_context_status_percent_zero_max` function L865-869 — `()` — handling token budget management and message formatting.

#### crates/arawn-agent/src/error.rs

- pub `Result` type L7 — `= std::result::Result<T, AgentError>` — Result type alias using the agent error type.
- pub `AgentError` enum L11-55 — `Llm | Tool | ToolNotFound | InvalidToolParams | Session | Context | Config | Ser...` — Error type for agent operations.
- pub `tool` function L59-61 — `(msg: impl Into<String>) -> Self` — Create a tool error.
- pub `session` function L64-66 — `(msg: impl Into<String>) -> Self` — Create a session error.
- pub `context` function L69-71 — `(msg: impl Into<String>) -> Self` — Create a context error.
- pub `internal` function L74-76 — `(msg: impl Into<String>) -> Self` — Create an internal error.
- pub `is_rate_limit` function L79-81 — `(&self) -> bool` — Check if this error wraps an LLM rate limit.
- pub `llm_error` function L84-89 — `(&self) -> Option<&arawn_llm::LlmError>` — Get the wrapped LLM error if present.
- pub `retry_after` function L92-97 — `(&self) -> Option<Duration>` — Get the retry-after duration if this is a rate limit error.
-  `AgentError` type L57-98 — `= AgentError` — Error types for the agent crate.
-  `tests` module L101-116 — `-` — Error types for the agent crate.
-  `test_error_display` function L105-109 — `()` — Error types for the agent crate.
-  `test_tool_not_found` function L112-115 — `()` — Error types for the agent crate.

#### crates/arawn-agent/src/lib.rs

- pub `agent` module L31 — `-` — This crate provides the agent loop, tool framework, and task execution
- pub `compaction` module L32 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `context` module L33 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `error` module L34 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `indexing` module L35 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `mcp` module L36 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `orchestrator` module L37 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `prompt` module L38 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `rlm` module L39 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `stream` module L40 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `tool` module L41 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `tools` module L42 — `-` — - [`AgentResponse`]: Output from an agent turn
- pub `types` module L43 — `-` — - [`AgentResponse`]: Output from an agent turn

#### crates/arawn-agent/src/mcp.rs

- pub `NAMESPACE_DELIMITER` variable L42 — `: &str` — Delimiter used in namespaced tool names.
- pub `MCP_PREFIX` variable L45 — `: &str` — Prefix for all MCP tool names.
- pub `McpToolAdapter` struct L54-67 — `{ full_name: String, server_name: String, tool_name: String, description: String...` — Adapter that wraps an MCP tool as an Arawn [`Tool`].
- pub `new` function L75-103 — `(client: Arc<McpClient>, tool_info: &ToolInfo) -> Self` — Create a new MCP tool adapter.
- pub `from_client` function L112-121 — `(client: Arc<McpClient>) -> std::result::Result<Vec<Self>, McpError>` — Create adapters for all tools available from an MCP client.
- pub `server_name` function L124-126 — `(&self) -> &str` — Get the server name this tool belongs to.
- pub `tool_name` function L129-131 — `(&self) -> &str` — Get the original tool name (without namespace).
- pub `matches_name` function L139-147 — `(&self, name: &str) -> bool` — Check if a tool name matches this adapter's namespaced name.
- pub `parse_namespaced_name` function L261-268 — `(name: &str) -> Option<(&str, &str, &str)>` — Parse a namespaced tool name into its components.
- pub `is_mcp_tool` function L271-273 — `(name: &str) -> bool` — Check if a tool name is an MCP tool (starts with "mcp:").
-  `McpToolAdapter` type L69-148 — `= McpToolAdapter` — ```
-  `McpToolAdapter` type L150-159 — `= McpToolAdapter` — ```
-  `fmt` function L151-158 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `McpToolAdapter` type L162-199 — `impl Tool for McpToolAdapter` — ```
-  `name` function L163-165 — `(&self) -> &str` — ```
-  `description` function L167-169 — `(&self) -> &str` — ```
-  `parameters` function L171-173 — `(&self) -> Value` — ```
-  `execute` function L175-198 — `(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult>` — ```
-  `convert_mcp_result` function L202-246 — `(mcp_result: CallToolResult) -> ToolResult` — Convert an MCP [`CallToolResult`] to an Arawn [`ToolResult`].
-  `tests` module L276-452 — `-` — ```
-  `test_namespace_delimiter` function L280-282 — `()` — ```
-  `test_mcp_prefix` function L285-287 — `()` — ```
-  `test_parse_namespaced_name_valid` function L290-293 — `()` — ```
-  `test_parse_namespaced_name_long_tool_name` function L296-299 — `()` — ```
-  `test_parse_namespaced_name_invalid_prefix` function L302-305 — `()` — ```
-  `test_parse_namespaced_name_too_few_parts` function L308-311 — `()` — ```
-  `test_parse_namespaced_name_no_delimiter` function L314-317 — `()` — ```
-  `test_is_mcp_tool_valid` function L320-323 — `()` — ```
-  `test_is_mcp_tool_invalid` function L326-330 — `()` — ```
-  `test_convert_mcp_result_text` function L333-346 — `()` — ```
-  `test_convert_mcp_result_multiple_text` function L349-367 — `()` — ```
-  `test_convert_mcp_result_error` function L370-380 — `()` — ```
-  `test_convert_mcp_result_empty` function L383-394 — `()` — ```
-  `test_convert_mcp_result_image` function L397-413 — `()` — ```
-  `test_convert_mcp_result_resource_with_text` function L416-431 — `()` — ```
-  `test_convert_mcp_result_resource_without_text` function L434-451 — `()` — ```

#### crates/arawn-agent/src/orchestrator.rs

- pub `OrchestratorConfig` struct L25-38 — `{ max_context_tokens: usize, compaction_threshold: f32, max_compactions: u32, ma...` — Configuration for the compaction orchestrator.
- pub `OrchestrationResult` struct L64-71 — `{ text: String, truncated: bool, metadata: OrchestrationMetadata }` — Result of an orchestrated run.
- pub `OrchestrationMetadata` struct L75-84 — `{ total_iterations: u32, compactions_performed: u32, total_input_tokens: u32, to...` — Metadata from an orchestration run.
- pub `total_tokens` function L88-90 — `(&self) -> u32` — Total tokens used (input + output).
- pub `CompactionOrchestrator` struct L111-115 — `{ agent: Agent, compactor: SessionCompactor, config: OrchestratorConfig }` — Manages the explore→compact→continue cycle for long-running agent tasks.
- pub `new` function L119-125 — `(agent: Agent, compactor: SessionCompactor, config: OrchestratorConfig) -> Self` — Create a new orchestrator.
- pub `with_compaction_backend` function L131-150 — `( agent: Agent, compaction_backend: SharedBackend, compaction_prompt: Option<Str...` — Create an orchestrator with a compaction backend that may differ from the agent's.
- pub `run` function L160-285 — `(&self, query: &str) -> Result<OrchestrationResult>` — Run the agent with compaction-managed context.
-  `OrchestratorConfig` type L40-49 — `impl Default for OrchestratorConfig` — This is generic infrastructure — any long-running agent can use it.
-  `default` function L41-48 — `() -> Self` — This is generic infrastructure — any long-running agent can use it.
-  `OrchestratorConfig` type L51-56 — `= OrchestratorConfig` — This is generic infrastructure — any long-running agent can use it.
-  `threshold_tokens` function L53-55 — `(&self) -> usize` — Token count that triggers compaction.
-  `OrchestrationMetadata` type L86-91 — `= OrchestrationMetadata` — This is generic infrastructure — any long-running agent can use it.
-  `CompactionOrchestrator` type L117-308 — `= CompactionOrchestrator` — This is generic infrastructure — any long-running agent can use it.
-  `estimate_session_tokens` function L288-307 — `(&self, session: &Session) -> usize` — Estimate total tokens in a session's conversation history.
-  `tests` module L315-596 — `-` — This is generic infrastructure — any long-running agent can use it.
-  `mock_text_response` function L321-332 — `(text: &str) -> CompletionResponse` — This is generic infrastructure — any long-running agent can use it.
-  `mock_tool_use_response` function L334-351 — `( tool_id: &str, tool_name: &str, args: serde_json::Value, ) -> CompletionRespon...` — This is generic infrastructure — any long-running agent can use it.
-  `make_agent` function L355-362 — `(backend: MockBackend, tools: ToolRegistry) -> Agent` — Build an agent with max_iterations=1 so the orchestrator controls
-  `make_compactor` function L364-373 — `(backend: MockBackend) -> SessionCompactor` — This is generic infrastructure — any long-running agent can use it.
-  `test_simple_run_no_tools` function L376-392 — `()` — This is generic infrastructure — any long-running agent can use it.
-  `test_run_with_tool_calls_then_done` function L395-421 — `()` — This is generic infrastructure — any long-running agent can use it.
-  `test_compaction_triggered_at_threshold` function L424-458 — `()` — This is generic infrastructure — any long-running agent can use it.
-  `test_no_compaction_when_under_threshold` function L461-491 — `()` — This is generic infrastructure — any long-running agent can use it.
-  `test_max_compactions_exceeded` function L494-531 — `()` — This is generic infrastructure — any long-running agent can use it.
-  `test_max_turns_stops_cleanly` function L534-562 — `()` — This is generic infrastructure — any long-running agent can use it.
-  `test_cumulative_stats` function L565-595 — `()` — This is generic infrastructure — any long-running agent can use it.

#### crates/arawn-agent/src/stream.rs

- pub `StreamChunk` enum L38-77 — `Text | ToolStart | ToolOutput | ToolEnd | Done | Error` — A chunk emitted during streaming response.
- pub `text` function L81-85 — `(content: impl Into<String>) -> Self` — Create a text chunk.
- pub `tool_start` function L88-93 — `(id: impl Into<String>, name: impl Into<String>) -> Self` — Create a tool start chunk.
- pub `tool_output` function L96-101 — `(id: impl Into<String>, content: impl Into<String>) -> Self` — Create a tool output chunk (partial output during execution).
- pub `tool_end` function L104-110 — `(id: impl Into<String>, success: bool, content: impl Into<String>) -> Self` — Create a tool end chunk.
- pub `done` function L113-115 — `(iterations: u32) -> Self` — Create a done chunk.
- pub `error` function L118-122 — `(message: impl Into<String>) -> Self` — Create an error chunk.
- pub `AgentStream` type L130 — `= Pin<Box<dyn Stream<Item = StreamChunk> + Send + 'static>>` — A boxed stream of chunks.
- pub `create_turn_stream` function L153-350 — `( backend: SharedBackend, tools: Arc<ToolRegistry>, config: AgentConfig, message...` — Create a streaming response for an agent turn.
-  `StreamChunk` type L79-123 — `= StreamChunk` — token-by-token output during agent responses.
-  `StreamState` struct L133-146 — `{ backend: SharedBackend, tools: Arc<ToolRegistry>, config: AgentConfig, message...` — State for streaming agent responses.
-  `build_stream_request` function L352-374 — `(state: &StreamState) -> CompletionRequest` — token-by-token output during agent responses.
-  `build_sync_request` function L376-397 — `(state: &StreamState) -> CompletionRequest` — token-by-token output during agent responses.
-  `tests` module L404-457 — `-` — token-by-token output during agent responses.
-  `test_stream_chunk_text` function L408-411 — `()` — token-by-token output during agent responses.
-  `test_stream_chunk_tool_start` function L414-420 — `()` — token-by-token output during agent responses.
-  `test_stream_chunk_tool_end` function L423-430 — `()` — token-by-token output during agent responses.
-  `test_stream_chunk_done` function L433-436 — `()` — token-by-token output during agent responses.
-  `test_stream_chunk_error` function L439-445 — `()` — token-by-token output during agent responses.
-  `test_stream_chunk_serialization` function L448-456 — `()` — token-by-token output during agent responses.

#### crates/arawn-agent/src/types.rs

- pub `SessionId` struct L22 — `-` — Unique identifier for a session.
- pub `new` function L26-28 — `() -> Self` — Create a new random session ID.
- pub `from_uuid` function L31-33 — `(uuid: Uuid) -> Self` — Create from an existing UUID.
- pub `as_uuid` function L36-38 — `(&self) -> &Uuid` — Get the underlying UUID.
- pub `TurnId` struct L55 — `-` — Unique identifier for a turn within a session.
- pub `new` function L59-61 — `() -> Self` — Create a new random turn ID.
- pub `from_uuid` function L64-66 — `(uuid: Uuid) -> Self` — Create from an existing UUID.
- pub `as_uuid` function L69-71 — `(&self) -> &Uuid` — Get the underlying UUID.
- pub `ToolCall` struct L105-112 — `{ id: String, name: String, arguments: serde_json::Value }` — A tool call made by the agent.
- pub `ToolResultRecord` struct L129-136 — `{ tool_call_id: String, success: bool, content: String }` — Result of a tool execution.
- pub `Turn` struct L156-171 — `{ id: TurnId, user_message: String, assistant_response: Option<String>, tool_cal...` — A single conversation turn (user message + agent response).
- pub `new` function L175-185 — `(user_message: impl Into<String>) -> Self` — Create a new turn with the given user message.
- pub `complete` function L188-191 — `(&mut self, response: impl Into<String>)` — Set the assistant response and mark as completed.
- pub `add_tool_call` function L194-196 — `(&mut self, call: ToolCall)` — Add a tool call to this turn.
- pub `add_tool_result` function L199-201 — `(&mut self, result: ToolResultRecord)` — Add a tool result to this turn.
- pub `is_complete` function L204-206 — `(&self) -> bool` — Check if this turn is complete.
- pub `has_tool_calls` function L209-211 — `(&self) -> bool` — Check if this turn has any tool calls.
- pub `Session` struct L220-239 — `{ id: SessionId, turns: Vec<Turn>, created_at: DateTime<Utc>, updated_at: DateTi...` — A conversation session containing multiple turns.
- pub `new` function L253-264 — `() -> Self` — Create a new empty session.
- pub `with_id` function L267-278 — `(id: SessionId) -> Self` — Create a session with a specific ID.
- pub `init_context_tracker` function L283-285 — `(&mut self, max_tokens: usize)` — Initialize context tracking for this session with the given max tokens.
- pub `context_tracker` function L288-290 — `(&self) -> Option<&crate::context::ContextTracker>` — Get the context tracker, if initialized.
- pub `context_tracker_mut` function L293-295 — `(&mut self) -> Option<&mut crate::context::ContextTracker>` — Get the context tracker mutably, if initialized.
- pub `set_context_preamble` function L308-310 — `(&mut self, preamble: impl Into<String>)` — Set a context preamble that's included in system prompts but not in turn history.
- pub `clear_context_preamble` function L313-315 — `(&mut self)` — Clear the context preamble.
- pub `context_preamble` function L318-320 — `(&self) -> Option<&str>` — Get the context preamble, if set.
- pub `start_turn` function L323-328 — `(&mut self, user_message: impl Into<String>) -> &mut Turn` — Start a new turn with the given user message.
- pub `current_turn` function L331-333 — `(&self) -> Option<&Turn>` — Get the current (most recent) turn, if any.
- pub `current_turn_mut` function L336-338 — `(&mut self) -> Option<&mut Turn>` — Get the current turn mutably.
- pub `recent_turns` function L341-344 — `(&self, n: usize) -> &[Turn]` — Get the N most recent turns.
- pub `all_turns` function L347-349 — `(&self) -> &[Turn]` — Get all turns.
- pub `turn_count` function L352-354 — `(&self) -> usize` — Get the number of turns.
- pub `is_empty` function L357-359 — `(&self) -> bool` — Check if the session is empty (no turns).
- pub `set_metadata` function L362-365 — `(&mut self, key: impl Into<String>, value: serde_json::Value)` — Set a metadata value.
- pub `get_metadata` function L368-370 — `(&self, key: &str) -> Option<&serde_json::Value>` — Get a metadata value.
- pub `remove_metadata` function L373-379 — `(&mut self, key: &str) -> Option<serde_json::Value>` — Remove a metadata value.
- pub `AgentConfig` struct L406-430 — `{ model: String, max_tokens: u32, temperature: Option<f32>, max_iterations: u32,...` — Configuration for the agent.
- pub `new` function L434-445 — `(model: impl Into<String>) -> Self` — Create a new config with the specified model.
- pub `with_max_tokens` function L448-451 — `(mut self, max_tokens: u32) -> Self` — Set max tokens.
- pub `with_temperature` function L454-457 — `(mut self, temperature: f32) -> Self` — Set temperature.
- pub `with_max_iterations` function L460-463 — `(mut self, max_iterations: u32) -> Self` — Set max iterations.
- pub `with_max_total_tokens` function L466-469 — `(mut self, max_total_tokens: usize) -> Self` — Set cumulative token budget.
- pub `with_timeout` function L472-475 — `(mut self, timeout: Duration) -> Self` — Set timeout.
- pub `with_system_prompt` function L478-481 — `(mut self, prompt: impl Into<String>) -> Self` — Set system prompt.
- pub `with_workspace` function L484-487 — `(mut self, path: impl Into<PathBuf>) -> Self` — Set the workspace path.
- pub `AgentResponse` struct L502-515 — `{ text: String, tool_calls: Vec<ToolCall>, tool_results: Vec<ToolResultRecord>, ...` — Response from an agent turn.
- pub `text` function L519-528 — `(content: impl Into<String>) -> Self` — Create a simple text response.
- pub `ResponseUsage` struct L542-547 — `{ input_tokens: u32, output_tokens: u32 }` — Token usage statistics.
- pub `new` function L551-556 — `(input_tokens: u32, output_tokens: u32) -> Self` — Create new usage stats.
- pub `total` function L559-561 — `(&self) -> u32` — Total tokens used.
- pub `serialize` function L806-811 — `(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>` — - [`AgentResponse`]: Agent output from a turn
- pub `deserialize` function L813-819 — `(deserializer: D) -> Result<Duration, D::Error>` — - [`AgentResponse`]: Agent output from a turn
-  `SessionId` type L24-39 — `= SessionId` — - [`AgentResponse`]: Agent output from a turn
-  `SessionId` type L41-45 — `impl Default for SessionId` — - [`AgentResponse`]: Agent output from a turn
-  `default` function L42-44 — `() -> Self` — - [`AgentResponse`]: Agent output from a turn
-  `SessionId` type L47-51 — `= SessionId` — - [`AgentResponse`]: Agent output from a turn
-  `fmt` function L48-50 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - [`AgentResponse`]: Agent output from a turn
-  `TurnId` type L57-72 — `= TurnId` — - [`AgentResponse`]: Agent output from a turn
-  `TurnId` type L74-78 — `impl Default for TurnId` — - [`AgentResponse`]: Agent output from a turn
-  `default` function L75-77 — `() -> Self` — - [`AgentResponse`]: Agent output from a turn
-  `TurnId` type L80-84 — `= TurnId` — - [`AgentResponse`]: Agent output from a turn
-  `fmt` function L81-83 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - [`AgentResponse`]: Agent output from a turn
-  `Turn` type L173-212 — `= Turn` — - [`AgentResponse`]: Agent output from a turn
-  `Session` type L241-380 — `= Session` — - [`AgentResponse`]: Agent output from a turn
-  `Session` type L382-386 — `impl Default for Session` — - [`AgentResponse`]: Agent output from a turn
-  `default` function L383-385 — `() -> Self` — - [`AgentResponse`]: Agent output from a turn
-  `AgentConfig` type L432-488 — `= AgentConfig` — - [`AgentResponse`]: Agent output from a turn
-  `AgentConfig` type L490-494 — `impl Default for AgentConfig` — - [`AgentResponse`]: Agent output from a turn
-  `default` function L491-493 — `() -> Self` — - [`AgentResponse`]: Agent output from a turn
-  `AgentResponse` type L517-529 — `= AgentResponse` — - [`AgentResponse`]: Agent output from a turn
-  `ResponseUsage` type L549-562 — `= ResponseUsage` — - [`AgentResponse`]: Agent output from a turn
-  `tests` module L569-800 — `-` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_id` function L573-581 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_turn_id` function L584-588 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_turn_creation` function L591-597 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_turn_completion` function L600-607 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_turn_tool_calls` function L610-629 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_creation` function L632-636 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_turns` function L639-657 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_metadata` function L660-676 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_agent_config` function L679-696 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_agent_config_default` function L699-704 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_agent_response` function L707-712 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_response_usage` function L715-720 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_serialization` function L723-733 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_context_preamble` function L736-752 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_context_preamble_not_in_turns` function L755-764 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_context_tracker` function L767-785 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `test_session_context_tracker_not_serialized` function L788-799 — `()` — - [`AgentResponse`]: Agent output from a turn
-  `humantime_serde` module L802-820 — `-` — - [`AgentResponse`]: Agent output from a turn

### crates/arawn-agent/src/indexing

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-agent/src/indexing/extraction.rs

- pub `ExtractionPrompt` struct L9 — `-` — Builds the extraction prompt for an LLM to extract entities, facts, and
- pub `build` function L15-30 — `(messages: &[(&str, &str)]) -> String` — Format a conversation history into an extraction prompt.
- pub `FactsOnlyPrompt` struct L96 — `-` — Builds a facts-only extraction prompt for hybrid mode.
- pub `build` function L103-123 — `(messages: &[(&str, &str)], entity_names: &[&str]) -> String` — Build a facts-only extraction prompt with NER entity context.
- pub `parse_extraction` function L155-172 — `(raw: &str) -> ExtractionResult` — Parse LLM output into an ExtractionResult.
-  `ExtractionPrompt` type L11-31 — `= ExtractionPrompt` — LLM extraction prompt and JSON parser.
-  `SYSTEM_INSTRUCTION` variable L33-60 — `: &str` — LLM extraction prompt and JSON parser.
-  `FEW_SHOT_EXAMPLE` variable L62-89 — `: &str` — LLM extraction prompt and JSON parser.
-  `FactsOnlyPrompt` type L98-124 — `= FactsOnlyPrompt` — LLM extraction prompt and JSON parser.
-  `FACTS_ONLY_INSTRUCTION` variable L126-147 — `: &str` — LLM extraction prompt and JSON parser.
-  `strip_code_fences` function L175-191 — `(s: &str) -> &str` — Strip markdown code fences from LLM output.
-  `extract_json_object` function L194-202 — `(s: &str) -> Option<&str>` — Try to find a top-level JSON object `{...}` in the text.
-  `tests` module L205-332 — `-` — LLM extraction prompt and JSON parser.
-  `test_build_prompt` function L209-219 — `()` — LLM extraction prompt and JSON parser.
-  `test_parse_valid_json` function L222-233 — `()` — LLM extraction prompt and JSON parser.
-  `test_parse_with_code_fences` function L236-248 — `()` — LLM extraction prompt and JSON parser.
-  `test_parse_with_surrounding_text` function L251-265 — `()` — LLM extraction prompt and JSON parser.
-  `test_parse_malformed_returns_empty` function L268-273 — `()` — LLM extraction prompt and JSON parser.
-  `test_parse_partial_json_missing_sections` function L276-281 — `()` — LLM extraction prompt and JSON parser.
-  `test_parse_empty_object` function L284-289 — `()` — LLM extraction prompt and JSON parser.
-  `test_strip_code_fences_plain` function L292-294 — `()` — LLM extraction prompt and JSON parser.
-  `test_strip_code_fences_json` function L297-299 — `()` — LLM extraction prompt and JSON parser.
-  `test_strip_code_fences_bare` function L302-304 — `()` — LLM extraction prompt and JSON parser.
-  `test_facts_only_prompt_build` function L307-314 — `()` — LLM extraction prompt and JSON parser.
-  `test_facts_only_prompt_no_entities` function L317-322 — `()` — LLM extraction prompt and JSON parser.
-  `test_extract_json_object` function L325-331 — `()` — LLM extraction prompt and JSON parser.

#### crates/arawn-agent/src/indexing/gliner.rs

- pub `GlinerEngine` struct L20-23 — `{ model: Mutex<GLiNER<SpanMode>>, threshold: f32 }` — GLiNER-based NER engine using span mode.
- pub `new` function L27-42 — `(config: &NerConfig) -> Result<Self, String>` — Create a new GlinerEngine from model and tokenizer file paths.
-  `GlinerEngine` type L25-43 — `= GlinerEngine` — This module is only compiled when the `gliner` feature is enabled.
-  `GlinerEngine` type L45-76 — `impl NerEngine for GlinerEngine` — This module is only compiled when the `gliner` feature is enabled.
-  `extract` function L46-75 — `(&self, texts: &[&str], entity_labels: &[&str]) -> Result<NerOutput, String>` — This module is only compiled when the `gliner` feature is enabled.

#### crates/arawn-agent/src/indexing/indexer.rs

- pub `IndexerConfig` struct L33-40 — `{ model: String, max_extraction_tokens: u32, max_summary_tokens: u32 }` — Configuration for the session indexer.
- pub `Completer` interface L54-56 — `{ fn complete() }` — Trait for LLM completion, enabling test mocking.
- pub `BackendCompleter` struct L59-61 — `{ backend: SharedBackend }` — Production completer that uses the real LLM backend.
- pub `new` function L64-66 — `(backend: SharedBackend) -> Self` — 5.
- pub `SessionIndexer` struct L96-102 — `{ store: Arc<MemoryStore>, completer: Arc<dyn Completer>, embedder: Option<Share...` — Orchestrates post-session indexing: extraction, graph storage, and summarization.
- pub `new` function L106-119 — `( store: Arc<MemoryStore>, completer: Arc<dyn Completer>, embedder: Option<Share...` — Create a new SessionIndexer with the given dependencies.
- pub `with_backend` function L122-134 — `( store: Arc<MemoryStore>, backend: SharedBackend, embedder: Option<SharedEmbedd...` — Create a SessionIndexer using a real LLM backend.
- pub `store` function L137-139 — `(&self) -> &Arc<MemoryStore>` — Get a reference to the underlying MemoryStore.
- pub `with_ner_engine` function L145-148 — `(mut self, engine: Arc<dyn NerEngine>) -> Self` — Set a local NER engine for hybrid extraction.
- pub `index_session` function L157-204 — `(&self, session_id: &str, messages: &[(&str, &str)]) -> IndexReport` — Run the full indexing pipeline for a session.
-  `IndexerConfig` type L42-50 — `impl Default for IndexerConfig` — 5.
-  `default` function L43-49 — `() -> Self` — 5.
-  `BackendCompleter` type L63-67 — `= BackendCompleter` — 5.
-  `BackendCompleter` type L70-93 — `impl Completer for BackendCompleter` — 5.
-  `complete` function L71-92 — `(&self, model: &str, prompt: &str, max_tokens: u32) -> Result<String, String>` — 5.
-  `SessionIndexer` type L104-471 — `= SessionIndexer` — 5.
-  `run_extraction` function L206-217 — `(&self, messages: &[(&str, &str)]) -> Result<ExtractionResult, String>` — 5.
-  `run_hybrid_extraction` function L220-290 — `( &self, ner: &dyn NerEngine, messages: &[(&str, &str)], ) -> ExtractionResult` — Hybrid extraction: NER for entities/relationships, LLM for facts only.
-  `store_entities` function L292-321 — `( &self, session_id: &str, entities: &[ExtractedEntity], report: &mut IndexRepor...` — 5.
-  `store_facts` function L323-369 — `( &self, session_id: &str, facts: &[ExtractedFact], report: &mut IndexReport, )` — 5.
-  `store_relationships` function L371-402 — `( &self, relationships: &[ExtractedRelationship], report: &mut IndexReport, )` — 5.
-  `store_summary` function L404-457 — `( &self, session_id: &str, messages: &[(&str, &str)], report: &mut IndexReport, ...` — 5.
-  `embed_text` function L459-470 — `(&self, text: &str) -> Option<Vec<f32>>` — 5.
-  `map_relationship_type` function L474-485 — `(label: &str) -> RelationshipType` — Map an extracted relationship label to a `RelationshipType`.
-  `tests` module L488-975 — `-` — 5.
-  `MockCompleter` struct L492-495 — `{ extraction_response: String, summary_response: String }` — Mock completer that returns pre-configured responses.
-  `MockCompleter` type L497-511 — `= MockCompleter` — 5.
-  `new` function L498-503 — `(extraction_json: &str, summary: &str) -> Self` — 5.
-  `failing` function L505-510 — `() -> Self` — 5.
-  `MockCompleter` type L514-538 — `impl Completer for MockCompleter` — 5.
-  `complete` function L515-537 — `( &self, _model: &str, prompt: &str, _max_tokens: u32, ) -> Result<String, Strin...` — 5.
-  `test_extraction_json` function L540-556 — `() -> String` — 5.
-  `test_indexer_config` function L558-563 — `() -> IndexerConfig` — 5.
-  `make_indexer` function L565-573 — `(completer: impl Completer + 'static) -> SessionIndexer` — 5.
-  `make_indexer_with_graph` function L575-584 — `(completer: impl Completer + 'static) -> SessionIndexer` — 5.
-  `test_index_session_empty_messages` function L587-594 — `()` — 5.
-  `test_index_session_facts_stored` function L597-629 — `()` — 5.
-  `test_index_session_with_graph` function L632-645 — `()` — 5.
-  `test_index_session_no_graph_skips_entities` function L648-660 — `()` — 5.
-  `test_index_session_extraction_failure_continues` function L663-675 — `()` — 5.
-  `test_index_session_fact_confidence_mapping` function L678-700 — `()` — 5.
-  `test_index_session_fact_reinforcement` function L703-741 — `()` — 5.
-  `test_index_session_fact_supersession` function L744-787 — `()` — 5.
-  `MockNer` struct L791-794 — `{ output: NerOutput, supports_rels: bool }` — 5.
-  `MockNer` type L796-813 — `impl NerEngine for MockNer` — 5.
-  `extract` function L797-799 — `(&self, _texts: &[&str], _entity_labels: &[&str]) -> Result<NerOutput, String>` — 5.
-  `supports_relations` function L801-803 — `(&self) -> bool` — 5.
-  `extract_relations` function L805-812 — `( &self, _texts: &[&str], _entity_labels: &[&str], _relation_labels: &[&str], ) ...` — 5.
-  `FailingNer` struct L815 — `-` — 5.
-  `FailingNer` type L817-821 — `impl NerEngine for FailingNer` — 5.
-  `extract` function L818-820 — `(&self, _texts: &[&str], _entity_labels: &[&str]) -> Result<NerOutput, String>` — 5.
-  `make_indexer_with_ner` function L823-835 — `( completer: impl Completer + 'static, ner: impl NerEngine + 'static, ) -> Sessi...` — 5.
-  `make_indexer_with_ner_and_graph` function L837-850 — `( completer: impl Completer + 'static, ner: impl NerEngine + 'static, ) -> Sessi...` — 5.
-  `test_hybrid_extraction_entities_from_ner` function L853-892 — `()` — 5.
-  `test_hybrid_extraction_with_graph_stores_ner_entities` function L895-924 — `()` — 5.
-  `test_hybrid_extraction_ner_failure_falls_back_to_llm` function L927-937 — `()` — 5.
-  `test_map_relationship_type` function L940-974 — `()` — 5.

#### crates/arawn-agent/src/indexing/mod.rs

- pub `extraction` module L3 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.
- pub `gliner` module L5 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.
- pub `indexer` module L6 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.
- pub `ner` module L7 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.
- pub `summarization` module L9 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.
-  `report` module L8 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.
-  `types` module L10 — `-` — Session indexing pipeline: extraction, summarization, and memory storage.

#### crates/arawn-agent/src/indexing/ner.rs

- pub `NerSpan` struct L13-20 — `{ text: String, label: String, score: f32 }` — A recognized entity span from NER inference.
- pub `NerRelation` struct L24-33 — `{ subject: String, relation: String, object: String, score: f32 }` — A recognized relationship between two entities.
- pub `NerOutput` struct L37-42 — `{ entities: Vec<NerSpan>, relations: Vec<NerRelation> }` — Output from NER engine inference.
- pub `ENTITY_LABELS` variable L45-54 — `: &[&str]` — Entity labels used for NER inference in Arawn's domain.
- pub `RELATION_LABELS` variable L57-66 — `: &[&str]` — Relation labels for relation extraction.
- pub `NerEngine` interface L72-100 — `{ fn extract(), fn supports_relations(), fn extract_relations() }` — Trait for local NER inference engines.
- pub `NerConfig` struct L104-111 — `{ model_path: String, tokenizer_path: String, threshold: f32 }` — Configuration for the NER engine.
- pub `ner_output_to_extracted` function L127-179 — `(output: &NerOutput, threshold: f32) -> NerExtraction` — Convert NER output to Arawn's extraction types.
- pub `NerExtraction` struct L183-186 — `{ entities: Vec<ExtractedEntity>, relationships: Vec<ExtractedRelationship> }` — Entities and relationships extracted by the NER engine.
-  `supports_relations` function L82-84 — `(&self) -> bool` — Whether this engine supports relation extraction.
-  `extract_relations` function L90-99 — `( &self, texts: &[&str], entity_labels: &[&str], relation_labels: &[&str], ) -> ...` — Run relation extraction on the given texts.
-  `NerConfig` type L113-121 — `impl Default for NerConfig` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `default` function L114-120 — `() -> Self` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `tests` module L189-412 — `-` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `MockNerEngine` struct L192-195 — `{ output: NerOutput, supports_rels: bool }` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `MockNerEngine` type L197-209 — `= MockNerEngine` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `new` function L198-203 — `(output: NerOutput) -> Self` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `with_relations` function L205-208 — `(mut self) -> Self` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `MockNerEngine` type L211-228 — `impl NerEngine for MockNerEngine` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `extract` function L212-214 — `(&self, _texts: &[&str], _entity_labels: &[&str]) -> Result<NerOutput, String>` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `supports_relations` function L216-218 — `(&self) -> bool` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `extract_relations` function L220-227 — `( &self, _texts: &[&str], _entity_labels: &[&str], _relation_labels: &[&str], ) ...` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_ner_output_to_extracted_entities` function L231-255 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_ner_output_filters_by_threshold` function L258-293 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_ner_output_deduplicates_entities` function L296-315 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_ner_output_empty` function L318-323 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_mock_ner_engine_extract` function L326-340 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_mock_ner_engine_relations` function L343-372 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_entity_labels_defined` function L375-381 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_relation_labels_defined` function L384-388 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_ner_config_default` function L391-395 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.
-  `test_context_cleaned_in_output` function L398-411 — `()` — extraction: fast local NER for entities/relationships, LLM for facts only.

#### crates/arawn-agent/src/indexing/report.rs

- pub `IndexReport` struct L5-20 — `{ entities_stored: usize, facts_inserted: usize, facts_reinforced: usize, facts_...` — Report summarizing the results of indexing a session.
- pub `total_facts` function L24-26 — `(&self) -> usize` — Total number of facts processed (inserted + reinforced + superseded).
- pub `has_errors` function L29-31 — `(&self) -> bool` — Whether any errors occurred during indexing.
-  `IndexReport` type L22-32 — `= IndexReport` — Index report types for session indexing pipeline results.
-  `IndexReport` type L34-49 — `= IndexReport` — Index report types for session indexing pipeline results.
-  `fmt` function L35-48 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Index report types for session indexing pipeline results.
-  `tests` module L52-100 — `-` — Index report types for session indexing pipeline results.
-  `test_report_default` function L56-62 — `()` — Index report types for session indexing pipeline results.
-  `test_report_total_facts` function L65-73 — `()` — Index report types for session indexing pipeline results.
-  `test_report_has_errors` function L76-81 — `()` — Index report types for session indexing pipeline results.
-  `test_report_display` function L84-99 — `()` — Index report types for session indexing pipeline results.

#### crates/arawn-agent/src/indexing/summarization.rs

- pub `SummarizationPrompt` struct L4 — `-` — Builds the summarization prompt for an LLM to generate a concise session summary.
- pub `build` function L11-30 — `(messages: &[(&str, &str)]) -> Option<String>` — Format a conversation history into a summarization prompt.
- pub `clean_summary` function L53-84 — `(raw: &str) -> String` — Clean up LLM summary output by stripping common wrapper patterns.
-  `SummarizationPrompt` type L6-31 — `= SummarizationPrompt` — Session summarization prompt and parser.
-  `SYSTEM_INSTRUCTION` variable L33-45 — `: &str` — Session summarization prompt and parser.
-  `tests` module L87-180 — `-` — Session summarization prompt and parser.
-  `test_build_prompt_basic` function L91-106 — `()` — Session summarization prompt and parser.
-  `test_build_prompt_empty_returns_none` function L109-111 — `()` — Session summarization prompt and parser.
-  `test_build_prompt_single_message` function L114-118 — `()` — Session summarization prompt and parser.
-  `test_build_prompt_contains_instructions` function L121-128 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_plain` function L131-136 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_strips_summary_prefix` function L139-141 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_strips_markdown_header` function L144-149 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_strips_code_fences` function L152-154 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_trims_whitespace` function L157-162 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_preserves_word_containing_summary` function L165-171 — `()` — Session summarization prompt and parser.
-  `test_clean_summary_strips_hash_summary_colon` function L174-179 — `()` — Session summarization prompt and parser.

#### crates/arawn-agent/src/indexing/types.rs

- pub `ExtractionResult` struct L7-17 — `{ entities: Vec<ExtractedEntity>, facts: Vec<ExtractedFact>, relationships: Vec<...` — Result of LLM extraction from a conversation.
- pub `ExtractedEntity` struct L21-29 — `{ name: String, entity_type: String, context: Option<String> }` — An entity extracted from conversation.
- pub `ExtractedFact` struct L33-43 — `{ subject: String, predicate: String, object: String, confidence: String }` — A fact extracted from conversation.
- pub `ExtractedRelationship` struct L51-58 — `{ from: String, relation: String, to: String }` — A relationship between two entities.
-  `default_confidence` function L45-47 — `() -> String` — Types for the extraction pipeline.
-  `tests` module L61-118 — `-` — Types for the extraction pipeline.
-  `test_extraction_result_deserialize` function L65-85 — `()` — Types for the extraction pipeline.
-  `test_extraction_result_missing_sections_default` function L88-94 — `()` — Types for the extraction pipeline.
-  `test_extraction_result_empty` function L97-103 — `()` — Types for the extraction pipeline.
-  `test_fact_default_confidence` function L106-110 — `()` — Types for the extraction pipeline.
-  `test_entity_optional_context` function L113-117 — `()` — Types for the extraction pipeline.

### crates/arawn-agent/src/prompt

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-agent/src/prompt/bootstrap.rs

- pub `DEFAULT_MAX_CHARS` variable L11 — `: usize` — Default maximum characters per bootstrap file before truncation.
- pub `BOOTSTRAP_FILES` variable L20 — `: &[&str]` — Standard bootstrap file names to look for.
- pub `BootstrapFile` struct L24-31 — `{ filename: String, content: String, truncated: bool }` — A single loaded bootstrap file.
- pub `BootstrapContext` struct L35-37 — `{ files: Vec<BootstrapFile> }` — Collection of loaded bootstrap context files.
- pub `new` function L41-43 — `() -> Self` — Create an empty bootstrap context.
- pub `load` function L56-58 — `(workspace: impl AsRef<Path>) -> io::Result<Self>` — Load bootstrap files from a workspace directory.
- pub `load_with_options` function L66-117 — `( workspace: impl AsRef<Path>, max_chars: usize, mut warn_fn: Option<F>, ) -> io...` — Load bootstrap files with custom options.
- pub `files` function L120-122 — `(&self) -> &[BootstrapFile]` — Get the loaded files.
- pub `is_empty` function L125-127 — `(&self) -> bool` — Check if any files were loaded.
- pub `len` function L130-132 — `(&self) -> usize` — Get the number of loaded files.
- pub `to_prompt_section` function L137-154 — `(&self) -> String` — Format the bootstrap context for inclusion in a system prompt.
- pub `add_file` function L157-165 — `(&mut self, filename: impl Into<String>, content: impl Into<String>)` — Add a file manually (for testing or custom files).
-  `HEAD_RATIO` variable L14 — `: f64` — Ratio of content to keep from the head when truncating.
-  `TAIL_RATIO` variable L17 — `: f64` — Ratio of content to keep from the tail when truncating.
-  `BootstrapContext` type L39-166 — `= BootstrapContext` — in system prompts.
-  `truncate_content` function L172-194 — `(content: &str, max_chars: usize) -> (String, bool)` — Truncate content if it exceeds max_chars.
-  `find_char_boundary` function L202-223 — `(s: &str, target: usize, search_backward: bool) -> usize` — Find a safe UTF-8 char boundary near the target position.
-  `tests` module L226-365 — `-` — in system prompts.
-  `test_empty_context` function L232-237 — `()` — in system prompts.
-  `test_load_nonexistent_dir` function L240-243 — `()` — in system prompts.
-  `test_load_empty_dir` function L246-250 — `()` — in system prompts.
-  `test_load_soul_md` function L253-263 — `()` — in system prompts.
-  `test_load_multiple_files` function L266-274 — `()` — in system prompts.
-  `test_truncation_under_limit` function L277-282 — `()` — in system prompts.
-  `test_truncation_over_limit` function L285-295 — `()` — in system prompts.
-  `test_truncation_unicode_boundary` function L298-306 — `()` — in system prompts.
-  `test_to_prompt_section_format` function L309-320 — `()` — in system prompts.
-  `test_to_prompt_section_shows_truncated` function L323-331 — `()` — in system prompts.
-  `test_warn_callback` function L334-349 — `()` — in system prompts.
-  `test_char_boundary_ascii` function L352-356 — `()` — in system prompts.
-  `test_char_boundary_unicode` function L359-364 — `()` — in system prompts.

#### crates/arawn-agent/src/prompt/builder.rs

- pub `ToolSummary` struct L16-21 — `{ name: String, description: String }` — A tool summary for prompt generation.
- pub `SystemPromptBuilder` struct L38-49 — `{ mode: PromptMode, identity: Option<(String, String)>, tools: Option<Vec<ToolSu...` — Builder for generating system prompts.
- pub `new` function L59-72 — `() -> Self` — Create a new builder with default settings.
- pub `with_mode` function L75-78 — `(mut self, mode: PromptMode) -> Self` — Set the prompt mode.
- pub `with_identity` function L85-92 — `( mut self, name: impl Into<String>, description: impl Into<String>, ) -> Self` — Set the agent identity.
- pub `with_tools` function L97-112 — `(mut self, registry: &ToolRegistry) -> Self` — Add tools from a registry.
- pub `with_tool_summaries` function L117-121 — `(mut self, summaries: Vec<ToolSummary>) -> Self` — Add tool summaries directly.
- pub `with_workspace` function L126-129 — `(mut self, path: impl AsRef<Path>) -> Self` — Set the workspace path.
- pub `with_datetime` function L136-140 — `(mut self, timezone: Option<&str>) -> Self` — Enable datetime section with optional timezone.
- pub `with_memory_hints` function L145-148 — `(mut self) -> Self` — Enable memory hints section.
- pub `with_bootstrap` function L153-156 — `(mut self, context: BootstrapContext) -> Self` — Add bootstrap context from workspace files.
- pub `with_plugin_prompts` function L162-165 — `(mut self, fragments: Vec<(String, String)>) -> Self` — Add plugin prompt fragments.
- pub `build` function L171-224 — `(self) -> String` — Build the final system prompt string.
-  `SystemPromptBuilder` type L51-55 — `impl Default for SystemPromptBuilder` — Provides a fluent builder for assembling system prompts from modular sections.
-  `default` function L52-54 — `() -> Self` — Provides a fluent builder for assembling system prompts from modular sections.
-  `SystemPromptBuilder` type L57-319 — `= SystemPromptBuilder` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_identity_section` function L230-234 — `(&self) -> Option<String>` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_tools_section` function L236-256 — `(&self) -> Option<String>` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_workspace_section` function L258-263 — `(&self) -> Option<String>` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_datetime_section` function L265-286 — `(&self) -> Option<String>` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_memory_section` function L288-298 — `(&self) -> String` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_think_section` function L300-311 — `() -> String` — Provides a fluent builder for assembling system prompts from modular sections.
-  `build_bootstrap_section` function L313-318 — `(&self) -> Option<String>` — Provides a fluent builder for assembling system prompts from modular sections.
-  `tests` module L322-534 — `-` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_default_empty` function L327-330 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_with_identity` function L333-339 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_with_tools_full_mode` function L342-355 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_with_tools_minimal_mode` function L358-372 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_with_workspace` function L375-383 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_with_datetime` function L386-394 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_with_memory_hints` function L397-405 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_builder_identity_mode` function L408-427 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_sections_joined_with_double_newline` function L430-437 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_think_section_included_when_tool_registered` function L440-451 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_think_section_omitted_when_no_think_tool` function L454-464 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_think_section_omitted_in_minimal_mode` function L467-477 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_tool_summaries_direct` function L480-492 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_plugin_prompts_included` function L495-511 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_plugin_prompts_empty_skipped` function L514-524 — `()` — Provides a fluent builder for assembling system prompts from modular sections.
-  `test_plugin_prompts_none` function L527-533 — `()` — Provides a fluent builder for assembling system prompts from modular sections.

#### crates/arawn-agent/src/prompt/mod.rs

-  `bootstrap` module L21 — `-` — This module provides a modular system for building agent system prompts.
-  `builder` module L22 — `-` — ```
-  `mode` module L23 — `-` — ```

#### crates/arawn-agent/src/prompt/mode.rs

- pub `PromptMode` enum L15-34 — `Full | Minimal | Identity` — Mode controlling prompt verbosity and sections.
- pub `include_tool_descriptions` function L38-40 — `(&self) -> bool` — Check if this mode includes tool descriptions.
- pub `include_datetime` function L43-45 — `(&self) -> bool` — Check if this mode includes datetime information.
- pub `include_memory_hints` function L48-50 — `(&self) -> bool` — Check if this mode includes memory hints.
- pub `include_bootstrap` function L53-55 — `(&self) -> bool` — Check if this mode includes bootstrap context.
- pub `include_workspace` function L58-60 — `(&self) -> bool` — Check if this mode includes workspace information.
-  `PromptMode` type L36-61 — `= PromptMode` — Different modes control which sections are included in the generated prompt.
-  `tests` module L64-111 — `-` — Different modes control which sections are included in the generated prompt.
-  `test_default_mode_is_full` function L68-70 — `()` — Different modes control which sections are included in the generated prompt.
-  `test_full_mode_includes_all` function L73-80 — `()` — Different modes control which sections are included in the generated prompt.
-  `test_minimal_mode_includes_subset` function L83-90 — `()` — Different modes control which sections are included in the generated prompt.
-  `test_identity_mode_includes_nothing` function L93-100 — `()` — Different modes control which sections are included in the generated prompt.
-  `test_serialization` function L103-110 — `()` — Different modes control which sections are included in the generated prompt.

### crates/arawn-agent/src/rlm

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-agent/src/rlm/integration_tests.rs

-  `mock_text_response` function L22-33 — `(text: &str) -> CompletionResponse` — together correctly.
-  `mock_text_response_with_usage` function L35-46 — `(text: &str, input: u32, output: u32) -> CompletionResponse` — together correctly.
-  `mock_tool_use_response` function L48-65 — `( tool_id: &str, tool_name: &str, args: serde_json::Value, ) -> CompletionRespon...` — together correctly.
-  `make_full_registry` function L68-86 — `() -> ToolRegistry` — Create a full tool registry with both read-only and write tools.
-  `make_spawner` function L88-90 — `(backend: MockBackend) -> Arc<RlmSpawner>` — together correctly.
-  `make_spawner_with_config` function L92-94 — `(backend: MockBackend, config: RlmConfig) -> Arc<RlmSpawner>` — together correctly.
-  `test_explore_tool_full_pipeline` function L101-124 — `()` — together correctly.
-  `test_explore_tool_multi_tool_research` function L127-149 — `()` — together correctly.
-  `test_explore_compaction_cycle` function L156-196 — `()` — together correctly.
-  `test_explore_multiple_compaction_cycles` function L199-242 — `()` — together correctly.
-  `test_explore_max_turns_enforced` function L249-272 — `()` — together correctly.
-  `test_explore_max_compactions_enforced` function L275-320 — `()` — together correctly.
-  `test_explore_token_budget_enforced` function L323-348 — `()` — together correctly.
-  `test_explore_excludes_write_tools` function L355-375 — `()` — together correctly.
-  `test_explore_includes_read_only_tools` function L378-397 — `()` — together correctly.
-  `test_explore_no_recursive_spawning` function L400-415 — `()` — together correctly.
-  `test_explore_custom_model_config` function L422-447 — `()` — together correctly.
-  `test_rlm_config_to_agent_config_model` function L450-465 — `()` — together correctly.
-  `test_rlm_default_config_model` function L468-479 — `()` — together correctly.
-  `test_rlm_toml_config_to_rlm_config` function L486-532 — `()` — together correctly.
-  `test_rlm_toml_defaults_preserve_agent_defaults` function L535-557 — `()` — together correctly.
-  `test_explore_tool_metadata_footer_format` function L564-586 — `()` — together correctly.
-  `test_explore_tool_compaction_metadata` function L589-627 — `()` — together correctly.
-  `test_explore_tool_truncated_metadata` function L630-654 — `()` — together correctly.

#### crates/arawn-agent/src/rlm/mod.rs

- pub `types` module L19 — `-` — ```
- pub `DEFAULT_READ_ONLY_TOOLS` variable L40-48 — `: &[&str]` — Default set of read-only tool names available to the RLM agent.
- pub `RlmSpawner` struct L59-68 — `{ backend: SharedBackend, compaction_backend: Option<SharedBackend>, tools: Tool...` — Spawns isolated RLM exploration agents.
- pub `new` function L72-79 — `(backend: SharedBackend, tools: ToolRegistry) -> Self` — Create a new spawner with default configuration.
- pub `with_config` function L82-85 — `(mut self, config: RlmConfig) -> Self` — Set the exploration configuration.
- pub `with_compaction_backend` function L88-91 — `(mut self, backend: SharedBackend) -> Self` — Set a separate backend for compaction (e.g., a cheaper model).
- pub `explore` function L98-157 — `(&self, query: &str) -> Result<ExplorationResult>` — Run an exploration for the given query.
-  `prompt` module L18 — `-` — The RLM module provides an isolated sub-agent that explores information
-  `integration_tests` module L22 — `-` — ```
-  `RlmSpawner` type L70-158 — `= RlmSpawner` — ```
-  `tests` module L165-326 — `-` — ```
-  `mock_text_response` function L170-181 — `(text: &str) -> CompletionResponse` — ```
-  `mock_tool_use_response` function L183-200 — `( tool_id: &str, tool_name: &str, args: serde_json::Value, ) -> CompletionRespon...` — ```
-  `make_full_registry` function L202-214 — `() -> ToolRegistry` — ```
-  `test_explore_simple_query` function L217-236 — `()` — ```
-  `test_explore_with_tool_calls` function L239-253 — `()` — ```
-  `test_explore_filters_tools` function L256-282 — `()` — ```
-  `test_explore_with_custom_config` function L285-301 — `()` — ```
-  `test_explore_metadata_tokens` function L304-315 — `()` — ```
-  `test_system_prompt_is_set` function L318-325 — `()` — ```

#### crates/arawn-agent/src/rlm/prompt.rs

- pub `RLM_SYSTEM_PROMPT` variable L9-40 — `: &str` — System prompt that instructs the agent to behave as a research explorer.

#### crates/arawn-agent/src/rlm/types.rs

- pub `RlmConfig` struct L5-24 — `{ model: String, max_iterations_per_turn: u32, max_total_tokens: Option<usize>, ...` — Configuration for an RLM exploration run.
- pub `ExplorationResult` struct L44-51 — `{ summary: String, truncated: bool, metadata: ExplorationMetadata }` — Result of an RLM exploration run.
- pub `ExplorationMetadata` struct L55-66 — `{ iterations_used: u32, input_tokens: u32, output_tokens: u32, compactions_perfo...` — Metadata from an RLM exploration run.
- pub `total_tokens` function L70-72 — `(&self) -> u32` — Total tokens used (input + output).
-  `RlmConfig` type L26-40 — `impl Default for RlmConfig` — Types for the RLM (Recursive Language Model) exploration module.
-  `default` function L27-39 — `() -> Self` — Types for the RLM (Recursive Language Model) exploration module.
-  `ExplorationMetadata` type L68-73 — `= ExplorationMetadata` — Types for the RLM (Recursive Language Model) exploration module.

### crates/arawn-agent/src/tool

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-agent/src/tool/command_validator.rs

- pub `CommandValidator` struct L16-19 — `{ blocked_patterns: Vec<(regex::Regex, String)> }` — Validates shell commands before execution as a defense-in-depth layer.
- pub `CommandValidation` enum L23-28 — `Allowed | Blocked` — Result of command validation.
- pub `validate` function L89-102 — `(&self, command: &str) -> CommandValidation` — Validate a shell command.
-  `CommandValidator` type L30-82 — `impl Default for CommandValidator` — Shell command validation as a defense-in-depth layer.
-  `default` function L31-81 — `() -> Self` — Shell command validation as a defense-in-depth layer.
-  `CommandValidator` type L84-111 — `= CommandValidator` — Shell command validation as a defense-in-depth layer.
-  `normalize` function L108-110 — `(command: &str) -> String` — Normalize a command for pattern matching.
-  `tests` module L114-375 — `-` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_rm_rf_root` function L118-136 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_system_control` function L139-162 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_sandbox_escape` function L165-187 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_kernel_module_manipulation` function L190-204 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_process_tracing` function L207-229 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_destructive_fs` function L232-242 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_fork_bomb` function L245-251 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_normalizes_whitespace` function L254-270 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_normalizes_case` function L273-291 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_allows_legitimate_commands` function L294-325 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_allows_rm_in_subdirectory` function L328-343 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_allows_piped_commands` function L346-360 — `()` — Shell command validation as a defense-in-depth layer.
-  `test_validator_blocks_dangerous_in_pipe` function L363-374 — `()` — Shell command validation as a defense-in-depth layer.

#### crates/arawn-agent/src/tool/context.rs

- pub `Tool` interface L24-45 — `{ fn name(), fn description(), fn parameters(), fn execute() }` — Trait for agent tools.
- pub `OutputSender` type L52 — `= tokio::sync::mpsc::UnboundedSender<String>` — Sender for streaming tool output chunks.
- pub `ToolContext` struct L67-82 — `{ session_id: SessionId, turn_id: TurnId, cancellation: CancellationToken, outpu...` — Context provided to tools during execution.
- pub `new` function L101-111 — `(session_id: SessionId, turn_id: TurnId) -> Self` — Create a new tool context.
- pub `with_cancellation` function L114-128 — `( session_id: SessionId, turn_id: TurnId, cancellation: CancellationToken, ) -> ...` — Create a context with a cancellation token.
- pub `with_fs_gate` function L131-134 — `(mut self, gate: SharedFsGate) -> Self` — Set the filesystem gate for workstream sandbox enforcement.
- pub `with_secret_resolver` function L137-140 — `(mut self, resolver: SharedSecretResolver) -> Self` — Set the secret resolver for `${{secrets.*}}` handle resolution.
- pub `with_streaming` function L143-147 — `(mut self, sender: OutputSender, tool_call_id: impl Into<String>) -> Self` — Add streaming output support to this context.
- pub `is_cancelled` function L150-152 — `(&self) -> bool` — Check if execution has been cancelled.
- pub `is_streaming` function L155-157 — `(&self) -> bool` — Check if streaming output is enabled.
- pub `send_output` function L161-167 — `(&self, content: impl Into<String>) -> bool` — Send streaming output chunk.
- pub `ToolResult` enum L203-221 — `Text | Json | Error` — Result of a tool execution.
- pub `text` function L225-229 — `(content: impl Into<String>) -> Self` — Create a text result.
- pub `json` function L232-234 — `(content: serde_json::Value) -> Self` — Create a JSON result.
- pub `error` function L237-242 — `(message: impl Into<String>) -> Self` — Create a recoverable error result.
- pub `fatal_error` function L245-250 — `(message: impl Into<String>) -> Self` — Create a non-recoverable error result.
- pub `is_error` function L253-255 — `(&self) -> bool` — Check if this result is an error.
- pub `is_success` function L258-260 — `(&self) -> bool` — Check if this result is successful.
- pub `to_llm_content` function L263-271 — `(&self) -> String` — Get the content as a string for LLM consumption.
- pub `sanitize` function L282-341 — `(self, config: &OutputConfig) -> Self` — Sanitize this result according to the given configuration.
- pub `sanitize_default` function L344-346 — `(self) -> Self` — Sanitize this result with default configuration.
- pub `was_truncated` function L349-355 — `(&self) -> bool` — Check if this result was truncated (looks for truncation indicator).
- pub `content_size` function L358-364 — `(&self) -> usize` — Get the size of the content in bytes.
-  `ToolContext` type L84-97 — `= ToolContext` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `fmt` function L85-96 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `ToolContext` type L99-168 — `= ToolContext` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `ToolContext` type L170-182 — `impl Default for ToolContext` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `default` function L171-181 — `() -> Self` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `ToolResult` type L223-365 — `= ToolResult` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `tests` module L368-507 — `-` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_text` function L372-377 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_json` function L380-386 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_error` function L389-394 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_serialization` function L397-402 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_context` function L405-414 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_text` function L416-426 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_text_truncated` function L429-434 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_json` function L437-447 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_json_truncated_becomes_text` function L450-459 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_error` function L462-472 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_binary_becomes_error` function L475-482 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_content_size` function L485-494 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.
-  `test_tool_result_sanitize_default` function L497-506 — `()` — Core tool types: the Tool trait, ToolContext, and ToolResult.

#### crates/arawn-agent/src/tool/execution.rs

- pub `execute` function L25-33 — `( &self, name: &str, params: serde_json::Value, ctx: &ToolContext, ) -> Result<T...` — Execute a tool by name.
- pub `execute_with_config` function L45-88 — `( &self, name: &str, params: serde_json::Value, ctx: &ToolContext, output_config...` — Execute a tool by name with custom output configuration.
- pub `execute_raw` function L94-135 — `( &self, name: &str, params: serde_json::Value, ctx: &ToolContext, ) -> Result<T...` — Execute a tool by name without sanitization.
-  `ToolRegistry` type L12-155 — `= ToolRegistry` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `resolve_secret_handles` function L142-154 — `( &self, params: serde_json::Value, ctx: &ToolContext, ) -> serde_json::Value` — Resolve `${{secrets.*}}` handles in tool parameters.
-  `tests` module L158-332 — `-` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `MockSecretResolver` struct L166-168 — `{ secrets: std::collections::HashMap<String, String> }` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `MockSecretResolver` type L170-179 — `= MockSecretResolver` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `new` function L171-178 — `(pairs: &[(&str, &str)]) -> Self` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `MockSecretResolver` type L181-188 — `= MockSecretResolver` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `resolve` function L182-184 — `(&self, name: &str) -> Option<String>` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `names` function L185-187 — `(&self) -> Vec<String>` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `ctx_with_resolver` function L190-195 — `(resolver: MockSecretResolver) -> ToolContext` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_registry_execute_sanitizes` function L198-217 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_registry_execute_raw_no_sanitize` function L220-238 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_secret_handles_resolved_in_params` function L245-259 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_secret_handles_no_resolver_passes_through` function L262-275 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_secret_handles_no_handles_in_params` function L278-291 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_secret_handles_resolved_in_execute_with_config` function L294-311 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.
-  `test_secret_handles_unknown_secret_left_as_is` function L314-327 — `()` — Implements execute, execute_with_config, execute_raw, and secret handle resolution.

#### crates/arawn-agent/src/tool/gate.rs

-  `ToolRegistry` type L13-112 — `= ToolRegistry` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `validate_tool_paths` function L18-56 — `( &self, tool_name: &str, mut params: serde_json::Value, gate: &SharedFsGate, ) ...` — Validate and rewrite file paths in tool params against the filesystem gate.
-  `execute_shell_sandboxed` function L62-111 — `( &self, _tool: &dyn Tool, params: &serde_json::Value, _ctx: &ToolContext, gate:...` — Execute a shell tool through the OS-level sandbox.
-  `tests` module L115-714 — `-` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `MockFsGate` struct L130-139 — `{ allowed_read: Vec<std::path::PathBuf>, allowed_write: Vec<std::path::PathBuf>,...` — Mock filesystem gate for testing enforcement logic.
-  `MockFsGate` type L141-165 — `= MockFsGate` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `new` function L142-149 — `(work_dir: impl Into<std::path::PathBuf>) -> Self` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `allow_read` function L151-154 — `(mut self, path: impl Into<std::path::PathBuf>) -> Self` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `allow_write` function L156-159 — `(mut self, path: impl Into<std::path::PathBuf>) -> Self` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `with_shell_result` function L161-164 — `(self, result: arawn_types::SandboxOutput) -> Self` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `MockFsGate` type L168-218 — `= MockFsGate` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `validate_read` function L169-182 — `( &self, path: &std::path::Path, ) -> std::result::Result<std::path::PathBuf, ar...` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `validate_write` function L184-197 — `( &self, path: &std::path::Path, ) -> std::result::Result<std::path::PathBuf, ar...` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `working_dir` function L199-201 — `(&self) -> &std::path::Path` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `sandbox_execute` function L203-217 — `( &self, _command: &str, _timeout: Option<std::time::Duration>, ) -> std::result...` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `ctx_with_gate` function L220-225 — `(gate: impl arawn_types::FsGate + 'static) -> ToolContext` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_is_gated_tool` function L228-240 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_deny_by_default_no_gate` function L243-262 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_deny_by_default_all_gated_tools` function L265-295 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_non_gated_tool_passes_through_without_gate` function L298-316 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_file_read_allowed` function L319-335 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_file_read_denied` function L338-354 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_file_write_allowed` function L357-371 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_file_write_denied` function L374-390 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_glob_allowed` function L393-408 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_glob_denied` function L411-426 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_grep_denied` function L429-444 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_routed_through_sandbox` function L447-473 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_sandbox_failure` function L476-496 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_execute_raw_deny_by_default` function L499-519 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_execute_raw_allowed_with_gate` function L522-539 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_execute_raw_non_gated_passes_through` function L542-554 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_file_read_no_path_param_passes_through` function L557-573 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_sandbox_combined_output` function L576-599 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_timeout_passed` function L602-618 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `MockSecretResolver` struct L622-624 — `{ secrets: std::collections::HashMap<String, String> }` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `MockSecretResolver` type L626-635 — `= MockSecretResolver` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `new` function L627-634 — `(pairs: &[(&str, &str)]) -> Self` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `MockSecretResolver` type L637-644 — `= MockSecretResolver` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `resolve` function L638-640 — `(&self, name: &str) -> Option<String>` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `names` function L641-643 — `(&self) -> Vec<String>` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `ctx_with_resolver` function L646-651 — `(resolver: MockSecretResolver) -> ToolContext` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_blocked_command_rejected` function L654-675 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_blocked_command_case_bypass` function L678-694 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.
-  `test_gate_shell_blocked_command_whitespace_bypass` function L697-713 — `()` — Validates file paths and routes shell commands through the OS-level sandbox.

#### crates/arawn-agent/src/tool/mod.rs

-  `command_validator` module L28 — `-` — This module defines the [`Tool`] trait that all agent tools must implement,
-  `context` module L29 — `-` — ```
-  `execution` module L30 — `-` — ```
-  `gate` module L31 — `-` — ```
-  `output` module L32 — `-` — ```
-  `params` module L33 — `-` — ```
-  `registry` module L34 — `-` — ```
-  `validation` module L35 — `-` — ```

#### crates/arawn-agent/src/tool/output.rs

- pub `DEFAULT_MAX_OUTPUT_SIZE` variable L8 — `: usize` — Default maximum output size in bytes (100KB).
- pub `OutputConfig` struct L15-26 — `{ max_size_bytes: usize, truncation_message: String, strip_control_chars: bool, ...` — Configuration for sanitizing tool output.
- pub `with_max_size` function L42-47 — `(max_size_bytes: usize) -> Self` — Create a new output config with the given size limit.
- pub `for_shell` function L50-52 — `() -> Self` — Configuration for shell output (100KB default).
- pub `for_file_read` function L55-57 — `() -> Self` — Configuration for file read output (500KB default).
- pub `for_web_fetch` function L60-62 — `() -> Self` — Configuration for web fetch output (200KB default).
- pub `for_search` function L65-67 — `() -> Self` — Configuration for search output (50KB default).
- pub `with_truncation_message` function L70-73 — `(mut self, message: impl Into<String>) -> Self` — Set a custom truncation message.
- pub `without_control_char_stripping` function L76-79 — `(mut self) -> Self` — Disable control character stripping.
- pub `OutputSanitizationError` enum L84-102 — `BinaryContent | MalformedJson` — Error type for output sanitization failures.
- pub `sanitize_output` function L113-170 — `( input: &str, config: &OutputConfig, ) -> std::result::Result<(String, bool), O...` — Sanitize a string according to the output configuration.
- pub `validate_json_output` function L175-206 — `( value: &serde_json::Value, ) -> std::result::Result<(), OutputSanitizationErro...` — Validate that a JSON value has the expected structure.
-  `OutputConfig` type L28-38 — `impl Default for OutputConfig` — Output configuration and sanitization for tool results.
-  `default` function L29-37 — `() -> Self` — Output configuration and sanitization for tool results.
-  `OutputConfig` type L40-80 — `= OutputConfig` — Output configuration and sanitization for tool results.
-  `check_depth` function L183-196 — `(value: &serde_json::Value, depth: usize, max_depth: usize) -> bool` — Output configuration and sanitization for tool results.
-  `MAX_JSON_DEPTH` variable L198 — `: usize` — Output configuration and sanitization for tool results.
-  `tests` module L209-335 — `-` — Output configuration and sanitization for tool results.
-  `test_output_config_defaults` function L217-223 — `()` — Output configuration and sanitization for tool results.
-  `test_output_config_per_tool` function L226-238 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_normal` function L241-246 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_strips_null_bytes` function L249-254 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_strips_control_chars` function L257-263 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_preserves_newlines_tabs` function L266-271 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_truncates` function L274-281 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_truncates_utf8_safe` function L284-292 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_detects_binary` function L295-304 — `()` — Output configuration and sanitization for tool results.
-  `test_sanitize_output_few_nulls_ok` function L307-313 — `()` — Output configuration and sanitization for tool results.
-  `test_validate_json_output_valid` function L316-320 — `()` — Output configuration and sanitization for tool results.
-  `test_validate_json_output_deep_nesting` function L323-334 — `()` — Output configuration and sanitization for tool results.

#### crates/arawn-agent/src/tool/params.rs

- pub `ShellParams` struct L11-22 — `{ command: String, pty: bool, stream: bool, cwd: Option<String>, timeout_secs: O...` — Validated parameters for the shell tool.
- pub `FileReadParams` struct L71-74 — `{ path: String }` — Validated parameters for file read tool.
- pub `FileWriteParams` struct L99-106 — `{ path: String, content: String, append: bool }` — Validated parameters for file write tool.
- pub `WebSearchParams` struct L134-139 — `{ query: String, max_results: u64 }` — Validated parameters for web search tool.
- pub `ThinkParams` struct L183-186 — `{ thought: String }` — Validated parameters for think tool.
- pub `MemoryStoreParams` struct L211-218 — `{ content: String, memory_type: Option<String>, importance: Option<f64> }` — Validated parameters for memory store tool.
- pub `MemoryRecallParams` struct L257-264 — `{ query: String, limit: u64, memory_type: Option<String> }` — Validated parameters for memory recall tool.
- pub `DelegateParams` struct L309-314 — `{ task: String, agent_type: Option<String> }` — Validated parameters for delegate tool.
-  `ShellParams` type L24-67 — `= ShellParams` — Typed parameter structs for built-in tools.
-  `Error` type L25 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L27-66 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `FileReadParams` type L76-95 — `= FileReadParams` — Typed parameter structs for built-in tools.
-  `Error` type L77 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L79-94 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `FileWriteParams` type L108-130 — `= FileWriteParams` — Typed parameter structs for built-in tools.
-  `Error` type L109 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L111-129 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `WebSearchParams` type L141-179 — `= WebSearchParams` — Typed parameter structs for built-in tools.
-  `Error` type L142 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L144-178 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `ThinkParams` type L188-207 — `= ThinkParams` — Typed parameter structs for built-in tools.
-  `Error` type L189 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L191-206 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `MemoryStoreParams` type L220-253 — `= MemoryStoreParams` — Typed parameter structs for built-in tools.
-  `Error` type L221 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L223-252 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `MemoryRecallParams` type L266-305 — `= MemoryRecallParams` — Typed parameter structs for built-in tools.
-  `Error` type L267 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L269-304 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `DelegateParams` type L316-336 — `= DelegateParams` — Typed parameter structs for built-in tools.
-  `Error` type L317 — `= ParameterValidationError` — Typed parameter structs for built-in tools.
-  `try_from` function L319-335 — `(params: serde_json::Value) -> std::result::Result<Self, Self::Error>` — Typed parameter structs for built-in tools.
-  `tests` module L339-622 — `-` — Typed parameter structs for built-in tools.
-  `test_shell_params_valid` function L347-361 — `()` — Typed parameter structs for built-in tools.
-  `test_shell_params_minimal` function L364-372 — `()` — Typed parameter structs for built-in tools.
-  `test_shell_params_missing_command` function L375-385 — `()` — Typed parameter structs for built-in tools.
-  `test_shell_params_empty_command` function L388-398 — `()` — Typed parameter structs for built-in tools.
-  `test_shell_params_timeout_zero` function L401-411 — `()` — Typed parameter structs for built-in tools.
-  `test_shell_params_timeout_too_large` function L414-424 — `()` — Typed parameter structs for built-in tools.
-  `test_file_read_params_valid` function L427-431 — `()` — Typed parameter structs for built-in tools.
-  `test_file_read_params_missing_path` function L434-441 — `()` — Typed parameter structs for built-in tools.
-  `test_file_read_params_empty_path` function L444-451 — `()` — Typed parameter structs for built-in tools.
-  `test_file_write_params_valid` function L454-464 — `()` — Typed parameter structs for built-in tools.
-  `test_file_write_params_missing_content` function L467-477 — `()` — Typed parameter structs for built-in tools.
-  `test_web_search_params_valid` function L480-485 — `()` — Typed parameter structs for built-in tools.
-  `test_web_search_params_default_max` function L488-492 — `()` — Typed parameter structs for built-in tools.
-  `test_web_search_params_max_zero` function L495-505 — `()` — Typed parameter structs for built-in tools.
-  `test_web_search_params_max_too_large` function L508-518 — `()` — Typed parameter structs for built-in tools.
-  `test_think_params_valid` function L521-525 — `()` — Typed parameter structs for built-in tools.
-  `test_think_params_empty` function L528-538 — `()` — Typed parameter structs for built-in tools.
-  `test_memory_store_params_valid` function L541-551 — `()` — Typed parameter structs for built-in tools.
-  `test_memory_store_params_importance_invalid` function L554-564 — `()` — Typed parameter structs for built-in tools.
-  `test_memory_store_params_importance_negative` function L567-577 — `()` — Typed parameter structs for built-in tools.
-  `test_memory_recall_params_valid` function L580-590 — `()` — Typed parameter structs for built-in tools.
-  `test_memory_recall_params_limit_zero` function L593-600 — `()` — Typed parameter structs for built-in tools.
-  `test_delegate_params_valid` function L603-611 — `()` — Typed parameter structs for built-in tools.
-  `test_delegate_params_empty_task` function L614-621 — `()` — Typed parameter structs for built-in tools.

#### crates/arawn-agent/src/tool/registry.rs

- pub `ToolRegistry` struct L24-28 — `{ tools: HashMap<String, Arc<dyn Tool>>, output_overrides: HashMap<String, Outpu...` — Registry for managing available tools.
- pub `new` function L41-46 — `() -> Self` — Create a new empty registry.
- pub `set_output_config` function L53-55 — `(&mut self, name: impl Into<String>, config: OutputConfig)` — Set a per-tool output config override.
- pub `register` function L70-73 — `(&mut self, tool: T)` — Register a tool.
- pub `register_arc` function L76-79 — `(&mut self, tool: Arc<dyn Tool>)` — Register a tool from an Arc.
- pub `get` function L82-84 — `(&self, name: &str) -> Option<Arc<dyn Tool>>` — Get a tool by name.
- pub `contains` function L87-89 — `(&self, name: &str) -> bool` — Check if a tool exists.
- pub `names` function L92-94 — `(&self) -> Vec<&str>` — Get all tool names.
- pub `len` function L97-99 — `(&self) -> usize` — Get the number of registered tools.
- pub `is_empty` function L102-104 — `(&self) -> bool` — Check if the registry is empty.
- pub `to_llm_definitions` function L107-114 — `(&self) -> Vec<arawn_llm::ToolDefinition>` — Convert all tools to LLM tool definitions.
- pub `filtered_by_names` function L121-144 — `(&self, names: &[&str]) -> ToolRegistry` — Create a new registry containing only tools whose names are in the allowlist.
- pub `output_config_for` function L150-164 — `(&self, name: &str) -> OutputConfig` — Get the output config for a tool by name.
- pub `MockTool` struct L184-190 — `{ name: String, description: String, parameters: serde_json::Value, response: st...` — A mock tool for testing.
- pub `new` function L195-206 — `(name: impl Into<String>) -> Self` — Create a new mock tool.
- pub `with_description` function L209-212 — `(mut self, description: impl Into<String>) -> Self` — Set the description.
- pub `with_parameters` function L215-218 — `(mut self, parameters: serde_json::Value) -> Self` — Set the parameters schema.
- pub `with_response` function L221-224 — `(self, response: ToolResult) -> Self` — Set the response to return.
- pub `calls` function L227-229 — `(&self) -> Vec<serde_json::Value>` — Get the calls that were made to this tool.
- pub `call_count` function L232-234 — `(&self) -> usize` — Get the number of calls made.
- pub `clear_calls` function L237-239 — `(&self)` — Clear recorded calls.
-  `ToolRegistry` type L30-165 — `= ToolRegistry` — Tool registry for managing available tools.
-  `ToolRegistry` type L167-173 — `= ToolRegistry` — Tool registry for managing available tools.
-  `fmt` function L168-172 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Tool registry for managing available tools.
-  `MockTool` type L193-240 — `= MockTool` — Tool registry for managing available tools.
-  `MockTool` type L244-269 — `impl Tool for MockTool` — Tool registry for managing available tools.
-  `name` function L245-247 — `(&self) -> &str` — Tool registry for managing available tools.
-  `description` function L249-251 — `(&self) -> &str` — Tool registry for managing available tools.
-  `parameters` function L253-255 — `(&self) -> serde_json::Value` — Tool registry for managing available tools.
-  `execute` function L257-268 — `(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult>` — Tool registry for managing available tools.
-  `tests` module L272-517 — `-` — Tool registry for managing available tools.
-  `test_registry_empty` function L278-283 — `()` — Tool registry for managing available tools.
-  `test_registry_register_and_get` function L286-300 — `()` — Tool registry for managing available tools.
-  `test_registry_names` function L303-312 — `()` — Tool registry for managing available tools.
-  `test_registry_to_llm_definitions` function L315-333 — `()` — Tool registry for managing available tools.
-  `test_mock_tool_execution` function L336-347 — `()` — Tool registry for managing available tools.
-  `test_registry_execute` function L350-366 — `()` — Tool registry for managing available tools.
-  `test_mock_tool_clear_calls` function L369-376 — `()` — Tool registry for managing available tools.
-  `test_registry_output_config_for` function L378-389 — `()` — Tool registry for managing available tools.
-  `test_registry_output_config_override` function L392-409 — `()` — Tool registry for managing available tools.
-  `test_registry_output_config_override_all_aliases` function L412-428 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_includes_matching` function L430-444 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_excludes_non_matching` function L447-455 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_ignores_unknown` function L458-466 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_preserves_original` function L469-480 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_carries_output_overrides` function L483-493 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_llm_definitions` function L496-506 — `()` — Tool registry for managing available tools.
-  `test_filtered_by_names_empty_allowlist` function L509-516 — `()` — Tool registry for managing available tools.

#### crates/arawn-agent/src/tool/validation.rs

- pub `ParameterValidationError` enum L20-66 — `MissingRequired | InvalidType | OutOfRange | InvalidValue | Multiple` — Error type for tool parameter validation failures.
- pub `missing` function L70-72 — `(name: &'static str, hint: &'static str) -> Self` — Create a missing required parameter error.
- pub `invalid_type` function L75-85 — `( name: &'static str, expected: &'static str, actual: impl Into<String>, ) -> Se...` — Create an invalid type error.
- pub `out_of_range` function L88-98 — `( name: &'static str, value: impl ToString, constraint: impl Into<String>, ) -> ...` — Create an out of range error.
- pub `invalid_value` function L101-111 — `( name: &'static str, value: impl Into<String>, message: impl Into<String>, ) ->...` — Create an invalid value error.
- pub `multiple` function L114-116 — `(errors: Vec<ParameterValidationError>) -> Self` — Create from multiple errors.
- pub `parameter_name` function L119-127 — `(&self) -> Option<&str>` — Get the parameter name associated with this error (if single error).
- pub `ParamResult` type L137 — `= std::result::Result<T, ParameterValidationError>` — Result type for parameter validation.
- pub `ParamExt` interface L150-174 — `{ fn required_str(), fn optional_str(), fn required_i64(), fn optional_i64(), fn...` — Helper trait for extracting and validating parameters from JSON.
-  `ParameterValidationError` type L68-128 — `= ParameterValidationError` — Parameter validation error types and helper traits.
-  `AgentError` type L130-134 — `= AgentError` — Parameter validation error types and helper traits.
-  `from` function L131-133 — `(err: ParameterValidationError) -> Self` — Parameter validation error types and helper traits.
-  `required_str` function L177-181 — `(&self, name: &'static str, hint: &'static str) -> ParamResult<&str>` — Parameter validation error types and helper traits.
-  `optional_str` function L183-185 — `(&self, name: &str) -> Option<&str>` — Parameter validation error types and helper traits.
-  `required_i64` function L187-191 — `(&self, name: &'static str, hint: &'static str) -> ParamResult<i64>` — Parameter validation error types and helper traits.
-  `optional_i64` function L193-195 — `(&self, name: &str, default: i64) -> i64` — Parameter validation error types and helper traits.
-  `optional_u64` function L197-199 — `(&self, name: &str, default: u64) -> u64` — Parameter validation error types and helper traits.
-  `required_bool` function L201-205 — `(&self, name: &'static str, hint: &'static str) -> ParamResult<bool>` — Parameter validation error types and helper traits.
-  `optional_bool` function L207-209 — `(&self, name: &str, default: bool) -> bool` — Parameter validation error types and helper traits.
-  `optional_array` function L211-213 — `(&self, name: &str) -> Option<&Vec<serde_json::Value>>` — Parameter validation error types and helper traits.
-  `tests` module L217-354 — `-` — Parameter validation error types and helper traits.
-  `test_param_validation_error_missing` function L221-226 — `()` — Parameter validation error types and helper traits.
-  `test_param_validation_error_invalid_type` function L229-234 — `()` — Parameter validation error types and helper traits.
-  `test_param_validation_error_out_of_range` function L237-242 — `()` — Parameter validation error types and helper traits.
-  `test_param_validation_error_invalid_value` function L245-253 — `()` — Parameter validation error types and helper traits.
-  `test_param_validation_error_multiple` function L256-266 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_required_str` function L269-281 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_optional_str` function L284-288 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_required_i64` function L291-303 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_optional_i64` function L306-310 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_optional_u64` function L313-317 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_required_bool` function L320-332 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_optional_bool` function L335-339 — `()` — Parameter validation error types and helper traits.
-  `test_param_ext_optional_array` function L342-346 — `()` — Parameter validation error types and helper traits.
-  `test_param_validation_error_into_agent_error` function L349-353 — `()` — Parameter validation error types and helper traits.

### crates/arawn-agent/src/tools

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-agent/src/tools/catalog.rs

- pub `CatalogTool` struct L45-48 — `{ catalog: Arc<RwLock<RuntimeCatalog>>, executor: Arc<ScriptExecutor> }` — Agent-facing tool for runtime catalog management.
- pub `new` function L52-54 — `(catalog: Arc<RwLock<RuntimeCatalog>>, executor: Arc<ScriptExecutor>) -> Self` — Create a new catalog tool backed by the given catalog and executor.
-  `validate_name` function L23-40 — `(name: &str) -> std::result::Result<(), String>` — Validate a runtime or workflow name for safe use as a filename component.
-  `CatalogTool` type L50-280 — `= CatalogTool` — from the runtime catalog.
-  `action_list` function L56-78 — `(&self) -> ToolResult` — from the runtime catalog.
-  `action_compile` function L80-152 — `(&self, params: &Value) -> ToolResult` — from the runtime catalog.
-  `action_register` function L154-212 — `(&self, params: &Value) -> ToolResult` — from the runtime catalog.
-  `action_inspect` function L214-240 — `(&self, params: &Value) -> ToolResult` — from the runtime catalog.
-  `action_remove` function L242-279 — `(&self, params: &Value) -> ToolResult` — from the runtime catalog.
-  `CatalogTool` type L283-347 — `impl Tool for CatalogTool` — from the runtime catalog.
-  `name` function L284-286 — `(&self) -> &str` — from the runtime catalog.
-  `description` function L288-292 — `(&self) -> &str` — from the runtime catalog.
-  `parameters` function L294-322 — `(&self) -> Value` — from the runtime catalog.
-  `execute` function L324-346 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — from the runtime catalog.
-  `tests` module L350-783 — `-` — from the runtime catalog.
-  `make_executor` function L356-358 — `(tmp: &TempDir) -> Arc<ScriptExecutor>` — from the runtime catalog.
-  `setup` function L360-366 — `() -> (CatalogTool, TempDir)` — from the runtime catalog.
-  `setup_with_entries` function L368-404 — `() -> (CatalogTool, TempDir)` — from the runtime catalog.
-  `test_list_empty` function L407-416 — `()` — from the runtime catalog.
-  `test_list_with_entries` function L419-434 — `()` — from the runtime catalog.
-  `test_inspect_existing` function L437-449 — `()` — from the runtime catalog.
-  `test_inspect_nonexistent` function L452-461 — `()` — from the runtime catalog.
-  `test_register_new_runtime` function L464-496 — `()` — from the runtime catalog.
-  `test_register_missing_wasm` function L499-515 — `()` — from the runtime catalog.
-  `test_remove_custom_runtime` function L518-536 — `()` — from the runtime catalog.
-  `test_remove_builtin_refused` function L539-548 — `()` — from the runtime catalog.
-  `test_remove_nonexistent` function L551-560 — `()` — from the runtime catalog.
-  `test_unknown_action` function L563-572 — `()` — from the runtime catalog.
-  `test_missing_action` function L575-581 — `()` — from the runtime catalog.
-  `test_parameters_schema` function L584-591 — `()` — from the runtime catalog.
-  `test_register_name_with_path_separator` function L596-615 — `()` — from the runtime catalog.
-  `test_register_name_with_dotdot` function L618-637 — `()` — from the runtime catalog.
-  `test_register_name_starting_with_dot` function L640-659 — `()` — from the runtime catalog.
-  `test_register_empty_name` function L662-681 — `()` — from the runtime catalog.
-  `test_register_missing_name` function L684-699 — `()` — from the runtime catalog.
-  `test_inspect_missing_name` function L702-711 — `()` — from the runtime catalog.
-  `test_remove_missing_name` function L714-723 — `()` — from the runtime catalog.
-  `test_compile_missing_name` function L726-738 — `()` — from the runtime catalog.
-  `test_compile_missing_source_path` function L741-750 — `()` — from the runtime catalog.
-  `test_compile_nonexistent_source` function L753-773 — `()` — from the runtime catalog.
-  `test_action_is_number` function L776-782 — `()` — from the runtime catalog.

#### crates/arawn-agent/src/tools/delegate.rs

- pub `DelegateTool` struct L34-36 — `{ spawner: SharedSubagentSpawner }` — Tool for delegating tasks to subagents.
- pub `new` function L48-50 — `(spawner: SharedSubagentSpawner) -> Self` — Create a new delegate tool with the given subagent spawner.
- pub `available_agents` function L53-55 — `(&self) -> Vec<SubagentInfo>` — List available subagents.
-  `DelegateTool` type L38-44 — `= DelegateTool` — with constrained tool sets and custom system prompts.
-  `fmt` function L39-43 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — with constrained tool sets and custom system prompts.
-  `DelegateTool` type L46-65 — `= DelegateTool` — with constrained tool sets and custom system prompts.
-  `format_available_agents` function L58-64 — `(agents: &[String]) -> String` — Format a list of available agent names for error messages.
-  `DelegateTool` type L68-178 — `impl Tool for DelegateTool` — with constrained tool sets and custom system prompts.
-  `name` function L69-71 — `(&self) -> &str` — with constrained tool sets and custom system prompts.
-  `description` function L73-78 — `(&self) -> &str` — with constrained tool sets and custom system prompts.
-  `parameters` function L80-108 — `(&self) -> Value` — with constrained tool sets and custom system prompts.
-  `execute` function L110-177 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — with constrained tool sets and custom system prompts.
-  `tests` module L185-431 — `-` — with constrained tool sets and custom system prompts.
-  `MockSpawner` struct L190-192 — `{ agents: Vec<SubagentInfo> }` — Mock spawner for testing.
-  `MockSpawner` type L194-213 — `= MockSpawner` — with constrained tool sets and custom system prompts.
-  `new` function L195-212 — `() -> Self` — with constrained tool sets and custom system prompts.
-  `MockSpawner` type L216-258 — `= MockSpawner` — with constrained tool sets and custom system prompts.
-  `list_agents` function L217-219 — `(&self) -> Vec<SubagentInfo>` — with constrained tool sets and custom system prompts.
-  `delegate` function L221-244 — `( &self, agent_name: &str, task: &str, _context: Option<&str>, _max_turns: Optio...` — with constrained tool sets and custom system prompts.
-  `delegate_background` function L246-257 — `( &self, agent_name: &str, _task: &str, _context: Option<&str>, _parent_session_...` — with constrained tool sets and custom system prompts.
-  `test_delegate_tool_metadata` function L261-277 — `()` — with constrained tool sets and custom system prompts.
-  `test_delegate_blocking_success` function L280-300 — `()` — with constrained tool sets and custom system prompts.
-  `test_delegate_unknown_agent` function L303-325 — `()` — with constrained tool sets and custom system prompts.
-  `test_delegate_background` function L328-349 — `()` — with constrained tool sets and custom system prompts.
-  `test_delegate_missing_agent_param` function L352-367 — `()` — with constrained tool sets and custom system prompts.
-  `test_delegate_missing_task_param` function L370-385 — `()` — with constrained tool sets and custom system prompts.
-  `test_delegate_with_context` function L388-406 — `()` — with constrained tool sets and custom system prompts.
-  `test_list_available_agents` function L409-417 — `()` — with constrained tool sets and custom system prompts.
-  `test_format_available_agents_empty` function L420-423 — `()` — with constrained tool sets and custom system prompts.
-  `test_format_available_agents` function L426-430 — `()` — with constrained tool sets and custom system prompts.

#### crates/arawn-agent/src/tools/explore.rs

- pub `ExploreTool` struct L23-25 — `{ spawner: Arc<RlmSpawner> }` — Tool that spawns an RLM exploration agent to research a query.
- pub `new` function L29-31 — `(spawner: Arc<RlmSpawner>) -> Self` — Create a new explore tool backed by the given spawner.
-  `ExploreTool` type L27-32 — `= ExploreTool` — delegate research tasks to an isolated exploration sub-agent.
-  `ExploreTool` type L35-107 — `impl Tool for ExploreTool` — delegate research tasks to an isolated exploration sub-agent.
-  `name` function L36-38 — `(&self) -> &str` — delegate research tasks to an isolated exploration sub-agent.
-  `description` function L40-46 — `(&self) -> &str` — delegate research tasks to an isolated exploration sub-agent.
-  `parameters` function L48-59 — `(&self) -> Value` — delegate research tasks to an isolated exploration sub-agent.
-  `execute` function L61-106 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — delegate research tasks to an isolated exploration sub-agent.
-  `tests` module L114-212 — `-` — delegate research tasks to an isolated exploration sub-agent.
-  `mock_text_response` function L119-130 — `(text: &str) -> CompletionResponse` — delegate research tasks to an isolated exploration sub-agent.
-  `make_spawner` function L132-137 — `(backend: MockBackend) -> Arc<RlmSpawner>` — delegate research tasks to an isolated exploration sub-agent.
-  `test_tool_definition` function L140-151 — `()` — delegate research tasks to an isolated exploration sub-agent.
-  `test_explore_returns_summary` function L154-174 — `()` — delegate research tasks to an isolated exploration sub-agent.
-  `test_explore_missing_query` function L177-189 — `()` — delegate research tasks to an isolated exploration sub-agent.
-  `test_explore_empty_query` function L192-200 — `()` — delegate research tasks to an isolated exploration sub-agent.
-  `test_explore_registerable` function L203-211 — `()` — delegate research tasks to an isolated exploration sub-agent.

#### crates/arawn-agent/src/tools/file.rs

- pub `FileReadTool` struct L60-63 — `{ base_dir: Option<String> }` — Tool for reading file contents.
- pub `new` function L67-69 — `() -> Self` — Create a new file read tool.
- pub `with_base_dir` function L72-76 — `(base_dir: impl Into<String>) -> Self` — Create a file read tool restricted to a base directory.
- pub `FileWriteTool` struct L182-189 — `{ base_dir: Option<String>, allow_create: bool, allow_overwrite: bool }` — Tool for writing file contents.
- pub `new` function L193-199 — `() -> Self` — Create a new file write tool with default settings.
- pub `with_base_dir` function L202-205 — `(mut self, base_dir: impl Into<String>) -> Self` — Create a file write tool restricted to a base directory.
- pub `allow_create` function L208-211 — `(mut self, allow: bool) -> Self` — Set whether creating new files is allowed.
- pub `allow_overwrite` function L214-217 — `(mut self, allow: bool) -> Self` — Set whether overwriting existing files is allowed.
-  `reject_traversal` function L19-29 — `(path: &Path) -> std::result::Result<(), crate::error::AgentError>` — Reject paths that contain `..` (parent directory) traversal components.
-  `normalize_path` function L35-52 — `(path: &Path) -> PathBuf` — Resolve `..` and `.` components lexically (without filesystem access).
-  `FileReadTool` type L65-112 — `= FileReadTool` — Provides tools for reading and writing files.
-  `resolve_path` function L79-111 — `(&self, path: &str) -> Result<std::path::PathBuf>` — Validate and resolve the file path.
-  `FileReadTool` type L115-174 — `impl Tool for FileReadTool` — Provides tools for reading and writing files.
-  `name` function L116-118 — `(&self) -> &str` — Provides tools for reading and writing files.
-  `description` function L120-122 — `(&self) -> &str` — Provides tools for reading and writing files.
-  `parameters` function L124-135 — `(&self) -> Value` — Provides tools for reading and writing files.
-  `execute` function L137-173 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides tools for reading and writing files.
-  `FileWriteTool` type L191-275 — `= FileWriteTool` — Provides tools for reading and writing files.
-  `resolve_path` function L220-274 — `(&self, path: &str) -> Result<std::path::PathBuf>` — Validate and resolve the file path for writing.
-  `FileWriteTool` type L278-377 — `impl Tool for FileWriteTool` — Provides tools for reading and writing files.
-  `name` function L279-281 — `(&self) -> &str` — Provides tools for reading and writing files.
-  `description` function L283-285 — `(&self) -> &str` — Provides tools for reading and writing files.
-  `parameters` function L287-307 — `(&self) -> Value` — Provides tools for reading and writing files.
-  `execute` function L309-376 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides tools for reading and writing files.
-  `tests` module L384-667 — `-` — Provides tools for reading and writing files.
-  `test_file_read_tool_metadata` function L389-397 — `()` — Provides tools for reading and writing files.
-  `test_file_write_tool_metadata` function L400-409 — `()` — Provides tools for reading and writing files.
-  `test_file_read_success` function L412-427 — `()` — Provides tools for reading and writing files.
-  `test_file_read_not_found` function L430-441 — `()` — Provides tools for reading and writing files.
-  `test_file_read_with_base_dir` function L444-458 — `()` — Provides tools for reading and writing files.
-  `test_file_write_success` function L461-482 — `()` — Provides tools for reading and writing files.
-  `test_file_write_append` function L485-507 — `()` — Provides tools for reading and writing files.
-  `test_file_write_no_create` function L510-530 — `()` — Provides tools for reading and writing files.
-  `test_file_write_no_overwrite` function L533-554 — `()` — Provides tools for reading and writing files.
-  `test_reject_traversal_blocks_dotdot` function L561-565 — `()` — Provides tools for reading and writing files.
-  `test_reject_traversal_allows_normal_paths` function L568-572 — `()` — Provides tools for reading and writing files.
-  `test_normalize_path_resolves_dotdot` function L575-588 — `()` — Provides tools for reading and writing files.
-  `test_file_write_traversal_rejected_no_base` function L591-612 — `()` — Provides tools for reading and writing files.
-  `test_file_write_traversal_rejected_with_base` function L615-632 — `()` — Provides tools for reading and writing files.
-  `test_file_read_traversal_rejected` function L635-645 — `()` — Provides tools for reading and writing files.
-  `test_file_write_base_dir_traversal_nonexistent_parent` function L648-666 — `()` — Provides tools for reading and writing files.

#### crates/arawn-agent/src/tools/memory.rs

- pub `MemorySearchTool` struct L27-30 — `{ store: Option<Arc<MemoryStore>> }` — Tool for searching the agent's memory/knowledge store.
- pub `new` function L40-42 — `() -> Self` — Create a new memory search tool (disconnected).
- pub `with_store` function L45-47 — `(store: Arc<MemoryStore>) -> Self` — Create a memory search tool backed by a real memory store.
-  `MemorySearchTool` type L32-36 — `impl Default for MemorySearchTool` — Provides a tool for searching the agent's memory/knowledge store.
-  `default` function L33-35 — `() -> Self` — Provides a tool for searching the agent's memory/knowledge store.
-  `MemorySearchTool` type L38-48 — `= MemorySearchTool` — Provides a tool for searching the agent's memory/knowledge store.
-  `MemorySearchTool` type L51-177 — `impl Tool for MemorySearchTool` — Provides a tool for searching the agent's memory/knowledge store.
-  `name` function L52-54 — `(&self) -> &str` — Provides a tool for searching the agent's memory/knowledge store.
-  `description` function L56-58 — `(&self) -> &str` — Provides a tool for searching the agent's memory/knowledge store.
-  `parameters` function L60-88 — `(&self) -> Value` — Provides a tool for searching the agent's memory/knowledge store.
-  `execute` function L90-176 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides a tool for searching the agent's memory/knowledge store.
-  `parse_time_range` function L183-190 — `(s: &str) -> TimeRange` — Provides a tool for searching the agent's memory/knowledge store.
-  `parse_content_type_filter` function L192-203 — `(memory_type: &str) -> Option<Vec<ContentType>>` — Provides a tool for searching the agent's memory/knowledge store.
-  `tests` module L210-346 — `-` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_memory_search_tool_metadata` function L214-223 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_memory_search_disconnected` function L226-238 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_memory_search_with_store` function L241-268 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_memory_search_with_time_range` function L271-295 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_memory_search_empty_results` function L298-311 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_memory_search_missing_query` function L314-320 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_parse_time_range` function L323-329 — `()` — Provides a tool for searching the agent's memory/knowledge store.
-  `test_parse_content_type_filter` function L332-345 — `()` — Provides a tool for searching the agent's memory/knowledge store.

#### crates/arawn-agent/src/tools/mod.rs

-  `catalog` module L12 — `-` — This module provides the core tools that give the agent basic capabilities:
-  `delegate` module L13 — `-` — - Subagent delegation
-  `explore` module L14 — `-` — - Subagent delegation
-  `file` module L15 — `-` — - Subagent delegation
-  `memory` module L16 — `-` — - Subagent delegation
-  `note` module L17 — `-` — - Subagent delegation
-  `search` module L18 — `-` — - Subagent delegation
-  `shell` module L19 — `-` — - Subagent delegation
-  `think` module L20 — `-` — - Subagent delegation
-  `web` module L21 — `-` — - Subagent delegation
-  `workflow` module L22 — `-` — - Subagent delegation

#### crates/arawn-agent/src/tools/note.rs

- pub `Note` struct L20-29 — `{ title: String, content: String, created_at: chrono::DateTime<chrono::Utc>, upd...` — A single note entry.
- pub `new` function L33-41 — `(title: impl Into<String>, content: impl Into<String>) -> Self` — Create a new note.
- pub `update` function L44-47 — `(&mut self, content: impl Into<String>)` — Update the note content.
- pub `NoteStorage` type L51 — `= Arc<RwLock<HashMap<String, Note>>>` — Shared storage for notes.
- pub `new_note_storage` function L54-56 — `() -> NoteStorage` — Create a new note storage.
- pub `NoteTool` struct L64-66 — `{ storage: NoteStorage }` — Tool for creating and managing notes.
- pub `new` function L70-74 — `() -> Self` — Create a new note tool with its own storage.
- pub `with_storage` function L77-79 — `(storage: NoteStorage) -> Self` — Create a note tool with shared storage.
- pub `storage` function L82-84 — `(&self) -> &NoteStorage` — Get the underlying storage.
- pub `get_all_notes` function L87-89 — `(&self) -> HashMap<String, Note>` — Get all notes (for inspection/testing).
- pub `get_note` function L92-94 — `(&self, title: &str) -> Option<Note>` — Get a specific note by title.
-  `Note` type L31-48 — `= Note` — Provides a tool for creating and managing notes/memory during a session.
-  `NoteTool` type L68-95 — `= NoteTool` — Provides a tool for creating and managing notes/memory during a session.
-  `NoteTool` type L97-101 — `impl Default for NoteTool` — Provides a tool for creating and managing notes/memory during a session.
-  `default` function L98-100 — `() -> Self` — Provides a tool for creating and managing notes/memory during a session.
-  `NoteTool` type L104-158 — `impl Tool for NoteTool` — Provides a tool for creating and managing notes/memory during a session.
-  `name` function L105-107 — `(&self) -> &str` — Provides a tool for creating and managing notes/memory during a session.
-  `description` function L109-111 — `(&self) -> &str` — Provides a tool for creating and managing notes/memory during a session.
-  `parameters` function L113-133 — `(&self) -> Value` — Provides a tool for creating and managing notes/memory during a session.
-  `execute` function L135-157 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides a tool for creating and managing notes/memory during a session.
-  `NoteTool` type L160-286 — `= NoteTool` — Provides a tool for creating and managing notes/memory during a session.
-  `create_note` function L161-189 — `(&self, params: &Value) -> Result<ToolResult>` — Provides a tool for creating and managing notes/memory during a session.
-  `update_note` function L191-217 — `(&self, params: &Value) -> Result<ToolResult>` — Provides a tool for creating and managing notes/memory during a session.
-  `get_note_action` function L219-239 — `(&self, params: &Value) -> Result<ToolResult>` — Provides a tool for creating and managing notes/memory during a session.
-  `list_notes` function L241-268 — `(&self) -> Result<ToolResult>` — Provides a tool for creating and managing notes/memory during a session.
-  `delete_note` function L270-285 — `(&self, params: &Value) -> Result<ToolResult>` — Provides a tool for creating and managing notes/memory during a session.
-  `tests` module L293-577 — `-` — Provides a tool for creating and managing notes/memory during a session.
-  `test_note_tool_metadata` function L297-305 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_note_creation` function L308-313 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_note_update` function L316-326 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_create_note` function L329-351 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_create_duplicate_note` function L354-385 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_update_note` function L388-422 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_update_nonexistent_note` function L425-443 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_get_note` function L446-477 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_list_notes` function L480-504 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_list_empty_notes` function L507-515 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_delete_note` function L518-551 — `()` — Provides a tool for creating and managing notes/memory during a session.
-  `test_shared_storage` function L554-576 — `()` — Provides a tool for creating and managing notes/memory during a session.

#### crates/arawn-agent/src/tools/search.rs

- pub `GlobTool` struct L20-27 — `{ base_dir: Option<PathBuf>, max_results: usize, max_depth: usize }` — Tool for finding files matching glob patterns.
- pub `new` function L31-37 — `() -> Self` — Create a new glob tool.
- pub `with_base_dir` function L40-43 — `(mut self, dir: impl Into<PathBuf>) -> Self` — Create a glob tool restricted to a base directory.
- pub `with_max_results` function L46-49 — `(mut self, max: usize) -> Self` — Set maximum number of results.
- pub `with_max_depth` function L52-55 — `(mut self, depth: usize) -> Self` — Set maximum traversal depth.
- pub `GrepTool` struct L221-232 — `{ base_dir: Option<PathBuf>, max_results: usize, max_depth: usize, max_file_size...` — Tool for searching file contents with regex.
- pub `new` function L236-244 — `() -> Self` — Create a new grep tool.
- pub `with_base_dir` function L247-250 — `(mut self, dir: impl Into<PathBuf>) -> Self` — Create a grep tool restricted to a base directory.
- pub `with_max_results` function L253-256 — `(mut self, max: usize) -> Self` — Set maximum number of results.
- pub `with_context_lines` function L259-262 — `(mut self, lines: usize) -> Self` — Set context lines to show before/after matches.
-  `GlobTool` type L29-95 — `= GlobTool` — Provides tools for searching files by pattern and content.
-  `resolve_dir` function L58-73 — `(&self, dir: Option<&str>) -> PathBuf` — Resolve the search directory.
-  `calculate_walk_depth` function L80-94 — `(&self, pattern: &str) -> usize` — Calculate the optimal walk depth for a pattern.
-  `GlobTool` type L97-101 — `impl Default for GlobTool` — Provides tools for searching files by pattern and content.
-  `default` function L98-100 — `() -> Self` — Provides tools for searching files by pattern and content.
-  `GlobTool` type L104-205 — `impl Tool for GlobTool` — Provides tools for searching files by pattern and content.
-  `name` function L105-107 — `(&self) -> &str` — Provides tools for searching files by pattern and content.
-  `description` function L109-111 — `(&self) -> &str` — Provides tools for searching files by pattern and content.
-  `parameters` function L113-128 — `(&self) -> Value` — Provides tools for searching files by pattern and content.
-  `execute` function L130-204 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides tools for searching files by pattern and content.
-  `GrepMatch` struct L213-217 — `{ file: String, line_number: usize, line: String }` — A single grep match.
-  `GrepTool` type L234-342 — `= GrepTool` — Provides tools for searching files by pattern and content.
-  `resolve_dir` function L265-280 — `(&self, dir: Option<&str>) -> PathBuf` — Resolve the search directory.
-  `should_search_file` function L283-313 — `(&self, path: &Path) -> bool` — Check if a file should be searched.
-  `search_file` function L316-341 — `(&self, path: &Path, regex: &Regex, base_dir: &Path) -> Vec<GrepMatch>` — Search a single file.
-  `GrepTool` type L344-348 — `impl Default for GrepTool` — Provides tools for searching files by pattern and content.
-  `default` function L345-347 — `() -> Self` — Provides tools for searching files by pattern and content.
-  `GrepTool` type L351-494 — `impl Tool for GrepTool` — Provides tools for searching files by pattern and content.
-  `name` function L352-354 — `(&self) -> &str` — Provides tools for searching files by pattern and content.
-  `description` function L356-358 — `(&self) -> &str` — Provides tools for searching files by pattern and content.
-  `parameters` function L360-384 — `(&self) -> Value` — Provides tools for searching files by pattern and content.
-  `execute` function L386-493 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides tools for searching files by pattern and content.
-  `tests` module L501-777 — `-` — Provides tools for searching files by pattern and content.
-  `test_glob_tool_metadata` function L507-514 — `()` — Provides tools for searching files by pattern and content.
-  `test_grep_tool_metadata` function L517-524 — `()` — Provides tools for searching files by pattern and content.
-  `test_calculate_walk_depth` function L527-546 — `()` — Provides tools for searching files by pattern and content.
-  `test_calculate_walk_depth_respects_max` function L549-557 — `()` — Provides tools for searching files by pattern and content.
-  `test_glob_find_files` function L560-581 — `()` — Provides tools for searching files by pattern and content.
-  `test_glob_recursive` function L584-605 — `()` — Provides tools for searching files by pattern and content.
-  `test_glob_non_recursive_excludes_nested` function L608-633 — `()` — Provides tools for searching files by pattern and content.
-  `test_glob_invalid_pattern` function L636-646 — `()` — Provides tools for searching files by pattern and content.
-  `test_grep_find_matches` function L649-668 — `()` — Provides tools for searching files by pattern and content.
-  `test_grep_case_insensitive` function L671-697 — `()` — Provides tools for searching files by pattern and content.
-  `test_grep_file_pattern` function L700-724 — `()` — Provides tools for searching files by pattern and content.
-  `test_grep_regex` function L727-743 — `()` — Provides tools for searching files by pattern and content.
-  `test_grep_invalid_regex` function L746-757 — `()` — Provides tools for searching files by pattern and content.
-  `test_should_search_file` function L760-776 — `()` — Provides tools for searching files by pattern and content.

#### crates/arawn-agent/src/tools/shell.rs

- pub `ShellConfig` struct L29-42 — `{ timeout: Duration, working_dir: Option<String>, allowed_commands: Vec<String>,...` — Configuration for shell command execution.
- pub `new` function L70-72 — `() -> Self` — Create a new shell configuration with defaults.
- pub `with_timeout` function L75-78 — `(mut self, timeout: Duration) -> Self` — Set the command timeout.
- pub `with_working_dir` function L81-84 — `(mut self, dir: impl Into<String>) -> Self` — Set the working directory.
- pub `with_allowed_commands` function L87-90 — `(mut self, commands: Vec<String>) -> Self` — Set allowed commands (whitelist).
- pub `block_command` function L93-96 — `(mut self, command: impl Into<String>) -> Self` — Add a blocked command.
- pub `with_max_output_size` function L99-102 — `(mut self, size: usize) -> Self` — Set maximum output size.
- pub `with_pty_size` function L105-108 — `(mut self, rows: u16, cols: u16) -> Self` — Set PTY terminal size.
- pub `SharedWorkingDirs` type L116 — `= Arc<Mutex<std::collections::HashMap<String, PathBuf>>>` — Shared working directory state across sessions.
- pub `ShellTool` struct L120-124 — `{ config: ShellConfig, working_dirs: SharedWorkingDirs }` — Tool for executing shell commands.
- pub `new` function L128-133 — `() -> Self` — Create a new shell tool with default configuration.
- pub `with_config` function L136-141 — `(config: ShellConfig) -> Self` — Create a shell tool with custom configuration.
-  `ShellConfig` type L44-66 — `impl Default for ShellConfig` — for commands that need terminal emulation (colored output, interactive prompts).
-  `default` function L45-65 — `() -> Self` — for commands that need terminal emulation (colored output, interactive prompts).
-  `ShellConfig` type L68-109 — `= ShellConfig` — for commands that need terminal emulation (colored output, interactive prompts).
-  `ShellTool` type L126-327 — `= ShellTool` — for commands that need terminal emulation (colored output, interactive prompts).
-  `get_working_dir` function L144-153 — `(&self, session_id: &str) -> Option<PathBuf>` — Get the working directory for a session.
-  `set_working_dir` function L156-160 — `(&self, session_id: &str, dir: PathBuf)` — Set the working directory for a session.
-  `execute_pty_with_callback` function L163-271 — `( &self, command: &str, working_dir: Option<&PathBuf>, timeout_duration: Duratio...` — Execute command in PTY mode with optional streaming callback.
-  `execute_pty` function L274-281 — `( &self, command: &str, working_dir: Option<&PathBuf>, timeout_duration: Duratio...` — Execute command in PTY mode (non-streaming).
-  `is_command_allowed` function L287-313 — `(&self, command: &str) -> bool` — Check if a command is allowed.
-  `truncate_output` function L316-326 — `(&self, output: String) -> String` — Truncate output if it exceeds the maximum size.
-  `ShellTool` type L329-333 — `impl Default for ShellTool` — for commands that need terminal emulation (colored output, interactive prompts).
-  `default` function L330-332 — `() -> Self` — for commands that need terminal emulation (colored output, interactive prompts).
-  `ShellTool` type L336-493 — `impl Tool for ShellTool` — for commands that need terminal emulation (colored output, interactive prompts).
-  `name` function L337-339 — `(&self) -> &str` — for commands that need terminal emulation (colored output, interactive prompts).
-  `description` function L341-343 — `(&self) -> &str` — for commands that need terminal emulation (colored output, interactive prompts).
-  `parameters` function L345-372 — `(&self) -> Value` — for commands that need terminal emulation (colored output, interactive prompts).
-  `execute` function L374-492 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — for commands that need terminal emulation (colored output, interactive prompts).
-  `ShellTool` type L495-755 — `= ShellTool` — for commands that need terminal emulation (colored output, interactive prompts).
-  `extract_cd_target` function L498-514 — `(&self, command: &str) -> Option<String>` — Extract the target path from a cd command, if it is one.
-  `resolve_cd_path` function L517-546 — `(&self, target: &str, current_dir: &Option<PathBuf>) -> PathBuf` — Resolve a cd target path to an absolute path.
-  `parse_cd_command` function L551-555 — `(&self, command: &str, current_dir: &Option<PathBuf>) -> Option<PathBuf>` — Check if this is a cd command and return the new directory path.
-  `execute_standard` function L558-622 — `( &self, command: &str, working_dir: Option<&PathBuf>, timeout_duration: Duratio...` — Standard process execution (non-PTY).
-  `execute_standard_streaming` function L626-754 — `( &self, command: &str, working_dir: Option<&PathBuf>, timeout_duration: Duratio...` — Streaming standard process execution.
-  `tests` module L762-1134 — `-` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_tool_metadata` function L766-778 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_config_defaults` function L781-787 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_command_blocking` function L790-802 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_command_whitelist` function L805-821 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_echo` function L824-835 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_pwd` function L838-846 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_working_dir` function L849-861 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_explicit_cwd` function L864-883 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_cd_persistence` function L886-909 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_cd_nonexistent` function L912-926 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_blocked_command` function L929-940 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_failed_command` function L943-954 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_timeout` function L957-969 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_custom_timeout` function L972-989 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_pty_echo` function L992-1009 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_pty_colored_output` function L1012-1032 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_streaming` function L1035-1067 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_shell_streaming_pty` function L1070-1101 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_output_truncation` function L1104-1113 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_parse_cd_command` function L1116-1127 — `()` — for commands that need terminal emulation (colored output, interactive prompts).
-  `test_pty_size_config` function L1130-1133 — `()` — for commands that need terminal emulation (colored output, interactive prompts).

#### crates/arawn-agent/src/tools/think.rs

- pub `ThinkTool` struct L25-27 — `{ store: Arc<MemoryStore> }` — Tool for persisting internal reasoning as Thought memories.
- pub `new` function L31-33 — `(store: Arc<MemoryStore>) -> Self` — Create a new think tool backed by the given memory store.
-  `ThinkTool` type L29-34 — `= ThinkTool` — for recall in subsequent turns but not shown to the user.
-  `ThinkTool` type L37-82 — `impl Tool for ThinkTool` — for recall in subsequent turns but not shown to the user.
-  `name` function L38-40 — `(&self) -> &str` — for recall in subsequent turns but not shown to the user.
-  `description` function L42-44 — `(&self) -> &str` — for recall in subsequent turns but not shown to the user.
-  `parameters` function L46-57 — `(&self) -> Value` — for recall in subsequent turns but not shown to the user.
-  `execute` function L59-81 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — for recall in subsequent turns but not shown to the user.
-  `tests` module L89-161 — `-` — for recall in subsequent turns but not shown to the user.
-  `create_test_tool` function L92-95 — `() -> ThinkTool` — for recall in subsequent turns but not shown to the user.
-  `test_think_tool_metadata` function L98-106 — `()` — for recall in subsequent turns but not shown to the user.
-  `test_think_stores_thought` function L109-135 — `()` — for recall in subsequent turns but not shown to the user.
-  `test_think_missing_param` function L138-150 — `()` — for recall in subsequent turns but not shown to the user.
-  `test_think_empty_thought` function L153-160 — `()` — for recall in subsequent turns but not shown to the user.

#### crates/arawn-agent/src/tools/web.rs

- pub `WebFetchConfig` struct L24-35 — `{ timeout: Duration, max_size: usize, user_agent: String, extract_text: bool, ma...` — Configuration for web fetching.
- pub `WebFetchTool` struct L52-55 — `{ client: Client, config: WebFetchConfig }` — Tool for fetching web page content.
- pub `new` function L59-68 — `() -> Self` — Create a new web fetch tool with default configuration.
- pub `with_config` function L71-79 — `(config: WebFetchConfig) -> Self` — Create a web fetch tool with custom configuration.
- pub `SearchProvider` enum L664-673 — `Brave | Serper | Tavily | DuckDuckGo` — Web search provider configuration.
- pub `WebSearchConfig` struct L677-684 — `{ provider: SearchProvider, max_results: usize, timeout: Duration }` — Configuration for web search.
- pub `SearchResult` struct L698-702 — `{ title: String, url: String, snippet: String }` — A single search result.
- pub `WebSearchTool` struct L706-709 — `{ client: Client, config: WebSearchConfig }` — Tool for searching the web.
- pub `new` function L713-721 — `() -> Self` — Create a new web search tool with default configuration (DuckDuckGo).
- pub `with_config` function L724-731 — `(config: WebSearchConfig) -> Self` — Create a web search tool with custom configuration.
- pub `brave` function L734-741 — `(api_key: impl Into<String>) -> Self` — Create a web search tool with Brave Search.
- pub `serper` function L744-751 — `(api_key: impl Into<String>) -> Self` — Create a web search tool with Serper.
- pub `tavily` function L754-761 — `(api_key: impl Into<String>) -> Self` — Create a web search tool with Tavily.
-  `WebFetchConfig` type L37-48 — `impl Default for WebFetchConfig` — Provides tools for web search and URL fetching.
-  `default` function L38-47 — `() -> Self` — Provides tools for web search and URL fetching.
-  `WebFetchTool` type L57-169 — `= WebFetchTool` — Provides tools for web search and URL fetching.
-  `extract_text_from_html` function L82-142 — `(&self, html: &str) -> String` — Extract readable text from HTML.
-  `extract_title` function L145-155 — `(&self, html: &str) -> Option<String>` — Extract title from HTML.
-  `extract_description` function L158-168 — `(&self, html: &str) -> Option<String>` — Extract meta description from HTML.
-  `WebFetchTool` type L171-175 — `impl Default for WebFetchTool` — Provides tools for web search and URL fetching.
-  `default` function L172-174 — `() -> Self` — Provides tools for web search and URL fetching.
-  `WebFetchTool` type L178-655 — `impl Tool for WebFetchTool` — Provides tools for web search and URL fetching.
-  `name` function L179-181 — `(&self) -> &str` — Provides tools for web search and URL fetching.
-  `description` function L183-185 — `(&self) -> &str` — Provides tools for web search and URL fetching.
-  `parameters` function L187-233 — `(&self) -> Value` — Provides tools for web search and URL fetching.
-  `execute` function L235-654 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides tools for web search and URL fetching.
-  `WebSearchConfig` type L686-694 — `impl Default for WebSearchConfig` — Provides tools for web search and URL fetching.
-  `default` function L687-693 — `() -> Self` — Provides tools for web search and URL fetching.
-  `WebSearchTool` type L711-946 — `= WebSearchTool` — Provides tools for web search and URL fetching.
-  `search_brave` function L763-806 — `(&self, query: &str, api_key: &str) -> Result<Vec<SearchResult>>` — Provides tools for web search and URL fetching.
-  `search_serper` function L808-849 — `(&self, query: &str, api_key: &str) -> Result<Vec<SearchResult>>` — Provides tools for web search and URL fetching.
-  `search_tavily` function L851-892 — `(&self, query: &str, api_key: &str) -> Result<Vec<SearchResult>>` — Provides tools for web search and URL fetching.
-  `search_duckduckgo` function L894-945 — `(&self, query: &str) -> Result<Vec<SearchResult>>` — Provides tools for web search and URL fetching.
-  `WebSearchTool` type L948-952 — `impl Default for WebSearchTool` — Provides tools for web search and URL fetching.
-  `default` function L949-951 — `() -> Self` — Provides tools for web search and URL fetching.
-  `WebSearchTool` type L955-1011 — `impl Tool for WebSearchTool` — Provides tools for web search and URL fetching.
-  `name` function L956-958 — `(&self) -> &str` — Provides tools for web search and URL fetching.
-  `description` function L960-962 — `(&self) -> &str` — Provides tools for web search and URL fetching.
-  `parameters` function L964-975 — `(&self) -> Value` — Provides tools for web search and URL fetching.
-  `execute` function L977-1010 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — Provides tools for web search and URL fetching.
-  `tests` module L1018-1243 — `-` — Provides tools for web search and URL fetching.
-  `test_web_fetch_tool_metadata` function L1022-1042 — `()` — Provides tools for web search and URL fetching.
-  `test_web_search_tool_metadata` function L1045-1052 — `()` — Provides tools for web search and URL fetching.
-  `test_extract_text_from_html` function L1055-1074 — `()` — Provides tools for web search and URL fetching.
-  `test_extract_title` function L1077-1081 — `()` — Provides tools for web search and URL fetching.
-  `test_extract_description` function L1084-1092 — `()` — Provides tools for web search and URL fetching.
-  `test_search_providers` function L1095-1101 — `()` — Provides tools for web search and URL fetching.
-  `test_web_fetch_invalid_url` function L1104-1115 — `()` — Provides tools for web search and URL fetching.
-  `test_web_fetch_non_http` function L1118-1129 — `()` — Provides tools for web search and URL fetching.
-  `test_web_fetch_unsupported_method` function L1132-1146 — `()` — Provides tools for web search and URL fetching.
-  `test_method_case_insensitivity` function L1149-1156 — `()` — Provides tools for web search and URL fetching.
-  `test_web_fetch_with_custom_headers_invalid_url` function L1159-1181 — `()` — Provides tools for web search and URL fetching.
-  `test_web_fetch_with_body_invalid_url` function L1184-1204 — `()` — Provides tools for web search and URL fetching.
-  `test_download_parameter_in_schema` function L1207-1213 — `()` — Provides tools for web search and URL fetching.
-  `test_max_size_config` function L1216-1220 — `()` — Provides tools for web search and URL fetching.
-  `test_web_fetch_download_invalid_url` function L1223-1242 — `()` — Provides tools for web search and URL fetching.

#### crates/arawn-agent/src/tools/workflow.rs

- pub `WorkflowTool` struct L43-48 — `{ engine: Arc<PipelineEngine>, workflow_dir: PathBuf, executor: Arc<ScriptExecut...` — Agent-facing tool for workflow management.
- pub `new` function L52-64 — `( engine: Arc<PipelineEngine>, workflow_dir: PathBuf, executor: Arc<ScriptExecut...` — Create a new workflow tool backed by the given engine, executor, and catalog.
-  `validate_name` function L21-38 — `(name: &str) -> std::result::Result<(), String>` — Validate a workflow name for safe use as a filename component.
-  `WorkflowTool` type L50-292 — `= WorkflowTool` — workflows via the pipeline engine.
-  `action_create` function L66-131 — `(&self, params: &Value) -> ToolResult` — workflows via the pipeline engine.
-  `action_run` function L133-177 — `(&self, params: &Value) -> ToolResult` — workflows via the pipeline engine.
-  `action_schedule` function L179-212 — `(&self, params: &Value) -> ToolResult` — workflows via the pipeline engine.
-  `action_list` function L214-240 — `(&self) -> ToolResult` — workflows via the pipeline engine.
-  `action_cancel` function L242-254 — `(&self, params: &Value) -> ToolResult` — workflows via the pipeline engine.
-  `action_status` function L256-291 — `(&self, params: &Value) -> ToolResult` — workflows via the pipeline engine.
-  `WorkflowTool` type L295-368 — `impl Tool for WorkflowTool` — workflows via the pipeline engine.
-  `name` function L296-298 — `(&self) -> &str` — workflows via the pipeline engine.
-  `description` function L300-304 — `(&self) -> &str` — workflows via the pipeline engine.
-  `parameters` function L306-342 — `(&self) -> Value` — workflows via the pipeline engine.
-  `execute` function L344-367 — `(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>` — workflows via the pipeline engine.
-  `tests` module L371-794 — `-` — workflows via the pipeline engine.
-  `setup` function L377-403 — `() -> (WorkflowTool, TempDir)` — workflows via the pipeline engine.
-  `test_parameters_schema` function L406-413 — `()` — workflows via the pipeline engine.
-  `test_create_writes_toml` function L416-447 — `()` — workflows via the pipeline engine.
-  `test_create_invalid_toml` function L450-468 — `()` — workflows via the pipeline engine.
-  `test_create_invalid_workflow` function L471-498 — `()` — workflows via the pipeline engine.
-  `test_create_missing_params` function L501-512 — `()` — workflows via the pipeline engine.
-  `test_list_empty` function L515-527 — `()` — workflows via the pipeline engine.
-  `test_run_unknown_workflow` function L530-552 — `()` — workflows via the pipeline engine.
-  `test_cancel_invalid_id` function L555-571 — `()` — workflows via the pipeline engine.
-  `test_status_unregistered` function L574-596 — `()` — workflows via the pipeline engine.
-  `test_unknown_action` function L599-610 — `()` — workflows via the pipeline engine.
-  `test_missing_action` function L613-621 — `()` — workflows via the pipeline engine.
-  `test_create_name_with_path_traversal` function L626-643 — `()` — workflows via the pipeline engine.
-  `test_create_empty_name` function L646-663 — `()` — workflows via the pipeline engine.
-  `test_create_name_with_control_chars` function L666-683 — `()` — workflows via the pipeline engine.
-  `test_run_missing_name` function L686-693 — `()` — workflows via the pipeline engine.
-  `test_run_accepts_name_param` function L696-712 — `()` — workflows via the pipeline engine.
-  `test_schedule_missing_name` function L715-725 — `()` — workflows via the pipeline engine.
-  `test_schedule_missing_cron` function L728-738 — `()` — workflows via the pipeline engine.
-  `test_cancel_missing_schedule_id` function L741-751 — `()` — workflows via the pipeline engine.
-  `test_status_missing_name` function L754-764 — `()` — workflows via the pipeline engine.
-  `test_action_is_number` function L767-774 — `()` — workflows via the pipeline engine.
-  `test_create_empty_definition` function L777-793 — `()` — workflows via the pipeline engine.

### crates/arawn-client/src/api

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-client/src/api/agents.rs

- pub `AgentsApi` struct L8-10 — `{ client: ArawnClient }` — Agents API client.
- pub `list` function L18-20 — `(&self) -> Result<ListAgentsResponse>` — List all available agents.
- pub `get` function L23-25 — `(&self, id: &str) -> Result<AgentDetail>` — Get an agent by ID.
- pub `main` function L28-30 — `(&self) -> Result<AgentDetail>` — Get the main/default agent.
-  `AgentsApi` type L12-31 — `= AgentsApi` — Agents API.
-  `new` function L13-15 — `(client: ArawnClient) -> Self` — Agents API.

#### crates/arawn-client/src/api/chat.rs

- pub `ChatApi` struct L12-14 — `{ client: ArawnClient }` — Chat API client.
- pub `send` function L22-24 — `(&self, request: ChatRequest) -> Result<ChatResponse>` — Send a chat message and get a response.
- pub `message` function L27-29 — `(&self, text: impl Into<String>) -> Result<ChatResponse>` — Send a message with just text (convenience method).
- pub `message_in_session` function L32-39 — `( &self, session_id: &str, text: impl Into<String>, ) -> Result<ChatResponse>` — Send a message in an existing session.
- pub `stream` function L44-73 — `( &self, request: ChatRequest, ) -> Result<impl Stream<Item = Result<StreamEvent...` — Stream a chat response.
- pub `stream_message` function L76-81 — `( &self, text: impl Into<String>, ) -> Result<impl Stream<Item = Result<StreamEv...` — Stream a message with just text (convenience method).
-  `ChatApi` type L16-82 — `= ChatApi` — Chat API.
-  `new` function L17-19 — `(client: ArawnClient) -> Self` — Chat API.

#### crates/arawn-client/src/api/config.rs

- pub `ConfigApi` struct L8-10 — `{ client: ArawnClient }` — Config API client.
- pub `get` function L18-20 — `(&self) -> Result<ConfigResponse>` — Get server configuration.
-  `ConfigApi` type L12-21 — `= ConfigApi` — Config API.
-  `new` function L13-15 — `(client: ArawnClient) -> Self` — Config API.

#### crates/arawn-client/src/api/health.rs

- pub `HealthApi` struct L10-12 — `{ client: ArawnClient }` — Health API client.
- pub `check` function L20-39 — `(&self) -> Result<HealthResponse>` — Check basic health.
- pub `is_healthy` function L42-44 — `(&self) -> bool` — Simple connectivity check - returns true if server is reachable.
-  `HealthApi` type L14-45 — `= HealthApi` — Health API.
-  `new` function L15-17 — `(client: ArawnClient) -> Self` — Health API.

#### crates/arawn-client/src/api/mcp.rs

- pub `McpApi` struct L8-10 — `{ client: ArawnClient }` — MCP API client.
- pub `list_servers` function L18-20 — `(&self) -> Result<ListServersResponse>` — List all MCP servers.
- pub `add_server` function L23-25 — `(&self, request: AddServerRequest) -> Result<AddServerResponse>` — Add an MCP server.
- pub `add_stdio_server` function L28-44 — `( &self, name: &str, command: &str, args: Vec<String>, auto_connect: bool, ) -> ...` — Add a stdio MCP server.
- pub `add_http_server` function L47-62 — `( &self, name: &str, url: &str, auto_connect: bool, ) -> Result<AddServerRespons...` — Add an HTTP MCP server.
- pub `remove_server` function L65-67 — `(&self, name: &str) -> Result<()>` — Remove an MCP server.
- pub `list_tools` function L70-74 — `(&self, server_name: &str) -> Result<ListToolsResponse>` — List tools for a server.
- pub `connect` function L77-85 — `(&self, server_name: &str) -> Result<()>` — Connect to a server.
- pub `disconnect` function L88-96 — `(&self, server_name: &str) -> Result<()>` — Disconnect from a server.
-  `McpApi` type L12-97 — `= McpApi` — MCP (Model Context Protocol) API.
-  `new` function L13-15 — `(client: ArawnClient) -> Self` — MCP (Model Context Protocol) API.

#### crates/arawn-client/src/api/memory.rs

- pub `MemorySearchQuery` struct L9-18 — `{ q: String, limit: Option<usize>, session_id: Option<String> }` — Query parameters for memory search.
- pub `MemoryApi` struct L21-23 — `{ client: ArawnClient }` — Memory API client.
- pub `search` function L31-37 — `(&self, query: &str) -> Result<MemorySearchResponse>` — Search memories.
- pub `search_with_options` function L40-45 — `( &self, query: MemorySearchQuery, ) -> Result<MemorySearchResponse>` — Search memories with options.
- pub `search_in_session` function L48-59 — `( &self, query: &str, session_id: &str, ) -> Result<MemorySearchResponse>` — Search memories in a specific session.
- pub `store` function L62-64 — `(&self, request: StoreMemoryRequest) -> Result<StoreMemoryResponse>` — Store a memory directly.
- pub `store_fact` function L67-76 — `(&self, content: impl Into<String>) -> Result<StoreMemoryResponse>` — Store a simple fact.
- pub `delete` function L79-81 — `(&self, id: &str) -> Result<()>` — Delete a memory by ID.
-  `MemoryApi` type L25-82 — `= MemoryApi` — Memory API.
-  `new` function L26-28 — `(client: ArawnClient) -> Self` — Memory API.

#### crates/arawn-client/src/api/mod.rs

-  `agents` module L3 — `-` — API endpoint implementations.
-  `chat` module L4 — `-` — API endpoint implementations.
-  `config` module L5 — `-` — API endpoint implementations.
-  `health` module L6 — `-` — API endpoint implementations.
-  `mcp` module L7 — `-` — API endpoint implementations.
-  `memory` module L8 — `-` — API endpoint implementations.
-  `notes` module L9 — `-` — API endpoint implementations.
-  `sessions` module L10 — `-` — API endpoint implementations.
-  `tasks` module L11 — `-` — API endpoint implementations.
-  `workstreams` module L12 — `-` — API endpoint implementations.

#### crates/arawn-client/src/api/notes.rs

- pub `ListNotesQuery` struct L9-16 — `{ tag: Option<String>, limit: Option<usize> }` — Query parameters for listing notes.
- pub `NotesApi` struct L19-21 — `{ client: ArawnClient }` — Notes API client.
- pub `list` function L29-31 — `(&self) -> Result<ListNotesResponse>` — List all notes.
- pub `list_with_query` function L34-36 — `(&self, query: ListNotesQuery) -> Result<ListNotesResponse>` — List notes with query parameters.
- pub `list_by_tag` function L39-45 — `(&self, tag: &str) -> Result<ListNotesResponse>` — List notes with a specific tag.
- pub `get` function L48-51 — `(&self, id: &str) -> Result<Note>` — Get a note by ID.
- pub `create` function L54-57 — `(&self, request: CreateNoteRequest) -> Result<Note>` — Create a new note.
- pub `create_simple` function L60-66 — `(&self, content: impl Into<String>) -> Result<Note>` — Create a note with just content.
- pub `update` function L69-72 — `(&self, id: &str, request: UpdateNoteRequest) -> Result<Note>` — Update a note.
- pub `delete` function L75-77 — `(&self, id: &str) -> Result<()>` — Delete a note.
-  `NotesApi` type L23-78 — `= NotesApi` — Notes API.
-  `new` function L24-26 — `(client: ArawnClient) -> Self` — Notes API.

#### crates/arawn-client/src/api/sessions.rs

- pub `SessionsApi` struct L11-13 — `{ client: ArawnClient }` — Sessions API client.
- pub `list` function L21-23 — `(&self) -> Result<ListSessionsResponse>` — List all sessions.
- pub `get` function L26-28 — `(&self, id: &str) -> Result<SessionDetail>` — Get a session by ID.
- pub `create` function L31-33 — `(&self, request: CreateSessionRequest) -> Result<SessionDetail>` — Create a new session.
- pub `update` function L36-40 — `(&self, id: &str, request: UpdateSessionRequest) -> Result<SessionDetail>` — Update a session.
- pub `delete` function L43-45 — `(&self, id: &str) -> Result<()>` — Delete a session.
- pub `messages` function L48-50 — `(&self, id: &str) -> Result<SessionMessagesResponse>` — Get messages for a session.
-  `SessionsApi` type L15-51 — `= SessionsApi` — Sessions API.
-  `new` function L16-18 — `(client: ArawnClient) -> Self` — Sessions API.

#### crates/arawn-client/src/api/tasks.rs

- pub `ListTasksQuery` struct L9-19 — `{ status: Option<String>, session_id: Option<String>, limit: Option<usize> }` — Query parameters for listing tasks.
- pub `TasksApi` struct L22-24 — `{ client: ArawnClient }` — Tasks API client.
- pub `list` function L32-34 — `(&self) -> Result<ListTasksResponse>` — List all tasks.
- pub `list_with_query` function L37-39 — `(&self, query: ListTasksQuery) -> Result<ListTasksResponse>` — List tasks with query parameters.
- pub `list_running` function L42-48 — `(&self) -> Result<ListTasksResponse>` — List running tasks.
- pub `list_for_session` function L51-57 — `(&self, session_id: &str) -> Result<ListTasksResponse>` — List tasks for a session.
- pub `get` function L60-62 — `(&self, id: &str) -> Result<TaskDetail>` — Get a task by ID.
- pub `cancel` function L65-67 — `(&self, id: &str) -> Result<()>` — Cancel a task.
-  `TasksApi` type L26-68 — `= TasksApi` — Tasks API.
-  `new` function L27-29 — `(client: ArawnClient) -> Self` — Tasks API.

#### crates/arawn-client/src/api/workstreams.rs

- pub `ListMessagesQuery` struct L13-17 — `{ since: Option<String> }` — Query parameters for listing messages.
- pub `ListWorkstreamsQuery` struct L21-25 — `{ include_archived: bool }` — Query parameters for listing workstreams.
- pub `WorkstreamsApi` struct L28-30 — `{ client: ArawnClient }` — Workstreams API client.
- pub `list` function L38-40 — `(&self) -> Result<ListWorkstreamsResponse>` — List all active workstreams.
- pub `list_all` function L43-48 — `(&self) -> Result<ListWorkstreamsResponse>` — List all workstreams including archived.
- pub `get` function L51-53 — `(&self, id: &str) -> Result<Workstream>` — Get a workstream by ID.
- pub `create` function L56-58 — `(&self, request: CreateWorkstreamRequest) -> Result<Workstream>` — Create a new workstream.
- pub `update` function L61-65 — `(&self, id: &str, request: UpdateWorkstreamRequest) -> Result<Workstream>` — Update a workstream.
- pub `delete` function L68-70 — `(&self, id: &str) -> Result<()>` — Delete (archive) a workstream.
- pub `send_message` function L73-81 — `( &self, workstream_id: &str, request: SendMessageRequest, ) -> Result<Workstrea...` — Send a message to a workstream.
- pub `messages` function L84-88 — `(&self, workstream_id: &str) -> Result<ListMessagesResponse>` — List messages in a workstream.
- pub `messages_since` function L91-102 — `( &self, workstream_id: &str, since: &str, ) -> Result<ListMessagesResponse>` — List messages since a timestamp.
- pub `sessions` function L105-109 — `(&self, workstream_id: &str) -> Result<ListWorkstreamSessionsResponse>` — List sessions in a workstream.
- pub `promote_scratch` function L112-116 — `(&self, request: PromoteRequest) -> Result<Workstream>` — Promote the scratch workstream to a named workstream.
-  `WorkstreamsApi` type L32-117 — `= WorkstreamsApi` — Workstreams API.
-  `new` function L33-35 — `(client: ArawnClient) -> Self` — Workstreams API.

### crates/arawn-client/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-client/src/client.rs

- pub `ArawnClient` struct L41-44 — `{ inner: Arc<ClientInner> }` — Arawn API client.
- pub `builder` function L67-69 — `() -> ClientBuilder` — Create a new client builder.
- pub `localhost` function L72-74 — `() -> Result<Self>` — Create a client with default settings pointing to localhost.
- pub `base_url` function L77-79 — `(&self) -> &Url` — Get the base URL.
- pub `sessions` function L86-88 — `(&self) -> SessionsApi` — Access the sessions API.
- pub `workstreams` function L91-93 — `(&self) -> WorkstreamsApi` — Access the workstreams API.
- pub `chat` function L96-98 — `(&self) -> ChatApi` — Access the chat API.
- pub `config` function L101-103 — `(&self) -> ConfigApi` — Access the config API.
- pub `agents` function L106-108 — `(&self) -> AgentsApi` — Access the agents API.
- pub `notes` function L111-113 — `(&self) -> NotesApi` — Access the notes API.
- pub `memory` function L116-118 — `(&self) -> MemoryApi` — Access the memory API.
- pub `tasks` function L121-123 — `(&self) -> TasksApi` — Access the tasks API.
- pub `mcp` function L126-128 — `(&self) -> McpApi` — Access the MCP API.
- pub `health` function L131-133 — `(&self) -> HealthApi` — Access the health API.
- pub `ClientBuilder` struct L315-321 — `{ base_url: Option<String>, auth_token: Option<String>, timeout: Duration, strea...` — Builder for creating an ArawnClient.
- pub `new` function L325-333 — `() -> Self` — Create a new builder with defaults.
- pub `base_url` function L336-339 — `(mut self, url: impl Into<String>) -> Self` — Set the base URL for the server.
- pub `auth_token` function L342-345 — `(mut self, token: impl Into<String>) -> Self` — Set the authentication token.
- pub `timeout` function L348-351 — `(mut self, timeout: Duration) -> Self` — Set the request timeout.
- pub `stream_timeout` function L354-357 — `(mut self, timeout: Duration) -> Self` — Set the streaming request timeout.
- pub `user_agent` function L360-363 — `(mut self, agent: impl Into<String>) -> Self` — Set a custom user agent.
- pub `build` function L366-405 — `(self) -> Result<ArawnClient>` — Build the client.
-  `DEFAULT_TIMEOUT` variable L16 — `: Duration` — Default timeout for requests.
-  `DEFAULT_STREAM_TIMEOUT` variable L19 — `: Duration` — Default timeout for streaming requests.
-  `ClientInner` struct L47-56 — `{ http: reqwest::Client, base_url: Url, timeout: Duration, stream_timeout: Durat...` — Inner client state (shared across clones).
-  `ArawnClient` type L58-63 — `= ArawnClient` — Main client implementation.
-  `inner` function L60-62 — `(&self) -> &ClientInner` — Get access to the inner client state (for API implementations).
-  `ArawnClient` type L65-311 — `= ArawnClient` — Main client implementation.
-  `url` function L140-146 — `(&self, path: &str) -> Result<Url>` — Build a URL for an API path.
-  `get` function L149-159 — `(&self, path: &str) -> Result<T>` — Make a GET request.
-  `get_with_query` function L162-177 — `(&self, path: &str, query: &Q) -> Result<T>` — Make a GET request with query parameters.
-  `post` function L180-195 — `(&self, path: &str, body: &B) -> Result<T>` — Make a POST request.
-  `post_stream` function L198-217 — `(&self, path: &str, body: &B) -> Result<reqwest::Response>` — Make a POST request for streaming (returns the response directly).
-  `patch` function L220-235 — `(&self, path: &str, body: &B) -> Result<T>` — Make a PATCH request.
-  `put` function L238-253 — `(&self, path: &str, body: &B) -> Result<T>` — Make a PUT request.
-  `delete` function L256-271 — `(&self, path: &str) -> Result<()>` — Make a DELETE request.
-  `handle_response` function L274-283 — `( &self, response: reqwest::Response, ) -> Result<T>` — Handle a response, extracting the body or error.
-  `extract_error` function L286-310 — `(&self, response: reqwest::Response) -> Error` — Extract an error from a failed response.
-  `ClientBuilder` type L323-406 — `= ClientBuilder` — Main client implementation.
-  `ClientBuilder` type L408-412 — `impl Default for ClientBuilder` — Main client implementation.
-  `default` function L409-411 — `() -> Self` — Main client implementation.
-  `tests` module L415-457 — `-` — Main client implementation.
-  `test_builder_requires_base_url` function L419-422 — `()` — Main client implementation.
-  `test_builder_with_base_url` function L425-432 — `()` — Main client implementation.
-  `test_builder_normalizes_trailing_slash` function L435-442 — `()` — Main client implementation.
-  `test_url_building` function L445-456 — `()` — Main client implementation.

#### crates/arawn-client/src/error.rs

- pub `Error` enum L7-46 — `Http | InvalidUrl | Json | Api | Auth | NotFound | Config | Stream` — Client error type.
- pub `is_not_found` function L50-52 — `(&self) -> bool` — Check if this is a not-found error.
- pub `is_auth_error` function L55-57 — `(&self) -> bool` — Check if this is an authentication error.
- pub `is_rate_limited` function L60-62 — `(&self) -> bool` — Check if this is a rate limit error.
- pub `is_server_error` function L65-67 — `(&self) -> bool` — Check if this is a server error.
- pub `Result` type L71 — `= std::result::Result<T, Error>` — Result type for client operations.
-  `Error` type L48-68 — `= Error` — Client error types.
-  `ErrorResponse` struct L75-78 — `{ code: String, message: String }` — Error response from the server.

#### crates/arawn-client/src/lib.rs

- pub `api` module L61 — `-` — This crate provides a typed client for interacting with the Arawn server API.
- pub `client` module L62 — `-` — - **Health**: Server health checks
- pub `error` module L63 — `-` — - **Health**: Server health checks
- pub `types` module L64 — `-` — - **Health**: Server health checks

#### crates/arawn-client/src/types.rs

- pub `CreateSessionRequest` struct L14-21 — `{ title: Option<String>, metadata: HashMap<String, serde_json::Value> }` — Request to create a new session.
- pub `UpdateSessionRequest` struct L25-35 — `{ title: Option<String>, metadata: Option<HashMap<String, serde_json::Value>>, w...` — Request to update a session.
- pub `SessionSummary` struct L39-51 — `{ id: String, title: Option<String>, turn_count: usize, created_at: String, upda...` — Summary info for a session.
- pub `SessionDetail` struct L55-67 — `{ id: String, turns: Vec<TurnInfo>, created_at: String, updated_at: String, meta...` — Full session details.
- pub `TurnInfo` struct L71-84 — `{ id: String, user_message: String, assistant_response: Option<String>, tool_cal...` — Turn info within a session.
- pub `MessageInfo` struct L88-95 — `{ role: String, content: String, timestamp: String }` — Message info for conversation history.
- pub `SessionMessagesResponse` struct L99-106 — `{ session_id: String, messages: Vec<MessageInfo>, count: usize }` — Response containing session messages.
- pub `ListSessionsResponse` struct L110-115 — `{ sessions: Vec<SessionSummary>, total: usize }` — Response for list sessions.
- pub `CreateWorkstreamRequest` struct L123-132 — `{ title: String, default_model: Option<String>, tags: Vec<String> }` — Request to create a workstream.
- pub `UpdateWorkstreamRequest` struct L136-149 — `{ title: Option<String>, summary: Option<String>, default_model: Option<String>,...` — Request to update a workstream.
- pub `Workstream` struct L153-175 — `{ id: String, title: String, summary: Option<String>, state: String, default_mod...` — Workstream details.
- pub `ListWorkstreamsResponse` struct L179-182 — `{ workstreams: Vec<Workstream> }` — Response for list workstreams.
- pub `SendMessageRequest` struct L186-195 — `{ role: Option<String>, content: String, metadata: Option<String> }` — Request to send a message.
- pub `WorkstreamMessage` struct L199-216 — `{ id: String, workstream_id: String, session_id: Option<String>, role: String, c...` — Workstream message.
- pub `ListMessagesResponse` struct L220-223 — `{ messages: Vec<WorkstreamMessage> }` — Response for list messages.
- pub `WorkstreamSession` struct L227-239 — `{ id: String, workstream_id: String, started_at: String, ended_at: Option<String...` — Workstream session info.
- pub `ListWorkstreamSessionsResponse` struct L243-246 — `{ sessions: Vec<WorkstreamSession> }` — Response for list workstream sessions.
- pub `PromoteRequest` struct L250-259 — `{ title: String, tags: Vec<String>, default_model: Option<String> }` — Request to promote scratch workstream.
- pub `ChatRequest` struct L267-285 — `{ message: String, session_id: Option<String>, model: Option<String>, system_pro...` — Chat request.
- pub `new` function L289-298 — `(message: impl Into<String>) -> Self` — Create a new chat request with just a message.
- pub `with_session` function L301-304 — `(mut self, session_id: impl Into<String>) -> Self` — Set the session ID.
- pub `with_model` function L307-310 — `(mut self, model: impl Into<String>) -> Self` — Set the model.
- pub `ChatResponse` struct L315-331 — `{ response: String, session_id: String, turn_id: String, tool_calls: Vec<ToolCal...` — Chat response.
- pub `ToolCallInfo` struct L335-342 — `{ name: String, id: String, success: bool }` — Tool call information.
- pub `TokenUsage` struct L346-353 — `{ prompt_tokens: u32, completion_tokens: u32, total_tokens: u32 }` — Token usage information.
- pub `StreamEvent` enum L358-383 — `SessionStart | Content | ToolStart | ToolOutput | ToolEnd | Done | Error` — Streaming chat event.
- pub `ConfigResponse` struct L391-405 — `{ version: String, api_version: Option<String>, features: ConfigFeatures, limits...` — Server configuration response.
- pub `ConfigFeatures` struct L409-420 — `{ workstreams_enabled: bool, memory_enabled: bool, mcp_enabled: bool, rate_limit...` — Server feature flags.
- pub `ConfigLimits` struct L424-428 — `{ max_concurrent_requests: Option<u32> }` — Server limits.
- pub `AgentSummary` struct L436-445 — `{ id: String, name: String, is_default: bool, tool_count: usize }` — Agent summary.
- pub `AgentDetail` struct L449-460 — `{ id: String, name: String, is_default: bool, tools: Vec<AgentToolInfo>, capabil...` — Agent details.
- pub `AgentToolInfo` struct L464-469 — `{ name: String, description: String }` — Tool info for an agent.
- pub `AgentCapabilities` struct L473-481 — `{ streaming: bool, tool_use: bool, max_context_length: Option<usize> }` — Agent capabilities.
- pub `ListAgentsResponse` struct L485-490 — `{ agents: Vec<AgentSummary>, total: usize }` — Response for list agents.
- pub `Note` struct L498-508 — `{ id: String, content: String, tags: Vec<String>, created_at: String }` — A note.
- pub `CreateNoteRequest` struct L512-518 — `{ content: String, tags: Vec<String> }` — Request to create a note.
- pub `UpdateNoteRequest` struct L522-529 — `{ content: Option<String>, tags: Option<Vec<String>> }` — Request to update a note.
- pub `ListNotesResponse` struct L533-538 — `{ notes: Vec<Note>, total: usize }` — Response for list notes.
- pub `NoteResponse` struct L542-545 — `{ note: Note }` — Response for single note operations.
- pub `StoreMemoryRequest` struct L553-568 — `{ content: String, content_type: String, session_id: Option<String>, metadata: H...` — Request to store a memory.
- pub `StoreMemoryResponse` struct L580-587 — `{ id: String, content_type: String, message: String }` — Response after storing a memory.
- pub `MemorySearchResult` struct L591-608 — `{ id: String, content_type: String, content: String, session_id: Option<String>,...` — Memory search result.
- pub `MemorySearchResponse` struct L612-619 — `{ results: Vec<MemorySearchResult>, query: String, count: usize }` — Response for memory search.
- pub `TaskStatus` enum L628-639 — `Pending | Running | Completed | Failed | Cancelled` — Task status.
- pub `TaskSummary` struct L643-655 — `{ id: String, task_type: String, status: TaskStatus, progress: Option<u8>, creat...` — Task summary.
- pub `TaskDetail` struct L659-686 — `{ id: String, task_type: String, status: TaskStatus, progress: Option<u8>, messa...` — Task details.
- pub `ListTasksResponse` struct L690-695 — `{ tasks: Vec<TaskSummary>, total: usize }` — Response for list tasks.
- pub `AddServerRequest` struct L703-721 — `{ name: String, command: Option<String>, args: Vec<String>, env: HashMap<String,...` — Request to add an MCP server.
- pub `AddServerResponse` struct L725-733 — `{ name: String, connected: bool, tools: Vec<String> }` — Response after adding a server.
- pub `ServerInfo` struct L737-747 — `{ name: String, server_type: String, connected: bool, tool_count: Option<usize> ...` — MCP server info.
- pub `ListServersResponse` struct L751-754 — `{ servers: Vec<ServerInfo> }` — Response for list servers.
- pub `McpToolInfo` struct L758-764 — `{ name: String, description: Option<String> }` — Tool info from MCP server.
- pub `ListToolsResponse` struct L768-773 — `{ server: String, tools: Vec<McpToolInfo> }` — Response for list server tools.
- pub `HealthResponse` struct L781-787 — `{ status: String, version: Option<String> }` — Health check response.
-  `ChatRequest` type L287-311 — `= ChatRequest` — These types mirror the server's API contract.
-  `default_content_type` function L570-572 — `() -> String` — These types mirror the server's API contract.
-  `default_confidence` function L574-576 — `() -> f32` — These types mirror the server's API contract.

### crates/arawn-config/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-config/src/age_crypto.rs

- pub `default_identity_path` function L13-15 — `() -> Option<PathBuf>` — Get the default path for the age identity file.
- pub `generate_identity` function L21-46 — `(path: &Path) -> Result<String, AgeError>` — Generate a new age identity and save it to a file.
- pub `load_or_generate_identity` function L52-59 — `(path: &Path) -> Result<age::x25519::Identity, AgeError>` — Load an age identity from a file, generating one if it doesn't exist.
- pub `load_identity` function L62-68 — `(path: &Path) -> Result<age::x25519::Identity, AgeError>` — Load an existing age identity from a file.
- pub `encrypt` function L73-75 — `(data: &[u8], recipient: &age::x25519::Recipient) -> Result<Vec<u8>, AgeError>` — Encrypt data to a recipient (public key).
- pub `decrypt` function L80-82 — `(encrypted: &[u8], identity: &age::x25519::Identity) -> Result<Vec<u8>, AgeError...` — Decrypt data with an identity (private key).
- pub `AgeError` enum L86-98 — `Io | Identity | Encrypt | Decrypt` — Errors from age crypto operations.
-  `tests` module L101-178 — `-` — `~/.config/arawn/identity.age`.
-  `test_encrypt_decrypt_roundtrip` function L105-115 — `()` — `~/.config/arawn/identity.age`.
-  `test_encrypt_decrypt_empty` function L118-125 — `()` — `~/.config/arawn/identity.age`.
-  `test_encrypt_decrypt_large` function L128-136 — `()` — `~/.config/arawn/identity.age`.
-  `test_wrong_identity_fails` function L139-147 — `()` — `~/.config/arawn/identity.age`.
-  `test_generate_and_load_identity` function L150-160 — `()` — `~/.config/arawn/identity.age`.
-  `test_load_or_generate_creates_when_missing` function L163-177 — `()` — `~/.config/arawn/identity.age`.

#### crates/arawn-config/src/client.rs

- pub `API_VERSION` variable L32 — `: &str` — API version for the client config file format.
- pub `KIND` variable L35 — `: &str` — Kind identifier for client config files.
- pub `ClientConfig` struct L45-65 — `{ api_version: String, kind: String, current_context: Option<String>, contexts: ...` — Root client configuration structure.
- pub `new` function L77-83 — `() -> Self` — Create an empty client config.
- pub `from_yaml` function L86-88 — `(yaml_str: &str) -> Result<Self>` — Parse from a YAML string.
- pub `to_yaml` function L91-93 — `(&self) -> Result<String>` — Serialize to a YAML string.
- pub `current` function L96-100 — `(&self) -> Option<&Context>` — Get the current context, if set and valid.
- pub `get_context` function L103-105 — `(&self, name: &str) -> Option<&Context>` — Get a context by name.
- pub `get_context_mut` function L108-110 — `(&mut self, name: &str) -> Option<&mut Context>` — Get a mutable context by name.
- pub `set_context` function L113-119 — `(&mut self, context: Context)` — Add or update a context.
- pub `remove_context` function L122-132 — `(&mut self, name: &str) -> Option<Context>` — Remove a context by name.
- pub `use_context` function L137-144 — `(&mut self, name: &str) -> Result<()>` — Set the current context by name.
- pub `context_names` function L147-149 — `(&self) -> Vec<&str>` — List all context names.
- pub `server_url` function L152-154 — `(&self, context_name: &str) -> Option<String>` — Get the effective server URL for a context, applying defaults.
- pub `current_server_url` function L157-159 — `(&self) -> Option<String>` — Get the effective server URL for the current context.
- pub `Context` struct L169-187 — `{ name: String, server: String, auth: Option<AuthConfig>, workstream: Option<Str...` — A named connection context (server + auth bundle).
- pub `new` function L191-199 — `(name: impl Into<String>, server: impl Into<String>) -> Self` — Create a new context with just a name and server URL.
- pub `with_auth` function L202-205 — `(mut self, auth: AuthConfig) -> Self` — Set the auth configuration.
- pub `with_workstream` function L208-211 — `(mut self, workstream: impl Into<String>) -> Self` — Set the default workstream.
- pub `with_timeout` function L214-217 — `(mut self, timeout: u64) -> Self` — Set the connection timeout.
- pub `AuthConfig` enum L227-257 — `None | ApiKey | Oauth | Bearer` — Authentication configuration for a context.
- pub `api_key_file` function L261-266 — `(path: impl Into<PathBuf>) -> Self` — Create API key auth referencing a file.
- pub `api_key_env` function L269-274 — `(var: impl Into<String>) -> Self` — Create API key auth referencing an environment variable.
- pub `oauth` function L277-282 — `(client_id: impl Into<String>) -> Self` — Create OAuth auth.
- pub `resolve` function L287-344 — `(&self) -> Result<Option<String>>` — Resolve the actual credential value.
- pub `ClientDefaults` struct L354-360 — `{ timeout: u64, workstream: String }` — Default settings applied to all contexts.
- pub `client_config_path` function L376-378 — `() -> Option<PathBuf>` — Get the path to the client config file.
- pub `load_client_config` function L392-394 — `() -> Result<ClientConfig>` — Load the client configuration.
- pub `load_client_config_from` function L397-412 — `(path: Option<&Path>) -> Result<ClientConfig>` — Load client config from a specific path.
- pub `save_client_config` function L415-419 — `(config: &ClientConfig) -> Result<()>` — Save the client configuration.
- pub `save_client_config_to` function L422-438 — `(config: &ClientConfig, path: &Path) -> Result<()>` — Save client config to a specific path.
-  `CLIENT_CONFIG_FILE` variable L38 — `: &str` — Default config filename.
-  `default_api_version` function L67-69 — `() -> String` — ```
-  `default_kind` function L71-73 — `() -> String` — ```
-  `ClientConfig` type L75-160 — `= ClientConfig` — ```
-  `Context` type L189-218 — `= Context` — ```
-  `AuthConfig` type L259-345 — `= AuthConfig` — ```
-  `ClientDefaults` type L362-369 — `impl Default for ClientDefaults` — ```
-  `default` function L363-368 — `() -> Self` — ```
-  `expand_path` function L445-453 — `(path: &Path) -> PathBuf` — Expand ~ to home directory in paths.
-  `tests` module L460-702 — `-` — ```
-  `test_empty_config` function L464-470 — `()` — ```
-  `test_parse_minimal_yaml` function L473-487 — `()` — ```
-  `test_parse_full_yaml` function L490-555 — `()` — ```
-  `test_current_context` function L558-572 — `()` — ```
-  `test_set_context` function L575-586 — `()` — ```
-  `test_remove_context` function L589-604 — `()` — ```
-  `test_use_context` function L607-618 — `()` — ```
-  `test_context_names` function L621-630 — `()` — ```
-  `test_roundtrip_yaml` function L633-651 — `()` — ```
-  `test_context_builder` function L654-665 — `()` — ```
-  `test_auth_api_key_env_resolve` function L668-680 — `()` — ```
-  `test_auth_none_resolve` function L683-687 — `()` — ```
-  `test_expand_path` function L690-701 — `()` — ```

#### crates/arawn-config/src/discovery.rs

- pub `ConfigSource` struct L23-28 — `{ path: PathBuf, loaded: bool }` — Tracks where each config layer was loaded from.
- pub `LoadedConfig` struct L32-41 — `{ config: ArawnConfig, sources: Vec<ConfigSource>, source: Option<ConfigSource>,...` — Result of config discovery and loading.
- pub `loaded_from` function L45-51 — `(&self) -> Vec<&Path>` — Get paths of sources that were actually loaded.
- pub `load_config` function L70-72 — `(project_dir: Option<&Path>) -> Result<LoadedConfig>` — Load configuration by discovering and merging all config layers.
- pub `load_config_with_options` function L78-115 — `( project_dir: Option<&Path>, config_dir: Option<&Path>, ) -> Result<LoadedConfi...` — Load configuration with explicit control over the user config directory.
- pub `load_config_file` function L118-124 — `(path: &Path) -> Result<ArawnConfig>` — Load config from a specific file path (no discovery).
- pub `save_config` function L129-145 — `(config: &ArawnConfig, path: &Path) -> Result<()>` — Save configuration to a file.
- pub `xdg_config_path` function L165-167 — `() -> Option<PathBuf>` — Get the XDG config directory path for arawn.
- pub `xdg_config_dir` function L173-180 — `() -> Option<PathBuf>` — Get the config directory for arawn.
-  `PROJECT_CONFIG_FILE` variable L13 — `: &str` — Default config filename for project-local config.
-  `USER_CONFIG_FILE` variable L16 — `: &str` — Default config filename within XDG config directory.
-  `APP_NAME` variable L19 — `: &str` — Application name for XDG directory resolution.
-  `LoadedConfig` type L43-52 — `= LoadedConfig` — 3.
-  `CONFIG_DIR_ENV` variable L151 — `: &str` — Environment variable to override the config directory.
-  `load_layer` function L183-211 — `( config: &mut ArawnConfig, path: &Path, warnings: &mut Vec<String>, ) -> Result...` — Try to load a config file and merge it into the existing config.
-  `check_plaintext_keys` function L214-235 — `(config: &ArawnConfig, warnings: &mut Vec<String>)` — Check for plaintext API keys in the config and emit warnings.
-  `tests` module L242-463 — `-` — 3.
-  `test_xdg_config_path_exists` function L250-257 — `()` — 3.
-  `test_load_config_file` function L260-275 — `()` — 3.
-  `test_load_config_file_not_found` function L278-281 — `()` — 3.
-  `test_load_config_invalid_toml` function L284-291 — `()` — 3.
-  `test_load_config_project_only` function L294-321 — `()` — 3.
-  `test_load_config_no_files` function L324-332 — `()` — 3.
-  `test_load_config_layered_merge` function L335-387 — `()` — 3.
-  `test_plaintext_key_warning` function L390-413 — `()` — 3.
-  `test_no_warnings_without_keys` function L416-431 — `()` — 3.
-  `test_malformed_config_warns_but_continues` function L434-443 — `()` — 3.
-  `test_loaded_from_tracks_sources` function L446-462 — `()` — 3.

#### crates/arawn-config/src/error.rs

- pub `Result` type L4 — `= std::result::Result<T, ConfigError>` — Result type alias for config operations.
- pub `ConfigError` enum L8-66 — `ReadFile | WriteFile | Parse | Serialize | LlmNotFound | NoDefaultLlm | MissingF...` — Errors that can occur during configuration loading and resolution.

#### crates/arawn-config/src/lib.rs

- pub `age_crypto` module L16 — `-` — Provides TOML-based configuration with:
- pub `client` module L17 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `discovery` module L18 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `error` module L19 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `paths` module L20 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `resolver` module L21 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `secret_store` module L22 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `secrets` module L23 — `-` — See ADR ARAWN-A-0001 for architectural decisions.
- pub `types` module L24 — `-` — See ADR ARAWN-A-0001 for architectural decisions.

#### crates/arawn-config/src/paths.rs

- pub `PathConfig` struct L43-58 — `{ base_path: Option<PathBuf>, usage: UsageThresholds, cleanup: CleanupConfig, mo...` — Path management configuration.
- pub `effective_base_path` function L67-79 — `(&self) -> PathBuf` — Get the effective base path, checking environment variable first.
- pub `total_warning_bytes` function L82-84 — `(&self) -> u64` — Get total usage warning threshold in bytes.
- pub `workstream_warning_bytes` function L87-89 — `(&self) -> u64` — Get per-workstream usage warning threshold in bytes.
- pub `session_warning_bytes` function L92-94 — `(&self) -> u64` — Get per-session usage warning threshold in bytes.
- pub `monitoring_enabled` function L97-103 — `(&self) -> bool` — Check if filesystem monitoring is enabled (respects env var).
- pub `UsageThresholds` struct L109-121 — `{ total_warning_gb: u64, workstream_warning_gb: u64, session_warning_mb: u64 }` — Disk usage warning thresholds.
- pub `CleanupConfig` struct L136-144 — `{ scratch_cleanup_days: u32, dry_run: bool }` — Cleanup configuration for scratch sessions and disk pressure.
- pub `MonitoringConfig` struct L158-174 — `{ enabled: bool, debounce_ms: u64, polling_interval_secs: u64 }` — Filesystem monitoring configuration.
-  `PathConfig` type L60-104 — `= PathConfig` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `UsageThresholds` type L123-131 — `impl Default for UsageThresholds` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `default` function L124-130 — `() -> Self` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `CleanupConfig` type L146-153 — `impl Default for CleanupConfig` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `default` function L147-152 — `() -> Self` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `MonitoringConfig` type L176-184 — `impl Default for MonitoringConfig` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `default` function L177-183 — `() -> Self` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `tests` module L187-358 — `-` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_path_config_defaults` function L191-202 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_usage_thresholds_defaults` function L205-210 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_cleanup_config_defaults` function L213-217 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_monitoring_config_defaults` function L220-225 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_total_warning_bytes` function L228-232 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_workstream_warning_bytes` function L235-239 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_session_warning_bytes` function L242-246 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_effective_base_path_default` function L249-259 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_effective_base_path_configured` function L262-271 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_effective_base_path_env_override` function L274-287 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_monitoring_enabled_default` function L290-296 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_monitoring_enabled_configured_false` function L299-307 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_monitoring_enabled_env_true` function L310-321 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_monitoring_enabled_env_false` function L324-334 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_monitoring_enabled_env_numeric` function L337-345 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")
-  `test_custom_usage_thresholds` function L348-357 — `()` — - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")

#### crates/arawn-config/src/resolver.rs

- pub `ResolvedLlm` struct L27-44 — `{ backend: Backend, model: String, base_url: Option<String>, api_key: Option<Str...` — A fully resolved LLM configuration ready to construct a backend.
- pub `ResolvedFrom` enum L63-70 — `AgentSpecific | AgentDefault | GlobalDefault` — Tracks how the LLM config was resolved for diagnostics.
- pub `ApiKeySource` enum L88-97 — `Keyring | EnvVar | ConfigFile | NotFound` — How an API key was resolved.
- pub `resolve_for_agent` function L113-149 — `(config: &ArawnConfig, agent_name: &str) -> Result<ResolvedLlm>` — Resolve the LLM config for a given agent name.
- pub `resolve_all_profiles` function L161-178 — `(config: &ArawnConfig) -> Vec<(String, Backend, String)>` — Resolve all named LLM configs into a summary for diagnostics.
-  `ResolvedLlm` type L46-59 — `= ResolvedLlm` — a given agent, handling cascading defaults and API key lookup.
-  `fmt` function L47-58 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — a given agent, handling cascading defaults and API key lookup.
-  `ResolvedFrom` type L72-84 — `= ResolvedFrom` — a given agent, handling cascading defaults and API key lookup.
-  `fmt` function L73-83 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — a given agent, handling cascading defaults and API key lookup.
-  `resolve_llm_config` function L181-229 — `( config: &'a ArawnConfig, agent_name: &str, ) -> Result<(&'a LlmConfig, Resolve...` — Inner resolution that returns both the config ref and how it was resolved.
-  `tests` module L236-459 — `-` — a given agent, handling cascading defaults and API key lookup.
-  `test_config` function L239-267 — `() -> ArawnConfig` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_agent_specific` function L270-281 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_agent_default_fallback` function L284-294 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_global_default` function L297-309 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_no_config` function L312-316 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_missing_backend` function L319-327 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_missing_model` function L330-338 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_missing_profile_reference` function L341-353 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_with_base_url` function L356-362 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_ollama_no_api_key_needed` function L365-378 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_api_key_from_config` function L381-391 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolve_all_profiles` function L394-402 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolved_from_display` function L405-419 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolved_llm_debug_redacts_api_key` function L422-441 — `()` — a given agent, handling cascading defaults and API key lookup.
-  `test_resolved_llm_debug_no_key` function L444-458 — `()` — a given agent, handling cascading defaults and API key lookup.

#### crates/arawn-config/src/secret_store.rs

- pub `default_secrets_path` function L18-20 — `() -> Option<PathBuf>` — Path for the encrypted secrets file.
- pub `AgeSecretStore` struct L27-32 — `{ identity: age::x25519::Identity, secrets_path: PathBuf, cache: RwLock<BTreeMap...` — An age-encrypted secret store.
- pub `open` function L39-65 — `(identity_path: &Path, secrets_path: &Path) -> Result<Self, SecretStoreError>` — Open or create a secret store.
- pub `open_default` function L68-75 — `() -> Result<Self, SecretStoreError>` — Open using default paths (`~/.config/arawn/identity.age` and `secrets.age`).
- pub `set` function L78-87 — `(&self, name: &str, value: &str) -> Result<(), SecretStoreError>` — Store a secret.
- pub `delete` function L92-104 — `(&self, name: &str) -> Result<bool, SecretStoreError>` — Delete a secret.
- pub `get` function L107-110 — `(&self, name: &str) -> Option<String>` — Get a secret value by name.
- pub `list` function L113-118 — `(&self) -> Vec<String>` — List all secret names (never values).
- pub `contains` function L121-126 — `(&self, name: &str) -> bool` — Check if a secret exists.
- pub `SecretStoreError` enum L184-193 — `Io | Age | Format` — Errors from the secret store.
-  `AgeSecretStore` type L34-160 — `= AgeSecretStore` — be injected into the agent's `ToolContext` for handle resolution.
-  `flush` function L129-159 — `(&self) -> Result<(), SecretStoreError>` — Flush the in-memory cache to the encrypted file.
-  `AgeSecretStore` type L162-170 — `impl SecretResolver for AgeSecretStore` — be injected into the agent's `ToolContext` for handle resolution.
-  `resolve` function L163-165 — `(&self, name: &str) -> Option<String>` — be injected into the agent's `ToolContext` for handle resolution.
-  `names` function L167-169 — `(&self) -> Vec<String>` — be injected into the agent's `ToolContext` for handle resolution.
-  `AgeSecretStore` type L172-180 — `= AgeSecretStore` — be injected into the agent's `ToolContext` for handle resolution.
-  `fmt` function L173-179 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — be injected into the agent's `ToolContext` for handle resolution.
-  `tests` module L196-313 — `-` — be injected into the agent's `ToolContext` for handle resolution.
-  `setup` function L199-205 — `() -> (tempfile::TempDir, AgeSecretStore)` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_empty_store` function L208-213 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_set_and_get` function L216-223 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_set_overwrite` function L226-233 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_delete` function L236-247 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_multiple_secrets` function L250-260 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_persistence_across_reopen` function L263-282 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_secret_resolver_trait` function L285-294 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_special_characters_in_values` function L297-302 — `()` — be injected into the agent's `ToolContext` for handle resolution.
-  `test_debug_hides_values` function L305-312 — `()` — be injected into the agent's `ToolContext` for handle resolution.

#### crates/arawn-config/src/secrets.rs

- pub `ResolvedSecret` struct L19-24 — `{ value: String, source: SecretSource }` — Result of API key resolution with provenance.
- pub `SecretSource` enum L37-46 — `AgeStore | Keyring | EnvVar | ConfigFile` — Where a secret was resolved from.
- pub `resolve_api_key` function L66-93 — `(backend: &Backend, config_value: Option<&str>) -> Option<ResolvedSecret>` — Resolve an API key for a backend using the full resolution chain.
- pub `has_age_store_entry` function L96-98 — `(backend: &Backend) -> bool` — Check if the age store has a key for this backend.
- pub `store_secret` function L108-111 — `(backend: &Backend, api_key: &str) -> std::result::Result<(), String>` — Store an API key in the age-encrypted secret store.
- pub `store_named_secret` function L121-127 — `(name: &str, value: &str) -> std::result::Result<(), String>` — Store a named secret in the age-encrypted secret store.
- pub `delete_secret` function L130-133 — `(backend: &Backend) -> std::result::Result<(), String>` — Delete an API key from the age-encrypted secret store.
- pub `delete_named_secret` function L136-143 — `(name: &str) -> std::result::Result<(), String>` — Delete a named secret from the age-encrypted secret store.
- pub `list_secrets` function L156-160 — `() -> std::result::Result<Vec<String>, String>` — List all secret names in the age store.
- pub `has_keyring_entry` function L163-165 — `(backend: &Backend) -> bool` — Check if an entry exists (age store or keyring).
- pub `store_in_keyring` function L168-171 — `(backend: &Backend, api_key: &str) -> std::result::Result<(), String>` — Store an API key in the system keyring (legacy).
- pub `delete_from_keyring` function L174-177 — `(backend: &Backend) -> std::result::Result<(), String>` — Delete an API key from the system keyring (legacy).
-  `ResolvedSecret` type L26-33 — `= ResolvedSecret` — as a legacy fallback but disabled by default.
-  `fmt` function L27-32 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — as a legacy fallback but disabled by default.
-  `SecretSource` type L48-57 — `= SecretSource` — as a legacy fallback but disabled by default.
-  `fmt` function L49-56 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — as a legacy fallback but disabled by default.
-  `age_store_name` function L184-186 — `(backend: &Backend) -> String` — The secret name used for backend API keys in the age store.
-  `get_from_age_store` function L188-206 — `(backend: &Backend) -> Option<ResolvedSecret>` — as a legacy fallback but disabled by default.
-  `KEYRING_SERVICE` variable L213 — `: &str` — Keyring service name (legacy).
-  `keyring_user` function L216-218 — `(backend: &Backend) -> String` — Keyring user name for a backend (legacy).
-  `get_from_keyring` function L221-236 — `(backend: &Backend) -> Option<ResolvedSecret>` — as a legacy fallback but disabled by default.
-  `store_keyring_entry` function L239-247 — `(service: &str, user: &str, secret: &str) -> std::result::Result<(), String>` — as a legacy fallback but disabled by default.
-  `delete_keyring_entry` function L250-258 — `(service: &str, user: &str) -> std::result::Result<(), String>` — as a legacy fallback but disabled by default.
-  `get_from_keyring` function L265-267 — `(_backend: &Backend) -> Option<ResolvedSecret>` — as a legacy fallback but disabled by default.
-  `store_keyring_entry` function L270-276 — `( _service: &str, _user: &str, _secret: &str, ) -> std::result::Result<(), Strin...` — as a legacy fallback but disabled by default.
-  `delete_keyring_entry` function L279-281 — `(_service: &str, _user: &str) -> std::result::Result<(), String>` — as a legacy fallback but disabled by default.
-  `tests` module L288-356 — `-` — as a legacy fallback but disabled by default.
-  `test_age_store_name_format` function L292-296 — `()` — as a legacy fallback but disabled by default.
-  `test_resolve_from_config_value` function L299-306 — `()` — as a legacy fallback but disabled by default.
-  `test_resolve_none_when_nothing_available` function L309-313 — `()` — as a legacy fallback but disabled by default.
-  `test_secret_source_display` function L316-327 — `()` — as a legacy fallback but disabled by default.
-  `test_has_keyring_entry_no_panic` function L330-332 — `()` — as a legacy fallback but disabled by default.
-  `test_store_keyring_disabled` function L336-340 — `()` — as a legacy fallback but disabled by default.
-  `test_resolved_secret_debug_redacts_value` function L343-355 — `()` — as a legacy fallback but disabled by default.

#### crates/arawn-config/src/types.rs

- pub `ArawnConfig` struct L27-81 — `{ llm: Option<LlmConfig>, llm_profiles: HashMap<String, LlmConfig>, agent: HashM...` — Root configuration structure.
- pub `new` function L85-87 — `() -> Self` — Create an empty config.
- pub `from_toml` function L101-105 — `(toml_str: &str) -> crate::Result<Self>` — Parse from a TOML string.
- pub `to_toml` function L108-112 — `(&self) -> crate::Result<String>` — Serialize to a TOML string.
- pub `merge` function L131-199 — `(&mut self, other: ArawnConfig)` — Merge another config on top of this one (other takes priority).
- pub `resolve_llm` function L226-243 — `(&self, agent_name: &str) -> crate::Result<&LlmConfig>` — Resolve the LLM config for a given agent name.
- pub `llm_names` function L256-264 — `(&self) -> Vec<String>` — Get all defined LLM config names (including "default" for the bare [llm]).
- pub `LlmConfig` struct L422-438 — `{ backend: Option<Backend>, model: Option<String>, base_url: Option<String>, api...` — Configuration for an LLM backend.
- pub `has_plaintext_api_key` function L442-444 — `(&self) -> bool` — Returns true if an API key is stored directly in the config file.
- pub `api_key_env_var` function L447-449 — `(&self) -> Option<&'static str>` — Get the environment variable name for this backend's API key.
- pub `require_max_context_tokens` function L452-459 — `(&self) -> crate::Result<usize>` — Get the maximum context tokens, returning an error if not configured.
- pub `Backend` enum L465-473 — `Anthropic | Openai | Groq | Ollama | Custom | ClaudeOauth` — Supported LLM backend providers.
- pub `env_var` function L477-486 — `(&self) -> &'static str` — Environment variable name for this backend's API key.
- pub `display_name` function L489-498 — `(&self) -> &'static str` — Human-readable name.
- pub `AgentProfileConfig` struct L527-536 — `{ llm: Option<String>, system_prompt: Option<String>, max_iterations: Option<u32...` — Per-agent configuration.
- pub `ServerConfig` struct L558-578 — `{ port: u16, bind: String, rate_limiting: bool, api_rpm: u32, request_logging: b...` — Server configuration.
- pub `LoggingConfig` struct L602-605 — `{ interactions: InteractionLogConfig }` — Logging configuration section.
- pub `InteractionLogConfig` struct L610-617 — `{ enabled: bool, path: Option<PathBuf>, retention_days: u32 }` — Settings for structured interaction logging (JSONL).
- pub `EmbeddingConfig` struct L648-657 — `{ provider: EmbeddingProvider, dimensions: Option<usize>, openai: Option<Embeddi...` — Embedding provider configuration.
- pub `effective_dimensions` function L672-687 — `(&self) -> usize` — Effective dimensions for the configured provider.
- pub `EmbeddingProvider` enum L693-700 — `Local | OpenAi | Mock` — Supported embedding providers.
- pub `EmbeddingOpenAiConfig` struct L705-714 — `{ model: String, dimensions: Option<usize>, base_url: Option<String>, api_key: O...` — OpenAI embedding provider settings.
- pub `EmbeddingLocalConfig` struct L731-742 — `{ model_path: Option<PathBuf>, tokenizer_path: Option<PathBuf>, model_url: Optio...` — Local ONNX embedding settings.
- pub `MemoryConfig` struct L759-769 — `{ database: Option<PathBuf>, recall: RecallConfig, indexing: IndexingConfig, con...` — Memory subsystem configuration.
- pub `RecallConfig` struct L777-784 — `{ enabled: bool, threshold: f32, limit: usize }` — Configuration for active recall behavior.
- pub `IndexingConfig` struct L808-830 — `{ enabled: bool, backend: String, model: String, ner_model_path: Option<String>,...` — Configuration for session indexing pipeline.
- pub `ConfidenceConfig` struct L858-867 — `{ fresh_days: f32, staleness_days: f32, staleness_floor: f32, reinforcement_cap:...` — Configuration for confidence scoring parameters.
- pub `DelegationConfig` struct L901-906 — `{ max_result_len: usize, compaction: CompactionConfig }` — Subagent delegation configuration.
- pub `CompactionConfig` struct L924-936 — `{ enabled: bool, threshold: usize, backend: String, model: String, target_len: u...` — Configuration for LLM-based result compaction.
- pub `PluginsConfig` struct L969-982 — `{ enabled: bool, dirs: Vec<PathBuf>, hot_reload: bool, auto_update: bool, subscr...` — Plugin system configuration.
- pub `PluginSubscription` struct L1003-1021 — `{ source: PluginSource, repo: Option<String>, url: Option<String>, path: Option<...` — A plugin subscription defining where to fetch a plugin from.
- pub `github` function L1029-1038 — `(repo: impl Into<String>) -> Self` — Create a GitHub subscription.
- pub `url` function L1041-1050 — `(url: impl Into<String>) -> Self` — Create a URL subscription.
- pub `local` function L1053-1062 — `(path: impl Into<PathBuf>) -> Self` — Create a local path subscription.
- pub `with_ref` function L1065-1068 — `(mut self, git_ref: impl Into<String>) -> Self` — Set the git ref (branch, tag, or commit).
- pub `effective_ref` function L1071-1073 — `(&self) -> &str` — Get the effective git ref, defaulting to "main".
- pub `id` function L1078-1100 — `(&self) -> String` — Generate a unique identifier for this subscription.
- pub `clone_url` function L1103-1112 — `(&self) -> Option<String>` — Get the clone URL for this subscription.
- pub `PluginSource` enum L1127-1134 — `GitHub | Url | Local` — Source type for plugin subscriptions.
- pub `PipelineSection` struct L1155-1174 — `{ enabled: bool, database: Option<PathBuf>, workflow_dir: Option<PathBuf>, max_c...` — Pipeline / workflow engine configuration.
- pub `McpConfig` struct L1217-1223 — `{ enabled: bool, servers: Vec<McpServerEntry> }` — MCP (Model Context Protocol) configuration.
- pub `McpTransportType` enum L1237-1243 — `Stdio | Http` — Transport type for MCP server connections.
- pub `McpServerEntry` struct L1249-1276 — `{ name: String, transport: McpTransportType, command: String, url: Option<String...` — Configuration for a single MCP server.
- pub `new` function L1280-1293 — `(name: impl Into<String>, command: impl Into<String>) -> Self` — Create a new MCP server entry for stdio transport.
- pub `http` function L1296-1309 — `(name: impl Into<String>, url: impl Into<String>) -> Self` — Create a new MCP server entry for HTTP transport.
- pub `with_arg` function L1312-1315 — `(mut self, arg: impl Into<String>) -> Self` — Add an argument (for stdio transport).
- pub `with_args` function L1318-1321 — `(mut self, args: Vec<String>) -> Self` — Add arguments (for stdio transport).
- pub `with_env` function L1324-1327 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add an environment variable (for stdio transport).
- pub `with_header` function L1330-1333 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add an HTTP header (for HTTP transport).
- pub `with_timeout_secs` function L1336-1339 — `(mut self, timeout: u64) -> Self` — Set request timeout in seconds (for HTTP transport).
- pub `with_retries` function L1342-1345 — `(mut self, retries: u32) -> Self` — Set number of retries (for HTTP transport).
- pub `with_enabled` function L1348-1351 — `(mut self, enabled: bool) -> Self` — Set enabled state.
- pub `is_http` function L1354-1356 — `(&self) -> bool` — Check if this is an HTTP transport.
- pub `is_stdio` function L1359-1361 — `(&self) -> bool` — Check if this is a stdio transport.
- pub `env_tuples` function L1364-1369 — `(&self) -> Vec<(String, String)>` — Convert environment variables to the tuple format expected by McpServerConfig.
- pub `header_tuples` function L1372-1377 — `(&self) -> Vec<(String, String)>` — Convert HTTP headers to the tuple format.
- pub `WorkstreamConfig` struct L1389-1400 — `{ database: Option<PathBuf>, data_dir: Option<PathBuf>, session_timeout_minutes:...` — Configuration for workstreams (persistent conversation contexts).
- pub `CompressionConfig` struct L1428-1440 — `{ enabled: bool, backend: String, model: String, max_summary_tokens: u32, token_...` — Configuration for automatic session/workstream compression.
- pub `SessionConfig` struct L1470-1475 — `{ max_sessions: usize, cleanup_interval_secs: u64 }` — Session cache configuration.
- pub `ToolsConfig` struct L1519-1526 — `{ output: ToolOutputConfig, shell: ShellToolConfig, web: WebToolConfig }` — Tool execution configuration.
- pub `ToolOutputConfig` struct L1536-1548 — `{ max_size_bytes: usize, shell: Option<usize>, file_read: Option<usize>, web_fet...` — Tool output configuration.
- pub `ShellToolConfig` struct L1565-1568 — `{ timeout_secs: u64 }` — Shell tool configuration.
- pub `WebToolConfig` struct L1579-1582 — `{ timeout_secs: u64 }` — Web tool configuration.
- pub `RlmTomlConfig` struct L1627-1642 — `{ model: Option<String>, max_turns: Option<u32>, max_context_tokens: Option<usiz...` — Configuration for the RLM (Recursive Language Model) exploration agent.
- pub `OAuthConfigOverride` struct L1665-1676 — `{ client_id: Option<String>, authorize_url: Option<String>, token_url: Option<St...` — OAuth configuration overrides for the `[oauth]` TOML section.
-  `ArawnConfig` type L83-265 — `= ArawnConfig` — ```
-  `lookup_llm` function L246-253 — `(&'a self, name: &str, context: &str) -> crate::Result<&'a LlmConfig>` — Look up a named LLM config.
-  `RawConfig` struct L277-295 — `{ llm: Option<RawLlmSection>, agent: HashMap<String, AgentProfileConfig>, server...` — Internal raw config matching the actual TOML layout.
-  `RawLlmSection` struct L300-319 — `{ backend: Option<Backend>, model: Option<String>, base_url: Option<String>, api...` — The `[llm]` section which can contain both direct fields and named sub-tables.
-  `ArawnConfig` type L321-363 — `= ArawnConfig` — ```
-  `from` function L322-362 — `(raw: RawConfig) -> Self` — ```
-  `RawConfig` type L365-402 — `= RawConfig` — ```
-  `from` function L366-401 — `(config: ArawnConfig) -> Self` — ```
-  `LlmConfig` type L440-460 — `= LlmConfig` — ```
-  `Backend` type L475-499 — `= Backend` — ```
-  `Backend` type L501-505 — `= Backend` — ```
-  `fmt` function L502-504 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `ServerConfig` type L580-593 — `impl Default for ServerConfig` — ```
-  `default` function L581-592 — `() -> Self` — ```
-  `InteractionLogConfig` type L619-627 — `impl Default for InteractionLogConfig` — ```
-  `default` function L620-626 — `() -> Self` — ```
-  `EmbeddingConfig` type L659-668 — `impl Default for EmbeddingConfig` — ```
-  `default` function L660-667 — `() -> Self` — ```
-  `EmbeddingConfig` type L670-688 — `= EmbeddingConfig` — ```
-  `EmbeddingOpenAiConfig` type L716-725 — `impl Default for EmbeddingOpenAiConfig` — ```
-  `default` function L717-724 — `() -> Self` — ```
-  `RecallConfig` type L786-794 — `impl Default for RecallConfig` — ```
-  `default` function L787-793 — `() -> Self` — ```
-  `IndexingConfig` type L832-845 — `impl Default for IndexingConfig` — ```
-  `default` function L833-844 — `() -> Self` — ```
-  `ConfidenceConfig` type L869-878 — `impl Default for ConfidenceConfig` — ```
-  `default` function L870-877 — `() -> Self` — ```
-  `DelegationConfig` type L908-915 — `impl Default for DelegationConfig` — ```
-  `default` function L909-914 — `() -> Self` — ```
-  `CompactionConfig` type L938-948 — `impl Default for CompactionConfig` — ```
-  `default` function L939-947 — `() -> Self` — ```
-  `PluginsConfig` type L984-994 — `impl Default for PluginsConfig` — ```
-  `default` function L985-993 — `() -> Self` — ```
-  `default_enabled` function L1023-1025 — `() -> bool` — ```
-  `PluginSubscription` type L1027-1113 — `= PluginSubscription` — ```
-  `simple_hash` function L1116-1122 — `(s: &str) -> u64` — Simple hash function for generating stable identifiers.
-  `PipelineSection` type L1176-1189 — `impl Default for PipelineSection` — ```
-  `default` function L1177-1188 — `() -> Self` — ```
-  `McpConfig` type L1225-1232 — `impl Default for McpConfig` — ```
-  `default` function L1226-1231 — `() -> Self` — ```
-  `McpServerEntry` type L1278-1378 — `= McpServerEntry` — ```
-  `WorkstreamConfig` type L1402-1411 — `impl Default for WorkstreamConfig` — ```
-  `default` function L1403-1410 — `() -> Self` — ```
-  `CompressionConfig` type L1442-1452 — `impl Default for CompressionConfig` — ```
-  `default` function L1443-1451 — `() -> Self` — ```
-  `SessionConfig` type L1477-1484 — `impl Default for SessionConfig` — ```
-  `default` function L1478-1483 — `() -> Self` — ```
-  `SessionConfig` type L1486 — `= SessionConfig` — ```
-  `SessionConfig` type L1488-1496 — `= SessionConfig` — ```
-  `max_sessions` function L1489-1491 — `(&self) -> usize` — ```
-  `cleanup_interval` function L1493-1495 — `(&self) -> std::time::Duration` — ```
-  `ToolOutputConfig` type L1550-1560 — `impl Default for ToolOutputConfig` — ```
-  `default` function L1551-1559 — `() -> Self` — ```
-  `ShellToolConfig` type L1570-1574 — `impl Default for ShellToolConfig` — ```
-  `default` function L1571-1573 — `() -> Self` — ```
-  `WebToolConfig` type L1584-1588 — `impl Default for WebToolConfig` — ```
-  `default` function L1585-1587 — `() -> Self` — ```
-  `ToolsConfig` type L1590 — `= ToolsConfig` — ```
-  `ToolsConfig` type L1592-1604 — `= ToolsConfig` — ```
-  `shell_timeout` function L1593-1595 — `(&self) -> std::time::Duration` — ```
-  `web_timeout` function L1597-1599 — `(&self) -> std::time::Duration` — ```
-  `max_output_bytes` function L1601-1603 — `(&self) -> usize` — ```
-  `tests` module L1683-3081 — `-` — ```
-  `test_empty_config` function L1687-1693 — `()` — ```
-  `test_parse_minimal` function L1696-1706 — `()` — ```
-  `test_parse_named_profiles` function L1709-1733 — `()` — ```
-  `test_parse_agents` function L1736-1759 — `()` — ```
-  `test_resolve_llm_agent_specific` function L1762-1778 — `()` — ```
-  `test_resolve_llm_agent_default` function L1781-1798 — `()` — ```
-  `test_resolve_llm_global_default` function L1801-1810 — `()` — ```
-  `test_resolve_llm_no_default` function L1813-1817 — `()` — ```
-  `test_resolve_llm_missing_reference` function L1820-1832 — `()` — ```
-  `test_merge_override` function L1835-1860 — `()` — ```
-  `test_merge_adds_profiles` function L1863-1887 — `()` — ```
-  `test_server_defaults` function L1890-1899 — `()` — ```
-  `test_backend_env_var` function L1902-1906 — `()` — ```
-  `test_plaintext_api_key_warning` function L1909-1918 — `()` — ```
-  `test_llm_names` function L1921-1938 — `()` — ```
-  `test_parse_full_example` function L1941-2001 — `()` — ```
-  `test_roundtrip_toml` function L2004-2020 — `()` — ```
-  `test_embedding_defaults` function L2025-2029 — `()` — ```
-  `test_embedding_explicit_dimensions` function L2032-2039 — `()` — ```
-  `test_embedding_openai_default_dimensions` function L2042-2050 — `()` — ```
-  `test_embedding_openai_provider_dimensions` function L2053-2064 — `()` — ```
-  `test_parse_embedding_config` function L2067-2085 — `()` — ```
-  `test_parse_embedding_local_default` function L2088-2097 — `()` — ```
-  `test_no_embedding_section_uses_default` function L2100-2111 — `()` — ```
-  `test_merge_embedding_override` function L2114-2131 — `()` — ```
-  `test_pipeline_defaults` function L2136-2144 — `()` — ```
-  `test_parse_pipeline_config` function L2147-2172 — `()` — ```
-  `test_parse_pipeline_disabled` function L2175-2183 — `()` — ```
-  `test_no_pipeline_section_uses_default` function L2186-2196 — `()` — ```
-  `test_recall_defaults` function L2201-2206 — `()` — ```
-  `test_parse_recall_config` function L2209-2221 — `()` — ```
-  `test_no_memory_section_uses_default` function L2224-2236 — `()` — ```
-  `test_merge_memory_override` function L2239-2260 — `()` — ```
-  `test_memory_indexing_defaults` function L2263-2272 — `()` — ```
-  `test_memory_confidence_defaults` function L2275-2285 — `()` — ```
-  `test_memory_indexing_override` function L2288-2300 — `()` — ```
-  `test_memory_confidence_override` function L2303-2317 — `()` — ```
-  `test_memory_partial_sections` function L2320-2333 — `()` — ```
-  `test_merge_memory_with_indexing` function L2336-2355 — `()` — ```
-  `test_merge_pipeline_override` function L2358-2377 — `()` — ```
-  `test_plugins_defaults` function L2382-2389 — `()` — ```
-  `test_plugin_subscription_github` function L2392-2398 — `()` — ```
-  `test_plugin_subscription_url` function L2401-2408 — `()` — ```
-  `test_plugin_subscription_local` function L2411-2418 — `()` — ```
-  `test_plugin_subscription_with_ref` function L2421-2425 — `()` — ```
-  `test_plugin_subscription_id` function L2428-2437 — `()` — ```
-  `test_plugin_subscription_clone_url` function L2440-2455 — `()` — ```
-  `test_parse_plugin_subscriptions` function L2458-2508 — `()` — ```
-  `test_parse_plugins_no_subscriptions` function L2511-2520 — `()` — ```
-  `test_delegation_defaults` function L2525-2533 — `()` — ```
-  `test_compaction_defaults` function L2536-2543 — `()` — ```
-  `test_parse_delegation_config` function L2546-2566 — `()` — ```
-  `test_parse_delegation_compaction_disabled` function L2569-2584 — `()` — ```
-  `test_no_delegation_section_uses_default` function L2587-2598 — `()` — ```
-  `test_merge_delegation_override` function L2601-2628 — `()` — ```
-  `test_mcp_defaults` function L2633-2637 — `()` — ```
-  `test_mcp_server_entry_new` function L2640-2647 — `()` — ```
-  `test_mcp_server_entry_builder` function L2650-2659 — `()` — ```
-  `test_mcp_server_entry_env_tuples` function L2662-2674 — `()` — ```
-  `test_parse_mcp_config` function L2677-2711 — `()` — ```
-  `test_parse_mcp_disabled` function L2714-2723 — `()` — ```
-  `test_no_mcp_section_uses_default` function L2726-2737 — `()` — ```
-  `test_merge_mcp_override` function L2740-2766 — `()` — ```
-  `test_model_config_parses_max_context_tokens` function L2771-2781 — `()` — ```
-  `test_model_config_context_tokens_in_profile` function L2784-2803 — `()` — ```
-  `test_require_max_context_tokens_success` function L2806-2813 — `()` — ```
-  `test_require_max_context_tokens_error` function L2816-2827 — `()` — ```
-  `test_model_context_roundtrip` function L2830-2844 — `()` — ```
-  `test_parse_paths_config` function L2849-2882 — `()` — ```
-  `test_no_paths_section_uses_default` function L2885-2897 — `()` — ```
-  `test_merge_paths_override` function L2900-2925 — `()` — ```
-  `test_paths_roundtrip` function L2928-2959 — `()` — ```
-  `test_tool_output_config_per_tool_fields` function L2962-2978 — `()` — ```
-  `test_tool_output_config_defaults_none` function L2981-2993 — `()` — ```
-  `test_rlm_config_deserialization` function L2996-3019 — `()` — ```
-  `test_rlm_config_defaults` function L3022-3035 — `()` — ```
-  `test_rlm_config_partial` function L3038-3050 — `()` — ```
-  `test_rlm_config_absent` function L3053-3056 — `()` — ```
-  `test_rlm_config_merge` function L3059-3080 — `()` — ```

### crates/arawn-domain/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-domain/src/error.rs

- pub `DomainError` enum L7-35 — `SessionNotFound | WorkstreamNotFound | Agent | Mcp | Workstream | Config | Inter...` — Domain-level errors.
- pub `Result` type L38 — `= std::result::Result<T, DomainError>` — Result type for domain operations.

#### crates/arawn-domain/src/lib.rs

- pub `services` module L24 — `-` — ```
-  `error` module L23 — `-` — This crate provides a unified interface for orchestrating the core Arawn

### crates/arawn-domain/src/services

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-domain/src/services/chat.rs

- pub `ChatResponse` struct L24-37 — `{ session_id: SessionId, response: String, truncated: bool, input_tokens: u32, o...` — Response from a chat turn.
- pub `ToolCallSummary` struct L41-48 — `{ id: String, name: String, success: bool }` — Summary of a tool call.
- pub `TurnOptions` struct L60-63 — `{ max_message_bytes: Option<usize> }` — Options for executing a turn.
- pub `ChatService` struct L67-72 — `{ agent: Arc<Agent>, workstreams: Option<Arc<WorkstreamManager>>, directory_mana...` — Chat service for conversation orchestration.
- pub `new` function L76-88 — `( agent: Arc<Agent>, workstreams: Option<Arc<WorkstreamManager>>, directory_mana...` — Create a new chat service.
- pub `agent` function L91-93 — `(&self) -> &Arc<Agent>` — Get the underlying agent.
- pub `workstreams` function L96-98 — `(&self) -> Option<&Arc<WorkstreamManager>>` — Get the workstream manager.
- pub `directory_manager` function L101-103 — `(&self) -> Option<&Arc<DirectoryManager>>` — Get the directory manager.
- pub `indexer` function L106-108 — `(&self) -> Option<&Arc<SessionIndexer>>` — Get the session indexer.
- pub `turn` function L117-141 — `( &self, session: &mut Session, message: &str, workstream_id: Option<&str>, ) ->...` — Execute a chat turn with an existing session.
- pub `create_scratch_session` function L144-150 — `(&self, session_id: &str) -> Result<()>` — Create a scratch session directory.
- pub `allowed_paths` function L153-161 — `( &self, workstream_id: &str, session_id: &str, ) -> Option<Vec<std::path::PathB...` — Get allowed paths for a session.
- pub `index_session` function L164-185 — `(&self, session_id: &str, session: &Session)` — Index a closed session for memory search.
-  `ChatService` type L74-215 — `= ChatService` — and workstream persistence.
-  `build_response` function L188-214 — `(&self, session_id: SessionId, response: &AgentResponse) -> ChatResponse` — Build a ChatResponse from an AgentResponse.
-  `session_to_messages` function L218-227 — `(session: &Session) -> Vec<(String, String)>` — Convert a session's turns into owned `(role, content)` pairs.
-  `messages_as_refs` function L230-235 — `(messages: &[(String, String)]) -> Vec<(&str, &str)>` — Convert owned message pairs to borrowed slices for the indexer API.
-  `tests` module L238-280 — `-` — and workstream persistence.
-  `create_test_agent` function L243-252 — `() -> Arc<Agent>` — and workstream persistence.
-  `test_chat_turn` function L255-264 — `()` — and workstream persistence.
-  `test_session_to_messages` function L267-279 — `()` — and workstream persistence.

#### crates/arawn-domain/src/services/mcp.rs

- pub `SharedMcpManager` type L15 — `= Arc<RwLock<McpManager>>` — Shared MCP manager type.
- pub `McpServerInfo` struct L19-28 — `{ name: String, command: String, connected: bool, tool_count: usize }` — Information about an MCP server.
- pub `McpToolInfo` struct L32-39 — `{ name: String, description: Option<String>, server: String }` — Information about an MCP tool.
- pub `McpService` struct L43-45 — `{ manager: Option<SharedMcpManager> }` — MCP service for tool discovery and management.
- pub `new` function L49-51 — `(manager: Option<SharedMcpManager>) -> Self` — Create a new MCP service.
- pub `is_enabled` function L54-56 — `(&self) -> bool` — Check if MCP is enabled.
- pub `manager` function L59-61 — `(&self) -> Option<&SharedMcpManager>` — Get the MCP manager.
- pub `list_server_names` function L64-75 — `(&self) -> Result<Vec<String>>` — List all configured MCP server names.
- pub `is_server_connected` function L78-86 — `(&self, name: &str) -> Result<bool>` — Check if a server is connected.
- pub `add_server` function L89-101 — `(&self, config: McpServerConfig) -> Result<()>` — Add a new MCP server configuration.
- pub `remove_server` function L104-117 — `(&self, name: &str) -> Result<bool>` — Remove an MCP server.
- pub `connect_all` function L120-133 — `(&self) -> Result<()>` — Connect to all configured MCP servers.
- pub `shutdown_all` function L136-149 — `(&self) -> Result<()>` — Shutdown all MCP server connections.
-  `McpService` type L47-150 — `= McpService` — and their tools.
-  `tests` module L153-161 — `-` — and their tools.
-  `test_mcp_service_disabled` function L157-160 — `()` — and their tools.

#### crates/arawn-domain/src/services/memory.rs

- pub `MemoryService` struct L17-19 — `{ store: Option<Arc<MemoryStore>> }` — Domain service for memory and note operations.
- pub `new` function L23-25 — `(store: Option<Arc<MemoryStore>>) -> Self` — Create a new memory service.
- pub `is_enabled` function L28-30 — `(&self) -> bool` — Whether the memory store is available.
- pub `store` function L35-37 — `(&self) -> Option<&Arc<MemoryStore>>` — Get the underlying memory store.
-  `MemoryService` type L21-38 — `= MemoryService` — agent's internal memory share the same backing store.

#### crates/arawn-domain/src/services/mod.rs

- pub `chat` module L6 — `-` — This module contains the core domain services that orchestrate
- pub `mcp` module L7 — `-` — Arawn's functionality.
- pub `memory` module L8 — `-` — Arawn's functionality.
- pub `DomainServices` struct L34-41 — `{ chat: chat::ChatService, mcp: mcp::McpService, memory: MemoryService }` — Domain services facade.
- pub `new` function L47-68 — `( agent: Arc<Agent>, workstreams: Option<Arc<WorkstreamManager>>, directory_mana...` — Create new domain services with the given components.
- pub `chat` function L71-73 — `(&self) -> &chat::ChatService` — Get the chat service.
- pub `mcp` function L76-78 — `(&self) -> &mcp::McpService` — Get the MCP service.
- pub `memory` function L81-83 — `(&self) -> &MemoryService` — Get the memory service.
- pub `agent` function L86-88 — `(&self) -> &Arc<Agent>` — Get the underlying agent.
-  `DomainServices` type L43-89 — `= DomainServices` — Arawn's functionality.
-  `tests` module L92-134 — `-` — Arawn's functionality.
-  `create_test_agent` function L97-106 — `() -> Arc<Agent>` — Arawn's functionality.
-  `test_domain_services_creation` function L109-116 — `()` — Arawn's functionality.
-  `test_domain_services_memory_disabled` function L119-124 — `()` — Arawn's functionality.
-  `test_domain_services_memory_enabled` function L127-133 — `()` — Arawn's functionality.

### crates/arawn-llm/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-llm/src/anthropic.rs

- pub `AnthropicConfig` struct L52-70 — `{ api_key: ApiKeyProvider, base_url: String, api_version: String, timeout: Durat...` — Configuration for the Anthropic backend.
- pub `new` function L74-83 — `(api_key: impl Into<String>) -> Self` — Create a new config with the given API key.
- pub `from_env` function L86-91 — `() -> Result<Self>` — Create config from environment variable.
- pub `with_base_url` function L94-97 — `(mut self, url: impl Into<String>) -> Self` — Set a custom base URL.
- pub `with_timeout` function L100-103 — `(mut self, timeout: Duration) -> Self` — Set request timeout.
- pub `with_max_retries` function L106-109 — `(mut self, retries: u32) -> Self` — Set max retries.
- pub `with_retry_backoff` function L112-115 — `(mut self, backoff: Duration) -> Self` — Set retry backoff.
- pub `AnthropicBackend` struct L123-126 — `{ client: Client, config: AnthropicConfig }` — Anthropic API backend.
- pub `new` function L130-137 — `(config: AnthropicConfig) -> Result<Self>` — Create a new Anthropic backend with the given configuration.
- pub `from_env` function L140-142 — `() -> Result<Self>` — Create a backend from environment configuration.
- pub `create_shared_backend` function L259-261 — `(config: AnthropicConfig) -> Result<Arc<dyn LlmBackend>>` — Create a shared Anthropic backend.
-  `DEFAULT_API_BASE` variable L20 — `: &str` — Default API base URL.
-  `DEFAULT_API_VERSION` variable L23 — `: &str` — Default API version.
-  `DEFAULT_TIMEOUT_SECS` variable L26 — `: u64` — Default timeout for requests.
-  `DEFAULT_MAX_RETRIES` variable L29 — `: u32` — Default maximum retries for transient errors.
-  `DEFAULT_RETRY_BACKOFF_MS` variable L32 — `: u64` — Default initial backoff between retries.
-  `AnthropicConfig` type L72-116 — `= AnthropicConfig` — Messages API for Claude completions.
-  `AnthropicBackend` type L128-205 — `= AnthropicBackend` — Messages API for Claude completions.
-  `messages_url` function L145-147 — `(&self) -> String` — Build the messages endpoint URL.
-  `add_headers` function L150-158 — `(&self, builder: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder>` — Add authentication and API headers to a request.
-  `handle_response` function L161-171 — `(response: Response) -> Result<CompletionResponse>` — Handle a successful response.
-  `handle_error_response` function L174-204 — `(response: Response) -> LlmError` — Handle an error response.
-  `AnthropicBackend` type L208-256 — `impl LlmBackend for AnthropicBackend` — Messages API for Claude completions.
-  `complete` function L209-229 — `(&self, request: CompletionRequest) -> Result<CompletionResponse>` — Messages API for Claude completions.
-  `complete_stream` function L231-247 — `(&self, request: CompletionRequest) -> Result<ResponseStream>` — Messages API for Claude completions.
-  `name` function L249-251 — `(&self) -> &str` — Messages API for Claude completions.
-  `supports_native_tools` function L253-255 — `(&self) -> bool` — Messages API for Claude completions.
-  `ApiResponse` struct L269-277 — `{ id: String, response_type: String, content: Vec<ApiContentBlock>, model: Strin...` — Internal API response structure.
-  `CompletionResponse` type L279-321 — `= CompletionResponse` — Messages API for Claude completions.
-  `from` function L280-320 — `(api: ApiResponse) -> Self` — Messages API for Claude completions.
-  `ApiContentBlock` enum L325-334 — `Text | ToolUse` — Messages API for Claude completions.
-  `ApiUsage` struct L337-342 — `{ input_tokens: u32, output_tokens: u32, cache_creation_input_tokens: Option<u32...` — Messages API for Claude completions.
-  `ApiError` struct L345-347 — `{ error: ApiErrorDetail }` — Messages API for Claude completions.
-  `ApiErrorDetail` struct L350-352 — `{ message: String }` — Messages API for Claude completions.
-  `parse_sse_stream` function L359-425 — `( byte_stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static, ) ->...` — Parse SSE events from a byte stream and convert to StreamEvents.
-  `SseState` struct L427-432 — `{ byte_stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>, buffe...` — Messages API for Claude completions.
-  `parse_sse_line` function L434-442 — `(line: &str) -> Option<(&str, &str)>` — Messages API for Claude completions.
-  `parse_stream_event` function L444-523 — `(event_type: &str, data: &str) -> Option<StreamEvent>` — Messages API for Claude completions.
-  `MessageStartEvent` struct L530-532 — `{ message: MessageStartMessage }` — Messages API for Claude completions.
-  `MessageStartMessage` struct L535-538 — `{ id: String, model: String }` — Messages API for Claude completions.
-  `ContentBlockStartEvent` struct L541-544 — `{ index: usize, content_block: ContentBlockType }` — Messages API for Claude completions.
-  `ContentBlockType` struct L547-550 — `{ block_type: String }` — Messages API for Claude completions.
-  `ContentBlockDeltaEvent` struct L553-556 — `{ index: usize, delta: DeltaContent }` — Messages API for Claude completions.
-  `DeltaContent` enum L560-563 — `TextDelta | InputJsonDelta` — Messages API for Claude completions.
-  `ContentBlockStopEvent` struct L566-568 — `{ index: usize }` — Messages API for Claude completions.
-  `MessageDeltaEvent` struct L571-574 — `{ delta: MessageDelta, usage: MessageDeltaUsage }` — Messages API for Claude completions.
-  `MessageDelta` struct L577-579 — `{ stop_reason: Option<String> }` — Messages API for Claude completions.
-  `MessageDeltaUsage` struct L582-584 — `{ output_tokens: u32 }` — Messages API for Claude completions.
-  `StreamErrorEvent` struct L587-589 — `{ error: StreamErrorDetail }` — Messages API for Claude completions.
-  `StreamErrorDetail` struct L592-594 — `{ message: String }` — Messages API for Claude completions.
-  `tests` module L601-729 — `-` — Messages API for Claude completions.
-  `test_config_new` function L605-610 — `()` — Messages API for Claude completions.
-  `test_config_with_base_url` function L613-616 — `()` — Messages API for Claude completions.
-  `test_config_with_timeout` function L619-622 — `()` — Messages API for Claude completions.
-  `test_parse_sse_line` function L625-635 — `()` — Messages API for Claude completions.
-  `test_api_response_conversion` function L638-661 — `()` — Messages API for Claude completions.
-  `test_api_response_with_tool_use` function L664-697 — `()` — Messages API for Claude completions.
-  `test_messages_url` function L700-707 — `()` — Messages API for Claude completions.
-  `test_messages_url_custom_base` function L710-714 — `()` — Messages API for Claude completions.
-  `test_backend_name` function L717-721 — `()` — Messages API for Claude completions.
-  `test_supports_native_tools` function L724-728 — `()` — Messages API for Claude completions.

#### crates/arawn-llm/src/api_key.rs

- pub `ApiKeyProvider` enum L14-21 — `None | Static | Dynamic` — Provides API keys for LLM backends.
- pub `resolve` function L25-31 — `(&self) -> Option<String>` — Resolve the current API key value.
- pub `from_static` function L34-36 — `(key: impl Into<String>) -> Self` — Create a static provider from a string.
- pub `dynamic` function L39-41 — `(resolver: impl Fn() -> Option<String> + Send + Sync + 'static) -> Self` — Create a dynamic provider from a closure.
-  `ApiKeyProvider` type L23-42 — `= ApiKeyProvider` — on each request, enabling hot-loading of secrets without server restart.
-  `ApiKeyProvider` type L44-52 — `= ApiKeyProvider` — on each request, enabling hot-loading of secrets without server restart.
-  `fmt` function L45-51 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — on each request, enabling hot-loading of secrets without server restart.
-  `ApiKeyProvider` type L54-58 — `= ApiKeyProvider` — on each request, enabling hot-loading of secrets without server restart.
-  `from` function L55-57 — `(s: String) -> Self` — on each request, enabling hot-loading of secrets without server restart.
-  `ApiKeyProvider` type L60-67 — `= ApiKeyProvider` — on each request, enabling hot-loading of secrets without server restart.
-  `from` function L61-66 — `(opt: Option<String>) -> Self` — on each request, enabling hot-loading of secrets without server restart.
-  `tests` module L70-129 — `-` — on each request, enabling hot-loading of secrets without server restart.
-  `test_static_provider` function L74-77 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_none_provider` function L80-83 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_dynamic_provider` function L86-95 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_from_string` function L98-101 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_from_option_some` function L104-107 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_from_option_none` function L110-113 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_debug_redacts` function L116-121 — `()` — on each request, enabling hot-loading of secrets without server restart.
-  `test_clone` function L124-128 — `()` — on each request, enabling hot-loading of secrets without server restart.

#### crates/arawn-llm/src/backend.rs

- pub `with_retry` function L28-80 — `( max_retries: u32, initial_backoff: Duration, backend_name: &str, mut f: F, ) -...` — Execute an async operation with exponential backoff retry.
- pub `ResponseStream` type L87 — `= Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send + 'static>>` — A streaming response from an LLM backend.
- pub `StreamEvent` enum L91-111 — `MessageStart | ContentBlockStart | ContentBlockDelta | ContentBlockStop | Messag...` — Events emitted during streaming.
- pub `ContentDelta` enum L115-120 — `TextDelta | InputJsonDelta` — Delta content in a streaming response.
- pub `validate` function L127-173 — `(&self) -> std::result::Result<(), ResponseValidationError>` — Validate the stream event structure.
- pub `is_error` function L176-178 — `(&self) -> bool` — Returns true if this is an error event.
- pub `is_terminal` function L181-183 — `(&self) -> bool` — Returns true if this is the final event in a message.
- pub `ParsedToolCall` struct L188-195 — `{ id: String, name: String, arguments: serde_json::Value }` — A parsed tool call from model output.
- pub `LlmBackend` interface L236-304 — `{ fn complete(), fn complete_stream(), fn name(), fn supports_native_tools(), fn...` — Trait for LLM backend providers.
- pub `default_format_tool_definitions` function L307-337 — `(tools: &[ToolDefinition]) -> String` — Default human-readable format for tool definitions.
- pub `default_format_tool_result` function L340-346 — `(tool_use_id: &str, content: &str, is_error: bool) -> String` — Default format for tool results.
- pub `MockResponse` enum L355-360 — `Success | Error` — A response or error that can be returned by MockBackend.
- pub `MockBackend` struct L373-377 — `{ name: String, responses: std::sync::Mutex<Vec<MockResponse>>, request_log: std...` — Returns pre-configured responses in order, useful for deterministic testing
- pub `new` function L385-393 — `(responses: Vec<CompletionResponse>) -> Self` — Create a new mock backend with the given responses.
- pub `with_results` function L398-404 — `(responses: Vec<MockResponse>) -> Self` — Create a mock backend with mixed responses and errors.
- pub `with_text` function L407-418 — `(text: impl Into<String>) -> Self` — Create a mock backend with a single text response.
- pub `requests` function L421-423 — `(&self) -> Vec<CompletionRequest>` — Get all requests that were made to this backend.
- pub `request_count` function L426-428 — `(&self) -> usize` — Get the number of requests made.
- pub `SharedBackend` type L490 — `= Arc<dyn LlmBackend>` — A backend that can be shared across threads.
-  `StreamEvent` type L122-184 — `= StreamEvent` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `ContentBlock` type L197-206 — `= ContentBlock` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `from` function L198-205 — `(call: ParsedToolCall) -> Self` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `supports_native_tools` function L260-262 — `(&self) -> bool` — Returns true if backend handles tools natively via API.
-  `tool_calling_instructions` function L270-272 — `(&self) -> Option<&str>` — Instructions for HOW to call tools (model-specific format).
-  `format_tool_definitions` function L280-282 — `(&self, tools: &[ToolDefinition]) -> String` — Format tool definitions for the system prompt.
-  `format_tool_result` function L290-292 — `(&self, tool_use_id: &str, content: &str, is_error: bool) -> String` — Format a tool result for the conversation.
-  `parse_tool_calls` function L301-303 — `(&self, text: &str) -> (String, Vec<ParsedToolCall>)` — Parse tool calls from model text output.
-  `MockResponse` type L363-367 — `= MockResponse` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `from` function L364-366 — `(response: CompletionResponse) -> Self` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `MockBackend` type L380-429 — `= MockBackend` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `MockBackend` type L433-483 — `impl LlmBackend for MockBackend` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `complete` function L434-450 — `(&self, request: CompletionRequest) -> Result<CompletionResponse>` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `complete_stream` function L452-478 — `(&self, request: CompletionRequest) -> Result<ResponseStream>` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `name` function L480-482 — `(&self) -> &str` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `tests` module L497-747 — `-` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_mock_backend_single_response` function L502-510 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_mock_backend_multiple_responses` function L513-546 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_mock_backend_exhausted` function L549-556 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_mock_backend_with_tool_use` function L559-588 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_mock_backend_stream` function L591-608 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_default_format_tool_definitions` function L611-630 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_default_format_tool_result` function L633-639 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_stream_event_validate_message_start` function L646-664 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_stream_event_validate_content_block_start` function L667-680 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_stream_event_validate_error` function L683-693 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_stream_event_is_error` function L696-705 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_stream_event_is_terminal` function L708-718 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.
-  `test_stream_event_validate_other_events` function L721-746 — `()` — (Anthropic, OpenAI, local models) and provides mock implementations for testing.

#### crates/arawn-llm/src/client.rs

- pub `Provider` enum L41-50 — `Anthropic | OpenAi | Groq | Ollama` — Supported LLM providers.
- pub `name` function L54-61 — `(&self) -> &'static str` — Get the string name for this provider.
- pub `from_name` function L64-72 — `(name: &str) -> Option<Self>` — Parse a provider from a string name.
- pub `requires_api_key` function L75-80 — `(&self) -> bool` — Check if this provider requires an API key.
- pub `LlmClientConfig` struct L95-116 — `{ anthropic: Option<AnthropicConfig>, openai: Option<OpenAiConfig>, groq: Option...` — Configuration for the LLM client.
- pub `new` function L120-122 — `() -> Self` — Create a new empty configuration.
- pub `with_anthropic` function L125-128 — `(mut self, config: AnthropicConfig) -> Self` — Configure Anthropic backend.
- pub `with_openai` function L131-134 — `(mut self, config: OpenAiConfig) -> Self` — Configure OpenAI backend.
- pub `with_groq` function L137-140 — `(mut self, config: OpenAiConfig) -> Self` — Configure Groq backend.
- pub `with_ollama` function L143-146 — `(mut self, config: OpenAiConfig) -> Self` — Configure Ollama backend.
- pub `with_primary` function L149-152 — `(mut self, provider: Provider) -> Self` — Set the primary provider.
- pub `with_fallbacks` function L155-158 — `(mut self, providers: Vec<Provider>) -> Self` — Set fallback providers.
- pub `with_auto_fallback` function L161-164 — `(mut self, enabled: bool) -> Self` — Enable automatic fallback.
- pub `from_env` function L175-215 — `() -> Self` — Create configuration from environment variables.
- pub `LlmClient` struct L260-265 — `{ backends: HashMap<Provider, SharedBackend>, primary: Provider, fallbacks: Vec<...` — High-level LLM client with multi-provider support.
- pub `new` function L269-316 — `(config: LlmClientConfig) -> Result<Self>` — Create a new client from configuration.
- pub `from_env` function L319-321 — `() -> Result<Self>` — Create a client from environment variables.
- pub `anthropic` function L324-330 — `(config: AnthropicConfig) -> Result<Self>` — Create a client with just an Anthropic backend.
- pub `openai` function L333-339 — `(config: OpenAiConfig) -> Result<Self>` — Create a client with just an OpenAI backend.
- pub `anthropic_from_env` function L342-344 — `() -> Result<Self>` — Create a client from environment with Anthropic as primary.
- pub `openai_from_env` function L347-349 — `() -> Result<Self>` — Create a client from environment with OpenAI as primary.
- pub `primary` function L352-354 — `(&self) -> Provider` — Get the primary provider.
- pub `available_providers` function L357-359 — `(&self) -> Vec<Provider>` — Get all available providers.
- pub `has_provider` function L362-364 — `(&self, provider: Provider) -> bool` — Check if a provider is available.
- pub `get_backend` function L367-369 — `(&self, provider: Provider) -> Option<&SharedBackend>` — Get a backend by provider.
- pub `complete` function L372-374 — `(&self, request: CompletionRequest) -> Result<CompletionResponse>` — Execute a completion using the primary provider.
- pub `complete_with` function L377-387 — `( &self, provider: Provider, request: CompletionRequest, ) -> Result<CompletionR...` — Execute a completion using a specific provider.
- pub `complete_stream` function L436-438 — `(&self, request: CompletionRequest) -> Result<ResponseStream>` — Execute a streaming completion using the primary provider.
- pub `complete_stream_with` function L441-451 — `( &self, provider: Provider, request: CompletionRequest, ) -> Result<ResponseStr...` — Execute a streaming completion using a specific provider.
-  `Provider` type L52-81 — `= Provider` — ```
-  `Provider` type L83-87 — `= Provider` — ```
-  `fmt` function L84-86 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `LlmClientConfig` type L118-248 — `= LlmClientConfig` — ```
-  `determine_primary` function L218-237 — `(&self) -> Option<Provider>` — Determine the primary provider based on what's configured.
-  `is_provider_configured` function L240-247 — `(&self, provider: Provider) -> bool` — Check if a provider is configured.
-  `LlmClient` type L267-461 — `= LlmClient` — ```
-  `complete_with_fallback` function L390-433 — `( &self, provider: Provider, request: CompletionRequest, ) -> Result<CompletionR...` — Execute a completion with automatic fallback.
-  `should_fallback` function L454-460 — `(&self, error: &LlmError) -> bool` — Determine if we should attempt fallback for this error.
-  `LlmClient` type L465-485 — `impl LlmBackend for LlmClient` — ```
-  `complete` function L466-468 — `(&self, request: CompletionRequest) -> Result<CompletionResponse>` — ```
-  `complete_stream` function L470-472 — `(&self, request: CompletionRequest) -> Result<ResponseStream>` — ```
-  `name` function L474-476 — `(&self) -> &str` — ```
-  `supports_native_tools` function L478-484 — `(&self) -> bool` — ```
-  `tests` module L492-608 — `-` — ```
-  `test_provider_name` function L497-502 — `()` — ```
-  `test_provider_from_name` function L505-514 — `()` — ```
-  `test_provider_requires_api_key` function L517-522 — `()` — ```
-  `test_client_config_builder` function L525-534 — `()` — ```
-  `test_config_is_provider_configured` function L537-543 — `()` — ```
-  `test_config_determine_primary` function L546-562 — `()` — ```
-  `test_client_with_ollama` function L565-575 — `()` — ```
-  `test_client_no_providers_error` function L578-582 — `()` — ```
-  `test_client_available_providers` function L585-596 — `()` — ```
-  `test_complete_with_unavailable_provider` function L599-607 — `()` — ```

#### crates/arawn-llm/src/embeddings.rs

- pub `Embedder` interface L30-51 — `{ fn embed(), fn embed_batch(), fn dimensions(), fn name() }` — Trait for generating text embeddings.
- pub `SharedEmbedder` type L54 — `= Arc<dyn Embedder>` — A shared embedder that can be used across threads.
- pub `MockEmbedder` struct L65-67 — `{ dimensions: usize }` — A mock embedder for testing purposes.
- pub `new` function L71-73 — `(dimensions: usize) -> Self` — Create a new mock embedder with the specified dimensions.
- pub `default_dimensions` function L76-78 — `() -> Self` — Create a mock embedder with 384 dimensions (same as all-MiniLM-L6-v2).
- pub `OpenAiEmbedderConfig` struct L140-153 — `{ api_key: ApiKeyProvider, base_url: String, model: String, timeout: Duration, d...` — Configuration for OpenAI embeddings.
- pub `new` function L157-165 — `(api_key: impl Into<String>) -> Self` — Create a new config with the given API key.
- pub `from_env` function L168-175 — `() -> Result<Self>` — Create config from environment variable.
- pub `with_base_url` function L178-181 — `(mut self, url: impl Into<String>) -> Self` — Set a custom base URL.
- pub `with_model` function L184-187 — `(mut self, model: impl Into<String>) -> Self` — Set the model to use.
- pub `with_dimensions` function L190-193 — `(mut self, dimensions: usize) -> Self` — Override output dimensions.
- pub `OpenAiEmbedder` struct L197-201 — `{ client: Client, config: OpenAiEmbedderConfig, dimensions: usize }` — OpenAI embeddings API client.
- pub `new` function L205-228 — `(config: OpenAiEmbedderConfig) -> Result<Self>` — Create a new OpenAI embedder.
- pub `from_env` function L231-233 — `() -> Result<Self>` — Create from environment configuration.
- pub `local` module L321-570 — `-` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
- pub `LocalEmbedder` struct L339-344 — `{ session: Mutex<Session>, tokenizer: Tokenizer, dimensions: usize, name: String...` — Local embedder using ONNX Runtime.
- pub `load` function L353-395 — `( model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>, dimensions: us...` — Load a local embedder from model files.
- pub `EmbedderSpec` struct L581-600 — `{ provider: String, openai_api_key: Option<String>, openai_model: Option<String>...` — Configuration for building an embedder from application config.
- pub `build_embedder` function L609-689 — `(spec: &EmbedderSpec) -> Result<SharedEmbedder>` — Build a `SharedEmbedder` from a spec.
- pub `DEFAULT_EMBEDDING_MODEL_URL` variable L702-703 — `: &str` — Default HuggingFace model URL for all-MiniLM-L6-v2 ONNX model.
- pub `DEFAULT_EMBEDDING_TOKENIZER_URL` variable L705-706 — `: &str` — Default HuggingFace tokenizer URL for all-MiniLM-L6-v2.
- pub `DEFAULT_NER_MODEL_URL` variable L709-710 — `: &str` — Default HuggingFace model URL for GLiNER small v2.1 (span mode).
- pub `DEFAULT_NER_TOKENIZER_URL` variable L712-713 — `: &str` — Default HuggingFace tokenizer URL for GLiNER small v2.1.
- pub `default_ner_model_dir` function L769-771 — `() -> Option<std::path::PathBuf>` — Default directory for NER (GLiNER) model files.
- pub `ensure_ner_model_files` function L777-822 — `( model_url: Option<&str>, tokenizer_url: Option<&str>, ) -> Option<(std::path::...` — Download NER (GLiNER) model files if they don't exist.
- pub `download_file` function L825-862 — `(url: &str, path: &std::path::Path) -> Result<()>` — Download a file from URL to path.
- pub `cosine_similarity` function L869-883 — `(a: &[f32], b: &[f32]) -> f32` — Calculate cosine similarity between two embeddings.
- pub `euclidean_distance` function L886-896 — `(a: &[f32], b: &[f32]) -> f32` — Calculate Euclidean distance between two embeddings.
-  `embed_batch` function L38-44 — `(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>` — Generate embeddings for multiple texts in a batch.
-  `MockEmbedder` type L69-79 — `= MockEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `MockEmbedder` type L81-85 — `impl Default for MockEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `default` function L82-84 — `() -> Self` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `MockEmbedder` type L88-120 — `impl Embedder for MockEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `embed` function L89-111 — `(&self, text: &str) -> Result<Vec<f32>>` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `dimensions` function L113-115 — `(&self) -> usize` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `name` function L117-119 — `(&self) -> &str` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `simple_hash` function L123-129 — `(s: &str) -> u64` — Simple hash function for deterministic embedding generation.
-  `OpenAiEmbedderConfig` type L155-194 — `= OpenAiEmbedderConfig` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `OpenAiEmbedder` type L203-238 — `= OpenAiEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `embeddings_url` function L235-237 — `(&self) -> String` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `OpenAiEmbedder` type L241-297 — `impl Embedder for OpenAiEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `embed` function L242-248 — `(&self, text: &str) -> Result<Vec<f32>>` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `embed_batch` function L250-288 — `(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `dimensions` function L290-292 — `(&self) -> usize` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `name` function L294-296 — `(&self) -> &str` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `EmbeddingRequest` struct L300-303 — `{ model: String, input: Vec<String> }` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `EmbeddingResponse` struct L306-308 — `{ data: Vec<EmbeddingData> }` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `EmbeddingData` struct L311-314 — `{ index: usize, embedding: Vec<f32> }` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `LocalEmbedder` type L346-396 — `= LocalEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `LocalEmbedder` type L399-441 — `impl Embedder for LocalEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `embed` function L400-405 — `(&self, text: &str) -> Result<Vec<f32>>` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `embed_batch` function L407-432 — `(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `dimensions` function L434-436 — `(&self) -> usize` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `name` function L438-440 — `(&self) -> &str` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `LocalEmbedder` type L443-569 — `= LocalEmbedder` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `run_batch` function L448-568 — `(&self, encodings: &[tokenizers::Encoding]) -> Result<Vec<Vec<f32>>>` — Run ONNX inference on a batch of encodings.
-  `default_local_model_dir` function L693-695 — `() -> Option<std::path::PathBuf>` — Default directory for local embedding model files.
-  `ensure_model_files` function L720-766 — `( model_url: Option<&str>, tokenizer_url: Option<&str>, ) -> Option<std::path::P...` — Download embedding model files if they don't exist.
-  `tests` module L903-1028 — `-` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_mock_embedder` function L907-918 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_mock_embedder_deterministic` function L921-929 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_mock_embedder_different_texts` function L932-940 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_embed_batch` function L943-953 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_cosine_similarity` function L956-966 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_euclidean_distance` function L969-976 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_openai_embedder_config` function L979-983 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_openai_embedder_config_builder` function L986-994 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_openai_embedder_dimensions_from_model_lookup` function L997-1007 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_openai_embedder_dimensions_override` function L1010-1017 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API
-  `test_openai_embedder_dimensions_override_unknown_model` function L1020-1027 — `()` — - [`OpenAiEmbedder`]: Uses OpenAI's embeddings API

#### crates/arawn-llm/src/error.rs

- pub `Result` type L7 — `= std::result::Result<T, LlmError>` — Result type alias using the LLM error type.
- pub `RateLimitInfo` struct L15-22 — `{ message: String, retry_after: Option<Duration>, limit_type: Option<RateLimitTy...` — Information about a rate limit error.
- pub `RateLimitType` enum L26-35 — `TokensPerMinute | RequestsPerMinute | RequestsPerDay | Other` — Type of rate limit encountered.
- pub `new` function L39-45 — `(message: impl Into<String>) -> Self` — Create a new rate limit info with just a message.
- pub `with_retry_after` function L48-54 — `(message: impl Into<String>, retry_after: Duration) -> Self` — Create a rate limit info with a retry duration.
- pub `parse_groq` function L60-77 — `(message: &str) -> Self` — Parse rate limit info from a Groq error message.
- pub `parse_openai` function L80-88 — `(message: &str, retry_after_header: Option<&str>) -> Self` — Parse rate limit info from OpenAI-style headers and body.
- pub `ResponseValidationError` enum L149-203 — `MissingField | InvalidToolUse | InvalidTokenCount | MalformedContent | InvalidSt...` — Error type for LLM response validation failures.
- pub `missing_field` function L207-209 — `(field: &'static str) -> Self` — Create a missing field error.
- pub `invalid_tool_use` function L212-217 — `(id: impl Into<String>, reason: impl Into<String>) -> Self` — Create an invalid tool use error.
- pub `invalid_token_count` function L220-226 — `(field: &'static str, value: i64, constraint: &'static str) -> Self` — Create an invalid token count error.
- pub `malformed_content` function L229-234 — `(index: usize, reason: impl Into<String>) -> Self` — Create a malformed content error.
- pub `invalid_stop_reason` function L237-241 — `(reason: impl Into<String>) -> Self` — Create an invalid stop reason error.
- pub `invalid_stream_event` function L244-248 — `(reason: impl Into<String>) -> Self` — Create an invalid stream event error.
- pub `multiple` function L251-253 — `(errors: Vec<ResponseValidationError>) -> Self` — Create from multiple errors.
- pub `is_critical` function L256-261 — `(&self) -> bool` — Returns true if this is a critical error that should abort processing.
- pub `LlmError` enum L272-304 — `Backend | Network | Config | Serialization | InvalidRequest | RateLimit | Auth |...` — Error type for LLM operations.
- pub `rate_limit` function L311-313 — `(message: impl Into<String>) -> Self` — Create a rate limit error from a message string.
- pub `rate_limit_with_retry` function L316-318 — `(message: impl Into<String>, retry_after: Duration) -> Self` — Create a rate limit error with retry timing.
- pub `retry_after` function L321-326 — `(&self) -> Option<Duration>` — Get the retry-after duration if this is a rate limit error.
- pub `is_retryable` function L329-331 — `(&self) -> bool` — Returns true if this error is retryable.
- pub `is_tool_validation_error` function L336-345 — `(&self) -> bool` — Returns true if this is a tool validation error (LLM hallucinated a tool name).
- pub `invalid_tool_name` function L348-362 — `(&self) -> Option<&str>` — Extract the invalid tool name from a tool validation error, if present.
- pub `is_retryable` function L387-389 — `(error: &LlmError) -> bool` — Check if an error is retryable.
-  `RateLimitInfo` type L37-89 — `= RateLimitInfo` — Error types for the LLM crate.
-  `RateLimitInfo` type L91-99 — `= RateLimitInfo` — Error types for the LLM crate.
-  `fmt` function L92-98 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Error types for the LLM crate.
-  `parse_groq_retry_after` function L102-124 — `(message: &str) -> Option<Duration>` — Parse Groq's "Please try again in Xs" format.
-  `parse_retry_after_header` function L129-137 — `(value: &str) -> Option<Duration>` — Parse a Retry-After header value.
-  `ResponseValidationError` type L205-262 — `= ResponseValidationError` — Error types for the LLM crate.
-  `LlmError` type L264-268 — `= LlmError` — Error types for the LLM crate.
-  `from` function L265-267 — `(err: ResponseValidationError) -> Self` — Error types for the LLM crate.
-  `LlmError` type L306-363 — `= LlmError` — Error types for the LLM crate.
-  `LlmError` type L365-375 — `= LlmError` — Error types for the LLM crate.
-  `from` function L366-374 — `(err: reqwest::Error) -> Self` — Error types for the LLM crate.
-  `LlmError` type L377-381 — `= LlmError` — Error types for the LLM crate.
-  `from` function L378-380 — `(err: serde_json::Error) -> Self` — Error types for the LLM crate.
-  `tests` module L392-582 — `-` — Error types for the LLM crate.
-  `test_is_retryable` function L396-404 — `()` — Error types for the LLM crate.
-  `test_rate_limit_info_new` function L407-412 — `()` — Error types for the LLM crate.
-  `test_rate_limit_info_with_retry` function L415-419 — `()` — Error types for the LLM crate.
-  `test_parse_groq_retry_after` function L422-440 — `()` — Error types for the LLM crate.
-  `test_parse_retry_after_header` function L443-450 — `()` — Error types for the LLM crate.
-  `test_llm_error_retry_after` function L453-462 — `()` — Error types for the LLM crate.
-  `test_rate_limit_info_display` function L465-471 — `()` — Error types for the LLM crate.
-  `test_missing_field_error` function L478-483 — `()` — Error types for the LLM crate.
-  `test_invalid_tool_use_error` function L486-491 — `()` — Error types for the LLM crate.
-  `test_invalid_token_count_error` function L494-500 — `()` — Error types for the LLM crate.
-  `test_malformed_content_error` function L503-508 — `()` — Error types for the LLM crate.
-  `test_invalid_stop_reason_error` function L511-515 — `()` — Error types for the LLM crate.
-  `test_invalid_stream_event_error` function L518-522 — `()` — Error types for the LLM crate.
-  `test_multiple_errors` function L525-534 — `()` — Error types for the LLM crate.
-  `test_validation_error_into_llm_error` function L537-541 — `()` — Error types for the LLM crate.
-  `test_is_tool_validation_error` function L544-561 — `()` — Error types for the LLM crate.
-  `test_invalid_tool_name_extraction` function L564-581 — `()` — Error types for the LLM crate.

#### crates/arawn-llm/src/interaction_log.rs

- pub `InteractionRecord` struct L24-63 — `{ id: String, timestamp: String, duration_ms: u64, model: String, message_count:...` — A single LLM interaction (request + response pair).
- pub `ToolCallRecord` struct L67-70 — `{ tool_name: String, call_id: String }` — A tool call captured from a response.
- pub `RoutingMetadata` struct L74-82 — `{ profile: String, reason: String, confidence: Option<f64> }` — Routing decision metadata (filled in by the routing layer).
- pub `from_exchange` function L86-129 — `( request: &CompletionRequest, response: &CompletionResponse, duration_ms: u64, ...` — Build a record from a completed request/response exchange.
- pub `with_routing` function L132-135 — `(mut self, routing: RoutingMetadata) -> Self` — Attach routing metadata after construction.
- pub `InteractionLogConfig` struct L145-152 — `{ enabled: bool, path: Option<PathBuf>, retention_days: u32 }` — Configuration for interaction logging.
- pub `resolved_path` function L166-173 — `(&self) -> PathBuf` — Resolve the log directory, falling back to the XDG default.
- pub `InteractionLogger` struct L177-180 — `{ config: InteractionLogConfig, state: Mutex<WriterState> }` — Thread-safe JSONL writer with daily file rotation.
- pub `new` function L189-203 — `(config: InteractionLogConfig) -> std::io::Result<Self>` — Create a new logger.
- pub `log` function L206-239 — `(&self, record: &InteractionRecord) -> std::io::Result<()>` — Log an interaction record.
-  `InteractionRecord` type L84-136 — `= InteractionRecord` — session indexer, and future training pipelines.
-  `InteractionLogConfig` type L154-162 — `impl Default for InteractionLogConfig` — session indexer, and future training pipelines.
-  `default` function L155-161 — `() -> Self` — session indexer, and future training pipelines.
-  `InteractionLogConfig` type L164-174 — `= InteractionLogConfig` — session indexer, and future training pipelines.
-  `WriterState` struct L182-185 — `{ current_date: Option<NaiveDate>, writer: Option<BufWriter<File>> }` — session indexer, and future training pipelines.
-  `InteractionLogger` type L187-240 — `= InteractionLogger` — session indexer, and future training pipelines.
-  `cleanup_old_files` function L243-264 — `(dir: &Path, retention_days: u32) -> std::io::Result<()>` — Delete JSONL files older than `retention_days`.
-  `tests` module L271-404 — `-` — session indexer, and future training pipelines.
-  `sample_request` function L275-277 — `() -> CompletionRequest` — session indexer, and future training pipelines.
-  `sample_response` function L279-305 — `() -> CompletionResponse` — session indexer, and future training pipelines.
-  `test_record_from_exchange` function L308-321 — `()` — session indexer, and future training pipelines.
-  `test_record_serialization_roundtrip` function L324-344 — `()` — session indexer, and future training pipelines.
-  `test_jsonl_format` function L347-357 — `()` — session indexer, and future training pipelines.
-  `test_logger_disabled_is_noop` function L360-373 — `()` — session indexer, and future training pipelines.
-  `test_logger_writes_jsonl` function L376-403 — `()` — session indexer, and future training pipelines.

#### crates/arawn-llm/src/lib.rs

- pub `api_key` module L25 — `-` — This crate provides a unified interface for interacting with various LLM providers
- pub `backend` module L26 — `-` — ```
- pub `client` module L27 — `-` — ```
- pub `embeddings` module L28 — `-` — ```
- pub `error` module L29 — `-` — ```
- pub `interaction_log` module L30 — `-` — ```
- pub `types` module L31 — `-` — ```
- pub `anthropic` module L34 — `-` — ```
- pub `openai` module L35 — `-` — ```

#### crates/arawn-llm/src/openai.rs

- pub `OpenAiConfig` struct L54-75 — `{ api_key: ApiKeyProvider, base_url: String, model: Option<String>, timeout: Dur...` — Configuration for the OpenAI-compatible backend.
- pub `openai` function L79-89 — `(api_key: impl Into<String>) -> Self` — Create a new config for OpenAI.
- pub `groq` function L92-102 — `(api_key: impl Into<String>) -> Self` — Create a new config for Groq.
- pub `ollama` function L105-115 — `() -> Self` — Create a new config for Ollama (local).
- pub `openai_from_env` function L118-123 — `() -> Result<Self>` — Create config from environment for OpenAI.
- pub `groq_from_env` function L126-131 — `() -> Result<Self>` — Create config from environment for Groq.
- pub `with_base_url` function L134-137 — `(mut self, url: impl Into<String>) -> Self` — Set a custom base URL.
- pub `with_model` function L140-143 — `(mut self, model: impl Into<String>) -> Self` — Set the default model.
- pub `with_name` function L146-149 — `(mut self, name: impl Into<String>) -> Self` — Set the backend name.
- pub `with_timeout` function L152-155 — `(mut self, timeout: Duration) -> Self` — Set request timeout.
- pub `with_max_retries` function L158-161 — `(mut self, retries: u32) -> Self` — Set max retries.
- pub `with_retry_backoff` function L164-167 — `(mut self, backoff: Duration) -> Self` — Set retry backoff.
- pub `OpenAiBackend` struct L175-178 — `{ client: Client, config: OpenAiConfig }` — OpenAI-compatible API backend.
- pub `new` function L182-189 — `(config: OpenAiConfig) -> Result<Self>` — Create a new OpenAI-compatible backend with the given configuration.
- pub `openai_from_env` function L192-194 — `() -> Result<Self>` — Create an OpenAI backend from environment.
- pub `groq_from_env` function L197-199 — `() -> Result<Self>` — Create a Groq backend from environment.
- pub `ollama` function L202-204 — `() -> Result<Self>` — Create an Ollama backend with default local settings.
- pub `create_shared_backend` function L496-498 — `(config: OpenAiConfig) -> Result<Arc<dyn LlmBackend>>` — Create a shared OpenAI-compatible backend.
-  `DEFAULT_OPENAI_BASE` variable L22 — `: &str` — Default OpenAI API base URL.
-  `DEFAULT_TIMEOUT_SECS` variable L25 — `: u64` — Default timeout for requests.
-  `DEFAULT_MAX_RETRIES` variable L28 — `: u32` — Default maximum retries for transient errors.
-  `DEFAULT_RETRY_BACKOFF_MS` variable L31 — `: u64` — Default initial backoff between retries.
-  `OpenAiConfig` type L77-168 — `= OpenAiConfig` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiBackend` type L180-432 — `= OpenAiBackend` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `completions_url` function L207-209 — `(&self) -> String` — Build the chat completions endpoint URL.
-  `add_headers` function L212-220 — `(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder` — Add authentication headers to a request.
-  `to_openai_request` function L223-377 — `(&self, request: &CompletionRequest) -> OpenAiChatRequest` — Convert our CompletionRequest to OpenAI-compatible format.
-  `handle_response` function L380-390 — `(response: Response) -> Result<CompletionResponse>` — Handle a successful response.
-  `handle_error_response` function L393-431 — `(response: Response) -> LlmError` — Handle an error response.
-  `OpenAiBackend` type L435-493 — `impl LlmBackend for OpenAiBackend` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `complete` function L436-465 — `(&self, request: CompletionRequest) -> Result<CompletionResponse>` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `complete_stream` function L467-484 — `(&self, request: CompletionRequest) -> Result<ResponseStream>` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `name` function L486-488 — `(&self) -> &str` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `supports_native_tools` function L490-492 — `(&self) -> bool` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiChatRequest` struct L505-520 — `{ model: String, messages: Vec<OpenAiMessage>, max_tokens: Option<u32>, temperat...` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiMessage` struct L523-531 — `{ role: String, content: Option<OpenAiContent>, tool_calls: Option<Vec<OpenAiToo...` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiContent` enum L535-537 — `Text` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiTool` struct L540-544 — `{ tool_type: String, function: OpenAiFunction }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiFunction` struct L547-552 — `{ name: String, description: Option<String>, parameters: serde_json::Value }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiToolCall` struct L555-560 — `{ id: String, call_type: String, function: OpenAiFunctionCall }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiFunctionCall` struct L563-566 — `{ name: String, arguments: String }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiChatResponse` struct L569-574 — `{ id: String, choices: Vec<OpenAiChoice>, model: String, usage: Option<OpenAiUsa...` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `CompletionResponse` type L576-639 — `= CompletionResponse` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `from` function L577-638 — `(resp: OpenAiChatResponse) -> Self` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiChoice` struct L642-645 — `{ message: OpenAiResponseMessage, finish_reason: Option<String> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiResponseMessage` struct L648-651 — `{ content: Option<String>, tool_calls: Option<Vec<OpenAiToolCall>> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiUsage` struct L654-657 — `{ prompt_tokens: u32, completion_tokens: u32 }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiErrorResponse` struct L660-662 — `{ error: OpenAiError }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiError` struct L665-667 — `{ message: String }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `parse_openai_sse_stream` function L673-794 — `( byte_stream: impl Stream<Item = reqwest::Result<Bytes>> + Send + 'static, ) ->...` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiSseState` struct L796-803 — `{ byte_stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>, buffe...` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiStreamChunk` struct L806-810 — `{ id: String, model: String, choices: Vec<OpenAiStreamChoice> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiStreamChoice` struct L813-816 — `{ delta: Option<OpenAiStreamDelta>, finish_reason: Option<String> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiStreamDelta` struct L819-822 — `{ content: Option<String>, tool_calls: Option<Vec<OpenAiStreamToolCall>> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiStreamToolCall` struct L825-828 — `{ index: Option<usize>, function: Option<OpenAiStreamFunction> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `OpenAiStreamFunction` struct L831-833 — `{ arguments: Option<String> }` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `tests` module L840-1002 — `-` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_openai_config` function L845-850 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_groq_config` function L853-859 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_ollama_config` function L862-868 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_config_builder` function L871-882 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_completions_url` function L885-892 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_groq_completions_url` function L895-902 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_ollama_completions_url` function L905-912 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_backend_name` function L915-919 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_supports_native_tools` function L922-926 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_openai_response_conversion` function L929-952 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_openai_response_with_tool_calls` function L955-986 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).
-  `test_to_openai_request` function L989-1001 — `()` — or any OpenAI-compatible service (Groq, Ollama, local LLMs, etc.).

#### crates/arawn-llm/src/types.rs

- pub `SystemPrompt` enum L19-24 — `Text | Blocks` — System prompt - can be a string or array of text blocks.
- pub `SystemBlock` struct L28-37 — `{ text: String, block_type: String, cache_control: Option<CacheControl> }` — A text block in a system prompt.
- pub `text` function L41-43 — `(content: impl Into<String>) -> Self` — Create a simple text system prompt.
- pub `to_text` function L46-55 — `(&self) -> String` — Get the text content of the system prompt.
- pub `CompletionRequest` struct L75-120 — `{ model: String, messages: Vec<Message>, max_tokens: u32, system: Option<SystemP...` — A completion request to an LLM provider.
- pub `new` function L124-139 — `(model: impl Into<String>, messages: Vec<Message>, max_tokens: u32) -> Self` — Create a new completion request with the given model and messages.
- pub `with_system` function L142-145 — `(mut self, system: impl Into<String>) -> Self` — Set the system prompt.
- pub `with_tools` function L148-151 — `(mut self, tools: Vec<ToolDefinition>) -> Self` — Add tools to the request.
- pub `with_tool_choice` function L154-157 — `(mut self, choice: ToolChoice) -> Self` — Set tool choice.
- pub `with_streaming` function L160-163 — `(mut self) -> Self` — Enable streaming.
- pub `with_temperature` function L166-169 — `(mut self, temperature: f32) -> Self` — Set temperature.
- pub `Message` struct L188-194 — `{ role: Role, content: Content }` — A message in the conversation.
- pub `user` function L198-203 — `(text: impl Into<String>) -> Self` — Create a user message with text content.
- pub `assistant` function L206-211 — `(text: impl Into<String>) -> Self` — Create an assistant message with text content.
- pub `assistant_blocks` function L214-219 — `(blocks: Vec<ContentBlock>) -> Self` — Create an assistant message with content blocks.
- pub `tool_results` function L222-227 — `(results: Vec<ToolResultBlock>) -> Self` — Create a user message with tool results.
- pub `Role` enum L233-236 — `User | Assistant` — The role of a message author.
- pub `Content` enum L241-246 — `Text | Blocks` — Message content - either a simple string or structured blocks.
- pub `as_text` function L250-255 — `(&self) -> Option<&str>` — Get the text content if this is simple text.
- pub `blocks` function L258-266 — `(&self) -> Vec<ContentBlock>` — Get the content blocks.
- pub `to_text` function L269-281 — `(&self) -> String` — Extract all text from the content.
- pub `CacheControl` enum L291-294 — `Ephemeral` — Cache control for prompt caching.
- pub `ContentBlock` enum L299-334 — `Text | ToolUse | ToolResult` — A content block in a message.
- pub `text` function L338-343 — `(content: impl Into<String>) -> Self` — Create a text content block.
- pub `tool_use` function L346-357 — `( id: impl Into<String>, name: impl Into<String>, input: serde_json::Value, ) ->...` — Create a tool use content block.
- pub `tool_result_success` function L360-367 — `(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self` — Create a successful tool result block.
- pub `tool_result_error` function L370-377 — `(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self` — Create an error tool result block.
- pub `ToolResultContent` enum L383-386 — `Text | Blocks` — Tool result content - can be a string or array of content blocks.
- pub `ToolUseBlock` struct L394-401 — `{ id: String, name: String, input: serde_json::Value }` — Convenience struct for creating tool use blocks.
- pub `ToolResultBlock` struct L416-425 — `{ tool_use_id: String, content: Option<ToolResultContent>, is_error: bool }` — Convenience struct for creating tool result blocks.
- pub `success` function L429-435 — `(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self` — Create a successful tool result.
- pub `error` function L438-444 — `(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self` — Create an error tool result.
- pub `ToolDefinition` struct L482-491 — `{ name: String, description: String, input_schema: serde_json::Value }` — Definition of a tool available to the model.
- pub `new` function L495-505 — `( name: impl Into<String>, description: impl Into<String>, input_schema: serde_j...` — Create a new tool definition.
- pub `ToolChoice` enum L511-520 — `Auto | Any | Tool | None` — How the model should choose which tool to use.
- pub `CompletionResponse` struct L528-550 — `{ id: String, response_type: String, role: Role, content: Vec<ContentBlock>, mod...` — A completion response from the model.
- pub `new` function L558-574 — `( id: impl Into<String>, model: impl Into<String>, content: Vec<ContentBlock>, s...` — Create a new completion response.
- pub `tool_uses` function L577-591 — `(&self) -> Vec<ToolUseBlock>` — Get all tool use blocks from the response.
- pub `text` function L594-603 — `(&self) -> String` — Get the text content from the response.
- pub `has_tool_use` function L606-610 — `(&self) -> bool` — Check if the response contains tool use requests.
- pub `validate` function L633-671 — `(&self) -> Result<(), ResponseValidationError>` — Validate the response structure.
- pub `validated` function L750-753 — `(self) -> Result<Self, ResponseValidationError>` — Validate and return the response, or return an error.
- pub `StopReason` enum L771-780 — `EndTurn | ToolUse | MaxTokens | StopSequence` — Why the model stopped generating.
- pub `Usage` struct L793-804 — `{ input_tokens: u32, output_tokens: u32, cache_creation_input_tokens: u32, cache...` — Token usage statistics.
- pub `new` function L808-815 — `(input_tokens: u32, output_tokens: u32) -> Self` — Create new usage statistics.
- pub `total` function L818-820 — `(&self) -> u32` — Total tokens used.
-  `SystemPrompt` type L39-56 — `= SystemPrompt` — while being provider-agnostic for use with other backends.
-  `CompletionRequest` type L122-170 — `= CompletionRequest` — while being provider-agnostic for use with other backends.
-  `Message` type L196-228 — `= Message` — while being provider-agnostic for use with other backends.
-  `Content` type L248-282 — `= Content` — while being provider-agnostic for use with other backends.
-  `ContentBlock` type L336-378 — `= ContentBlock` — while being provider-agnostic for use with other backends.
-  `ContentBlock` type L403-412 — `= ContentBlock` — while being provider-agnostic for use with other backends.
-  `from` function L404-411 — `(block: ToolUseBlock) -> Self` — while being provider-agnostic for use with other backends.
-  `ToolResultBlock` type L427-445 — `= ToolResultBlock` — while being provider-agnostic for use with other backends.
-  `ContentBlock` type L447-456 — `= ContentBlock` — while being provider-agnostic for use with other backends.
-  `from` function L448-455 — `(block: ToolResultBlock) -> Self` — while being provider-agnostic for use with other backends.
-  `ToolDefinition` type L493-506 — `= ToolDefinition` — while being provider-agnostic for use with other backends.
-  `default_message_type` function L552-554 — `() -> String` — while being provider-agnostic for use with other backends.
-  `CompletionResponse` type L556-754 — `= CompletionResponse` — while being provider-agnostic for use with other backends.
-  `validate_content_block` function L674-745 — `( &self, block: &ContentBlock, index: usize, seen_tool_ids: &mut HashSet<String>...` — Validate a single content block.
-  `json_type_name` function L757-766 — `(value: &serde_json::Value) -> &'static str` — Get a human-readable name for a JSON value type.
-  `Usage` type L806-821 — `= Usage` — while being provider-agnostic for use with other backends.
-  `tests` module L828-1164 — `-` — while being provider-agnostic for use with other backends.
-  `test_message_user` function L832-836 — `()` — while being provider-agnostic for use with other backends.
-  `test_message_assistant` function L839-843 — `()` — while being provider-agnostic for use with other backends.
-  `test_completion_request_builder` function L846-861 — `()` — while being provider-agnostic for use with other backends.
-  `test_completion_response_tool_uses` function L864-890 — `()` — while being provider-agnostic for use with other backends.
-  `test_tool_result_block` function L893-903 — `()` — while being provider-agnostic for use with other backends.
-  `test_serialize_deserialize_request` function L906-918 — `()` — while being provider-agnostic for use with other backends.
-  `test_content_blocks` function L921-936 — `()` — while being provider-agnostic for use with other backends.
-  `test_usage` function L939-942 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_valid_response` function L949-959 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_response_with_tool_use` function L962-975 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_empty_id` function L978-991 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_empty_model` function L994-1007 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_tool_use_empty_id` function L1010-1026 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_tool_use_empty_name` function L1029-1045 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_tool_use_invalid_name_chars` function L1048-1064 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_tool_use_duplicate_ids` function L1067-1081 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_tool_use_input_not_object` function L1084-1100 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_stop_reason_mismatch` function L1103-1116 — `()` — while being provider-agnostic for use with other backends.
-  `test_validate_multiple_errors` function L1119-1139 — `()` — while being provider-agnostic for use with other backends.
-  `test_validated_convenience` function L1142-1153 — `()` — while being provider-agnostic for use with other backends.
-  `test_json_type_name` function L1156-1163 — `()` — while being provider-agnostic for use with other backends.

### crates/arawn-mcp/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-mcp/src/client.rs

- pub `TransportType` enum L18-24 — `Stdio | Http` — Transport type for MCP server connections.
- pub `McpServerConfig` struct L28-47 — `{ name: String, transport: TransportType, command: String, url: Option<String>, ...` — Configuration for an MCP server connection.
- pub `new` function L51-63 — `(name: impl Into<String>, command: impl Into<String>) -> Self` — Create a new server config for stdio transport.
- pub `http` function L66-78 — `(name: impl Into<String>, url: impl Into<String>) -> Self` — Create a new server config for HTTP transport.
- pub `with_args` function L81-84 — `(mut self, args: Vec<String>) -> Self` — Add arguments.
- pub `with_arg` function L87-90 — `(mut self, arg: impl Into<String>) -> Self` — Add an argument.
- pub `with_env` function L93-96 — `(mut self, env: Vec<(String, String)>) -> Self` — Add environment variables.
- pub `with_env_var` function L99-102 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add an environment variable.
- pub `with_header` function L105-108 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add an HTTP header (for HTTP transport).
- pub `with_timeout` function L111-114 — `(mut self, timeout: Duration) -> Self` — Set request timeout (for HTTP transport).
- pub `with_retries` function L117-120 — `(mut self, retries: u32) -> Self` — Set number of retries (for HTTP transport).
- pub `is_http` function L123-125 — `(&self) -> bool` — Check if this is an HTTP transport config.
- pub `is_stdio` function L128-130 — `(&self) -> bool` — Check if this is a stdio transport config.
- pub `McpClient` struct L134-145 — `{ config: McpServerConfig, transport: Mutex<McpTransport>, server_info: Option<S...` — An MCP client connected to a single MCP server.
- pub `connect` function L152-157 — `(config: McpServerConfig) -> Result<Self>` — Connect to an MCP server using the configured transport.
- pub `connect_stdio` function L163-185 — `(config: McpServerConfig) -> Result<Self>` — Connect to an MCP server using stdio transport.
- pub `connect_http` function L191-224 — `(config: McpServerConfig) -> Result<Self>` — Connect to an MCP server using HTTP transport.
- pub `name` function L227-229 — `(&self) -> &str` — Get the server name.
- pub `server_info` function L232-234 — `(&self) -> Option<&ServerInfo>` — Get the server info (after initialization).
- pub `is_initialized` function L237-239 — `(&self) -> bool` — Check if the client has been initialized.
- pub `is_http` function L242-244 — `(&self) -> bool` — Check if the client is using HTTP transport.
- pub `is_stdio` function L247-249 — `(&self) -> bool` — Check if the client is using stdio transport.
- pub `initialize` function L288-312 — `(&mut self) -> Result<&ServerInfo>` — Initialize the connection with the MCP server.
- pub `list_tools` function L315-330 — `(&self) -> Result<Vec<ToolInfo>>` — List available tools from the server.
- pub `call_tool` function L337-365 — `(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult>` — Call a tool on the server.
- pub `shutdown` function L368-377 — `(&mut self) -> Result<()>` — Shutdown the connection gracefully.
- pub `is_connected` function L380-386 — `(&self) -> bool` — Check if the connection is still active.
-  `McpServerConfig` type L49-131 — `= McpServerConfig` — MCP client for communicating with MCP servers.
-  `McpClient` type L147-387 — `= McpClient` — MCP client for communicating with MCP servers.
-  `next_request_id` function L252-254 — `(&self) -> u64` — Get the next request ID.
-  `send_request` function L257-270 — `(&self, method: &str, params: Option<Value>) -> Result<Value>` — Send a request and get the response.
-  `send_notification` function L273-282 — `(&self, method: &str, params: Option<Value>) -> Result<()>` — Send a notification (no response expected).
-  `McpClient` type L389-393 — `impl Drop for McpClient` — MCP client for communicating with MCP servers.
-  `drop` function L390-392 — `(&mut self)` — MCP client for communicating with MCP servers.
-  `tests` module L396-481 — `-` — MCP client for communicating with MCP servers.
-  `test_server_config_builder` function L400-412 — `()` — MCP client for communicating with MCP servers.
-  `test_http_server_config_builder` function L415-431 — `()` — MCP client for communicating with MCP servers.
-  `test_connect_nonexistent_server` function L434-438 — `()` — MCP client for communicating with MCP servers.
-  `test_connect_http_no_url` function L441-449 — `()` — MCP client for communicating with MCP servers.
-  `test_connect_http_valid` function L452-456 — `()` — MCP client for communicating with MCP servers.
-  `test_connect_auto_select_transport` function L459-471 — `()` — MCP client for communicating with MCP servers.
-  `test_request_id_increments` function L474-480 — `()` — MCP client for communicating with MCP servers.

#### crates/arawn-mcp/src/error.rs

- pub `Result` type L6 — `= std::result::Result<T, McpError>` — Result type for MCP operations.
- pub `McpError` enum L10-57 — `SpawnFailed | Transport | Protocol | Json | Io | ServerError | ToolError | NotIn...` — Error type for MCP operations.
- pub `spawn_failed` function L61-63 — `(msg: impl Into<String>) -> Self` — Create a spawn failed error.
- pub `transport` function L66-68 — `(msg: impl Into<String>) -> Self` — Create a transport error.
- pub `protocol` function L71-73 — `(msg: impl Into<String>) -> Self` — Create a protocol error.
- pub `server_error` function L76-86 — `( code: i64, message: impl Into<String>, data: Option<serde_json::Value>, ) -> S...` — Create a server error from an error response.
- pub `tool_error` function L89-91 — `(msg: impl Into<String>) -> Self` — Create a tool error.
-  `McpError` type L59-92 — `= McpError` — Error types for MCP operations.
-  `tests` module L95-122 — `-` — Error types for MCP operations.
-  `test_error_display` function L99-107 — `()` — Error types for MCP operations.
-  `test_json_error_conversion` function L110-114 — `()` — Error types for MCP operations.
-  `test_io_error_conversion` function L117-121 — `()` — Error types for MCP operations.

#### crates/arawn-mcp/src/lib.rs

- pub `client` module L65 — `-` — This crate provides a client implementation for the Model Context Protocol,
- pub `error` module L66 — `-` — 4.
- pub `manager` module L67 — `-` — 4.
- pub `protocol` module L68 — `-` — 4.
- pub `transport` module L69 — `-` — 4.

#### crates/arawn-mcp/src/manager.rs

- pub `McpManager` struct L44-49 — `{ configs: HashMap<String, McpServerConfig>, clients: HashMap<String, Arc<McpCli...` — Manager for multiple MCP server connections.
- pub `new` function L53-58 — `() -> Self` — Create a new empty MCP manager.
- pub `with_configs` function L61-67 — `(configs: Vec<McpServerConfig>) -> Self` — Create a manager with the given server configurations.
- pub `add_server` function L73-77 — `(&mut self, config: McpServerConfig)` — Add a server configuration.
- pub `remove_server` function L83-98 — `(&mut self, name: &str) -> bool` — Remove a server by name.
- pub `server_names` function L101-103 — `(&self) -> Vec<&str>` — Get the names of all configured servers.
- pub `connected_server_names` function L106-108 — `(&self) -> Vec<&str>` — Get the names of all connected servers.
- pub `has_server` function L111-113 — `(&self, name: &str) -> bool` — Check if a server is configured.
- pub `is_connected` function L116-118 — `(&self, name: &str) -> bool` — Check if a server is connected.
- pub `get_client` function L121-123 — `(&self, name: &str) -> Option<Arc<McpClient>>` — Get a connected client by name.
- pub `connect_all` function L134-162 — `(&mut self) -> Result<usize>` — Connect to all configured servers.
- pub `connect_server_by_name` function L174-189 — `(&mut self, name: &str) -> Result<()>` — Connect a single server by name.
- pub `list_all_tools` function L194-210 — `(&self) -> Result<HashMap<String, Vec<ToolInfo>>>` — List all tools from all connected servers.
- pub `all_tools_flat` function L215-226 — `(&self) -> Result<Vec<(String, ToolInfo)>>` — Get a flat list of all tools with their server names.
- pub `tool_count` function L229-232 — `(&self) -> Result<usize>` — Get the total number of tools across all servers.
- pub `clients` function L235-237 — `(&self) -> impl Iterator<Item = (&String, &Arc<McpClient>)>` — Get all connected clients.
- pub `shutdown_all` function L243-253 — `(&mut self) -> Result<()>` — Shutdown all connected servers.
- pub `shutdown_server` function L258-266 — `(&mut self, name: &str) -> bool` — Shutdown a specific server by name.
- pub `config_count` function L269-271 — `(&self) -> usize` — Get the number of configured servers.
- pub `connected_count` function L274-276 — `(&self) -> usize` — Get the number of connected servers.
- pub `has_connections` function L279-281 — `(&self) -> bool` — Check if any servers are connected.
-  `McpManager` type L51-282 — `= McpManager` — ```
-  `connect_server` function L165-169 — `(&self, config: McpServerConfig) -> Result<McpClient>` — Connect to a single server.
-  `McpManager` type L284-294 — `impl Drop for McpManager` — ```
-  `drop` function L285-293 — `(&mut self)` — ```
-  `McpManager` type L296-303 — `= McpManager` — ```
-  `fmt` function L297-302 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `tests` module L306-404 — `-` — ```
-  `test_new_manager_empty` function L310-315 — `()` — ```
-  `test_with_configs` function L318-328 — `()` — ```
-  `test_add_server` function L331-336 — `()` — ```
-  `test_remove_server` function L339-351 — `()` — ```
-  `test_server_names` function L354-363 — `()` — ```
-  `test_connect_all_no_servers` function L366-370 — `()` — ```
-  `test_connect_all_invalid_command` function L373-381 — `()` — ```
-  `test_debug_format` function L384-390 — `()` — ```
-  `test_shutdown_server_not_connected` function L393-403 — `()` — ```

#### crates/arawn-mcp/src/protocol.rs

- pub `JSONRPC_VERSION` variable L9 — `: &str` — JSON-RPC version string.
- pub `MCP_PROTOCOL_VERSION` variable L12 — `: &str` — MCP protocol version.
- pub `JsonRpcRequest` struct L32-42 — `{ jsonrpc: String, id: u64, method: String, params: Option<Value> }` — A JSON-RPC request.
- pub `new` function L46-53 — `(id: u64, method: impl Into<String>, params: Option<Value>) -> Self` — Create a new JSON-RPC request.
- pub `JsonRpcNotification` struct L58-66 — `{ jsonrpc: String, method: String, params: Option<Value> }` — A JSON-RPC notification (no id, no response expected).
- pub `new` function L70-76 — `(method: impl Into<String>, params: Option<Value>) -> Self` — Create a new notification.
- pub `JsonRpcResponse` struct L81-92 — `{ jsonrpc: String, id: u64, result: Option<Value>, error: Option<JsonRpcError> }` — A JSON-RPC response.
- pub `is_error` function L96-98 — `(&self) -> bool` — Check if this is an error response.
- pub `into_result` function L101-107 — `(self) -> Result<Value, JsonRpcError>` — Get the result, or return an error if this is an error response.
- pub `JsonRpcError` struct L112-120 — `{ code: i64, message: String, data: Option<Value> }` — A JSON-RPC error object.
- pub `PARSE_ERROR` variable L125 — `: i64` — Parse error - Invalid JSON.
- pub `INVALID_REQUEST` variable L127 — `: i64` — Invalid Request - Not a valid Request object.
- pub `METHOD_NOT_FOUND` variable L129 — `: i64` — Method not found.
- pub `INVALID_PARAMS` variable L131 — `: i64` — Invalid params.
- pub `INTERNAL_ERROR` variable L133 — `: i64` — Internal error.
- pub `ClientCapabilities` struct L142-149 — `{ experimental: Option<Value>, sampling: Option<Value> }` — Client capabilities sent during initialization.
- pub `ClientInfo` struct L153-158 — `{ name: String, version: String }` — Client info sent during initialization.
- pub `InitializeParams` struct L172-179 — `{ protocol_version: String, capabilities: ClientCapabilities, client_info: Clien...` — Parameters for the initialize request.
- pub `ServerCapabilities` struct L193-209 — `{ tools: Option<ToolsCapability>, resources: Option<Value>, prompts: Option<Valu...` — Server capabilities returned during initialization.
- pub `ToolsCapability` struct L214-218 — `{ list_changed: Option<bool> }` — Tools capability details.
- pub `ServerInfo` struct L222-227 — `{ name: String, version: String }` — Server info returned during initialization.
- pub `InitializeResult` struct L232-239 — `{ protocol_version: String, capabilities: ServerCapabilities, server_info: Serve...` — Result of the initialize request.
- pub `ToolInfo` struct L256-265 — `{ name: String, description: Option<String>, input_schema: Option<Value> }` — A tool definition from the server.
- pub `ListToolsResult` struct L269-272 — `{ tools: Vec<ToolInfo> }` — Result of the tools/list request.
- pub `CallToolParams` struct L276-282 — `{ name: String, arguments: Option<Value> }` — Parameters for the tools/call request.
- pub `ToolContent` enum L287-312 — `Text | Image | Resource` — Content item in a tool result.
- pub `CallToolResult` struct L330-336 — `{ content: Vec<ToolContent>, is_error: Option<bool> }` — Result of the tools/call request.
- pub `text` function L340-350 — `(&self) -> Option<String>` — Get the text content from the result.
- pub `is_error` function L353-355 — `(&self) -> bool` — Check if the tool call was an error.
-  `JsonRpcRequest` type L44-54 — `= JsonRpcRequest` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `JsonRpcNotification` type L68-77 — `= JsonRpcNotification` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `JsonRpcResponse` type L94-108 — `= JsonRpcResponse` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `JsonRpcError` type L123-134 — `= JsonRpcError` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `ClientInfo` type L160-167 — `impl Default for ClientInfo` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `default` function L161-166 — `() -> Self` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `InitializeParams` type L181-189 — `impl Default for InitializeParams` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `default` function L182-188 — `() -> Self` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `CallToolResult` type L338-356 — `= CallToolResult` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `tests` module L359-439 — `-` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_request_serialization` function L363-369 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_response_deserialization` function L372-378 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_error_response` function L381-388 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_initialize_params` function L391-396 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_tool_info_deserialization` function L399-415 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_tool_content_text` function L418-425 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.
-  `test_call_tool_result` function L428-438 — `()` — MCP uses JSON-RPC 2.0 with Content-Length framing for stdio transport.

#### crates/arawn-mcp/src/transport.rs

- pub `HttpTransportConfig` struct L16-25 — `{ url: String, timeout: Duration, retries: u32, headers: Vec<(String, String)> }` — Configuration for HTTP transport.
- pub `new` function L40-45 — `(url: impl Into<String>) -> Self` — Create a new HTTP transport config with the given URL.
- pub `with_timeout` function L48-51 — `(mut self, timeout: Duration) -> Self` — Set the request timeout.
- pub `with_retries` function L54-57 — `(mut self, retries: u32) -> Self` — Set the number of retries.
- pub `with_header` function L60-63 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add a header.
- pub `McpTransport` enum L67-84 — `Stdio | Http` — Transport for communicating with an MCP server.
- pub `connect_http` function L91-119 — `(config: HttpTransportConfig) -> Result<Self>` — Create a new HTTP transport.
- pub `spawn_stdio` function L127-164 — `( command: &str, args: &[String], env: Option<&[(String, String)]>, ) -> Result<...` — Spawn a new stdio transport.
- pub `send_request` function L167-180 — `(&mut self, request: &JsonRpcRequest) -> Result<JsonRpcResponse>` — Send a JSON-RPC request and wait for the response.
- pub `send_notification` function L183-202 — `(&mut self, notification: &JsonRpcNotification) -> Result<()>` — Send a JSON-RPC notification (no response expected).
- pub `shutdown` function L352-367 — `(&mut self) -> Result<()>` — Shutdown the transport gracefully.
- pub `is_connected` function L370-381 — `(&mut self) -> bool` — Check if the transport is still connected.
- pub `is_http` function L384-386 — `(&self) -> bool` — Check if this is an HTTP transport.
- pub `is_stdio` function L389-391 — `(&self) -> bool` — Check if this is a stdio transport.
-  `HttpTransportConfig` type L27-36 — `impl Default for HttpTransportConfig` — or HTTP POST for remote servers.
-  `default` function L28-35 — `() -> Self` — or HTTP POST for remote servers.
-  `HttpTransportConfig` type L38-64 — `= HttpTransportConfig` — or HTTP POST for remote servers.
-  `McpTransport` type L86-392 — `= McpTransport` — or HTTP POST for remote servers.
-  `send_request_http_impl` function L205-266 — `( client: &reqwest::blocking::Client, config: &HttpTransportConfig, request: &Js...` — Send a JSON-RPC request over HTTP and get the response.
-  `send_message_stdio` function L269-293 — `(&mut self, message: &serde_json::Value) -> Result<()>` — Send a JSON message with Content-Length framing (stdio only).
-  `receive_response_stdio` function L296-349 — `(&mut self) -> Result<JsonRpcResponse>` — Receive a JSON-RPC response with Content-Length framing (stdio only).
-  `McpTransport` type L394-398 — `impl Drop for McpTransport` — or HTTP POST for remote servers.
-  `drop` function L395-397 — `(&mut self)` — or HTTP POST for remote servers.
-  `tests` module L401-491 — `-` — or HTTP POST for remote servers.
-  `test_spawn_nonexistent_command` function L405-411 — `()` — or HTTP POST for remote servers.
-  `test_spawn_with_args` function L414-427 — `()` — or HTTP POST for remote servers.
-  `test_http_transport_config` function L430-444 — `()` — or HTTP POST for remote servers.
-  `test_http_transport_config_default` function L447-452 — `()` — or HTTP POST for remote servers.
-  `test_http_transport_creation` function L455-463 — `()` — or HTTP POST for remote servers.
-  `test_http_transport_invalid_url` function L466-475 — `()` — or HTTP POST for remote servers.
-  `test_http_transport_is_always_connected` function L478-490 — `()` — or HTTP POST for remote servers.

### crates/arawn-mcp/tests

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-mcp/tests/integration.rs

-  `mock_server_path` function L12-25 — `() -> PathBuf` — Get the path to the mock MCP server binary.
-  `mock_server_exists` function L28-30 — `() -> bool` — Check if the mock server binary exists.
-  `test_connect_and_initialize` function L33-48 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_list_tools` function L51-88 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_call_echo_tool` function L91-107 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_call_add_tool` function L110-126 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_call_unknown_tool` function L129-145 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_call_before_initialize_fails` function L148-163 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_shutdown` function L166-181 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_server_crash_detection` function L188-208 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_connection_closed_detection` function L211-227 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_multiple_servers` function L234-269 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_manager_connect_and_disconnect_individual` function L272-305 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_manager_remove_server` function L308-330 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_manager_tool_count` function L333-351 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_http_transport_config` function L358-368 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_http_transport_creation` function L371-380 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_http_transport_invalid_url` function L383-389 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_server_config_http_builder` function L392-402 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_client_connect_auto_selects_transport` function L405-422 — `()` — These tests use a mock MCP server to verify the full protocol flow.
-  `test_all_tools_flat` function L429-453 — `()` — These tests use a mock MCP server to verify the full protocol flow.

#### crates/arawn-mcp/tests/mock_server.rs

-  `JsonRpcRequest` struct L25-31 — `{ jsonrpc: String, id: u64, method: String, params: Option<Value> }` — JSON-RPC request structure.
-  `JsonRpcResponse` struct L35-42 — `{ jsonrpc: String, id: u64, result: Option<Value>, error: Option<Value> }` — JSON-RPC response structure.
-  `ServerConfig` struct L45-49 — `{ delay_ms: u64, crash_on: Option<String>, slow_tools: Vec<(String, u64)> }` — Server configuration parsed from command line.
-  `ServerConfig` type L51-108 — `= ServerConfig` — --slow-tool T:MS   Add MS delay when tool T is called
-  `from_args` function L52-98 — `() -> Self` — --slow-tool T:MS   Add MS delay when tool T is called
-  `get_tool_delay` function L100-107 — `(&self, tool_name: &str) -> u64` — --slow-tool T:MS   Add MS delay when tool T is called
-  `main` function L110-172 — `()` — --slow-tool T:MS   Add MS delay when tool T is called
-  `handle_request` function L174-309 — `(request: &JsonRpcRequest, config: &ServerConfig) -> JsonRpcResponse` — --slow-tool T:MS   Add MS delay when tool T is called

### crates/arawn-memory/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-memory/src/backend.rs

- pub `MemoryBackend` interface L36-87 — `{ fn insert(), fn get(), fn update(), fn delete(), fn list(), fn count(), fn tou...` — Trait for memory storage backends.
- pub `MemoryBackendExt` interface L93-123 — `{ fn find_contradictions(), fn supersede(), fn reinforce(), fn update_last_acces...` — Extension trait for advanced memory operations.
- pub `MockMemoryBackend` struct L131-133 — `{ memories: std::sync::Mutex<std::collections::HashMap<MemoryId, Memory>> }` — Mock memory backend for testing.
- pub `new` function L138-140 — `() -> Self` — Create a new empty mock backend.
- pub `len` function L143-145 — `(&self) -> usize` — Get the number of stored memories.
- pub `is_empty` function L148-150 — `(&self) -> bool` — Check if the backend is empty.
- pub `clear` function L153-155 — `(&self)` — Clear all stored memories.
-  `find_contradictions` function L97-100 — `(&self, subject: &str, predicate: &str) -> Result<Vec<Memory>>` — Find memories that contradict a given subject/predicate pair.
-  `supersede` function L105-108 — `(&self, old_id: MemoryId, new_id: MemoryId) -> Result<()>` — Mark a memory as superseded by another.
-  `reinforce` function L113-116 — `(&self, id: MemoryId) -> Result<()>` — Reinforce a memory (increment reinforcement count).
-  `update_last_accessed` function L119-122 — `(&self, id: MemoryId) -> Result<()>` — Update the last_accessed timestamp without incrementing access_count.
-  `MockMemoryBackend` type L136-156 — `= MockMemoryBackend` — ```
-  `MockMemoryBackend` type L159-233 — `impl MemoryBackend for MockMemoryBackend` — ```
-  `insert` function L160-164 — `(&self, memory: &Memory) -> Result<()>` — ```
-  `get` function L166-169 — `(&self, id: MemoryId) -> Result<Option<Memory>>` — ```
-  `update` function L171-182 — `(&self, memory: &Memory) -> Result<()>` — ```
-  `delete` function L184-187 — `(&self, id: MemoryId) -> Result<bool>` — ```
-  `list` function L189-209 — `( &self, content_type: Option<ContentType>, limit: usize, offset: usize, ) -> Re...` — ```
-  `count` function L211-218 — `(&self, content_type: Option<ContentType>) -> Result<usize>` — ```
-  `touch` function L220-232 — `(&self, id: MemoryId) -> Result<()>` — ```
-  `MockMemoryBackend` type L236 — `impl MemoryBackendExt for MockMemoryBackend` — ```
-  `tests` module L239-320 — `-` — ```
-  `test_mock_backend_insert_and_get` function L244-252 — `()` — ```
-  `test_mock_backend_update` function L255-266 — `()` — ```
-  `test_mock_backend_delete` function L269-278 — `()` — ```
-  `test_mock_backend_list_and_count` function L281-305 — `()` — ```
-  `test_mock_backend_touch` function L308-319 — `()` — ```

#### crates/arawn-memory/src/error.rs

- pub `MemoryError` enum L7-35 — `Database | Serialization | Query | NotFound | Migration | InvalidUuid | InvalidD...` — Errors that can occur in the memory crate.
- pub `Result` type L38 — `= std::result::Result<T, MemoryError>` — Result type alias for memory operations.

#### crates/arawn-memory/src/graph.rs

- pub `GraphNode` struct L21-28 — `{ id: String, label: String, properties: Vec<(String, String)> }` — A node/entity in the knowledge graph.
- pub `new` function L32-38 — `(id: impl Into<String>, label: impl Into<String>) -> Self` — Create a new graph node.
- pub `with_property` function L41-44 — `(mut self, key: impl Into<String>, value: impl ToString) -> Self` — Add a property to the node.
- pub `RelationshipType` enum L50-67 — `Supports | Contradicts | RelatedTo | CitedIn | Mentions | PartOf | CreatedBy | I...` — Relationship types supported in the knowledge graph.
- pub `as_str` function L71-82 — `(&self) -> &'static str` — Get the string representation for Cypher queries.
- pub `GraphRelationship` struct L87-96 — `{ from_id: String, to_id: String, rel_type: RelationshipType, properties: Vec<(S...` — A relationship/edge in the knowledge graph.
- pub `new` function L100-111 — `( from_id: impl Into<String>, to_id: impl Into<String>, rel_type: RelationshipTy...` — Create a new relationship.
- pub `with_property` function L114-117 — `(mut self, key: impl Into<String>, value: impl ToString) -> Self` — Add a property to the relationship.
- pub `QueryResult` struct L122-125 — `{ row_count: usize }` — Result of a Cypher query.
- pub `GraphStore` struct L134-136 — `{ graph: Graph }` — Knowledge graph backed by graphqlite.
- pub `open` function L140-151 — `(path: impl AsRef<Path>) -> Result<Self>` — Open or create a graph store at the given path.
- pub `open_in_memory` function L154-164 — `() -> Result<Self>` — Create an in-memory graph store.
- pub `add_entity` function L167-181 — `(&self, node: &GraphNode) -> Result<()>` — Add an entity/node to the graph.
- pub `delete_entity` function L186-193 — `(&self, id: &str) -> Result<bool>` — Delete an entity by ID (and all its relationships).
- pub `add_relationship` function L196-214 — `(&self, rel: &GraphRelationship) -> Result<()>` — Add a relationship between two entities.
- pub `get_neighbors` function L220-251 — `(&self, id: &str) -> Result<Vec<String>>` — Get neighbors of an entity.
- pub `stats` function L254-264 — `(&self) -> Result<GraphStats>` — Get graph statistics.
- pub `GraphStats` struct L269-274 — `{ node_count: usize, relationship_count: usize }` — Statistics about the graph store.
-  `GraphNode` type L30-45 — `= GraphNode` — and Cypher query support.
-  `RelationshipType` type L69-83 — `= RelationshipType` — and Cypher query support.
-  `GraphRelationship` type L98-118 — `= GraphRelationship` — and Cypher query support.
-  `GraphStore` type L138-265 — `= GraphStore` — and Cypher query support.
-  `tests` module L281-510 — `-` — and Cypher query support.
-  `create_test_graph` function L285-287 — `() -> GraphStore` — and Cypher query support.
-  `test_open_in_memory` function L291-296 — `()` — and Cypher query support.
-  `test_add_entity` function L300-311 — `()` — and Cypher query support.
-  `test_add_multiple_entities` function L315-330 — `()` — and Cypher query support.
-  `test_add_relationship` function L334-351 — `()` — and Cypher query support.
-  `test_relationship_types` function L355-387 — `()` — and Cypher query support.
-  `test_get_neighbors` function L391-418 — `()` — and Cypher query support.
-  `test_delete_entity` function L422-430 — `()` — and Cypher query support.
-  `test_knowledge_graph_integration` function L434-489 — `()` — and Cypher query support.
-  `test_graph_node_builder` function L492-501 — `()` — and Cypher query support.
-  `test_relationship_type_as_str` function L504-509 — `()` — and Cypher query support.

#### crates/arawn-memory/src/lib.rs

- pub `backend` module L58 — `-` — This crate provides persistent storage for the agent's memories, conversation
- pub `error` module L59 — `-` — - `WebContent`: Fetched web page content
- pub `graph` module L60 — `-` — - `WebContent`: Fetched web page content
- pub `store` module L61 — `-` — - `WebContent`: Fetched web page content
- pub `types` module L62 — `-` — - `WebContent`: Fetched web page content
- pub `validation` module L63 — `-` — - `WebContent`: Fetched web page content
- pub `vector` module L64 — `-` — - `WebContent`: Fetched web page content

#### crates/arawn-memory/src/types.rs

- pub `MemoryId` struct L15 — `-` — Unique identifier for a memory.
- pub `new` function L19-21 — `() -> Self` — Generate a new random memory ID.
- pub `from_uuid` function L24-26 — `(uuid: Uuid) -> Self` — Create from an existing UUID.
- pub `parse` function L29-31 — `(s: &str) -> Result<Self, uuid::Error>` — Parse from a string.
- pub `ContentType` enum L49-68 — `UserMessage | AssistantMessage | ToolUse | FileContent | Note | Fact | WebConten...` — Type of content stored in a memory.
- pub `as_str` function L72-84 — `(&self) -> &'static str` — Get the string representation for database storage.
- pub `parse` function L87-100 — `(s: &str) -> Option<Self>` — Parse from database string.
- pub `ConfidenceSource` enum L110-119 — `Stated | Observed | Inferred | System` — How a fact or memory was established.
- pub `as_str` function L123-130 — `(&self) -> &'static str` — Get the string representation for database storage.
- pub `from_db_str` function L133-141 — `(s: &str) -> Option<Self>` — Parse from database string.
- pub `MemoryConfidence` struct L146-160 — `{ source: ConfidenceSource, reinforcement_count: u32, superseded: bool, supersed...` — Confidence metadata for a memory.
- pub `with_source` function L177-182 — `(source: ConfidenceSource) -> Self` — Create confidence with a specific source.
- pub `compute_score` function L191-215 — `(&self, params: &ConfidenceParams) -> f32` — Compute a composite confidence score from all factors.
- pub `base_score` function L220-227 — `(&self) -> f32` — Base confidence score for this source type.
- pub `ConfidenceParams` struct L232-241 — `{ fresh_days: f32, staleness_days: f32, staleness_floor: f32, reinforcement_cap:...` — Configurable parameters for confidence scoring.
- pub `Citation` enum L264-328 — `Session | File | Web | User | System` — Citation tracking for memory provenance.
- pub `session` function L332-338 — `(session_id: impl Into<String>, message_index: usize) -> Self` — Create a session citation.
- pub `file` function L341-350 — `(path: impl Into<PathBuf>) -> Self` — Create a file citation.
- pub `web` function L353-361 — `(url: impl Into<String>) -> Self` — Create a web citation.
- pub `user` function L364-369 — `(session_id: impl Into<String>) -> Self` — Create a user citation.
- pub `system` function L372-377 — `(method: impl Into<String>) -> Self` — Create a system citation.
- pub `citation_type` function L380-388 — `(&self) -> &'static str` — Get the citation type as a string.
- pub `Staleness` enum L401-425 — `Fresh | PotentiallyStale | Invalidated | Unknown` — Staleness status for a memory's citation source.
- pub `is_fresh` function L429-431 — `(&self) -> bool` — Check if this status indicates fresh data.
- pub `is_stale` function L434-439 — `(&self) -> bool` — Check if this status indicates potential or confirmed staleness.
- pub `Metadata` struct L448-488 — `{ source_path: Option<String>, source_url: Option<String>, session_id: Option<St...` — Metadata associated with a memory.
- pub `Memory` struct L492-524 — `{ id: MemoryId, session_id: Option<String>, content_type: ContentType, content: ...` — A stored memory unit.
- pub `new` function L528-542 — `(content_type: ContentType, content: impl Into<String>) -> Self` — Create a new memory with the given content.
- pub `with_session` function L545-548 — `(mut self, session_id: impl Into<String>) -> Self` — Set the session ID for this memory.
- pub `with_confidence` function L551-554 — `(mut self, confidence: MemoryConfidence) -> Self` — Set the confidence source for this memory.
- pub `with_metadata` function L557-560 — `(mut self, metadata: Metadata) -> Self` — Set metadata for this memory.
- pub `with_tag` function L563-566 — `(mut self, tag: impl Into<String>) -> Self` — Add a tag to this memory.
- pub `with_citation` function L569-572 — `(mut self, citation: Citation) -> Self` — Set the citation for this memory.
- pub `SessionId` struct L581 — `-` — Unique identifier for a session.
- pub `new` function L585-587 — `() -> Self` — Generate a new random session ID.
- pub `from_uuid` function L590-592 — `(uuid: Uuid) -> Self` — Create from an existing UUID.
- pub `parse` function L595-597 — `(s: &str) -> Result<Self, uuid::Error>` — Parse from a string.
- pub `Session` struct L614-626 — `{ id: SessionId, title: Option<String>, created_at: DateTime<Utc>, updated_at: D...` — A conversation session.
- pub `new` function L630-638 — `() -> Self` — Create a new session.
- pub `with_title` function L641-644 — `(mut self, title: impl Into<String>) -> Self` — Create a session with a title.
- pub `NoteId` struct L659 — `-` — Unique identifier for a note.
- pub `new` function L663-665 — `() -> Self` — Generate a new random note ID.
- pub `from_uuid` function L668-670 — `(uuid: Uuid) -> Self` — Create from an existing UUID.
- pub `parse` function L673-675 — `(s: &str) -> Result<Self, uuid::Error>` — Parse from a string.
- pub `Note` struct L692-710 — `{ id: NoteId, title: Option<String>, content: String, tags: Vec<String>, created...` — A user or agent note.
- pub `new` function L714-724 — `(content: impl Into<String>) -> Self` — Create a new note with the given content.
- pub `with_title` function L727-730 — `(mut self, title: impl Into<String>) -> Self` — Set a title for this note.
- pub `with_tag` function L733-736 — `(mut self, tag: impl Into<String>) -> Self` — Add a tag to this note.
- pub `EntityId` struct L745 — `-` — Unique identifier for an entity in the knowledge graph.
- pub `new` function L749-751 — `() -> Self` — Generate a new random entity ID.
- pub `from_uuid` function L754-756 — `(uuid: Uuid) -> Self` — Create from an existing UUID.
- pub `parse` function L759-761 — `(s: &str) -> Result<Self, uuid::Error>` — Parse from a string.
- pub `Entity` struct L778-796 — `{ id: EntityId, label: String, name: String, properties: serde_json::Map<String,...` — An entity in the knowledge graph.
- pub `new` function L800-810 — `(label: impl Into<String>, name: impl Into<String>) -> Self` — Create a new entity with the given label and name.
- pub `with_property` function L813-820 — `( mut self, key: impl Into<String>, value: impl Into<serde_json::Value>, ) -> Se...` — Set a property on this entity.
-  `MemoryId` type L17-32 — `= MemoryId` — Core types for the memory store.
-  `MemoryId` type L34-38 — `impl Default for MemoryId` — Core types for the memory store.
-  `default` function L35-37 — `() -> Self` — Core types for the memory store.
-  `MemoryId` type L40-44 — `= MemoryId` — Core types for the memory store.
-  `fmt` function L41-43 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Core types for the memory store.
-  `ContentType` type L70-101 — `= ContentType` — Core types for the memory store.
-  `ConfidenceSource` type L121-142 — `= ConfidenceSource` — Core types for the memory store.
-  `MemoryConfidence` type L162-173 — `impl Default for MemoryConfidence` — Core types for the memory store.
-  `default` function L163-172 — `() -> Self` — Core types for the memory store.
-  `MemoryConfidence` type L175-216 — `= MemoryConfidence` — Core types for the memory store.
-  `ConfidenceSource` type L218-228 — `= ConfidenceSource` — Core types for the memory store.
-  `ConfidenceParams` type L243-252 — `impl Default for ConfidenceParams` — Core types for the memory store.
-  `default` function L244-251 — `() -> Self` — Core types for the memory store.
-  `Citation` type L330-389 — `= Citation` — Core types for the memory store.
-  `Staleness` type L427-440 — `= Staleness` — Core types for the memory store.
-  `Memory` type L526-573 — `= Memory` — Core types for the memory store.
-  `SessionId` type L583-598 — `= SessionId` — Core types for the memory store.
-  `SessionId` type L600-604 — `impl Default for SessionId` — Core types for the memory store.
-  `default` function L601-603 — `() -> Self` — Core types for the memory store.
-  `SessionId` type L606-610 — `= SessionId` — Core types for the memory store.
-  `fmt` function L607-609 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Core types for the memory store.
-  `Session` type L628-645 — `= Session` — Core types for the memory store.
-  `Session` type L647-651 — `impl Default for Session` — Core types for the memory store.
-  `default` function L648-650 — `() -> Self` — Core types for the memory store.
-  `NoteId` type L661-676 — `= NoteId` — Core types for the memory store.
-  `NoteId` type L678-682 — `impl Default for NoteId` — Core types for the memory store.
-  `default` function L679-681 — `() -> Self` — Core types for the memory store.
-  `NoteId` type L684-688 — `= NoteId` — Core types for the memory store.
-  `fmt` function L685-687 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Core types for the memory store.
-  `Note` type L712-737 — `= Note` — Core types for the memory store.
-  `EntityId` type L747-762 — `= EntityId` — Core types for the memory store.
-  `EntityId` type L764-768 — `impl Default for EntityId` — Core types for the memory store.
-  `default` function L765-767 — `() -> Self` — Core types for the memory store.
-  `EntityId` type L770-774 — `= EntityId` — Core types for the memory store.
-  `fmt` function L771-773 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Core types for the memory store.
-  `Entity` type L798-821 — `= Entity` — Core types for the memory store.
-  `tests` module L828-1182 — `-` — Core types for the memory store.
-  `test_memory_id_roundtrip` function L832-837 — `()` — Core types for the memory store.
-  `test_content_type_roundtrip` function L840-856 — `()` — Core types for the memory store.
-  `test_memory_builder` function L859-875 — `()` — Core types for the memory store.
-  `test_session_builder` function L878-881 — `()` — Core types for the memory store.
-  `test_note_builder` function L884-892 — `()` — Core types for the memory store.
-  `test_entity_builder` function L895-906 — `()` — Core types for the memory store.
-  `test_confidence_source_roundtrip` function L909-920 — `()` — Core types for the memory store.
-  `test_confidence_default` function L923-930 — `()` — Core types for the memory store.
-  `test_confidence_with_source` function L933-937 — `()` — Core types for the memory store.
-  `test_memory_with_confidence` function L940-944 — `()` — Core types for the memory store.
-  `test_base_scores` function L947-952 — `()` — Core types for the memory store.
-  `test_compute_score_fresh_no_reinforcement` function L955-961 — `()` — Core types for the memory store.
-  `test_compute_score_inferred_fresh` function L964-969 — `()` — Core types for the memory store.
-  `test_compute_score_reinforcement_boost` function L972-979 — `()` — Core types for the memory store.
-  `test_compute_score_reinforcement_capped` function L982-989 — `()` — Core types for the memory store.
-  `test_compute_score_superseded` function L992-997 — `()` — Core types for the memory store.
-  `test_compute_score_stale` function L1000-1008 — `()` — Core types for the memory store.
-  `test_compute_score_half_stale` function L1011-1019 — `()` — Core types for the memory store.
-  `test_compute_score_clamped_to_1` function L1022-1032 — `()` — Core types for the memory store.
-  `test_confidence_params_default` function L1035-1041 — `()` — Core types for the memory store.
-  `test_metadata_serialization` function L1044-1056 — `()` — Core types for the memory store.
-  `test_citation_session` function L1063-1077 — `()` — Core types for the memory store.
-  `test_citation_file` function L1080-1088 — `()` — Core types for the memory store.
-  `test_citation_web` function L1091-1099 — `()` — Core types for the memory store.
-  `test_citation_user` function L1102-1110 — `()` — Core types for the memory store.
-  `test_citation_system` function L1113-1121 — `()` — Core types for the memory store.
-  `test_citation_serialization` function L1124-1148 — `()` — Core types for the memory store.
-  `test_memory_with_citation` function L1151-1158 — `()` — Core types for the memory store.
-  `test_staleness_methods` function L1161-1181 — `()` — Core types for the memory store.

#### crates/arawn-memory/src/validation.rs

- pub `ValidationError` enum L18-54 — `EmptyContent | InvalidUtf8 | InvalidConfidence | DimensionMismatch | InvalidEmbe...` — Specific validation error types for memory data.
- pub `validate_embedding` function L78-103 — `( embedding: &[f32], expected_dim: usize, ) -> std::result::Result<(), Validatio...` — Validate an embedding vector.
- pub `validate_embedding_result` function L108-110 — `(embedding: &[f32], expected_dim: usize) -> Result<()>` — Validate an embedding vector, returning a Result<(), MemoryError>.
- pub `validate_memory_content` function L124-136 — `(content: &str) -> std::result::Result<(), ValidationError>` — Validate a memory's content.
- pub `validate_memory` function L146-150 — `(memory: &Memory) -> std::result::Result<(), ValidationError>` — Validate a complete memory structure.
- pub `validate_memory_result` function L155-157 — `(memory: &Memory) -> Result<()>` — Validate a memory, returning a Result<(), MemoryError>.
- pub `validate_confidence_score` function L167-172 — `(score: f32) -> std::result::Result<(), ValidationError>` — Validate a confidence score is in the valid range [0.0, 1.0].
- pub `validate_session_id` function L186-199 — `(session_id: &str) -> std::result::Result<(), ValidationError>` — Validate a session ID string.
- pub `validate_session_id_result` function L204-206 — `(session_id: &str) -> Result<()>` — Validate a session ID, returning a Result<(), MemoryError>.
-  `MemoryError` type L56-60 — `= MemoryError` — - Session ID formats
-  `from` function L57-59 — `(err: ValidationError) -> Self` — - Session ID formats
-  `tests` module L213-391 — `-` — - Session ID formats
-  `test_validate_embedding_valid` function L218-221 — `()` — - Session ID formats
-  `test_validate_embedding_wrong_dimension` function L224-234 — `()` — - Session ID formats
-  `test_validate_embedding_nan` function L237-244 — `()` — - Session ID formats
-  `test_validate_embedding_infinity` function L247-254 — `()` — - Session ID formats
-  `test_validate_embedding_empty` function L257-269 — `()` — - Session ID formats
-  `test_validate_memory_content_valid` function L272-276 — `()` — - Session ID formats
-  `test_validate_memory_content_empty` function L279-282 — `()` — - Session ID formats
-  `test_validate_memory_content_null_byte` function L285-288 — `()` — - Session ID formats
-  `test_validate_confidence_score_valid` function L291-295 — `()` — - Session ID formats
-  `test_validate_confidence_score_invalid` function L298-307 — `()` — - Session ID formats
-  `test_validate_memory_valid` function L310-313 — `()` — - Session ID formats
-  `test_validate_memory_empty_content` function L316-321 — `()` — - Session ID formats
-  `test_validate_memory_invalid_confidence` function L324-332 — `()` — - Session ID formats
-  `test_validate_session_id_valid` function L335-338 — `()` — - Session ID formats
-  `test_validate_session_id_empty` function L341-344 — `()` — - Session ID formats
-  `test_validate_session_id_invalid_format` function L347-359 — `()` — - Session ID formats
-  `test_validation_error_to_memory_error` function L362-366 — `()` — - Session ID formats
-  `test_validate_embedding_result` function L369-375 — `()` — - Session ID formats
-  `test_validate_memory_result` function L378-381 — `()` — - Session ID formats
-  `test_validate_session_id_result` function L384-390 — `()` — - Session ID formats

#### crates/arawn-memory/src/vector.rs

- pub `DEFAULT_EMBEDDING_DIMS` variable L18 — `: usize` — Default embedding dimensions (MiniLM-L6-v2 produces 384-dim vectors).
- pub `init_vector_extension` function L28-36 — `()` — Initialize sqlite-vec extension for a connection.
- pub `check_vector_extension` function L39-42 — `(conn: &Connection) -> Result<String>` — Check if sqlite-vec extension is loaded.
- pub `create_vector_table` function L47-62 — `(conn: &Connection, dims: usize) -> Result<()>` — Create the vector embeddings table.
- pub `drop_vector_table` function L67-71 — `(conn: &Connection) -> Result<()>` — Drop the vector embeddings table.
- pub `store_embedding` function L76-90 — `(conn: &Connection, memory_id: MemoryId, embedding: &[f32]) -> Result<()>` — Store an embedding for a memory.
- pub `delete_embedding` function L93-100 — `(conn: &Connection, memory_id: MemoryId) -> Result<bool>` — Delete an embedding for a memory.
- pub `SimilarityResult` struct L104-109 — `{ memory_id: MemoryId, distance: f32 }` — Result of a similarity search.
- pub `search_similar` function L114-149 — `( conn: &Connection, query_embedding: &[f32], limit: usize, ) -> Result<Vec<Simi...` — Search for memories similar to a query embedding.
- pub `search_similar_filtered` function L155-210 — `( conn: &Connection, query_embedding: &[f32], memory_ids: &[MemoryId], limit: us...` — Search for memories similar to a query, filtered by memory IDs.
- pub `count_embeddings` function L213-218 — `(conn: &Connection) -> Result<usize>` — Get the count of stored embeddings.
- pub `has_embedding` function L221-228 — `(conn: &Connection, memory_id: MemoryId) -> Result<bool>` — Check if an embedding exists for a memory.
-  `tests` module L235-358 — `-` — using the sqlite-vec SQLite extension.
-  `create_test_connection` function L238-243 — `() -> Connection` — using the sqlite-vec SQLite extension.
-  `test_vector_extension_loads` function L246-252 — `()` — using the sqlite-vec SQLite extension.
-  `test_create_vector_table` function L255-260 — `()` — using the sqlite-vec SQLite extension.
-  `test_store_and_retrieve_embedding` function L263-273 — `()` — using the sqlite-vec SQLite extension.
-  `test_delete_embedding` function L276-288 — `()` — using the sqlite-vec SQLite extension.
-  `test_similarity_search` function L291-318 — `()` — using the sqlite-vec SQLite extension.
-  `test_similarity_search_with_limit` function L321-335 — `()` — using the sqlite-vec SQLite extension.
-  `test_update_embedding` function L338-357 — `()` — using the sqlite-vec SQLite extension.

### crates/arawn-memory/src/store

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-memory/src/store/graph_ops.rs

- pub `add_graph_entity` function L10-15 — `(&self, node: &GraphNode) -> Result<()>` — Add an entity to the knowledge graph.
- pub `add_graph_relationship` function L18-23 — `(&self, rel: &GraphRelationship) -> Result<()>` — Add a relationship to the knowledge graph.
- pub `delete_graph_entity` function L26-31 — `(&self, id: &str) -> Result<bool>` — Delete an entity from the knowledge graph.
- pub `get_graph_neighbors` function L34-39 — `(&self, id: &str) -> Result<Vec<String>>` — Get neighbors of an entity in the knowledge graph.
- pub `graph_stats` function L42-47 — `(&self) -> Result<GraphStats>` — Get knowledge graph statistics.
-  `MemoryStore` type L8-48 — `= MemoryStore` — Graph passthrough operations.
-  `tests` module L51-112 — `-` — Graph passthrough operations.
-  `test_graph_operations_without_init` function L57-66 — `()` — Graph passthrough operations.
-  `test_graph_passthrough_operations` function L70-92 — `()` — Graph passthrough operations.
-  `test_has_vectors_and_has_graph` function L96-111 — `()` — Graph passthrough operations.

#### crates/arawn-memory/src/store/memory_ops.rs

- pub `insert_memory` function L16-54 — `(&self, memory: &Memory) -> Result<()>` — Insert a new memory.
- pub `get_memory` function L57-78 — `(&self, id: MemoryId) -> Result<Option<Memory>>` — Get a memory by ID.
- pub `update_memory` function L81-125 — `(&self, memory: &Memory) -> Result<()>` — Update an existing memory.
- pub `delete_memory` function L128-141 — `(&self, id: MemoryId) -> Result<bool>` — Delete a memory by ID.
- pub `list_memories` function L144-196 — `( &self, content_type: Option<ContentType>, limit: usize, offset: usize, ) -> Re...` — List memories with optional filtering.
- pub `count_memories` function L199-213 — `(&self, content_type: Option<ContentType>) -> Result<usize>` — Count memories with optional filtering.
- pub `touch_memory` function L216-233 — `(&self, id: MemoryId) -> Result<()>` — Record access to a memory (updates accessed_at and access_count).
- pub `find_contradictions` function L311-334 — `(&self, subject: &str, predicate: &str) -> Result<Vec<Memory>>` — Find existing non-superseded memories that match the given subject and predicate.
- pub `supersede` function L339-357 — `(&self, old_id: MemoryId, new_id: MemoryId) -> Result<()>` — Mark a memory as superseded by another memory.
- pub `reinforce` function L362-381 — `(&self, id: MemoryId) -> Result<()>` — Reinforce a memory by incrementing its reinforcement count and updating last_accessed.
- pub `update_last_accessed` function L384-397 — `(&self, id: MemoryId) -> Result<()>` — Update the last_accessed timestamp on a memory (e.g., when recalled).
-  `MemoryStore` type L14-398 — `= MemoryStore` — Memory CRUD operations.
-  `row_to_memory` function L240-305 — `(row: &rusqlite::Row) -> Result<Memory>` — Convert a database row to a Memory struct.
-  `tests` module L401-589 — `-` — Memory CRUD operations.
-  `create_test_store` function L404-406 — `() -> MemoryStore` — Memory CRUD operations.
-  `test_memory_crud` function L409-432 — `()` — Memory CRUD operations.
-  `test_memory_list_and_count` function L435-461 — `()` — Memory CRUD operations.
-  `test_touch_memory` function L464-479 — `()` — Memory CRUD operations.
-  `make_fact` function L481-486 — `(subject: &str, predicate: &str, content: &str) -> Memory` — Memory CRUD operations.
-  `test_find_contradictions` function L489-510 — `()` — Memory CRUD operations.
-  `test_supersede` function L513-533 — `()` — Memory CRUD operations.
-  `test_supersede_not_found` function L536-541 — `()` — Memory CRUD operations.
-  `test_reinforce` function L544-559 — `()` — Memory CRUD operations.
-  `test_reinforce_not_found` function L562-565 — `()` — Memory CRUD operations.
-  `test_update_last_accessed` function L568-582 — `()` — Memory CRUD operations.
-  `test_update_last_accessed_not_found` function L585-588 — `()` — Memory CRUD operations.

#### crates/arawn-memory/src/store/mod.rs

- pub `query` module L19 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
- pub `MemoryStore` struct L58-67 — `{ conn: Mutex<Connection>, graph: Option<GraphStore>, vectors_initialized: Mutex...` — Memory store backed by SQLite.
- pub `StoreOptions` struct L87-92 — `{ embedding: Option<Vec<f32>>, entities: Vec<EntityLink> }` — Options for storing a memory with the unified API.
- pub `EntityLink` struct L96-105 — `{ entity_id: String, label: String, relationship: RelationshipType, properties: ...` — An entity link to create in the knowledge graph.
- pub `new` function L109-120 — `( entity_id: impl Into<String>, label: impl Into<String>, relationship: Relation...` — Create a new entity link.
- pub `with_property` function L123-126 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add a property to the entity.
- pub `open` function L137-166 — `(path: impl AsRef<Path>) -> Result<Self>` — Open or create a memory store at the given path.
- pub `open_in_memory` function L169-181 — `() -> Result<Self>` — Create an in-memory store (useful for testing).
- pub `init_graph` function L187-192 — `(&mut self) -> Result<()>` — Initialize knowledge graph capabilities.
- pub `init_graph_at_path` function L195-200 — `(&mut self, path: impl AsRef<Path>) -> Result<()>` — Initialize knowledge graph at a specific path.
- pub `has_graph` function L203-205 — `(&self) -> bool` — Check if the knowledge graph is initialized.
- pub `has_vectors` function L208-210 — `(&self) -> bool` — Check if vectors are initialized.
- pub `graph` function L213-215 — `(&self) -> Option<&GraphStore>` — Get a reference to the graph store (if initialized).
- pub `with_transaction` function L438-455 — `(&self, f: F) -> Result<T>` — Execute a function within a transaction.
- pub `get_meta` function L464-476 — `(&self, key: &str) -> Result<Option<String>>` — Get or set a metadata value.
- pub `set_meta` function L479-488 — `(&self, key: &str, value: &str) -> Result<()>` — Set a metadata value.
- pub `stats` function L491-528 — `(&self) -> Result<StoreStats>` — Get database statistics.
-  `graph_ops` module L16 — `-` — Provides persistent storage for memories, sessions, and notes using rusqlite.
-  `memory_ops` module L17 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `note_ops` module L18 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `recall` module L20 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `session_ops` module L21 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `unified_ops` module L22 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `vector_ops` module L23 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `SCHEMA_VERSION` variable L44 — `: i32` — Current schema version for migrations.
-  `MemoryStore` type L72 — `impl Send for MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `MemoryStore` type L73 — `impl Sync for MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `MemoryStore` type L75-83 — `= MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `fmt` function L76-82 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `EntityLink` type L107-127 — `= EntityLink` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `MemoryStore` type L133-415 — `= MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `initialize` function L218-230 — `(&self) -> Result<()>` — Initialize the database with schema and pragmas.
-  `create_schema` function L233-325 — `(&self, conn: &Connection) -> Result<()>` — Create the database schema.
-  `migrate_v2` function L328-349 — `(&self, conn: &Connection) -> Result<()>` — Migration v2: Add confidence columns to memories table.
-  `migrate_v3` function L352-385 — `(&self, conn: &Connection) -> Result<()>` — Migration v3: Add session_id column to memories table and backfill from metadata JSON.
-  `migrate_v4` function L388-414 — `(&self, conn: &Connection) -> Result<()>` — Migration v4: Add citation column to memories table.
-  `MemoryStore` type L421-456 — `= MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `MemoryStore` type L462-529 — `= MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `MemoryStore` type L535-568 — `= MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `insert` function L536-538 — `(&self, memory: &crate::types::Memory) -> Result<()>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `get` function L540-542 — `(&self, id: crate::types::MemoryId) -> Result<Option<crate::types::Memory>>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `update` function L544-546 — `(&self, memory: &crate::types::Memory) -> Result<()>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `delete` function L548-550 — `(&self, id: crate::types::MemoryId) -> Result<bool>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `list` function L552-559 — `( &self, content_type: Option<crate::types::ContentType>, limit: usize, offset: ...` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `count` function L561-563 — `(&self, content_type: Option<crate::types::ContentType>) -> Result<usize>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `touch` function L565-567 — `(&self, id: crate::types::MemoryId) -> Result<()>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `MemoryStore` type L570-594 — `= MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `find_contradictions` function L571-577 — `( &self, subject: &str, predicate: &str, ) -> Result<Vec<crate::types::Memory>>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `supersede` function L579-585 — `( &self, old_id: crate::types::MemoryId, new_id: crate::types::MemoryId, ) -> Re...` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `reinforce` function L587-589 — `(&self, id: crate::types::MemoryId) -> Result<()>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `update_last_accessed` function L591-593 — `(&self, id: crate::types::MemoryId) -> Result<()>` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `tests` module L597-669 — `-` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `create_test_store` function L601-603 — `() -> MemoryStore` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `test_open_in_memory` function L606-611 — `()` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `test_meta_operations` function L614-630 — `()` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `test_store_stats` function L633-649 — `()` — - `update_indexed()`: Update a memory and re-index its embedding/entities
-  `test_with_transaction` function L652-668 — `()` — - `update_indexed()`: Update a memory and re-index its embedding/entities

#### crates/arawn-memory/src/store/note_ops.rs

- pub `insert_note` function L14-36 — `(&self, note: &Note) -> Result<()>` — Insert a new note.
- pub `get_note` function L39-54 — `(&self, id: NoteId) -> Result<Option<Note>>` — Get a note by ID.
- pub `update_note` function L57-82 — `(&self, note: &Note) -> Result<()>` — Update a note.
- pub `delete_note` function L85-92 — `(&self, id: NoteId) -> Result<bool>` — Delete a note by ID.
- pub `list_notes` function L95-115 — `(&self, limit: usize, offset: usize) -> Result<Vec<Note>>` — List notes ordered by updated_at descending.
- pub `search_notes` function L118-141 — `(&self, query: &str, limit: usize) -> Result<Vec<Note>>` — Search notes by content or title.
- pub `list_notes_by_tag` function L144-167 — `(&self, tag: &str, limit: usize) -> Result<Vec<Note>>` — List notes that have a specific tag.
- pub `list_notes_by_tags` function L170-214 — `(&self, tags: &[&str], limit: usize) -> Result<Vec<Note>>` — List notes that have all of the specified tags.
- pub `count_notes_by_tag` function L217-229 — `(&self, tag: &str) -> Result<usize>` — Count notes with a specific tag.
-  `MemoryStore` type L12-258 — `= MemoryStore` — Note CRUD, search, and tag operations.
-  `row_to_note` function L232-257 — `(row: &rusqlite::Row) -> Result<Note>` — Convert a database row to a Note struct.
-  `tests` module L261-391 — `-` — Note CRUD, search, and tag operations.
-  `create_test_store` function L264-266 — `() -> MemoryStore` — Note CRUD, search, and tag operations.
-  `test_note_crud` function L269-291 — `()` — Note CRUD, search, and tag operations.
-  `test_note_search` function L294-310 — `()` — Note CRUD, search, and tag operations.
-  `test_list_notes_by_tag` function L313-340 — `()` — Note CRUD, search, and tag operations.
-  `test_list_notes_by_tags_multiple` function L343-371 — `()` — Note CRUD, search, and tag operations.
-  `test_count_notes_by_tag` function L374-390 — `()` — Note CRUD, search, and tag operations.

#### crates/arawn-memory/src/store/query.rs

- pub `TimeRange` enum L14-24 — `Today | Week | Month | All` — Time range filter for recall queries.
- pub `cutoff` function L28-36 — `(&self) -> Option<DateTime<Utc>>` — Get the cutoff datetime for this time range.
- pub `RecallQuery` struct L45-62 — `{ embedding: Vec<f32>, limit: usize, time_range: TimeRange, content_types: Vec<C...` — Query parameters for combined recall.
- pub `new` function L66-77 — `(embedding: Vec<f32>) -> Self` — Create a new recall query with an embedding.
- pub `with_limit` function L80-83 — `(mut self, limit: usize) -> Self` — Set the maximum number of results.
- pub `with_time_range` function L86-89 — `(mut self, range: TimeRange) -> Self` — Set the time range filter.
- pub `with_content_type` function L92-95 — `(mut self, ct: ContentType) -> Self` — Add a content type filter.
- pub `with_vector_weight` function L98-101 — `(mut self, weight: f32) -> Self` — Set the vector weight for blending (0.0-1.0).
- pub `with_graph_context` function L104-107 — `(mut self, include: bool) -> Self` — Set whether to include graph context.
- pub `with_min_score` function L110-113 — `(mut self, score: f32) -> Self` — Set the minimum score threshold (0.0-1.0).
- pub `with_session` function L116-119 — `(mut self, session_id: impl Into<String>) -> Self` — Filter results to a specific session.
- pub `RecallMatch` struct L128-143 — `{ memory: Memory, distance: f32, similarity_score: f32, confidence_score: f32, s...` — A single match in recall results.
- pub `RecallResult` struct L147-156 — `{ matches: Vec<RecallMatch>, entities: Vec<String>, searched_count: usize, query...` — Result of a recall query.
- pub `MemoryWithContext` struct L164-171 — `{ memory: Memory, related_entities: Vec<RelatedEntity>, has_embedding: bool }` — A memory with its graph context.
- pub `RelatedEntity` struct L175-180 — `{ entity_id: String, relationship: RelationshipType }` — An entity related to a memory.
- pub `StoreStats` struct L188-205 — `{ memory_count: usize, session_count: usize, note_count: usize, embedding_count:...` — Statistics about the memory store.
- pub `ReindexReport` struct L209-218 — `{ total: usize, embedded: usize, skipped: usize, elapsed: std::time::Duration }` — Report from a reindex operation.
- pub `ReindexDryRun` struct L222-227 — `{ memory_count: usize, estimated_tokens: usize }` — Dry-run result for a reindex operation.
- pub `StoreFactResult` enum L231-244 — `Inserted | Reinforced | Superseded` — Result of a `store_fact()` operation.
-  `TimeRange` type L26-37 — `= TimeRange` — Query types for memory recall and search.
-  `RecallQuery` type L64-120 — `= RecallQuery` — Query types for memory recall and search.

#### crates/arawn-memory/src/store/recall.rs

- pub `recall` function L33-171 — `(&self, query: RecallQuery) -> Result<RecallResult>` — Combined recall query blending vector similarity and graph context.
- pub `search_memories` function L177-202 — `(&self, query: &str, limit: usize) -> Result<Vec<Memory>>` — Simple text search across memories.
- pub `search_memories_in_range` function L268-304 — `( &self, query: &str, time_range: TimeRange, limit: usize, ) -> Result<Vec<Memor...` — Search memories with time range filter.
-  `MemoryStore` type L11-305 — `= MemoryStore` — Recall and text search operations.
-  `compute_staleness` function L209-265 — `(memory: &Memory) -> Staleness` — Compute staleness status for a memory based on its citation.
-  `tests` module L308-727 — `-` — Recall and text search operations.
-  `create_test_store` function L315-317 — `() -> MemoryStore` — Recall and text search operations.
-  `create_test_store_with_vectors` function L319-324 — `() -> MemoryStore` — Recall and text search operations.
-  `test_recall_basic` function L327-353 — `()` — Recall and text search operations.
-  `test_recall_with_content_type_filter` function L356-377 — `()` — Recall and text search operations.
-  `test_recall_with_time_filter` function L380-401 — `()` — Recall and text search operations.
-  `test_recall_with_graph_context` function L405-430 — `()` — Recall and text search operations.
-  `test_recall_vector_weight` function L433-451 — `()` — Recall and text search operations.
-  `test_recall_result_ordering` function L454-478 — `()` — Recall and text search operations.
-  `test_recall_query_builder` function L481-495 — `()` — Recall and text search operations.
-  `test_recall_without_vectors_fails` function L498-505 — `()` — Recall and text search operations.
-  `test_search_memories_text` function L508-523 — `()` — Recall and text search operations.
-  `test_time_range_cutoffs` function L526-531 — `()` — Recall and text search operations.
-  `test_recall_performance_many_memories` function L534-565 — `()` — Recall and text search operations.
-  `test_recall_mixed_content_integration` function L569-652 — `()` — Recall and text search operations.
-  `test_recall_high_confidence_ranks_above_low` function L655-678 — `()` — Recall and text search operations.
-  `test_recall_superseded_excluded_by_min_score` function L681-705 — `()` — Recall and text search operations.
-  `test_recall_match_includes_confidence_score` function L708-726 — `()` — Recall and text search operations.

#### crates/arawn-memory/src/store/session_ops.rs

- pub `insert_session` function L14-32 — `(&self, session: &Session) -> Result<()>` — Insert a new session.
- pub `get_session` function L35-49 — `(&self, id: SessionId) -> Result<Option<Session>>` — Get a session by ID.
- pub `update_session` function L52-69 — `(&self, session: &Session) -> Result<()>` — Update a session.
- pub `delete_session` function L72-81 — `(&self, id: SessionId) -> Result<bool>` — Delete a session by ID.
- pub `list_sessions` function L84-104 — `(&self, limit: usize, offset: usize) -> Result<Vec<Session>>` — List sessions ordered by updated_at descending.
- pub `get_or_create_session` function L110-124 — `(&self, id: SessionId) -> Result<Session>` — Get or create a session by ID.
- pub `append_to_session` function L132-179 — `( &self, session_id: SessionId, content_type: ContentType, content: impl Into<St...` — Append an entry to a session.
- pub `append_to_session_with_embedding` function L184-225 — `( &self, session_id: SessionId, content_type: ContentType, content: impl Into<St...` — Append an entry to a session with an optional embedding.
- pub `get_session_history` function L230-260 — `( &self, session_id: SessionId, limit: usize, offset: usize, ) -> Result<Vec<Mem...` — Get session history (all memories associated with a session).
- pub `count_session_entries` function L263-275 — `(&self, session_id: SessionId) -> Result<usize>` — Count entries in a session.
-  `MemoryStore` type L12-299 — `= MemoryStore` — Session CRUD and entry operations.
-  `row_to_session` function L278-298 — `(row: &rusqlite::Row) -> Result<Session>` — Convert a database row to a Session struct.
-  `tests` module L302-484 — `-` — Session CRUD and entry operations.
-  `create_test_store` function L305-307 — `() -> MemoryStore` — Session CRUD and entry operations.
-  `create_test_store_with_vectors` function L309-314 — `() -> MemoryStore` — Session CRUD and entry operations.
-  `test_session_crud` function L317-335 — `()` — Session CRUD and entry operations.
-  `test_get_or_create_session_existing` function L338-347 — `()` — Session CRUD and entry operations.
-  `test_get_or_create_session_new` function L350-361 — `()` — Session CRUD and entry operations.
-  `test_append_to_session` function L364-388 — `()` — Session CRUD and entry operations.
-  `test_get_session_history` function L391-412 — `()` — Session CRUD and entry operations.
-  `test_get_session_history_pagination` function L415-439 — `()` — Session CRUD and entry operations.
-  `test_count_session_entries` function L442-461 — `()` — Session CRUD and entry operations.
-  `test_append_to_session_with_embedding` function L464-483 — `()` — Session CRUD and entry operations.

#### crates/arawn-memory/src/store/unified_ops.rs

- pub `store` function L32-80 — `(&self, memory: &Memory, options: StoreOptions) -> Result<()>` — Store a memory with optional embedding and graph entities.
- pub `get_with_context` function L86-119 — `(&self, id: MemoryId) -> Result<Option<MemoryWithContext>>` — Retrieve a memory with its graph context.
- pub `delete_cascade` function L129-145 — `(&self, id: MemoryId) -> Result<bool>` — Delete a memory and all associated data (cascade delete).
- pub `update_indexed` function L155-194 — `(&self, memory: &Memory, options: StoreOptions) -> Result<()>` — Update a memory and re-index its embedding and entities.
- pub `store_fact` function L204-252 — `(&self, memory: &Memory, options: StoreOptions) -> Result<StoreFactResult>` — Store a fact with automatic reinforcement and contradiction detection.
-  `MemoryStore` type L11-253 — `= MemoryStore` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `tests` module L256-622 — `-` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `create_unified_test_store` function L262-268 — `() -> MemoryStore` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_with_embedding` function L272-287 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_with_entities` function L291-310 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_full_options` function L314-331 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_get_with_context` function L335-356 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_get_with_context_not_found` function L360-366 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_delete_cascade` function L370-395 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_update_indexed` function L399-433 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_without_subsystems` function L436-453 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `make_fact` function L455-460 — `(subject: &str, predicate: &str, content: &str) -> Memory` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_fact_supersedes_contradiction` function L463-489 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_fact_no_contradiction_different_predicate` function L492-501 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_fact_no_subject_skips_contradiction_check` function L504-510 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_fact_reinforces_exact_match` function L513-535 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_fact_reinforced_score_higher` function L538-572 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.
-  `test_store_fact_multiple_supersessions` function L575-621 — `()` — Unified API: store, get_with_context, delete_cascade, update_indexed.

#### crates/arawn-memory/src/store/vector_ops.rs

- pub `init_vectors` function L22-62 — `(&self, dims: usize, provider: &str) -> Result<()>` — Initialize vector search capabilities.
- pub `vectors_stale` function L65-67 — `(&self) -> bool` — Check if vector embeddings are stale (dimension/provider mismatch).
- pub `reindex_dry_run` function L70-83 — `(&self) -> Result<ReindexDryRun>` — Dry-run reindex: returns counts without doing any work.
- pub `reindex` function L92-182 — `( &self, embed_batch: F, new_dims: usize, new_provider: &str, ) -> Result<Reinde...` — Reindex all memory embeddings with a new embedder/dimensions.
- pub `insert_memory_with_embedding` function L188-195 — `(&self, memory: &Memory, embedding: &[f32]) -> Result<()>` — Store a memory with its embedding.
- pub `store_embedding` function L198-201 — `(&self, memory_id: MemoryId, embedding: &[f32]) -> Result<()>` — Store an embedding for an existing memory.
- pub `delete_embedding` function L204-207 — `(&self, memory_id: MemoryId) -> Result<bool>` — Delete an embedding for a memory.
- pub `search_similar` function L212-219 — `( &self, query_embedding: &[f32], limit: usize, ) -> Result<Vec<crate::vector::S...` — Search for similar memories using vector similarity.
- pub `search_similar_memories` function L224-243 — `( &self, query_embedding: &[f32], limit: usize, ) -> Result<Vec<(Memory, f32)>>` — Search for similar memories and return the full Memory objects.
- pub `has_embedding` function L246-249 — `(&self, memory_id: MemoryId) -> Result<bool>` — Check if a memory has an embedding.
- pub `count_embeddings` function L252-255 — `(&self) -> Result<usize>` — Get the count of stored embeddings.
-  `MemoryStore` type L11-256 — `= MemoryStore` — Vector search and embedding operations.
-  `tests` module L259-514 — `-` — Vector search and embedding operations.
-  `create_test_store_with_vectors` function L263-268 — `() -> MemoryStore` — Vector search and embedding operations.
-  `test_memory_with_embedding` function L271-286 — `()` — Vector search and embedding operations.
-  `test_vector_search_via_store` function L289-313 — `()` — Vector search and embedding operations.
-  `test_vector_search_100_memories` function L316-352 — `()` — Vector search and embedding operations.
-  `test_stats_with_embeddings` function L355-369 — `()` — Vector search and embedding operations.
-  `test_init_vectors_stores_metadata` function L372-385 — `()` — Vector search and embedding operations.
-  `test_init_vectors_same_dims_ok` function L388-393 — `()` — Vector search and embedding operations.
-  `test_init_vectors_dimension_mismatch_marks_stale` function L396-403 — `()` — Vector search and embedding operations.
-  `test_stale_vectors_search_returns_empty` function L406-422 — `()` — Vector search and embedding operations.
-  `test_stats_includes_embedding_metadata` function L425-433 — `()` — Vector search and embedding operations.
-  `test_reindex_dry_run` function L436-448 — `()` — Vector search and embedding operations.
-  `test_reindex_reembeds_all_memories` function L451-487 — `()` — Vector search and embedding operations.
-  `test_reindex_skips_empty_content` function L490-513 — `()` — Vector search and embedding operations.

### crates/arawn-oauth/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-oauth/src/error.rs

- pub `Result` type L4 — `= std::result::Result<T, OAuthError>` — Result type alias for this crate.
- pub `OAuthError` enum L8-28 — `Network | Backend | InvalidRequest | Config | Serialization` — Errors that can occur in the OAuth proxy.
-  `OAuthError` type L30-34 — `= OAuthError` — Error types for the OAuth proxy.
-  `from` function L31-33 — `(e: reqwest::Error) -> Self` — Error types for the OAuth proxy.

#### crates/arawn-oauth/src/lib.rs

- pub `error` module L14 — `-` — Provides a vendored OAuth proxy that enables Arawn to use Claude MAX
- pub `oauth` module L15 — `-` — - [`proxy`] — Axum-based localhost proxy server
- pub `passthrough` module L16 — `-` — - [`proxy`] — Axum-based localhost proxy server
- pub `proxy` module L17 — `-` — - [`proxy`] — Axum-based localhost proxy server
- pub `token_manager` module L18 — `-` — - [`proxy`] — Axum-based localhost proxy server

#### crates/arawn-oauth/src/oauth.rs

- pub `OAuthConfig` struct L23-29 — `{ client_id: String, authorize_url: String, token_url: String, redirect_uri: Str...` — OAuth configuration for Anthropic MAX plan.
- pub `anthropic_max` function L53-66 — `() -> Self` — Create OAuth config for Anthropic MAX plan.
- pub `with_overrides` function L69-93 — `( mut self, client_id: Option<&str>, authorize_url: Option<&str>, token_url: Opt...` — Apply config overrides.
- pub `PkceChallenge` struct L108-111 — `{ verifier: String, challenge: String }` — PKCE code verifier and challenge pair.
- pub `generate` function L115-129 — `() -> Self` — Generate a new PKCE challenge pair.
- pub `generate_state` function L133-137 — `() -> String` — Generate a random state string for CSRF protection.
- pub `build_authorization_url` function L152-171 — `(config: &OAuthConfig, challenge: &str, state: &str) -> String` — Build the authorization URL for the OAuth flow.
- pub `OAuthTokens` struct L175-185 — `{ access_token: String, refresh_token: String, expires_in: u64, token_type: Stri...` — OAuth tokens returned from token exchange.
- pub `exchange_code_for_tokens` function L205-253 — `( config: &OAuthConfig, code: &str, verifier: &str, state: &str, ) -> Result<OAu...` — Exchange an authorization code for OAuth tokens.
- pub `refresh_access_token` function L256-299 — `( config: &OAuthConfig, refresh_token: &str, ) -> Result<OAuthTokens>` — Refresh an access token using a refresh token.
- pub `parse_code_state` function L312-328 — `(input: &str) -> Result<(String, String)>` — Parse the code#state response from the OAuth callback.
-  `OAuthConfig` type L31-35 — `impl Default for OAuthConfig` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `default` function L32-34 — `() -> Self` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `OAuthConfig` type L37-94 — `= OAuthConfig` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `DEFAULT_CLIENT_ID` variable L39 — `: &str` — Default client ID for Anthropic MAX plan OAuth.
-  `DEFAULT_AUTHORIZE_URL` variable L40 — `: &str` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `DEFAULT_TOKEN_URL` variable L41 — `: &str` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `DEFAULT_REDIRECT_URI` variable L42 — `: &str` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `DEFAULT_SCOPE` variable L43 — `: &str` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `PkceChallenge` type L113-130 — `= PkceChallenge` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `TokenExchangeRequest` struct L188-195 — `{ code: String, state: String, grant_type: String, client_id: String, redirect_u...` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `TokenRefreshRequest` struct L198-202 — `{ grant_type: String, client_id: String, refresh_token: String }` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `tests` module L331-389 — `-` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_pkce_generation` function L335-340 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_state_generation` function L343-348 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_authorization_url` function L351-360 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_parse_code_state_valid` function L363-367 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_parse_code_state_with_whitespace` function L370-374 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_parse_code_state_invalid` function L377-381 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.
-  `test_oauth_config_default` function L384-388 — `()` — OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.

#### crates/arawn-oauth/src/passthrough.rs

- pub `ANTHROPIC_API_URL` variable L13 — `: &str` — Anthropic API base URL.
- pub `ANTHROPIC_VERSION` variable L16 — `: &str` — Anthropic API version header.
- pub `ANTHROPIC_BETA` variable L19 — `: &str` — Required anthropic-beta header for OAuth/MAX plan.
- pub `CLAUDE_CODE_SYSTEM_PROMPT` variable L22-23 — `: &str` — Required system prompt for Claude Code with MAX plan.
- pub `AuthMode` enum L27-34 — `ApiKey | OAuth | OAuthWithFallback` — Authentication mode for passthrough requests.
- pub `PassthroughConfig` struct L38-45 — `{ base_url: String, messages_path: String, auth_header: String, extra_headers: H...` — Configuration for the passthrough client.
- pub `anthropic_oauth` function L49-65 — `() -> Self` — Create config for Anthropic API with OAuth (MAX plan).
- pub `anthropic_api_key` function L68-83 — `() -> Self` — Create config for Anthropic API with API key auth.
- pub `Passthrough` struct L94-98 — `{ client: Client, config: PassthroughConfig, token_manager: Option<SharedTokenMa...` — Passthrough client for forwarding requests to upstream APIs.
- pub `new` function L102-104 — `() -> Self` — Create a new passthrough client with default config (OAuth mode).
- pub `with_config` function L107-113 — `(config: PassthroughConfig) -> Self` — Create with custom config.
- pub `with_token_manager` function L116-119 — `(mut self, manager: SharedTokenManager) -> Self` — Set the token manager for OAuth authentication.
- pub `config` function L122-124 — `(&self) -> &PassthroughConfig` — Get the config.
- pub `forward_raw` function L127-171 — `( &self, request: serde_json::Value, api_key: Option<&str>, ) -> Result<serde_js...` — Forward a raw JSON request to the upstream API (non-streaming).
- pub `forward_raw_stream` function L174-214 — `( &self, request: serde_json::Value, api_key: Option<&str>, ) -> Result<reqwest:...` — Forward a raw JSON streaming request, returning the raw response.
- pub `extract_api_key` function L344-371 — `( headers: &axum::http::HeaderMap, config: &PassthroughConfig, ) -> Option<Strin...` — Extract API key from request headers.
-  `PassthroughConfig` type L47-84 — `= PassthroughConfig` — field stripping, and anthropic-beta header injection for MAX plan.
-  `PassthroughConfig` type L86-90 — `impl Default for PassthroughConfig` — field stripping, and anthropic-beta header injection for MAX plan.
-  `default` function L87-89 — `() -> Self` — field stripping, and anthropic-beta header injection for MAX plan.
-  `Passthrough` type L100-267 — `= Passthrough` — field stripping, and anthropic-beta header injection for MAX plan.
-  `get_auth_value` function L217-255 — `(&self, api_key: Option<&str>) -> Result<String>` — Get the authentication value based on the configured mode.
-  `prepare_raw_request` function L258-266 — `(&self, request: serde_json::Value) -> serde_json::Value` — Prepare a raw JSON request: strip unknown fields, inject system prompt.
-  `Passthrough` type L269-273 — `impl Default for Passthrough` — field stripping, and anthropic-beta header injection for MAX plan.
-  `default` function L270-272 — `() -> Self` — field stripping, and anthropic-beta header injection for MAX plan.
-  `VALID_REQUEST_FIELDS` variable L276-290 — `: &[&str]` — Valid top-level fields for Anthropic API requests.
-  `strip_unknown_fields` function L293-306 — `(request: &serde_json::Value) -> serde_json::Value` — Strip unknown fields from a raw JSON request.
-  `inject_system_prompt` function L309-341 — `(request: &mut serde_json::Value)` — Inject the required system prompt into a raw JSON request.
-  `tests` module L374-469 — `-` — field stripping, and anthropic-beta header injection for MAX plan.
-  `test_config_default_is_oauth` function L378-383 — `()` — field stripping, and anthropic-beta header injection for MAX plan.
-  `test_strip_unknown_fields` function L386-401 — `()` — field stripping, and anthropic-beta header injection for MAX plan.
-  `test_inject_system_prompt_empty` function L404-414 — `()` — field stripping, and anthropic-beta header injection for MAX plan.
-  `test_inject_system_prompt_prepend` function L417-435 — `()` — field stripping, and anthropic-beta header injection for MAX plan.
-  `test_inject_system_prompt_already_present` function L438-451 — `()` — field stripping, and anthropic-beta header injection for MAX plan.
-  `test_inject_system_prompt_string_format` function L454-468 — `()` — field stripping, and anthropic-beta header injection for MAX plan.

#### crates/arawn-oauth/src/proxy.rs

- pub `ProxyConfig` struct L25-30 — `{ bind_addr: SocketAddr, enable_cors: bool, passthrough: PassthroughConfig, toke...` — Configuration for the proxy server.
- pub `new` function L44-49 — `(bind_addr: SocketAddr) -> Self` — upstream with OAuth Bearer token authentication and request mangling.
- pub `with_token_manager` function L51-54 — `(mut self, manager: SharedTokenManager) -> Self` — upstream with OAuth Bearer token authentication and request mangling.
- pub `ProxyServer` struct L63-66 — `{ config: ProxyConfig, state: Arc<ProxyState> }` — The OAuth proxy server.
- pub `new` function L70-80 — `(config: ProxyConfig) -> Self` — Create a passthrough-only proxy.
- pub `router` function L83-99 — `(&self) -> AxumRouter` — Build the axum router.
- pub `run` function L102-107 — `(self) -> std::io::Result<()>` — Run the proxy server.
- pub `run_with_shutdown` function L110-124 — `( self, shutdown: impl std::future::Future<Output = ()> + Send + 'static, ) -> s...` — Run with graceful shutdown, returning the bound address.
- pub `ProxyError` struct L188 — `-` — Error type for proxy responses.
-  `ProxyConfig` type L32-41 — `impl Default for ProxyConfig` — upstream with OAuth Bearer token authentication and request mangling.
-  `default` function L33-40 — `() -> Self` — upstream with OAuth Bearer token authentication and request mangling.
-  `ProxyConfig` type L43-55 — `= ProxyConfig` — upstream with OAuth Bearer token authentication and request mangling.
-  `ProxyState` struct L58-60 — `{ passthrough: Passthrough }` — Shared state for the proxy server.
-  `ProxyServer` type L68-125 — `= ProxyServer` — upstream with OAuth Bearer token authentication and request mangling.
-  `handle_messages` function L128-176 — `( State(state): State<Arc<ProxyState>>, headers: HeaderMap, body: String, ) -> R...` — Handle POST /v1/messages
-  `handle_health` function L179-184 — `() -> impl IntoResponse` — Handle GET /health
-  `ProxyError` type L190-194 — `= ProxyError` — upstream with OAuth Bearer token authentication and request mangling.
-  `from` function L191-193 — `(err: OAuthError) -> Self` — upstream with OAuth Bearer token authentication and request mangling.
-  `ProxyError` type L196-224 — `impl IntoResponse for ProxyError` — upstream with OAuth Bearer token authentication and request mangling.
-  `into_response` function L197-223 — `(self) -> axum::response::Response` — upstream with OAuth Bearer token authentication and request mangling.
-  `tests` module L227-257 — `-` — upstream with OAuth Bearer token authentication and request mangling.
-  `test_health_endpoint` function L234-250 — `()` — upstream with OAuth Bearer token authentication and request mangling.
-  `test_proxy_config_default` function L253-256 — `()` — upstream with OAuth Bearer token authentication and request mangling.

#### crates/arawn-oauth/src/token_manager.rs

- pub `TOKEN_FILE` variable L16 — `: &str` — Default token file name within the arawn data directory.
- pub `TokenManager` interface L27-48 — `{ fn get_valid_access_token(), fn has_tokens(), fn save_tokens(), fn load_tokens...` — Trait for managing OAuth token lifecycle.
- pub `FileTokenManager` struct L69-73 — `{ token_path: PathBuf, config: OAuthConfig, cached_tokens: Arc<RwLock<Option<OAu...` — File-based token manager for production use.
- pub `new` function L77-83 — `(data_dir: &Path) -> Self` — Create a new file-based token manager.
- pub `with_path` function L86-92 — `(token_path: PathBuf) -> Self` — Create with a custom token path.
- pub `with_config` function L95-98 — `(mut self, config: OAuthConfig) -> Self` — Create with a custom OAuth config.
- pub `token_path` function L101-103 — `(&self) -> &Path` — Get the token file path.
- pub `is_token_expired` function L106-117 — `(tokens: &OAuthTokens) -> bool` — Check if tokens are expired (with buffer time).
- pub `InMemoryTokenManager` struct L239-242 — `{ tokens: RwLock<Option<OAuthTokens>>, refresh_count: std::sync::atomic::AtomicU...` — In-memory token manager for testing.
- pub `new` function L245-250 — `() -> Self` — Anthropic MAX plan authentication.
- pub `with_tokens` function L252-257 — `(tokens: OAuthTokens) -> Self` — Anthropic MAX plan authentication.
- pub `refresh_count` function L259-261 — `(&self) -> u32` — Anthropic MAX plan authentication.
- pub `TokenInfo` struct L349-354 — `{ created_at: String, expires_in_secs: u64, is_expired: bool, scope: String }` — Information about stored tokens for display.
- pub `expires_in_display` function L357-365 — `(&self) -> String` — Anthropic MAX plan authentication.
- pub `SharedTokenManager` type L373 — `= Arc<dyn TokenManager>` — Shared token manager for use across async contexts.
- pub `create_token_manager` function L376-378 — `(data_dir: &Path) -> SharedTokenManager` — Create a shared file-based token manager.
- pub `create_token_manager_with_config` function L381-386 — `( data_dir: &Path, config: OAuthConfig, ) -> SharedTokenManager` — Create a shared file-based token manager with a custom OAuth config.
- pub `create_memory_token_manager` function L389-391 — `() -> SharedTokenManager` — Create a shared in-memory token manager (for testing).
-  `REFRESH_BUFFER_MS` variable L19 — `: u64` — Buffer time before expiry to trigger refresh (5 minutes in milliseconds).
-  `FileTokenManager` type L75-118 — `= FileTokenManager` — Anthropic MAX plan authentication.
-  `FileTokenManager` type L121-231 — `impl TokenManager for FileTokenManager` — Anthropic MAX plan authentication.
-  `has_tokens` function L122-124 — `(&self) -> bool` — Anthropic MAX plan authentication.
-  `save_tokens` function L126-144 — `(&self, tokens: &OAuthTokens) -> Result<()>` — Anthropic MAX plan authentication.
-  `load_tokens` function L146-168 — `(&self) -> Result<Option<OAuthTokens>>` — Anthropic MAX plan authentication.
-  `get_valid_access_token` function L170-189 — `(&self) -> Result<String>` — Anthropic MAX plan authentication.
-  `clear_cache` function L191-194 — `(&self)` — Anthropic MAX plan authentication.
-  `delete_tokens` function L196-203 — `(&self) -> Result<()>` — Anthropic MAX plan authentication.
-  `get_token_info` function L205-230 — `(&self) -> Result<Option<TokenInfo>>` — Anthropic MAX plan authentication.
-  `InMemoryTokenManager` type L244-262 — `= InMemoryTokenManager` — Anthropic MAX plan authentication.
-  `InMemoryTokenManager` type L264-268 — `impl Default for InMemoryTokenManager` — Anthropic MAX plan authentication.
-  `default` function L265-267 — `() -> Self` — Anthropic MAX plan authentication.
-  `InMemoryTokenManager` type L271-341 — `impl TokenManager for InMemoryTokenManager` — Anthropic MAX plan authentication.
-  `has_tokens` function L272-277 — `(&self) -> bool` — Anthropic MAX plan authentication.
-  `save_tokens` function L279-283 — `(&self, tokens: &OAuthTokens) -> Result<()>` — Anthropic MAX plan authentication.
-  `load_tokens` function L285-288 — `(&self) -> Result<Option<OAuthTokens>>` — Anthropic MAX plan authentication.
-  `get_valid_access_token` function L290-303 — `(&self) -> Result<String>` — Anthropic MAX plan authentication.
-  `clear_cache` function L305-308 — `(&self)` — Anthropic MAX plan authentication.
-  `delete_tokens` function L310-313 — `(&self) -> Result<()>` — Anthropic MAX plan authentication.
-  `get_token_info` function L315-340 — `(&self) -> Result<Option<TokenInfo>>` — Anthropic MAX plan authentication.
-  `TokenInfo` type L356-366 — `= TokenInfo` — Anthropic MAX plan authentication.
-  `tests` module L394-534 — `-` — Anthropic MAX plan authentication.
-  `test_file_token_manager_new` function L399-403 — `()` — Anthropic MAX plan authentication.
-  `test_file_save_and_load_tokens` function L406-426 — `()` — Anthropic MAX plan authentication.
-  `test_is_token_expired` function L429-457 — `()` — Anthropic MAX plan authentication.
-  `test_file_delete_tokens` function L460-479 — `()` — Anthropic MAX plan authentication.
-  `test_inmemory_token_manager` function L482-507 — `()` — Anthropic MAX plan authentication.
-  `test_inmemory_no_tokens_error` function L510-514 — `()` — Anthropic MAX plan authentication.
-  `test_token_info_display` function L517-533 — `()` — Anthropic MAX plan authentication.

### crates/arawn-pipeline/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-pipeline/src/catalog.rs

- pub `RuntimeCategory` enum L26-29 — `Builtin | Custom` — Category of a runtime module.
- pub `CatalogEntry` struct L33-41 — `{ description: String, path: String, category: RuntimeCategory }` — A single runtime entry in the catalog.
- pub `RuntimeCatalog` struct L52-57 — `{ root: PathBuf, entries: BTreeMap<String, CatalogEntry> }` — In-memory runtime catalog with CRUD operations and persistence.
- pub `load` function L64-99 — `(root: &Path) -> Result<Self, PipelineError>` — Load or initialize a catalog from the given runtimes directory.
- pub `save` function L102-114 — `(&self) -> Result<(), PipelineError>` — Persist the current catalog to `catalog.toml`.
- pub `add` function L117-120 — `(&mut self, name: &str, entry: CatalogEntry) -> Result<(), PipelineError>` — Add or update a runtime entry and persist.
- pub `remove` function L123-127 — `(&mut self, name: &str) -> Result<Option<CatalogEntry>, PipelineError>` — Remove a runtime entry and persist.
- pub `get` function L130-132 — `(&self, name: &str) -> Option<&CatalogEntry>` — Get a runtime entry by name.
- pub `list` function L135-137 — `(&self) -> &BTreeMap<String, CatalogEntry>` — List all runtime entries.
- pub `resolve_path` function L140-142 — `(&self, name: &str) -> Option<PathBuf>` — Resolve the absolute path to a runtime's `.wasm` file.
- pub `root` function L145-147 — `(&self) -> &Path` — The root directory of the catalog.
-  `CatalogFile` struct L45-49 — `{ runtimes: BTreeMap<String, CatalogEntry> }` — Serialization wrapper for the catalog TOML file.
-  `RuntimeCatalog` type L59-148 — `= RuntimeCatalog` — ```
-  `tests` module L151-328 — `-` — ```
-  `test_load_creates_directories` function L156-162 — `()` — ```
-  `test_empty_catalog` function L165-170 — `()` — ```
-  `test_add_and_get` function L173-190 — `()` — ```
-  `test_remove` function L193-211 — `()` — ```
-  `test_remove_nonexistent` function L214-219 — `()` — ```
-  `test_list_returns_all` function L222-246 — `()` — ```
-  `test_roundtrip_persistence` function L249-280 — `()` — ```
-  `test_resolve_path` function L283-299 — `()` — ```
-  `test_add_overwrites` function L302-327 — `()` — ```

#### crates/arawn-pipeline/src/context.rs

- pub `ContextResolver` struct L23-25 — `{ data: &'a HashMap<String, Value> }` — Resolves `{{expression}}` templates against a context data map.
- pub `new` function L32-34 — `(data: &'a HashMap<String, Value>) -> Self` — Create a resolver backed by a context data map.
- pub `resolve_value` function L41-58 — `(&self, value: &Value) -> Result<Value, PipelineError>` — Resolve all `{{...}}` templates in a JSON value tree.
- pub `resolve_params` function L238-248 — `( params: &HashMap<String, Value>, context_data: &HashMap<String, Value>, ) -> R...` — Resolve all template expressions in a set of action parameters.
- pub `resolve_template_string` function L251-260 — `( template: &str, context_data: &HashMap<String, Value>, ) -> Result<String, Pip...` — Resolve template expressions in a single string (e.g., LLM prompt).
-  `resolve_string` function L67-89 — `(&self, s: &str) -> Result<Value, PipelineError>` — Resolve all `{{...}}` templates in a string.
-  `resolve_expression` function L92-120 — `(&self, path: &str) -> Result<Value, PipelineError>` — Resolve a single dot-separated path expression against the context.
-  `TemplateExpression` struct L129-134 — `{ full_match: String, path: String }` — A parsed `{{expression}}` occurrence in a string.
-  `parse_template_expressions` function L137-161 — `(s: &str) -> Vec<TemplateExpression>` — Find all `{{...}}` expressions in a string.
-  `PathSegment` struct L169-172 — `{ name: String, index: Option<usize> }` — A segment of a dot-separated path, optionally with an array index.
-  `PathSegment` type L174-181 — `= PathSegment` — LLM prompts, or tool parameters.
-  `fmt` function L175-180 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — LLM prompts, or tool parameters.
-  `parse_path_segments` function L186-203 — `(path: &str) -> Vec<PathSegment>` — Parse a dot-separated path into segments, handling array indices.
-  `navigate_segment` function L206-219 — `(value: &'a Value, segment: &PathSegment) -> Option<&'a Value>` — Navigate one segment of a path through a JSON value.
-  `value_to_string` function L222-231 — `(value: &Value) -> String` — Convert a JSON value to its string representation for template interpolation.
-  `tests` module L263-544 — `-` — LLM prompts, or tool parameters.
-  `test_context` function L267-291 — `() -> HashMap<String, Value>` — LLM prompts, or tool parameters.
-  `test_simple_field_access` function L294-299 — `()` — LLM prompts, or tool parameters.
-  `test_numeric_field` function L302-307 — `()` — LLM prompts, or tool parameters.
-  `test_nested_object_access` function L310-317 — `()` — LLM prompts, or tool parameters.
-  `test_array_index_access` function L320-327 — `()` — LLM prompts, or tool parameters.
-  `test_array_index_second_element` function L330-337 — `()` — LLM prompts, or tool parameters.
-  `test_full_output_object` function L340-346 — `()` — LLM prompts, or tool parameters.
-  `test_full_array_access` function L349-354 — `()` — LLM prompts, or tool parameters.
-  `test_string_template_preserves_type` function L357-363 — `()` — LLM prompts, or tool parameters.
-  `test_mixed_text_and_template` function L366-373 — `()` — LLM prompts, or tool parameters.
-  `test_multiple_templates_in_string` function L376-385 — `()` — LLM prompts, or tool parameters.
-  `test_no_templates` function L388-393 — `()` — LLM prompts, or tool parameters.
-  `test_missing_root_key_error` function L396-406 — `()` — LLM prompts, or tool parameters.
-  `test_missing_nested_field_error` function L409-416 — `()` — LLM prompts, or tool parameters.
-  `test_array_index_out_of_bounds` function L419-426 — `()` — LLM prompts, or tool parameters.
-  `test_resolve_value_object` function L429-441 — `()` — LLM prompts, or tool parameters.
-  `test_resolve_value_array` function L444-452 — `()` — LLM prompts, or tool parameters.
-  `test_resolve_value_nested_objects` function L455-465 — `()` — LLM prompts, or tool parameters.
-  `test_resolve_value_primitives_unchanged` function L468-474 — `()` — LLM prompts, or tool parameters.
-  `test_resolve_params_convenience` function L477-486 — `()` — LLM prompts, or tool parameters.
-  `test_resolve_template_string_convenience` function L489-494 — `()` — LLM prompts, or tool parameters.
-  `test_object_in_mixed_string_serialized` function L497-506 — `()` — LLM prompts, or tool parameters.
-  `test_boolean_in_mixed_string` function L509-517 — `()` — LLM prompts, or tool parameters.
-  `test_null_in_mixed_string` function L520-526 — `()` — LLM prompts, or tool parameters.
-  `test_whitespace_in_expression` function L529-535 — `()` — LLM prompts, or tool parameters.
-  `test_unclosed_brace_ignored` function L538-543 — `()` — LLM prompts, or tool parameters.

#### crates/arawn-pipeline/src/definition.rs

- pub `WorkflowFile` struct L48-50 — `{ workflow: WorkflowDefinition }` — Top-level wrapper matching the TOML structure `[workflow]`.
- pub `WorkflowDefinition` struct L70-92 — `{ name: String, description: String, tasks: Vec<TaskDefinition>, schedule: Optio...` — A complete declarative workflow definition.
- pub `TaskDefinition` struct L115-146 — `{ id: String, action: Option<ActionDefinition>, runtime: Option<String>, config:...` — A single task within a workflow.
- pub `effective_runtime` function L155-165 — `(&self) -> Option<&str>` — Returns the effective runtime name.
- pub `effective_config` function L170-189 — `(&self) -> serde_json::Value` — Returns the effective config value.
- pub `ActionDefinition` enum L195-222 — `Tool | Script | Llm` — What a task actually does.
- pub `Capabilities` struct L230-237 — `{ filesystem: Vec<String>, network: bool }` — WASI capability grants for sandboxed script execution.
- pub `ScheduleConfig` struct L241-247 — `{ cron: String, timezone: String }` — Cron/schedule configuration for a workflow.
- pub `RuntimeConfig` struct L255-262 — `{ timeout_secs: Option<u64>, max_retries: Option<u32> }` — Runtime configuration for a workflow.
- pub `TriggerConfig` struct L266-269 — `{ on_event: String }` — Trigger configuration for event-driven execution.
- pub `from_toml` function L277-280 — `(toml_str: &str) -> Result<Self, PipelineError>` — Parse a workflow definition from a TOML string.
- pub `from_file` function L283-288 — `(path: &Path) -> Result<Self, PipelineError>` — Load a workflow definition from a file path.
- pub `validate` function L304-372 — `(&self) -> Result<(), PipelineError>` — Validate the workflow definition.
- pub `ActionExecutorFactory` type L428 — `= Arc<dyn Fn(&str, &ActionDefinition) -> TaskFn + Send + Sync>` — Type alias for a factory that produces a `TaskFn` from an `ActionDefinition`.
- pub `to_dynamic_tasks` function L437-496 — `( &self, executor_factory: &ActionExecutorFactory, ) -> Result<Vec<DynamicTask>,...` — Convert this declarative definition into Cloacina `DynamicTask`s.
-  `TaskDefinition` type L148-190 — `= TaskDefinition` — ```
-  `default_script_language` function L224-226 — `() -> String` — ```
-  `default_timezone` function L249-251 — `() -> String` — ```
-  `WorkflowFile` type L275-289 — `= WorkflowFile` — ```
-  `WorkflowDefinition` type L295-417 — `= WorkflowDefinition` — ```
-  `detect_cycles` function L375-416 — `(&self) -> Result<(), PipelineError>` — Detect cycles in the task dependency graph using Kahn's algorithm.
-  `WorkflowDefinition` type L430-497 — `= WorkflowDefinition` — ```
-  `tests` module L500-913 — `-` — ```
-  `VALID_WORKFLOW` variable L503-527 — `: &str` — ```
-  `test_parse_valid_workflow` function L530-548 — `()` — ```
-  `test_parse_tool_action` function L551-567 — `()` — ```
-  `test_parse_script_action` function L570-593 — `()` — ```
-  `test_parse_llm_action` function L596-612 — `()` — ```
-  `test_validate_empty_name` function L615-625 — `()` — ```
-  `test_validate_no_tasks` function L628-636 — `()` — ```
-  `test_validate_duplicate_task_ids` function L639-653 — `()` — ```
-  `test_validate_unknown_dependency` function L656-668 — `()` — ```
-  `test_validate_cycle_detection` function L671-687 — `()` — ```
-  `test_validate_self_cycle` function L690-702 — `()` — ```
-  `test_validate_unsupported_script_language` function L705-716 — `()` — ```
-  `test_valid_workflow_validates` function L719-722 — `()` — ```
-  `test_to_dynamic_tasks` function L725-734 — `()` — ```
-  `test_to_dynamic_tasks_with_retry` function L737-755 — `()` — ```
-  `test_roundtrip_serialize` function L758-764 — `()` — ```
-  `test_minimal_workflow` function L767-780 — `()` — ```
-  `test_complex_dag` function L783-805 — `()` — ```
-  `test_invalid_toml_syntax` function L808-811 — `()` — ```
-  `test_parse_runtime_schema` function L816-831 — `()` — ```
-  `test_runtime_effective_methods` function L834-847 — `()` — ```
-  `test_legacy_effective_methods` function L850-862 — `()` — ```
-  `test_mixed_runtime_and_action_tasks` function L865-882 — `()` — ```
-  `test_task_with_neither_runtime_nor_action` function L885-895 — `()` — ```
-  `test_runtime_to_dynamic_tasks` function L898-912 — `()` — ```

#### crates/arawn-pipeline/src/engine.rs

- pub `PipelineConfig` struct L33-48 — `{ max_concurrent_tasks: usize, task_timeout_secs: u64, pipeline_timeout_secs: u6...` — Configuration for the pipeline engine.
- pub `ExecutionResult` struct L64-71 — `{ execution_id: String, status: ExecutionStatus, output: Option<serde_json::Valu...` — Result of a workflow execution.
- pub `ExecutionStatus` enum L75-84 — `Completed | Failed | Running | TimedOut` — Status of an execution.
- pub `ScheduleInfo` struct L88-97 — `{ id: String, workflow_name: String, cron_expr: String, enabled: bool }` — Information about a scheduled workflow.
- pub `PipelineEngine` struct L272-276 — `{ runner: DefaultRunner, workflows: Arc<RwLock<HashMap<String, Workflow>>> }` — The pipeline engine — Arawn's execution backbone.
- pub `new` function L285-307 — `(db_path: &Path, config: PipelineConfig) -> Result<Self, PipelineError>` — Initialize the pipeline engine with a SQLite database.
- pub `register_workflow` function L313-326 — `(&self, workflow: Workflow) -> Result<(), PipelineError>` — Register a dynamically constructed workflow.
- pub `register_dynamic_workflow` function L332-361 — `( &self, name: &str, description: &str, tasks: Vec<DynamicTask>, ) -> Result<(),...` — Build and register a workflow from dynamic tasks.
- pub `execute` function L367-408 — `( &self, workflow_name: &str, context: Context<serde_json::Value>, ) -> Result<E...` — Execute a registered workflow.
- pub `trigger` function L414-421 — `( &self, workflow_name: &str, context: Context<serde_json::Value>, ) -> Result<E...` — Execute a workflow via push trigger.
- pub `schedule_cron` function L430-454 — `( &self, workflow_name: &str, cron_expr: &str, timezone: &str, ) -> Result<Strin...` — Register a cron schedule for a workflow.
- pub `list_schedules` function L457-473 — `(&self) -> Result<Vec<ScheduleInfo>, PipelineError>` — List all cron schedules.
- pub `cancel_schedule` function L476-486 — `(&self, schedule_id: &str) -> Result<(), PipelineError>` — Cancel a cron schedule.
- pub `list_workflows` function L489-491 — `(&self) -> Vec<String>` — List registered workflow names.
- pub `has_workflow` function L494-496 — `(&self, name: &str) -> bool` — Check if a workflow is registered.
- pub `shutdown` function L501-511 — `(self) -> Result<(), PipelineError>` — Gracefully shut down the engine.
-  `PipelineConfig` type L50-60 — `impl Default for PipelineConfig` — cron scheduling, push triggers, and graceful shutdown.
-  `default` function L51-59 — `() -> Self` — cron scheduling, push triggers, and graceful shutdown.
-  `tests` module L100-253 — `-` — cron scheduling, push triggers, and graceful shutdown.
-  `test_engine` function L104-112 — `(dir: &Path) -> PipelineEngine` — cron scheduling, push triggers, and graceful shutdown.
-  `test_pipeline_config_defaults` function L115-122 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_execution_status_eq` function L125-137 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_has_workflow_false_initially` function L140-145 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_list_workflows_empty` function L148-153 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_execute_missing_workflow` function L156-167 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_trigger_missing_workflow` function L170-177 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_schedule_cron_missing_workflow` function L180-192 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_cancel_schedule_invalid_uuid` function L195-205 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_cancel_schedule_nonexistent_uuid` function L208-218 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_register_and_has_workflow` function L221-238 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `test_register_empty_tasks` function L241-252 — `()` — cron scheduling, push triggers, and graceful shutdown.
-  `PipelineEngine` type L278-512 — `= PipelineEngine` — cron scheduling, push triggers, and graceful shutdown.

#### crates/arawn-pipeline/src/error.rs

- pub `Result` type L6 — `= std::result::Result<T, PipelineError>` — Result type for pipeline operations.
- pub `PipelineError` enum L10-46 — `InitFailed | WorkflowNotFound | ExecutionFailed | InvalidWorkflow | SchedulingEr...` — Errors that can occur during pipeline operations.
-  `PipelineError` type L48-52 — `= PipelineError` — Error types for the pipeline engine.
-  `from` function L49-51 — `(err: cloacina::PipelineError) -> Self` — Error types for the pipeline engine.

#### crates/arawn-pipeline/src/factory.rs

- pub `build_executor_factory` function L24-120 — `( executor: Arc<ScriptExecutor>, catalog: Arc<RwLock<RuntimeCatalog>>, ) -> Acti...` — Build an `ActionExecutorFactory` that dispatches to WASM runtimes via
-  `tests` module L123-277 — `-` — definitions to WASM runtime execution via `ScriptExecutor`.
-  `setup_with_passthrough` function L132-168 — `() -> (Arc<ScriptExecutor>, Arc<RwLock<RuntimeCatalog>>, TempDir)` — Helper: set up executor, compile a simple passthrough wasm, register in catalog.
-  `can_compile_wasm` function L170-177 — `() -> bool` — definitions to WASM runtime execution via `ScriptExecutor`.
-  `test_factory_produces_working_task_fn` function L180-203 — `()` — definitions to WASM runtime execution via `ScriptExecutor`.
-  `test_factory_context_propagation` function L206-244 — `()` — definitions to WASM runtime execution via `ScriptExecutor`.
-  `test_factory_unknown_runtime_error` function L247-276 — `()` — definitions to WASM runtime execution via `ScriptExecutor`.

#### crates/arawn-pipeline/src/lib.rs

- pub `catalog` module L19 — `-` — This crate provides the `PipelineEngine` — Arawn's execution backbone for
- pub `context` module L20 — `-` — ```
- pub `definition` module L21 — `-` — ```
- pub `engine` module L22 — `-` — ```
- pub `error` module L23 — `-` — ```
- pub `factory` module L24 — `-` — ```
- pub `loader` module L25 — `-` — ```
- pub `protocol` module L26 — `-` — ```
- pub `sandbox` module L27 — `-` — ```
- pub `task` module L28 — `-` — ```

#### crates/arawn-pipeline/src/loader.rs

- pub `WorkflowEvent` enum L21-28 — `Loaded | Removed | Error` — Event emitted when workflow files change.
- pub `WorkflowLoader` struct L39-46 — `{ workflow_dir: PathBuf, workflows: Arc<RwLock<HashMap<String, LoadedWorkflow>>>...` — Manages loading and hot-reloading of workflow TOML files from a directory.
- pub `new` function L52-81 — `(workflow_dir: impl Into<PathBuf>) -> Result<Self, PipelineError>` — Create a new loader for the given workflow directory.
- pub `load_all` function L87-118 — `(&self) -> Vec<WorkflowEvent>` — Load all TOML workflow files from the directory.
- pub `get` function L200-206 — `(&self, name: &str) -> Option<crate::definition::WorkflowDefinition>` — Get a loaded workflow definition by name.
- pub `list_names` function L209-211 — `(&self) -> Vec<String>` — List all loaded workflow names.
- pub `len` function L214-216 — `(&self) -> usize` — Get the number of loaded workflows.
- pub `is_empty` function L219-221 — `(&self) -> bool` — Check if any workflows are loaded.
- pub `watch` function L230-317 — `( &self, ) -> Result<(tokio::sync::mpsc::Receiver<WorkflowEvent>, WatcherHandle)...` — Start watching the workflow directory for changes.
- pub `WatcherHandle` struct L394-396 — `{ _thread: std::thread::JoinHandle<()> }` — Handle that keeps the file watcher alive.
-  `LoadedWorkflow` struct L33-36 — `{ definition: crate::definition::WorkflowDefinition, path: PathBuf }` — In-memory cache of loaded workflow definitions.
-  `WorkflowLoader` type L48-323 — `= WorkflowLoader` — picked up without restarting the server.
-  `normalize_path` function L125-131 — `(&self, path: &Path) -> PathBuf` — Normalize a path to use the canonical `workflow_dir` prefix.
-  `load_file` function L134-178 — `(&self, path: &Path) -> WorkflowEvent` — Load or reload a single workflow file.
-  `remove_file` function L182-197 — `(&self, path: &Path) -> Option<WorkflowEvent>` — Handle a file being removed.
-  `is_workflow_file` function L320-322 — `(path: &Path) -> bool` — Check if a path is a workflow TOML file.
-  `WorkflowLoaderView` struct L326-329 — `{ workflows: Arc<RwLock<HashMap<String, LoadedWorkflow>>>, path_to_name: Arc<RwL...` — Internal view used by the watcher thread to update workflow state.
-  `WorkflowLoaderView` type L331-389 — `= WorkflowLoaderView` — picked up without restarting the server.
-  `load_file` function L332-372 — `(&self, path: &Path) -> WorkflowEvent` — picked up without restarting the server.
-  `remove_file` function L374-388 — `(&self, path: &Path) -> Option<WorkflowEvent>` — picked up without restarting the server.
-  `tests` module L399-654 — `-` — picked up without restarting the server.
-  `write_workflow` function L402-415 — `(dir: &Path, filename: &str, name: &str)` — picked up without restarting the server.
-  `write_invalid` function L417-419 — `(dir: &Path, filename: &str)` — picked up without restarting the server.
-  `test_load_empty_directory` function L422-428 — `()` — picked up without restarting the server.
-  `test_load_single_workflow` function L431-444 — `()` — picked up without restarting the server.
-  `test_load_multiple_workflows` function L447-461 — `()` — picked up without restarting the server.
-  `test_invalid_file_doesnt_crash` function L464-482 — `()` — picked up without restarting the server.
-  `test_skips_non_toml_files` function L485-496 — `()` — picked up without restarting the server.
-  `test_creates_directory_if_missing` function L499-506 — `()` — picked up without restarting the server.
-  `test_reload_modified_file` function L509-523 — `()` — picked up without restarting the server.
-  `test_remove_file` function L526-537 — `()` — picked up without restarting the server.
-  `test_get_nonexistent` function L540-544 — `()` — picked up without restarting the server.
-  `test_watch_detects_new_file` function L548-574 — `()` — picked up without restarting the server.
-  `test_watch_detects_modified_file` function L578-604 — `()` — picked up without restarting the server.
-  `test_watch_detects_deleted_file` function L608-634 — `()` — picked up without restarting the server.
-  `test_watch_ignores_non_toml` function L638-653 — `()` — picked up without restarting the server.

#### crates/arawn-pipeline/src/protocol.rs

- pub `RuntimeInput` struct L11-16 — `{ config: Value, context: Value }` — Input envelope sent to a WASM runtime on stdin.
- pub `RuntimeOutput` struct L20-29 — `{ status: String, output: Option<Value>, error: Option<String> }` — Output envelope expected from a WASM runtime on stdout.
- pub `is_ok` function L33-35 — `(&self) -> bool` — Returns true if the runtime reported success.
-  `RuntimeOutput` type L31-36 — `= RuntimeOutput` — and writes a `RuntimeOutput` to stdout, both as JSON.
-  `tests` module L39-89 — `-` — and writes a `RuntimeOutput` to stdout, both as JSON.
-  `test_runtime_input_roundtrip` function L44-53 — `()` — and writes a `RuntimeOutput` to stdout, both as JSON.
-  `test_runtime_output_ok` function L56-65 — `()` — and writes a `RuntimeOutput` to stdout, both as JSON.
-  `test_runtime_output_error` function L68-77 — `()` — and writes a `RuntimeOutput` to stdout, both as JSON.
-  `test_runtime_output_minimal` function L80-88 — `()` — and writes a `RuntimeOutput` to stdout, both as JSON.

#### crates/arawn-pipeline/src/sandbox.rs

- pub `ScriptExecutor` struct L30-39 — `{ engine: Engine, cache_dir: PathBuf, module_cache: Arc<RwLock<HashMap<String, M...` — Manages compilation and sandboxed execution of Rust scripts as WASM modules.
- pub `CompileResult` struct L43-52 — `{ source_hash: String, wasm_path: PathBuf, cached: bool, compile_time: Duration ...` — Result of compiling a Rust source file to WASM.
- pub `ScriptOutput` struct L56-65 — `{ stdout: String, stderr: String, exit_code: i32, elapsed: Duration }` — Result of executing a WASM module.
- pub `ScriptConfig` struct L69-76 — `{ capabilities: Capabilities, timeout: Option<Duration>, max_memory_bytes: Optio...` — Configuration for a single script execution.
- pub `new` function L93-110 — `(cache_dir: PathBuf, default_timeout: Duration) -> Result<Self, PipelineError>` — Create a new executor with the given cache directory and default timeout.
- pub `compile` function L116-185 — `(&self, source: &str) -> Result<CompileResult, PipelineError>` — Compile Rust source code to a WASM module targeting `wasm32-wasip1`.
- pub `compile_crate` function L191-254 — `(&self, crate_dir: &Path) -> Result<PathBuf, PipelineError>` — Compile an entire Cargo crate to `wasm32-wasip1` and return the `.wasm` path.
- pub `execute` function L259-285 — `( &self, source_hash: &str, context_json: &str, config: &ScriptConfig, ) -> Resu...` — Execute a previously compiled WASM module with the given context and capabilities.
- pub `compile_and_execute` function L288-299 — `( &self, source: &str, context_json: &str, config: &ScriptConfig, ) -> Result<(C...` — Compile and execute in one call.
- pub `clear_cache` function L302-304 — `(&self)` — Clear the in-memory module cache.
- pub `execute_runtime` function L310-384 — `( &self, name: &str, input: &RuntimeInput, catalog: &RuntimeCatalog, ) -> Result...` — Execute a named runtime from the catalog with the given input.
-  `ScriptConfig` type L78-89 — `impl Default for ScriptConfig` — 5.
-  `default` function L79-88 — `() -> Self` — 5.
-  `ScriptExecutor` type L91-500 — `= ScriptExecutor` — 5.
-  `check_wasm_target` function L387-405 — `() -> Result<(), PipelineError>` — Check if the `wasm32-wasip1` target is installed.
-  `execute_sync` function L408-499 — `( engine: &Engine, module: &Module, context_json: &str, capabilities: &Capabilit...` — Synchronous WASM execution with Wasmtime + WASI Preview 1.
-  `sha256_hex` function L503-507 — `(input: &str) -> String` — Compute SHA-256 hex digest of a string.
-  `tests` module L510-901 — `-` — 5.
-  `test_executor` function L514-519 — `() -> (ScriptExecutor, TempDir)` — 5.
-  `test_sha256_deterministic` function L522-527 — `()` — 5.
-  `test_sha256_different_inputs` function L530-534 — `()` — 5.
-  `test_executor_creation` function L537-540 — `()` — 5.
-  `test_default_script_config` function L543-549 — `()` — 5.
-  `test_compile_simple_rust` function L552-568 — `()` — 5.
-  `test_compile_cache_hit` function L571-589 — `()` — 5.
-  `test_compile_error_returned` function L592-609 — `()` — 5.
-  `test_execute_simple_script` function L612-632 — `()` — 5.
-  `test_execute_reads_stdin_context` function L635-665 — `()` — 5.
-  `test_execute_nonexistent_hash` function L668-679 — `()` — 5.
-  `test_execute_exit_code` function L682-701 — `()` — 5.
-  `test_execute_runtime_unknown_name` function L704-716 — `()` — 5.
-  `test_execute_runtime_missing_wasm` function L719-739 — `()` — 5.
-  `test_execute_runtime_passthrough` function L742-815 — `()` — 5.
-  `test_execute_runtime_caches_module` function L818-871 — `()` — 5.
-  `test_clear_cache` function L874-900 — `()` — 5.

#### crates/arawn-pipeline/src/task.rs

- pub `TaskFn` type L21-31 — `= Arc< dyn Fn( Context<serde_json::Value>, ) -> Pin< Box< dyn Future<Output = st...` — Type alias for the async function that executes a dynamic task.
- pub `DynamicTask` struct L38-43 — `{ id: String, dependencies: Vec<TaskNamespace>, retry_policy: RetryPolicy, execu...` — A task that can be constructed at runtime without macros.
- pub `new` function L52-59 — `(id: impl Into<String>, execute_fn: TaskFn) -> Self` — Create a new dynamic task.
- pub `with_dependency` function L62-65 — `(mut self, dep: TaskNamespace) -> Self` — Add a dependency on another task by its namespace.
- pub `with_dependency_id` function L71-80 — `(mut self, task_id: &str) -> Self` — Add a dependency on another task by its short ID within the same workflow.
- pub `with_retry_policy` function L93-96 — `(mut self, policy: RetryPolicy) -> Self` — Set the retry policy for this task.
-  `DynamicTask` type L45-97 — `= DynamicTask` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `resolve_workflow_name` function L83-90 — `(mut self, workflow_name: &str) -> Self` — Resolve pending dependency namespaces with the actual workflow name.
-  `DynamicTask` type L99-106 — `= DynamicTask` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `fmt` function L100-105 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `tests` module L109-247 — `-` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `noop_fn` function L113-115 — `() -> TaskFn` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `failing_fn` function L117-127 — `() -> TaskFn` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_new_task_id` function L130-133 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_new_task_no_dependencies` function L136-139 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_new_task_default_retry_policy` function L142-146 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_with_dependency` function L149-154 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_with_multiple_dependencies` function L157-162 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_with_dependency_id_uses_pending` function L165-169 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_resolve_workflow_name` function L172-178 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_resolve_preserves_non_pending` function L181-191 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_with_retry_policy` function L194-206 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_execute_success` function L209-224 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_execute_failure` function L227-232 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_debug_format` function L235-240 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `test_string_id_conversion` function L243-246 — `()` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `DynamicTask` type L250-269 — `impl Task for DynamicTask` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `execute` function L251-256 — `( &self, context: Context<serde_json::Value>, ) -> std::result::Result<Context<s...` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `id` function L258-260 — `(&self) -> &str` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `dependencies` function L262-264 — `(&self) -> &[TaskNamespace]` — declarative workflow definitions (TOML files) and Cloacina's execution engine.
-  `retry_policy` function L266-268 — `(&self) -> RetryPolicy` — declarative workflow definitions (TOML files) and Cloacina's execution engine.

### crates/arawn-pipeline/tests

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-pipeline/tests/e2e_runtime_test.rs

-  `can_compile_wasm` function L15-21 — `() -> bool` — Test 2: Agent self-extension — compile, register, and execute a custom runtime.
-  `setup` function L28-123 — `() -> ( Arc<PipelineEngine>, Arc<ScriptExecutor>, Arc<RwLock<RuntimeCatalog>>, t...` — Set up executor + catalog with compiled test runtimes.
-  `test_multistep_workflow_context_propagation` function L132-205 — `()` — Test 1: Multi-step workflow with context propagation.
-  `test_agent_self_extension` function L210-311 — `()` — Test 2: Agent self-extension — compile a custom runtime, register it,
-  `test_workflow_unknown_runtime_error` function L315-370 — `()` — Test 3: Verify unknown runtime produces a clear error.

#### crates/arawn-pipeline/tests/engine_test.rs

-  `test_engine` function L10-20 — `(dir: &Path) -> PipelineEngine` — Helper to create an engine with a temp database.
-  `test_engine_init_shutdown` function L23-29 — `()` — Integration tests for PipelineEngine.
-  `test_register_and_list_workflows` function L32-49 — `()` — Integration tests for PipelineEngine.
-  `test_execute_simple_workflow` function L52-93 — `()` — Integration tests for PipelineEngine.
-  `test_execute_nonexistent_workflow` function L96-105 — `()` — Integration tests for PipelineEngine.
-  `test_trigger_is_execute` function L108-124 — `()` — Integration tests for PipelineEngine.
-  `test_dynamic_task_with_dependencies` function L127-166 — `()` — Integration tests for PipelineEngine.

### crates/arawn-plugin/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-plugin/src/agent_spawner.rs

- pub `AgentSpawner` struct L184-191 — `{ parent_tools: Arc<ToolRegistry>, backend: SharedBackend, default_max_iteration...` — Spawns agents from plugin agent configurations.
- pub `new` function L195-201 — `(parent_tools: Arc<ToolRegistry>, backend: SharedBackend) -> Self` — Create a new agent spawner.
- pub `with_default_max_iterations` function L207-210 — `(mut self, max_iterations: u32) -> Self` — Create a new agent spawner with a default max_iterations.
- pub `spawn` function L223-259 — `(&self, config: &PluginAgentConfig) -> Result<Agent>` — Spawn an agent from a plugin agent configuration.
- pub `PluginSubagentSpawner` struct L309-322 — `{ spawner: AgentSpawner, agent_configs: HashMap<String, PluginAgentConfig>, agen...` — A subagent spawner backed by plugin-defined agent configurations.
- pub `new` function L331-344 — `( parent_tools: Arc<ToolRegistry>, backend: SharedBackend, agent_configs: HashMa...` — Create a new plugin subagent spawner.
- pub `with_sources` function L350-364 — `( parent_tools: Arc<ToolRegistry>, backend: SharedBackend, agent_configs: HashMa...` — Create a spawner with source plugin tracking.
- pub `with_hook_dispatcher` function L370-373 — `(mut self, dispatcher: SharedHookDispatcher) -> Self` — Set the hook dispatcher for subagent lifecycle events.
- pub `with_compaction` function L379-383 — `(mut self, backend: SharedBackend, config: CompactionConfig) -> Self` — Set the compaction backend and configuration.
- pub `with_default_max_iterations` function L389-392 — `(mut self, max_iterations: u32) -> Self` — Set the default max_iterations for all spawned agents.
- pub `agent_count` function L395-397 — `(&self) -> usize` — Get the number of available agents.
- pub `is_empty` function L400-402 — `(&self) -> bool` — Check if any agents are available.
- pub `agent_names` function L405-407 — `(&self) -> Vec<&str>` — Get the names of all available agents.
-  `DEFAULT_MAX_CONTEXT_LEN` variable L28 — `: usize` — Default maximum length for context passed to subagents (in characters).
-  `DEFAULT_MAX_RESULT_LEN` variable L31 — `: usize` — Default maximum length for subagent results (in characters).
-  `truncate_context` function L34-45 — `(context: &str, max_len: usize) -> String` — Truncate context to a maximum length, preserving word boundaries where possible.
-  `TruncatedResult` struct L48-55 — `{ text: String, truncated: bool, original_len: Option<usize> }` — Result of truncating a subagent response.
-  `truncate_result` function L61-100 — `(text: &str, max_len: usize) -> TruncatedResult` — Truncate a subagent result, preserving beginning and end of the response.
-  `COMPACTION_SYSTEM_PROMPT` variable L103-126 — `: &str` — System prompt for context compaction.
-  `CompactionResult` struct L129-136 — `{ text: String, success: bool, original_len: usize }` — Result of compacting a subagent response.
-  `compact_result` function L139-181 — `( text: &str, backend: &SharedBackend, model: &str, target_len: usize, ) -> Comp...` — Compact a long subagent result using LLM summarization.
-  `AgentSpawner` type L193-284 — `= AgentSpawner` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `constrain_tools` function L262-283 — `(&self, config: &PluginAgentConfig) -> ToolRegistry` — Create a constrained tool registry from the parent's tools.
-  `PluginSubagentSpawner` type L324-408 — `= PluginSubagentSpawner` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `PluginSubagentSpawner` type L411-680 — `impl SubagentSpawner for PluginSubagentSpawner` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `list_agents` function L412-431 — `(&self) -> Vec<SubagentInfo>` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `delegate` function L433-555 — `( &self, agent_name: &str, task: &str, context: Option<&str>, max_turns: Option<...` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `delegate_background` function L557-675 — `( &self, agent_name: &str, task: &str, context: Option<&str>, parent_session_id:...` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `has_agent` function L677-679 — `(&self, name: &str) -> bool` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `tests` module L683-1114 — `-` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `TestTool` struct L690-692 — `{ tool_name: String }` — A simple test tool.
-  `TestTool` type L694-700 — `= TestTool` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `new` function L695-699 — `(name: &str) -> Self` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `TestTool` type L703-722 — `impl Tool for TestTool` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `name` function L704-706 — `(&self) -> &str` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `description` function L707-709 — `(&self) -> &str` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `parameters` function L710-712 — `(&self) -> serde_json::Value` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `execute` function L713-721 — `( &self, _params: serde_json::Value, _ctx: &ToolContext, ) -> arawn_agent::error...` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `make_parent_tools` function L724-730 — `() -> Arc<ToolRegistry>` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `make_agent_config` function L732-753 — `( name: &str, tools: Vec<&str>, max_iter: Option<usize>, ) -> PluginAgentConfig` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_spawn_agent_with_constrained_tools` function L756-768 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_spawn_agent_missing_tool_skipped` function L771-782 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_spawn_agent_max_iterations` function L785-794 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_spawn_agent_system_prompt` function L797-807 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_spawn_agent_no_constraints` function L810-828 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_plugin_subagent_spawner_list_agents` function L833-863 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_plugin_subagent_spawner_has_agent` function L866-880 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_plugin_subagent_spawner_delegate_unknown_agent` function L883-898 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_plugin_subagent_spawner_agent_count` function L901-917 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_plugin_subagent_spawner_empty` function L920-929 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_context_short` function L934-938 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_context_exact_limit` function L941-945 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_context_over_limit` function L948-954 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_context_word_boundary` function L957-963 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_context_no_spaces` function L966-971 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_result_short` function L976-982 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_result_exact_limit` function L985-991 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_result_over_limit` function L994-1008 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_result_preserves_beginning_and_end` function L1011-1026 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_result_metadata` function L1029-1042 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_truncate_result_word_boundaries` function L1045-1060 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_compact_result_success` function L1065-1080 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_compaction_config_default` function L1083-1090 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`
-  `test_spawner_with_compaction` function L1093-1113 — `()` — - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`

#### crates/arawn-plugin/src/hooks.rs

- pub `HookDispatcher` struct L32-37 — `{ hooks: HashMap<HookEvent, Vec<CompiledHook>>, timeout: Duration }` — Dispatches hooks at lifecycle events.
- pub `new` function L41-46 — `() -> Self` — Create an empty dispatcher.
- pub `with_timeout` function L49-52 — `(mut self, timeout: Duration) -> Self` — Set the subprocess timeout.
- pub `register` function L55-90 — `(&mut self, def: HookDef, plugin_dir: PathBuf)` — Register a hook from a plugin.
- pub `len` function L93-95 — `(&self) -> usize` — Get the number of registered hooks.
- pub `is_empty` function L98-100 — `(&self) -> bool` — Check if the dispatcher has no hooks.
- pub `count_for_event` function L103-105 — `(&self, event: HookEvent) -> usize` — Get the number of hooks for a specific event.
- pub `dispatch_pre_tool_use` function L110-126 — `( &self, tool_name: &str, params: &serde_json::Value, ) -> HookOutcome` — Dispatch hooks for a PreToolUse event.
- pub `dispatch_post_tool_use` function L129-147 — `( &self, tool_name: &str, params: &serde_json::Value, result: &serde_json::Value...` — Dispatch hooks for a PostToolUse event.
- pub `dispatch_session_start` function L150-154 — `(&self, session_id: &str) -> HookOutcome` — Dispatch hooks for a SessionStart event.
- pub `dispatch_session_end` function L157-164 — `(&self, session_id: &str, turn_count: usize) -> HookOutcome` — Dispatch hooks for a SessionEnd event.
- pub `dispatch_stop` function L167-171 — `(&self, response: &str) -> HookOutcome` — Dispatch hooks for a Stop event.
- pub `dispatch_subagent_started` function L174-187 — `( &self, parent_session_id: &str, subagent_name: &str, task_preview: &str, ) -> ...` — Dispatch hooks for a SubagentStarted event.
- pub `dispatch_subagent_completed` function L190-207 — `( &self, parent_session_id: &str, subagent_name: &str, result_preview: &str, dur...` — Dispatch hooks for a SubagentCompleted event.
- pub `register_from_config` function L340-377 — `( &mut self, config: &crate::HooksConfig, plugin_dir: &std::path::Path, )` — Register hooks from a Claude-format `HooksConfig`.
-  `DEFAULT_HOOK_TIMEOUT` variable L15 — `: Duration` — Default timeout for hook subprocesses.
-  `CompiledHook` struct L19-28 — `{ def: HookDef, tool_pattern: Option<glob::Pattern>, param_regex: Option<regex::...` — A compiled hook ready for matching and execution.
-  `HookDispatcher` type L39-324 — `= HookDispatcher` — (PreToolUse) or provide informational side effects.
-  `dispatch_blocking` function L210-267 — `( &self, event: HookEvent, context: &C, tool_name: Option<&str>, params: Option<...` — Dispatch hooks that can block (PreToolUse).
-  `dispatch_info` function L270-323 — `( &self, event: HookEvent, context: &C, tool_name: Option<&str>, params: Option<...` — Dispatch informational hooks (PostToolUse, SessionStart, SessionEnd, Stop).
-  `HookDispatcher` type L326-330 — `impl Default for HookDispatcher` — (PreToolUse) or provide informational side effects.
-  `default` function L327-329 — `() -> Self` — (PreToolUse) or provide informational side effects.
-  `HookDispatcher` type L332-378 — `= HookDispatcher` — (PreToolUse) or provide informational side effects.
-  `HookDispatcher` type L385-456 — `impl HookDispatch for HookDispatcher` — Implement the HookDispatch trait for HookDispatcher.
-  `dispatch_pre_tool_use` function L386-392 — `( &self, tool_name: &str, params: &serde_json::Value, ) -> HookOutcome` — (PreToolUse) or provide informational side effects.
-  `dispatch_post_tool_use` function L394-401 — `( &self, tool_name: &str, params: &serde_json::Value, result: &serde_json::Value...` — (PreToolUse) or provide informational side effects.
-  `dispatch_session_start` function L403-405 — `(&self, session_id: &str) -> HookOutcome` — (PreToolUse) or provide informational side effects.
-  `dispatch_session_end` function L407-409 — `(&self, session_id: &str, turn_count: usize) -> HookOutcome` — (PreToolUse) or provide informational side effects.
-  `dispatch_stop` function L411-413 — `(&self, response: &str) -> HookOutcome` — (PreToolUse) or provide informational side effects.
-  `dispatch_subagent_started` function L415-428 — `( &self, parent_session_id: &str, subagent_name: &str, task_preview: &str, ) -> ...` — (PreToolUse) or provide informational side effects.
-  `dispatch_subagent_completed` function L430-447 — `( &self, parent_session_id: &str, subagent_name: &str, result_preview: &str, dur...` — (PreToolUse) or provide informational side effects.
-  `len` function L449-451 — `(&self) -> usize` — (PreToolUse) or provide informational side effects.
-  `is_empty` function L453-455 — `(&self) -> bool` — (PreToolUse) or provide informational side effects.
-  `PreToolUseContext` struct L463-466 — `{ tool: &'a str, params: &'a serde_json::Value }` — (PreToolUse) or provide informational side effects.
-  `PostToolUseContext` struct L469-473 — `{ tool: &'a str, params: &'a serde_json::Value, result: &'a serde_json::Value }` — (PreToolUse) or provide informational side effects.
-  `SessionContext` struct L476-478 — `{ session_id: &'a str }` — (PreToolUse) or provide informational side effects.
-  `SessionEndContext` struct L481-484 — `{ session_id: &'a str, turn_count: usize }` — (PreToolUse) or provide informational side effects.
-  `StopContext` struct L487-489 — `{ response: &'a str }` — (PreToolUse) or provide informational side effects.
-  `SubagentStartedContext` struct L492-496 — `{ parent_session_id: &'a str, subagent_name: &'a str, task_preview: &'a str }` — (PreToolUse) or provide informational side effects.
-  `SubagentCompletedContext` struct L499-505 — `{ parent_session_id: &'a str, subagent_name: &'a str, result_preview: &'a str, d...` — (PreToolUse) or provide informational side effects.
-  `matches_hook` function L511-542 — `( hook: &CompiledHook, tool_name: Option<&str>, params: Option<&serde_json::Valu...` — (PreToolUse) or provide informational side effects.
-  `HookRunResult` enum L548-555 — `Success | Blocked | Error` — (PreToolUse) or provide informational side effects.
-  `run_hook_command` function L557-607 — `( command: &std::path::Path, plugin_dir: &std::path::Path, stdin_data: &str, tim...` — (PreToolUse) or provide informational side effects.
-  `tests` module L610-1021 — `-` — (PreToolUse) or provide informational side effects.
-  `create_hook_script` function L616-621 — `(dir: &std::path::Path, name: &str, script: &str) -> PathBuf` — (PreToolUse) or provide informational side effects.
-  `make_hook` function L623-630 — `(event: HookEvent, command: PathBuf) -> HookDef` — (PreToolUse) or provide informational side effects.
-  `test_pre_tool_use_allow` function L633-647 — `()` — (PreToolUse) or provide informational side effects.
-  `test_pre_tool_use_block` function L650-673 — `()` — (PreToolUse) or provide informational side effects.
-  `test_tool_match_glob` function L676-696 — `()` — (PreToolUse) or provide informational side effects.
-  `test_match_pattern_regex` function L699-719 — `()` — (PreToolUse) or provide informational side effects.
-  `test_session_start_info` function L722-743 — `()` — (PreToolUse) or provide informational side effects.
-  `test_session_end` function L746-758 — `()` — (PreToolUse) or provide informational side effects.
-  `test_stop_hook` function L761-779 — `()` — (PreToolUse) or provide informational side effects.
-  `test_no_hooks_registered` function L782-788 — `()` — (PreToolUse) or provide informational side effects.
-  `test_post_tool_use` function L791-812 — `()` — (PreToolUse) or provide informational side effects.
-  `test_hook_receives_stdin` function L815-837 — `()` — (PreToolUse) or provide informational side effects.
-  `test_dispatcher_len` function L840-862 — `()` — (PreToolUse) or provide informational side effects.
-  `test_hook_timeout` function L865-879 — `()` — (PreToolUse) or provide informational side effects.
-  `test_matches_hook_no_filters` function L882-896 — `()` — (PreToolUse) or provide informational side effects.
-  `test_matches_hook_tool_pattern_no_tool_name` function L899-908 — `()` — (PreToolUse) or provide informational side effects.
-  `test_subagent_started_event` function L911-937 — `()` — (PreToolUse) or provide informational side effects.
-  `test_subagent_completed_event` function L940-974 — `()` — (PreToolUse) or provide informational side effects.
-  `test_subagent_completed_failure_event` function L977-1004 — `()` — (PreToolUse) or provide informational side effects.
-  `test_subagent_events_no_hooks_registered` function L1007-1020 — `()` — (PreToolUse) or provide informational side effects.

#### crates/arawn-plugin/src/lib.rs

- pub `agent_spawner` module L24 — `-` — Plugins bundle skills, hooks, agents, and prompt fragments together with a
- pub `hooks` module L25 — `-` — ```
- pub `manager` module L26 — `-` — ```
- pub `manifest` module L27 — `-` — ```
- pub `skill` module L28 — `-` — ```
- pub `subscription` module L29 — `-` — ```
- pub `types` module L30 — `-` — ```
- pub `validation` module L31 — `-` — ```
- pub `watcher` module L32 — `-` — ```
- pub `PluginError` enum L51-67 — `ManifestParse | Validation | Io | AgentConfigParse` — Plugin error type.
- pub `Result` type L70 — `= std::result::Result<T, PluginError>` — Result type for plugin operations.
- pub `CLAUDE_PLUGIN_ROOT_VAR` variable L76 — `: &str` — The environment variable name for the plugin root directory.
- pub `expand_plugin_root` function L94-96 — `(s: &str, plugin_dir: &std::path::Path) -> String` — Expand `${CLAUDE_PLUGIN_ROOT}` in a string to the actual plugin directory path.
- pub `expand_plugin_root_path` function L99-109 — `( path: &std::path::Path, plugin_dir: &std::path::Path, ) -> std::path::PathBuf` — Expand `${CLAUDE_PLUGIN_ROOT}` in a PathBuf.
-  `tests` module L112-173 — `-` — ```
-  `test_expand_plugin_root` function L117-124 — `()` — ```
-  `test_expand_plugin_root_multiple` function L127-134 — `()` — ```
-  `test_expand_plugin_root_no_variable` function L137-144 — `()` — ```
-  `test_expand_plugin_root_path` function L147-157 — `()` — ```
-  `test_expand_plugin_root_path_no_variable` function L160-167 — `()` — ```
-  `test_claude_plugin_root_var_name` function L170-172 — `()` — ```

#### crates/arawn-plugin/src/manager.rs

- pub `MANIFEST_PATH` variable L13 — `: &str` — The path to the plugin manifest relative to the plugin root.
- pub `LoadedPlugin` struct L17-28 — `{ manifest: PluginManifest, plugin_dir: PathBuf, skill_contents: Vec<LoadedSkill...` — A fully loaded plugin with all component content read from disk.
- pub `meta` function L32-34 — `(&self) -> PluginMeta` — Get the plugin metadata (name, version, description).
- pub `LoadedSkill` struct L39-44 — `{ def: SkillDef, content: String }` — A skill with its markdown content loaded from disk.
- pub `LoadedAgent` struct L48-53 — `{ def: PluginAgentDef, config: PluginAgentConfig }` — An agent with its config loaded from disk.
- pub `PluginManager` struct L57-60 — `{ plugin_dirs: Vec<PathBuf> }` — Manages plugin discovery and loading.
- pub `new` function L64-66 — `(plugin_dirs: Vec<PathBuf>) -> Self` — Create a new `PluginManager` with the given plugin directories.
- pub `with_defaults` function L71-83 — `() -> Self` — Create a `PluginManager` with default directories.
- pub `plugin_dirs` function L86-88 — `(&self) -> &[PathBuf]` — Get the configured plugin directories.
- pub `load_all` function L94-122 — `(&self) -> Vec<LoadedPlugin>` — Discover and load all plugins from configured directories.
- pub `load_single` function L366-375 — `(&self, plugin_dir: &Path) -> Result<LoadedPlugin>` — Load a single plugin by directory path (for hot-reload).
-  `LoadedPlugin` type L30-35 — `= LoadedPlugin` — component files (skills, agent configs) from disk.
-  `PluginManager` type L62-376 — `= PluginManager` — component files (skills, agent configs) from disk.
-  `scan_directory` function L127-167 — `(&self, dir: &Path) -> Result<Vec<LoadedPlugin>>` — Scan a single directory for plugin subdirectories.
-  `load_plugin` function L173-192 — `(&self, plugin_dir: &Path, manifest_path: &Path) -> Result<LoadedPlugin>` — Load a single plugin from its directory.
-  `discover_skills` function L197-257 — `(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Vec<LoadedSkill>` — Discover skills from the skills directories.
-  `discover_agents` function L262-321 — `(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Vec<LoadedAgent>` — Discover agents from the agents directories.
-  `load_hooks` function L326-363 — `(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Option<HooksConfig>` — Load hooks configuration from hooks.json.
-  `extract_frontmatter_field` function L381-405 — `(content: &str, field: &str) -> Option<String>` — Extract a field value from YAML frontmatter in a markdown file.
-  `parse_agent_markdown` function L421-471 — `(name: &str, content: &str) -> Result<(PluginAgentDef, PluginAgentConfig)>` — Parse an agent configuration from a Claude-format markdown file.
-  `tests` module L474-868 — `-` — component files (skills, agent configs) from disk.
-  `create_test_plugin` function L480-566 — `(base_dir: &Path, name: &str) -> PathBuf` — Create a minimal plugin directory structure for testing (Claude format).
-  `test_load_single_plugin` function L569-588 — `()` — component files (skills, agent configs) from disk.
-  `test_load_all_discovers_multiple_plugins` function L591-603 — `()` — component files (skills, agent configs) from disk.
-  `test_load_all_skips_nonexistent_dirs` function L606-610 — `()` — component files (skills, agent configs) from disk.
-  `test_load_all_skips_invalid_plugins` function L613-633 — `()` — component files (skills, agent configs) from disk.
-  `test_load_skips_missing_skill_dirs` function L636-653 — `()` — component files (skills, agent configs) from disk.
-  `test_load_skips_missing_agent_dirs` function L656-672 — `()` — component files (skills, agent configs) from disk.
-  `test_load_single_missing_manifest` function L675-679 — `()` — component files (skills, agent configs) from disk.
-  `test_plugin_dir_stored` function L682-690 — `()` — component files (skills, agent configs) from disk.
-  `test_with_defaults` function L693-697 — `()` — component files (skills, agent configs) from disk.
-  `test_ignores_files_in_plugin_dir` function L700-708 — `()` — component files (skills, agent configs) from disk.
-  `test_plugin_meta` function L711-722 — `()` — component files (skills, agent configs) from disk.
-  `test_extract_frontmatter_field` function L725-742 — `()` — component files (skills, agent configs) from disk.
-  `test_extract_frontmatter_field_no_frontmatter` function L745-748 — `()` — component files (skills, agent configs) from disk.
-  `test_parse_agent_markdown` function L751-781 — `()` — component files (skills, agent configs) from disk.
-  `test_manifest_path_constant` function L784-786 — `()` — component files (skills, agent configs) from disk.
-  `test_load_hooks_from_default_path` function L789-824 — `()` — component files (skills, agent configs) from disk.
-  `test_load_hooks_missing_file` function L827-844 — `()` — component files (skills, agent configs) from disk.
-  `test_load_hooks_invalid_json` function L847-867 — `()` — component files (skills, agent configs) from disk.

#### crates/arawn-plugin/src/manifest.rs

- pub `CapabilitySummary` struct L13-30 — `{ skills_declared: bool, skills_found: usize, agents_declared: bool, agents_foun...` — Summary of declared vs discovered capabilities for a plugin.
- pub `has_errors` function L38-43 — `(&self) -> bool` — Check if there are any capability mismatches.
- pub `warnings` function L46-63 — `(&self) -> Vec<String>` — Get a list of warnings (undeclared but found capabilities).
- pub `errors` function L66-99 — `(&self) -> Vec<ManifestValidationError>` — Get a list of errors (declared but not found capabilities).
- pub `PluginManifest` struct L120-179 — `{ name: String, version: Option<String>, description: Option<String>, author: Op...` — Top-level plugin manifest parsed from `.claude-plugin/plugin.json`.
- pub `PathOrPaths` enum L184-189 — `Single | Multiple` — A path or array of paths (Claude supports both).
- pub `to_vec` function L193-198 — `(&self) -> Vec<PathBuf>` — Get all paths as a vector.
- pub `resolve` function L201-206 — `(&self, base: &Path) -> Vec<PathBuf>` — Resolve all paths against a base directory.
- pub `PluginAuthor` struct L211-220 — `{ name: String, email: Option<String>, url: Option<String> }` — Plugin author information.
- pub `PluginMeta` struct L225-232 — `{ name: String, version: String, description: String }` — Legacy plugin metadata (for internal compatibility).
- pub `from_json` function L249-256 — `(json_str: &str) -> Result<Self>` — Parse a manifest from a JSON string.
- pub `from_file` function L261-265 — `(path: &Path) -> Result<Self>` — Parse a manifest from a file on disk.
- pub `validate` function L275-291 — `(&self) -> Result<()>` — Validate required fields and constraints.
- pub `validate_paths` function L306-339 — `(&self, plugin_dir: &Path) -> Vec<ManifestValidationError>` — Validate that declared paths exist on disk.
- pub `capability_summary` function L345-365 — `(&self, plugin_dir: &Path) -> CapabilitySummary` — Get a summary of declared vs discovered capabilities.
- pub `skills_paths` function L368-373 — `(&self, plugin_dir: &Path) -> Vec<PathBuf>` — Get the skills directory paths resolved against a base directory.
- pub `agents_paths` function L376-381 — `(&self, plugin_dir: &Path) -> Vec<PathBuf>` — Get the agents directory paths resolved against a base directory.
- pub `hooks_paths` function L384-389 — `(&self, plugin_dir: &Path) -> Vec<PathBuf>` — Get the hooks config paths resolved against a base directory.
- pub `commands_paths` function L392-397 — `(&self, plugin_dir: &Path) -> Vec<PathBuf>` — Get the commands paths resolved against a base directory.
- pub `plugin_meta` function L400-402 — `(&self) -> PluginMeta` — Get plugin metadata in the legacy format.
-  `CapabilitySummary` type L32-100 — `= CapabilitySummary` — and paths to component directories.
-  `PathOrPaths` type L191-207 — `= PathOrPaths` — and paths to component directories.
-  `PluginMeta` type L234-245 — `= PluginMeta` — and paths to component directories.
-  `from` function L235-244 — `(manifest: &PluginManifest) -> Self` — and paths to component directories.
-  `PluginManifest` type L247-403 — `= PluginManifest` — and paths to component directories.
-  `tests` module L406-702 — `-` — and paths to component directories.
-  `sample_manifest_json` function L409-426 — `() -> &'static str` — and paths to component directories.
-  `test_parse_full_manifest` function L429-449 — `()` — and paths to component directories.
-  `test_minimal_manifest` function L452-460 — `()` — and paths to component directories.
-  `test_empty_name_fails_validation` function L463-467 — `()` — and paths to component directories.
-  `test_non_kebab_name_fails_validation` function L470-474 — `()` — and paths to component directories.
-  `test_uppercase_name_fails_validation` function L477-481 — `()` — and paths to component directories.
-  `test_path_or_paths_single` function L484-490 — `()` — and paths to component directories.
-  `test_path_or_paths_multiple` function L493-505 — `()` — and paths to component directories.
-  `test_agents_paths` function L508-514 — `()` — and paths to component directories.
-  `test_hooks_paths` function L517-523 — `()` — and paths to component directories.
-  `test_plugin_meta_conversion` function L526-533 — `()` — and paths to component directories.
-  `test_plugin_meta_defaults` function L536-544 — `()` — and paths to component directories.
-  `test_roundtrip_serialize` function L547-553 — `()` — and paths to component directories.
-  `test_from_file` function L556-563 — `()` — and paths to component directories.
-  `test_invalid_json` function L566-569 — `()` — and paths to component directories.
-  `test_valid_version` function L572-576 — `()` — and paths to component directories.
-  `test_valid_version_with_prerelease` function L579-583 — `()` — and paths to component directories.
-  `test_valid_version_two_parts` function L586-590 — `()` — and paths to component directories.
-  `test_invalid_version_single_number` function L593-597 — `()` — and paths to component directories.
-  `test_invalid_version_non_numeric` function L600-604 — `()` — and paths to component directories.
-  `test_invalid_version_leading_zero` function L607-611 — `()` — and paths to component directories.
-  `test_name_starts_with_hyphen_fails` function L614-618 — `()` — and paths to component directories.
-  `test_name_ends_with_hyphen_fails` function L621-625 — `()` — and paths to component directories.
-  `test_name_consecutive_hyphens_fails` function L628-632 — `()` — and paths to component directories.
-  `test_name_starts_with_number_fails` function L635-639 — `()` — and paths to component directories.
-  `test_capability_summary_empty` function L642-649 — `()` — and paths to component directories.
-  `test_capability_summary_declared_but_not_found` function L652-661 — `()` — and paths to component directories.
-  `test_validate_paths_missing` function L664-670 — `()` — and paths to component directories.
-  `test_validate_paths_exists` function L673-680 — `()` — and paths to component directories.
-  `test_mcp_servers_inline` function L683-694 — `()` — and paths to component directories.
-  `test_mcp_servers_path` function L697-701 — `()` — and paths to component directories.

#### crates/arawn-plugin/src/skill.rs

- pub `Skill` struct L34-47 — `{ name: String, description: String, uses_tools: Vec<String>, args: Vec<SkillArg...` — A parsed skill ready for invocation.
- pub `SkillInvocation` struct L51-58 — `{ name: String, plugin: Option<String>, raw_args: String }` — Result of parsing a `/skill-name args` or `/plugin:skill args` invocation from a user message.
- pub `parse_skill` function L76-99 — `(content: &str, plugin_name: &str) -> Result<Skill>` — Parse a skill from its markdown content.
- pub `detect_invocation` function L135-185 — `(message: &str) -> Option<SkillInvocation>` — Detect a skill invocation in a user message.
- pub `substitute_args` function L191-221 — `(skill: &Skill, raw_args: &str) -> Result<String>` — Substitute arguments into a skill body template.
- pub `SkillRegistry` struct L229-234 — `{ skills: HashMap<String, Skill>, by_simple_name: HashMap<String, Vec<String>> }` — Registry of loaded skills, queryable by name or qualified name.
- pub `new` function L238-240 — `() -> Self` — Create an empty skill registry.
- pub `register` function L246-256 — `(&mut self, skill: Skill)` — Register a skill.
- pub `get` function L262-277 — `(&self, name: &str) -> Option<&Skill>` — Look up a skill by name (simple) or qualified name (plugin:skill).
- pub `get_by_invocation` function L280-289 — `(&self, invocation: &SkillInvocation) -> Option<&Skill>` — Look up a skill by invocation (handles namespacing).
- pub `names` function L292-294 — `(&self) -> Vec<&str>` — Get all registered skill names (qualified names).
- pub `len` function L297-299 — `(&self) -> usize` — Get the number of registered skills.
- pub `is_empty` function L302-304 — `(&self) -> bool` — Check if the registry is empty.
- pub `invoke` function L310-318 — `(&self, invocation: &SkillInvocation) -> Result<Option<String>>` — Invoke a skill by invocation with raw arguments.
- pub `invoke_simple` function L324-332 — `(&self, name: &str, raw_args: &str) -> Result<Option<String>>` — Invoke a skill by simple name with raw arguments (convenience method).
-  `SkillFrontmatter` struct L62-70 — `{ name: String, description: String, uses_tools: Vec<String>, args: Vec<SkillArg...` — Frontmatter parsed from a skill markdown file.
-  `split_frontmatter` function L102-125 — `(content: &str) -> Result<(String, String)>` — Split markdown content into frontmatter and body.
-  `SkillRegistry` type L236-333 — `= SkillRegistry` — ```
-  `tests` module L336-616 — `-` — ```
-  `SAMPLE_SKILL` variable L339-362 — `: &str` — ```
-  `test_parse_skill` function L365-377 — `()` — ```
-  `test_parse_skill_no_frontmatter` function L380-383 — `()` — ```
-  `test_parse_skill_no_closing_delimiter` function L386-389 — `()` — ```
-  `test_parse_skill_empty_name` function L392-396 — `()` — ```
-  `test_parse_skill_minimal` function L399-406 — `()` — ```
-  `test_detect_invocation_basic` function L409-414 — `()` — ```
-  `test_detect_invocation_no_args` function L417-422 — `()` — ```
-  `test_detect_invocation_with_whitespace` function L425-430 — `()` — ```
-  `test_detect_invocation_not_a_skill` function L433-437 — `()` — ```
-  `test_detect_invocation_uppercase_stops` function L440-446 — `()` — ```
-  `test_detect_invocation_namespaced` function L449-454 — `()` — ```
-  `test_detect_invocation_namespaced_no_args` function L457-462 — `()` — ```
-  `test_detect_invocation_invalid_namespace` function L465-469 — `()` — ```
-  `test_substitute_args_basic` function L472-479 — `()` — ```
-  `test_substitute_args_missing_required` function L482-487 — `()` — ```
-  `test_substitute_args_optional_missing` function L490-497 — `()` — ```
-  `test_substitute_args_no_args_needed` function L500-505 — `()` — ```
-  `test_skill_registry` function L508-527 — `()` — ```
-  `test_skill_registry_invoke` function L530-551 — `()` — ```
-  `test_skill_registry_invoke_missing_arg` function L554-568 — `()` — ```
-  `test_skill_registry_namespaced_lookup` function L571-589 — `()` — ```
-  `test_skill_registry_invoke_namespaced` function L592-605 — `()` — ```
-  `test_skill_registry_invoke_simple` function L608-615 — `()` — ```

#### crates/arawn-plugin/src/subscription.rs

- pub `RuntimePluginsConfig` struct L40-53 — `{ enabled_plugins: HashMap<String, bool>, subscriptions: Vec<PluginSubscription>...` — Runtime plugins configuration file format.
- pub `load` function L57-64 — `(path: &Path) -> crate::Result<Self>` — Load from a JSON file, returning default if file doesn't exist.
- pub `from_json` function L67-71 — `(json_str: &str) -> crate::Result<Self>` — Parse from a JSON string.
- pub `to_json` function L74-78 — `(&self) -> crate::Result<String>` — Serialize to a JSON string (pretty printed).
- pub `save` function L81-89 — `(&self, path: &Path) -> crate::Result<()>` — Save to a JSON file.
- pub `is_enabled` function L94-96 — `(&self, plugin_id: &str) -> Option<bool>` — Check if a plugin is enabled.
- pub `set_enabled` function L99-101 — `(&mut self, plugin_id: impl Into<String>, enabled: bool)` — Set a plugin's enabled state.
- pub `add_subscription` function L104-110 — `(&mut self, subscription: PluginSubscription)` — Add a subscription.
- pub `remove_subscription` function L113-115 — `(&mut self, subscription_id: &str)` — Remove a subscription by its ID.
- pub `merge` function L120-133 — `(&mut self, other: RuntimePluginsConfig)` — Merge another config into this one.
- pub `SubscriptionManager` struct L138-151 — `{ config_subscriptions: Vec<PluginSubscription>, global_config: RuntimePluginsCo...` — Manager for plugin subscriptions across all sources.
- pub `new` function L160-191 — `( config_subscriptions: Vec<PluginSubscription>, project_dir: Option<&Path>, ) -...` — Create a new subscription manager.
- pub `all_subscriptions` function L197-234 — `(&self) -> Vec<PluginSubscription>` — Get all active subscriptions, merged from all sources.
- pub `cache_dir_for` function L237-239 — `(&self, subscription: &PluginSubscription) -> PathBuf` — Get the cache directory for a subscription.
- pub `global_config` function L242-244 — `(&self) -> &RuntimePluginsConfig` — Get the global runtime config.
- pub `project_config` function L247-249 — `(&self) -> &RuntimePluginsConfig` — Get the project runtime config.
- pub `global_config_mut` function L252-254 — `(&mut self) -> &mut RuntimePluginsConfig` — Get a mutable reference to the global runtime config.
- pub `project_config_mut` function L257-259 — `(&mut self) -> &mut RuntimePluginsConfig` — Get a mutable reference to the project runtime config.
- pub `save_global_config` function L262-264 — `(&self) -> crate::Result<()>` — Save the global runtime config.
- pub `save_project_config` function L267-273 — `(&self) -> crate::Result<()>` — Save the project runtime config.
- pub `add_global_subscription` function L276-278 — `(&mut self, subscription: PluginSubscription)` — Add a subscription to the global config.
- pub `add_project_subscription` function L281-283 — `(&mut self, subscription: PluginSubscription)` — Add a subscription to the project config.
- pub `set_global_enabled` function L286-288 — `(&mut self, plugin_id: impl Into<String>, enabled: bool)` — Enable or disable a plugin globally.
- pub `set_project_enabled` function L291-293 — `(&mut self, plugin_id: impl Into<String>, enabled: bool)` — Enable or disable a plugin for the current project.
- pub `cache_dir` function L296-298 — `(&self) -> &Path` — Get the cache directory.
- pub `is_auto_update_disabled` function L303-307 — `() -> bool` — Check if auto-update is disabled via environment variable.
- pub `update_timeout_secs` function L313-318 — `() -> u64` — Get the update timeout from environment variable.
- pub `sync_all_async` function L324-457 — `(&self) -> Vec<SyncResult>` — Sync all subscriptions in parallel (async version).
- pub `sync_all` function L462-472 — `(&self) -> Vec<SyncResult>` — Sync all subscriptions (clone or update).
- pub `sync_subscription` function L475-535 — `(&self, subscription: &PluginSubscription) -> SyncResult` — Sync a single subscription (clone or update).
- pub `plugin_dir_for` function L541-553 — `(&self, subscription: &PluginSubscription) -> Option<PathBuf>` — Get the plugin directory for a subscription.
- pub `plugin_dirs` function L556-561 — `(&self) -> Vec<PathBuf>` — Get all plugin directories (synced subscriptions + local paths).
- pub `GitOps` struct L572 — `-` — Git operations for plugin syncing.
- pub `clone` function L578-611 — `(url: &str, dest: &Path, git_ref: &str) -> Result<(), String>` — Clone a repository to a destination directory.
- pub `pull` function L616-670 — `(repo_dir: &Path, git_ref: &str) -> Result<(), String>` — Pull updates for an existing repository.
- pub `is_available` function L673-679 — `() -> bool` — Check if git is available on the system.
- pub `current_commit` function L682-690 — `(repo_dir: &Path) -> Option<String>` — Get the current commit hash of a repository.
- pub `current_branch` function L693-702 — `(repo_dir: &Path) -> Option<String>` — Get the current branch name (if on a branch).
- pub `SyncResult` struct L711-720 — `{ subscription_id: String, action: SyncAction, path: Option<PathBuf>, error: Opt...` — Result of syncing a subscription.
- pub `is_success` function L724-729 — `(&self) -> bool` — Check if the sync was successful.
- pub `is_failure` function L732-737 — `(&self) -> bool` — Check if this was a failure.
- pub `SyncAction` enum L742-753 — `Cloned | Updated | Skipped | CloneFailed | UpdateFailed` — Action taken during sync.
-  `RuntimePluginsConfig` type L55-134 — `= RuntimePluginsConfig` — ```
-  `SubscriptionManager` type L153-562 — `= SubscriptionManager` — ```
-  `GitOps` type L574-703 — `= GitOps` — ```
-  `SyncResult` type L722-738 — `= SyncResult` — ```
-  `SyncAction` type L755-765 — `= SyncAction` — ```
-  `fmt` function L756-764 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — ```
-  `tests` module L768-1234 — `-` — ```
-  `test_runtime_config_parse` function L774-801 — `()` — ```
-  `test_runtime_config_empty` function L804-809 — `()` — ```
-  `test_runtime_config_roundtrip` function L812-822 — `()` — ```
-  `test_runtime_config_save_load` function L825-838 — `()` — ```
-  `test_runtime_config_load_missing_file` function L841-845 — `()` — ```
-  `test_runtime_config_merge` function L848-864 — `()` — ```
-  `test_subscription_id_github` function L867-870 — `()` — ```
-  `test_subscription_id_url` function L873-877 — `()` — ```
-  `test_subscription_id_local` function L880-884 — `()` — ```
-  `test_subscription_clone_url` function L887-902 — `()` — ```
-  `test_subscription_effective_ref` function L905-911 — `()` — ```
-  `test_subscription_manager_merge` function L914-933 — `()` — ```
-  `test_subscription_manager_dedup` function L936-955 — `()` — ```
-  `test_subscription_manager_enabled_filter` function L958-981 — `()` — ```
-  `test_git_is_available` function L986-992 — `()` — ```
-  `test_sync_result_is_success` function L995-1007 — `()` — ```
-  `test_sync_result_is_failure` function L1010-1022 — `()` — ```
-  `test_sync_action_display` function L1025-1031 — `()` — ```
-  `test_sync_local_subscription_skipped` function L1034-1046 — `()` — ```
-  `test_plugin_dir_for_local` function L1049-1060 — `()` — ```
-  `test_plugin_dir_for_remote_not_synced` function L1063-1070 — `()` — ```
-  `test_sync_subscription_no_clone_url` function L1073-1090 — `()` — ```
-  `test_auto_update_disabled_check` function L1093-1128 — `()` — ```
-  `EnvGuard` struct L1100 — `-` — ```
-  `EnvGuard` type L1101-1106 — `impl Drop for EnvGuard` — ```
-  `drop` function L1102-1105 — `(&mut self)` — ```
-  `test_update_timeout_secs` function L1131-1155 — `()` — ```
-  `EnvGuard` struct L1139 — `-` — ```
-  `EnvGuard` type L1140-1145 — `impl Drop for EnvGuard` — ```
-  `drop` function L1141-1144 — `(&mut self)` — ```
-  `test_sync_all_async_local_skipped` function L1162-1178 — `()` — ```
-  `test_git_clone_real_repo` function L1183-1209 — `()` — ```
-  `test_git_pull_real_repo` function L1213-1233 — `()` — ```

#### crates/arawn-plugin/src/types.rs

- pub `HooksConfigExt` interface L16-21 — `{ fn from_json(), fn from_file() }` — Extension trait for HooksConfig to add parsing methods.
- pub `SkillDef` struct L42-52 — `{ name: String, description: String, file: PathBuf, uses_tools: Vec<String> }` — A skill definition from a plugin manifest.
- pub `SkillArg` struct L56-65 — `{ name: String, description: String, required: bool }` — A skill argument declaration (parsed from skill markdown frontmatter).
- pub `PluginAgentDef` struct L75-85 — `{ name: String, description: String, file: PathBuf, tools: Vec<String> }` — A plugin-defined agent (subagent) definition.
- pub `PluginAgentConfig` struct L89-92 — `{ agent: AgentSection }` — Full agent configuration parsed from an agent markdown file.
- pub `AgentSection` struct L96-111 — `{ name: String, description: String, model: Option<String>, system_prompt: Optio...` — Agent configuration section.
- pub `AgentSystemPrompt` struct L115-118 — `{ text: String }` — System prompt for a plugin agent.
- pub `AgentConstraints` struct L122-129 — `{ tools: Vec<String>, max_iterations: Option<usize> }` — Constraints on a plugin agent.
- pub `PromptFragment` struct L137-141 — `{ system: Option<String> }` — Plugin-provided prompt fragment injected into the system prompt.
-  `HooksConfig` type L23-34 — `impl HooksConfigExt for HooksConfig` — Core types for the plugin system.
-  `from_json` function L24-28 — `(json_str: &str) -> Result<HooksConfig, crate::PluginError>` — Core types for the plugin system.
-  `from_file` function L30-33 — `(path: &std::path::Path) -> Result<HooksConfig, crate::PluginError>` — Core types for the plugin system.
-  `tests` module L144-268 — `-` — Core types for the plugin system.
-  `test_hook_event_display` function L148-153 — `()` — Core types for the plugin system.
-  `test_hook_event_serde_roundtrip` function L156-162 — `()` — Core types for the plugin system.
-  `test_new_hook_events_serde` function L165-181 — `()` — Core types for the plugin system.
-  `test_plugin_agent_config_parse` function L184-208 — `()` — Core types for the plugin system.
-  `test_hooks_config_parse` function L211-251 — `()` — Core types for the plugin system.
-  `test_hooks_config_empty` function L254-261 — `()` — Core types for the plugin system.
-  `test_hook_type_default` function L264-267 — `()` — Core types for the plugin system.

#### crates/arawn-plugin/src/validation.rs

- pub `ManifestValidationError` enum L13-62 — `MissingField | InvalidField | InvalidVersion | CapabilityMismatch | PathNotFound` — Error type for manifest validation failures.
- pub `missing_field` function L66-68 — `(field: &'static str, hint: &'static str) -> Self` — Create a missing field error.
- pub `invalid_field` function L71-76 — `(field: &'static str, message: impl Into<String>) -> Self` — Create an invalid field error.
- pub `invalid_version` function L79-84 — `(version: impl Into<String>, reason: impl Into<String>) -> Self` — Create an invalid version error.
- pub `capability_mismatch` function L87-97 — `( capability: &'static str, declared: impl Into<String>, actual: impl Into<Strin...` — Create a capability mismatch error.
- pub `path_not_found` function L100-105 — `(field: &'static str, path: impl Into<String>) -> Self` — Create a path not found error.
- pub `field_name` function L108-116 — `(&self) -> Option<&str>` — Get the field name associated with this error (if any).
- pub `ValidationResult` type L120 — `= std::result::Result<T, ManifestValidationError>` — Result type for validation operations.
- pub `validate_name` function L129-178 — `(name: &str) -> ValidationResult<()>` — Validate a plugin name.
- pub `validate_version` function L188-234 — `(version: &str) -> ValidationResult<()>` — Validate a semantic version string.
- pub `validate_paths_exist` function L239-260 — `( field: &'static str, paths: &[std::path::PathBuf], plugin_dir: &Path, ) -> Val...` — Validate that declared paths exist relative to a plugin directory.
- pub `count_discovered_items` function L266-299 — `( paths: &[std::path::PathBuf], plugin_dir: &Path, pattern: &str, ) -> usize` — Count items discovered at the given paths.
-  `ManifestValidationError` type L64-117 — `= ManifestValidationError` — capabilities match actual exports.
-  `tests` module L302-505 — `-` — capabilities match actual exports.
-  `test_valid_names` function L310-316 — `()` — capabilities match actual exports.
-  `test_empty_name` function L319-325 — `()` — capabilities match actual exports.
-  `test_name_starts_with_number` function L328-334 — `()` — capabilities match actual exports.
-  `test_name_starts_with_hyphen` function L337-343 — `()` — capabilities match actual exports.
-  `test_name_ends_with_hyphen` function L346-352 — `()` — capabilities match actual exports.
-  `test_name_consecutive_hyphens` function L355-361 — `()` — capabilities match actual exports.
-  `test_name_uppercase` function L364-370 — `()` — capabilities match actual exports.
-  `test_name_spaces` function L373-379 — `()` — capabilities match actual exports.
-  `test_name_underscores` function L382-388 — `()` — capabilities match actual exports.
-  `test_valid_versions` function L395-404 — `()` — capabilities match actual exports.
-  `test_empty_version` function L407-413 — `()` — capabilities match actual exports.
-  `test_version_single_number` function L416-422 — `()` — capabilities match actual exports.
-  `test_version_four_parts` function L425-431 — `()` — capabilities match actual exports.
-  `test_version_non_numeric` function L434-440 — `()` — capabilities match actual exports.
-  `test_version_leading_zero` function L443-449 — `()` — capabilities match actual exports.
-  `test_version_empty_component` function L452-458 — `()` — capabilities match actual exports.
-  `test_error_display` function L465-469 — `()` — capabilities match actual exports.
-  `test_error_field_name` function L472-481 — `()` — capabilities match actual exports.
-  `test_paths_exist_empty` function L488-491 — `()` — capabilities match actual exports.
-  `test_paths_exist_missing` function L494-504 — `()` — capabilities match actual exports.

#### crates/arawn-plugin/src/watcher.rs

- pub `PluginEvent` enum L18-25 — `Reloaded | Removed | Error` — Event emitted when a plugin is reloaded, added, or removed.
- pub `PluginState` struct L29-32 — `{ plugins: HashMap<PathBuf, LoadedPlugin> }` — Shared plugin state that can be read concurrently and swapped on reload.
- pub `plugins` function L36-38 — `(&self) -> Vec<&LoadedPlugin>` — Get all loaded plugins.
- pub `get_by_name` function L41-43 — `(&self, name: &str) -> Option<&LoadedPlugin>` — Get a plugin by its name.
- pub `len` function L46-48 — `(&self) -> usize` — Get the number of loaded plugins.
- pub `is_empty` function L51-53 — `(&self) -> bool` — Check if empty.
- pub `PluginWatcher` struct L57-64 — `{ manager: PluginManager, state: Arc<RwLock<PluginState>>, debounce: Duration }` — File watcher that monitors plugin directories and triggers reloads.
- pub `new` function L68-74 — `(manager: PluginManager) -> Self` — Create a new plugin watcher.
- pub `with_debounce` function L77-80 — `(mut self, duration: Duration) -> Self` — Set the debounce duration.
- pub `state` function L83-85 — `(&self) -> Arc<RwLock<PluginState>>` — Get a reference to the shared plugin state.
- pub `load_initial` function L88-104 — `(&self) -> Vec<PluginEvent>` — Perform initial load of all plugins.
- pub `reload_plugin` function L107-128 — `(&self, plugin_dir: &Path) -> PluginEvent` — Reload a single plugin by its directory path.
- pub `remove_plugin` function L131-143 — `(&self, plugin_dir: &Path) -> Option<PluginEvent>` — Remove a plugin by its directory path.
- pub `watch` function L149-228 — `( &self, ) -> Result<(mpsc::Receiver<PluginEvent>, WatcherHandle), crate::Plugin...` — Start watching all plugin directories for changes.
- pub `WatcherHandle` struct L277-279 — `{ _thread: std::thread::JoinHandle<()> }` — Handle that keeps the file watcher alive.
-  `PluginState` type L34-54 — `= PluginState` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `PluginWatcher` type L66-229 — `= PluginWatcher` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `find_plugin_dir` function L235-248 — `(path: &Path, plugin_dirs: &[PathBuf]) -> Option<PathBuf>` — Find the plugin directory containing a given path.
-  `reload_from_dir` function L251-274 — `(state: &Arc<RwLock<PluginState>>, plugin_dir: &Path) -> PluginEvent` — Reload a plugin from its directory into the shared state.
-  `tests` module L282-438 — `-` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `create_test_plugin` function L287-304 — `(base_dir: &Path, name: &str) -> PathBuf` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_load_initial` function L307-323 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_reload_plugin` function L326-352 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_remove_plugin` function L355-370 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_remove_nonexistent` function L373-380 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_reload_invalid_plugin` function L383-394 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_state_get_by_name` function L397-409 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_find_plugin_dir` function L412-430 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).
-  `test_debounce_config` function L433-437 — `()` — Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).

### crates/arawn-sandbox/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-sandbox/src/config.rs

- pub `SandboxConfig` struct L28-49 — `{ write_paths: Vec<PathBuf>, deny_read_paths: Vec<PathBuf>, allowed_domains: Vec...` — Configuration for sandbox execution.
- pub `new` function L67-69 — `() -> Self` — Create a new sandbox configuration with defaults.
- pub `with_write_paths` function L72-75 — `(mut self, paths: Vec<PathBuf>) -> Self` — Set paths allowed for writing.
- pub `add_write_path` function L78-81 — `(mut self, path: impl Into<PathBuf>) -> Self` — Add a single write path.
- pub `with_deny_read_paths` function L84-87 — `(mut self, paths: Vec<PathBuf>) -> Self` — Set paths denied for reading.
- pub `add_deny_read_path` function L90-93 — `(mut self, path: impl Into<PathBuf>) -> Self` — Add a path to deny for reading.
- pub `with_allowed_domains` function L96-99 — `(mut self, domains: Vec<String>) -> Self` — Set allowed network domains.
- pub `add_allowed_domain` function L102-105 — `(mut self, domain: impl Into<String>) -> Self` — Add an allowed network domain.
- pub `with_working_dir` function L108-111 — `(mut self, dir: impl Into<PathBuf>) -> Self` — Set the working directory.
- pub `with_timeout` function L114-117 — `(mut self, timeout: Duration) -> Self` — Set the command timeout.
- pub `add_env` function L120-123 — `(mut self, key: impl Into<String>, value: impl Into<String>) -> Self` — Add an environment variable.
- pub `with_git_access` function L126-129 — `(mut self, allow: bool) -> Self` — Allow access to .git directories.
- pub `default_deny_read_paths` function L135-202 — `() -> Vec<PathBuf>` — Get the default paths to deny for reading.
- pub `for_workstream` function L207-209 — `(workstream_production: PathBuf, workstream_work: PathBuf) -> Self` — Create a config for a workstream session.
- pub `for_scratch_session` function L214-216 — `(session_work: PathBuf) -> Self` — Create a config for a scratch session.
-  `SandboxConfig` type L51-63 — `impl Default for SandboxConfig` — Sandbox configuration.
-  `default` function L52-62 — `() -> Self` — Sandbox configuration.
-  `SandboxConfig` type L65-217 — `= SandboxConfig` — Sandbox configuration.
-  `tests` module L220-289 — `-` — Sandbox configuration.
-  `test_default_config` function L224-231 — `()` — Sandbox configuration.
-  `test_builder_pattern` function L234-245 — `()` — Sandbox configuration.
-  `test_default_deny_paths` function L248-259 — `()` — Sandbox configuration.
-  `test_workstream_config` function L262-275 — `()` — Sandbox configuration.
-  `test_scratch_config` function L278-288 — `()` — Sandbox configuration.

#### crates/arawn-sandbox/src/error.rs

- pub `SandboxError` enum L8-43 — `Unavailable | InitializationFailed | ExecutionFailed | PathNotAllowed | ConfigEr...` — Errors that can occur during sandbox operations.
- pub `SandboxResult` type L46 — `= std::result::Result<T, SandboxError>` — Result type for sandbox operations.

#### crates/arawn-sandbox/src/lib.rs

-  `config` module L50 — `-` — This crate provides a high-level interface to the `sandbox-runtime` crate,
-  `error` module L51 — `-` — ```
-  `manager` module L52 — `-` — ```
-  `platform` module L53 — `-` — ```

#### crates/arawn-sandbox/src/manager.rs

- pub `CommandOutput` struct L19-28 — `{ stdout: String, stderr: String, exit_code: i32, success: bool }` — Output from a sandboxed command execution.
- pub `new` function L32-39 — `(stdout: String, stderr: String, exit_code: i32) -> Self` — Create a new command output.
- pub `error` function L42-49 — `(message: String) -> Self` — Create an output for a failed command.
- pub `combined_output` function L52-60 — `(&self) -> String` — Combine stdout and stderr for display.
- pub `SandboxManager` struct L84-87 — `{ runtime: RuntimeSandboxManager, platform: Platform }` — Manager for sandboxed command execution.
- pub `new` function L95-116 — `() -> SandboxResult<Self>` — Create a new sandbox manager.
- pub `check_availability` function L119-121 — `() -> SandboxStatus` — Check if sandbox is available on this platform.
- pub `platform` function L124-126 — `(&self) -> Platform` — Get the current platform.
- pub `execute` function L142-176 — `( &self, command: &str, config: &SandboxConfig, ) -> SandboxResult<CommandOutput...` — Execute a command in the sandbox.
- pub `execute_with_paths` function L257-270 — `( &self, command: &str, working_dir: &Path, allowed_write_paths: &[std::path::Pa...` — Execute a command with explicit path restrictions.
- pub `validate_config` function L275-297 — `(&self, config: &SandboxConfig) -> SandboxResult<()>` — Check if a command would be allowed under the given config.
-  `CommandOutput` type L30-61 — `= CommandOutput` — Sandbox manager for command execution.
-  `SandboxManager` type L89-298 — `= SandboxManager` — Sandbox manager for command execution.
-  `execute_wrapped` function L179-221 — `( &self, wrapped_command: &str, config: &SandboxConfig, ) -> SandboxResult<Comma...` — Execute the already-wrapped command.
-  `build_runtime_config` function L224-252 — `(&self, config: &SandboxConfig) -> SandboxResult<SandboxRuntimeConfig>` — Build the sandbox-runtime configuration from our config.
-  `tests` module L301-436 — `-` — Sandbox manager for command execution.
-  `test_command_output_success` function L305-310 — `()` — Sandbox manager for command execution.
-  `test_command_output_error` function L313-318 — `()` — Sandbox manager for command execution.
-  `test_command_output_combined` function L321-327 — `()` — Sandbox manager for command execution.
-  `test_sandbox_manager_creation` function L330-341 — `()` — Sandbox manager for command execution.
-  `test_validate_config_working_dir` function L344-360 — `()` — Sandbox manager for command execution.
-  `test_sandboxed_echo` function L367-383 — `()` — Sandbox manager for command execution.
-  `test_sandboxed_write_allowed` function L387-408 — `()` — Sandbox manager for command execution.
-  `test_sandboxed_write_denied` function L412-435 — `()` — Sandbox manager for command execution.

#### crates/arawn-sandbox/src/platform.rs

- pub `Platform` enum L8-15 — `MacOS | Linux | Unsupported` — Supported sandbox platforms.
- pub `detect` function L19-34 — `() -> Self` — Detect the current platform.
- pub `name` function L37-43 — `(&self) -> &'static str` — Get the display name for this platform.
- pub `SandboxStatus` enum L54-67 — `Available | MissingDependency | Unsupported` — Status of sandbox availability.
- pub `is_available` function L71-73 — `(&self) -> bool` — Check if sandbox is available.
- pub `install_hint` function L76-81 — `(&self) -> Option<&str>` — Get the install hint if dependencies are missing.
- pub `detect` function L84-94 — `() -> Self` — Detect sandbox availability for the current platform.
-  `Platform` type L17-44 — `= Platform` — Platform detection and availability checking.
-  `Platform` type L46-50 — `= Platform` — Platform detection and availability checking.
-  `fmt` function L47-49 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` — Platform detection and availability checking.
-  `SandboxStatus` type L69-172 — `= SandboxStatus` — Platform detection and availability checking.
-  `check_macos` function L97-117 — `() -> Self` — Check macOS sandbox availability.
-  `check_linux` function L120-171 — `() -> Self` — Check Linux sandbox availability.
-  `SandboxStatus` type L174-196 — `= SandboxStatus` — Platform detection and availability checking.
-  `fmt` function L175-195 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` — Platform detection and availability checking.
-  `tests` module L199-248 — `-` — Platform detection and availability checking.
-  `test_platform_detect` function L203-211 — `()` — Platform detection and availability checking.
-  `test_platform_name` function L214-218 — `()` — Platform detection and availability checking.
-  `test_sandbox_status_detect` function L221-231 — `()` — Platform detection and availability checking.
-  `test_sandbox_status_display` function L234-247 — `()` — Platform detection and availability checking.

### crates/arawn-script-sdk/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-script-sdk/src/context.rs

- pub `Context` struct L7-9 — `{ data: Value }` — Wrapper around the JSON context passed to a script via stdin.
- pub `from_json` function L13-16 — `(json: &str) -> Result<Self, serde_json::Error>` — Parse a `Context` from a JSON string.
- pub `from_value` function L19-21 — `(data: Value) -> Self` — Create a context from an existing JSON value.
- pub `raw` function L24-26 — `(&self) -> &Value` — Get the raw JSON value.
- pub `get` function L29-40 — `(&self, path: &str) -> Option<&Value>` — Get a nested value by dot-separated path (e.g.
- pub `get_str` function L43-45 — `(&self, path: &str) -> Option<&str>` — Get a string value at the given path.
- pub `get_i64` function L48-50 — `(&self, path: &str) -> Option<i64>` — Get an i64 value at the given path.
- pub `get_f64` function L53-55 — `(&self, path: &str) -> Option<f64>` — Get an f64 value at the given path.
- pub `get_bool` function L58-60 — `(&self, path: &str) -> Option<bool>` — Get a bool value at the given path.
- pub `get_array` function L63-65 — `(&self, path: &str) -> Option<&Vec<Value>>` — Get an array value at the given path.
- pub `get_object` function L68-70 — `(&self, path: &str) -> Option<&serde_json::Map<String, Value>>` — Get an object value at the given path.
- pub `get_as` function L73-76 — `(&self, path: &str) -> Option<T>` — Deserialize a value at the given path into a typed struct.
-  `Context` type L11-77 — `= Context` — JSON context wrapper with typed field access helpers.
-  `tests` module L80-155 — `-` — JSON context wrapper with typed field access helpers.
-  `test_from_json` function L85-88 — `()` — JSON context wrapper with typed field access helpers.
-  `test_nested_path` function L91-97 — `()` — JSON context wrapper with typed field access helpers.
-  `test_array_index` function L100-105 — `()` — JSON context wrapper with typed field access helpers.
-  `test_missing_path` function L108-112 — `()` — JSON context wrapper with typed field access helpers.
-  `test_get_bool` function L115-118 — `()` — JSON context wrapper with typed field access helpers.
-  `test_get_f64` function L121-124 — `()` — JSON context wrapper with typed field access helpers.
-  `test_get_array` function L127-131 — `()` — JSON context wrapper with typed field access helpers.
-  `test_get_object` function L134-138 — `()` — JSON context wrapper with typed field access helpers.
-  `test_get_as` function L141-154 — `()` — JSON context wrapper with typed field access helpers.
-  `Item` struct L143-146 — `{ name: String, count: u32 }` — JSON context wrapper with typed field access helpers.

#### crates/arawn-script-sdk/src/error.rs

- pub `ScriptResult` type L6 — `= Result<T, ScriptError>` — Result type for script functions.
- pub `ScriptError` enum L10-19 — `Message | Json | Io | Regex` — Error type that scripts return.
-  `ScriptError` type L21-30 — `= ScriptError` — Error types for script execution.
-  `fmt` function L22-29 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` — Error types for script execution.
-  `ScriptError` type L32-36 — `= ScriptError` — Error types for script execution.
-  `from` function L33-35 — `(msg: String) -> Self` — Error types for script execution.
-  `ScriptError` type L38-42 — `= ScriptError` — Error types for script execution.
-  `from` function L39-41 — `(msg: &str) -> Self` — Error types for script execution.
-  `ScriptError` type L44-48 — `= ScriptError` — Error types for script execution.
-  `from` function L45-47 — `(e: serde_json::Error) -> Self` — Error types for script execution.
-  `ScriptError` type L50-54 — `= ScriptError` — Error types for script execution.
-  `from` function L51-53 — `(e: std::io::Error) -> Self` — Error types for script execution.
-  `ScriptError` type L56-60 — `= ScriptError` — Error types for script execution.
-  `from` function L57-59 — `(e: regex::Error) -> Self` — Error types for script execution.

#### crates/arawn-script-sdk/src/lib.rs

- pub `context` module L23 — `-` — This crate is pre-compiled for `wasm32-wasip1` and linked into sandbox
- pub `error` module L24 — `-` — ```
- pub `text` module L25 — `-` — ```
- pub `prelude` module L28-33 — `-` — Re-exports for convenient `use arawn_script_sdk::prelude::*`.
- pub `run_harness` function L70-112 — `(f: fn(Context) -> ScriptResult<serde_json::Value>) -> Result<(), i32>` — Internal harness called by the `entry!` macro.
-  `entry` macro L56-65 — `-` — Entry-point macro that generates a `main()` function.

#### crates/arawn-script-sdk/src/text.rs

- pub `matches` function L8-11 — `(text: &str, pattern: &str) -> ScriptResult<bool>` — Check if a string matches a regex pattern.
- pub `find_all` function L14-17 — `(text: &str, pattern: &str) -> ScriptResult<Vec<String>>` — Find all matches of a regex pattern in a string.
- pub `replace_all` function L20-23 — `(text: &str, pattern: &str, replacement: &str) -> ScriptResult<String>` — Replace all matches of a regex pattern.
- pub `split` function L26-29 — `(text: &str, pattern: &str) -> ScriptResult<Vec<String>>` — Split a string by a regex pattern.
- pub `extract` function L32-48 — `( text: &str, pattern: &str, ) -> ScriptResult<Option<std::collections::HashMap<...` — Extract named capture groups from a regex match.
- pub `truncate` function L51-59 — `(text: &str, max_len: usize) -> String` — Truncate a string to a maximum length, appending `...` if truncated.
- pub `word_count` function L62-64 — `(text: &str) -> usize` — Count words in a string (whitespace-separated).
- pub `estimate_tokens` function L67-69 — `(text: &str) -> usize` — Estimate token count (rough approximation: chars / 4).
-  `tests` module L72-135 — `-` — Text and string utilities for scripts.
-  `test_matches` function L76-79 — `()` — Text and string utilities for scripts.
-  `test_find_all` function L82-85 — `()` — Text and string utilities for scripts.
-  `test_replace_all` function L88-91 — `()` — Text and string utilities for scripts.
-  `test_split` function L94-97 — `()` — Text and string utilities for scripts.
-  `test_extract` function L100-110 — `()` — Text and string utilities for scripts.
-  `test_extract_no_match` function L113-116 — `()` — Text and string utilities for scripts.
-  `test_truncate` function L119-123 — `()` — Text and string utilities for scripts.
-  `test_word_count` function L126-129 — `()` — Text and string utilities for scripts.
-  `test_estimate_tokens` function L132-134 — `()` — Text and string utilities for scripts.

### crates/arawn-server/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-server/src/auth.rs

- pub `Identity` enum L28-33 — `Token | Tailscale` — Authenticated identity.
- pub `is_token` function L37-39 — `(&self) -> bool` — Check if this is a token identity.
- pub `is_tailscale` function L42-44 — `(&self) -> bool` — Check if this is a Tailscale identity.
- pub `tailscale_user` function L47-52 — `(&self) -> Option<&str>` — Get the Tailscale user if this is a Tailscale identity.
- pub `AuthError` enum L61-70 — `MissingToken | InvalidFormat | InvalidToken | TailscaleNotAllowed` — Authentication error.
- pub `TAILSCALE_USER_HEADER` variable L108 — `: &str` — Header name for Tailscale user login.
- pub `auth_middleware` function L146-157 — `( State(state): State<AppState>, mut request: Request<Body>, next: Next, ) -> Re...` — Authentication middleware function.
- pub `AuthIdentity` struct L224 — `-` — Type alias for extracting the authenticated identity from request extensions.
-  `Identity` type L35-53 — `= Identity` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `AuthError` type L72-81 — `= AuthError` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `fmt` function L73-80 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `AuthError` type L83 — `= AuthError` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `AuthError` type L85-101 — `impl IntoResponse for AuthError` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `into_response` function L86-100 — `(self) -> Response` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `constant_time_eq` function L119-137 — `(a: &str, b: &str) -> bool` — Compare two strings in constant time.
-  `validate_request` function L160-200 — `(request: &Request<Body>, state: &AppState) -> Result<Identity, AuthError>` — Validate a request and return the identity.
-  `AuthIdentity` type L226-230 — `= AuthIdentity` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `from` function L227-229 — `(ext: axum::Extension<Identity>) -> Self` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `tests` module L237-500 — `-` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `create_test_state` function L251-263 — `(tailscale_users: Option<Vec<String>>) -> AppState` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `protected_handler` function L265-270 — `(axum::Extension(identity): axum::Extension<Identity>) -> String` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `create_test_router` function L272-280 — `(state: AppState) -> Router` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_with_valid_bearer_token` function L283-304 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_with_invalid_token` function L307-323 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_missing_token` function L326-341 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_invalid_format` function L344-360 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_with_tailscale_allowed` function L363-384 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_with_tailscale_not_allowed` function L387-403 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_tailscale_disabled_ignores_header` function L406-423 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_auth_bearer_takes_precedence_over_tailscale` function L426-449 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_identity_methods` function L452-464 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_constant_time_eq_equal_strings` function L469-476 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_constant_time_eq_different_strings` function L479-484 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_constant_time_eq_different_lengths` function L487-493 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.
-  `test_constant_time_eq_unicode` function L496-499 — `()` — Token comparison uses constant-time comparison to prevent timing attacks.

#### crates/arawn-server/src/config.rs

- pub `DEFAULT_RECONNECT_GRACE_PERIOD` variable L7 — `: Duration` — Default grace period for session reconnect tokens (30 seconds).
- pub `DEFAULT_MAX_WS_MESSAGE_SIZE` variable L10 — `: usize` — Default max message size for WebSocket (1 MB).
- pub `DEFAULT_MAX_BODY_SIZE` variable L13 — `: usize` — Default max body size for REST requests (10 MB).
- pub `DEFAULT_WS_CONNECTIONS_PER_MINUTE` variable L16 — `: u32` — Default WebSocket connections per minute per IP.
- pub `ServerConfig` struct L20-65 — `{ bind_address: SocketAddr, auth_token: Option<String>, tailscale_users: Option<...` — Server configuration.
- pub `new` function L89-94 — `(auth_token: Option<String>) -> Self` — Create a new server config with an optional auth token.
- pub `with_bind_address` function L97-100 — `(mut self, addr: SocketAddr) -> Self` — Set the bind address.
- pub `with_tailscale_users` function L103-106 — `(mut self, users: Vec<String>) -> Self` — Set allowed Tailscale users.
- pub `with_rate_limiting` function L109-112 — `(mut self, enabled: bool) -> Self` — Enable or disable rate limiting.
- pub `with_request_logging` function L115-118 — `(mut self, enabled: bool) -> Self` — Enable or disable request logging.
- pub `with_cors_origins` function L121-124 — `(mut self, origins: Vec<String>) -> Self` — Set CORS allowed origins.
- pub `with_api_rpm` function L127-130 — `(mut self, rpm: u32) -> Self` — Set the API rate limit (requests per minute).
- pub `with_reconnect_grace_period` function L133-136 — `(mut self, duration: Duration) -> Self` — Set the reconnect grace period for session ownership.
- pub `with_max_ws_message_size` function L139-142 — `(mut self, size: usize) -> Self` — Set the maximum WebSocket message size.
- pub `with_max_body_size` function L145-148 — `(mut self, size: usize) -> Self` — Set the maximum REST request body size.
- pub `with_ws_allowed_origins` function L151-154 — `(mut self, origins: Vec<String>) -> Self` — Set allowed origins for WebSocket connections.
- pub `with_ws_connections_per_minute` function L157-160 — `(mut self, rate: u32) -> Self` — Set the maximum WebSocket connections per minute per IP.
-  `ServerConfig` type L67-84 — `impl Default for ServerConfig` — Server configuration.
-  `default` function L68-83 — `() -> Self` — Server configuration.
-  `ServerConfig` type L86-161 — `= ServerConfig` — Server configuration.

#### crates/arawn-server/src/error.rs

- pub `ServerError` enum L14-58 — `Unauthorized | NotFound | BadRequest | RateLimitExceeded | Conflict | ServiceUna...` — Server error type.
- pub `RateLimitError` struct L62-67 — `{ message: String, retry_after: Option<Duration> }` — Rate limit error with optional retry timing.
- pub `new` function L81-86 — `(message: impl Into<String>) -> Self` — Create a new rate limit error.
- pub `with_retry_after` function L89-94 — `(message: impl Into<String>, retry_after: Duration) -> Self` — Create a rate limit error with retry timing.
- pub `retry_after` function L99-112 — `(&self) -> Option<Duration>` — Check if this is a rate limit error and extract retry timing.
- pub `is_rate_limit` function L115-121 — `(&self) -> bool` — Check if this error should be returned as HTTP 429.
- pub `Result` type L164 — `= std::result::Result<T, ServerError>` — Result type for server operations.
- pub `ErrorResponse` struct L168-173 — `{ code: String, message: String }` — Error response body.
-  `RateLimitError` type L69-77 — `= RateLimitError` — Error types for the server.
-  `fmt` function L70-76 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Error types for the server.
-  `RateLimitError` type L79-95 — `= RateLimitError` — Error types for the server.
-  `ServerError` type L97-122 — `= ServerError` — Error types for the server.
-  `ServerError` type L124-137 — `= ServerError` — Error types for the server.
-  `from` function L125-136 — `(e: arawn_domain::WorkstreamError) -> Self` — Error types for the server.
-  `ServerError` type L139-161 — `= ServerError` — Error types for the server.
-  `from` function L140-160 — `(e: arawn_domain::ConfigError) -> Self` — Error types for the server.
-  `ServerError` type L175-235 — `impl IntoResponse for ServerError` — Error types for the server.
-  `into_response` function L176-234 — `(self) -> Response` — Error types for the server.

#### crates/arawn-server/src/lib.rs

- pub `auth` module L29 — `-` — This crate provides the network transport layer for interacting
- pub `config` module L30 — `-` — ```
- pub `error` module L31 — `-` — ```
- pub `ratelimit` module L32 — `-` — ```
- pub `routes` module L33 — `-` — ```
- pub `session_cache` module L34 — `-` — ```
- pub `state` module L35 — `-` — ```
- pub `Server` struct L55-58 — `{ state: AppState }` — The Arawn HTTP/WebSocket server.
- pub `new` function L62-66 — `(agent: Agent, config: ServerConfig) -> Self` — Create a new server with the given agent and configuration.
- pub `from_state` function L69-71 — `(state: AppState) -> Self` — Create a server from a pre-built application state.
- pub `router` function L74-101 — `(&self) -> Router` — Build the router with all routes and middleware.
- pub `run` function L222-241 — `(self) -> Result<()>` — Run the server.
- pub `run_on` function L244-262 — `(self, addr: SocketAddr) -> Result<()>` — Run the server on a specific address (useful for testing).
- pub `bind_address` function L265-267 — `(&self) -> SocketAddr` — Get the configured bind address.
-  `Server` type L60-268 — `= Server` — ```
-  `api_routes` function L106-219 — `(&self) -> Router<AppState>` — API routes (v1).
-  `tests` module L271-323 — `-` — ```
-  `create_test_agent` function L281-288 — `() -> Agent` — ```
-  `test_server_health_endpoint` function L291-309 — `()` — ```
-  `test_server_config_builder` function L312-322 — `()` — ```

#### crates/arawn-server/src/ratelimit.rs

- pub `PerIpRateLimiter` type L26 — `= RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>` — Per-IP rate limiter type alias (keyed by IpAddr).
- pub `SharedRateLimiter` type L29 — `= Arc<PerIpRateLimiter>` — Shared per-IP rate limiter.
- pub `RateLimitConfig` struct L33-40 — `{ chat_rpm: u32, api_rpm: u32, enabled: bool }` — Rate limit configuration.
- pub `create_rate_limiter` function L65-70 — `(requests_per_minute: u32) -> SharedRateLimiter` — Create a per-IP rate limiter with the specified requests per minute.
- pub `rate_limit_middleware` function L123-166 — `( State(state): State<AppState>, request: Request<Body>, next: Next, ) -> Respon...` — Rate limiting middleware for API endpoints.
- pub `request_logging_middleware` function L176-226 — `( State(state): State<AppState>, request: Request<Body>, next: Next, ) -> Respon...` — Structured request logging middleware.
-  `RateLimitConfig` type L42-50 — `impl Default for RateLimitConfig` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `default` function L43-49 — `() -> Self` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `RateLimitError` struct L54-58 — `{ error: String, code: u16, retry_after_seconds: Option<u64> }` — Rate limit error response.
-  `extract_client_ip` function L79-110 — `(request: &Request<Body>) -> IpAddr` — Extract client IP address from request headers.
-  `tests` module L233-320 — `-` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `create_test_state` function L247-258 — `(rate_limiting: bool) -> AppState` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `test_handler` function L260-262 — `() -> &'static str` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `create_test_router` function L264-272 — `(state: AppState) -> Router` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `test_rate_limit_disabled` function L275-289 — `()` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `test_rate_limit_allows_requests` function L292-303 — `()` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `test_create_rate_limiter` function L306-311 — `()` — Provides per-IP rate limiting for API endpoints to prevent abuse.
-  `test_rate_limit_config_default` function L314-319 — `()` — Provides per-IP rate limiting for API endpoints to prevent abuse.

#### crates/arawn-server/src/session_cache.rs

- pub `SessionCacheError` enum L36-47 — `NotFound | WorkstreamNotFound | NoWorkstreamManager | Workstream | Cache` — Error type for session cache operations.
- pub `Result` type L49 — `= std::result::Result<T, SessionCacheError>` — workstream JSONL storage.
- pub `WorkstreamPersistence` struct L56-58 — `{ workstreams: Option<Arc<WorkstreamManager>> }` — Persistence hook that loads/saves sessions from workstream JSONL storage.
- pub `SessionCache` struct L120-125 — `{ inner: SessionCacheImpl<WorkstreamPersistence>, workstreams: Option<Arc<Workst...` — Session cache that loads from and persists to workstream storage.
- pub `new` function L129-131 — `(workstreams: Option<Arc<WorkstreamManager>>) -> Self` — Create a new session cache with default capacity and TTL.
- pub `from_session_config` function L137-142 — `( workstreams: Option<Arc<WorkstreamManager>>, config: &C, ) -> Self` — Create a session cache from a configuration provider.
- pub `with_capacity` function L145-147 — `(workstreams: Option<Arc<WorkstreamManager>>, max_sessions: usize) -> Self` — Create a new session cache with specified capacity.
- pub `with_config` function L150-168 — `( workstreams: Option<Arc<WorkstreamManager>>, max_sessions: usize, ttl: Option<...` — Create a new session cache with full configuration.
- pub `len` function L171-173 — `(&self) -> usize` — Get the current number of cached sessions.
- pub `is_empty` function L176-178 — `(&self) -> bool` — Check if the cache is empty.
- pub `cleanup_expired` function L183-185 — `(&self) -> usize` — Clean up expired sessions.
- pub `get_or_load` function L191-204 — `( &self, session_id: SessionId, workstream_id: &str, ) -> Result<(Session, Strin...` — Get a session from cache or load from workstream.
- pub `create_session` function L207-218 — `(&self, workstream_id: &str) -> (SessionId, Session)` — Create a new session and add it to the cache.
- pub `get_or_create` function L225-250 — `( &self, session_id: Option<SessionId>, workstream_id: &str, ) -> Result<(Sessio...` — Get or create a session.
- pub `contains` function L253-255 — `(&self, session_id: &SessionId) -> bool` — Check if a session exists in cache (and is not expired).
- pub `get` function L259-261 — `(&self, session_id: &SessionId) -> Option<Session>` — Get a session from cache only (no workstream loading).
- pub `get_workstream_id` function L265-267 — `(&self, session_id: &SessionId) -> Option<String>` — Get the workstream ID for a cached session.
- pub `update` function L270-283 — `(&self, session_id: SessionId, session: Session) -> Result<()>` — Update a session in cache.
- pub `save_turn` function L286-328 — `( &self, session_id: SessionId, turn: &Turn, workstream_id: &str, ) -> Result<()...` — Save a completed turn to workstream storage.
- pub `remove` function L331-346 — `(&self, session_id: &SessionId) -> Option<Session>` — Remove a session from cache.
- pub `invalidate` function L349-351 — `(&self, session_id: &SessionId)` — Invalidate a cached session (e.g., after reassignment).
- pub `list_cached` function L354-365 — `(&self) -> Vec<(SessionId, String)>` — List all cached sessions (excludes expired).
- pub `all_sessions` function L368-379 — `(&self) -> std::collections::HashMap<SessionId, Session>` — Get all sessions (for backwards compatibility, excludes expired).
- pub `with_session` function L382-387 — `(&self, session_id: &SessionId, f: F) -> Option<R>` — Direct access to cache for backwards compatibility during migration.
- pub `with_session_mut` function L390-395 — `(&self, session_id: &SessionId, f: F) -> Option<R>` — Direct mutable access to cache for backwards compatibility during migration.
- pub `insert` function L398-404 — `(&self, session_id: SessionId, session: Session, workstream_id: &str)` — Insert a session directly into cache.
-  `DEFAULT_MAX_SESSIONS` variable L29 — `: usize` — Default maximum number of sessions to cache.
-  `DEFAULT_SESSION_TTL` variable L32 — `: Option<Duration>` — Default TTL for sessions (1 hour).
-  `WorkstreamPersistence` type L60-102 — `impl PersistenceHook for WorkstreamPersistence` — workstream JSONL storage.
-  `Value` type L61 — `= Session` — workstream JSONL storage.
-  `load` function L63-86 — `(&self, session_id: &str, context_id: &str) -> SessionStoreResult<Option<Session...` — workstream JSONL storage.
-  `save` function L88-96 — `( &self, _session_id: &str, _context_id: &str, _value: &Session, ) -> SessionSto...` — workstream JSONL storage.
-  `delete` function L98-101 — `(&self, _session_id: &str, _context_id: &str) -> SessionStoreResult<()>` — workstream JSONL storage.
-  `parse_session_id` function L105-109 — `(session_id: &str) -> SessionStoreResult<SessionId>` — Parse a session ID string into a `SessionId`.
-  `SessionCache` type L127-405 — `= SessionCache` — workstream JSONL storage.
-  `convert_reconstructed_to_session` function L408-446 — `( reconstructed: &ReconstructedSession, session_id: SessionId, ) -> Session` — Convert a reconstructed session from workstream to an agent Session.
-  `tests` module L449-635 — `-` — workstream JSONL storage.
-  `test_create_session` function L453-463 — `()` — workstream JSONL storage.
-  `test_get_nonexistent_creates_empty` function L466-477 — `()` — workstream JSONL storage.
-  `test_remove_session` function L480-489 — `()` — workstream JSONL storage.
-  `test_invalidate_session` function L492-500 — `()` — workstream JSONL storage.
-  `test_update_session` function L503-513 — `()` — workstream JSONL storage.
-  `test_list_cached` function L516-524 — `()` — workstream JSONL storage.
-  `test_lru_eviction` function L527-549 — `()` — workstream JSONL storage.
-  `test_lru_access_updates_order` function L552-572 — `()` — workstream JSONL storage.
-  `test_ttl_expiration` function L575-589 — `()` — workstream JSONL storage.
-  `test_ttl_access_resets_timer` function L592-611 — `()` — workstream JSONL storage.
-  `test_cleanup_expired` function L614-634 — `()` — workstream JSONL storage.

#### crates/arawn-server/src/state.rs

- pub `SessionOwners` type L55 — `= Arc<RwLock<HashMap<SessionId, ConnectionId>>>` — Session ownership tracking - maps session IDs to owning connection IDs.
- pub `PendingReconnect` struct L59-64 — `{ token: String, expires_at: std::time::Instant }` — Pending reconnect entry for session ownership recovery after disconnect.
- pub `new` function L68-73 — `(token: String, grace_period: std::time::Duration) -> Self` — Create a new pending reconnect with the given grace period.
- pub `is_expired` function L76-78 — `(&self) -> bool` — Check if this pending reconnect has expired.
- pub `PendingReconnects` type L82 — `= Arc<RwLock<HashMap<SessionId, PendingReconnect>>>` — Pending reconnects storage - maps session IDs to pending reconnect entries.
- pub `SharedMcpManager` type L85 — `= Arc<RwLock<McpManager>>` — Thread-safe MCP manager.
- pub `TaskStatus` enum L94-105 — `Pending | Running | Completed | Failed | Cancelled` — Task status.
- pub `TrackedTask` struct L109-130 — `{ id: String, task_type: String, status: TaskStatus, progress: Option<u8>, messa...` — A tracked task/operation.
- pub `new` function L134-147 — `(id: impl Into<String>, task_type: impl Into<String>) -> Self` — Create a new pending task.
- pub `with_session` function L150-153 — `(mut self, session_id: impl Into<String>) -> Self` — Set the session ID.
- pub `start` function L156-159 — `(&mut self)` — Mark the task as running.
- pub `update_progress` function L162-165 — `(&mut self, progress: u8, message: Option<String>)` — Update progress.
- pub `complete` function L168-173 — `(&mut self, message: Option<String>)` — Mark the task as completed.
- pub `fail` function L176-180 — `(&mut self, error: impl Into<String>)` — Mark the task as failed.
- pub `cancel` function L183-186 — `(&mut self)` — Mark the task as cancelled.
- pub `TaskStore` type L190 — `= Arc<RwLock<HashMap<String, TrackedTask>>>` — In-memory task store.
- pub `WsConnectionTracker` struct L201-204 — `{ connections: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>> }` — Tracks WebSocket connection attempts per IP address.
- pub `new` function L208-212 — `() -> Self` — Create a new connection tracker.
- pub `check_rate` function L218-250 — `(&self, ip: IpAddr, max_per_minute: u32) -> Result<(), Response>` — Check if a new connection from this IP should be allowed.
- pub `cleanup` function L253-265 — `(&self)` — Cleanup old entries from all IPs.
- pub `SharedServices` struct L283-322 — `{ agent: Arc<Agent>, config: Arc<ServerConfig>, rate_limiter: SharedRateLimiter,...` — Immutable services created at startup.
- pub `new` function L326-344 — `(agent: Agent, config: ServerConfig) -> Self` — Create new shared services with the given agent and config.
- pub `with_workstreams` function L347-350 — `(mut self, manager: WorkstreamManager) -> Self` — Configure workstream support.
- pub `with_indexer` function L353-356 — `(mut self, indexer: SessionIndexer) -> Self` — Configure session indexer.
- pub `with_hook_dispatcher` function L359-362 — `(mut self, dispatcher: SharedHookDispatcher) -> Self` — Configure hook dispatcher for lifecycle events.
- pub `with_mcp_manager` function L365-368 — `(mut self, manager: McpManager) -> Self` — Configure MCP manager.
- pub `with_directory_manager` function L371-374 — `(mut self, manager: DirectoryManager) -> Self` — Configure directory manager for path management.
- pub `with_sandbox_manager` function L377-380 — `(mut self, manager: SandboxManager) -> Self` — Configure sandbox manager for shell execution.
- pub `with_file_watcher` function L383-386 — `(mut self, watcher: WatcherHandle) -> Self` — Configure file watcher for filesystem monitoring.
- pub `with_memory_store` function L389-392 — `(mut self, store: Arc<MemoryStore>) -> Self` — Configure memory store for persistent notes and memories.
- pub `with_compressor` function L395-398 — `(mut self, compressor: Compressor) -> Self` — Configure session/workstream compressor.
- pub `build_domain_services` function L404-415 — `(mut self) -> Self` — Build domain services from the configured components.
- pub `domain` function L420-422 — `(&self) -> Option<&Arc<DomainServices>>` — Get the domain services facade.
- pub `allowed_paths` function L427-435 — `( &self, workstream_id: &str, session_id: &str, ) -> Option<Vec<std::path::PathB...` — Get allowed paths for a session based on its workstream.
- pub `path_validator` function L440-448 — `( &self, workstream_id: &str, session_id: &str, ) -> Option<arawn_domain::PathVa...` — Get a PathValidator for a session.
- pub `RuntimeState` struct L468-492 — `{ session_cache: SessionCache, tasks: TaskStore, session_owners: SessionOwners, ...` — Mutable state that changes during operation.
- pub `new` function L496-504 — `() -> Self` — Create new runtime state.
- pub `with_workstream_cache` function L507-515 — `(workstreams: Arc<WorkstreamManager>) -> Self` — Create runtime state with workstream-backed session cache.
- pub `with_session_config` function L518-525 — `( mut self, workstreams: Option<Arc<WorkstreamManager>>, config: &C, ) -> Self` — Configure session cache using a config provider.
- pub `AppState` struct L546-552 — `{ services: SharedServices, runtime: RuntimeState }` — Application state shared across all handlers.
- pub `new` function L556-561 — `(agent: Agent, config: ServerConfig) -> Self` — Create a new application state.
- pub `with_workstreams` function L564-569 — `(mut self, manager: WorkstreamManager) -> Self` — Create application state with workstream support.
- pub `with_indexer` function L572-575 — `(mut self, indexer: SessionIndexer) -> Self` — Create application state with session indexer.
- pub `with_hook_dispatcher` function L578-581 — `(mut self, dispatcher: SharedHookDispatcher) -> Self` — Create application state with hook dispatcher for lifecycle events.
- pub `with_mcp_manager` function L584-587 — `(mut self, manager: McpManager) -> Self` — Create application state with MCP manager.
- pub `with_directory_manager` function L590-593 — `(mut self, manager: DirectoryManager) -> Self` — Create application state with directory manager for path management.
- pub `with_sandbox_manager` function L596-599 — `(mut self, manager: SandboxManager) -> Self` — Create application state with sandbox manager for shell execution.
- pub `with_file_watcher` function L602-605 — `(mut self, watcher: WatcherHandle) -> Self` — Create application state with file watcher for filesystem monitoring.
- pub `with_compressor` function L608-611 — `(mut self, compressor: Compressor) -> Self` — Create application state with session/workstream compressor.
- pub `with_session_config` function L614-618 — `(mut self, config: &C) -> Self` — Configure session cache using a config provider.
- pub `build_domain_services` function L624-627 — `(mut self) -> Self` — Build domain services from the configured components.
- pub `agent` function L633-635 — `(&self) -> &Arc<Agent>` — Get the agent.
- pub `config` function L639-641 — `(&self) -> &Arc<ServerConfig>` — Get the server config.
- pub `rate_limiter` function L645-647 — `(&self) -> &SharedRateLimiter` — Get the rate limiter.
- pub `workstreams` function L651-653 — `(&self) -> Option<&Arc<WorkstreamManager>>` — Get the workstream manager.
- pub `indexer` function L657-659 — `(&self) -> Option<&Arc<SessionIndexer>>` — Get the session indexer.
- pub `hook_dispatcher` function L663-665 — `(&self) -> Option<&SharedHookDispatcher>` — Get the hook dispatcher.
- pub `mcp_manager` function L669-671 — `(&self) -> Option<&SharedMcpManager>` — Get the MCP manager.
- pub `directory_manager` function L675-677 — `(&self) -> Option<&Arc<DirectoryManager>>` — Get the directory manager.
- pub `sandbox_manager` function L681-683 — `(&self) -> Option<&Arc<SandboxManager>>` — Get the sandbox manager.
- pub `file_watcher` function L687-689 — `(&self) -> Option<&Arc<WatcherHandle>>` — Get the file watcher.
- pub `memory_store` function L693-695 — `(&self) -> Option<&Arc<MemoryStore>>` — Get the memory store.
- pub `domain` function L699-701 — `(&self) -> Option<&Arc<DomainServices>>` — Get the domain services facade.
- pub `compressor` function L705-707 — `(&self) -> Option<&Arc<Compressor>>` — Get the compressor.
- pub `session_cache` function L711-713 — `(&self) -> &SessionCache` — Get the session cache.
- pub `tasks` function L717-719 — `(&self) -> &TaskStore` — Get the task store.
- pub `session_owners` function L723-725 — `(&self) -> &SessionOwners` — Get the session owners.
- pub `pending_reconnects` function L729-731 — `(&self) -> &PendingReconnects` — Get the pending reconnects.
- pub `ws_connection_tracker` function L735-737 — `(&self) -> &WsConnectionTracker` — Get the WebSocket connection tracker.
- pub `check_ws_connection_rate` function L742-747 — `(&self, ip: IpAddr) -> Result<(), Response>` — Check WebSocket connection rate for an IP address.
- pub `allowed_paths` function L755-761 — `( &self, workstream_id: &str, session_id: &str, ) -> Option<Vec<std::path::PathB...` — Get allowed paths for a session based on its workstream.
- pub `path_validator` function L766-772 — `( &self, workstream_id: &str, session_id: &str, ) -> Option<arawn_domain::PathVa...` — Get a PathValidator for a session.
- pub `get_or_create_session` function L780-783 — `(&self, session_id: Option<SessionId>) -> SessionId` — Get or create a session by ID.
- pub `get_or_create_session_in_workstream` function L789-830 — `( &self, session_id: Option<SessionId>, workstream_id: &str, ) -> SessionId` — Get or create a session in a specific workstream.
- pub `close_session` function L836-948 — `(&self, session_id: SessionId) -> bool` — Close a session: remove it from the cache and trigger background indexing/compression.
- pub `get_session` function L951-961 — `(&self, session_id: SessionId, workstream_id: &str) -> Option<Session>` — Get session from cache (loading from workstream if needed).
- pub `update_session` function L964-966 — `(&self, session_id: SessionId, session: Session)` — Update session in cache.
- pub `invalidate_session` function L969-971 — `(&self, session_id: SessionId)` — Invalidate a cached session (e.g., after workstream reassignment).
- pub `try_claim_session_ownership` function L981-1015 — `( &self, session_id: SessionId, connection_id: ConnectionId, ) -> bool` — Try to claim ownership of a session for a connection.
- pub `is_session_owner` function L1018-1025 — `( &self, session_id: SessionId, connection_id: ConnectionId, ) -> bool` — Check if a connection owns a session.
- pub `release_session_ownership` function L1031-1044 — `( &self, session_id: SessionId, connection_id: ConnectionId, ) -> bool` — Release ownership of a session.
- pub `release_all_session_ownerships` function L1053-1092 — `( &self, connection_id: ConnectionId, reconnect_tokens: &HashMap<SessionId, Stri...` — Release all session ownerships held by a connection, creating pending reconnects.
- pub `try_reclaim_with_token` function L1098-1142 — `( &self, session_id: SessionId, token: &str, connection_id: ConnectionId, ) -> O...` — Try to reclaim session ownership using a reconnect token.
- pub `cleanup_expired_pending_reconnects` function L1147-1168 — `(&self) -> usize` — Clean up expired pending reconnects.
- pub `has_pending_reconnect` function L1171-1178 — `(&self, session_id: SessionId) -> bool` — Check if a session has a pending reconnect (ownership held for reconnection).
-  `PendingReconnect` type L66-79 — `= PendingReconnect` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `TrackedTask` type L132-187 — `= TrackedTask` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `WS_RATE_WINDOW` variable L197 — `: std::time::Duration` — Sliding window duration for WebSocket rate limiting.
-  `WsConnectionTracker` type L206-266 — `= WsConnectionTracker` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `WsConnectionTracker` type L268-272 — `impl Default for WsConnectionTracker` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `default` function L269-271 — `() -> Self` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `SharedServices` type L324-449 — `= SharedServices` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `RuntimeState` type L494-526 — `= RuntimeState` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `RuntimeState` type L528-532 — `impl Default for RuntimeState` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `default` function L529-531 — `() -> Self` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `AppState` type L554-1179 — `= AppState` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `session_to_messages` function L1186-1195 — `(session: &Session) -> Vec<(String, String)>` — Convert a session's turns into owned `(role, content)` pairs.
-  `messages_as_refs` function L1198-1203 — `(messages: &[(String, String)]) -> Vec<(&str, &str)>` — Convert owned message pairs to borrowed slices for the indexer API.
-  `tests` module L1206-1612 — `-` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `create_test_state` function L1211-1219 — `() -> AppState` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_to_messages_empty` function L1222-1226 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_to_messages_with_turns` function L1229-1251 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_to_messages_incomplete_turn` function L1254-1262 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_messages_as_refs` function L1265-1272 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_close_session_removes_session` function L1275-1287 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_close_session_nonexistent_returns_false` function L1290-1294 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_close_session_without_indexer` function L1297-1314 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_default_state_has_no_indexer` function L1317-1320 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_ownership_first_claimer_wins` function L1323-1339 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_ownership_release` function L1342-1362 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_ownership_release_all_on_disconnect` function L1365-1407 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_session_ownership_same_connection_reclaim` function L1410-1421 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_reconnect_token_wrong_token_rejected` function L1424-1446 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_reconnect_token_new_connection_can_reclaim` function L1449-1470 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_reconnect_cleanup_expired` function L1473-1511 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_shared_services_builder` function L1514-1528 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_runtime_state_defaults` function L1531-1536 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_convenience_accessors` function L1539-1550 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_ws_connection_tracker_allows_under_limit` function L1555-1564 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_ws_connection_tracker_rate_limits` function L1567-1579 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_ws_connection_tracker_per_ip` function L1582-1597 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.
-  `test_ws_connection_tracker_cleanup` function L1600-1611 — `()` — - See `docs/src/architecture/concurrency.md` for the full concurrency guide.

### crates/arawn-server/src/routes

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-server/src/routes/agents.rs

- pub `AgentToolInfo` struct L24-29 — `{ name: String, description: String }` — Information about a tool available to an agent.
- pub `AgentSummary` struct L33-42 — `{ id: String, name: String, is_default: bool, tool_count: usize }` — Summary information about an agent.
- pub `AgentDetail` struct L46-57 — `{ id: String, name: String, is_default: bool, tools: Vec<AgentToolInfo>, capabil...` — Detailed information about an agent.
- pub `AgentCapabilities` struct L61-68 — `{ streaming: bool, tool_use: bool, max_context_length: Option<usize> }` — Agent capabilities.
- pub `ListAgentsResponse` struct L72-77 — `{ agents: Vec<AgentSummary>, total: usize }` — Response for listing agents.
- pub `list_agents_handler` function L94-112 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, ) ->...` — multi-agent support.
- pub `get_agent_handler` function L129-165 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — multi-agent support.
-  `tests` module L172-299 — `-` — multi-agent support.
-  `create_test_state` function L187-196 — `() -> AppState` — multi-agent support.
-  `create_test_router` function L198-207 — `(state: AppState) -> Router` — multi-agent support.
-  `test_list_agents` function L210-234 — `()` — multi-agent support.
-  `test_get_agent` function L237-261 — `()` — multi-agent support.
-  `test_get_agent_not_found` function L264-280 — `()` — multi-agent support.
-  `test_list_agents_requires_auth` function L283-298 — `()` — multi-agent support.

#### crates/arawn-server/src/routes/chat.rs

- pub `ChatRequest` struct L39-46 — `{ session_id: Option<String>, message: String }` — Request body for chat endpoints.
- pub `ChatResponse` struct L50-66 — `{ session_id: String, response: String, tool_calls: Vec<ToolCallInfo>, truncated...` — Response from the synchronous chat endpoint.
- pub `ToolCallInfo` struct L70-77 — `{ id: String, name: String, success: bool }` — Simplified tool call info for API response.
- pub `UsageInfo` struct L81-86 — `{ input_tokens: u32, output_tokens: u32 }` — Token usage info.
- pub `chat_handler` function L111-210 — `( State(state): State<AppState>, Extension(identity): Extension<Identity>, Json(...` — Provides both synchronous and streaming (SSE) endpoints for chat.
- pub `chat_stream_handler` function L227-347 — `( State(state): State<AppState>, Extension(identity): Extension<Identity>, Json(...` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `MAX_MESSAGE_BYTES` variable L93 — `: usize` — Maximum message size in bytes (100KB).
-  `SseSessionEvent` struct L354-356 — `{ session_id: String }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `SseTextEvent` struct L359-361 — `{ content: String }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `SseToolStartEvent` struct L364-367 — `{ id: String, name: String }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `SseToolOutputEvent` struct L370-373 — `{ id: String, content: String }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `SseToolEndEvent` struct L376-380 — `{ id: String, success: bool, content: String }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `SseDoneEvent` struct L383-385 — `{ iterations: u32 }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `SseErrorEvent` struct L388-390 — `{ message: String }` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `tests` module L397-607 — `-` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `create_test_state` function L412-421 — `() -> AppState` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `create_test_router` function L423-432 — `(state: AppState) -> Router` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `test_chat_new_session` function L435-462 — `()` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `test_chat_existing_session` function L465-530 — `()` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `test_chat_requires_auth` function L533-550 — `()` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `test_chat_stream_returns_sse` function L553-575 — `()` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `test_chat_request_parsing` function L578-587 — `()` — Provides both synchronous and streaming (SSE) endpoints for chat.
-  `test_chat_response_serialization` function L590-606 — `()` — Provides both synchronous and streaming (SSE) endpoints for chat.

#### crates/arawn-server/src/routes/commands.rs

- pub `CommandResult` type L32 — `= std::result::Result<T, CommandError>` — Result type for command execution.
- pub `CommandError` struct L36-41 — `{ code: String, message: String }` — Error type for command execution.
- pub `not_found` function L44-49 — `(msg: impl Into<String>) -> Self` — The `/` syntax is purely client-side presentation.
- pub `invalid_params` function L51-56 — `(msg: impl Into<String>) -> Self` — The `/` syntax is purely client-side presentation.
- pub `execution_failed` function L58-63 — `(msg: impl Into<String>) -> Self` — The `/` syntax is purely client-side presentation.
- pub `CommandHandler` interface L78-91 — `{ fn name(), fn description(), fn execute() }` — Command handler trait.
- pub `CommandOutput` enum L96-107 — `Text | Json | Progress | Completed | Error` — Output from command execution.
- pub `CommandRegistry` struct L125-127 — `{ handlers: HashMap<String, Arc<dyn CommandHandler>> }` — Registry for command handlers.
- pub `new` function L131-135 — `() -> Self` — Create a new empty registry.
- pub `with_compact` function L138-145 — `(model: &str) -> Self` — Create a registry with standard commands using the given model.
- pub `register` function L148-151 — `(&mut self, handler: H)` — Register a command handler.
- pub `get` function L154-156 — `(&self, name: &str) -> Option<Arc<dyn CommandHandler>>` — Get a command handler by name.
- pub `list` function L159-167 — `(&self) -> Vec<CommandInfo>` — List all registered commands.
- pub `SharedCommandRegistry` type L171 — `= Arc<RwLock<CommandRegistry>>` — Thread-safe command registry.
- pub `CommandInfo` struct L179-184 — `{ name: String, description: String }` — Command info for API responses.
- pub `ListCommandsResponse` struct L188-191 — `{ commands: Vec<CommandInfo> }` — Response for listing commands.
- pub `CompactRequest` struct L195-201 — `{ session_id: String, force: bool }` — Request to execute the compact command.
- pub `CompactResponse` struct L205-219 — `{ compacted: bool, turns_compacted: Option<usize>, tokens_before: Option<usize>,...` — Response from compact command.
- pub `CompactEvent` enum L240-251 — `Started | Summarizing | Completed | Cancelled | Error` — SSE event for compact progress.
- pub `CompactCommand` struct L258-260 — `{ config: CompactorConfig }` — The compact command handler.
- pub `new` function L264-266 — `(config: CompactorConfig) -> Self` — Create with the given config.
- pub `list_commands_handler` function L363-372 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, ) ->...` — The `/` syntax is purely client-side presentation.
- pub `compact_command_handler` function L388-412 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Json...` — The `/` syntax is purely client-side presentation.
- pub `compact_command_stream_handler` function L428-506 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Json...` — The `/` syntax is purely client-side presentation.
-  `CommandError` type L43-64 — `= CommandError` — The `/` syntax is purely client-side presentation.
-  `ServerError` type L66-74 — `= ServerError` — The `/` syntax is purely client-side presentation.
-  `from` function L67-73 — `(e: CommandError) -> Self` — The `/` syntax is purely client-side presentation.
-  `CommandRegistry` type L129-168 — `= CommandRegistry` — The `/` syntax is purely client-side presentation.
-  `CompactResponse` type L221-235 — `= CompactResponse` — The `/` syntax is purely client-side presentation.
-  `from` function L222-234 — `(result: CompactionResult) -> Self` — The `/` syntax is purely client-side presentation.
-  `CompactCommand` type L262-267 — `= CompactCommand` — The `/` syntax is purely client-side presentation.
-  `CompactCommand` type L270-346 — `impl CommandHandler for CompactCommand` — The `/` syntax is purely client-side presentation.
-  `name` function L271-273 — `(&self) -> &str` — The `/` syntax is purely client-side presentation.
-  `description` function L275-277 — `(&self) -> &str` — The `/` syntax is purely client-side presentation.
-  `execute` function L279-345 — `( &self, state: &AppState, params: serde_json::Value, ) -> CommandResult<Command...` — The `/` syntax is purely client-side presentation.
-  `tests` module L513-718 — `-` — The `/` syntax is purely client-side presentation.
-  `create_test_state` function L519-527 — `() -> AppState` — The `/` syntax is purely client-side presentation.
-  `test_command_registry_new` function L530-533 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_command` function L535-540 — `() -> CompactCommand` — The `/` syntax is purely client-side presentation.
-  `test_command_registry_with_compact` function L543-548 — `()` — The `/` syntax is purely client-side presentation.
-  `test_command_registry_register_and_lookup` function L551-558 — `()` — The `/` syntax is purely client-side presentation.
-  `test_command_registry_list` function L561-569 — `()` — The `/` syntax is purely client-side presentation.
-  `test_command_registry_get_nonexistent` function L572-575 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_command_metadata` function L578-582 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_command_invalid_session_id` function L585-596 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_command_session_not_found` function L599-610 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_command_no_compaction_needed` function L613-642 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_command_force` function L645-675 — `()` — The `/` syntax is purely client-side presentation.
-  `test_compact_response_from_result` function L678-693 — `()` — The `/` syntax is purely client-side presentation.
-  `test_command_error_types` function L696-705 — `()` — The `/` syntax is purely client-side presentation.
-  `test_command_error_to_server_error` function L708-717 — `()` — The `/` syntax is purely client-side presentation.

#### crates/arawn-server/src/routes/config.rs

- pub `ConfigFeatures` struct L19-30 — `{ workstreams_enabled: bool, memory_enabled: bool, mcp_enabled: bool, rate_limit...` — Server feature flags.
- pub `ConfigLimits` struct L34-37 — `{ max_concurrent_requests: Option<u32> }` — Server limits configuration.
- pub `ConfigResponse` struct L41-57 — `{ version: String, api_version: String, features: ConfigFeatures, limits: Config...` — Server configuration response.
- pub `get_config_handler` function L74-106 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, ) ->...` — Exposes non-sensitive server configuration for clients.
-  `tests` module L113-195 — `-` — Exposes non-sensitive server configuration for clients.
-  `create_test_state` function L128-137 — `() -> AppState` — Exposes non-sensitive server configuration for clients.
-  `create_test_router` function L139-147 — `(state: AppState) -> Router` — Exposes non-sensitive server configuration for clients.
-  `test_get_config` function L150-176 — `()` — Exposes non-sensitive server configuration for clients.
-  `test_get_config_requires_auth` function L179-194 — `()` — Exposes non-sensitive server configuration for clients.

#### crates/arawn-server/src/routes/health.rs

- pub `HealthResponse` struct L11-16 — `{ status: String, version: String }` — Health check response.
- pub `health` function L27-32 — `() -> Json<HealthResponse>` — Health check endpoints.
- pub `health_routes` function L35-37 — `() -> Router<AppState>` — Create health check routes.
-  `tests` module L40-72 — `-` — Health check endpoints.
-  `test_health_endpoint` function L49-71 — `()` — Health check endpoints.

#### crates/arawn-server/src/routes/mcp.rs

- pub `AddServerRequest` struct L43-95 — `{ name: String, transport: String, command: String, url: Option<String>, args: V...` — Request to add a new MCP server.
- pub `AddServerResponse` struct L103-112 — `{ name: String, connected: bool, tool_count: Option<usize>, error: Option<String...` — Response after adding a server.
- pub `ServerInfo` struct L116-125 — `{ name: String, connected: bool, tool_count: usize, tools: Vec<String> }` — Information about a connected MCP server.
- pub `ListServersResponse` struct L129-136 — `{ servers: Vec<ServerInfo>, total: usize, connected: usize }` — Response for listing servers.
- pub `ToolInfo` struct L140-149 — `{ name: String, description: Option<String>, input_schema: Option<serde_json::Va...` — Information about a tool.
- pub `ListToolsResponse` struct L153-158 — `{ server: String, tools: Vec<ToolInfo> }` — Response for listing tools from a server.
- pub `RemoveServerResponse` struct L162-167 — `{ name: String, removed: bool }` — Response after removing a server.
- pub `add_server_handler` function L187-284 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Json...` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
- pub `remove_server_handler` function L304-329 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
- pub `list_servers_handler` function L345-388 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, ) ->...` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
- pub `list_server_tools_handler` function L410-459 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
- pub `connect_server_handler` function L480-507 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
- pub `disconnect_server_handler` function L528-548 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `default_connect` function L97-99 — `() -> bool` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `tests` module L555-989 — `-` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `create_test_state_with_mcp` function L571-581 — `() -> AppState` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `create_test_state_without_mcp` function L583-592 — `() -> AppState` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `create_test_router` function L594-612 — `(state: AppState) -> Router` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_list_servers_empty` function L615-639 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_list_servers_mcp_disabled` function L642-658 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_add_server_missing_name` function L661-679 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_add_server_missing_command` function L682-702 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_add_server_success_no_connect` function L705-733 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_add_server_duplicate` function L736-763 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_remove_server_not_found` function L766-783 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_remove_server_success` function L786-814 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_list_server_tools_not_found` function L817-833 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_list_server_tools_not_connected` function L836-859 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_connect_server_not_found` function L862-879 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_disconnect_server_not_found` function L882-899 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_disconnect_server_success` function L902-926 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_add_http_server` function L929-961 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server
-  `test_add_http_server_missing_url` function L964-988 — `()` — - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server

#### crates/arawn-server/src/routes/memory.rs

- pub `Note` struct L29-44 — `{ id: String, title: Option<String>, content: String, tags: Vec<String>, created...` — A note (API representation).
- pub `CreateNoteRequest` struct L48-57 — `{ content: String, title: Option<String>, tags: Vec<String> }` — Request to create a note.
- pub `ListNotesQuery` struct L61-64 — `{ tag: Option<String> }` — Query params for listing notes.
- pub `UpdateNoteRequest` struct L68-78 — `{ title: Option<String>, content: Option<String>, tags: Option<Vec<String>> }` — Request to update a note.
- pub `ListNotesResponse` struct L82-91 — `{ notes: Vec<Note>, total: usize, limit: usize, offset: usize }` — Response for listing notes.
- pub `MemorySearchQuery` struct L95-103 — `{ q: String, limit: usize, session_id: Option<String> }` — Query params for memory search.
- pub `MemorySearchResult` struct L111-129 — `{ id: String, content_type: String, content: String, session_id: Option<String>,...` — Memory search result item.
- pub `MemorySearchResponse` struct L133-145 — `{ results: Vec<MemorySearchResult>, query: String, count: usize, degraded: bool ...` — Response for memory search.
- pub `StoreMemoryRequest` struct L153-175 — `{ content: String, content_type: String, session_id: Option<String>, metadata: H...` — Request to store a memory directly.
- pub `StoreMemoryResponse` struct L187-194 — `{ id: String, content_type: String, message: String }` — Response after storing a memory.
- pub `create_note_handler` function L236-256 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Json...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `list_notes_handler` function L274-309 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Quer...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `get_note_handler` function L327-343 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `update_note_handler` function L362-394 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `delete_note_handler` function L412-431 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `memory_search_handler` function L457-531 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Quer...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `store_memory_handler` function L546-578 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Json...` — backed by `arawn-memory::MemoryStore` for persistent storage.
- pub `delete_memory_handler` function L596-614 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `default_limit` function L105-107 — `() -> usize` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `default_content_type` function L177-179 — `() -> String` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `default_confidence` function L181-183 — `() -> f32` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `require_memory_store` function L201-205 — `(state: &AppState) -> Result<&Arc<MemoryStore>, ServerError>` — Get the memory store from app state, returning 503 if not configured.
-  `to_api_note` function L208-217 — `(note: MemoryNote) -> Note` — Convert an `arawn_memory::Note` to the API `Note` type.
-  `tests` module L621-1107 — `-` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `create_test_state` function L636-648 — `() -> AppState` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `create_test_router` function L650-667 — `(state: AppState) -> Router` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_create_note` function L670-698 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_create_note_with_title` function L701-727 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_get_note` function L730-760 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_get_note_not_found` function L763-779 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_update_note` function L782-815 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_delete_note` function L818-844 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_list_notes` function L847-879 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_list_notes_with_tag_filter` function L882-912 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_memory_search` function L915-938 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_memory_search_with_store` function L941-972 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_memory_search_includes_notes` function L975-1005 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_memory_search_requires_auth` function L1008-1023 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_notes_require_memory_store` function L1026-1051 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_store_memory` function L1054-1080 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.
-  `test_delete_memory` function L1083-1106 — `()` — backed by `arawn-memory::MemoryStore` for persistent storage.

#### crates/arawn-server/src/routes/mod.rs

- pub `agents` module L3 — `-` — API routes.
- pub `chat` module L4 — `-` — API routes.
- pub `commands` module L5 — `-` — API routes.
- pub `config` module L6 — `-` — API routes.
- pub `health` module L7 — `-` — API routes.
- pub `mcp` module L8 — `-` — API routes.
- pub `memory` module L9 — `-` — API routes.
- pub `openapi` module L10 — `-` — API routes.
- pub `pagination` module L11 — `-` — API routes.
- pub `sessions` module L12 — `-` — API routes.
- pub `tasks` module L13 — `-` — API routes.
- pub `workstreams` module L14 — `-` — API routes.
- pub `ws` module L15 — `-` — API routes.

#### crates/arawn-server/src/routes/openapi.rs

- pub `ApiDoc` struct L168 — `-` — OpenAPI documentation configuration.
- pub `swagger_ui` function L189-191 — `() -> SwaggerUi` — Create the Swagger UI router.
-  `SecurityAddon` struct L171 — `-` — Add bearer token security scheme.
-  `SecurityAddon` type L173-186 — `= SecurityAddon` — OpenAPI documentation configuration.
-  `modify` function L174-185 — `(&self, openapi: &mut utoipa::openapi::OpenApi)` — OpenAPI documentation configuration.

#### crates/arawn-server/src/routes/pagination.rs

- pub `MAX_PAGE_SIZE` variable L7 — `: usize` — Maximum allowed page size.
- pub `DEFAULT_PAGE_SIZE` variable L10 — `: usize` — Default page size.
- pub `PaginationParams` struct L26-36 — `{ limit: usize, offset: usize }` — Pagination query parameters for list endpoints.
- pub `effective_limit` function L49-51 — `(&self) -> usize` — Get the effective limit, clamped to MAX_PAGE_SIZE.
- pub `paginate` function L54-61 — `(&self, items: &[T]) -> (Vec<T>, usize)` — Apply pagination to a slice, returning (paginated_items, total).
- pub `PaginatedResponse` struct L72-81 — `{ items: Vec<T>, total: usize, limit: usize, offset: usize }` — Paginated response wrapper.
- pub `new` function L85-92 — `(items: Vec<T>, total: usize, params: &PaginationParams) -> Self` — Create a new paginated response from pagination params and total count.
-  `PaginationParams` type L38-45 — `impl Default for PaginationParams` — Shared pagination types for list endpoints.
-  `default` function L39-44 — `() -> Self` — Shared pagination types for list endpoints.
-  `PaginationParams` type L47-62 — `= PaginationParams` — Shared pagination types for list endpoints.
-  `default_limit` function L64-66 — `() -> usize` — Shared pagination types for list endpoints.
-  `tests` module L96-168 — `-` — Shared pagination types for list endpoints.
-  `test_default_pagination` function L100-104 — `()` — Shared pagination types for list endpoints.
-  `test_effective_limit_clamped` function L107-113 — `()` — Shared pagination types for list endpoints.
-  `test_effective_limit_minimum` function L116-122 — `()` — Shared pagination types for list endpoints.
-  `test_paginate_basic` function L125-134 — `()` — Shared pagination types for list endpoints.
-  `test_paginate_with_offset` function L137-146 — `()` — Shared pagination types for list endpoints.
-  `test_paginate_offset_beyond_end` function L149-158 — `()` — Shared pagination types for list endpoints.
-  `test_paginate_empty` function L161-167 — `()` — Shared pagination types for list endpoints.

#### crates/arawn-server/src/routes/sessions.rs

- pub `CreateSessionRequest` struct L25-33 — `{ title: Option<String>, metadata: HashMap<String, serde_json::Value> }` — Request to create a new session.
- pub `UpdateSessionRequest` struct L37-48 — `{ title: Option<String>, metadata: Option<HashMap<String, serde_json::Value>>, w...` — Request to update a session.
- pub `MessageInfo` struct L52-59 — `{ role: String, content: String, timestamp: String }` — Message info for conversation history.
- pub `SessionMessagesResponse` struct L63-70 — `{ session_id: String, messages: Vec<MessageInfo>, count: usize }` — Response containing session messages.
- pub `SessionSummary` struct L74-86 — `{ id: String, title: Option<String>, turn_count: usize, created_at: String, upda...` — Summary info for a session.
- pub `SessionDetail` struct L90-112 — `{ id: String, turns: Vec<TurnInfo>, created_at: String, updated_at: String, meta...` — Full session details.
- pub `TurnInfo` struct L116-129 — `{ id: String, user_message: String, assistant_response: Option<String>, tool_cal...` — Turn info for API responses.
- pub `ListSessionsResponse` struct L133-142 — `{ sessions: Vec<SessionSummary>, total: usize, limit: usize, offset: usize }` — Response for list sessions.
- pub `create_session_handler` function L160-214 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Json...` — Session management endpoints.
- pub `list_sessions_handler` function L228-293 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Quer...` — Session management endpoints.
- pub `get_session_handler` function L308-365 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — Session management endpoints.
- pub `delete_session_handler` function L382-397 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — Session management endpoints.
- pub `update_session_handler` function L414-620 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — Session management endpoints.
- pub `get_session_messages_handler` function L635-696 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — Session management endpoints.
-  `parse_session_id` function L702-706 — `(s: &str) -> Result<SessionId, ServerError>` — Session management endpoints.
-  `session_to_detail` function L708-710 — `(session: &Session) -> SessionDetail` — Session management endpoints.
-  `session_to_detail_with_migration` function L712-739 — `( session: &Session, workstream_id: Option<String>, files_migrated: Option<usize...` — Session management endpoints.
-  `tests` module L746-1156 — `-` — Session management endpoints.
-  `create_test_state` function L761-770 — `() -> AppState` — Session management endpoints.
-  `create_test_router` function L772-790 — `(state: AppState) -> Router` — Session management endpoints.
-  `test_list_sessions_empty` function L793-818 — `()` — Session management endpoints.
-  `test_list_sessions_with_data` function L821-848 — `()` — Session management endpoints.
-  `test_get_session` function L851-876 — `()` — Session management endpoints.
-  `test_get_session_not_found` function L879-895 — `()` — Session management endpoints.
-  `test_get_session_invalid_id` function L898-914 — `()` — Session management endpoints.
-  `test_delete_session` function L917-939 — `()` — Session management endpoints.
-  `test_delete_session_not_found` function L942-959 — `()` — Session management endpoints.
-  `test_create_session` function L962-988 — `()` — Session management endpoints.
-  `test_create_session_with_metadata` function L991-1017 — `()` — Session management endpoints.
-  `test_update_session` function L1020-1046 — `()` — Session management endpoints.
-  `test_update_session_not_found` function L1049-1067 — `()` — Session management endpoints.
-  `test_get_session_messages_empty` function L1070-1096 — `()` — Session management endpoints.
-  `test_get_session_messages_with_data` function L1099-1136 — `()` — Session management endpoints.
-  `test_get_session_messages_not_found` function L1139-1155 — `()` — Session management endpoints.

#### crates/arawn-server/src/routes/tasks.rs

- pub `ListTasksQuery` struct L23-33 — `{ status: Option<String>, session_id: Option<String>, limit: usize }` — Query params for listing tasks.
- pub `TaskSummary` struct L41-54 — `{ id: String, task_type: String, status: TaskStatus, progress: Option<u8>, creat...` — Summary info for a task.
- pub `TaskDetail` struct L58-86 — `{ id: String, task_type: String, status: TaskStatus, progress: Option<u8>, messa...` — Full task details.
- pub `ListTasksResponse` struct L90-95 — `{ tasks: Vec<TaskSummary>, total: usize }` — Response for listing tasks.
- pub `list_tasks_handler` function L157-199 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Quer...` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
- pub `get_task_handler` function L216-228 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
- pub `cancel_task_handler` function L246-268 — `( State(state): State<AppState>, Extension(_identity): Extension<Identity>, Path...` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `default_limit` function L35-37 — `() -> usize` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `task_to_summary` function L101-109 — `(task: &TrackedTask) -> TaskSummary` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `task_to_detail` function L111-124 — `(task: &TrackedTask) -> TaskDetail` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `parse_status` function L126-135 — `(s: &str) -> Option<TaskStatus>` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `tests` module L275-528 — `-` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `create_test_state` function L290-299 — `() -> AppState` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `create_test_router` function L301-313 — `(state: AppState) -> Router` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_list_tasks_empty` function L316-339 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_list_tasks_with_data` function L342-374 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_get_task` function L377-407 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_get_task_not_found` function L410-426 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_cancel_task` function L429-459 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_cancel_completed_task_fails` function L462-488 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.
-  `test_list_tasks_filter_by_status` function L491-527 — `()` — Provides endpoints for listing, viewing, and cancelling long-running tasks.

#### crates/arawn-server/src/routes/workstreams.rs

- pub `CreateWorkstreamRequest` struct L37-46 — `{ title: String, default_model: Option<String>, tags: Vec<String> }`
- pub `WorkstreamResponse` struct L49-69 — `{ id: String, title: String, summary: Option<String>, state: String, default_mod...`
- pub `WorkstreamListResponse` struct L72-81 — `{ workstreams: Vec<WorkstreamResponse>, total: usize, limit: usize, offset: usiz...`
- pub `SendMessageRequest` struct L84-99 — `{ role: Option<String>, content: String, metadata: Option<String> }`
- pub `MessageResponse` struct L102-118 — `{ id: String, workstream_id: String, session_id: Option<String>, role: String, c...`
- pub `MessageListResponse` struct L121-130 — `{ messages: Vec<MessageResponse>, total: usize, limit: usize, offset: usize }`
- pub `MessageQuery` struct L133-135 — `{ since: Option<String> }`
- pub `ListWorkstreamsQuery` struct L138-142 — `{ include_archived: bool }`
- pub `PromoteRequest` struct L145-154 — `{ title: String, tags: Vec<String>, default_model: Option<String> }`
- pub `PromoteFileRequest` struct L158-163 — `{ source: String, destination: String }` — Request to promote a file from work/ to production/.
- pub `PromoteFileResponse` struct L167-175 — `{ path: String, bytes: u64, renamed: bool }` — Response from file promotion.
- pub `ExportFileRequest` struct L179-184 — `{ source: String, destination: String }` — Request to export a file from production/ to external path.
- pub `ExportFileResponse` struct L188-193 — `{ exported_to: String, bytes: u64 }` — Response from file export.
- pub `CloneRepoRequest` struct L197-203 — `{ url: String, name: Option<String> }` — Request to clone a git repository into production/.
- pub `CloneRepoResponse` struct L207-212 — `{ path: String, commit: String }` — Response from git clone operation.
- pub `SessionUsageResponse` struct L216-221 — `{ id: String, mb: f64 }` — Per-session disk usage info.
- pub `UsageResponse` struct L225-238 — `{ production_mb: f64, work_mb: f64, sessions: Vec<SessionUsageResponse>, total_m...` — Response from usage stats endpoint.
- pub `CleanupRequest` struct L242-249 — `{ older_than_days: Option<u32>, confirm: bool }` — Request to clean up work directory files.
- pub `CleanupResponse` struct L253-264 — `{ deleted_files: usize, freed_mb: f64, pending_files: usize, requires_confirmati...` — Response from cleanup operation.
- pub `UpdateWorkstreamRequest` struct L271-284 — `{ title: Option<String>, summary: Option<String>, default_model: Option<String>,...`
- pub `SessionResponse` struct L287-298 — `{ id: String, workstream_id: String, started_at: String, ended_at: Option<String...`
- pub `SessionListResponse` struct L301-310 — `{ sessions: Vec<SessionResponse>, total: usize, limit: usize, offset: usize }`
- pub `create_workstream_handler` function L364-382 — `( State(state): State<AppState>, Json(req): Json<CreateWorkstreamRequest>, ) -> ...`
- pub `list_workstreams_handler` function L400-425 — `( State(state): State<AppState>, Query(query): Query<ListWorkstreamsQuery>, Quer...`
- pub `get_workstream_handler` function L443-454 — `( State(state): State<AppState>, Path(id): Path<String>, ) -> Result<Json<Workst...`
- pub `delete_workstream_handler` function L472-482 — `( State(state): State<AppState>, Path(id): Path<String>, ) -> Result<StatusCode,...`
- pub `update_workstream_handler` function L501-524 — `( State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<UpdateW...`
- pub `list_workstream_sessions_handler` function L543-571 — `( State(state): State<AppState>, Path(id): Path<String>, Query(pagination): Quer...`
- pub `send_message_handler` function L591-612 — `( State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<SendMes...`
- pub `list_messages_handler` function L633-662 — `( State(state): State<AppState>, Path(id): Path<String>, Query(query): Query<Mes...`
- pub `promote_handler` function L681-699 — `( State(state): State<AppState>, Path(id): Path<String>, Json(req): Json<Promote...`
- pub `promote_file_handler` function L719-774 — `( State(state): State<AppState>, Path(workstream_id): Path<String>, Json(req): J...`
- pub `export_file_handler` function L794-837 — `( State(state): State<AppState>, Path(workstream_id): Path<String>, Json(req): J...`
- pub `clone_repo_handler` function L858-909 — `( State(state): State<AppState>, Path(workstream_id): Path<String>, Json(req): J...`
- pub `get_usage_handler` function L927-967 — `( State(state): State<AppState>, Path(workstream_id): Path<String>, ) -> Result<...`
- pub `cleanup_handler` function L989-1021 — `( State(state): State<AppState>, Path(workstream_id): Path<String>, Json(req): J...`
- pub `CompressResponse` struct L1025-1030 — `{ summary: String, sessions_compressed: usize }` — Response from compression operation.
- pub `compress_workstream_handler` function L1050-1093 — `( State(state): State<AppState>, Path(id): Path<String>, ) -> Result<Json<Compre...`
-  `validate_id` function L25-32 — `(id: &str) -> Result<(), ServerError>` — Validate a workstream ID from a URL path parameter.
-  `is_zero` function L266-268 — `(v: &usize) -> bool`
-  `get_manager` function L314-318 — `(state: &AppState) -> Result<&Arc<WorkstreamManager>, ServerError>`
-  `to_workstream_response` function L320-335 — `( ws: &arawn_domain::Workstream, tags: Option<Vec<String>>, ) -> WorkstreamRespo...`
-  `to_message_response` function L337-347 — `(msg: &WorkstreamMessage) -> MessageResponse`

### crates/arawn-server/src/routes/ws

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-server/src/routes/ws/connection.rs

- pub `ConnectionId` struct L20 — `-` — Unique identifier for a WebSocket connection.
- pub `new` function L24-26 — `() -> Self` — Create a new unique connection ID.
- pub `IDLE_TIMEOUT` variable L43 — `: Duration` — Idle timeout for WebSocket connections (5 minutes).
- pub `ConnectionState` struct L46-58 — `{ id: ConnectionId, authenticated: bool, subscriptions: std::collections::HashSe...` — State for a WebSocket connection.
- pub `new` function L62-70 — `() -> Self` — Create a new connection state.
- pub `handle_socket` function L86-217 — `(socket: WebSocket, state: AppState, addr: SocketAddr)` — Handle a WebSocket connection.
- pub `send_message` function L220-229 — `( sender: &mut futures::stream::SplitSink<WebSocket, Message>, msg: ServerMessag...` — Send a message over the WebSocket.
-  `ConnectionId` type L22-27 — `= ConnectionId` — WebSocket connection lifecycle and state management.
-  `ConnectionId` type L29-33 — `impl Default for ConnectionId` — WebSocket connection lifecycle and state management.
-  `default` function L30-32 — `() -> Self` — WebSocket connection lifecycle and state management.
-  `ConnectionId` type L35-39 — `= ConnectionId` — WebSocket connection lifecycle and state management.
-  `fmt` function L36-38 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — WebSocket connection lifecycle and state management.
-  `ConnectionState` type L60-71 — `= ConnectionState` — WebSocket connection lifecycle and state management.
-  `ConnectionState` type L73-77 — `impl Default for ConnectionState` — WebSocket connection lifecycle and state management.
-  `default` function L74-76 — `() -> Self` — WebSocket connection lifecycle and state management.
-  `ConnectionState` type L79-83 — `impl Drop for ConnectionState` — WebSocket connection lifecycle and state management.
-  `drop` function L80-82 — `(&mut self)` — WebSocket connection lifecycle and state management.

#### crates/arawn-server/src/routes/ws/handlers.rs

- pub `MessageResponse` enum L15-22 — `Single | Stream | None` — Response from handling a message.
- pub `handle_message` function L25-56 — `( msg: ClientMessage, conn_state: &mut ConnectionState, app_state: &AppState, ) ...` — Handle a client message.
-  `handle_auth` function L59-78 — `( token: String, conn_state: &mut ConnectionState, app_state: &AppState, ) -> Me...` — Handle authentication.
-  `handle_subscribe` function L85-155 — `( session_id: String, reconnect_token: Option<String>, conn_state: &mut Connecti...` — Handle session subscription.
-  `handle_unsubscribe` function L161-177 — `( session_id: String, conn_state: &mut ConnectionState, app_state: &AppState, ) ...` — Handle session unsubscription.
-  `handle_cancel` function L180-202 — `(session_id: String, conn_state: &mut ConnectionState) -> MessageResponse` — Handle cancellation request.
-  `handle_command` function L205-278 — `( command: String, args: serde_json::Value, conn_state: &ConnectionState, app_st...` — Handle command execution.
-  `inject_session_context` function L281-304 — `( mut args: serde_json::Value, conn_state: &ConnectionState, ) -> serde_json::Va...` — Inject session context from the connection state if not provided in args.
-  `handle_chat` function L310-522 — `( session_id: Option<String>, workstream_id: Option<String>, message: String, co...` — Handle chat message.
-  `tests` module L525-583 — `-` — WebSocket message handlers.
-  `test_inject_session_context_null_args` function L529-537 — `()` — WebSocket message handlers.
-  `test_inject_session_context_with_subscription` function L540-554 — `()` — WebSocket message handlers.
-  `test_inject_session_context_preserves_existing` function L557-568 — `()` — WebSocket message handlers.
-  `test_inject_session_context_preserves_other_args` function L571-582 — `()` — WebSocket message handlers.

#### crates/arawn-server/src/routes/ws/mod.rs

- pub `ws_handler` function L47-76 — `( ws: WebSocketUpgrade, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<Socke...` — GET /ws - WebSocket upgrade handler.
-  `connection` module L20 — `-` — This module provides WebSocket support for the Arawn server, enabling:
-  `handlers` module L21 — `-` — - Connection rate limiting prevents connection floods
-  `protocol` module L22 — `-` — - Connection rate limiting prevents connection floods
-  `validate_origin` function L82-132 — `(headers: &HeaderMap, allowed_origins: &[String]) -> Result<(), Response>` — Validate the Origin header against allowed origins.
-  `is_localhost_origin` function L135-143 — `(origin: &str) -> bool` — Check if an origin is a localhost-class origin (no port specified).
-  `origin_matches_ignoring_port` function L148-157 — `(origin: &str, allowed: &str) -> bool` — Check if an origin matches an allowed origin ignoring port differences.
-  `tests` module L160-394 — `-` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_exact_match` function L164-170 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_not_allowed` function L173-179 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_missing_header` function L182-188 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_wildcard_subdomain` function L191-197 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_wildcard_no_match` function L200-206 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_multiple_allowed` function L209-218 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_localhost_with_port` function L223-229 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_localhost_any_port` function L232-247 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_localhost_bare` function L250-256 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_127_0_0_1_with_port` function L259-265 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_ipv6_localhost_with_port` function L268-274 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_localhost_wrong_scheme` function L277-284 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_non_localhost_no_port_match` function L287-294 — `()` — - Connection rate limiting prevents connection floods
-  `test_validate_origin_default_localhost_variants` function L297-345 — `()` — - Connection rate limiting prevents connection floods
-  `test_is_localhost_origin` function L350-361 — `()` — - Connection rate limiting prevents connection floods
-  `test_origin_matches_ignoring_port` function L364-393 — `()` — - Connection rate limiting prevents connection floods

#### crates/arawn-server/src/routes/ws/protocol.rs

- pub `ClientMessage` enum L8-51 — `Chat | Subscribe | Unsubscribe | Ping | Auth | Cancel | Command` — Messages from client to server.
- pub `ServerMessage` enum L56-182 — `AuthResult | SessionCreated | ChatChunk | ToolStart | ToolOutput | ToolEnd | Err...` — Messages from server to client.
- pub `error` function L186-191 — `(code: impl Into<String>, message: impl Into<String>) -> Self` — Create an error message.
- pub `auth_success` function L194-199 — `() -> Self` — Create an auth success message.
- pub `auth_failure` function L202-207 — `(error: impl Into<String>) -> Self` — Create an auth failure message.
- pub `command_progress` function L210-220 — `( command: impl Into<String>, message: impl Into<String>, percent: Option<u8>, )...` — Create a command progress message.
- pub `command_success` function L223-229 — `(command: impl Into<String>, result: serde_json::Value) -> Self` — Create a successful command result message.
- pub `command_failure` function L232-238 — `(command: impl Into<String>, error: impl Into<String>) -> Self` — Create a failed command result message.
- pub `context_info` function L241-266 — `( session_id: impl Into<String>, current_tokens: usize, max_tokens: usize, ) -> ...` — Create a context info message.
- pub `fs_change` function L269-276 — `(event: &arawn_domain::FsChangeEvent) -> Self` — Create a filesystem change notification from an FsChangeEvent.
- pub `subscribe_ack` function L279-289 — `( session_id: impl Into<String>, owner: bool, reconnect_token: Option<String>, )...` — Create a subscription acknowledgment message.
- pub `disk_pressure` function L292-300 — `(event: &arawn_domain::DiskPressureEvent) -> Self` — Create a disk pressure alert from a DiskPressureEvent.
-  `ServerMessage` type L184-301 — `= ServerMessage` — WebSocket protocol types for client-server communication.
-  `tests` module L304-591 — `-` — WebSocket protocol types for client-server communication.
-  `test_client_message_parsing` function L308-347 — `()` — WebSocket protocol types for client-server communication.
-  `test_command_message_parsing` function L350-373 — `()` — WebSocket protocol types for client-server communication.
-  `test_server_message_serialization` function L376-400 — `()` — WebSocket protocol types for client-server communication.
-  `test_auth_messages` function L403-412 — `()` — WebSocket protocol types for client-server communication.
-  `test_subscribe_ack_serialization` function L415-431 — `()` — WebSocket protocol types for client-server communication.
-  `test_command_progress_serialization` function L434-446 — `()` — WebSocket protocol types for client-server communication.
-  `test_command_result_serialization` function L449-469 — `()` — WebSocket protocol types for client-server communication.
-  `test_context_info_serialization` function L472-494 — `()` — WebSocket protocol types for client-server communication.
-  `test_context_info_boundary_conditions` function L497-533 — `()` — WebSocket protocol types for client-server communication.
-  `test_fs_change_serialization` function L536-560 — `()` — WebSocket protocol types for client-server communication.
-  `test_disk_pressure_serialization` function L563-590 — `()` — WebSocket protocol types for client-server communication.

### crates/arawn-server/tests

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-server/tests/chat_integration.rs

-  `common` module L5 — `-` — These tests verify chat requests work through the server API.
-  `test_chat_endpoint_returns_response` function L11-35 — `() -> Result<()>` — These tests verify chat requests work through the server API.
-  `test_chat_creates_session` function L38-75 — `() -> Result<()>` — These tests verify chat requests work through the server API.
-  `test_chat_with_existing_session` function L78-127 — `() -> Result<()>` — These tests verify chat requests work through the server API.
-  `test_chat_requires_message` function L130-143 — `() -> Result<()>` — These tests verify chat requests work through the server API.
-  `test_session_can_be_retrieved` function L146-180 — `() -> Result<()>` — These tests verify chat requests work through the server API.
-  `test_session_not_found` function L183-199 — `() -> Result<()>` — These tests verify chat requests work through the server API.
-  `test_session_can_be_deleted` function L202-241 — `() -> Result<()>` — These tests verify chat requests work through the server API.

#### crates/arawn-server/tests/context_management.rs

-  `common` module L5 — `-` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_command_requires_session_id` function L15-31 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_command_invalid_session_id` function L34-52 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_command_session_not_found` function L55-74 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_command_no_compaction_needed` function L77-112 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_command_with_many_turns` function L115-171 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_force_flag` function L174-208 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_list_commands_includes_compact` function L211-224 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_stream_session_not_found` function L231-245 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_stream_returns_sse` function L248-303 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_sessions_have_context_info` function L314-346 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_multiple_turns_accumulate_context` function L357-396 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compaction_response_structure` function L407-451 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_compact_same_session_concurrent` function L458-494 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.
-  `test_command_list_via_api` function L505-524 — `() -> Result<()>` — These tests verify context tracking, session compaction, and the /compact command.

#### crates/arawn-server/tests/memory_integration.rs

-  `common` module L8 — `-` — These tests verify memory persistence through the server API.
-  `test_create_note` function L14-36 — `() -> Result<()>` — isolated.
-  `test_list_notes_returns_array` function L39-56 — `() -> Result<()>` — isolated.
-  `test_create_note_appears_in_list` function L59-90 — `() -> Result<()>` — isolated.
-  `test_create_note_requires_content` function L93-105 — `() -> Result<()>` — isolated.
-  `test_note_has_created_at` function L108-128 — `() -> Result<()>` — isolated.
-  `test_note_with_tags` function L131-156 — `() -> Result<()>` — isolated.
-  `test_memory_search_endpoint` function L159-186 — `() -> Result<()>` — isolated.
-  `test_memory_search_finds_matching_notes` function L189-229 — `() -> Result<()>` — isolated.

#### crates/arawn-server/tests/server_integration.rs

-  `common` module L5 — `-` — These tests verify the server starts correctly and handles requests.
-  `test_server_starts_and_responds_to_health` function L10-18 — `() -> Result<()>` — These tests verify the server starts correctly and handles requests.
-  `test_server_health_returns_version` function L21-37 — `() -> Result<()>` — These tests verify the server starts correctly and handles requests.
-  `test_api_requires_auth` function L40-53 — `() -> Result<()>` — These tests verify the server starts correctly and handles requests.
-  `test_api_accepts_valid_auth` function L56-65 — `() -> Result<()>` — These tests verify the server starts correctly and handles requests.
-  `test_api_rejects_invalid_auth` function L68-82 — `() -> Result<()>` — These tests verify the server starts correctly and handles requests.
-  `test_multiple_servers_different_ports` function L85-100 — `() -> Result<()>` — These tests verify the server starts correctly and handles requests.

#### crates/arawn-server/tests/validation_integration.rs

- pub `plugin_manifest_missing_name` function L29-34 — `() -> serde_json::Value` — Create an invalid plugin manifest missing required fields.
- pub `plugin_manifest_invalid_name` function L37-42 — `() -> serde_json::Value` — Create an invalid plugin manifest with non-kebab-case name.
- pub `plugin_manifest_invalid_version` function L45-50 — `() -> serde_json::Value` — Create an invalid plugin manifest with bad version format.
- pub `shell_params_missing_command` function L53-57 — `() -> serde_json::Value` — Create tool parameters with missing required field.
- pub `shell_params_empty_command` function L60-65 — `() -> serde_json::Value` — Create tool parameters with empty command.
- pub `shell_params_invalid_timeout` function L68-73 — `() -> serde_json::Value` — Create tool parameters with invalid timeout.
- pub `shell_params_timeout_too_large` function L76-81 — `() -> serde_json::Value` — Create tool parameters with out of range timeout.
- pub `memory_store_empty_content` function L84-89 — `() -> serde_json::Value` — Create memory store params with empty content.
- pub `memory_store_invalid_importance` function L92-97 — `() -> serde_json::Value` — Create memory store params with invalid importance.
- pub `web_search_zero_results` function L100-105 — `() -> serde_json::Value` — Create web search params with zero max_results.
- pub `file_read_empty_path` function L108-112 — `() -> serde_json::Value` — Create file read params with empty path.
-  `common` module L15 — `-` — These tests verify that validation works correctly at interface boundaries,
-  `fixtures` module L25-113 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `plugin_tests` module L119-289 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_missing_name_rejected` function L124-138 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_invalid_name_format_rejected` function L141-159 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_invalid_version_rejected` function L162-179 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_missing_path_detected` function L182-204 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_capability_mismatch_detected` function L207-226 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_name_edge_cases` function L229-257 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_manifest_version_edge_cases` function L260-288 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `tool_tests` module L295-612 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `test_shell_params_missing_command` function L304-331 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_shell_params_empty_command` function L334-351 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_shell_params_zero_timeout` function L354-371 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_shell_params_timeout_too_large` function L374-399 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_memory_store_empty_content` function L402-419 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_memory_store_invalid_importance` function L422-439 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_web_search_zero_results` function L442-459 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_web_search_too_many_results` function L462-470 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_file_read_empty_path` function L473-487 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_file_write_empty_path` function L490-498 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_file_write_missing_content` function L501-520 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_think_params_empty_thought` function L523-528 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_delegate_params_empty_task` function L531-536 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_memory_recall_empty_query` function L539-544 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_memory_recall_zero_limit` function L547-555 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_memory_recall_limit_too_large` function L558-566 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_valid_params_accepted` function L569-597 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_parameter_error_into_agent_error` function L600-611 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `llm_tests` module L618-704 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `test_missing_field_error_is_critical` function L622-625 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_invalid_tool_use_error_is_critical` function L628-631 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_invalid_token_count_is_not_critical` function L634-640 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_malformed_content_is_not_critical` function L643-649 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_error_messages_are_actionable` function L652-676 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_multiple_errors_aggregated` function L679-692 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_validation_error_into_llm_error` function L695-703 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `memory_tests` module L710-868 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `test_empty_content_rejected` function L718-721 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_null_byte_content_rejected` function L724-727 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_valid_content_accepted` function L730-734 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_confidence_range_validation` function L737-754 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_embedding_dimension_validation` function L757-770 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_embedding_nan_rejected` function L773-780 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_embedding_infinity_rejected` function L783-790 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_session_id_validation` function L793-809 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_full_memory_validation` function L812-834 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_validation_error_into_memory_error` function L837-845 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_error_messages_are_descriptive` function L848-867 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `output_tests` module L874-1021 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `test_default_config` function L882-887 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_tool_specific_configs` function L890-902 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_truncation` function L905-920 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_no_truncation_for_small_output` function L923-931 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_binary_content_detected` function L934-946 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_control_chars_stripped` function L949-960 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_null_bytes_stripped` function L963-972 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_json_depth_validation` function L975-997 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `create_deep_json` function L981-987 — `(depth: usize) -> serde_json::Value` — - Output sanitization (oversized/binary content handled correctly)
-  `test_truncation_preserves_utf8` function L1000-1010 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_custom_truncation_message` function L1013-1020 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `integration_tests` module L1027-1082 — `-` — - Output sanitization (oversized/binary content handled correctly)
-  `test_server_starts_with_validation` function L1031-1036 — `() -> Result<()>` — - Output sanitization (oversized/binary content handled correctly)
-  `test_error_chain_plugin_to_user` function L1039-1051 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_error_chain_tool_to_user` function L1054-1066 — `()` — - Output sanitization (oversized/binary content handled correctly)
-  `test_error_chain_memory_to_user` function L1069-1081 — `()` — - Output sanitization (oversized/binary content handled correctly)

### crates/arawn-server/tests/common

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-server/tests/common/mod.rs

- pub `TestServer` struct L19-30 — `{ addr: SocketAddr, token: String, client: Client, _handle: JoinHandle<()>, temp...` — A test server that runs in the background.
- pub `start` function L34-36 — `() -> Result<Self>` — Start a new test server with default configuration.
- pub `start_with_responses` function L39-101 — `(responses: Vec<String>) -> Result<Self>` — Start a new test server with mock responses.
- pub `base_url` function L104-106 — `(&self) -> String` — Get the base URL for the server.
- pub `get` function L109-113 — `(&self, path: &str) -> reqwest::RequestBuilder` — Get an authenticated request builder.
- pub `post` function L116-120 — `(&self, path: &str) -> reqwest::RequestBuilder` — Get an authenticated POST request builder.
- pub `delete` function L123-127 — `(&self, path: &str) -> reqwest::RequestBuilder` — Get an authenticated DELETE request builder.
- pub `health` function L130-137 — `(&self) -> Result<bool>` — Check if server is healthy.
-  `TestServer` type L32-138 — `= TestServer` — Common test utilities for integration tests.
-  `find_available_port` function L141-146 — `() -> Result<SocketAddr>` — Find an available port for the test server.
-  `wait_for_server` function L149-167 — `(client: &Client, addr: SocketAddr) -> Result<()>` — Wait for the server to become ready.

### crates/arawn-session/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-session/src/cache.rs

- pub `CacheEntry` struct L27-39 — `{ value: V, context_id: String, cached_at: Instant, dirty: bool }` — Entry stored in the cache.
- pub `new` function L43-50 — `(value: V, context_id: String) -> Self` — Create a new cache entry.
- pub `mark_dirty` function L53-55 — `(&mut self)` — Mark the entry as dirty (has unsaved changes).
- pub `mark_clean` function L58-60 — `(&mut self)` — Mark the entry as clean (saved).
- pub `SessionCache` struct L98-101 — `{ inner: Arc<RwLock<CacheInner<P>>>, config: CacheConfig }` — Session cache with LRU eviction and optional TTL.
- pub `new` function L105-107 — `(config: CacheConfig) -> Self` — Create a new session cache with no persistence backend.
- pub `with_persistence` function L112-126 — `(config: CacheConfig, persistence: P) -> Self` — Create a new session cache with a persistence backend.
- pub `config` function L129-131 — `(&self) -> &CacheConfig` — Get the cache configuration.
- pub `len` function L134-136 — `(&self) -> usize` — Get the current number of cached sessions.
- pub `is_empty` function L139-141 — `(&self) -> bool` — Check if the cache is empty.
- pub `get_or_load` function L147-191 — `(&self, session_id: &str, context_id: &str) -> Result<P::Value>` — Get a session from cache or load from persistence.
- pub `insert` function L197-227 — `(&self, session_id: &str, context_id: &str, value: P::Value) -> Result<()>` — Insert a session into the cache.
- pub `update` function L230-261 — `( &self, session_id: &str, context_id: &str, value: P::Value, persist: bool, ) -...` — Update a session in the cache and optionally persist.
- pub `save` function L264-276 — `(&self, session_id: &str) -> Result<()>` — Save a session to persistence.
- pub `contains` function L279-282 — `(&self, session_id: &str) -> bool` — Check if a session exists in cache (without loading).
- pub `peek` function L285-292 — `(&self, session_id: &str) -> Option<P::Value>` — Peek at a session value without updating LRU order or TTL.
- pub `peek_entry` function L295-302 — `(&self, session_id: &str) -> Option<CacheEntry<P::Value>>` — Peek at a cache entry without updating LRU order or TTL.
- pub `peek_context_id` function L305-312 — `(&self, session_id: &str) -> Option<String>` — Get the context_id for a cached session without updating LRU.
- pub `remove` function L315-326 — `(&self, session_id: &str, context_id: &str) -> Result<Option<P::Value>>` — Remove a session from cache and persistence.
- pub `invalidate` function L329-336 — `(&self, session_id: &str)` — Invalidate a session (remove from cache only, don't delete from persistence).
- pub `cleanup_expired` function L342-359 — `(&self) -> usize` — Clean up expired sessions.
- pub `list_cached` function L362-370 — `(&self) -> Vec<(String, String)>` — List all cached session IDs with their context IDs.
- pub `stats` function L373-380 — `(&self) -> CacheStats` — Get cache statistics.
- pub `for_each` function L383-394 — `(&self, mut f: F) -> Vec<R>` — Iterate over all non-expired entries, calling the provided closure.
- pub `with_mut` function L397-411 — `(&self, session_id: &str, f: F) -> Option<R>` — Mutable access to a cached entry's value.
- pub `with_ref` function L414-424 — `(&self, session_id: &str, f: F) -> Option<R>` — Read-only access to a cached entry's value.
- pub `CacheStats` struct L438-447 — `{ size: usize, capacity: usize, ttl_tracked: usize }` — Cache statistics.
-  `CacheInner` struct L64-73 — `{ lru: LruCache<String, CacheEntry<P::Value>>, ttl: TtlTracker, persistence: P }` — Inner state protected by RwLock.
-  `clone` function L428-433 — `(&self) -> Self` — Session cache with LRU eviction and TTL support.
-  `tests` module L450-707 — `-` — Session cache with LRU eviction and TTL support.
-  `test_insert_and_get` function L455-466 — `()` — Session cache with LRU eviction and TTL support.
-  `test_not_found` function L469-475 — `()` — Session cache with LRU eviction and TTL support.
-  `test_lru_eviction` function L478-502 — `()` — Session cache with LRU eviction and TTL support.
-  `test_lru_access_updates_order` function L505-529 — `()` — Session cache with LRU eviction and TTL support.
-  `test_ttl_expiration` function L532-551 — `()` — Session cache with LRU eviction and TTL support.
-  `test_touch_resets_ttl` function L554-577 — `()` — Session cache with LRU eviction and TTL support.
-  `test_invalidate` function L580-592 — `()` — Session cache with LRU eviction and TTL support.
-  `test_cleanup_expired` function L595-622 — `()` — Session cache with LRU eviction and TTL support.
-  `test_stats` function L625-640 — `()` — Session cache with LRU eviction and TTL support.
-  `test_peek_context_id` function L643-655 — `()` — Session cache with LRU eviction and TTL support.
-  `test_with_mut` function L658-676 — `()` — Session cache with LRU eviction and TTL support.
-  `test_with_ref` function L679-691 — `()` — Session cache with LRU eviction and TTL support.
-  `test_for_each` function L694-706 — `()` — Session cache with LRU eviction and TTL support.

#### crates/arawn-session/src/config.rs

- pub `DEFAULT_MAX_SESSIONS` variable L7 — `: usize` — Default maximum number of sessions to cache.
- pub `DEFAULT_TTL` variable L10 — `: Option<Duration>` — Default TTL for sessions (none by default - sessions don't expire).
- pub `CacheConfig` struct L26-40 — `{ max_sessions: usize, ttl: Option<Duration>, enable_cleanup_task: bool, cleanup...` — Configuration for the session cache.
- pub `new` function L55-57 — `() -> Self` — Create a new configuration with default values.
- pub `with_max_sessions` function L60-63 — `(mut self, max: usize) -> Self` — Set the maximum number of sessions to cache.
- pub `with_ttl` function L66-69 — `(mut self, ttl: Duration) -> Self` — Set the TTL for cached sessions.
- pub `without_ttl` function L72-75 — `(mut self) -> Self` — Disable TTL (sessions don't expire based on time).
- pub `with_cleanup_task` function L78-81 — `(mut self, enabled: bool) -> Self` — Enable or disable the background cleanup task.
- pub `with_cleanup_interval` function L84-87 — `(mut self, interval: Duration) -> Self` — Set the cleanup interval.
-  `CacheConfig` type L42-51 — `impl Default for CacheConfig` — Configuration for the session cache.
-  `default` function L43-50 — `() -> Self` — Configuration for the session cache.
-  `CacheConfig` type L53-88 — `= CacheConfig` — Configuration for the session cache.

#### crates/arawn-session/src/error.rs

- pub `Error` enum L5-25 — `NotFound | ContextNotFound | NoPersistence | Persistence | Expired` — Error type for session cache operations.
- pub `Result` type L28 — `= std::result::Result<T, Error>` — Result type for session cache operations.

#### crates/arawn-session/src/lib.rs

-  `cache` module L20 — `-` — This crate provides a generic caching layer for sessions with:
-  `config` module L21 — `-` — ```
-  `error` module L22 — `-` — ```
-  `persistence` module L23 — `-` — ```
-  `ttl` module L24 — `-` — ```

#### crates/arawn-session/src/persistence.rs

- pub `SessionData` struct L16-31 — `{ id: String, context_id: String, state: Vec<u8>, created_at: Option<chrono::Dat...` — Data container for session state.
- pub `new` function L35-43 — `(id: impl Into<String>, context_id: impl Into<String>, state: Vec<u8>) -> Self` — Create a new session data container.
- pub `with_created_at` function L46-49 — `(mut self, ts: chrono::DateTime<chrono::Utc>) -> Self` — Set creation timestamp.
- pub `with_updated_at` function L52-55 — `(mut self, ts: chrono::DateTime<chrono::Utc>) -> Self` — Set update timestamp.
- pub `PersistenceHook` interface L66-93 — `{ fn load(), fn save(), fn delete(), fn on_evict() }` — Trait for persistence backends.
- pub `NoPersistence` struct L97 — `-` — A no-op persistence hook for in-memory only caching.
-  `SessionData` type L33-56 — `= SessionData` — (e.g., a rich `Session` object) without serialization overhead.
-  `on_evict` function L90-92 — `(&self, _session_id: &str, _context_id: &str) -> Result<()>` — Called when a session is evicted from cache due to LRU or TTL.
-  `NoPersistence` type L99-113 — `impl PersistenceHook for NoPersistence` — (e.g., a rich `Session` object) without serialization overhead.
-  `Value` type L100 — `= SessionData` — (e.g., a rich `Session` object) without serialization overhead.
-  `load` function L102-104 — `(&self, _session_id: &str, _context_id: &str) -> Result<Option<SessionData>>` — (e.g., a rich `Session` object) without serialization overhead.
-  `save` function L106-108 — `(&self, _session_id: &str, _context_id: &str, _value: &SessionData) -> Result<()...` — (e.g., a rich `Session` object) without serialization overhead.
-  `delete` function L110-112 — `(&self, _session_id: &str, _context_id: &str) -> Result<()>` — (e.g., a rich `Session` object) without serialization overhead.

#### crates/arawn-session/src/ttl.rs

- pub `TtlTracker` struct L8-14 — `{ access_times: HashMap<String, Instant>, ttl: Option<Duration> }` — Tracks last access times for TTL-based expiration.
- pub `new` function L18-23 — `(ttl: Option<Duration>) -> Self` — Create a new TTL tracker with the given duration.
- pub `touch` function L26-29 — `(&mut self, session_id: &str)` — Record an access for a session (resets its TTL timer).
- pub `is_expired` function L32-42 — `(&self, session_id: &str) -> bool` — Check if a session has expired.
- pub `remove` function L45-47 — `(&mut self, session_id: &str)` — Remove tracking for a session.
- pub `get_expired` function L50-62 — `(&self) -> Vec<String>` — Get all expired session IDs.
- pub `drain_expired` function L65-71 — `(&mut self) -> Vec<String>` — Remove all expired entries and return their IDs.
- pub `len` function L74-76 — `(&self) -> usize` — Get the number of tracked sessions.
- pub `is_empty` function L79-81 — `(&self) -> bool` — Check if there are no tracked sessions.
- pub `clear` function L84-86 — `(&mut self)` — Clear all tracking data.
- pub `ttl` function L89-91 — `(&self) -> Option<Duration>` — Get the configured TTL.
- pub `set_ttl` function L94-96 — `(&mut self, ttl: Option<Duration>)` — Update the TTL configuration.
-  `TtlTracker` type L16-97 — `= TtlTracker` — TTL tracking for session expiration.
-  `tests` module L100-169 — `-` — TTL tracking for session expiration.
-  `test_no_ttl_never_expires` function L105-111 — `()` — TTL tracking for session expiration.
-  `test_touch_resets_timer` function L114-129 — `()` — TTL tracking for session expiration.
-  `test_expiration` function L132-141 — `()` — TTL tracking for session expiration.
-  `test_drain_expired` function L144-155 — `()` — TTL tracking for session expiration.
-  `test_remove` function L158-168 — `()` — TTL tracking for session expiration.

### crates/arawn-tui/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-tui/src/app.rs

- pub `PendingAction` enum L29-46 — `CreateWorkstream | RenameWorkstream | DeleteSession | DeleteWorkstream | Refresh...` — Pending async actions to be executed in the main loop.
- pub `InputMode` enum L50-58 — `Chat | NewWorkstream | RenameWorkstream` — Input mode determines what the input field is being used for.
- pub `ChatMessage` struct L63-70 — `{ is_user: bool, content: String, streaming: bool }` — A chat message for display.
- pub `ToolExecution` struct L74-91 — `{ id: String, name: String, args: String, output: String, running: bool, success...` — A tool execution for display.
- pub `App` struct L94-179 — `{ server_url: String, ws_client: WsClient, api: ArawnClient, connection_status: ...` — Main application state.
- pub `ContextState` struct L183-192 — `{ current_tokens: usize, max_tokens: usize, percent: u8, status: String }` — Context usage state for display in status bar.
- pub `UsageStats` struct L196-213 — `{ workstream_id: String, workstream_name: String, is_scratch: bool, production_b...` — Disk usage statistics for a workstream.
- pub `format_size` function L217-227 — `(bytes: u64) -> String` — Format size as human-readable string.
- pub `production_size` function L230-232 — `(&self) -> String` — Get formatted production size.
- pub `work_size` function L235-237 — `(&self) -> String` — Get formatted work size.
- pub `total_size` function L240-242 — `(&self) -> String` — Get formatted total size.
- pub `limit_size` function L245-251 — `(&self) -> String` — Get formatted limit.
- pub `DiskWarning` struct L256-269 — `{ workstream: String, level: String, usage_bytes: u64, limit_bytes: u64, percent...` — A disk usage warning.
- pub `new` function L276-330 — `(server_url: String, log_buffer: LogBuffer) -> Result<Self>` — Create a new App instance.
- pub `run` function L343-389 — `(&mut self, terminal: &mut Tui) -> Result<()>` — Run the main application loop.
-  `MAX_MESSAGES` variable L8 — `: usize` — Maximum number of chat messages to retain (prevents unbounded memory growth).
-  `MAX_TOOLS` variable L11 — `: usize` — Maximum number of tool executions to retain per response.
-  `UsageStats` type L215-252 — `= UsageStats` — Application state and main loop.
-  `App` type L271-2105 — `= App` — Application state and main loop.
-  `push_message` function L333-335 — `(&mut self, message: ChatMessage)` — Push a message (BoundedVec handles eviction automatically).
-  `push_tool` function L338-340 — `(&mut self, tool: ToolExecution)` — Push a tool execution (BoundedVec handles eviction automatically).
-  `process_pending_actions` function L392-434 — `(&mut self)` — Process pending async actions.
-  `do_create_workstream` function L437-473 — `(&mut self, title: &str)` — Create a workstream via API.
-  `do_rename_workstream` function L476-504 — `(&mut self, id: &str, new_title: &str)` — Rename a workstream via API.
-  `do_delete_session` function L507-528 — `(&mut self, id: &str)` — Delete a session via API.
-  `do_delete_workstream` function L531-553 — `(&mut self, id: &str)` — Delete a workstream via API.
-  `do_fetch_workstream_sessions` function L556-612 — `(&mut self, workstream_id: &str)` — Fetch sessions for a specific workstream.
-  `do_fetch_session_messages` function L615-645 — `(&mut self, session_id: &str)` — Fetch message history for a session.
-  `do_move_session_to_workstream` function L648-685 — `(&mut self, session_id: &str, workstream_id: &str)` — Move a session to a different workstream via API.
-  `refresh_sidebar_data` function L688-737 — `(&mut self)` — Refresh sidebar data from the server API.
-  `handle_server_message` function L740-971 — `(&mut self, msg: ServerMessage)` — Handle a message from the server.
-  `handle_key` function L974-1068 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle keyboard input.
-  `handle_input_key` function L1071-1269 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle input-focused key events.
-  `scroll_chat_up` function L1276-1279 — `(&mut self, lines: usize)` — Scroll chat up by the given number of lines.
-  `scroll_chat_down` function L1285-1289 — `(&mut self, lines: usize)` — Scroll chat down by the given number of lines.
-  `update_command_popup` function L1292-1302 — `(&mut self)` — Update the command popup based on current input.
-  `send_command` function L1305-1342 — `(&mut self)` — Send the current input as a command.
-  `build_command_args` function L1345-1369 — `(&self, cmd: &crate::input::ParsedCommand) -> serde_json::Value` — Build command arguments JSON from parsed command.
-  `get_help_text` function L1372-1378 — `(&self) -> String` — Get help text for available commands.
-  `send_message` function L1381-1415 — `(&mut self)` — Send the current input as a chat message.
-  `handle_sessions_key` function L1418-1462 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle sessions overlay key events.
-  `handle_palette_key` function L1465-1503 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle command palette key events.
-  `execute_action` function L1506-1553 — `(&mut self, action_id: ActionId)` — Execute a palette action.
-  `switch_to_session` function L1556-1586 — `(&mut self, session_id: &str)` — Switch to a different session.
-  `create_new_session` function L1589-1596 — `(&mut self)` — Create a new session.
-  `open_sessions_panel` function L1599-1605 — `(&mut self)` — Open the sessions panel.
-  `handle_overlay_key` function L1608-1644 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle workstreams overlay key events.
-  `handle_tool_pane_key` function L1647-1716 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle tool pane key events.
-  `open_tool_in_editor` function L1722-1751 — `(&mut self)` — Open the selected tool's output in an external pager.
-  `run_pager` function L1754-1788 — `(&self, pager: &str, content: &str) -> std::io::Result<()>` — Run a pager with the given content, suspending and restoring the TUI.
-  `handle_logs_key` function L1791-1823 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle logs panel key events.
-  `clear_pending_deletes` function L1826-1829 — `(&mut self)` — Clear any pending delete confirmations.
-  `handle_sidebar_key` function L1832-2067 — `(&mut self, key: crossterm::event::KeyEvent)` — Handle sidebar key events.
-  `switch_to_workstream` function L2070-2104 — `(&mut self, workstream_name: &str)` — Switch to a different workstream.

#### crates/arawn-tui/src/bounded.rs

- pub `BoundedVec` struct L10-13 — `{ inner: Vec<T>, max_capacity: usize }` — A vector with a maximum capacity that evicts oldest elements when full.
- pub `new` function L20-26 — `(max_capacity: usize) -> Self` — Create a new bounded vector with the specified maximum capacity.
- pub `with_capacity` function L29-35 — `(max_capacity: usize, initial_capacity: usize) -> Self` — Create a new bounded vector with pre-allocated capacity.
- pub `push` function L41-48 — `(&mut self, item: T)` — Push an element, evicting oldest elements if at capacity.
- pub `max_capacity` function L51-53 — `(&self) -> usize` — Get the maximum capacity.
- pub `len` function L56-58 — `(&self) -> usize` — Get the current length.
- pub `is_empty` function L61-63 — `(&self) -> bool` — Check if empty.
- pub `clear` function L66-68 — `(&mut self)` — Clear all elements.
- pub `last` function L71-73 — `(&self) -> Option<&T>` — Get a reference to the last element.
- pub `last_mut` function L76-78 — `(&mut self) -> Option<&mut T>` — Get a mutable reference to the last element.
- pub `iter` function L81-83 — `(&self) -> std::slice::Iter<'_, T>` — Iterate over elements.
- pub `iter_mut` function L86-88 — `(&mut self) -> std::slice::IterMut<'_, T>` — Iterate mutably over elements.
- pub `get` function L91-93 — `(&self, index: usize) -> Option<&T>` — Get element by index.
- pub `get_mut` function L96-98 — `(&mut self, index: usize) -> Option<&mut T>` — Get mutable element by index.
- pub `pop` function L101-103 — `(&mut self) -> Option<T>` — Pop the last element.
- pub `replace_from_vec` function L106-110 — `(&mut self, items: Vec<T>)` — Replace contents with items from a Vec, keeping only the last `max_capacity` items.
- pub `from_vec` function L113-120 — `(items: Vec<T>, max_capacity: usize) -> Self` — Create from a Vec, keeping only the last `max_capacity` items.
- pub `extend` function L123-127 — `(&mut self, iter: I)` — Extend with items from an iterator.
-  `Target` type L132 — `= [T]` — Bounded collection types to prevent unbounded memory growth.
-  `deref` function L134-136 — `(&self) -> &Self::Target` — Bounded collection types to prevent unbounded memory growth.
-  `deref_mut` function L140-142 — `(&mut self) -> &mut Self::Target` — Bounded collection types to prevent unbounded memory growth.
-  `default` function L146-149 — `() -> Self` — Bounded collection types to prevent unbounded memory growth.
-  `Output` type L154 — `= T` — Bounded collection types to prevent unbounded memory growth.
-  `index` function L156-158 — `(&self, index: usize) -> &Self::Output` — Bounded collection types to prevent unbounded memory growth.
-  `index_mut` function L162-164 — `(&mut self, index: usize) -> &mut Self::Output` — Bounded collection types to prevent unbounded memory growth.
-  `tests` module L168-293 — `-` — Bounded collection types to prevent unbounded memory growth.
-  `test_basic_push` function L172-181 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_eviction_at_capacity` function L184-199 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_eviction_removes_ten_percent` function L202-215 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_last` function L218-227 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_last_mut` function L230-239 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_clear` function L242-249 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_iter` function L252-260 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_deref_slice_methods` function L263-273 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_zero_capacity_panics` function L277-279 — `()` — Bounded collection types to prevent unbounded memory growth.
-  `test_small_capacity_eviction` function L282-292 — `()` — Bounded collection types to prevent unbounded memory growth.

#### crates/arawn-tui/src/client.rs

- pub `ConnectionStatus` enum L13-22 — `Disconnected | Connecting | Connected | Reconnecting` — Connection status for display in the UI.
- pub `WsClient` struct L36-50 — `{ server_url: String, tx: mpsc::UnboundedSender<ClientMessage>, rx: mpsc::Unboun...` — WebSocket client for real-time communication with the Arawn server.
- pub `new` function L54-70 — `(server_url: &str) -> Self` — Create a new client and start connecting to the server.
- pub `server_url` function L73-75 — `(&self) -> &str` — Get the server URL.
- pub `status` function L78-80 — `(&self) -> ConnectionStatus` — Get the current connection status.
- pub `poll_status` function L83-91 — `(&mut self) -> Option<ConnectionStatus>` — Poll for status updates (non-blocking).
- pub `recv` function L94-96 — `(&mut self) -> Option<ServerMessage>` — Receive the next server message (async).
- pub `try_recv` function L99-101 — `(&mut self) -> Option<ServerMessage>` — Try to receive a server message (non-blocking).
- pub `send_chat` function L104-117 — `( &self, message: String, session_id: Option<String>, workstream_id: Option<Stri...` — Send a chat message.
- pub `send_ping` function L120-124 — `(&self) -> Result<()>` — Send a ping.
- pub `subscribe` function L129-136 — `(&self, session_id: String, reconnect_token: Option<String>) -> Result<()>` — Subscribe to a session.
- pub `authenticate` function L139-143 — `(&self, token: String) -> Result<()>` — Authenticate with a token.
- pub `cancel` function L146-150 — `(&self, session_id: String) -> Result<()>` — Cancel the current operation for a session.
- pub `send_command` function L153-157 — `(&self, command: String, args: serde_json::Value) -> Result<()>` — Send a command to the server.
-  `ConnectionStatus` type L24-33 — `= ConnectionStatus` — WebSocket client for connecting to the Arawn server.
-  `fmt` function L25-32 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — WebSocket client for connecting to the Arawn server.
-  `WsClient` type L52-158 — `= WsClient` — WebSocket client for connecting to the Arawn server.
-  `connection_loop` function L161-221 — `( server_url: String, mut client_rx: mpsc::UnboundedReceiver<ClientMessage>, ser...` — Connection loop that handles reconnection with exponential backoff.
-  `handle_connection` function L225-293 — `( ws_stream: tokio_tungstenite::WebSocketStream< tokio_tungstenite::MaybeTlsStre...` — Handle an active WebSocket connection.
-  `http_to_ws_url` function L296-313 — `(http_url: &str) -> Result<String>` — Convert an HTTP URL to a WebSocket URL with /ws path.
-  `tests` module L316-349 — `-` — WebSocket client for connecting to the Arawn server.
-  `test_http_to_ws_url` function L320-337 — `()` — WebSocket client for connecting to the Arawn server.
-  `test_connection_status_display` function L340-348 — `()` — WebSocket client for connecting to the Arawn server.

#### crates/arawn-tui/src/events.rs

- pub `Event` enum L12-19 — `Key | Resize | Tick` — Terminal events.
- pub `EventHandler` struct L22-28 — `{ rx: mpsc::UnboundedReceiver<Event>, task: tokio::task::JoinHandle<()> }` — Handles terminal events using crossterm's async event stream.
- pub `new` function L32-79 — `() -> Self` — Create a new event handler.
- pub `next` function L82-87 — `(&mut self) -> Result<Event>` — Wait for the next event.
-  `EventHandler` type L30-88 — `= EventHandler` — Event handling for the TUI.
-  `EventHandler` type L90-94 — `impl Default for EventHandler` — Event handling for the TUI.
-  `default` function L91-93 — `() -> Self` — Event handling for the TUI.

#### crates/arawn-tui/src/focus.rs

- pub `FocusTarget` enum L8-24 — `Input | Sidebar | ToolPane | Logs | CommandPalette | Sessions | Workstreams` — Focus targets - all focusable areas in the TUI.
- pub `is_overlay` function L31-36 — `(&self) -> bool` — Returns true if this target is an overlay (modal popup).
- pub `is_panel` function L39-41 — `(&self) -> bool` — Returns true if this is a main panel (not an overlay).
- pub `name` function L44-54 — `(&self) -> &'static str` — Get the display name for this focus target.
- pub `FocusManager` struct L86-93 — `{ current: FocusTarget, previous: Option<FocusTarget>, overlay_stack: Vec<FocusT...` — Manages focus state and transitions for the TUI.
- pub `new` function L103-109 — `() -> Self` — Create a new focus manager with default focus on Input.
- pub `current` function L112-114 — `(&self) -> FocusTarget` — Get the current focus target.
- pub `is` function L117-119 — `(&self, target: FocusTarget) -> bool` — Check if currently focused on a specific target.
- pub `has_overlay` function L122-124 — `(&self) -> bool` — Check if any overlay is active.
- pub `focus` function L129-142 — `(&mut self, target: FocusTarget)` — Direct focus change to a panel (not an overlay).
- pub `push_overlay` function L145-158 — `(&mut self, overlay: FocusTarget)` — Open an overlay, remembering the current focus to return to.
- pub `pop_overlay` function L163-175 — `(&mut self) -> Option<FocusTarget>` — Close the current overlay and return to previous focus.
- pub `close_all_overlays` function L178-181 — `(&mut self)` — Close all overlays and return to the previous panel focus.
- pub `cycle_next` function L187-199 — `(&mut self)` — Cycle focus to the next main panel.
- pub `cycle_prev` function L205-221 — `(&mut self)` — Cycle focus to the previous main panel.
- pub `toggle` function L227-233 — `(&mut self, target: FocusTarget)` — Toggle focus between the current panel and a specific target.
- pub `return_to_input` function L236-239 — `(&mut self)` — Return focus to Input (common operation).
-  `FocusTarget` type L26-55 — `= FocusTarget` — adding new panels easier and focus behavior more predictable.
-  `CYCLABLE_PANELS` variable L58-63 — `: &[FocusTarget]` — Main panels that can be cycled through with Tab.
-  `FocusManager` type L95-99 — `impl Default for FocusManager` — adding new panels easier and focus behavior more predictable.
-  `default` function L96-98 — `() -> Self` — adding new panels easier and focus behavior more predictable.
-  `FocusManager` type L101-240 — `= FocusManager` — adding new panels easier and focus behavior more predictable.
-  `tests` module L243-373 — `-` — adding new panels easier and focus behavior more predictable.
-  `test_default_focus` function L247-251 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_direct_focus` function L254-259 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_overlay_push_pop` function L262-278 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_overlay_returns_to_previous_panel` function L281-295 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_cycle_next` function L298-315 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_cycle_prev` function L318-327 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_cycle_blocked_during_overlay` function L330-338 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_toggle` function L341-351 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_is_overlay` function L354-360 — `()` — adding new panels easier and focus behavior more predictable.
-  `test_close_all_overlays` function L363-372 — `()` — adding new panels easier and focus behavior more predictable.

#### crates/arawn-tui/src/input.rs

- pub `ParsedCommand` struct L10-15 — `{ name: String, args: String }` — Parsed command from input starting with '/'.
- pub `parse` function L21-41 — `(input: &str) -> Option<Self>` — Parse a command from input text.
- pub `name_lower` function L44-46 — `(&self) -> String` — Get the command name in lowercase for matching.
- pub `InputState` struct L51-62 — `{ content: String, cursor: usize, history: VecDeque<String>, history_index: Opti...` — Input state with text editing and history navigation.
- pub `new` function L72-80 — `() -> Self` — Create a new empty input state.
- pub `content` function L83-85 — `(&self) -> &str` — Get the current input content.
- pub `cursor` function L88-90 — `(&self) -> usize` — Get the cursor position (byte offset).
- pub `is_empty` function L93-95 — `(&self) -> bool` — Check if the input is empty.
- pub `is_command` function L98-100 — `(&self) -> bool` — Check if the input starts with a command prefix '/'.
- pub `parse_command` function L103-105 — `(&self) -> Option<ParsedCommand>` — Parse the input as a command if it starts with '/'.
- pub `command_prefix` function L109-117 — `(&self) -> Option<&str>` — Get the command prefix for autocomplete filtering.
- pub `line_count` function L120-122 — `(&self) -> usize` — Count the number of lines in the input.
- pub `cursor_position` function L128-136 — `(&self) -> (usize, usize)` — Get the cursor's line and column position.
- pub `insert_char` function L139-143 — `(&mut self, c: char)` — Insert a character at the cursor position.
- pub `insert_newline` function L146-148 — `(&mut self)` — Insert a newline at the cursor position.
- pub `delete_char_before` function L151-163 — `(&mut self)` — Delete the character before the cursor (backspace).
- pub `delete_char_at` function L166-171 — `(&mut self)` — Delete the character at the cursor (delete key).
- pub `move_left` function L174-183 — `(&mut self)` — Move cursor left by one character.
- pub `move_right` function L186-195 — `(&mut self)` — Move cursor right by one character.
- pub `move_to_line_start` function L198-201 — `(&mut self)` — Move cursor to the start of the current line.
- pub `move_to_line_end` function L204-210 — `(&mut self)` — Move cursor to the end of the current line.
- pub `move_to_start` function L213-215 — `(&mut self)` — Move cursor to the start of input.
- pub `move_to_end` function L218-220 — `(&mut self)` — Move cursor to the end of input.
- pub `move_up` function L223-242 — `(&mut self)` — Move cursor up one line.
- pub `move_down` function L245-266 — `(&mut self)` — Move cursor down one line.
- pub `history_prev` function L270-298 — `(&mut self) -> bool` — Navigate to previous history entry.
- pub `history_next` function L302-323 — `(&mut self) -> bool` — Navigate to next history entry or restore draft.
- pub `is_browsing_history` function L326-328 — `(&self) -> bool` — Check if currently browsing history.
- pub `submit` function L344-359 — `(&mut self) -> String` — Submit the current input and add to history.
- pub `clear` function L362-366 — `(&mut self)` — Clear the current input.
- pub `set_text` function L369-373 — `(&mut self, text: &str)` — Set the input text and move cursor to the end.
-  `MAX_HISTORY` variable L6 — `: usize` — Maximum number of history entries to keep.
-  `ParsedCommand` type L17-47 — `= ParsedCommand` — Input state management with history support.
-  `InputState` type L64-68 — `impl Default for InputState` — Input state management with history support.
-  `default` function L65-67 — `() -> Self` — Input state management with history support.
-  `InputState` type L70-374 — `= InputState` — Input state management with history support.
-  `exit_history_mode` function L337-340 — `(&mut self)` — Exit history browsing mode without restoring draft.
-  `tests` module L377-557 — `-` — Input state management with history support.
-  `test_basic_input` function L381-389 — `()` — Input state management with history support.
-  `test_cursor_movement` function L392-406 — `()` — Input state management with history support.
-  `test_backspace` function L409-415 — `()` — Input state management with history support.
-  `test_history` function L418-445 — `()` — Input state management with history support.
-  `test_multiline` function L448-460 — `()` — Input state management with history support.
-  `test_history_with_draft` function L463-484 — `()` — Input state management with history support.
-  `test_is_command` function L487-508 — `()` — Input state management with history support.
-  `test_parse_command` function L511-535 — `()` — Input state management with history support.
-  `test_command_prefix` function L538-556 — `()` — Input state management with history support.

#### crates/arawn-tui/src/lib.rs

- pub `app` module L5 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `bounded` module L6 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `client` module L7 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `events` module L8 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `focus` module L9 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `input` module L10 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `logs` module L11 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `palette` module L12 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `protocol` module L13 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `sessions` module L14 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `sidebar` module L15 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `ui` module L16 — `-` — A minimal, keyboard-driven terminal interface for Arawn.
- pub `Tui` type L34 — `= Terminal<CrosstermBackend<Stdout>>` — Terminal type alias for convenience.
- pub `init_terminal` function L37-44 — `() -> Result<Tui>` — Initialize the terminal for TUI mode.
- pub `restore_terminal` function L47-56 — `(terminal: &mut Tui) -> Result<()>` — Restore the terminal to normal mode.
- pub `install_panic_hook` function L59-67 — `()` — Install a panic hook that restores the terminal before panicking.
- pub `TuiConfig` struct L70-79 — `{ server_url: String, workstream: Option<String>, context_name: Option<String>, ...` — Configuration for running the TUI.
- pub `new` function L83-90 — `(server_url: impl Into<String>) -> Self` — Create config with just a server URL.
- pub `from_client_config` function L95-117 — `(context_name: Option<&str>) -> Result<Self>` — Load config from the client config file.
- pub `run` function L121-123 — `(server_url: &str) -> Result<()>` — Run the TUI application.
- pub `run_with_config` function L126-154 — `(config: TuiConfig) -> Result<()>` — Run the TUI application with full configuration.
-  `TuiConfig` type L81-118 — `= TuiConfig` — A minimal, keyboard-driven terminal interface for Arawn.

#### crates/arawn-tui/src/logs.rs

- pub `LogEntry` struct L19-26 — `{ level: Level, target: String, message: String }` — A single log entry.
- pub `level_color` function L30-39 — `(&self) -> ratatui::style::Color` — Get a color for this log level.
- pub `level_prefix` function L42-50 — `(&self) -> &'static str` — Get a short level prefix.
- pub `LogBuffer` struct L55-57 — `{ entries: Arc<Mutex<VecDeque<LogEntry>>> }` — Shared log buffer that can be read by the TUI.
- pub `new` function L61-65 — `() -> Self` — Create a new log buffer.
- pub `entries` function L68-70 — `(&self) -> Vec<LogEntry>` — Get all current entries.
- pub `len` function L73-75 — `(&self) -> usize` — Get the number of entries.
- pub `is_empty` function L78-80 — `(&self) -> bool` — Check if empty.
- pub `clear` function L83-85 — `(&self)` — Clear all entries.
- pub `TuiLogLayer` struct L98-102 — `{ buffer: LogBuffer, min_level: Level }` — A tracing layer that captures logs to a buffer.
- pub `new` function L106-111 — `(buffer: LogBuffer) -> Self` — Create a new TUI log layer.
- pub `with_min_level` function L114-117 — `(mut self, level: Level) -> Self` — Set minimum log level to capture.
-  `MAX_LOG_ENTRIES` variable L15 — `: usize` — Maximum number of log entries to keep.
-  `LogEntry` type L28-51 — `= LogEntry` — Captures tracing events and stores them in a ring buffer for display.
-  `LogBuffer` type L59-95 — `= LogBuffer` — Captures tracing events and stores them in a ring buffer for display.
-  `push` function L88-94 — `(&self, entry: LogEntry)` — Add an entry.
-  `TuiLogLayer` type L104-118 — `= TuiLogLayer` — Captures tracing events and stores them in a ring buffer for display.
-  `MessageVisitor` struct L121-123 — `{ message: String }` — Visitor to extract the message field from events.
-  `MessageVisitor` type L125-131 — `= MessageVisitor` — Captures tracing events and stores them in a ring buffer for display.
-  `new` function L126-130 — `() -> Self` — Captures tracing events and stores them in a ring buffer for display.
-  `MessageVisitor` type L133-152 — `impl Visit for MessageVisitor` — Captures tracing events and stores them in a ring buffer for display.
-  `record_debug` function L134-145 — `(&mut self, field: &Field, value: &dyn std::fmt::Debug)` — Captures tracing events and stores them in a ring buffer for display.
-  `record_str` function L147-151 — `(&mut self, field: &Field, value: &str)` — Captures tracing events and stores them in a ring buffer for display.
-  `TuiLogLayer` type L154-175 — `= TuiLogLayer` — Captures tracing events and stores them in a ring buffer for display.
-  `on_event` function L155-174 — `(&self, event: &Event<'_>, _ctx: Context<'_, S>)` — Captures tracing events and stores them in a ring buffer for display.
-  `tests` module L178-217 — `-` — Captures tracing events and stores them in a ring buffer for display.
-  `test_log_buffer` function L182-195 — `()` — Captures tracing events and stores them in a ring buffer for display.
-  `test_log_entry_colors` function L198-216 — `()` — Captures tracing events and stores them in a ring buffer for display.

#### crates/arawn-tui/src/palette.rs

- pub `Action` struct L5-14 — `{ id: ActionId, label: &'static str, category: &'static str, shortcut: Option<&'...` — An action that can be executed from the command palette.
- pub `ActionId` enum L18-31 — `SessionsSwitch | SessionsNew | SessionsDelete | SessionsMoveToWorkstream | Works...` — Identifiers for all palette actions.
- pub `DEFAULT_ACTIONS` variable L51-95 — `: &[Action]` — Default set of actions available in the palette.
- pub `CommandPalette` struct L99-108 — `{ actions: Vec<Action>, filter: String, filtered_indices: Vec<usize>, selected: ...` — State for the command palette.
- pub `new` function L118-128 — `() -> Self` — Create a new command palette with default actions.
- pub `filter` function L131-133 — `(&self) -> &str` — Get the current filter text.
- pub `selected_action` function L136-140 — `(&self) -> Option<&Action>` — Get the selected action (if any).
- pub `selected_index` function L143-145 — `(&self) -> usize` — Get the selected index in the filtered list.
- pub `visible_actions` function L149-162 — `(&self) -> impl Iterator<Item = (bool, bool, &Action)>` — Get an iterator over visible actions with metadata.
- pub `visible_count` function L165-167 — `(&self) -> usize` — Get the count of visible actions.
- pub `select_prev` function L170-174 — `(&mut self)` — Move selection up.
- pub `select_next` function L177-181 — `(&mut self)` — Move selection down.
- pub `select_first` function L184-186 — `(&mut self)` — Move selection to first item.
- pub `select_last` function L189-193 — `(&mut self)` — Move selection to last item.
- pub `filter_push` function L196-199 — `(&mut self, c: char)` — Add a character to the filter.
- pub `filter_pop` function L202-205 — `(&mut self)` — Remove last character from filter.
- pub `filter_clear` function L208-211 — `(&mut self)` — Clear the filter.
- pub `reset` function L236-240 — `(&mut self)` — Reset the palette state.
- pub `register_action` function L243-246 — `(&mut self, action: Action)` — Register a new action.
-  `Action` type L33-48 — `= Action` — Command palette state and action registry.
-  `new` function L35-47 — `( id: ActionId, label: &'static str, category: &'static str, shortcut: Option<&'...` — Create a new action.
-  `CommandPalette` type L110-114 — `impl Default for CommandPalette` — Command palette state and action registry.
-  `default` function L111-113 — `() -> Self` — Command palette state and action registry.
-  `CommandPalette` type L116-247 — `= CommandPalette` — Command palette state and action registry.
-  `update_filtered` function L214-233 — `(&mut self)` — Update filtered indices based on current filter.
-  `fuzzy_match` function L250-264 — `(text: &str, filter: &str) -> bool` — Simple fuzzy matching - checks if all filter characters appear in order.
-  `tests` module L267-345 — `-` — Command palette state and action registry.
-  `test_palette_filtering` function L271-290 — `()` — Command palette state and action registry.
-  `test_palette_navigation` function L293-310 — `()` — Command palette state and action registry.
-  `test_palette_action_selection` function L313-324 — `()` — Command palette state and action registry.
-  `test_category_grouping` function L327-344 — `()` — Command palette state and action registry.

#### crates/arawn-tui/src/protocol.rs

- pub `ClientMessage` enum L10-55 — `Chat | Subscribe | Unsubscribe | Ping | Auth | Cancel | Command` — Messages from client to server.
- pub `ServerMessage` enum L60-196 — `AuthResult | SessionCreated | ChatChunk | ToolStart | ToolOutput | ToolEnd | Err...` — Messages from server to client.
-  `tests` module L199-382 — `-` — These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.
-  `test_client_message_serialization` function L203-234 — `()` — These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.
-  `test_server_message_deserialization` function L237-260 — `()` — These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.
-  `test_command_message_serialization` function L263-281 — `()` — These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.
-  `test_command_response_deserialization` function L284-339 — `()` — These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.
-  `test_context_info_deserialization` function L342-381 — `()` — These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.

#### crates/arawn-tui/src/sessions.rs

- pub `SessionSummary` struct L7-18 — `{ id: String, title: String, last_active: DateTime<Utc>, message_count: usize, i...` — Summary information about a session.
- pub `SessionList` struct L35-46 — `{ items: Vec<SessionSummary>, selected: usize, filter: String, filtered_indices:...` — State for the session list overlay.
- pub `new` function L56-64 — `() -> Self` — Create a new empty session list.
- pub `filter` function L67-69 — `(&self) -> &str` — Get the filter text.
- pub `is_loading` function L72-74 — `(&self) -> bool` — Check if the list is loading.
- pub `set_loading` function L77-79 — `(&mut self, loading: bool)` — Set loading state.
- pub `set_items` function L82-86 — `(&mut self, items: Vec<SessionSummary>)` — Update the session list with new items.
- pub `selected_session` function L89-93 — `(&self) -> Option<&SessionSummary>` — Get the currently selected session (if any).
- pub `selected_index` function L96-98 — `(&self) -> usize` — Get the selected index in the filtered list.
- pub `visible_sessions` function L101-106 — `(&self) -> impl Iterator<Item = (bool, &SessionSummary)>` — Get an iterator over visible sessions with their selected state.
- pub `visible_count` function L109-111 — `(&self) -> usize` — Get the count of visible sessions.
- pub `select_prev` function L114-118 — `(&mut self)` — Move selection up.
- pub `select_next` function L121-125 — `(&mut self)` — Move selection down.
- pub `select_first` function L128-130 — `(&mut self)` — Move selection to first item.
- pub `select_last` function L133-137 — `(&mut self)` — Move selection to last item.
- pub `filter_push` function L140-143 — `(&mut self, c: char)` — Add a character to the filter.
- pub `filter_pop` function L146-149 — `(&mut self)` — Remove last character from filter.
- pub `filter_clear` function L152-155 — `(&mut self)` — Clear the filter.
- pub `reset` function L180-184 — `(&mut self)` — Reset the list state (e.g., when closing the overlay).
- pub `set_current` function L187-191 — `(&mut self, session_id: &str)` — Mark a session as current by ID.
- pub `format_relative_time` function L219-249 — `(time: DateTime<Utc>) -> String` — Format a timestamp as a relative time string.
-  `SessionList` type L48-52 — `impl Default for SessionList` — Session list state and management.
-  `default` function L49-51 — `() -> Self` — Session list state and management.
-  `SessionList` type L54-192 — `= SessionList` — Session list state and management.
-  `update_filtered` function L158-177 — `(&mut self)` — Update the filtered indices based on current filter.
-  `fuzzy_match` function L202-216 — `(text: &str, filter: &str) -> bool` — Simple fuzzy matching - checks if all filter characters appear in order.
-  `tests` module L252-344 — `-` — Session list state and management.
-  `test_fuzzy_match` function L256-265 — `()` — Session list state and management.
-  `test_session_list_filtering` function L268-295 — `()` — Session list state and management.
-  `test_session_list_navigation` function L298-343 — `()` — Session list state and management.

#### crates/arawn-tui/src/sidebar.rs

- pub `WorkstreamEntry` struct L7-24 — `{ id: String, name: String, session_count: usize, is_current: bool, is_scratch: ...` — A workstream entry for display.
- pub `is_archived` function L28-30 — `(&self) -> bool` — Check if this workstream is archived.
- pub `SidebarSection` enum L35-39 — `Workstreams | Sessions` — Which section of the sidebar has focus.
- pub `Sidebar` struct L61-76 — `{ open: bool, section: SidebarSection, workstreams: Vec<WorkstreamEntry>, workst...` — Sidebar state managing workstreams and sessions lists.
- pub `new` function L86-96 — `() -> Self` — Create a new sidebar (starts closed).
- pub `toggle` function L99-101 — `(&mut self)` — Toggle sidebar open/closed.
- pub `open` function L104-106 — `(&mut self)` — Open the sidebar.
- pub `close` function L109-111 — `(&mut self)` — Close the sidebar.
- pub `is_open` function L114-116 — `(&self) -> bool` — Check if the sidebar is open.
- pub `toggle_section` function L119-124 — `(&mut self)` — Switch focus between workstreams and sessions.
- pub `select_prev` function L129-159 — `(&mut self) -> Option<String>` — Move selection up in current section (circular).
- pub `select_next` function L164-194 — `(&mut self) -> Option<String>` — Move selection down in current section (circular).
- pub `selected_workstream` function L197-199 — `(&self) -> Option<&WorkstreamEntry>` — Get the currently selected workstream.
- pub `is_new_session_selected` function L202-204 — `(&self) -> bool` — Check if "+ New Session" is currently selected.
- pub `selected_session` function L207-213 — `(&self) -> Option<&SessionSummary>` — Get the currently selected session (None if "+ New Session" is selected).
- pub `filter_push` function L216-218 — `(&mut self, c: char)` — Add a character to the filter.
- pub `filter_pop` function L221-223 — `(&mut self)` — Remove the last character from the filter.
- pub `filter_clear` function L226-228 — `(&mut self)` — Clear the filter.
- pub `visible_workstreams` function L231-240 — `(&self) -> impl Iterator<Item = (bool, &WorkstreamEntry)>` — Get visible active workstreams (filtered).
- pub `visible_archived_workstreams` function L243-252 — `(&self) -> impl Iterator<Item = (bool, &WorkstreamEntry)>` — Get visible archived workstreams (filtered).
- pub `has_archived_workstreams` function L255-257 — `(&self) -> bool` — Check if there are any archived workstreams.
- pub `visible_sessions` function L261-270 — `(&self) -> impl Iterator<Item = (bool, &SessionSummary)>` — Get visible sessions (filtered).
- pub `set_current_session` function L273-281 — `(&mut self, session_id: &str)` — Set the current session as selected in sessions list.
-  `WorkstreamEntry` type L26-31 — `= WorkstreamEntry` — Sidebar state for workstreams and sessions navigation.
-  `Sidebar` type L78-82 — `impl Default for Sidebar` — Sidebar state for workstreams and sessions navigation.
-  `default` function L79-81 — `() -> Self` — Sidebar state for workstreams and sessions navigation.
-  `Sidebar` type L84-282 — `= Sidebar` — Sidebar state for workstreams and sessions navigation.
-  `tests` module L285-485 — `-` — Sidebar state for workstreams and sessions navigation.
-  `test_sidebar_toggle` function L290-306 — `()` — Sidebar state for workstreams and sessions navigation.
-  `test_section_toggle` function L309-318 — `()` — Sidebar state for workstreams and sessions navigation.
-  `setup_test_workstreams` function L321-364 — `(sidebar: &mut Sidebar)` — Helper to set up test workstreams.
-  `setup_test_sessions` function L367-406 — `(sidebar: &mut Sidebar)` — Helper to set up test sessions.
-  `test_navigation` function L409-460 — `()` — Sidebar state for workstreams and sessions navigation.
-  `test_workstream_navigation_returns_id` function L463-484 — `()` — Sidebar state for workstreams and sessions navigation.

### crates/arawn-tui/src/ui

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-tui/src/ui/chat.rs

- pub `render_chat` function L16-71 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the chat view with all messages.
-  `STREAMING_CURSOR` variable L13 — `: &str` — Streaming cursor indicator.
-  `render_user_message` function L74-78 — `(lines: &mut Vec<Line<'static>>, msg: &ChatMessage)` — Render user message with > prefix.
-  `render_assistant_message` function L81-105 — `(lines: &mut Vec<Line<'static>>, msg: &ChatMessage, _width: usize)` — Render assistant message with word wrapping and streaming cursor.
-  `TOOL_SEPARATOR` variable L108 — `: &str` — Dotted separator character for tool display.
-  `render_tools` function L111-183 — `(lines: &mut Vec<Line<'static>>, tools: &[ToolExecution])` — Render tool executions between messages.
-  `truncate_str` function L186-192 — `(s: &str, max_len: usize) -> String` — Truncate a string to max length, adding "..." if truncated.
-  `format_duration` function L195-206 — `(ms: u64) -> String` — Format duration in human-readable form.
-  `render_welcome` function L209-244 — `(frame: &mut Frame, area: Rect)` — Render the welcome screen when there are no messages.

#### crates/arawn-tui/src/ui/command_popup.rs

- pub `CommandInfo` struct L13-18 — `{ name: String, description: String }` — A command available for execution.
- pub `new` function L21-26 — `(name: impl Into<String>, description: impl Into<String>) -> Self` — Command autocomplete popup component.
- pub `CommandPopup` struct L31-40 — `{ commands: Vec<CommandInfo>, filtered: Vec<usize>, selected: usize, visible: bo...` — State for the command autocomplete popup.
- pub `new` function L44-51 — `() -> Self` — Create a new command popup with available commands.
- pub `set_commands` function L66-70 — `(&mut self, commands: Vec<CommandInfo>)` — Set the available commands (fetched from server).
- pub `show` function L73-76 — `(&mut self, prefix: &str)` — Show the popup and filter by prefix.
- pub `hide` function L79-82 — `(&mut self)` — Hide the popup.
- pub `is_visible` function L85-87 — `(&self) -> bool` — Check if the popup is visible.
- pub `filter` function L90-104 — `(&mut self, prefix: &str)` — Filter commands by prefix.
- pub `select_prev` function L107-111 — `(&mut self)` — Select previous item.
- pub `select_next` function L114-118 — `(&mut self)` — Select next item.
- pub `selected_command` function L121-125 — `(&self) -> Option<&CommandInfo>` — Get the currently selected command.
- pub `filtered_count` function L128-130 — `(&self) -> usize` — Get the number of filtered commands.
- pub `render` function L133-192 — `(&self, frame: &mut Frame, area: Rect)` — Render the popup.
-  `CommandInfo` type L20-27 — `= CommandInfo` — Command autocomplete popup component.
-  `CommandPopup` type L42-193 — `= CommandPopup` — Command autocomplete popup component.
-  `default_commands` function L55-63 — `() -> Vec<CommandInfo>` — Get the default list of commands.
-  `tests` module L196-279 — `-` — Command autocomplete popup component.
-  `test_command_popup_filter` function L200-221 — `()` — Command autocomplete popup component.
-  `test_command_popup_navigation` function L224-247 — `()` — Command autocomplete popup component.
-  `test_command_popup_visibility` function L250-260 — `()` — Command autocomplete popup component.
-  `test_command_popup_set_commands` function L263-278 — `()` — Command autocomplete popup component.

#### crates/arawn-tui/src/ui/input.rs

- pub `MIN_INPUT_HEIGHT` variable L13 — `: u16` — Minimum height for the input area (in lines).
- pub `MAX_INPUT_FRACTION` variable L16 — `: f32` — Maximum height for the input area as fraction of screen (30%).
- pub `calculate_input_height` function L19-26 — `(input: &InputState, available_height: u16) -> u16` — Calculate the desired height for the input area based on content.
- pub `render_input` function L29-109 — `( input: &InputState, waiting: bool, read_only: bool, frame: &mut Frame, area: R...` — Render the input area with multi-line support.

#### crates/arawn-tui/src/ui/layout.rs

- pub `render` function L26-128 — `(app: &App, frame: &mut Frame)` — Render the entire application UI.
-  `CONTEXT_WARNING_PERCENT` variable L4 — `: u8` — Main layout rendering.
-  `CONTEXT_CRITICAL_PERCENT` variable L5 — `: u8` — Main layout rendering.
-  `render_header` function L131-206 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the header bar.
-  `render_content` function L209-226 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the main content area (chat messages + optional tool pane).
-  `render_input` function L229-232 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the input area.
-  `render_status_bar` function L235-292 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the status bar.
-  `format_context_indicator` function L295-313 — `(ctx: &crate::app::ContextState) -> (String, Color)` — Format the context indicator with appropriate color.
-  `render_sessions_overlay` function L316-318 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the sessions overlay.
-  `render_workstreams_overlay` function L321-399 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the workstreams overlay.
-  `render_command_palette` function L402-404 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the command palette.
-  `centered_rect` function L407-421 — `(percent_x: u16, percent_y: u16, area: Rect) -> Rect` — Create a centered rectangle within the given area.
-  `render_warning_banner` function L424-448 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the disk warning banner.
-  `render_usage_popup` function L451-574 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the usage stats popup (Ctrl+U).

#### crates/arawn-tui/src/ui/logs.rs

- pub `render_logs_panel` function L13-72 — `(log_buffer: &LogBuffer, scroll: usize, frame: &mut Frame, area: Rect)` — Render the logs panel.
- pub `render_logs_footer` function L75-87 — `(frame: &mut Frame, area: Rect)` — Render the logs footer with keyboard hints.

#### crates/arawn-tui/src/ui/mod.rs

- pub `chat` module L3 — `-` — UI rendering components.
- pub `command_popup` module L4 — `-` — UI rendering components.
- pub `input` module L5 — `-` — UI rendering components.
- pub `logs` module L7 — `-` — UI rendering components.
- pub `palette` module L8 — `-` — UI rendering components.
- pub `sessions` module L9 — `-` — UI rendering components.
- pub `sidebar` module L10 — `-` — UI rendering components.
- pub `tools` module L11 — `-` — UI rendering components.
-  `layout` module L6 — `-` — UI rendering components.

#### crates/arawn-tui/src/ui/palette.rs

- pub `render_palette_overlay` function L13-39 — `(palette: &CommandPalette, frame: &mut Frame, area: Rect)` — Render the command palette overlay.
-  `render_search_box` function L42-57 — `(palette: &CommandPalette, frame: &mut Frame, area: Rect)` — Render the search/filter box.
-  `render_separator` function L60-66 — `(frame: &mut Frame, area: Rect)` — Render a separator line.
-  `render_action_list` function L69-98 — `(palette: &CommandPalette, frame: &mut Frame, area: Rect)` — Render the action list.
-  `format_action_line` function L101-147 — `( action: &crate::palette::Action, is_selected: bool, width: usize, ) -> Line<'s...` — Format a single action line.
-  `render_footer` function L150-162 — `(frame: &mut Frame, area: Rect)` — Render the footer with keyboard hints.
-  `centered_rect` function L165-179 — `(percent_x: u16, percent_y: u16, area: Rect) -> Rect` — Create a centered rectangle within the given area.

#### crates/arawn-tui/src/ui/sessions.rs

- pub `render_sessions_overlay` function L13-39 — `(sessions: &SessionList, frame: &mut Frame, area: Rect)` — Render the sessions overlay.
-  `render_search_box` function L42-51 — `(sessions: &SessionList, frame: &mut Frame, area: Rect)` — Render the search/filter box.
-  `render_separator` function L54-60 — `(frame: &mut Frame, area: Rect)` — Render a separator line.
-  `render_session_list` function L63-97 — `(sessions: &SessionList, frame: &mut Frame, area: Rect)` — Render the session list.
-  `format_session_line` function L100-146 — `( session: &crate::sessions::SessionSummary, is_selected: bool, width: usize, ) ...` — Format a single session line.
-  `render_footer` function L149-164 — `(frame: &mut Frame, area: Rect)` — Render the footer with keyboard hints.
-  `centered_rect` function L167-181 — `(percent_x: u16, percent_y: u16, area: Rect) -> Rect` — Create a centered rectangle within the given area.

#### crates/arawn-tui/src/ui/sidebar.rs

- pub `SIDEBAR_WIDTH` variable L18 — `: u16` — Width of the expanded sidebar (when open).
- pub `SIDEBAR_HINT_WIDTH` variable L20 — `: u16` — Width of the closed sidebar hint.
- pub `render_sidebar` function L23-29 — `(sidebar: &Sidebar, frame: &mut Frame, area: Rect)` — Render the sidebar panel based on open/closed state.
-  `CONTEXT_WARNING_PERCENT` variable L4 — `: u8` — Sidebar panel rendering for workstreams and sessions.
-  `CONTEXT_CRITICAL_PERCENT` variable L5 — `: u8` — Sidebar panel rendering for workstreams and sessions.
-  `render_closed_hint` function L32-39 — `(frame: &mut Frame, area: Rect)` — Render the closed sidebar hint (minimal indicator).
-  `render_open_sidebar` function L42-65 — `(sidebar: &Sidebar, frame: &mut Frame, area: Rect)` — Render the open sidebar with full content (has focus).
-  `render_workstreams_header` function L68-79 — `(sidebar: &Sidebar, frame: &mut Frame, area: Rect)` — Render the workstreams section header.
-  `render_workstreams_list` function L82-134 — `(sidebar: &Sidebar, frame: &mut Frame, area: Rect)` — Render the workstreams list.
-  `render_workstream_line` function L137-223 — `( sidebar: &Sidebar, ws: &crate::sidebar::WorkstreamEntry, is_selected: bool, wi...` — Render a single workstream line.
-  `format_size` function L226-236 — `(bytes: u64) -> String` — Format byte size as human-readable string.
-  `render_sessions_header` function L239-260 — `(sidebar: &Sidebar, frame: &mut Frame, area: Rect)` — Render the sessions section header.
-  `render_sessions_list` function L263-318 — `(sidebar: &Sidebar, frame: &mut Frame, area: Rect)` — Render the sessions list.
-  `render_sidebar_footer` function L321-327 — `(frame: &mut Frame, area: Rect)` — Render the sidebar footer with keybinding hints.
-  `truncate_str` function L330-338 — `(s: &str, max_width: usize) -> String` — Truncate a string to fit within the given width.
-  `tests` module L341-351 — `-` — Sidebar panel rendering for workstreams and sessions.
-  `test_truncate_str` function L345-350 — `()` — Sidebar panel rendering for workstreams and sessions.

#### crates/arawn-tui/src/ui/tools.rs

- pub `render_tool_pane` function L13-41 — `(app: &App, frame: &mut Frame, area: Rect)` — Render the tool output pane (split view at bottom of screen).
- pub `render_tool_pane_footer` function L172-186 — `(frame: &mut Frame, area: Rect)` — Render help footer for tool pane.
-  `build_title` function L44-89 — `(app: &App) -> Line<'static>` — Build the title line with tool selector.
-  `get_selected_tool` function L92-94 — `(app: &App) -> Option<&ToolExecution>` — Get the currently selected tool.
-  `render_tool_output` function L97-140 — `(tool: &ToolExecution, scroll: usize, frame: &mut Frame, area: Rect)` — Render the output of a tool.
-  `render_no_tools` function L143-157 — `(frame: &mut Frame, area: Rect)` — Render placeholder when no tools exist.
-  `render_no_selection` function L160-169 — `(frame: &mut Frame, area: Rect)` — Render placeholder when no tool is selected.

### crates/arawn-types/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-types/src/config.rs

- pub `ConfigProvider` interface L13 — `-` — Base trait for all configuration types.
- pub `HasSessionConfig` interface L19-30 — `{ fn max_sessions(), fn cleanup_interval(), fn session_ttl() }` — Session management configuration.
- pub `HasToolConfig` interface L35-44 — `{ fn shell_timeout(), fn web_timeout(), fn max_output_bytes() }` — Tool execution configuration.
- pub `HasAgentConfig` interface L49-57 — `{ fn max_iterations(), fn default_timeout() }` — Agent execution configuration.
- pub `HasRateLimitConfig` interface L62-73 — `{ fn rate_limiting_enabled(), fn requests_per_minute(), fn burst_size() }` — Rate limiting configuration.
- pub `defaults` module L80-109 — `-` — Default session configuration values.
- pub `MAX_SESSIONS` variable L83 — `: usize` — represents a specific configuration capability.
- pub `CLEANUP_INTERVAL_SECS` variable L84 — `: u64` — represents a specific configuration capability.
- pub `SHELL_TIMEOUT_SECS` variable L85 — `: u64` — represents a specific configuration capability.
- pub `WEB_TIMEOUT_SECS` variable L86 — `: u64` — represents a specific configuration capability.
- pub `MAX_OUTPUT_BYTES` variable L87 — `: usize` — represents a specific configuration capability.
- pub `MAX_ITERATIONS` variable L88 — `: u32` — represents a specific configuration capability.
- pub `REQUESTS_PER_MINUTE` variable L89 — `: u32` — represents a specific configuration capability.
- pub `BURST_SIZE` variable L90 — `: u32` — represents a specific configuration capability.
- pub `DEFAULT_PORT` variable L91 — `: u16` — represents a specific configuration capability.
- pub `DEFAULT_BIND` variable L92 — `: &str` — represents a specific configuration capability.
- pub `CONTEXT_WARNING_PERCENT` variable L94 — `: u8` — Context usage warning threshold (percentage).
- pub `CONTEXT_CRITICAL_PERCENT` variable L96 — `: u8` — Context usage critical threshold (percentage).
- pub `cleanup_interval` function L98-100 — `() -> Duration` — represents a specific configuration capability.
- pub `shell_timeout` function L102-104 — `() -> Duration` — represents a specific configuration capability.
- pub `web_timeout` function L106-108 — `() -> Duration` — represents a specific configuration capability.
- pub `SessionConfigProvider` struct L127-131 — `{ max_sessions: usize, cleanup_interval: Duration, session_ttl: Option<Duration>...` — Standalone session configuration.
- pub `ToolConfigProvider` struct L175-179 — `{ shell_timeout: Duration, web_timeout: Duration, max_output_bytes: usize }` — Standalone tool configuration.
- pub `AgentConfigProvider` struct L219-222 — `{ max_iterations: u32, default_timeout: Duration }` — Standalone agent configuration.
-  `session_ttl` function L27-29 — `(&self) -> Option<Duration>` — Optional TTL for sessions (None = no expiry).
-  `default_timeout` function L54-56 — `(&self) -> Duration` — Default timeout for agent operations.
-  `burst_size` function L70-72 — `(&self) -> u32` — Burst allowance above steady rate.
-  `SessionConfigProvider` type L133-141 — `impl Default for SessionConfigProvider` — represents a specific configuration capability.
-  `default` function L134-140 — `() -> Self` — represents a specific configuration capability.
-  `SessionConfigProvider` type L143 — `impl ConfigProvider for SessionConfigProvider` — represents a specific configuration capability.
-  `SessionConfigProvider` type L145-157 — `impl HasSessionConfig for SessionConfigProvider` — represents a specific configuration capability.
-  `max_sessions` function L146-148 — `(&self) -> usize` — represents a specific configuration capability.
-  `cleanup_interval` function L150-152 — `(&self) -> Duration` — represents a specific configuration capability.
-  `session_ttl` function L154-156 — `(&self) -> Option<Duration>` — represents a specific configuration capability.
-  `ToolConfigProvider` type L181-189 — `impl Default for ToolConfigProvider` — represents a specific configuration capability.
-  `default` function L182-188 — `() -> Self` — represents a specific configuration capability.
-  `ToolConfigProvider` type L191 — `impl ConfigProvider for ToolConfigProvider` — represents a specific configuration capability.
-  `ToolConfigProvider` type L193-205 — `impl HasToolConfig for ToolConfigProvider` — represents a specific configuration capability.
-  `shell_timeout` function L194-196 — `(&self) -> Duration` — represents a specific configuration capability.
-  `web_timeout` function L198-200 — `(&self) -> Duration` — represents a specific configuration capability.
-  `max_output_bytes` function L202-204 — `(&self) -> usize` — represents a specific configuration capability.
-  `AgentConfigProvider` type L224-231 — `impl Default for AgentConfigProvider` — represents a specific configuration capability.
-  `default` function L225-230 — `() -> Self` — represents a specific configuration capability.
-  `AgentConfigProvider` type L233 — `impl ConfigProvider for AgentConfigProvider` — represents a specific configuration capability.
-  `AgentConfigProvider` type L235-243 — `impl HasAgentConfig for AgentConfigProvider` — represents a specific configuration capability.
-  `max_iterations` function L236-238 — `(&self) -> u32` — represents a specific configuration capability.
-  `default_timeout` function L240-242 — `(&self) -> Duration` — represents a specific configuration capability.
-  `tests` module L246-282 — `-` — represents a specific configuration capability.
-  `test_session_config_defaults` function L250-255 — `()` — represents a specific configuration capability.
-  `test_tool_config_defaults` function L258-263 — `()` — represents a specific configuration capability.
-  `test_agent_config_defaults` function L266-269 — `()` — represents a specific configuration capability.
-  `test_custom_session_config` function L272-281 — `()` — represents a specific configuration capability.

#### crates/arawn-types/src/delegation.rs

- pub `SubagentInfo` struct L27-36 — `{ name: String, description: String, tools: Vec<String>, source: Option<String> ...` — Information about an available subagent.
- pub `SubagentResult` struct L40-58 — `{ text: String, success: bool, turns: usize, duration_ms: u64, truncated: bool, ...` — Result of a subagent execution.
- pub `DelegationOutcome` enum L79-89 — `Success | Error | UnknownAgent` — Outcome of a subagent delegation attempt.
- pub `SubagentSpawner` interface L97-144 — `{ fn list_agents(), fn delegate(), fn delegate_background(), fn has_agent() }` — Trait for spawning and executing subagents.
- pub `SharedSubagentSpawner` type L147 — `= Arc<dyn SubagentSpawner>` — Shared subagent spawner type for use across crates.
-  `has_agent` function L141-143 — `(&self, name: &str) -> bool` — Check if a subagent with the given name exists.

#### crates/arawn-types/src/fs_gate.rs

- pub `FsGateError` enum L17-29 — `AccessDenied | InvalidPath | SandboxError` — Errors from filesystem gate operations.
- pub `SandboxOutput` struct L33-42 — `{ stdout: String, stderr: String, exit_code: i32, success: bool }` — Output from a sandboxed shell command.
- pub `FsGate` interface L50-76 — `{ fn validate_read(), fn validate_write(), fn working_dir(), fn sandbox_execute(...` — Filesystem access gate that enforces workstream boundaries.
- pub `SharedFsGate` type L79 — `= Arc<dyn FsGate>` — Type alias for a shared filesystem gate.
- pub `FsGateResolver` type L86 — `= Arc<dyn Fn(&str, &str) -> Option<Arc<dyn FsGate>> + Send + Sync>` — Resolver that creates an FsGate for a given session and workstream.
- pub `GATED_TOOLS` variable L89 — `: &[&str]` — Tool names that require filesystem gate enforcement.
- pub `is_gated_tool` function L102-104 — `(name: &str) -> bool` — Check if a tool name requires filesystem gate enforcement.

#### crates/arawn-types/src/hooks.rs

- pub `HookEvent` enum L13-40 — `PreToolUse | PostToolUse | PostToolUseFailure | PermissionRequest | UserPromptSu...` — A lifecycle event that hooks can listen for (Claude Code compatible).
- pub `HookType` enum L65-73 — `Command | Prompt | Agent` — Hook type (Claude Code compatible).
- pub `HookAction` struct L92-108 — `{ hook_type: HookType, command: Option<String>, prompt: Option<String>, agent: O...` — A single hook action (Claude Code format).
- pub `HookMatcherGroup` struct L112-118 — `{ matcher: Option<String>, hooks: Vec<HookAction> }` — A matcher group containing hooks (Claude Code format).
- pub `HooksConfig` struct L135-139 — `{ hooks: HashMap<HookEvent, Vec<HookMatcherGroup>> }` — The root hooks.json structure (Claude Code format).
- pub `is_empty` function L143-145 — `(&self) -> bool` — Check if this config has any hooks defined.
- pub `HookDef` struct L150-161 — `{ event: HookEvent, tool_match: Option<String>, match_pattern: Option<String>, c...` — A hook definition (internal format for the dispatcher).
- pub `HookOutcome` enum L165-172 — `Allow | Block | Info` — Outcome of dispatching hooks for an event.
- pub `HookDispatch` interface L179-235 — `{ fn dispatch_pre_tool_use(), fn dispatch_post_tool_use(), fn dispatch_session_s...` — Trait for hook dispatch that can be implemented by different hook systems.
- pub `SharedHookDispatcher` type L238 — `= std::sync::Arc<dyn HookDispatch>` — Shared hook dispatcher type.
-  `HookEvent` type L42-60 — `= HookEvent` — and `arawn-agent` (which calls hooks during tool execution).
-  `fmt` function L43-59 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — and `arawn-agent` (which calls hooks during tool execution).
-  `HooksConfig` type L141-146 — `= HooksConfig` — and `arawn-agent` (which calls hooks during tool execution).
-  `is_empty` function L232-234 — `(&self) -> bool` — Check if the dispatcher has no hooks.
-  `tests` module L241-292 — `-` — and `arawn-agent` (which calls hooks during tool execution).
-  `test_hook_event_display` function L245-253 — `()` — and `arawn-agent` (which calls hooks during tool execution).
-  `test_hook_event_serde_roundtrip` function L256-262 — `()` — and `arawn-agent` (which calls hooks during tool execution).
-  `test_subagent_events_serde` function L265-279 — `()` — and `arawn-agent` (which calls hooks during tool execution).
-  `test_hooks_config_empty` function L282-285 — `()` — and `arawn-agent` (which calls hooks during tool execution).
-  `test_hook_type_default` function L288-291 — `()` — and `arawn-agent` (which calls hooks during tool execution).

#### crates/arawn-types/src/lib.rs

- pub `config` module L3 — `-` — Shared types for the Arawn agent system.
- pub `delegation` module L4 — `-` — Shared types for the Arawn agent system.
- pub `fs_gate` module L5 — `-` — Shared types for the Arawn agent system.
- pub `hooks` module L6 — `-` — Shared types for the Arawn agent system.
- pub `secret_resolver` module L7 — `-` — Shared types for the Arawn agent system.

#### crates/arawn-types/src/secret_resolver.rs

- pub `SecretResolver` interface L15-23 — `{ fn resolve(), fn names() }` — Resolver that looks up secrets by name.
- pub `SharedSecretResolver` type L26 — `= Arc<dyn SecretResolver>` — Type alias for a shared secret resolver.
- pub `SECRET_HANDLE_PREFIX` variable L29 — `: &str` — The handle pattern prefix and suffix for secret references in tool params.
- pub `SECRET_HANDLE_SUFFIX` variable L30 — `: &str` — and `arawn-config` (implementor) can reference it without circular dependencies.
- pub `extract_secret_name` function L45-52 — `(s: &str) -> Option<&str>` — Extract a secret name from a handle string, if it matches the pattern.
- pub `contains_secret_handle` function L55-57 — `(s: &str) -> bool` — Check if a string contains any secret handle references.
- pub `resolve_handles_in_string` function L75-101 — `(s: &str, resolver: &dyn SecretResolver) -> String` — Resolve all `${{secrets.*}}` handles in a string using the given resolver.
- pub `resolve_handles_in_json` function L107-136 — `( value: &serde_json::Value, resolver: &dyn SecretResolver, ) -> serde_json::Val...` — Recursively resolve all secret handles in a JSON value.
-  `tests` module L139-238 — `-` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `TestResolver` struct L142-144 — `{ secrets: std::collections::HashMap<String, String> }` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `TestResolver` type L146-155 — `= TestResolver` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `new` function L147-154 — `(pairs: &[(&str, &str)]) -> Self` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `TestResolver` type L157-164 — `impl SecretResolver for TestResolver` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `resolve` function L158-160 — `(&self, name: &str) -> Option<String>` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `names` function L161-163 — `(&self) -> Vec<String>` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `test_extract_secret_name` function L167-176 — `()` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `test_contains_secret_handle` function L179-183 — `()` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `test_resolve_handles_in_string` function L186-205 — `()` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `test_resolve_handles_in_json_deep` function L208-229 — `()` — and `arawn-config` (implementor) can reference it without circular dependencies.
-  `test_resolve_handles_in_json_no_handles` function L232-237 — `()` — and `arawn-config` (implementor) can reference it without circular dependencies.

### crates/arawn-workstream/src

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-workstream/src/cleanup.rs

- pub `CleanupConfig` struct L17-26 — `{ scratch_cleanup_days: i64, total_usage_warning_bytes: u64, workstream_usage_wa...` — Configuration for cleanup tasks.
- pub `CleanupResult` struct L41-52 — `{ sessions_checked: usize, sessions_cleaned: usize, bytes_reclaimed: u64, cleane...` — Result of a scratch cleanup operation.
- pub `PressureLevel` enum L57-64 — `Ok | Warning | Critical` — Disk pressure alert levels.
- pub `DiskPressureEvent` struct L78-89 — `{ level: PressureLevel, scope: String, usage_mb: f64, limit_mb: f64, timestamp: ...` — Disk pressure event for notification.
- pub `new` function L93-106 — `( level: PressureLevel, scope: impl Into<String>, usage_mb: f64, limit_mb: f64, ...` — Create a new disk pressure event.
- pub `DiskPressureResult` struct L111-120 — `{ total_usage_bytes: u64, workstream_usage: Vec<WorkstreamUsage>, events: Vec<Di...` — Result of a disk pressure check.
- pub `WorkstreamUsage` struct L124-129 — `{ id: String, bytes: u64 }` — Usage for a single workstream.
- pub `cleanup_scratch_sessions` function L145-234 — `( dir_manager: &DirectoryManager, workstream_manager: &WorkstreamManager, config...` — Clean up inactive scratch sessions.
- pub `check_disk_pressure` function L264-371 — `( dir_manager: &DirectoryManager, workstream_manager: &WorkstreamManager, config...` — Check disk pressure across workstreams.
- pub `CleanupContext` struct L378-385 — `{ dir_manager: Arc<DirectoryManager>, workstream_manager: Arc<WorkstreamManager>...` — Cleanup task context for cloacina integration.
- pub `new` function L389-399 — `( dir_manager: Arc<DirectoryManager>, workstream_manager: Arc<WorkstreamManager>...` — Create a new cleanup context.
- pub `run_scratch_cleanup` function L402-404 — `(&self) -> CleanupResult` — Run scratch cleanup.
- pub `run_disk_pressure_check` function L407-409 — `(&self) -> DiskPressureResult` — Run disk pressure check.
-  `CleanupConfig` type L28-37 — `impl Default for CleanupConfig` — disk pressure.
-  `default` function L29-36 — `() -> Self` — disk pressure.
-  `PressureLevel` type L66-74 — `= PressureLevel` — disk pressure.
-  `fmt` function L67-73 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — disk pressure.
-  `DiskPressureEvent` type L91-107 — `= DiskPressureEvent` — disk pressure.
-  `delete_scratch_session_work` function L237-251 — `( dir_manager: &DirectoryManager, session_id: &str, ) -> std::io::Result<()>` — Delete a scratch session's work directory.
-  `CleanupContext` type L387-410 — `= CleanupContext` — disk pressure.
-  `tests` module L413-509 — `-` — disk pressure.
-  `setup` function L417-421 — `() -> (tempfile::TempDir, DirectoryManager)` — disk pressure.
-  `test_cleanup_config_default` function L424-433 — `()` — disk pressure.
-  `test_pressure_level_display` function L436-440 — `()` — disk pressure.
-  `test_disk_pressure_event_new` function L443-449 — `()` — disk pressure.
-  `test_disk_pressure_event_serialization` function L452-461 — `()` — disk pressure.
-  `test_cleanup_result_serialization` function L464-480 — `()` — disk pressure.
-  `test_delete_scratch_session_work_nonexistent` function L483-488 — `()` — disk pressure.
-  `test_delete_scratch_session_work` function L491-508 — `()` — disk pressure.

#### crates/arawn-workstream/src/compression.rs

- pub `CompressorConfig` struct L29-36 — `{ model: String, max_summary_tokens: u32, token_threshold_chars: usize }` — Configuration for the compressor.
- pub `Compressor` struct L53-56 — `{ backend: SharedBackend, config: CompressorConfig }` — Map-reduce context compressor.
- pub `new` function L59-61 — `(backend: SharedBackend, config: CompressorConfig) -> Self`
- pub `compress_session` function L67-96 — `( &self, manager: &WorkstreamManager, session_id: &str, ) -> Result<String>` — Compress a single session's messages into a summary.
- pub `compress_workstream` function L102-143 — `( &self, manager: &WorkstreamManager, workstream_id: &str, ) -> Result<String>` — Reduce all session summaries for a workstream into a single workstream summary.
- pub `needs_compression` function L151-154 — `(&self, messages: &[WorkstreamMessage]) -> bool` — Check if a workstream's current session exceeds the token threshold.
-  `SESSION_SUMMARY_PROMPT` variable L9-16 — `: &str` — Prompts used for compression.
-  `WORKSTREAM_REDUCE_PROMPT` variable L18-25 — `: &str`
-  `CompressorConfig` type L38-47 — `impl Default for CompressorConfig`
-  `default` function L39-46 — `() -> Self`
-  `Compressor` type L58-184 — `= Compressor`
-  `summarize` function L157-183 — `( &self, messages: &[WorkstreamMessage], system_prompt: &str, ) -> Result<String...` — Send messages to LLM with a system prompt for summarization.
-  `filter_session_messages` function L187-198 — `( messages: &'a [WorkstreamMessage], session: &Session, ) -> Vec<&'a WorkstreamM...` — Filter messages that belong to a specific session's time range.
-  `tests` module L201-360 — `-`
-  `test_manager` function L207-213 — `() -> (tempfile::TempDir, WorkstreamManager)`
-  `test_needs_compression_below_threshold` function L216-237 — `()`
-  `test_needs_compression_above_threshold` function L240-261 — `()`
-  `test_compress_session` function L264-299 — `()`
-  `test_compress_workstream_reduces_sessions` function L302-340 — `()`
-  `test_compress_active_session_fails` function L343-359 — `()`

#### crates/arawn-workstream/src/context.rs

- pub `AssembledContext` struct L7-12 — `{ summary: Option<String>, messages: Vec<ContextMessage> }` — Assembled context ready for injection into an LLM request.
- pub `ContextMessage` struct L16-20 — `{ role: ContextRole, content: String }` — A message prepared for LLM context, with role mapped to user/assistant.
- pub `ContextRole` enum L23-27 — `User | Assistant | System`
- pub `as_str` function L30-36 — `(&self) -> &'static str`
- pub `ContextAssembler` struct L40-42 — `{ manager: &'a WorkstreamManager }` — Assembles workstream history into LLM-ready context.
- pub `new` function L45-47 — `(manager: &'a WorkstreamManager) -> Self`
- pub `assemble` function L53-70 — `(&self, workstream_id: &str, max_chars: usize) -> Result<AssembledContext>` — Assemble context for a workstream, fitting within `max_chars` (approximate token budget).
-  `ContextRole` type L29-37 — `= ContextRole`
-  `map_role` function L74-83 — `(role: MessageRole) -> ContextRole` — Map a WorkstreamMessage role to a ContextRole.
-  `fit_messages` function L87-105 — `(messages: &[WorkstreamMessage], budget: usize) -> Vec<ContextMessage>` — Select the most recent messages that fit within `budget` characters.
-  `tests` module L108-233 — `-`
-  `test_manager` function L113-119 — `() -> (tempfile::TempDir, WorkstreamManager)`
-  `test_empty_workstream` function L122-131 — `()`
-  `test_short_history_fits` function L134-150 — `()`
-  `test_long_history_truncated` function L153-176 — `()`
-  `test_summary_reduces_message_budget` function L179-207 — `()`
-  `test_role_mapping` function L210-232 — `()`

#### crates/arawn-workstream/src/error.rs

- pub `WorkstreamError` enum L4-19 — `Database | Migration | NotFound | Io | Serde`
- pub `Result` type L21 — `= std::result::Result<T, WorkstreamError>`

#### crates/arawn-workstream/src/fs_gate.rs

- pub `WorkstreamFsGate` struct L23-29 — `{ path_validator: PathValidator, sandbox_manager: Arc<SandboxManager>, working_d...` — Filesystem gate scoped to a workstream.
- pub `new` function L37-59 — `( dm: &DirectoryManager, sandbox: Arc<SandboxManager>, workstream_id: &str, sess...` — Create a gate for a specific workstream and session.
-  `WorkstreamFsGate` type L31-60 — `= WorkstreamFsGate` — boundaries for all agent tool execution.
-  `WorkstreamFsGate` type L63-142 — `impl FsGate for WorkstreamFsGate` — boundaries for all agent tool execution.
-  `validate_read` function L64-84 — `(&self, path: &Path) -> Result<PathBuf, FsGateError>` — boundaries for all agent tool execution.
-  `validate_write` function L86-104 — `(&self, path: &Path) -> Result<PathBuf, FsGateError>` — boundaries for all agent tool execution.
-  `working_dir` function L106-108 — `(&self) -> &Path` — boundaries for all agent tool execution.
-  `sandbox_execute` function L110-141 — `( &self, command: &str, timeout: Option<Duration>, ) -> Result<SandboxOutput, Fs...` — boundaries for all agent tool execution.
-  `tests` module L145-261 — `-` — boundaries for all agent tool execution.
-  `test_named_workstream_gate_allows_workstream_paths` function L150-171 — `()` — boundaries for all agent tool execution.
-  `test_named_workstream_gate_allows_production_paths` function L174-187 — `()` — boundaries for all agent tool execution.
-  `test_named_workstream_gate_denies_outside_paths` function L190-203 — `()` — boundaries for all agent tool execution.
-  `test_scratch_gate_isolates_sessions` function L206-235 — `()` — boundaries for all agent tool execution.
-  `test_working_dir_named_workstream` function L238-246 — `()` — boundaries for all agent tool execution.
-  `test_working_dir_scratch` function L249-260 — `()` — boundaries for all agent tool execution.

#### crates/arawn-workstream/src/lib.rs

- pub `cleanup` module L6 — `-` — Provides persistent conversational contexts (workstreams) with JSONL message
- pub `compression` module L7 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `context` module L8 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `directory` module L9 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `error` module L10 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `fs_gate` module L11 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `manager` module L12 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `message_store` module L13 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `path_validator` module L14 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `scratch` module L15 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `session` module L16 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `session_loader` module L17 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `storage` module L18 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `store` module L19 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `types` module L20 — `-` — history as the source of truth and SQLite as an operational cache layer.
- pub `watcher` module L21 — `-` — history as the source of truth and SQLite as an operational cache layer.

#### crates/arawn-workstream/src/manager.rs

- pub `WorkstreamConfig` struct L13-20 — `{ db_path: PathBuf, data_dir: PathBuf, session_timeout_minutes: i64 }` — Configuration for the workstream manager.
- pub `WorkstreamManager` struct L27-32 — `{ store: WorkstreamStore, message_store: MessageStore, session_timeout_minutes: ...` — High-level facade coordinating message store, session manager,
- pub `new` function L36-52 — `(config: &WorkstreamConfig) -> Result<Self>` — Initialize the manager: opens SQLite, runs migrations, sets up data dirs.
- pub `from_parts` function L55-66 — `( store: WorkstreamStore, message_store: MessageStore, session_timeout_minutes: ...` — Create from pre-built components (for testing).
- pub `with_directory_manager` function L72-75 — `(mut self, dm: DirectoryManager) -> Self` — Set the directory manager for file path management.
- pub `directory_manager` function L78-80 — `(&self) -> Option<&DirectoryManager>` — Get a reference to the directory manager, if configured.
- pub `create_workstream` function L84-109 — `( &self, title: &str, default_model: Option<&str>, tags: &[String], ) -> Result<...`
- pub `get_workstream` function L111-113 — `(&self, id: &str) -> Result<Workstream>`
- pub `list_workstreams` function L115-117 — `(&self) -> Result<Vec<Workstream>>`
- pub `list_all_workstreams` function L120-122 — `(&self) -> Result<Vec<Workstream>>` — List all workstreams (including archived).
- pub `archive_workstream` function L124-132 — `(&self, id: &str) -> Result<()>`
- pub `update_workstream` function L135-151 — `( &self, id: &str, title: Option<&str>, summary: Option<&str>, default_model: Op...` — Update a workstream's title, summary, and/or default model.
- pub `set_tags` function L154-158 — `(&self, workstream_id: &str, tags: &[String]) -> Result<()>` — Update tags for a workstream.
- pub `get_tags` function L160-162 — `(&self, workstream_id: &str) -> Result<Vec<String>>`
- pub `send_message` function L168-211 — `( &self, workstream_id: Option<&str>, session_id: Option<&str>, role: MessageRol...` — Send a message to a workstream.
- pub `push_agent_message` function L214-231 — `( &self, workstream_id: &str, content: &str, metadata: Option<&str>, ) -> Result...` — Push a message from a background agent/process into a workstream.
- pub `get_messages` function L234-236 — `(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>>` — Read all messages for a workstream.
- pub `get_messages_since` function L239-245 — `( &self, workstream_id: &str, since: chrono::DateTime<chrono::Utc>, ) -> Result<...` — Read messages since a given timestamp.
- pub `get_active_session` function L249-251 — `(&self, workstream_id: &str) -> Result<Option<Session>>`
- pub `end_session` function L253-255 — `(&self, session_id: &str) -> Result<()>`
- pub `list_sessions` function L257-259 — `(&self, workstream_id: &str) -> Result<Vec<Session>>`
- pub `reassign_session` function L262-286 — `(&self, session_id: &str, new_workstream_id: &str) -> Result<Session>` — Move a session to a different workstream.
- pub `timeout_check` function L289-291 — `(&self) -> Result<usize>` — Run a timeout check across all workstreams.
- pub `promote_scratch` function L295-303 — `( &self, new_title: &str, tags: &[String], default_model: Option<&str>, ) -> Res...`
- pub `store` function L335-337 — `(&self) -> &WorkstreamStore` — Access the underlying store (for advanced operations).
- pub `message_store` function L340-342 — `(&self) -> &MessageStore` — Access the underlying message store.
-  `WorkstreamManager` type L34-343 — `= WorkstreamManager`
-  `resolve_workstream` function L308-320 — `(&self, workstream_id: Option<&str>) -> Result<String>` — Resolve workstream_id, defaulting to scratch.
-  `session_manager` function L322-328 — `(&self) -> SessionManager<'_>`
-  `scratch_manager` function L330-332 — `(&self) -> ScratchManager<'_>`
-  `tests` module L346-495 — `-`
-  `test_manager` function L349-355 — `() -> (tempfile::TempDir, WorkstreamManager)`
-  `test_create_and_list_workstreams` function L358-372 — `()`
-  `test_send_message_full_cycle` function L375-403 — `()`
-  `test_scratch_auto_create_on_send` function L406-417 — `()`
-  `test_agent_push` function L420-433 — `()`
-  `test_archive_workstream` function L436-449 — `()`
-  `test_cannot_archive_scratch` function L452-461 — `()`
-  `test_send_to_nonexistent_workstream_fails` function L464-471 — `()`
-  `test_promote_scratch_via_manager` function L474-494 — `()`

#### crates/arawn-workstream/src/message_store.rs

- pub `MessageStore` struct L14-16 — `{ data_dir: PathBuf }` — Append-only JSONL message store.
- pub `new` function L19-23 — `(data_dir: &Path) -> Self`
- pub `append` function L26-57 — `( &self, workstream_id: &str, session_id: Option<&str>, role: MessageRole, conte...` — Append a message to the workstream's JSONL file.
- pub `read_all` function L60-80 — `(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>>` — Read all messages for a workstream.
- pub `read_range` function L83-90 — `( &self, workstream_id: &str, since: DateTime<Utc>, ) -> Result<Vec<WorkstreamMe...` — Read messages after a given timestamp.
- pub `read_for_session` function L93-103 — `( &self, workstream_id: &str, session_id: &str, ) -> Result<Vec<WorkstreamMessag...` — Read all messages for a specific session.
- pub `workstream_dir` function L106-108 — `(&self, workstream_id: &str) -> PathBuf` — Path to a workstream's data directory.
- pub `jsonl_path` function L111-113 — `(&self, workstream_id: &str) -> PathBuf` — Path to a workstream's JSONL file.
- pub `move_messages` function L119-154 — `(&self, from_workstream: &str, to_workstream: &str) -> Result<()>` — Move all messages from one workstream to another.
- pub `delete_all` function L157-163 — `(&self, workstream_id: &str) -> Result<()>` — Delete all messages for a workstream.
-  `MessageStore` type L18-164 — `= MessageStore`
-  `MessageStore` type L170-201 — `= MessageStore`
-  `append` function L171-180 — `( &self, workstream_id: &str, session_id: Option<&str>, role: MessageRole, conte...`
-  `read_all` function L182-184 — `(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>>`
-  `read_range` function L186-192 — `( &self, workstream_id: &str, since: DateTime<Utc>, ) -> Result<Vec<WorkstreamMe...`
-  `move_messages` function L194-196 — `(&self, from_workstream: &str, to_workstream: &str) -> Result<()>`
-  `delete_all` function L198-200 — `(&self, workstream_id: &str) -> Result<()>`
-  `tests` module L204-356 — `-`
-  `temp_store` function L207-211 — `() -> (tempfile::TempDir, MessageStore)`
-  `test_append_and_read_all` function L214-227 — `()`
-  `test_multi_message_append` function L230-248 — `()`
-  `test_read_range` function L251-267 — `()`
-  `test_missing_workstream_returns_empty` function L270-274 — `()`
-  `test_metadata_roundtrip` function L277-293 — `()`
-  `test_separate_workstreams` function L296-308 — `()`
-  `test_read_for_session` function L311-355 — `()`

#### crates/arawn-workstream/src/path_validator.rs

- pub `PathError` enum L36-63 — `NotAllowed | DeniedPath | SymlinkEscape | Invalid | ParentNotFound | Io` — Errors that can occur during path validation.
- pub `PathResult` type L66 — `= std::result::Result<T, PathError>` — Result type for path validation operations.
- pub `PathValidator` struct L72-77 — `{ allowed_paths: Vec<PathBuf>, denied_paths: Vec<PathBuf> }` — Validates that file operations stay within allowed boundaries.
- pub `new` function L92-98 — `(allowed_paths: Vec<PathBuf>) -> Self` — Creates a new PathValidator with the given allowed paths.
- pub `with_denied` function L101-106 — `(allowed_paths: Vec<PathBuf>, denied_paths: Vec<PathBuf>) -> Self` — Creates a PathValidator with custom allowed and denied paths.
- pub `default_denied_paths` function L113-150 — `() -> Vec<PathBuf>` — Returns the default list of denied system paths.
- pub `allowed_paths` function L153-155 — `(&self) -> &[PathBuf]` — Get the allowed paths.
- pub `denied_paths` function L158-160 — `(&self) -> &[PathBuf]` — Get the denied paths.
- pub `validate` function L177-213 — `(&self, path: &Path) -> PathResult<PathBuf>` — Validate a path for read operations.
- pub `validate_write` function L244-302 — `(&self, path: &Path) -> PathResult<PathBuf>` — Validate a path for write operations.
- pub `validate_for_shell` function L380-397 — `(&self, path: &Path) -> PathResult<PathBuf>` — Validate that a path is safe for shell execution.
- pub `for_session` function L408-415 — `( directory_manager: &crate::directory::DirectoryManager, workstream: &str, sess...` — Create a validator for a specific workstream and session.
-  `PathValidator` type L79-398 — `= PathValidator` — ```
-  `check_denied` function L305-322 — `(&self, path: &Path) -> PathResult<()>` — Check if a path is within any denied directory.
-  `check_allowed` function L325-343 — `(&self, path: &Path) -> PathResult<()>` — Check if a path is within any allowed directory.
-  `is_under_allowed_canonical` function L348-368 — `(&self, path: &Path) -> bool` — Check if a path is under an allowed directory (using canonicalized allowed paths).
-  `SHELL_METACHARACTERS` variable L386-388 — `: &[char]` — ```
-  `PathValidator` type L404-416 — `= PathValidator` — Create a PathValidator from a DirectoryManager for a specific session.
-  `tests` module L419-760 — `-` — ```
-  `setup` function L424-429 — `() -> (tempfile::TempDir, PathValidator)` — ```
-  `test_validate_existing_file` function L432-442 — `()` — ```
-  `test_validate_nonexistent_file_fails` function L445-452 — `()` — ```
-  `test_validate_write_new_file` function L455-464 — `()` — ```
-  `test_validate_write_nested_directory` function L467-479 — `()` — ```
-  `test_validate_write_nonexistent_parent_fails` function L482-489 — `()` — ```
-  `test_path_outside_allowed_rejected` function L492-506 — `()` — ```
-  `test_traversal_attack_rejected` function L510-523 — `()` — ```
-  `test_symlink_within_allowed_succeeds` function L526-539 — `()` — ```
-  `test_symlink_escape_rejected` function L542-564 — `()` — ```
-  `test_denied_path_rejected` function L567-583 — `()` — ```
-  `test_validate_for_shell_rejects_metacharacters` function L586-604 — `()` — ```
-  `test_default_denied_paths` function L607-619 — `()` — ```
-  `test_empty_allowed_paths_rejects_all` function L622-631 — `()` — ```
-  `test_multiple_allowed_paths` function L634-650 — `()` — ```
-  `test_for_session_creates_validator` function L653-664 — `()` — ```
-  `test_thread_safety` function L667-670 — `()` — ```
-  `assert_send_sync` function L668 — `()` — ```
-  `test_validate_write_symlink_escape_rejected` function L673-695 — `()` — ```
-  `test_validate_write_symlink_within_allowed_succeeds` function L698-715 — `()` — ```
-  `test_validate_write_symlink_dir_escape_rejected` function L718-738 — `()` — ```
-  `test_validate_write_just_filename` function L741-759 — `()` — ```
-  `proptests` module L764-905 — `-` — Property-based tests for path validation security.
-  `traversal_path_strategy` function L770-781 — `() -> impl Strategy<Value = String>` — Strategy to generate paths with path traversal sequences.
-  `shell_metachar_path_strategy` function L784-794 — `() -> impl Strategy<Value = String>` — Strategy to generate paths with shell metacharacters.

#### crates/arawn-workstream/src/scratch.rs

- pub `SCRATCH_ID` variable L11 — `: &str` — Well-known scratch workstream ID.
- pub `ScratchManager` struct L14-17 — `{ store: &'a WorkstreamStore, message_store: &'a MessageStore }` — Manages the scratch workstream and promotion to named workstreams.
- pub `new` function L20-25 — `(store: &'a WorkstreamStore, message_store: &'a MessageStore) -> Self`
- pub `ensure_scratch` function L28-30 — `(&self) -> Result<crate::store::Workstream>` — Ensure the scratch workstream exists, creating it if missing.
- pub `promote` function L38-117 — `( &self, new_title: &str, tags: &[String], default_model: Option<&str>, ) -> Res...` — Promote the scratch workstream to a named workstream.
-  `tests` module L121-221 — `-`
-  `setup` function L125-130 — `() -> (tempfile::TempDir, WorkstreamStore, MessageStore)`
-  `test_ensure_scratch_idempotent` function L133-142 — `()`
-  `test_promote_moves_messages` function L145-188 — `()`
-  `test_promote_empty_scratch_fails` function L191-202 — `()`
-  `test_scratch_cannot_be_deleted` function L205-220 — `()`

#### crates/arawn-workstream/src/session.rs

- pub `SessionManager` struct L11-15 — `{ store: &'a WorkstreamStore, message_store: &'a MessageStore, timeout: Duration...` — Manages session lifecycle within workstreams.
- pub `new` function L18-28 — `( store: &'a WorkstreamStore, message_store: &'a MessageStore, timeout_minutes: ...`
- pub `get_or_start_session` function L35-45 — `(&self, workstream_id: &str) -> Result<Session>` — Get or start a session for the workstream.
- pub `end_session` function L48-52 — `(&self, session_id: &str) -> Result<()>` — Explicitly end a session, counting its messages from JSONL.
- pub `timeout_check` function L56-71 — `(&self) -> Result<usize>` — Scan for and end all timed-out sessions across all workstreams.
-  `is_timed_out` function L73-75 — `(&self, session: &Session) -> bool`
-  `count_session_messages` function L77-86 — `(&self, session: &Session) -> Result<i32>`
-  `tests` module L90-202 — `-`
-  `setup` function L94-99 — `() -> (tempfile::TempDir, WorkstreamStore, MessageStore)`
-  `test_get_or_start_creates_session` function L102-113 — `()`
-  `test_end_session_counts_messages` function L116-141 — `()`
-  `test_one_active_constraint` function L144-160 — `()`
-  `test_timeout_creates_new_session` function L163-178 — `()`
-  `test_timeout_check_bulk` function L181-201 — `()`

#### crates/arawn-workstream/src/session_loader.rs

- pub `ToolUseMetadata` struct L18-25 — `{ tool_id: String, name: String, arguments: serde_json::Value }` — Metadata for a tool use message.
- pub `ToolResultMetadata` struct L29-34 — `{ tool_call_id: String, success: bool }` — Metadata for a tool result message.
- pub `ReconstructedTurn` struct L38-53 — `{ id: String, user_message: String, assistant_response: Option<String>, tool_cal...` — A reconstructed turn from JSONL messages.
- pub `ReconstructedToolCall` struct L57-64 — `{ id: String, name: String, arguments: serde_json::Value }` — A reconstructed tool call.
- pub `ReconstructedToolResult` struct L68-75 — `{ tool_call_id: String, success: bool, content: String }` — A reconstructed tool result.
- pub `ReconstructedSession` struct L79-90 — `{ session_id: String, workstream_id: String, turns: Vec<ReconstructedTurn>, crea...` — A fully reconstructed session from JSONL messages.
- pub `SessionLoader` struct L93-95 — `{ message_store: &'a MessageStore }` — Loads and reconstructs sessions from JSONL message history.
- pub `new` function L99-101 — `(message_store: &'a MessageStore) -> Self` — Create a new session loader.
- pub `load_session` function L106-136 — `( &self, workstream_id: &str, session_id: &str, ) -> Result<Option<Reconstructed...` — Load and reconstruct a session from JSONL messages.
- pub `save_turn` function L252-317 — `( &self, workstream_id: &str, session_id: &str, user_message: &str, tool_calls: ...` — Save a turn to JSONL storage.
-  `reconstruct_turns` function L141-246 — `(&self, messages: &[WorkstreamMessage]) -> Vec<ReconstructedTurn>` — Reconstruct turns from a list of messages.
-  `tests` module L321-525 — `-` — of truth for conversation history.
-  `temp_store` function L324-328 — `() -> (tempfile::TempDir, MessageStore)` — of truth for conversation history.
-  `test_load_empty_session` function L331-337 — `()` — of truth for conversation history.
-  `test_load_simple_session` function L340-393 — `()` — of truth for conversation history.
-  `test_load_session_with_tool_calls` function L396-472 — `()` — of truth for conversation history.
-  `test_save_turn` function L475-506 — `()` — of truth for conversation history.
-  `test_incomplete_turn` function L509-524 — `()` — of truth for conversation history.

#### crates/arawn-workstream/src/storage.rs

- pub `WorkstreamStorage` interface L29-86 — `{ fn create_workstream(), fn get_workstream(), fn list_workstreams(), fn update_...` — Trait for workstream metadata storage.
- pub `MessageStorage` interface L92-118 — `{ fn append(), fn read_all(), fn read_range(), fn move_messages(), fn delete_all...` — Trait for message storage (conversation history).
- pub `MockWorkstreamStorage` struct L123-127 — `{ workstreams: std::sync::Mutex<std::collections::HashMap<String, Workstream>>, ...` — Mock implementation of WorkstreamStorage for testing.
- pub `new` function L132-134 — `() -> Self` — Create a new empty mock storage.
- pub `MockMessageStorage` struct L324-326 — `{ messages: std::sync::Mutex<std::collections::HashMap<String, Vec<WorkstreamMes...` — Mock implementation of MessageStorage for testing.
- pub `new` function L331-333 — `() -> Self` — Create a new empty mock storage.
-  `MockWorkstreamStorage` type L130-135 — `= MockWorkstreamStorage` — ```
-  `MockWorkstreamStorage` type L138-319 — `impl WorkstreamStorage for MockWorkstreamStorage` — ```
-  `create_workstream` function L139-168 — `( &self, title: &str, default_model: Option<&str>, is_scratch: bool, ) -> Result...` — ```
-  `get_workstream` function L170-177 — `(&self, id: &str) -> Result<Workstream>` — ```
-  `list_workstreams` function L179-188 — `(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>>` — ```
-  `update_workstream` function L190-217 — `( &self, id: &str, title: Option<&str>, summary: Option<&str>, state: Option<&st...` — ```
-  `set_tags` function L219-229 — `(&self, workstream_id: &str, tags: &[String]) -> Result<()>` — ```
-  `get_tags` function L231-239 — `(&self, workstream_id: &str) -> Result<Vec<String>>` — ```
-  `create_session` function L241-244 — `(&self, workstream_id: &str) -> Result<Session>` — ```
-  `create_session_with_id` function L246-267 — `(&self, session_id: &str, workstream_id: &str) -> Result<Session>` — ```
-  `get_active_session` function L269-275 — `(&self, workstream_id: &str) -> Result<Option<Session>>` — ```
-  `list_sessions` function L277-286 — `(&self, workstream_id: &str) -> Result<Vec<Session>>` — ```
-  `end_session` function L288-296 — `(&self, session_id: &str) -> Result<()>` — ```
-  `reassign_session` function L298-318 — `(&self, session_id: &str, new_workstream_id: &str) -> Result<Session>` — ```
-  `MockMessageStorage` type L329-334 — `= MockMessageStorage` — ```
-  `MockMessageStorage` type L337-412 — `impl MessageStorage for MockMessageStorage` — ```
-  `append` function L338-364 — `( &self, workstream_id: &str, session_id: Option<&str>, role: crate::types::Mess...` — ```
-  `read_all` function L366-374 — `(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>>` — ```
-  `read_range` function L376-393 — `( &self, workstream_id: &str, since: DateTime<Utc>, ) -> Result<Vec<WorkstreamMe...` — ```
-  `move_messages` function L395-406 — `(&self, from_workstream: &str, to_workstream: &str) -> Result<()>` — ```
-  `delete_all` function L408-411 — `(&self, workstream_id: &str) -> Result<()>` — ```
-  `tests` module L415-515 — `-` — ```
-  `test_mock_workstream_storage_crud` function L420-444 — `()` — ```
-  `test_mock_workstream_storage_tags` function L447-460 — `()` — ```
-  `test_mock_workstream_storage_sessions` function L463-483 — `()` — ```
-  `test_mock_message_storage` function L486-514 — `()` — ```

#### crates/arawn-workstream/src/store.rs

- pub `Workstream` struct L19-28 — `{ id: String, title: String, summary: Option<String>, is_scratch: bool, state: S...` — A persistent conversational context.
- pub `Session` struct L32-40 — `{ id: String, workstream_id: String, started_at: DateTime<Utc>, ended_at: Option...` — A turn batch within a workstream.
- pub `WorkstreamStore` struct L45-47 — `{ conn: Mutex<Connection> }` — Thin repository over SQLite for workstream operational data.
- pub `open` function L51-60 — `(path: &Path) -> Result<Self>` — Open (or create) the database at `path` and run pending migrations.
- pub `open_in_memory` function L63-72 — `() -> Result<Self>` — Open an in-memory database (for testing).
- pub `create_workstream` function L89-119 — `( &self, title: &str, default_model: Option<&str>, is_scratch: bool, ) -> Result...`
- pub `get_workstream` function L121-142 — `(&self, id: &str) -> Result<Workstream>`
- pub `list_workstreams` function L144-170 — `(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>>`
- pub `update_workstream` function L172-222 — `( &self, id: &str, title: Option<&str>, summary: Option<&str>, state: Option<&st...`
- pub `reassign_sessions` function L227-233 — `(&self, from_id: &str, to_id: &str) -> Result<()>` — Move all sessions from one workstream to another.
- pub `reassign_tags` function L236-242 — `(&self, from_id: &str, to_id: &str) -> Result<()>` — Move all tags from one workstream to another.
- pub `set_tags` function L246-259 — `(&self, workstream_id: &str, tags: &[String]) -> Result<()>`
- pub `get_tags` function L261-271 — `(&self, workstream_id: &str) -> Result<Vec<String>>`
- pub `create_session` function L275-278 — `(&self, workstream_id: &str) -> Result<Session>`
- pub `create_session_with_id` function L281-321 — `(&self, id: &str, workstream_id: &str) -> Result<Session>` — Create a session with a specific ID, or return existing if already exists.
- pub `get_session` function L323-333 — `(&self, id: &str) -> Result<Session>`
- pub `get_active_session` function L335-346 — `(&self, workstream_id: &str) -> Result<Option<Session>>`
- pub `end_session` function L348-358 — `(&self, id: &str, turn_count: i32) -> Result<()>`
- pub `update_session_summary` function L360-369 — `(&self, id: &str, summary: &str) -> Result<()>`
- pub `reassign_session` function L372-426 — `(&self, session_id: &str, new_workstream_id: &str) -> Result<Session>` — Move a session to a different workstream.
- pub `list_sessions` function L428-440 — `(&self, workstream_id: &str) -> Result<Vec<Session>>`
- pub `ensure_scratch` function L445-451 — `(&self) -> Result<Workstream>` — Ensure the well-known scratch workstream exists, creating it if missing.
-  `embedded` module L12-15 — `-`
-  `WorkstreamStore` type L49-452 — `= WorkstreamStore`
-  `run_migrations` function L74-80 — `(&mut self) -> Result<()>`
-  `conn` function L83-85 — `(&self) -> parking_lot::MutexGuard<'_, Connection>` — Lock the connection for use.
-  `parse_dt` function L456-467 — `(s: &str) -> DateTime<Utc>`
-  `row_to_workstream` function L469-480 — `(row: &rusqlite::Row<'_>) -> rusqlite::Result<Workstream>`
-  `row_to_session` function L482-492 — `(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session>`
-  `WorkstreamStore` type L498-559 — `= WorkstreamStore`
-  `create_workstream` function L499-506 — `( &self, title: &str, default_model: Option<&str>, is_scratch: bool, ) -> Result...`
-  `get_workstream` function L508-510 — `(&self, id: &str) -> Result<Workstream>`
-  `list_workstreams` function L512-514 — `(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>>`
-  `update_workstream` function L516-525 — `( &self, id: &str, title: Option<&str>, summary: Option<&str>, state: Option<&st...`
-  `set_tags` function L527-529 — `(&self, workstream_id: &str, tags: &[String]) -> Result<()>`
-  `get_tags` function L531-533 — `(&self, workstream_id: &str) -> Result<Vec<String>>`
-  `create_session` function L535-537 — `(&self, workstream_id: &str) -> Result<Session>`
-  `create_session_with_id` function L539-541 — `(&self, session_id: &str, workstream_id: &str) -> Result<Session>`
-  `get_active_session` function L543-545 — `(&self, workstream_id: &str) -> Result<Option<Session>>`
-  `list_sessions` function L547-549 — `(&self, workstream_id: &str) -> Result<Vec<Session>>`
-  `end_session` function L551-554 — `(&self, session_id: &str) -> Result<()>`
-  `reassign_session` function L556-558 — `(&self, session_id: &str, new_workstream_id: &str) -> Result<Session>`
-  `tests` module L562-665 — `-`
-  `test_store` function L565-567 — `() -> WorkstreamStore`
-  `test_migrations_run` function L570-572 — `()`
-  `test_workstream_crud` function L575-602 — `()`
-  `test_tags` function L605-619 — `()`
-  `test_session_lifecycle` function L622-644 — `()`
-  `test_scratch_auto_creation` function L647-657 — `()`
-  `test_not_found` function L660-664 — `()`

#### crates/arawn-workstream/src/types.rs

- pub `MessageRole` enum L7-18 — `User | Assistant | System | ToolUse | ToolResult | AgentPush` — Role of a message within a workstream.
- pub `as_str` function L21-30 — `(&self) -> &'static str`
- pub `WorkstreamMessage` struct L41-51 — `{ id: String, workstream_id: String, session_id: Option<String>, role: MessageRo...` — A single message in a workstream's conversation history.
-  `MessageRole` type L20-31 — `= MessageRole`
-  `MessageRole` type L33-37 — `= MessageRole`
-  `fmt` function L34-36 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

#### crates/arawn-workstream/src/watcher.rs

- pub `DEFAULT_DEBOUNCE_MS` variable L23 — `: u64` — Default debounce duration in milliseconds.
- pub `DEFAULT_POLL_INTERVAL_SECS` variable L26 — `: u64` — Default polling interval when native watching is unavailable.
- pub `WatcherError` enum L30-46 — `InitFailed | WatchFailed | WorkstreamNotFound | InvalidName` — Errors that can occur during filesystem watching.
- pub `WatcherResult` type L49 — `= std::result::Result<T, WatcherError>` — Result type for watcher operations.
- pub `FsAction` enum L54-61 — `Created | Modified | Deleted` — Actions that can occur on a file.
- pub `FsChangeEvent` struct L75-84 — `{ workstream: String, path: String, action: FsAction, timestamp: DateTime<Utc> }` — Event emitted when a file changes in a workstream directory.
- pub `new` function L88-95 — `(workstream: impl Into<String>, path: impl Into<String>, action: FsAction) -> Se...` — Create a new filesystem change event.
- pub `WatcherHandle` struct L99-101 — `{ handle: std::thread::JoinHandle<()> }` — Handle to the running watcher thread.
- pub `is_running` function L105-107 — `(&self) -> bool` — Check if the watcher thread is still running.
- pub `FileWatcherConfig` struct L112-117 — `{ debounce_ms: u64, buffer_size: usize }` — Configuration for the file watcher.
- pub `FileWatcher` struct L132-139 — `{ directory_manager: DirectoryManager, config: FileWatcherConfig, watched: Arc<R...` — Watches workstream directories for file changes.
- pub `new` function L143-145 — `(directory_manager: DirectoryManager) -> Self` — Create a new file watcher with default configuration.
- pub `with_config` function L148-154 — `(directory_manager: DirectoryManager, config: FileWatcherConfig) -> Self` — Create a new file watcher with custom configuration.
- pub `start` function L169-291 — `( &self, workstreams: &[&str], ) -> WatcherResult<(mpsc::Receiver<FsChangeEvent>...` — Start watching and return a receiver for events.
- pub `watched_workstreams` function L327-329 — `(&self) -> Vec<String>` — List currently watched workstreams.
-  `FsAction` type L63-71 — `= FsAction` — that can be broadcast via WebSocket to connected clients.
-  `fmt` function L64-70 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — that can be broadcast via WebSocket to connected clients.
-  `FsChangeEvent` type L86-96 — `= FsChangeEvent` — that can be broadcast via WebSocket to connected clients.
-  `WatcherHandle` type L103-108 — `= WatcherHandle` — that can be broadcast via WebSocket to connected clients.
-  `FileWatcherConfig` type L119-126 — `impl Default for FileWatcherConfig` — that can be broadcast via WebSocket to connected clients.
-  `default` function L120-125 — `() -> Self` — that can be broadcast via WebSocket to connected clients.
-  `FileWatcher` type L141-330 — `= FileWatcher` — that can be broadcast via WebSocket to connected clients.
-  `get_watch_paths` function L294-324 — `(&self, workstream_id: &str) -> WatcherResult<Vec<PathBuf>>` — Get the paths to watch for a workstream.
-  `find_workstream_for_path` function L333-356 — `( path: &Path, workstreams_root: &Path, path_to_workstream: &HashMap<PathBuf, St...` — Find the workstream ID for a given file path.
-  `calculate_relative_path` function L359-369 — `( path: &Path, workstreams_root: &Path, workstream: &str, ) -> Option<String>` — Calculate the relative path within a workstream.
-  `tests` module L372-557 — `-` — that can be broadcast via WebSocket to connected clients.
-  `setup` function L377-381 — `() -> (tempfile::TempDir, DirectoryManager)` — that can be broadcast via WebSocket to connected clients.
-  `test_fs_action_display` function L384-388 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_fs_change_event_new` function L391-397 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_fs_change_event_serialization` function L400-412 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_file_watcher_config_default` function L415-419 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_get_watch_paths_named_workstream` function L422-434 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_get_watch_paths_scratch` function L437-449 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_get_watch_paths_nonexistent` function L452-459 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_get_watch_paths_invalid_name` function L462-472 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_find_workstream_for_path` function L475-498 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_calculate_relative_path` function L501-517 — `()` — that can be broadcast via WebSocket to connected clients.
-  `test_watcher_start_and_detect_changes` function L521-556 — `()` — that can be broadcast via WebSocket to connected clients.

### crates/arawn-workstream/src/directory

> *Semantic summary to be generated by AI agent.*

#### crates/arawn-workstream/src/directory/clone.rs

- pub `clone_repo` function L46-115 — `( &self, workstream: &str, url: &str, name: Option<&str>, ) -> DirectoryResult<C...` — Clones a git repository into the workstream's `production/` directory.
-  `DirectoryManager` type L9-154 — `= DirectoryManager` — Git clone operations for workstreams.
-  `repo_name_from_url` function L123-129 — `(url: &str) -> &str` — Derive repository name from URL.
-  `is_git_available` function L132-138 — `() -> bool` — Check if git is available on the system.
-  `get_head_commit` function L141-153 — `(repo_path: &Path) -> DirectoryResult<String>` — Get the HEAD commit hash for a repository.
-  `tests` module L157-314 — `-` — Git clone operations for workstreams.
-  `setup` function L163-167 — `() -> (tempfile::TempDir, DirectoryManager)` — Git clone operations for workstreams.
-  `test_repo_name_from_url_https` function L169-182 — `()` — Git clone operations for workstreams.
-  `test_repo_name_from_url_ssh` function L185-192 — `()` — Git clone operations for workstreams.
-  `test_repo_name_from_url_fallback` function L195-198 — `()` — Git clone operations for workstreams.
-  `test_clone_workstream_not_found` function L201-209 — `()` — Git clone operations for workstreams.
-  `test_clone_invalid_workstream_name` function L212-220 — `()` — Git clone operations for workstreams.
-  `test_clone_destination_exists` function L223-241 — `()` — Git clone operations for workstreams.
-  `test_clone_custom_name_conflict` function L244-262 — `()` — Git clone operations for workstreams.
-  `test_is_git_available` function L265-271 — `()` — Git clone operations for workstreams.
-  `test_clone_public_repo` function L276-295 — `()` — Git clone operations for workstreams.
-  `test_clone_invalid_url` function L299-313 — `()` — Git clone operations for workstreams.

#### crates/arawn-workstream/src/directory/manager.rs

- pub `DirectoryManager` struct L28-30 — `{ base_path: PathBuf }` — Manages the convention-based directory structure for workstreams and sessions.
- pub `new` function L44-48 — `(base_path: impl Into<PathBuf>) -> Self` — Creates a new DirectoryManager with a custom base path.
- pub `base_path` function L51-53 — `(&self) -> &Path` — Returns the base path for all arawn data.
- pub `workstreams_root` function L56-58 — `(&self) -> PathBuf` — Returns the root path for all workstreams.
- pub `workstream_path` function L66-72 — `(&self, name: &str) -> PathBuf` — Returns the path to a specific workstream's directory.
- pub `production_path` function L75-77 — `(&self, workstream: &str) -> PathBuf` — Returns the production directory path for a workstream.
- pub `work_path` function L80-82 — `(&self, workstream: &str) -> PathBuf` — Returns the work directory path for a workstream.
- pub `scratch_session_path` function L89-98 — `(&self, session_id: &str) -> PathBuf` — Returns the path for a scratch session's isolated work directory.
- pub `is_valid_name` function L107-121 — `(name: &str) -> bool` — Checks if a workstream name is valid.
- pub `is_valid_session_id` function L126-128 — `(id: &str) -> bool` — Checks if a session ID is valid.
- pub `validate_workstream_id` function L134-140 — `(id: &str) -> DirectoryResult<()>` — Validate a workstream ID, returning an error if invalid.
- pub `validate_session_id` function L143-149 — `(id: &str) -> DirectoryResult<()>` — Validate a session ID, returning an error if invalid.
- pub `workstream_exists` function L152-154 — `(&self, name: &str) -> bool` — Checks if a workstream exists (has a directory).
- pub `allowed_paths` function L170-178 — `(&self, workstream: &str, session_id: &str) -> Vec<PathBuf>` — Returns the allowed paths for a session based on its workstream.
- pub `create_workstream` function L201-221 — `(&self, name: &str) -> DirectoryResult<PathBuf>` — Creates a workstream directory structure.
- pub `create_scratch_session` function L241-257 — `(&self, session_id: &str) -> DirectoryResult<PathBuf>` — Creates a scratch session's isolated work directory.
- pub `remove_scratch_session` function L269-290 — `(&self, session_id: &str) -> DirectoryResult<()>` — Removes a scratch session's directory tree.
- pub `list_scratch_sessions` function L293-311 — `(&self) -> DirectoryResult<Vec<String>>` — Lists all scratch session IDs that have directories.
- pub `list_workstreams` function L314-335 — `(&self) -> DirectoryResult<Vec<String>>` — Lists all workstream names that have directories (excluding scratch).
-  `DirectoryManager` type L32-40 — `impl Default for DirectoryManager` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `default` function L34-39 — `() -> Self` — Creates a DirectoryManager with the default base path `~/.arawn`.
-  `DirectoryManager` type L42-336 — `= DirectoryManager` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `tests` module L339-598 — `-` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `setup` function L343-347 — `() -> (tempfile::TempDir, DirectoryManager)` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_default_base_path` function L350-354 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_custom_base_path` function L357-360 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_is_valid_name` function L363-377 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_is_valid_name_rejects_traversal` function L380-385 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_validate_workstream_id` function L388-400 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_validate_session_id` function L403-407 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_workstream_paths` function L410-421 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_scratch_session_path` function L424-429 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_allowed_paths_named_workstream` function L432-443 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_allowed_paths_scratch` function L446-458 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_create_workstream` function L461-468 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_create_workstream_idempotent` function L471-477 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_create_workstream_invalid_name` function L480-488 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_create_scratch_session` function L491-497 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_create_scratch_session_idempotent` function L500-506 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_create_scratch_session_invalid_id` function L509-514 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_remove_scratch_session` function L517-534 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_remove_nonexistent_session_is_noop` function L537-542 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_list_scratch_sessions` function L545-559 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_list_workstreams` function L562-579 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_workstream_exists` function L582-590 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `test_thread_safety` function L593-597 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.
-  `assert_send_sync` function L595 — `()` — DirectoryManager core: struct, path construction, validation, and CRUD operations.

#### crates/arawn-workstream/src/directory/mod.rs

- pub `DirectoryError` enum L28-68 — `Io | InvalidName | InvalidSessionId | SourceNotFound | NotAFile | WorkstreamNotF...` — Errors that can occur during directory operations.
- pub `DirectoryResult` type L71 — `= std::result::Result<T, DirectoryError>` — Result type for directory operations.
- pub `PromoteResult` struct L75-84 — `{ path: PathBuf, bytes: u64, renamed: bool, original_destination: PathBuf }` — Result of a file promotion operation.
- pub `ExportResult` struct L88-93 — `{ path: PathBuf, bytes: u64 }` — Result of a file export operation.
- pub `CloneResult` struct L97-102 — `{ path: PathBuf, commit: String }` — Result of a git clone operation.
- pub `AttachResult` struct L106-113 — `{ files_migrated: usize, new_work_path: PathBuf, allowed_paths: Vec<PathBuf> }` — Result of attaching a scratch session to a named workstream.
- pub `SessionUsage` struct L117-122 — `{ id: String, bytes: u64 }` — Usage statistics for a single session.
- pub `ManualCleanupResult` struct L126-135 — `{ deleted_files: usize, freed_bytes: u64, pending_files: usize, requires_confirm...` — Result of a manual cleanup operation.
- pub `freed_mb` function L139-141 — `(&self) -> f64` — Convert freed bytes to megabytes.
- pub `UsageStats` struct L146-157 — `{ production_bytes: u64, work_bytes: u64, sessions: Vec<SessionUsage>, total_byt...` — Disk usage statistics for a workstream.
- pub `production_mb` function L161-163 — `(&self) -> f64` — Convert production bytes to megabytes.
- pub `work_mb` function L166-168 — `(&self) -> f64` — Convert work bytes to megabytes.
- pub `total_mb` function L171-173 — `(&self) -> f64` — Convert total bytes to megabytes.
- pub `SCRATCH_WORKSTREAM` variable L177 — `: &str` — Well-known scratch workstream ID (matches crate::scratch::SCRATCH_ID).
-  `ManualCleanupResult` type L137-142 — `= ManualCleanupResult` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |
-  `UsageStats` type L159-174 — `= UsageStats` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |
-  `WORKSTREAMS_DIR` variable L180 — `: &str` — Subdirectory name for workstreams.
-  `PRODUCTION_DIR` variable L183 — `: &str` — Subdirectory for production artifacts.
-  `WORK_DIR` variable L186 — `: &str` — Subdirectory for work-in-progress files.
-  `SESSIONS_DIR` variable L189 — `: &str` — Subdirectory for scratch sessions.
-  `clone` module L191 — `-` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |
-  `manager` module L192 — `-` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |
-  `operations` module L193 — `-` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |
-  `session` module L194 — `-` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |
-  `usage` module L195 — `-` — | my-blog | any | `my-blog/production/`, `my-blog/work/` |

#### crates/arawn-workstream/src/directory/operations.rs

- pub `promote` function L48-113 — `( &self, workstream: &str, source: &Path, destination: &Path, ) -> DirectoryResu...` — Promotes a file from `work/` to `production/`.
- pub `export` function L190-251 — `( &self, workstream: &str, source: &Path, destination: &Path, ) -> DirectoryResu...` — Exports a file from `production/` to an external path.
-  `DirectoryManager` type L8-252 — `= DirectoryManager` — File operations: promote and export.
-  `resolve_conflict` function L119-145 — `(path: &Path) -> PathBuf` — Resolves a filename conflict by appending a suffix.
-  `tests` module L255-482 — `-` — File operations: promote and export.
-  `setup` function L262-266 — `() -> (tempfile::TempDir, DirectoryManager)` — File operations: promote and export.
-  `test_promote_basic` function L268-300 — `()` — File operations: promote and export.
-  `test_promote_to_subdirectory` function L303-325 — `()` — File operations: promote and export.
-  `test_promote_with_conflict` function L328-357 — `()` — File operations: promote and export.
-  `test_promote_with_multiple_conflicts` function L360-382 — `()` — File operations: promote and export.
-  `test_promote_file_without_extension` function L385-402 — `()` — File operations: promote and export.
-  `test_promote_source_not_found` function L405-419 — `()` — File operations: promote and export.
-  `test_promote_source_is_directory` function L422-436 — `()` — File operations: promote and export.
-  `test_promote_workstream_not_found` function L439-447 — `()` — File operations: promote and export.
-  `test_promote_invalid_workstream_name` function L450-458 — `()` — File operations: promote and export.
-  `test_resolve_conflict_basic` function L461-468 — `()` — File operations: promote and export.
-  `test_resolve_conflict_finds_gap` function L471-481 — `()` — File operations: promote and export.

#### crates/arawn-workstream/src/directory/session.rs

- pub `attach_session` function L41-132 — `( &self, session_id: &str, target_workstream: &str, ) -> DirectoryResult<AttachR...` — Attaches a scratch session to a named workstream by migrating its files.
-  `DirectoryManager` type L8-149 — `= DirectoryManager` — Session attachment: migrating scratch sessions to named workstreams.
-  `copy_dir_recursive` function L135-148 — `(src: &Path, dest: &Path) -> DirectoryResult<()>` — Recursively copy a directory.
-  `tests` module L152-297 — `-` — Session attachment: migrating scratch sessions to named workstreams.
-  `setup` function L158-162 — `() -> (tempfile::TempDir, DirectoryManager)` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_basic` function L164-193 — `()` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_with_subdirectories` function L196-217 — `()` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_no_files` function L220-234 — `()` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_invalid_session_id` function L237-244 — `()` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_invalid_workstream_name` function L247-257 — `()` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_workstream_not_found` function L260-270 — `()` — Session attachment: migrating scratch sessions to named workstreams.
-  `test_attach_session_preserves_content` function L273-296 — `()` — Session attachment: migrating scratch sessions to named workstreams.

#### crates/arawn-workstream/src/directory/usage.rs

- pub `get_usage` function L52-119 — `(&self, workstream: &str) -> DirectoryResult<UsageStats>` — Calculate disk usage statistics for a workstream.
- pub `cleanup_work` function L232-373 — `( &self, workstream: &str, older_than_days: Option<u32>, confirmed: bool, ) -> D...` — Clean up files in the work directory.
-  `DirectoryManager` type L11-396 — `= DirectoryManager` — Usage statistics and cleanup operations.
-  `WORK_WARNING_THRESHOLD` variable L15 — `: u64` — Default warning threshold for work directory (500MB).
-  `PRODUCTION_WARNING_THRESHOLD` variable L17 — `: u64` — Default warning threshold for production directory (1GB).
-  `SESSION_WARNING_THRESHOLD` variable L19 — `: u64` — Default warning threshold for session work directory (100MB).
-  `get_session_usages` function L124-157 — `( &self, sessions_path: &Path, ) -> DirectoryResult<(u64, Vec<SessionUsage>)>` — Calculate disk usage for all sessions in a directory.
-  `dir_size` function L160-178 — `(path: &Path) -> DirectoryResult<u64>` — Calculate the total size of a directory recursively.
-  `CLEANUP_CONFIRMATION_THRESHOLD` variable L183 — `: usize` — Threshold for requiring confirmation (>100 files).
-  `remove_empty_dirs` function L376-395 — `(path: &Path)` — Remove empty directories recursively (bottom-up).
-  `tests` module L399-730 — `-` — Usage statistics and cleanup operations.
-  `setup` function L405-409 — `() -> (tempfile::TempDir, DirectoryManager)` — Usage statistics and cleanup operations.
-  `test_get_usage_basic` function L411-438 — `()` — Usage statistics and cleanup operations.
-  `test_get_usage_scratch_with_sessions` function L441-470 — `()` — Usage statistics and cleanup operations.
-  `test_get_usage_empty_workstream` function L473-486 — `()` — Usage statistics and cleanup operations.
-  `test_get_usage_nonexistent_workstream` function L489-494 — `()` — Usage statistics and cleanup operations.
-  `test_get_usage_invalid_name` function L497-502 — `()` — Usage statistics and cleanup operations.
-  `test_get_usage_nested_directories` function L505-523 — `()` — Usage statistics and cleanup operations.
-  `test_usage_stats_mb_conversions` function L526-539 — `()` — Usage statistics and cleanup operations.
-  `test_dir_size_nonexistent` function L542-546 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_basic` function L551-571 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_with_age_filter` function L574-588 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_requires_confirmation` function L591-618 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_nested_directories` function L621-641 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_scratch_sessions` function L644-664 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_preserves_production` function L667-684 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_empty_workstream` function L687-697 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_workstream_not_found` function L700-707 — `()` — Usage statistics and cleanup operations.
-  `test_cleanup_work_invalid_name` function L710-717 — `()` — Usage statistics and cleanup operations.
-  `test_manual_cleanup_result_freed_mb` function L720-729 — `()` — Usage statistics and cleanup operations.

### crates/gline-rs-vendored/src

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/lib.rs

- pub `model` module L10 — `-` — zero-shot [Named Entity Recognition](https://paperswithcode.com/task/cg) (NER) and many other tasks such
- pub `text` module L11 — `-` — as well as a clean and maintainable implementation of the mechanics surrounding the model itself.
- pub `util` module L12 — `-` — as well as a clean and maintainable implementation of the mechanics surrounding the model itself.

### crates/gline-rs-vendored/src/model/input

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/input/encoded.rs

- pub `EncodedInput` struct L8-18 — `{ texts: Vec<String>, tokens: Vec<Vec<Token>>, entities: Vec<String>, num_words:...` — Represents encoded prompts (after sub-word tokenization)
- pub `from` function L32-136 — `(input: PromptInput, tokenizer: &impl Tokenizer) -> Result<Self>`
- pub `PromptsToEncoded` struct L140-142 — `{ tokenizer: &'a T }` — Composable: Prompts => Encoded
- pub `new` function L145-147 — `(tokenizer: &'a T) -> Self`
-  `EncodedPrompt` struct L21-26 — `{ encoding: Vec<Vec<u32>>, text_offset: usize }` — Utility struct
-  `EncodedInput` type L28-137 — `= EncodedInput`
-  `apply` function L151-153 — `(&self, input: PromptInput) -> Result<EncodedInput>`
-  `tests` module L158-346 — `-` — Unit tests
-  `test` function L162-204 — `() -> Result<()>`
-  `ENT_ID` variable L184 — `: i64`
-  `SEP_ID` variable L185 — `: i64`
-  `test2` function L207-294 — `() -> Result<()>`
-  `test_multiword_entity_label` function L297-322 — `() -> Result<()>`
-  `test_words_mask_multi_token_first_word` function L325-345 — `() -> Result<()>`

#### crates/gline-rs-vendored/src/model/input/mod.rs

- pub `encoded` module L14 — `-` — For NER, they will normally be applied in that order:
- pub `prompt` module L15 — `-` — - ...
- pub `relation` module L16 — `-` — - ...
- pub `tensors` module L17 — `-` — - ...
- pub `text` module L18 — `-` — - ...
- pub `tokenized` module L19 — `-` — - ...

#### crates/gline-rs-vendored/src/model/input/prompt.rs

- pub `PromptInput` struct L12-25 — `{ texts: Vec<String>, tokens: Vec<Vec<Token>>, entities: Vec<String>, text_lengt...` — Prepared prompts, appending entity and text tokens.
- pub `from` function L28-61 — `(input: TokenizedInput) -> Self`
- pub `TokenizedToPrompt` struct L81 — `-` — Composable: Tokenized => Prompt
-  `PromptInput` type L27-77 — `= PromptInput`
-  `entities_prompt` function L64-76 — `(entities: &Vec<String>) -> Vec<String>` — Create the entities part of the prompt.
-  `ENTITY_TOKEN` variable L65 — `: &str`
-  `SEP_TOKEN` variable L66 — `: &str`
-  `TokenizedToPrompt` type L83-87 — `= TokenizedToPrompt`
-  `apply` function L84-86 — `(&self, input: TokenizedInput) -> Result<PromptInput>`
-  `tests` module L91-126 — `-` — Unit tests
-  `test` function L95-125 — `() -> Result<()>`

#### crates/gline-rs-vendored/src/model/input/text.rs

- pub `TextInput` struct L6-9 — `{ texts: Vec<String>, entities: Vec<String> }` — Represents the raw text input, as a list of text chunks and a list of entity classes
- pub `new` function L14-20 — `(texts: Vec<String>, entities: Vec<String>) -> Result<Self>` — Default constructor that moves the input data given as a vector of the text
- pub `from_str` function L23-28 — `(texts: &[&str], entities: &[&str]) -> Result<Self>` — This constructor will mostly be used to test with plain arrays of static `str`s.
- pub `new_from_csv` function L32-45 — `( path: P, column: usize, limit: usize, entities: Vec<String>, ) -> Result<Self>` — For testing purposes.
-  `TextInput` type L11-46 — `= TextInput`

#### crates/gline-rs-vendored/src/model/input/tokenized.rs

- pub `TokenizedInput` struct L8-15 — `{ tokens: Vec<Vec<Token>>, texts: Vec<String>, entities: Vec<String> }` — Represents the output of the word-level segmentation
- pub `from` function L18-34 — `( input: TextInput, splitter: &impl Splitter, max_length: Option<usize>, ) -> Re...`
- pub `RawToTokenized` struct L38-41 — `{ splitter: &'a S, max_length: Option<usize> }` — Composable: Text => Tokenized
- pub `new` function L44-49 — `(splitter: &'a S, max_length: Option<usize>) -> Self`
-  `TokenizedInput` type L17-35 — `= TokenizedInput`
-  `apply` function L53-55 — `(&self, input: TextInput) -> Result<TokenizedInput>`
-  `tests` module L60-98 — `-` — Unit tests
-  `test` function L64-97 — `() -> Result<()>`

### crates/gline-rs-vendored/src/model/input/relation

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/input/relation/mod.rs

- pub `schema` module L1 — `-`
- pub `RelationInput` struct L13-17 — `{ prompts: Vec<String>, labels: Vec<String>, entity_labels: HashMap<String, Hash...` — Input data for Relation Extraction
- pub `from_spans` function L21-27 — `(spans: SpanOutput, schema: &RelationSchema) -> Self` — Builds a relation input from a span output and a relation schema
- pub `SpanOutputToRelationInput` struct L85-87 — `{ schema: &'a RelationSchema }`
- pub `new` function L90-92 — `(schema: &'a RelationSchema) -> Self`
- pub `RelationInputToTextInput` struct L102 — `-`
-  `PROMPT_PREFIX` variable L10 — `: &str`
-  `RelationInput` type L19-83 — `= RelationInput`
-  `make_prompts` function L30-36 — `(spans: &SpanOutput, prefix: &str) -> Vec<String>` — Prepare the prompts basing on the provided prefix
-  `make_labels` function L39-63 — `(spans: &SpanOutput, schema: &RelationSchema) -> Vec<String>` — Prepare the labels basing on extracted entities and the provided schema
-  `make_entity_labels` function L71-82 — `(spans: &SpanOutput) -> HashMap<String, HashSet<String>>` — Build entity-text -> entity-labels map (which will be used when decoding, to filter relations basing on allowed objects).
-  `apply` function L96-98 — `(&self, input: SpanOutput) -> Result<RelationInput>`
-  `RelationInputToTextInput` type L104-115 — `= RelationInputToTextInput`
-  `apply` function L107-114 — `(&self, input: RelationInput) -> Result<(super::text::TextInput, RelationContext...`

#### crates/gline-rs-vendored/src/model/input/relation/schema.rs

- pub `RelationSchema` struct L3-5 — `{ relations: HashMap<String, RelationSpec> }`
- pub `new` function L8-12 — `() -> Self`
- pub `from_str` function L14-21 — `(relations: &[&str]) -> Self`
- pub `push` function L23-26 — `(&mut self, relation: &str)`
- pub `push_with_allowed_labels` function L28-38 — `( &mut self, relation: &str, allowed_subjects: &[&str], allowed_objects: &[&str]...`
- pub `push_with_spec` function L40-42 — `(&mut self, relation: &str, spec: RelationSpec)`
- pub `relations` function L44-46 — `(&self) -> &HashMap<String, RelationSpec>`
- pub `RelationSpec` struct L55-58 — `{ allowed_subjects: Option<HashSet<String>>, allowed_objects: Option<HashSet<Str...`
- pub `new` function L61-66 — `(allowed_subjects: &[&str], allowed_objects: &[&str]) -> Self`
- pub `allows_subject` function L68-73 — `(&self, label: &str) -> bool`
- pub `allows_object` function L75-80 — `(&self, label: &str) -> bool`
- pub `allows_one_of_subjects` function L82-87 — `(&self, labels: &HashSet<String>) -> bool`
- pub `allows_one_of_objects` function L89-94 — `(&self, labels: &HashSet<String>) -> bool`
-  `RelationSchema` type L7-47 — `= RelationSchema`
-  `RelationSchema` type L49-53 — `impl Default for RelationSchema`
-  `default` function L50-52 — `() -> Self`
-  `RelationSpec` type L60-95 — `= RelationSpec`
-  `RelationSpec` type L97-104 — `impl Default for RelationSpec`
-  `default` function L98-103 — `() -> Self`

### crates/gline-rs-vendored/src/model/input/tensors

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/input/tensors/mod.rs

- pub `span` module L1 — `-`
- pub `token` module L2 — `-`

#### crates/gline-rs-vendored/src/model/input/tensors/span.rs

- pub `SpanTensors` struct L16-19 — `{ tensors: SessionInputs<'a, 'a>, context: EntityContext }` — Ready-for-inference tensors (span mode)
- pub `from` function L22-41 — `(encoded: EncodedInput, max_width: usize) -> Result<Self>`
- pub `inputs` function L43-52 — `() -> [&'static str; 6]`
- pub `EncodedToTensors` struct L116-118 — `{ max_width: usize }` — Composable: Encoded => SpanTensors
- pub `new` function L121-123 — `(max_width: usize) -> Self`
- pub `TensorsToSessionInput` struct L134 — `-` — Composable: SpanTensors => (SessionInput, EntityContext)
-  `TENSOR_INPUT_IDS` variable L8 — `: &str`
-  `TENSOR_ATTENTION_MASK` variable L9 — `: &str`
-  `TENSOR_WORD_MASK` variable L10 — `: &str`
-  `TENSOR_TEXT_LENGTHS` variable L11 — `: &str`
-  `TENSOR_SPAN_IDX` variable L12 — `: &str`
-  `TENSOR_SPAN_MASK` variable L13 — `: &str`
-  `make_spans_tensors` function L76-112 — `( encoded: &EncodedInput, max_width: usize, ) -> (ndarray::Array3<i64>, ndarray:...` — Expected tensor for num_words=4 and max_width=12:
-  `EncodedToTensors` type L120-124 — `= EncodedToTensors`
-  `EncodedToTensors` type L126-130 — `= EncodedToTensors`
-  `apply` function L127-129 — `(&self, input: EncodedInput) -> Result<SpanTensors<'a>>`
-  `TensorsToSessionInput` type L136-142 — `= TensorsToSessionInput`
-  `apply` function L139-141 — `(&self, input: SpanTensors<'a>) -> Result<(SessionInputs<'a, 'a>, EntityContext)...`
-  `tests` module L146-197 — `-` — Unit tests
-  `test` function L151-182 — `() -> Result<()>`
-  `get_tensor` function L184-196 — `( key: &str, si: &'a SessionInputs<'a, 'a>, ) -> Result<&'a SessionInputValue<'a...`

#### crates/gline-rs-vendored/src/model/input/tensors/token.rs

- pub `TokenTensors` struct L14-17 — `{ tensors: SessionInputs<'a, 'a>, context: EntityContext }` — Ready-for-inference tensors (token mode)
- pub `from` function L20-36 — `(encoded: EncodedInput) -> Result<Self>`
- pub `inputs` function L38-45 — `() -> [&'static str; 4]`
- pub `EncodedToTensors` struct L50 — `-` — Composable: Encoded => TokenTensors
- pub `TensorsToSessionInput` struct L60 — `-` — Composable: TokenTensors => (SessionInput, TensorsMeta)
-  `TENSOR_INPUT_IDS` variable L8 — `: &str`
-  `TENSOR_ATTENTION_MASK` variable L9 — `: &str`
-  `TENSOR_WORD_MASK` variable L10 — `: &str`
-  `TENSOR_TEXT_LENGTHS` variable L11 — `: &str`
-  `EncodedToTensors` type L52-56 — `= EncodedToTensors`
-  `apply` function L53-55 — `(&self, input: EncodedInput) -> Result<TokenTensors<'a>>`
-  `TensorsToSessionInput` type L62-68 — `= TensorsToSessionInput`
-  `apply` function L65-67 — `(&self, input: TokenTensors<'a>) -> Result<(SessionInputs<'a, 'a>, EntityContext...`

### crates/gline-rs-vendored/src/model

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/mod.rs

- pub `input` module L3 — `-` — The core of `gline-rs`: everything about pre-/post-processing, and inferencing
- pub `output` module L4 — `-` — The core of `gline-rs`: everything about pre-/post-processing, and inferencing
- pub `params` module L5 — `-` — The core of `gline-rs`: everything about pre-/post-processing, and inferencing
- pub `pipeline` module L6 — `-` — The core of `gline-rs`: everything about pre-/post-processing, and inferencing
- pub `GLiNER` struct L16-20 — `{ params: Parameters, model: Model, pipeline: P }` — Basic GLiNER, to be parametrized by a specific pipeline (see implementations within the pipeline module)
- pub `inference` function L23-25 — `(&'a mut self, input: P::Input) -> Result<P::Output>` — The core of `gline-rs`: everything about pre-/post-processing, and inferencing

#### crates/gline-rs-vendored/src/model/params.rs

- pub `Parameters` struct L10-23 — `{ threshold: f32, flat_ner: bool, dup_label: bool, multi_label: bool, max_width:...` — Represents the set of parameters for the whole pipeline
- pub `new` function L34-50 — `( threshold: f32, max_width: usize, max_length: Option<usize>, flat_ner: bool, d...` — New configuration specifying every parameter
- pub `with_threshold` function L52-55 — `(mut self, threshold: f32) -> Self` — Processing parameters
- pub `with_max_width` function L57-60 — `(mut self, max_width: usize) -> Self` — Processing parameters
- pub `with_max_length` function L62-65 — `(mut self, max_length: Option<usize>) -> Self` — Processing parameters
- pub `with_flat_ner` function L67-70 — `(mut self, flat_ner: bool) -> Self` — Processing parameters
- pub `with_dup_label` function L72-75 — `(mut self, dup_label: bool) -> Self` — Processing parameters
- pub `with_multi_label` function L77-80 — `(mut self, multi_label: bool) -> Self` — Processing parameters
-  `Parameters` type L25-30 — `impl Default for Parameters` — Processing parameters
-  `default` function L27-29 — `() -> Self` — Default configuration, which can be safely used in most cases
-  `Parameters` type L32-81 — `= Parameters` — Processing parameters

### crates/gline-rs-vendored/src/model/output/decoded

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/output/decoded/greedy.rs

- pub `GreedySearch` struct L11-15 — `{ flat_ner: bool, dup_label: bool, multi_label: bool }` — Greedy decoding implementation.
- pub `new` function L23-29 — `(flat_ner: bool, dup_label: bool, multi_label: bool) -> Self` — Creates a new greedy-search performer
- pub `search` function L34-59 — `(&self, spans: &[Span]) -> Vec<Span>` — Perform greedy search
-  `GreedySearch` type L17-87 — `= GreedySearch` — Greedy-search is the second step of span decoding
-  `accept` function L69-86 — `(&self, s1: &Span, s2: &Span) -> bool` — Returns `true` iif the span is valid wrt.
-  `GreedySearch` type L90-95 — `= GreedySearch` — Composable: SpanOutput => SpanOutput
-  `apply` function L91-94 — `(&self, input: SpanOutput) -> Result<SpanOutput>` — Greedy-search is the second step of span decoding

#### crates/gline-rs-vendored/src/model/output/decoded/mod.rs

- pub `greedy` module L3 — `-` — Span decoding steps
- pub `sort` module L4 — `-` — Span decoding steps
- pub `span` module L5 — `-` — Span decoding steps
- pub `token` module L6 — `-` — Span decoding steps
- pub `token_flat` module L7 — `-` — Span decoding steps
- pub `SpanOutput` struct L13-17 — `{ texts: Vec<String>, entities: Vec<String>, spans: Vec<Vec<Span>> }` — Represents the final output of the post-processing steps, as a list of spans for each input sequence
- pub `new` function L20-26 — `(texts: Vec<String>, entities: Vec<String>, spans: Vec<Vec<Span>>) -> Self` — Span decoding steps
-  `SpanOutput` type L19-27 — `= SpanOutput` — Span decoding steps
-  `SpanOutput` type L29-45 — `= SpanOutput` — Span decoding steps
-  `fmt` function L30-44 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — Span decoding steps

#### crates/gline-rs-vendored/src/model/output/decoded/sort.rs

- pub `SpanSort` struct L8 — `-` — Sort spans by offsets (which is expected by greedy-search)
-  `SpanSort` type L11-22 — `= SpanSort` — Composable: SpanOutput => SpanOutput
-  `apply` function L12-21 — `(&self, input: SpanOutput) -> Result<SpanOutput>` — Sort spans by offsets (which is expected by greedy-search)

#### crates/gline-rs-vendored/src/model/output/decoded/span.rs

- pub `TensorsToDecoded` struct L17-20 — `{ threshold: f32, max_width: usize }` — Decoding method for span mode.
- pub `new` function L23-28 — `(threshold: f32, max_width: usize) -> Self` — First step of span decoding (in span mode)
- pub `outputs` function L30-32 — `() -> [&'static str; 1]` — First step of span decoding (in span mode)
-  `TENSOR_LOGITS` variable L11 — `: &str` — First step of span decoding (in span mode)
-  `TensorsToDecoded` type L22-102 — `= TensorsToDecoded` — First step of span decoding (in span mode)
-  `decode` function L34-85 — `(&self, input: &TensorOutput) -> Result<Vec<Vec<Span>>>` — First step of span decoding (in span mode)
-  `check_shape` function L89-101 — `(&self, actual_shape: Vec<i64>, context: &EntityContext) -> Result<()>` — Checks coherence of the output shape
-  `TensorsToDecoded` type L104-113 — `= TensorsToDecoded` — First step of span decoding (in span mode)
-  `apply` function L105-112 — `(&self, input: TensorOutput) -> Result<SpanOutput>` — First step of span decoding (in span mode)

#### crates/gline-rs-vendored/src/model/output/decoded/token.rs

- pub `TensorsToDecoded` struct L23-25 — `{ threshold: f32 }` — Decoding method for token mode.
- pub `new` function L28-30 — `(threshold: f32) -> Self` — First step of span decoding (in token mode)
- pub `outputs` function L32-34 — `() -> [&'static str; 1]` — First step of span decoding (in token mode)
-  `TENSOR_LOGITS` variable L11 — `: &str` — First step of span decoding (in token mode)
-  `TensorsToDecoded` type L27-151 — `= TensorsToDecoded` — First step of span decoding (in token mode)
-  `decode` function L36-81 — `(&self, input: &TensorOutput) -> Result<Vec<Vec<Span>>>` — First step of span decoding (in token mode)
-  `generate_spans` function L88-112 — `( &self, scores_start: &ndarray::ArrayView2<f32>, scores_end: &ndarray::ArrayVie...` — Generates all possible `(i,j,c)` spans where:
-  `compute_span_score` function L117-133 — `( &self, span: (usize, usize, usize), scores_inside: &ndarray::ArrayView2<f32>, ...` — Computes the score of a span, defined as the mean of the inside scores (see above).
-  `check_shape` function L138-150 — `(&self, actual_shape: Vec<i64>, context: &EntityContext) -> Result<()>` — Checks coherence of the output shape.
-  `TensorsToDecoded` type L153-162 — `= TensorsToDecoded` — First step of span decoding (in token mode)
-  `apply` function L154-161 — `(&self, input: TensorOutput) -> Result<SpanOutput>` — First step of span decoding (in token mode)

#### crates/gline-rs-vendored/src/model/output/decoded/token_flat.rs

- pub `FlatTokenDecoder` struct L16-18 — `{ threshold: f32 }` — *Experimental* token decoding with a one-dimensional approach, working directly on a flat representation of
- pub `TensorsToDecoded` struct L102-104 — `{ decoder: FlatTokenDecoder }` — Experimental alternative for the first step of span decoding (in token mode)
- pub `new` function L107-111 — `(threshold: f32) -> Self` — Experimental alternative for the first step of span decoding (in token mode)
-  `FlatTokenDecoder` type L20-100 — `= FlatTokenDecoder` — Experimental alternative for the first step of span decoding (in token mode)
-  `new` function L21-23 — `(threshold: f32) -> Self` — Experimental alternative for the first step of span decoding (in token mode)
-  `decode` function L25-94 — `(&self, model_output: &[f32], input: &EntityContext) -> Result<Vec<Vec<Span>>>` — Experimental alternative for the first step of span decoding (in token mode)
-  `get` function L97-99 — `(model_output: &[f32], index: usize) -> f32` — Experimental alternative for the first step of span decoding (in token mode)
-  `TensorsToDecoded` type L106-112 — `= TensorsToDecoded` — Experimental alternative for the first step of span decoding (in token mode)
-  `TensorsToDecoded` type L114-128 — `= TensorsToDecoded` — Experimental alternative for the first step of span decoding (in token mode)
-  `apply` function L115-127 — `(&self, input: TensorOutput) -> Result<SpanOutput>` — Experimental alternative for the first step of span decoding (in token mode)

### crates/gline-rs-vendored/src/model/output

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/output/mod.rs

- pub `decoded` module L3 — `-` — Post-processing steps
- pub `relation` module L4 — `-` — Post-processing steps
- pub `tensors` module L5 — `-` — Post-processing steps

#### crates/gline-rs-vendored/src/model/output/relation.rs

- pub `RelationOutput` struct L9-13 — `{ texts: Vec<String>, entities: Vec<String>, relations: Vec<Vec<Relation>> }` — Defines the final output of the relation extraction pipeline
- pub `Relation` struct L16-31 — `{ class: String, subject: String, object: String, sequence: usize, start: usize,...` — Defines an individual relation
- pub `from` function L34-46 — `(span: Span) -> Result<Self>`
- pub `class` function L48-50 — `(&self) -> &str`
- pub `subject` function L52-54 — `(&self) -> &str`
- pub `object` function L56-58 — `(&self) -> &str`
- pub `sequence` function L60-62 — `(&self) -> usize`
- pub `offsets` function L64-66 — `(&self) -> (usize, usize)`
- pub `probability` function L68-70 — `(&self) -> f32`
- pub `SpanOutputToRelationOutput` struct L105-107 — `{ schema: &'a RelationSchema }` — SpanOutput -> RelationOutput
- pub `new` function L110-112 — `(schema: &'a RelationSchema) -> Self`
- pub `RelationFormatError` struct L157-159 — `{ message: String }` — Defines an error caused by an malformed or unexpected span label
- pub `invalid_relation_label` function L162-166 — `(label: &str) -> Self`
- pub `unexpected_relation_label` function L168-172 — `(label: &str) -> Self`
- pub `err` function L174-176 — `(self) -> Result<T>`
-  `Relation` type L33-83 — `= Relation`
-  `decode` function L72-82 — `(rel_class: &str) -> Result<(String, String)>`
-  `RelationOutput` type L85-102 — `= RelationOutput`
-  `fmt` function L86-101 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`
-  `is_valid` function L114-128 — `(&self, relation: &Relation, context: &RelationContext) -> Result<bool>`
-  `apply` function L132-150 — `(&self, input: (SpanOutput, RelationContext)) -> Result<RelationOutput>`
-  `RelationFormatError` type L161-177 — `= RelationFormatError`
-  `RelationFormatError` type L179 — `= RelationFormatError`
-  `RelationFormatError` type L181-185 — `= RelationFormatError`
-  `fmt` function L182-184 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

#### crates/gline-rs-vendored/src/model/output/tensors.rs

- pub `TensorOutput` struct L9-12 — `{ context: EntityContext, tensors: SessionOutputs<'a> }` — Represents the raw tensor output of the inference step
- pub `from` function L15-17 — `(tensors: SessionOutputs<'a>, context: EntityContext) -> Self` — Encapsulation of raw tensor outputs
- pub `SessionOutputToTensors` struct L22 — `-` — Composable: (SessionOutput, TensorMeta) => TensorOutput
-  `SessionOutputToTensors` type L24-30 — `= SessionOutputToTensors` — Encapsulation of raw tensor outputs
-  `apply` function L27-29 — `(&self, input: (SessionOutputs<'a>, EntityContext)) -> Result<TensorOutput<'a>>` — Encapsulation of raw tensor outputs

### crates/gline-rs-vendored/src/model/pipeline

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/model/pipeline/context.rs

- pub `EntityContext` struct L9-14 — `{ texts: Vec<String>, tokens: Vec<Vec<Token>>, entities: Vec<String>, num_words:...` — Data to be transmitted, beside the tensors themselves, from pre-processing to post-processing.
- pub `create_span` function L18-56 — `( &self, sequence_id: usize, start_token: usize, end_token: usize, class: usize,...` — Creates a span given the necessary indexes and the tensor meta data.
- pub `RelationContext` struct L60-62 — `{ entity_labels: HashMap<String, HashSet<String>> }` — Data to be transmitted, beside the tensors themselves, from pre-processing to post-processing.
-  `EntityContext` type L16-57 — `= EntityContext` — Data to be transmitted, beside the tensors themselves, from pre-processing to post-processing.

#### crates/gline-rs-vendored/src/model/pipeline/mod.rs

- pub `context` module L3 — `-` — Defines the `Pipeline` trait and its implementations
- pub `relation` module L4 — `-` — Defines the `Pipeline` trait and its implementations
- pub `span` module L5 — `-` — Defines the `Pipeline` trait and its implementations
- pub `token` module L6 — `-` — Defines the `Pipeline` trait and its implementations

#### crates/gline-rs-vendored/src/model/pipeline/relation.rs

- pub `RelationPipeline` struct L18-21 — `{ token_pipeline: TokenPipeline<S, T>, relation_schema: &'a RelationSchema }` — Relation Extraction pipeline
- pub `new` function L54-59 — `(token_pipeline: TokenPipeline<S, T>, relation_schema: &'a RelationSchema) -> Se...` — Pre-defined pipeline for Relation Extraction
- pub `default` function L66-74 — `( tokenizer_path: P, relation_schema: &'a RelationSchema, ) -> Result<Self>` — Pre-defined pipeline for Relation Extraction
-  `Input` type L24 — `= SpanOutput` — Pre-defined pipeline for Relation Extraction
-  `Output` type L25 — `= RelationOutput` — Pre-defined pipeline for Relation Extraction
-  `Context` type L26 — `= (RelationContext, EntityContext)` — Pre-defined pipeline for Relation Extraction
-  `Parameters` type L27 — `= Parameters` — Pre-defined pipeline for Relation Extraction
-  `pre_processor` function L29-40 — `( &self, params: &Parameters, ) -> impl PreProcessor<'a, Self::Input, Self::Cont...` — Pre-defined pipeline for Relation Extraction
-  `post_processor` function L42-50 — `( &self, params: &Parameters, ) -> impl PostProcessor<'a, Self::Output, Self::Co...` — Pre-defined pipeline for Relation Extraction

#### crates/gline-rs-vendored/src/model/pipeline/span.rs

- pub `SpanPipeline` struct L13-18 — `{ splitter: S, tokenizer: T, expected_inputs: HashSet<&'static str>, expected_ou...` — Generic span-level pipeline
- pub `new` function L66-77 — `(tokenizer_path: P) -> Result<Self>` — Pre-defined pipeline for NER (span mode)
- pub `new_from_bytes` function L79-90 — `(tokenizer_bytes: &[u8]) -> Result<Self>` — Pre-defined pipeline for NER (span mode)
- pub `SpanMode` type L94-95 — `= SpanPipeline<crate::text::splitter::RegexSplitter, crate::text::tokenizer::HFT...` — Shorthand for the default span pipeline type (eases disambiguation when calling `GLiNER::new`)
- pub `new` function L99-110 — `( params: params::Parameters, runtime_params: RuntimeParameters, tokenizer_path:...` — Pre-defined pipeline for NER (span mode)
- pub `new_from_bytes` function L112-123 — `( params: params::Parameters, runtime_params: RuntimeParameters, tokenizer_bytes...` — Pre-defined pipeline for NER (span mode)
-  `Input` type L21 — `= input::text::TextInput` — Pre-defined pipeline for NER (span mode)
-  `Output` type L22 — `= output::decoded::SpanOutput` — Pre-defined pipeline for NER (span mode)
-  `Context` type L23 — `= EntityContext` — Pre-defined pipeline for NER (span mode)
-  `Parameters` type L24 — `= params::Parameters` — Pre-defined pipeline for NER (span mode)
-  `pre_processor` function L26-37 — `( &self, params: &Self::Parameters, ) -> impl PreProcessor<'a, Self::Input, Self...` — Pre-defined pipeline for NER (span mode)
-  `post_processor` function L39-53 — `( &self, params: &Self::Parameters, ) -> impl PostProcessor<'a, Self::Output, Se...` — Pre-defined pipeline for NER (span mode)
-  `expected_inputs` function L55-57 — `(&self, _params: &Self::Parameters) -> Option<impl Iterator<Item = &str>>` — Pre-defined pipeline for NER (span mode)
-  `expected_outputs` function L59-61 — `(&self, _params: &Self::Parameters) -> Option<impl Iterator<Item = &str>>` — Pre-defined pipeline for NER (span mode)

#### crates/gline-rs-vendored/src/model/pipeline/token.rs

- pub `TokenPipeline` struct L13-18 — `{ splitter: S, tokenizer: T, expected_inputs: HashSet<&'static str>, expected_ou...` — Generic token-level pipeline
- pub `new` function L66-77 — `(tokenizer_path: P) -> Result<Self>` — Pre-defined pipeline for NER (token mode)
- pub `TokenMode` type L81-82 — `= TokenPipeline<crate::text::splitter::RegexSplitter, crate::text::tokenizer::HF...` — Shorthand for the default token pipeline type (eases disambiguation when calling `GLiNER::new`)
- pub `new` function L86-97 — `( params: params::Parameters, runtime_params: RuntimeParameters, tokenizer_path:...` — Pre-defined pipeline for NER (token mode)
-  `Input` type L21 — `= input::text::TextInput` — Pre-defined pipeline for NER (token mode)
-  `Output` type L22 — `= output::decoded::SpanOutput` — Pre-defined pipeline for NER (token mode)
-  `Context` type L23 — `= EntityContext` — Pre-defined pipeline for NER (token mode)
-  `Parameters` type L24 — `= params::Parameters` — Pre-defined pipeline for NER (token mode)
-  `pre_processor` function L26-37 — `( &self, params: &Self::Parameters, ) -> impl PreProcessor<'a, Self::Input, Self...` — Pre-defined pipeline for NER (token mode)
-  `post_processor` function L39-53 — `( &self, params: &Self::Parameters, ) -> impl PostProcessor<'a, Self::Output, Se...` — Pre-defined pipeline for NER (token mode)
-  `expected_inputs` function L55-57 — `(&self, _params: &Self::Parameters) -> Option<impl Iterator<Item = &str>>` — Pre-defined pipeline for NER (token mode)
-  `expected_outputs` function L59-61 — `(&self, _params: &Self::Parameters) -> Option<impl Iterator<Item = &str>>` — Pre-defined pipeline for NER (token mode)

### crates/gline-rs-vendored/src/text

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/text/mod.rs

- pub `prompt` module L3 — `-` — Everything that relates to text processing
- pub `span` module L4 — `-` — Everything that relates to text processing
- pub `splitter` module L5 — `-` — Everything that relates to text processing
- pub `token` module L6 — `-` — Everything that relates to text processing
- pub `tokenizer` module L7 — `-` — Everything that relates to text processing

#### crates/gline-rs-vendored/src/text/prompt.rs

- pub `Prompt` struct L3-7 — `{ prompt: Vec<String>, text_length: usize, entities_length: usize }`
- pub `new` function L10-16 — `(prompt: Vec<String>, text_length: usize, entities_length: usize) -> Self`
- pub `tokens` function L19-21 — `(&self) -> &Vec<String>` — The actual prompt tokens
- pub `text_len` function L24-26 — `(&self) -> usize` — Number of tokens in the text part
- pub `entities_len` function L29-31 — `(&self) -> usize` — Number of tokens in the entities part
-  `Prompt` type L9-32 — `= Prompt`

#### crates/gline-rs-vendored/src/text/span.rs

- pub `Span` struct L2-15 — `{ sequence: usize, start: usize, end: usize, text: String, class: String, probab...`
- pub `new` function L18-35 — `( sequence: usize, start: usize, end: usize, text: String, class: String, probab...`
- pub `sequence` function L37-39 — `(&self) -> usize`
- pub `offsets` function L41-43 — `(&self) -> (usize, usize)`
- pub `text` function L45-47 — `(&self) -> &str`
- pub `class` function L49-51 — `(&self) -> &str`
- pub `probability` function L53-55 — `(&self) -> f32`
- pub `is_nested_in` function L58-60 — `(&self, other: &Span) -> bool` — returns `true` iif this span is nested inside (or equals) the given span
- pub `overlaps` function L63-65 — `(&self, other: &Span) -> bool` — returns `true` iif this span overlaps with the given one (symetric)
- pub `is_disjoint` function L68-70 — `(&self, other: &Span) -> bool` — returns `true` iif the spans do not overlap
- pub `same_offsets` function L73-75 — `(&self, other: &Span) -> bool` — returns `true` iif this span has the same offsets as the given one
-  `Span` type L17-76 — `= Span`

#### crates/gline-rs-vendored/src/text/splitter.rs

- pub `Splitter` interface L6-8 — `{ fn split() }` — Word-level tokenization
- pub `RegexSplitter` struct L11-13 — `{ regex: Regex }` — Word-level tokenization implemented using regular expressions
- pub `new` function L16-20 — `(regex: &str) -> Result<Self>`
-  `RegexSplitter` type L15-21 — `= RegexSplitter`
-  `RegexSplitter` type L23-28 — `impl Default for RegexSplitter`
-  `default` function L24-27 — `() -> Self`
-  `DEFAULT_REGEX` variable L25 — `: &str`
-  `RegexSplitter` type L30-43 — `impl Splitter for RegexSplitter`
-  `split` function L31-42 — `(&self, input: &str, limit: Option<usize>) -> Result<Vec<Token>>`
-  `tests` module L46-78 — `-`
-  `test_default_regex_splitter` function L51-60 — `() -> Result<()>`
-  `test_unicode` function L63-68 — `() -> Result<()>`
-  `test_limit` function L71-77 — `() -> Result<()>`

#### crates/gline-rs-vendored/src/text/token.rs

- pub `Token` struct L3-7 — `{ start: usize, end: usize, text: String }`
- pub `new` function L10-16 — `(start: usize, end: usize, text: &str) -> Self`
- pub `start` function L18-20 — `(&self) -> usize`
- pub `end` function L22-24 — `(&self) -> usize`
- pub `text` function L26-28 — `(&self) -> &str`
-  `Token` type L9-29 — `= Token`

#### crates/gline-rs-vendored/src/text/tokenizer.rs

- pub `Tokenizer` interface L5-7 — `{ fn encode() }` — Sub-word tokenization (aka encoding)
- pub `HFTokenizer` struct L10-12 — `{ inner: tokenizers::Tokenizer }` — Implement `Tokenizer` as a wrapper around Hugging Face tokenizers
- pub `from_file` function L15-19 — `(path: P) -> Result<Self>`
- pub `from_pretrained` function L21-25 — `(identifier: &str) -> Result<Self>`
- pub `from_bytes` function L27-31 — `(bytes: &[u8]) -> Result<Self>`
-  `HFTokenizer` type L14-32 — `= HFTokenizer`
-  `HFTokenizer` type L34-39 — `impl Tokenizer for HFTokenizer`
-  `encode` function L35-38 — `(&self, input: &str) -> Result<Vec<u32>>`

### crates/gline-rs-vendored/src/util

> *Semantic summary to be generated by AI agent.*

#### crates/gline-rs-vendored/src/util/error.rs

- pub `IndexError` struct L12-14 — `{ message: String }` — Defines an error caused by the use of an incorrect index in one of the
- pub `new` function L17-21 — `(array_desc: &str, index: usize) -> Self`
- pub `with` function L23-27 — `(message: &str) -> Self`
-  `IndexError` type L16-28 — `= IndexError`
-  `IndexError` type L30 — `= IndexError`
-  `IndexError` type L32-36 — `impl Display for IndexError`
-  `fmt` function L33-35 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

#### crates/gline-rs-vendored/src/util/math.rs

- pub `sigmoid` function L3-5 — `(x: T) -> T`

#### crates/gline-rs-vendored/src/util/memprof.rs

- pub `print_memory_usage` function L9-28 — `()`
-  `ALLOCATOR` variable L7 — `: Cap<System>`

#### crates/gline-rs-vendored/src/util/mod.rs

- pub `error` module L3 — `-` — Various utilities
- pub `math` module L4 — `-` — Various utilities
- pub `result` module L5 — `-` — Various utilities
- pub `memprof` module L8 — `-` — Various utilities

#### crates/gline-rs-vendored/src/util/result.rs

- pub `Result` type L3 — `= core::result::Result<T, Box<dyn Error + Send + Sync>>`
- pub `TryDefault` interface L5-9 — `{ fn default() }`

### crates/orp-vendored/src/bin

> *Semantic summary to be generated by AI agent.*

#### crates/orp-vendored/src/bin/inspect.rs

- pub `main` function L3-9 — `() -> Result<(), Box<dyn std::error::Error + Send + Sync>>` — Inspects an onnx file and prints info about the model and input/output tensors

### crates/orp-vendored/src

> *Semantic summary to be generated by AI agent.*

#### crates/orp-vendored/src/error.rs

- pub `UnexpectedModelSchemaError` struct L6-8 — `{ message: String }` — Defines an error caused by a mismatch between pipeline's expected input
- pub `new_for_input` function L11-15 — `(pipeline: &HashSet<&str>, model: &HashSet<&str>) -> Self`
- pub `new_for_output` function L17-21 — `(pipeline: &HashSet<&str>, model: &HashSet<&str>) -> Self`
- pub `with` function L23-27 — `(message: &str) -> Self`
- pub `into_err` function L29-31 — `(self) -> super::Result<T>`
-  `UnexpectedModelSchemaError` type L10-32 — `= UnexpectedModelSchemaError`
-  `UnexpectedModelSchemaError` type L34 — `= UnexpectedModelSchemaError`
-  `UnexpectedModelSchemaError` type L36-40 — `impl Display for UnexpectedModelSchemaError`
-  `fmt` function L37-39 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

#### crates/orp-vendored/src/lib.rs

- pub `error` module L6 — `-` — Built on top of [`ort`](https://ort.pyke.io), it provides a simple way to handle data pre- and post-processing, chain
- pub `model` module L7 — `-` — multiple ONNX models together, while encouraging code reuse and clarity.
- pub `params` module L8 — `-` — multiple ONNX models together, while encouraging code reuse and clarity.
- pub `pipeline` module L9 — `-` — multiple ONNX models together, while encouraging code reuse and clarity.
- pub `Result` type L11 — `= core::result::Result<T, Box<dyn std::error::Error + Send + Sync>>` — multiple ONNX models together, while encouraging code reuse and clarity.

#### crates/orp-vendored/src/model.rs

- pub `Model` struct L12-14 — `{ session: Session }` — A `Model` can load an ONNX model, and run it using the provided pipeline.
- pub `new` function L17-25 — `(model_path: P, params: RuntimeParameters) -> Result<Self>`
- pub `new_from_bytes` function L27-35 — `(model_bytes: &[u8], params: RuntimeParameters) -> Result<Self>`
- pub `inference` function L38-54 — `( &'a mut self, input: P::Input, pipeline: &P, params: &P::Parameters, ) -> Resu...` — Perform inferences using the provided pipeline and parameters
- pub `to_composable` function L56-62 — `( &'a mut self, pipeline: &'a P, params: &'a P::Parameters, ) -> impl Composable...`
- pub `inspect` function L65-87 — `(&self, mut writer: W) -> Result<()>` — Writes various model properties from metadata and input/output tensors
- pub `new` function L134-140 — `(model: &'a mut Model, pipeline: &'a P, params: &'a P::Parameters) -> Self`
-  `Model` type L16-124 — `= Model`
-  `check_schema` function L90-119 — `( &self, pipeline: &P, params: &P::Parameters, ) -> Result<()>` — Check model schema wrt.
-  `run` function L121-123 — `(&mut self, input: SessionInputs<'_, '_>) -> Result<SessionOutputs<'_>>`
-  `ComposableModel` struct L127-131 — `{ model: &'a mut Model, pipeline: &'a P, params: &'a P::Parameters }` — References a model, a pipeline and some parameters to implement `Composable`
-  `apply` function L144-148 — `(&self, _input: P::Input) -> Result<P::Output>`

#### crates/orp-vendored/src/params.rs

- pub `RuntimeParameters` struct L7-12 — `{ threads: usize, execution_providers: Vec<ExecutionProviderDispatch> }` — Represents the set of parameters for the inference engine
- pub `new` function L15-23 — `( threads: usize, execution_providers: impl IntoIterator<Item = ExecutionProvide...`
- pub `with_threads` function L26-29 — `(mut self, threads: usize) -> Self` — Set the number ot threads (default: 4)
- pub `with_execution_providers` function L32-38 — `( mut self, execution_providers: impl IntoIterator<Item = ExecutionProviderDispa...` — Set the execution providers (default: none, ie.
- pub `threads` function L41-43 — `(&self) -> usize` — Get the number of threads
- pub `execution_providers` function L46-48 — `(&self) -> &[ExecutionProviderDispatch]` — Get the execution providers
-  `RuntimeParameters` type L14-54 — `= RuntimeParameters`
-  `into_execution_providers` function L51-53 — `(self) -> std::vec::IntoIter<ExecutionProviderDispatch>`
-  `RuntimeParameters` type L56-60 — `impl Default for RuntimeParameters`
-  `default` function L57-59 — `() -> Self`

#### crates/orp-vendored/src/pipeline.rs

- pub `Pipeline` interface L7-45 — `{ fn pre_processor(), fn post_processor(), fn to_composable(), fn expected_input...` — Defines a generic pipeline
- pub `PreProcessor` interface L48 — `-` — Defines a generic pre-processor
- pub `PostProcessor` interface L52 — `-` — Defines a generic post-processor
- pub `new` function L63-69 — `(pipeline: P, model: &'a mut Model, params: &'a P::Parameters) -> Self`
-  `to_composable` function L23-32 — `( self, model: &'a mut Model, params: &'a Self::Parameters, ) -> impl Composable...`
-  `expected_inputs` function L36-38 — `(&self, _params: &Self::Parameters) -> Option<impl Iterator<Item = &str>>` — Optionally, the pipeline can expose the (exact) set of input tensors that must be exposed by the model
-  `expected_outputs` function L42-44 — `(&self, _params: &Self::Parameters) -> Option<impl Iterator<Item = &str>>` — Optionally, the pipeline can expose the (sub-)set of output tensors that must be exposed by the model
-  `T` type L49 — `= T`
-  `T` type L53 — `= T`
-  `ComposablePipeline` struct L56-60 — `{ pipeline: P, params: &'a P::Parameters, model: &'a mut Model }` — Owns a pipeline, and references a model and some parameters to implement `Composable`
-  `apply` function L73-77 — `(&self, _input: P::Input) -> Result<P::Output>`

### runtimes/file_read/src

> *Semantic summary to be generated by AI agent.*

#### runtimes/file_read/src/main.rs

-  `RuntimeInput` struct L13-18 — `{ config: Value, context: Value }` — Under WASI preview 1, only preopened directories are accessible.
-  `RuntimeOutput` struct L21-27 — `{ status: String, output: Option<Value>, error: Option<String> }` — Under WASI preview 1, only preopened directories are accessible.
-  `emit_error` function L29-40 — `(msg: &str)` — Under WASI preview 1, only preopened directories are accessible.
-  `emit_output` function L42-47 — `(out: &RuntimeOutput)` — Under WASI preview 1, only preopened directories are accessible.
-  `process` function L50-89 — `(input: &str) -> RuntimeOutput` — Core processing logic, separated for testability.
-  `main` function L91-104 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `tests` module L107-169 — `-` — Under WASI preview 1, only preopened directories are accessible.
-  `test_read_existing_file` function L112-123 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_read_nonexistent_file` function L126-131 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_read_missing_path_config` function L134-139 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_read_empty_file` function L142-153 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_read_invalid_json_input` function L156-160 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_read_path_is_number` function L163-168 — `()` — Under WASI preview 1, only preopened directories are accessible.

### runtimes/file_write/src

> *Semantic summary to be generated by AI agent.*

#### runtimes/file_write/src/main.rs

-  `RuntimeInput` struct L15-20 — `{ config: Value, context: Value }` — Under WASI preview 1, only preopened directories are accessible.
-  `RuntimeOutput` struct L23-29 — `{ status: String, output: Option<Value>, error: Option<String> }` — Under WASI preview 1, only preopened directories are accessible.
-  `emit_error` function L31-42 — `(msg: &str)` — Under WASI preview 1, only preopened directories are accessible.
-  `emit_output` function L44-49 — `(out: &RuntimeOutput)` — Under WASI preview 1, only preopened directories are accessible.
-  `process` function L52-115 — `(input: &str) -> RuntimeOutput` — Core processing logic, separated for testability.
-  `main` function L117-130 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `tests` module L133-233 — `-` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_new_file` function L138-152 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_overwrites_existing` function L155-167 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_append_mode` function L170-183 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_missing_path` function L186-191 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_missing_content` function L194-199 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_invalid_path` function L202-210 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_invalid_json` function L213-217 — `()` — Under WASI preview 1, only preopened directories are accessible.
-  `test_write_empty_content` function L220-232 — `()` — Under WASI preview 1, only preopened directories are accessible.

### runtimes/http/src

> *Semantic summary to be generated by AI agent.*

#### runtimes/http/src/main.rs

-  `RuntimeInput` struct L22-27 — `{ config: Value, context: Value }` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `RuntimeOutput` struct L30-36 — `{ status: String, output: Option<Value>, error: Option<String> }` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `HttpConfig` struct L39-47 — `{ url: String, method: String, headers: HashMap<String, String>, body: Option<St...` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `default_method` function L49-51 — `() -> String` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `process` function L54-92 — `(input: &str) -> RuntimeOutput` — Core processing logic, separated for testability.
-  `main` function L94-107 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `tests` module L110-205 — `-` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_get_request` function L115-125 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_post_with_body` function L128-142 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_with_headers` function L145-157 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_missing_url` function L160-168 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_empty_config` function L171-176 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_invalid_json` function L179-183 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_preserves_context` function L186-194 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `test_http_default_method` function L197-204 — `()` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `emit_error` function L207-218 — `(msg: &str)` — In standalone WASI preview 2 environments, this would use wasi-http directly.
-  `emit_output` function L220-225 — `(out: &RuntimeOutput)` — In standalone WASI preview 2 environments, this would use wasi-http directly.

### runtimes/passthrough/src

> *Semantic summary to be generated by AI agent.*

#### runtimes/passthrough/src/main.rs

-  `RuntimeInput` struct L10-15 — `{ config: Value, context: Value }` — Useful for testing pipelines and debugging context propagation.
-  `RuntimeOutput` struct L18-24 — `{ status: String, output: Option<Value>, error: Option<String> }` — Useful for testing pipelines and debugging context propagation.
-  `emit_error` function L26-37 — `(msg: &str)` — Useful for testing pipelines and debugging context propagation.
-  `emit_output` function L39-44 — `(out: &RuntimeOutput)` — Useful for testing pipelines and debugging context propagation.
-  `process` function L47-69 — `(input: &str) -> RuntimeOutput` — Core processing logic, separated for testability.
-  `main` function L71-84 — `()` — Useful for testing pipelines and debugging context propagation.
-  `tests` module L87-142 — `-` — Useful for testing pipelines and debugging context propagation.
-  `test_passthrough_preserves_context` function L92-99 — `()` — Useful for testing pipelines and debugging context propagation.
-  `test_passthrough_empty_input` function L102-107 — `()` — Useful for testing pipelines and debugging context propagation.
-  `test_passthrough_defaults_on_missing_fields` function L110-117 — `()` — Useful for testing pipelines and debugging context propagation.
-  `test_passthrough_invalid_json` function L120-124 — `()` — Useful for testing pipelines and debugging context propagation.
-  `test_passthrough_empty_string` function L127-130 — `()` — Useful for testing pipelines and debugging context propagation.
-  `test_passthrough_nested_context` function L133-141 — `()` — Useful for testing pipelines and debugging context propagation.

### runtimes/shell/src

> *Semantic summary to be generated by AI agent.*

#### runtimes/shell/src/main.rs

-  `RuntimeInput` struct L20-25 — `{ config: Value, context: Value }` — runtime like Wasmtime with command support.
-  `RuntimeOutput` struct L28-34 — `{ status: String, output: Option<Value>, error: Option<String> }` — runtime like Wasmtime with command support.
-  `emit_error` function L36-47 — `(msg: &str)` — runtime like Wasmtime with command support.
-  `emit_output` function L49-54 — `(out: &RuntimeOutput)` — runtime like Wasmtime with command support.
-  `process` function L57-159 — `(input: &str) -> RuntimeOutput` — Core processing logic, separated for testability.
-  `main` function L161-176 — `()` — runtime like Wasmtime with command support.
-  `tests` module L179-293 — `-` — runtime like Wasmtime with command support.
-  `test_shell_echo` function L184-194 — `()` — runtime like Wasmtime with command support.
-  `test_shell_failed_command` function L197-207 — `()` — runtime like Wasmtime with command support.
-  `test_shell_nonexistent_command` function L210-218 — `()` — runtime like Wasmtime with command support.
-  `test_shell_missing_command` function L221-226 — `()` — runtime like Wasmtime with command support.
-  `test_shell_stdin_pipe` function L229-237 — `()` — runtime like Wasmtime with command support.
-  `test_shell_args_array` function L240-248 — `()` — runtime like Wasmtime with command support.
-  `test_shell_invalid_json` function L251-255 — `()` — runtime like Wasmtime with command support.
-  `test_shell_stderr_capture` function L258-268 — `()` — runtime like Wasmtime with command support.
-  `test_shell_empty_args` function L271-278 — `()` — runtime like Wasmtime with command support.
-  `test_shell_non_string_args_filtered` function L281-292 — `()` — runtime like Wasmtime with command support.

### runtimes/transform/src

> *Semantic summary to be generated by AI agent.*

#### runtimes/transform/src/main.rs

-  `RuntimeInput` struct L18-23 — `{ config: Value, context: Value }` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `RuntimeOutput` struct L26-32 — `{ status: String, output: Option<Value>, error: Option<String> }` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `emit_error` function L34-45 — `(msg: &str)` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `emit_output` function L47-52 — `(out: &RuntimeOutput)` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `resolve_path` function L56-74 — `(root: &'a Value, path: &str) -> Option<&'a Value>` — Resolve a dot-path expression against a JSON value.
-  `interpolate` function L77-105 — `(template: &str, context: &Value) -> String` — Interpolate `{{expression}}` placeholders in a template string.
-  `main` function L107-159 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `tests` module L162-281 — `-` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_simple_path` function L167-170 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_nested_path` function L173-176 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_missing_path` function L179-182 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_array_index` function L185-188 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_root_key` function L191-194 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_basic` function L197-200 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_nested` function L203-209 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_missing` function L212-218 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_number` function L221-224 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_multiple` function L227-230 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_empty_path` function L233-237 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_array_out_of_bounds` function L240-243 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_null_value` function L246-249 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_resolve_boolean_value` function L252-255 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_unclosed_braces` function L258-261 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_empty_expression` function L264-268 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_no_placeholders` function L271-274 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.
-  `test_interpolate_adjacent_placeholders` function L277-280 — `()` — At least one of `expression`, `template`, or `mappings` must be provided.

