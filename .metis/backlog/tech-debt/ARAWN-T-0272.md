---
id: system-prompt-overhaul-dynamic
level: task
title: "System prompt overhaul: dynamic rebuild, core behavioral foundation, and missing wiring"
short_code: "ARAWN-T-0272"
created_at: 2026-03-06T03:48:42.881831+00:00
updated_at: 2026-03-06T04:26:12.678827+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# System prompt overhaul: dynamic rebuild, core behavioral foundation, and missing wiring

## Objective

The system prompt is built once at server startup and never rebuilt. It's missing core sections (identity, datetime, memory guidance, behavioral instructions) and can't incorporate runtime context like workstream info, plugin changes, or session-specific state. The agent performs poorly compared to Claude Code and other well-prompted agents because it receives almost no guidance on who it is, how to behave, or how to use its tools effectively.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**:
  - System prompt built once at `Agent::builder().build()` — stored as a static string, never refreshed
  - `with_identity()`, `with_datetime()`, `with_memory_hints()` exist but are never called from `start.rs`
  - No core behavioral instructions — agent has no guidance on task approach, tool usage patterns, error recovery, response style
  - Agent doesn't know its own name, the current date/time, or that it has full filesystem/OS access
  - Can't hot-load plugin prompts, workstream context, or session-specific preambles after startup
  - Bootstrap files (BEHAVIOR.md, IDENTITY.md) are optional user-provided files — no built-in foundation
  - `with_memory_store()` and `with_embedder()` exist on AgentBuilder but aren't wired in `start.rs`, so active recall is dead code
  - Result: agent gives tutorial-style responses, claims it can't access the filesystem, doesn't use tools proactively

- **Benefits of Fixing**:
  - Agent behaves like a capable local assistant instead of a confused chatbot
  - Dynamic prompt means runtime context (time, workstream, plugins) stays current
  - Core behavioral foundation means consistent quality regardless of whether user provides BEHAVIOR.md
  - Active recall actually works, giving the agent persistent memory

- **Risk Assessment**:
  - This is the single biggest factor in agent quality — every interaction suffers from the current state

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] System prompt is rebuilt per-request (or per-turn), not cached at startup
- [ ] Core behavioral section provides built-in agentic guidance (tool usage patterns, task approach, error handling, response style)
- [ ] Identity is set (agent name, role description) — from config or sensible default
- [ ] Current datetime is included and accurate per-request
- [ ] Memory/think guidance is included when those tools are registered
- [ ] Environment section describes capabilities (filesystem, shell, OS access)
- [ ] Workstream context (current workstream ID, working directory, scratch vs named) is injected per-session
- [ ] Plugin prompt fragments can be updated without server restart
- [ ] `with_memory_store()` and `with_embedder()` are wired in `start.rs` so active recall works
- [ ] Agent no longer claims it lacks capabilities it has
- [ ] Prompt structure follows best practices (identity → behavior → tools → environment → context)

## Implementation Plan

### Phase 1: Make the prompt dynamic (rebuild per-turn)

**Problem**: `SystemPromptBuilder` runs once at build time, result stored as `config.system_prompt: Option<String>`.

**Approach**: 
- Move prompt assembly from `AgentBuilder::build()` to `Agent::build_request()` (called every LLM turn)
- Store the `SystemPromptBuilder` (or its inputs) on `Agent` instead of the built string
- `build_request()` calls `builder.build()` each time, giving fresh datetime etc.
- The `context_preamble` mechanism already exists for per-request additions — this just extends it

**Key files**:
- `crates/arawn-agent/src/agent.rs` — move prompt build from `build()` to `build_request()`
- `crates/arawn-agent/src/prompt/builder.rs` — make `build()` take `&self` not `self` (non-consuming)

### Phase 2: Add core behavioral foundation

**Problem**: No built-in behavioral instructions. Agent relies entirely on user-provided BEHAVIOR.md.

**Approach**:
- Add `build_behavior_section()` to `SystemPromptBuilder` — always included in Full mode
- Content should cover:
  - **Task approach**: Read before editing. Understand context before acting. Plan multi-step tasks.
  - **Tool usage patterns**: Prefer dedicated tools over shell equivalents. Use shell for system operations. Read files before modifying them.
  - **Error handling**: Don't retry blindly. Investigate root causes. Try alternative approaches.
  - **Response style**: Be direct and concise. Lead with actions, not explanations. Show your work through tool use, not prose.
  - **Agentic loop**: When given a task, act on it — don't just describe what you'd do. Use tools to accomplish goals directly.
- This is the baseline — BEHAVIOR.md can override/extend it

**Key files**:
- `crates/arawn-agent/src/prompt/builder.rs` — new `build_behavior_section()` method

### Phase 3: Wire missing pieces in start.rs

**Problem**: Several builder methods exist but aren't called.

**Approach**:
- Wire `with_identity("Arawn", <description from config or default>)` 
- Wire `with_datetime(timezone)` 
- Wire `with_memory_hints()` when memory is enabled
- Wire `with_memory_store()` and `with_embedder()` to enable active recall
- Add agent identity config to `AgentProfileConfig` or `ServerConfig` (name, description fields)

**Key files**:
- `crates/arawn/src/commands/start.rs` — wire the calls
- `crates/arawn-config/src/types.rs` — add identity fields to config

### Phase 4: Inject workstream/session context

**Problem**: Agent doesn't know which workstream it's in, what its working directory is, or the session structure.

**Approach**:
- The `context_preamble` mechanism in `build_request()` already supports per-request injection
- WS handler should pass workstream context: ID, type (scratch vs named), working directory, allowed paths
- This gives the agent spatial awareness of its sandbox

**Key files**:
- `crates/arawn-server/src/routes/ws/handlers.rs` — pass workstream context
- `crates/arawn-agent/src/agent.rs` — consume in `build_request()`

### Phase 5: Prompt section ordering (best practices)

Reorder sections to match proven patterns:
1. **Identity** — who you are
2. **Behavior** — how you operate (core built-in + BEHAVIOR.md overlay)
3. **Tools** — what you can do
4. **Environment** — where you are (workspace, OS, filesystem)
5. **Memory** — how to remember things
6. **Thinking** — how to reason
7. **DateTime** — when it is
8. **Context Files** — bootstrap files (IDENTITY.md, BOOTSTRAP.md, etc.)
9. **Plugins** — plugin prompt fragments
10. **Session Context** — workstream, session preamble (injected per-request)

## Status Updates

### Session 1 (prior)
- Phase 1: Dynamic prompt ✅ — `build()` takes `&self`, builder stored on Agent, rebuilt per-turn
- Phase 2: Core behavioral foundation ✅ — `build_behavior_section()` with Task Approach, Tool Usage, Communication
- Phase 3: Wiring in start.rs ✅ — identity, datetime, memory hints, memory store, embedder all wired
- Phase 5: Section ordering ✅ — Identity → Behavior → Tools → Environment → Memory → Thinking → DateTime → Context → Plugins

### Session 2
- Added **Interactive vs Autonomous** guidance to behavior section in `builder.rs:261`
  - Interactive: ask for clarification on ambiguous tasks, confirm before large/irreversible changes
  - Autonomous: make reasonable decisions, don't block on questions, document assumptions
- Phase 4: Session context injection ✅ — `handlers.rs` now sets `context_preamble` with session ID and workstream ID on first chat message
- All phases complete. Workspace compiles clean, all unit tests pass.