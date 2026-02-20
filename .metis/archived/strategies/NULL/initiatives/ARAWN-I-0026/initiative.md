---
id: context-management-and-auto
level: initiative
title: "Context Management and Auto-Compaction"
short_code: "ARAWN-I-0026"
created_at: 2026-02-16T16:05:50.737839+00:00
updated_at: 2026-02-19T18:10:55.530892+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: context-management-and-auto
---

# Context Management and Auto-Compaction Initiative

## Context

Currently, Arawn sessions grow unbounded until they hit the LLM context window limit, at which point old turns are simply dropped (truncated). There's no visibility into context growth, no proactive management, and no way for users to trigger compaction manually.

The existing compression system (`arawn-workstream/src/compression.rs`) only runs post-hoc after sessions end - it can't help during long-running sessions.

This initiative adds:
1. Real-time context tracking with thresholds
2. Automatic compaction when context gets too large
3. Manual `/compact` command (introducing slash commands to Arawn)

## Goals & Non-Goals

**Goals:**
- Track token usage per session with configurable thresholds
- Automatically trigger LLM-based summarization when context exceeds critical threshold
- Add `/compact` slash command for user-triggered compaction
- Introduce slash command infrastructure to Arawn CLI/TUI
- Expose context status to TUI (optional indicator)

**Non-Goals:**
- RLM subagent for exploration (separate initiative, needs dedicated design)
- Mid-turn compaction (only between turns)
- Cross-session context sharing

## Use Cases

### Use Case 1: Long Coding Session
- **Actor**: Developer using Arawn TUI
- **Scenario**: Multi-hour session with many tool calls, context growing large
- **Expected Outcome**: At 90% capacity, auto-compaction summarizes old turns. Developer continues uninterrupted.

### Use Case 2: Manual Cleanup
- **Actor**: Developer who notices context feels "stale"
- **Scenario**: Types `/compact` in TUI
- **Expected Outcome**: Current session is summarized, freeing context space.

### Use Case 3: Context Awareness
- **Actor**: Developer monitoring session
- **Scenario**: Glances at TUI status bar
- **Expected Outcome**: Sees context usage indicator (e.g., "Context: 72%")

## Architecture

### Overview

```
┌─────────────────────────────────────────────────────────┐
│                      Agent.turn()                       │
└─────────────────────────────────────────────────────────┘
         │                                    │
         ▼                                    ▼
┌─────────────────┐                  ┌─────────────────┐
│ ContextTracker  │ ◄────────────────│  ContextBuilder │
│                 │   updates after  │                 │
│ - current_tokens│   build_messages │ - builds context│
│ - thresholds    │                  │ - estimates size│
└─────────────────┘                  └─────────────────┘
         │
         │ threshold exceeded?
         ▼
┌─────────────────┐
│ SessionCompactor│
│                 │
│ - summarize()   │
│ - replace turns │
└─────────────────┘
```

### Slash Command Infrastructure

Commands are a **server-side concept**. The `/command` syntax is client presentation.

```
┌─────────────┐          ┌─────────────┐          ┌─────────────────┐
│   TUI/CLI   │          │   Server    │          │  Command        │
│             │    WS    │             │   REST   │  Handlers       │
│  /compact   │ ───────► │  WS Bridge  │ ───────► │                 │
│             │          │             │          │  POST /commands │
└─────────────┘          └─────────────┘          │  /compact       │
                                                  └─────────────────┘
       │                                                   ▲
       │                      REST                         │
       └───────────────────────────────────────────────────┘
                    (CLI can call directly)
```

**Key points:**
- REST endpoints handle command logic (testable, documented)
- WS acts as thin bridge for active sessions
- CLI can call REST directly without WS connection
- Progress streaming via WS when connected

## Detailed Design

### ContextTracker

Lives **per-session** (and per-exploration-context for subagents).

```rust
pub struct ContextTracker {
    /// Maximum tokens for current model (from LLM config)
    max_tokens: usize,
    /// Current estimated token count
    current_tokens: usize,
    /// Warn user at this percentage (default: 0.7)
    warning_threshold: f32,
    /// Trigger auto-compact at this percentage (default: 0.9)
    critical_threshold: f32,
}

impl ContextTracker {
    /// Create from model config
    pub fn for_model(model_config: &ModelConfig) -> Self;
    
    pub fn update(&mut self, token_count: usize);
    pub fn status(&self) -> ContextStatus; // Ok, Warning, Critical
    pub fn usage_percent(&self) -> f32;
    pub fn should_compact(&self) -> bool;
}

pub enum ContextStatus {
    Ok,
    Warning { percent: f32 },
    Critical { percent: f32 },
}
```

**Model context limits** come from LLM provider config:

```toml
[llm.providers.anthropic]
# ...

[llm.providers.anthropic.models.claude-sonnet]
max_context_tokens = 200000

[llm.providers.groq.models.llama-70b]
max_context_tokens = 32000
```

