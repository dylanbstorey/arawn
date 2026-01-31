---
id: plugin-runtime-hot-loadable-skills
level: initiative
title: "Plugin Runtime: Hot-Loadable Skills and Extensions"
short_code: "ARAWN-I-0013"
created_at: 2026-01-29T01:21:49.170851+00:00
updated_at: 2026-02-04T14:08:01.699374+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: plugin-runtime-hot-loadable-skills
---

# Plugin Runtime: Hot-Loadable Skills and Extensions Initiative

## Context

Arawn currently has a fixed set of compiled-in tools. Adding new capabilities requires modifying Rust source and recompiling. For an edge computing agent, extensibility is critical: different deployments need different tools (home automation vs. research vs. devops). A plugin system allows tailoring without forking.

Claude Code's plugin architecture provides a strong reference model with three distinct extension types:
- **Skills** — prompt templates that teach the agent how to do something (instructions + tool usage patterns, not executable code)
- **Hooks** — event-driven triggers (PreToolUse, PostToolUse, SessionStart, etc.) that run shell commands for validation, logging, or side effects
- **Agents** — specialized subagent configurations with constrained tool sets, custom system prompts, and specific behavioral parameters

Plugins bundle these components together with a manifest. A single plugin can provide skills, hooks, agents, tools, and prompt fragments — giving plugin authors full control over how the agent behaves in their domain.

MCP (Model Context Protocol) server support is handled separately in ARAWN-I-0016. Plugins often guide and orchestrate MCP servers, but the MCP runtime is its own concern.

## Goals & Non-Goals

**Goals:**
- Define a plugin manifest format (TOML) describing skills, hooks, agents, tools, and prompt fragments
- Implement plugin discovery from `~/.config/arawn/plugins/` and `./plugins/`
- **Skills**: Prompt templates with progressive disclosure — teach the agent domain-specific workflows
- **Hooks**: Lifecycle event handlers (PreToolUse, PostToolUse, SessionStart, SessionEnd, Stop) that run shell commands
- **Agents**: Subagent configurations with constrained tool sets and custom system prompts
- **Tools**: CLI-wrapper tools (JSON stdin/stdout protocol) for any-language extensibility
- **Prompt fragments**: Plugin-provided system prompt sections injected into the prompt builder
- Hot-reload via file watcher — add/update/remove plugins without restart
- Wire plugin components into existing systems (ToolRegistry, SystemPromptBuilder, agent turn loop)

**Non-Goals:**
- MCP server support — separate initiative (ARAWN-I-0016)
- WASM plugin runtime — too complex for v1
- Rust dylib plugins — ABI stability issues
- Plugin marketplace / distribution — manual install for now
- Permission enforcement — trust model is "user installed it"

## Architecture

### Directory Layout

```
~/.config/arawn/plugins/
  github/
    plugin.toml              # manifest
    skills/
      review-pr.md           # skill: prompt template for PR review workflow
      create-issue.md        # skill: guided issue creation
    hooks/
      pre-commit-check.sh    # hook: runs before shell tool executes git commit
    agents/
      code-reviewer.toml     # agent: subagent config for code review
    tools/
      gh-wrapper.sh          # tool: CLI wrapper for gh CLI
  home-automation/
    plugin.toml
    skills/
      morning-routine.md
    tools/
      hue-control             # compiled binary
```

### Plugin Manifest (`plugin.toml`)

```toml
[plugin]
name = "github"
version = "0.1.0"
description = "GitHub operations, PR review, and issue management"

# Prompt fragment injected into system prompt when plugin is loaded
[prompt]
system = """You have access to GitHub via the github tool.
Use the /review-pr skill for pull request reviews.
Use the /create-issue skill for filing new issues."""

# CLI-wrapper tools (JSON stdin/stdout)
[[tools]]
name = "github"
description = "GitHub CLI operations: issues, PRs, repos, actions"
command = "./tools/gh-wrapper.sh"

[tools.parameters]
type = "object"
properties.action = { type = "string", enum = ["issues.list", "issues.create", "pr.list", "pr.create", "pr.diff"] }
properties.repo = { type = "string" }
properties.query = { type = "string" }

# Skills — prompt templates the agent can invoke
[[skills]]
name = "review-pr"
description = "Review a pull request with structured feedback"
file = "./skills/review-pr.md"
# Skills can declare which tools they use
uses_tools = ["github", "shell"]

[[skills]]
name = "create-issue"
description = "Create a well-structured GitHub issue"
file = "./skills/create-issue.md"
uses_tools = ["github"]

# Hooks — lifecycle event handlers
[[hooks]]
event = "PreToolUse"
tool_match = "shell"           # only trigger for shell tool
match_pattern = "git commit"   # only when command contains this
command = "./hooks/pre-commit-check.sh"

[[hooks]]
event = "SessionStart"
command = "./hooks/load-repo-context.sh"

# Agents — subagent configurations
[[agents]]
name = "code-reviewer"
description = "Specialized code review agent"
file = "./agents/code-reviewer.toml"
tools = ["github", "shell", "file_read", "grep"]  # constrained tool set
```

