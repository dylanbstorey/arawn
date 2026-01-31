# Hooks

Event-driven extensibility for Arawn.

## Overview

Hooks allow plugins to:
- Intercept tool executions
- React to session events
- Monitor subagent lifecycle
- Implement cross-cutting concerns

## Hook Events

| Event | Trigger | Can Block |
|-------|---------|-----------|
| `PreToolUse` | Before tool execution | Yes |
| `PostToolUse` | After tool execution | No |
| `SessionStart` | Session begins | No |
| `SessionEnd` | Session closes | No |
| `SubagentStarted` | Subagent spawned | No |
| `SubagentCompleted` | Subagent finished | No |

## Hook Configuration

Hooks are defined in plugin directories:

```
my-plugin/
├── plugin.json
└── hooks/
    ├── pre-shell.sh       # Matches: PreToolUse + shell
    ├── post-file-write.sh # Matches: PostToolUse + file_write
    └── session-start.sh   # Matches: SessionStart
```

### Naming Convention

Hook files follow the pattern: `{event}-{tool}.{ext}`

Examples:
- `pre-shell.sh` — Before shell commands
- `post-file-write.js` — After file writes
- `session-start.sh` — When sessions start

### Supported Extensions

| Extension | Runtime |
|-----------|---------|
| `.sh` | Bash |
| `.js` | Node.js |
| `.json` | Static response |

## Hook Implementation

### PreToolUse Hooks

Can block tool execution:

```bash
#!/bin/bash
# hooks/pre-shell.sh

input=$(cat)
command=$(echo "$input" | jq -r '.params.command')

# Block dangerous commands
if [[ "$command" == *"rm -rf /"* ]]; then
  echo '{"blocked": true, "reason": "Blocked: destructive root command"}'
else
  echo '{"blocked": false}'
fi
```

### PostToolUse Hooks

React to tool results:

```bash
#!/bin/bash
# hooks/post-file-write.sh

input=$(cat)
path=$(echo "$input" | jq -r '.params.path')

# Log file writes
logger "Arawn wrote file: $path"

# PostToolUse hooks don't return blocking decisions
echo '{}'
```

### Session Hooks

React to session lifecycle:

```bash
#!/bin/bash
# hooks/session-start.sh

input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id')

# Initialize session-specific resources
mkdir -p "/tmp/arawn-sessions/$session_id"

echo '{}'
```

## Hook Input Format

Hooks receive JSON on stdin:

### PreToolUse Input

```json
{
  "event": "PreToolUse",
  "tool": "shell",
  "params": {
    "command": "ls -la"
  },
  "session_id": "abc123",
  "timestamp": "2024-01-15T10:00:00Z"
}
```

### PostToolUse Input

```json
{
  "event": "PostToolUse",
  "tool": "shell",
  "params": {
    "command": "ls -la"
  },
  "result": {
    "success": true,
    "output": "total 42\n..."
  },
  "session_id": "abc123",
  "duration_ms": 150
}
```

## Hook Output Format

### Blocking Response (PreToolUse only)

```json
{
  "blocked": true,
  "reason": "Command not allowed in this context"
}
```

### Allow Response

```json
{
  "blocked": false
}
```

### Empty Response (PostToolUse, Session events)

```json
{}
```

## Hook Dispatcher

The HookDispatcher manages hook execution:

```
┌─────────────────────────────────────────────────────────────────┐
│ HookDispatcher                                                   │
│                                                                  │
│  matchers: Vec<Hook>                                             │
│                                                                  │
│  Events:                                                         │
│  • PreToolUse  ─────▶ Check all matching hooks                  │
│  • PostToolUse ─────▶ Fire and forget                           │
│  • SessionStart ────▶ Fire and forget                           │
│  • SessionEnd ──────▶ Fire and forget                           │
│  • SubagentStarted ─▶ Fire and forget                           │
│  • SubagentCompleted▶ Fire and forget                           │
└─────────────────────────────────────────────────────────────────┘
```

## Use Cases

### Security Enforcement

Block dangerous operations:

```bash
#!/bin/bash
# pre-shell.sh - Block network tools

command=$(cat | jq -r '.params.command')

if echo "$command" | grep -qE '(curl|wget|nc|netcat)'; then
  echo '{"blocked": true, "reason": "Network commands not allowed"}'
else
  echo '{"blocked": false}'
fi
```

### Audit Logging

Log all tool executions:

```bash
#!/bin/bash
# post-any.sh - Log everything

input=$(cat)
echo "$input" >> /var/log/arawn-audit.json
echo '{}'
```

### Resource Management

Clean up on session end:

```bash
#!/bin/bash
# session-end.sh

session_id=$(cat | jq -r '.session_id')
rm -rf "/tmp/arawn-sessions/$session_id"
echo '{}'
```

## Best Practices

1. **Keep hooks fast** — Slow hooks delay tool execution
2. **Handle errors gracefully** — Return valid JSON even on failure
3. **Log for debugging** — Write to stderr for debug output
4. **Use blocking sparingly** — Only block when necessary
5. **Match specifically** — Don't over-match with broad patterns
