# OpenClaw vs Arawn Comparison

A comprehensive analysis of OpenClaw (TypeScript) vs Arawn (Rust) capabilities, identifying patterns and ideas worth adopting.

---

## Executive Summary

| Dimension | OpenClaw | Arawn | Gap Assessment |
|-----------|----------|-------|----------------|
| **Architecture** | TypeScript, Node 22+, extensive plugin system | Rust, modular crates, emerging plugin system | Arawn needs plugin maturity |
| **Memory** | Multiple backends (SQLite, LanceDB), hybrid search | SQLite + vec0 + graphqlite, confidence scoring | Arawn has stronger graph/confidence |
| **Messaging** | 15+ channels (Telegram, Discord, Slack, etc.) | HTTP/WS only | Major gap - Arawn CLI-focused |
| **Tools** | 50+ tools with channel-specific variants | 10 core tools | Arawn needs tool expansion |
| **Multi-Agent** | Per-session agent isolation, delegation | Single agent model | Gap - but simpler for personal use |
| **Security** | Per-channel policies, pairing, gating | Basic auth middleware | OpenClaw more enterprise-ready |
| **Configuration** | Zod-validated, env substitution, UI hints | TOML + keyring, cascading merge | Arawn simpler but less flexible |
| **Session Mgmt** | Persistent JSONL logs, cross-agent routing | In-memory + indexing to memory.db | Comparable |
| **Edge/Local** | Node-based, optional local models | Rust + ONNX, designed for edge | Arawn's core strength |

---

## 1. Architecture Patterns Worth Adopting

### 1.1 Tool Factory Pattern
**OpenClaw Pattern:**
```typescript
// Tools are factories, not singletons
type ChannelAgentToolFactory = (params: { cfg?: OpenClawConfig }) => ChannelAgentTool[];

// Tools instantiated per-session with context
const api = createApi(record, { config, pluginConfig });
const tools = register(api);
```

**Arawn Current:**
```rust
// Tools are static structs with config
pub struct ShellTool { config: ShellConfig }
impl Tool for ShellTool { ... }
```

**Recommendation:** Consider a context-aware tool instantiation pattern where tools can access session-specific configuration. This would enable:
- Per-user tool restrictions
- Session-specific tool state
- Plugin-provided tools with custom config

### 1.2 Plugin Registry with Hot-Loading
**OpenClaw:**
- `jiti` for runtime TypeScript/ESM loading
- Workspace + npm discovery
- Config schema validation per plugin
- Diagnostic events for plugin lifecycle

**Arawn Current:**
- Static plugin trait (`arawn-plugin`)
- WASM runtime for sandboxed execution
- No hot-reload

