# Plugin System

Arawn supports Claude Code-compatible plugins for extensibility.

## Plugin Structure

```
~/.arawn/plugins/
└── my-plugin/
    ├── plugin.json          # Manifest
    ├── skills/
    │   └── research.md      # Skill with frontmatter
    ├── agents/
    │   └── code-review.md   # Subagent definition
    ├── hooks/
    │   └── pre-shell.sh     # Hook script
    └── cli-tools/
        └── format.sh        # CLI tool wrapper
```

## Plugin Manifest

`plugin.json` defines the plugin:

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My custom plugin",
  "author": "Your Name"
}
```

## Plugin Components

### Skills

Reusable prompts with YAML frontmatter:

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

Subagent definitions:

```markdown
---
name: code-reviewer
description: Code review specialist
model: sonnet
tools: ["file_read", "grep", "glob"]
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

Event handlers:

```bash
#!/bin/bash
# hooks/pre-shell.sh
#
# Runs before shell commands

input=$(cat)
command=$(echo "$input" | jq -r '.command')

# Block dangerous commands
if [[ "$command" == *"rm -rf"* ]]; then
  echo '{"blocked": true, "reason": "Destructive command blocked"}'
else
  echo '{"blocked": false}'
fi
```

### CLI Tools

External tools exposed to the agent:

```bash
#!/bin/bash
# cli-tools/format.sh

input=$(cat)
file=$(echo "$input" | jq -r '.file')

prettier --write "$file"
echo "{\"success\": true, \"formatted\": \"$file\"}"
```

## Plugin Loading

Plugins are loaded from:

1. `~/.arawn/plugins/` — User plugins
2. `$XDG_CONFIG_HOME/arawn/plugins/` — XDG plugins
3. `.arawn/plugins/` — Project-local plugins

### Load Order

1. Scan plugin directories
2. Parse `plugin.json` manifests
3. Load skills, agents, hooks, tools
4. Register with appropriate managers

## CLI Commands

```bash
# List installed plugins
arawn plugin list

# Plugin details
arawn plugin info my-plugin

# Install from path
arawn plugin install /path/to/plugin

# Uninstall
arawn plugin uninstall my-plugin
```

## Plugin Development

### Creating a Plugin

```bash
mkdir -p ~/.arawn/plugins/my-plugin
cd ~/.arawn/plugins/my-plugin

# Create manifest
cat > plugin.json << 'EOF'
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My first plugin"
}
EOF

# Create a skill
mkdir skills
cat > skills/helper.md << 'EOF'
---
name: helper
description: General helper
---

You are a helpful assistant.
EOF
```

### Testing

```bash
# Verify plugin loads
arawn plugin list

# Test a skill
arawn skill invoke my-plugin/helper

# Test an agent
arawn agent info my-plugin/code-reviewer
```

## Best Practices

1. **Single Responsibility** — Each plugin should have a focused purpose
2. **Clear Documentation** — Write good descriptions for skills and agents
3. **Minimal Tools** — Grant agents only the tools they need
4. **Error Handling** — CLI tools should handle errors gracefully
5. **Version Control** — Track plugins in git for reproducibility