### Skill Format (Markdown)

Skills are prompt templates with TOML frontmatter:

```markdown
---
name: review-pr
description: Review a pull request with structured feedback
uses_tools: ["github", "shell"]
args:
  - name: pr_number
    description: PR number or URL to review
    required: true
---

# PR Review Skill

## Steps

1. Use the `github` tool to fetch the PR diff:
   action: "pr.diff", repo: "{repo}", query: "{pr_number}"

2. Read the diff carefully. For each file changed, assess:
   - Correctness: Are there bugs or logic errors?
   - Style: Does it follow project conventions?
   - Security: Any OWASP top-10 concerns?

3. Provide structured feedback as:
   ## Summary
   [1-2 sentence overview]
   
   ## Issues Found
   - [file:line] severity: description
   
   ## Suggestions
   - [optional improvements]
```

### Agent Config Format (TOML)

```toml
[agent]
name = "code-reviewer"
description = "Specialized agent for code review tasks"
model = "claude-sonnet"  # optional model override

[agent.system_prompt]
text = """You are a senior code reviewer. Focus on correctness, 
security, and maintainability. Be direct and specific."""

[agent.constraints]
tools = ["github", "shell", "file_read", "grep"]
max_iterations = 10
```

### Plugin Runtime

```rust
// crates/arawn-agent/src/plugin.rs
pub struct PluginManager {
    plugin_dirs: Vec<PathBuf>,
    loaded: HashMap<String, LoadedPlugin>,
    watcher: notify::RecommendedWatcher,
}

pub struct LoadedPlugin {
    manifest: PluginManifest,
    tools: Vec<Box<dyn Tool>>,
    skills: Vec<Skill>,
    hooks: Vec<Hook>,
    agents: Vec<AgentConfig>,
    prompt_fragment: Option<String>,
}

pub struct Skill {
    name: String,
    description: String,
    content: String,       // rendered markdown template
    uses_tools: Vec<String>,
    args: Vec<SkillArg>,
}

pub struct Hook {
    event: HookEvent,
    tool_match: Option<String>,
    match_pattern: Option<String>,
    command: PathBuf,
}

pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    SessionStart,
    SessionEnd,
    Stop,
}
```

### CLI Tool Protocol

CLI-wrapper tools receive JSON on stdin, return JSON on stdout:
```json
// stdin
{"action": "issues.list", "repo": "dstorey/arawn"}

// stdout  
{"success": true, "content": "Issue #1: Fix keyring...\nIssue #2: ..."}
```

## Detailed Design

### Plugin Discovery
1. On startup, scan `~/.config/arawn/plugins/` and `./plugins/`
2. Parse each `plugin.toml` manifest
3. For each plugin, load all components:
   - **Tools**: Wrap CLI tool definitions in `CliPluginTool` adapters
   - **Skills**: Parse markdown files with frontmatter, register as invocable skills
   - **Hooks**: Register event handlers in the hook dispatcher
   - **Agents**: Parse agent configs, make available for subagent spawning
   - **Prompt fragments**: Collect for injection into `SystemPromptBuilder`
4. Register everything in their respective registries

### Skill Invocation
Skills are invoked by name (e.g., `/review-pr 123`). The agent turn loop:
1. Detects skill invocation in user message (or agent decides to use one)
2. Loads skill markdown template
3. Substitutes args into template
4. Injects skill content as a system-level instruction for the current turn
5. The skill's `uses_tools` list can optionally constrain available tools for that turn

