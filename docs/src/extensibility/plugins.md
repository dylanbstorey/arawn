# Plugin System

Arawn supports Claude Code-compatible plugins for extensibility.

## Plugin Structure

A plugin is a directory containing a `.claude-plugin/plugin.json` manifest and
component subdirectories:

```
my-plugin/
├── .claude-plugin/
│   └── plugin.json       # Manifest (required)
├── skills/
│   └── research/
│       └── SKILL.md      # Skill definition
├── agents/
│   └── code-review.md    # Agent definition
├── hooks/
│   └── hooks.json        # Hook configuration
└── tools/
    └── format/
        └── tool.json     # CLI tool definition
```

## Plugin Manifest

The manifest at `.claude-plugin/plugin.json` declares the plugin's metadata and
component paths:

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My custom plugin",
  "author": {
    "name": "Your Name",
    "email": "you@example.com"
  },
  "homepage": "https://example.com/my-plugin",
  "repository": "https://github.com/you/my-plugin",
  "license": "MIT",
  "keywords": ["example"],
  "skills": "./skills/",
  "agents": "./agents/",
  "hooks": "./hooks/hooks.json",
  "commands": "./tools/"
}
```

### Manifest Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier (kebab-case, starts with letter) |
| `version` | No | Semantic version (e.g., `"1.0.0"`) |
| `description` | No | Human-readable description |
| `author` | No | Author object (`name`, optional `email`, `url`) |
| `homepage` | No | Documentation URL |
| `repository` | No | Source repository URL |
| `license` | No | SPDX license identifier |
| `keywords` | No | Discovery keywords |
| `skills` | No | Path or array of paths to skills directories |
| `agents` | No | Path or array of paths to agent files |
| `hooks` | No | Path to `hooks.json` config |
| `commands` | No | Path or array of paths to CLI tool directories |
| `mcpServers` | No | Inline MCP server config or path to `.mcp.json` |

Path fields accept a single string or an array of strings:

```json
{
  "skills": ["./skills/", "./extra-skills/"]
}
```

## Plugin Components

### Skills

Reusable prompts with YAML frontmatter. Discovered from `skills/<name>/SKILL.md`:

```markdown
---
name: research
description: Deep research on a topic
---

You are a research specialist. When given a topic:

1. Search for authoritative sources
2. Cross-reference information
3. Synthesize findings
4. Cite your sources

Focus on accuracy over speed.
```

### Agents

Subagent definitions parsed from `agents/<name>.md` with YAML frontmatter:

```markdown
---
name: code-reviewer
description: Code review specialist
model: claude-sonnet
tools:
  - file_read
  - grep
  - glob
max_iterations: 15
---

You are a code review specialist. Analyze code for:

- Security vulnerabilities
- Performance issues
- Code style violations
- Logic errors

Provide specific, actionable feedback.
```

### Hooks

Event-driven handlers configured via `hooks.json`. See [Hooks](hooks.md) for
full documentation.

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "shell",
        "hooks": [
          {
            "type": "command",
            "command": "./hooks/validate-shell.sh"
          }
        ]
      }
    ]
  }
}
```

### CLI Tools

External executables exposed to the agent as tools. Each tool is a directory
containing a `tool.json` manifest:

```json
{
  "name": "format",
  "description": "Format a source file",
  "command": "./format.sh",
  "parameters": {
    "type": "object",
    "properties": {
      "file": { "type": "string", "description": "File path to format" }
    }
  }
}
```

Tools receive JSON parameters on stdin and return a JSON response on stdout:

```json
{"success": true, "content": "Formatted file.rs"}
```

Or on error:

```json
{"error": "File not found"}
```

## Plugin Loading

Plugins are scanned from these directories (in order):

1. `$XDG_CONFIG_HOME/arawn/plugins/` — User-level plugins
2. `./plugins/` — Project-local plugins
3. Additional directories from `[plugins].dirs` in config

### Load Process

1. Scan each directory for subdirectories containing `.claude-plugin/plugin.json`
2. Parse and validate the manifest (name format, version, path existence)
3. Discover skills, agents, hooks, and CLI tools from declared paths
4. Register components with their respective managers

### Configuration

```toml
[plugins]
enabled = true
dirs = ["~/.config/arawn/plugins", "./plugins"]
hot_reload = true
```

## Plugin Development

### Creating a Plugin

```bash
mkdir -p my-plugin/.claude-plugin
cd my-plugin

# Create manifest
cat > .claude-plugin/plugin.json << 'EOF'
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My first plugin",
  "skills": "./skills/"
}
EOF

# Create a skill
mkdir -p skills/helper
cat > skills/helper/SKILL.md << 'EOF'
---
name: helper
description: General helper
---

You are a helpful assistant.
EOF
```

### Validation

The manifest is validated on load:

- **Name**: Must be kebab-case, start with a letter, no consecutive hyphens
- **Version**: Must be semver if provided (e.g., `1.0.0`, `1.0.0-alpha`)
- **Paths**: Declared component directories must exist on disk

## Best Practices

1. **Single Responsibility** — Each plugin should have a focused purpose
2. **Clear Documentation** — Write good descriptions for skills and agents
3. **Minimal Tools** — Grant agents only the tools they need
4. **Error Handling** — CLI tools should return valid JSON even on failure
5. **Version Control** — Track plugins in git for reproducibility