**Subagent awareness**: When spawning RLM or delegate, they get their own ContextTracker instance scoped to their exploration context.

### SessionCompactor

Reuses existing `Compressor` from arawn-workstream but operates mid-session:

```rust
pub struct SessionCompactor {
    compressor: Compressor,
    /// Number of recent turns to preserve verbatim
    preserve_recent: usize,  // default: 3
}

impl SessionCompactor {
    /// Compact a session, keeping recent turns intact
    pub async fn compact(&self, session: &mut Session) -> Result<CompactionResult>;
}
```

### Command REST API

```
GET  /api/v1/commands
     → List available commands with descriptions

POST /api/v1/commands/compact
     Body: { "session_id": "...", "force": false }
     Response: Streams progress events, then final result
     
POST /api/v1/commands/help
     Body: { "command": "compact" }  // optional
     Response: { "commands": [...] } or single command details
```

### Command Handler Trait

```rust
#[async_trait]
pub trait CommandHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: CommandContext,
    ) -> Result<CommandStream>;  // Streams progress + final result
}

pub struct CommandRegistry {
    handlers: HashMap<String, Arc<dyn CommandHandler>>,
}

// Built-in commands
pub struct CompactCommand;  // /compact [--force]
pub struct HelpCommand;     // /help [command]
```

### WebSocket Bridge Messages

```rust
// Client → Server
pub struct WsCommandRequest {
    pub command: String,
    pub args: serde_json::Value,
}

// Server → Client (progress)
pub struct WsCommandProgress {
    pub command: String,
    pub message: String,
    pub percent: Option<f32>,
}

// Server → Client (complete)
pub struct WsCommandResult {
    pub command: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}
```

## UI/UX Design

### TUI Context Indicator

Status bar addition:
```
[workstream: arawn-dev] [session: abc123] [Context: 72%] [tokens: ~85k/120k]
                                          ^^^^^^^^^^^^^^
                                          New indicator
```

Color coding:
- Green: < 70% (healthy)
- Yellow: 70-90% (warning)
- Red: > 90% (critical, auto-compact imminent)

### Slash Command Input

When user types `/` in input:
- Show autocomplete popup with available commands
- `/compact` - Compact current session
- `/help` - Show available commands

## Design Decisions

1. **ContextTracker scope**: Per-session. Subagents (RLM, delegate) get their own instance for isolation.

2. **Model context limits**: Configured in LLM provider/model config. Tracker looks up limit based on active model.

3. **Commands are server-side**: `/command` is client syntax. Server defines and executes commands via REST API.

4. **Dual-mode access**: WebSocket bridges to REST for active sessions. CLI can call REST directly.

5. **Progress streaming**: Commands stream progress via WS when connected, or via SSE for REST.

## Alternatives Considered

### 1. Sliding Window (Rejected)
Keep last N turns + rolling summary updated every turn.
- **Rejected**: Too much LLM overhead for continuous updates.

### 2. Tiered Hot/Warm/Cold (Deferred)
Multiple compression levels based on age.
- **Deferred**: Over-engineered for initial implementation. Can add later.

### 3. Hook-Based Architecture (Rejected)
Use existing hook system for compaction triggers.
- **Rejected**: Hooks are for plugins. This is core functionality.

### 4. Server-Side Only (Rejected)
All compaction happens server-side, client unaware.
- **Rejected**: User needs visibility and control via `/compact`.

## Implementation Plan

### Task Breakdown

1. **Model context limits in LLM config** (arawn-config, arawn-llm)
   - Add `max_context_tokens` to model config schema
   - Lookup method for model → limit

2. **ContextTracker implementation** (arawn-agent)
   - Add ContextTracker struct with `for_model()` constructor
   - Integrate with Session (per-session tracking)
   - ContextStatus enum and threshold logic

3. **SessionCompactor implementation** (arawn-agent)
   - Adapt existing Compressor for mid-session use
   - Implement turn preservation logic
   - Add compaction result types with progress

4. **Command REST API** (arawn-server)
   - `GET /api/v1/commands` - list commands
   - `POST /api/v1/commands/compact` - execute compact
   - CommandHandler trait and registry
   - SSE streaming for progress

5. **WebSocket command bridge** (arawn-server)
   - WsCommandRequest/Progress/Result message types
   - Bridge WS messages to REST handlers
   - Stream progress back over WS

6. **TUI command input** (arawn-tui)
   - Parse `/` prefix as command
   - Autocomplete popup for available commands
   - Send WsCommandRequest, handle responses

7. **TUI context indicator** (arawn-tui)
   - Status bar integration
   - Color-coded usage display (green/yellow/red)
   - Update on each turn

8. **Integration and testing**
   - End-to-end test with long session
   - Verify compaction preserves recent context
   - Test `/compact` via both WS and REST

### Configuration

```toml
[context]
warning_threshold = 0.7
critical_threshold = 0.9
preserve_recent_turns = 3
compaction_model = "default"  # or specific model name
```