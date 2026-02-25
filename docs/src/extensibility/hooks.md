# Hooks

Event-driven extensibility for Arawn.

## Overview

Hooks allow plugins to:
- Intercept and block tool executions (PreToolUse)
- React to tool results and session events
- Monitor subagent lifecycle
- Inject context at session start

## Hook Events

All 13 lifecycle events supported by the hook system:

| Event | Trigger | Can Block |
|-------|---------|-----------|
| `PreToolUse` | Before tool execution | Yes |
| `PostToolUse` | After successful tool execution | No |
| `PostToolUseFailure` | After failed tool execution | No |
| `PermissionRequest` | When a permission request is made | No |
| `UserPromptSubmit` | When the user submits a prompt | No |
| `Notification` | When a notification is sent | No |
| `SubagentStop` | When a subagent stops | No |
| `PreCompact` | Before context compaction | No |
| `SessionStart` | When a session begins | No |
| `SessionEnd` | When a session ends | No |
| `Stop` | When the agent produces a final response | No |
| `SubagentStarted` | When a background subagent starts | No |
| `SubagentCompleted` | When a background subagent finishes | No |

Only `PreToolUse` hooks can block execution. All other events are informational.

## Hook Configuration

Hooks are configured in a `hooks.json` file referenced from the plugin manifest:

```
my-plugin/
├── .claude-plugin/
│   └── plugin.json          # "hooks": "./hooks/hooks.json"
└── hooks/
    ├── hooks.json           # Hook configuration
    └── validate-shell.sh    # Hook script
```

### hooks.json Format

The configuration uses Claude Code's hooks format — a map of event names to
arrays of matcher groups:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "shell",
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/hooks/validate-shell.sh",
            "timeout": 5000
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/hooks/audit-log.sh"
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/hooks/session-init.sh"
          }
        ]
      }
    ]
  }
}
```

### Matcher Groups

Each event maps to an array of matcher groups. A matcher group contains:

| Field | Description |
|-------|-------------|
| `matcher` | Optional regex to match tool names (for `PreToolUse`/`PostToolUse`) |
| `hooks` | Array of hook actions to execute |

If `matcher` is omitted, the hook fires for all tools (or unconditionally for
non-tool events).

### Hook Actions

Each hook action specifies:

| Field | Description |
|-------|-------------|
| `type` | `"command"` (shell command), `"prompt"` (LLM eval), or `"agent"` (agentic verifier) |
| `command` | Shell command to execute (for `command` type) |
| `prompt` | Prompt text (for `prompt` type) |
| `agent` | Agent name (for `agent` type) |
| `timeout` | Timeout in milliseconds (optional) |

> **Note:** Currently only `command` type hooks are executed. `prompt` and
> `agent` types are recognized but not yet implemented.

### Variable Expansion

Use `${CLAUDE_PLUGIN_ROOT}` in command paths to reference the plugin root
directory. The environment variable `ARAWN_PLUGIN_DIR` is also set for
subprocess execution.

## Hook Implementation

### PreToolUse (Blocking)

PreToolUse hooks can block tool execution. Exit code 0 means allow; non-zero
means block. Stdout from a blocking hook becomes the block reason.

```bash
#!/bin/bash
# hooks/validate-shell.sh
# Receives JSON context on stdin: {"tool": "shell", "params": {...}}

input=$(cat)
command=$(echo "$input" | jq -r '.params.command // .params.cmd // ""')

# Block dangerous commands
if echo "$command" | grep -qE 'rm -rf /|dd if=|mkfs'; then
  echo "Destructive command blocked by policy"
  exit 1
fi

exit 0
```

### PostToolUse (Informational)

Fires after successful tool execution. Non-zero exit is logged but does not
block anything.

```bash
#!/bin/bash
# hooks/audit-log.sh
# Receives: {"tool": "shell", "params": {...}, "result": {...}}

input=$(cat)
echo "$input" >> /var/log/arawn-audit.jsonl
```

### SessionStart

Fires when a new session begins. Stdout is returned as informational output.

```bash
#!/bin/bash
# hooks/session-init.sh
# Receives: {"session_id": "abc123"}

session_id=$(echo "$(cat)" | jq -r '.session_id')
mkdir -p "/tmp/arawn-sessions/$session_id"
echo "Session workspace initialized"
```

### SessionEnd

Fires when a session ends.

```bash
#!/bin/bash
# hooks/session-cleanup.sh
# Receives: {"session_id": "abc123", "turn_count": 5}

input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id')
rm -rf "/tmp/arawn-sessions/$session_id"
```

### Stop

Fires when the agent produces a final response.

```bash
#!/bin/bash
# hooks/on-stop.sh
# Receives: {"response": "Here is the answer..."}

cat > /dev/null  # consume stdin
echo "Response logged"
```

### SubagentStarted / SubagentCompleted

Fire when background subagents start and finish:

```bash
#!/bin/bash
# hooks/subagent-monitor.sh
# SubagentStarted receives: {"parent_session_id": "...", "subagent_name": "...", "task_preview": "..."}
# SubagentCompleted receives: {"parent_session_id": "...", "subagent_name": "...", "result_preview": "...", "duration_ms": 1500, "success": true}

cat > /dev/null
```

## Hook Input Format

Hooks receive JSON on stdin. The shape depends on the event:

### PreToolUse

```json
{"tool": "shell", "params": {"command": "ls -la"}}
```

### PostToolUse

```json
{"tool": "shell", "params": {"command": "ls -la"}, "result": {"output": "..."}}
```

### SessionStart

```json
{"session_id": "abc123"}
```

### SessionEnd

```json
{"session_id": "abc123", "turn_count": 5}
```

### Stop

```json
{"response": "Here is the final answer..."}
```

### SubagentStarted

```json
{"parent_session_id": "sess-1", "subagent_name": "researcher", "task_preview": "Find papers on RAG"}
```

### SubagentCompleted

```json
{"parent_session_id": "sess-1", "subagent_name": "researcher", "result_preview": "Found 5 papers", "duration_ms": 1500, "success": true}
```

## Execution Details

- **Timeout**: Default 10 seconds per hook subprocess. Override with `timeout`
  field in the hook action.
- **Working directory**: Set to the plugin directory.
- **Parallel execution**: Hooks for the same event run sequentially. PreToolUse
  stops at the first blocker.
- **Error handling**: Subprocess failures are logged. For informational hooks,
  errors don't affect the outcome.

## Best Practices

1. **Keep hooks fast** — Slow hooks delay tool execution
2. **Handle errors gracefully** — Return valid output even on failure
3. **Log to stderr** — Debug output goes to stderr, which is captured by tracing
4. **Use blocking sparingly** — Only block when necessary (PreToolUse only)
5. **Match specifically** — Use precise matchers to avoid over-matching