**Recommendation:** Keep Arawn's WASM sandboxing (security advantage), but add:
- Plugin manifest validation at load time
- Runtime diagnostic events
- Plugin config schema support (like OpenClaw's JSON Schema validation)

### 1.3 Configuration UI Hints
**OpenClaw Pattern:**
```typescript
// Every config option has UI metadata
{
  label: "Enable Image Understanding",
  help: "Allow agents to see and analyze images",
  advanced: false,
  sensitive: false,
  placeholder: "true",
  group: "Tools"
}
```

**Recommendation:** Add UI hint annotations to `arawn-config` for:
- Future web UI configuration
- CLI `config describe` command
- Better documentation generation

---

## 2. Tool Capabilities Comparison

### 2.1 OpenClaw Tools Arawn Could Benefit From

| Tool | OpenClaw | Arawn | Priority |
|------|----------|-------|----------|
| **browser** | Full Puppeteer automation | None | Medium - useful for research |
| **cron** | Scheduled task management | None | Low - background daemon needed |
| **message** | 11+ channel routing | None | Low - not core use case |
| **session_status** | Multi-session awareness | Single session | Low - personal agent |
| **memory** | Hybrid search + citations | ✅ Similar | - |
| **canvas/nodes** | Drawing, screen capture | None | Low |

### 2.2 Arawn Tools OpenClaw Lacks

| Tool | Arawn | OpenClaw Equivalent |
|------|-------|---------------------|
| **workflow** | WASM-sandboxed pipelines | None (hooks are simpler) |
| **think** | Explicit reasoning traces | None |
| **catalog** | Tool discovery/filtering | Implicit in agent config |
| **note** | Persistent notes | Via memory/filesystem |

### 2.3 Shared Tools - Implementation Comparison

**web_fetch:**
- Arawn: Full HTTP methods, headers, body, auto-download on size exceeded ✅
- OpenClaw: Basic fetch, relies on browser tool for complex cases

**shell:**
- Arawn: Basic command execution with timeout
- OpenClaw: More sophisticated with PTY mode, streaming output

**Recommendation:** Enhance Arawn's shell tool with:
- PTY mode for interactive commands
- Streaming output for long-running processes
- Working directory persistence (OpenClaw does this)

---

## 3. Memory Systems Comparison

### 3.1 Architecture

| Aspect | OpenClaw | Arawn |
|--------|----------|-------|
| **Storage** | SQLite (default), LanceDB (plugin) | SQLite + vec0 + graphqlite |
| **Vector Search** | Via embedding provider plugins | Built-in sqlite-vec |
| **Graph** | None | graphqlite integration |
| **Confidence** | None | Full scoring system |
| **Contradiction** | None | Detect/supersede/reinforce |

**Arawn Advantage:** The confidence scoring and contradiction detection is more sophisticated:
```rust
// Arawn's confidence formula
score = base × reinforcement × staleness
// base: stated=1.0, system=0.9, observed=0.7, inferred=0.5
// reinforcement: min(1 + 0.1n, 1.5)
// staleness: linear decay to 0.3 over 365 days
```

### 3.2 OpenClaw Memory Features Worth Considering

**Batch Operations:**
```typescript
// OpenClaw supports batch embedding/storage
await memoryIndex.addBatch(memories, { batchSize: 100 });
```

**Citation Tracking:**
```typescript
// Memories track source citations
type Memory = {
  content: string;
  citation?: { file: string; line: number };
};
```

**Recommendation:** Add citation support to Arawn memories for source tracking.

---

## 4. Session Intelligence Comparison

### 4.1 Session Lifecycle

| Phase | OpenClaw | Arawn |
|-------|----------|-------|
| **Creation** | Per-agent session with routing key | Simple session ID |
| **Persistence** | JSONL logs in workspace | In-memory during session |
| **Indexing** | Via hooks (session:end event) | SessionIndexer on close |
| **Retrieval** | Memory tool search | recall() in context |

**OpenClaw Session Key Format:**
```
agent:agentId:sessionKey
// Enables per-agent isolation
```

**Recommendation:** Consider session namespacing if Arawn ever supports multiple agent configs.

### 4.2 Session Indexing

**Arawn Advantage:** More sophisticated extraction pipeline:
1. Optional NER (GlinerEngine) for entities
2. LLM extraction for facts + relationships
3. Graph storage for relationships
4. Confidence-scored fact storage with contradiction detection
5. Session summarization

**OpenClaw:** Simpler hook-based approach - plugins can register for `session:end` events.

---

## 5. Prompts and Guidelines Worth Adopting

### 5.1 AGENTS.md / CLAUDE.md Patterns

OpenClaw's CLAUDE.md is comprehensive (18KB+) with:

**Multi-agent safety rules:**
```markdown
- Do **not** create/apply/drop `git stash` entries unless explicitly requested
- Do **not** switch branches unless explicitly requested
- When you see unrecognized files, keep going; focus on your changes
```

**Tool schema guardrails:**
```markdown
- Avoid `Type.Union` in tool input schemas; no `anyOf`/`oneOf`/`allOf`
- Use `stringEnum`/`optionalStringEnum` for string lists
- Keep top-level tool schema as `type: "object"` with `properties`
```

**Recommendation:** Create an AGENTS.md for Arawn with:
- Tool usage best practices
- Session memory guidelines
- Multi-turn conversation patterns

### 5.2 Shorthand Commands

**OpenClaw Pattern:**
```markdown
## Shorthand Commands
- `sync`: if dirty, commit all changes, then `git pull --rebase`, then `git push`
```

**Recommendation:** Document common workflows as shorthand commands that agents can understand.

---

## 6. Hook/Event System Comparison

### 6.1 OpenClaw Hooks

- **4 sources:** bundled, npm, workspace, plugin
- **Event-driven:** `session:start`, `session:end`, `command:new`, etc.
- **Metadata:** OS requirements, binary dependencies, install specs
- **Lifecycle:** Discovery → Eligibility check → Lazy load → Execute

**Example:**
```typescript
type OpenClawHookMetadata = {
  events: string[];  // ["command:new", "session:start"]
  requires?: {
    bins?: string[];  // Required binaries
    env?: string[];   // Environment variables
  };
  install?: HookInstallSpec[];  // npm/git dependencies
};
```

### 6.2 Arawn Current State

- No formal hook system
- Indexing triggered programmatically on session close
- Plugin trait for custom logic

**Recommendation:** Consider a lightweight event system:
```rust
pub enum AgentEvent {
    SessionStart { session_id: String },
    SessionEnd { session_id: String, message_count: usize },
    ToolExecuted { tool_name: String, duration_ms: u64 },
    MemoryStored { memory_id: i64, content_preview: String },
}

pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &AgentEvent) -> Result<()>;
}
```

---

## 7. CLI/Gateway Architecture

### 7.1 OpenClaw Model

```
CLI (stateless) ──RPC──▶ Gateway (long-running)
                              │
                              ├── Channel connections
                              ├── Session persistence
                              └── Webhook handling
```

- CLI makes RPC calls to gateway for channel operations
- Gateway maintains persistent connections (Discord, Telegram bots)
- macOS menubar app wraps gateway

### 7.2 Arawn Model

```
CLI/HTTP Client ──▶ Server (arawn-server)
                        │
                        ├── Agent execution
                        ├── Session management
                        └── Memory operations
```

- Simpler model appropriate for personal/edge use
- No persistent channel connections needed
- `arawn start` runs the server

**Assessment:** Arawn's model is appropriate for its use case. Only adopt gateway separation if adding persistent messaging channels.

---

## 8. Security Patterns

### 8.1 OpenClaw Security Features

| Feature | OpenClaw | Arawn |
|---------|----------|-------|
| DM Policy | pairing/allowlist/open per channel | N/A |
| Group Tool Policy | Per-sender tool restrictions | N/A |
| Mention Gating | Require @mention for response | N/A |
| Approval System | Pre-execution approval workflow | N/A |
| Sandbox | Docker-based optional sandbox | WASM sandbox for pipelines |

### 8.2 Arawn Security Model

- API key auth middleware
- Rate limiting
- WASM sandbox for untrusted pipeline scripts
- Keyring-based secret storage

**Recommendation:** Arawn's model is sufficient for personal use. Add approval workflows if supporting untrusted tool execution.

---

## 9. Patterns to Avoid

### 9.1 Over-Complexity for Personal Use

OpenClaw patterns that don't fit Arawn's edge-first model:
- 15+ messaging channel support (complexity)
- Multi-auth profile management (enterprise feature)
- Channel-specific tool variants (bloat)
- Extensive diagnostic telemetry (overkill)

### 9.2 Keep Arawn's Strengths

- **Rust performance:** Stay with native, not Node
- **Edge-first:** ONNX for local embeddings/NER
- **Confidence scoring:** More sophisticated than OpenClaw
- **Graph integration:** OpenClaw lacks this entirely
- **WASM sandbox:** Better security model than Docker for scripts

---

## 10. Prioritized Adoption Roadmap

### High Priority (Core Improvements)

1. **Plugin Config Schema Validation**
   - Add JSON Schema support to plugin manifests
   - Validate plugin config at load time

2. **Shell Tool Enhancements**
   - PTY mode for interactive commands
   - Streaming output support
   - Working directory state

3. **Citation Tracking in Memories**
   - Add source file/line to memory metadata
   - Surface citations in recall results

### Medium Priority (UX Improvements)

4. **Config UI Hints**
   - Add label/help/group metadata to config options
   - Enable future web UI or better CLI help

5. **Event System**
   - Lightweight `AgentEvent` enum
   - Plugin-registerable handlers
   - Enables hooks without full hook infrastructure

6. **AGENTS.md Guidelines**
   - Document tool usage patterns
   - Multi-turn best practices
   - Memory interaction guidelines

### Low Priority (Future Consideration)

7. **Multi-Agent Sessions**
   - Session key namespacing
   - Per-agent workspace isolation

8. **Browser Automation Tool**
   - Consider headless browser for research tasks
   - Lower priority given web_fetch capabilities

9. **Messaging Channels**
   - Only if there's a use case beyond HTTP/WS
   - Would require significant architecture changes

---

## 11. Key Takeaways

### What OpenClaw Does Better
1. **Plugin ecosystem maturity** - Discovery, validation, hot-loading
2. **Tool breadth** - 50+ tools vs 10
3. **Multi-channel support** - 15+ messaging platforms
4. **Configuration flexibility** - Zod validation, env substitution, UI hints
5. **Documentation** - Comprehensive CLAUDE.md guidelines

### What Arawn Does Better
1. **Memory sophistication** - Confidence scoring, contradiction detection, graph integration
2. **Edge deployment** - Rust + ONNX, designed for local execution
3. **Security model** - WASM sandbox for untrusted code
4. **Architecture simplicity** - Appropriate for personal use case
5. **Session intelligence** - Multi-stage extraction with NER + LLM

### Conclusion

Arawn and OpenClaw serve different use cases:
- **OpenClaw:** Enterprise-grade, multi-channel bot platform
- **Arawn:** Personal research agent optimized for edge computing

The adoption priority should focus on patterns that enhance Arawn's core mission without adding enterprise complexity:
1. Better plugin validation and config
2. Enhanced shell tool capabilities
3. Citation tracking in memories
4. Lightweight event system
5. Better documentation/guidelines

Avoid adopting patterns that would bloat Arawn or conflict with its edge-first philosophy.