### Hook Dispatch
Hooks fire at lifecycle events in the agent turn loop:
1. **PreToolUse**: Before a tool executes. Hook receives tool name + args as JSON on stdin. If hook exits non-zero, tool call is blocked. Stdout can provide a reason message.
2. **PostToolUse**: After a tool executes. Hook receives tool name + args + result. Informational only (logging, side effects).
3. **SessionStart**: When a new session begins. Hook can inject context.
4. **SessionEnd**: When a session closes. Hook can trigger cleanup/summarization.
5. **Stop**: When the agent produces a final response. Hook can validate output.

Hook matching:
- `tool_match`: only fire for specific tool names (glob patterns)
- `match_pattern`: only fire when tool args match a pattern (regex)
- Both optional — omit for "fire on every event of this type"

### Agent Spawning
Plugin agents are subagent configurations. When invoked:
1. Create a new `Agent` instance with the plugin agent's system prompt
2. Constrain `ToolRegistry` to only the declared tool set
3. Optionally override the LLM model
4. Run the subagent with the user's message
5. Return the subagent's response to the parent agent

### Hot Reload
- Use `notify` crate to watch plugin directories
- On manifest/skill/hook change: unload old plugin state, reload from disk, re-register
- On tool executable change: no action needed (CLI tools are stateless, invoked per-call)
- Reload is atomic per-plugin: swap the entire `LoadedPlugin` struct
- Log reload events at INFO level

### Integration Points
- **ToolRegistry**: plugin tools registered alongside built-in tools
- **SystemPromptBuilder**: prompt fragments appended to system prompt
- **Agent turn loop**: hooks dispatched at PreToolUse/PostToolUse/Stop points
- **Session lifecycle**: hooks dispatched at SessionStart/SessionEnd
- **Chat API**: skill invocation parsed from user messages

## Alternatives Considered

- **WASM plugins**: Excellent sandboxing, portable. But WASI ecosystem is immature for our needs (filesystem, network, subprocess). Revisit when WASI-P2 stabilizes.
- **Lua/Rhai scripting**: Embedded scripting language. Adds complexity, limited ecosystem. CLI wrappers are simpler and language-agnostic.
- **Moltbot's approach (Node.js skills)**: Skills are Markdown files with embedded CLI instructions. Works well but tightly coupled to Node.js runtime and specific CLI tools. Our approach is language-agnostic.
- **Single monolithic plugin type**: Could treat everything as "just tools." But skills, hooks, and agents are fundamentally different extension points with different interfaces. Distinct types are clearer for plugin authors.
- **MCP for everything**: MCP is great for tool injection but doesn't cover skills, hooks, or agents. Plugin system is the higher-level orchestration layer; MCP is one tool transport (ARAWN-I-0016).

## Implementation Plan

### Phase 1: Core Plugin System (Complete)
1. ~~Define `PluginManifest`, `Skill`, `Hook`, `AgentConfig` structs and TOML parsing~~ ✓
2. ~~Implement `PluginManager` with directory scanning and component loading~~ ✓
3. ~~Implement `CliPluginTool` adapter (stdin/stdout JSON protocol)~~ ✓
4. ~~Implement skill loading, parsing, and invocation~~ ✓
5. ~~Implement hook dispatcher with event matching and shell execution~~ ✓
6. ~~Implement agent spawning from plugin agent configs~~ ✓
7. ~~Wire prompt fragments into `SystemPromptBuilder`~~ ✓
8. ~~Add `notify`-based hot-reload watcher~~ ✓
9. ~~Create example plugin: Journal~~ ✓

### Phase 2: Claude Code Compatibility & Plugin Subscription
10. Migrate manifest from `plugin.toml` to `.claude-plugin/plugin.json` (T-0120)
11. Migrate skills to `skills/<name>/SKILL.md` format (T-0121)
12. Migrate hooks to `hooks/hooks.json` format (T-0122)
13. Migrate agents to markdown format (T-0123)
14. Implement `${CLAUDE_PLUGIN_ROOT}` variable substitution (T-0124)
15. Plugin subscription config and storage (T-0125)
16. Git clone and update for subscribed plugins (T-0126)
17. Auto-update subscribed plugins on startup (T-0127)
18. CLI commands: `arawn plugin add/update/remove/list` (T-0128)
19. Migrate journal example plugin to Claude format (T-0129)