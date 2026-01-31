# Custom Tools

Extending Arawn with plugin-provided tools.

## CLI Tools

Plugins can provide CLI tools that accept JSON input and return JSON output.

### Tool Structure

```
my-plugin/
├── plugin.json
└── cli-tools/
    ├── format.sh
    └── validate.js
```

### Tool Definition

In `plugin.json`:

```json
{
  "name": "my-plugin",
  "cli_tools": [
    {
      "name": "format",
      "path": "cli-tools/format.sh",
      "description": "Format code files",
      "parameters": {
        "type": "object",
        "properties": {
          "file": { "type": "string", "description": "File to format" },
          "language": { "type": "string", "description": "Language hint" }
        },
        "required": ["file"]
      }
    }
  ]
}
```

### Tool Implementation

Tools receive JSON on stdin and write JSON to stdout.

**Shell Script Example (format.sh):**

```bash
#!/bin/bash
# Read JSON input
input=$(cat)

# Extract parameters
file=$(echo "$input" | jq -r '.file')
language=$(echo "$input" | jq -r '.language // "auto"')

# Do the work
result=$(prettier --write "$file" 2>&1)

# Return JSON output
echo "{\"success\": true, \"message\": \"Formatted $file\", \"output\": \"$result\"}"
```

**Node.js Example (validate.js):**

```javascript
#!/usr/bin/env node
const fs = require('fs');

// Read JSON from stdin
let input = '';
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  const params = JSON.parse(input);

  // Do validation
  const content = fs.readFileSync(params.file, 'utf8');
  const isValid = content.includes('TODO') === false;

  // Return result
  console.log(JSON.stringify({
    valid: isValid,
    issues: isValid ? [] : ['Contains TODO comments']
  }));
});
```

## Tool Registration

CLI tools are registered when plugins load:

1. Plugin manager scans `cli-tools/` directory
2. Tool schemas parsed from `plugin.json`
3. Tools wrapped as `CliTool` implementing `Tool` trait
4. Added to agent's `ToolRegistry`

## Execution Flow

```
LLM decides to call "my-plugin__format"
     │
     ▼
┌─────────────────────────────────────┐
│ CliTool.execute()                   │
│                                     │
│ 1. Serialize params to JSON         │
│ 2. Spawn subprocess                 │
│ 3. Write JSON to stdin              │
│ 4. Wait for completion              │
│ 5. Read JSON from stdout            │
│ 6. Return as ToolResult             │
└─────────────────────────────────────┘
```

## Tool Naming

Plugin tools are namespaced:

- **Pattern:** `{plugin_name}__{tool_name}`
- **Example:** `my-plugin__format`

## Best Practices

### Input Validation

Always validate input before processing:

```bash
#!/bin/bash
input=$(cat)

# Validate required fields
file=$(echo "$input" | jq -r '.file')
if [ -z "$file" ] || [ "$file" = "null" ]; then
  echo '{"error": "file parameter is required"}'
  exit 0  # Exit 0, error in JSON
fi
```

### Error Handling

Return errors in JSON, not via exit codes:

```javascript
try {
  // ... do work
  console.log(JSON.stringify({ success: true, result }));
} catch (error) {
  console.log(JSON.stringify({
    success: false,
    error: error.message
  }));
}
```

### Timeout Awareness

CLI tools have a 30-second default timeout. For long operations:

```json
{
  "name": "long-task",
  "timeout": 120,
  "path": "cli-tools/long-task.sh"
}
```

### Security

- Validate all file paths
- Don't execute arbitrary commands
- Sanitize user input
- Use absolute paths when possible

## Testing Tools

Test tools manually:

```bash
echo '{"file": "src/main.rs"}' | ./cli-tools/format.sh
```

Or via Arawn:

```bash
arawn tool call my-plugin__format '{"file": "src/main.rs"}'
```

## Debugging

Enable tool debug logging:

```bash
RUST_LOG=arawn_plugin=debug arawn chat
```

Check plugin loading:

```bash
arawn plugin list --verbose
```
