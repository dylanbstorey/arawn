# Built-in Tools

Core tools shipped with Arawn.

## File System Tools

### file_read

Read file contents for analysis.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | Yes | File path to read |
| `start_line` | number | No | Starting line (1-indexed) |
| `end_line` | number | No | Ending line |

**Example:**
```json
{"path": "/src/main.rs", "start_line": 1, "end_line": 50}
```

### file_write

Write or create files.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | Yes | File path to write |
| `content` | string | Yes | Content to write |

**Example:**
```json
{"path": "/tmp/output.txt", "content": "Hello, World!"}
```

### glob

Find files by pattern.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pattern` | string | Yes | Glob pattern (e.g., `**/*.rs`) |
| `path` | string | No | Base directory |

**Example:**
```json
{"pattern": "src/**/*.rs"}
```

### grep

Search file contents.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `pattern` | string | Yes | Regex pattern |
| `path` | string | No | File/directory to search |
| `glob` | string | No | File pattern filter |

**Example:**
```json
{"pattern": "fn main", "glob": "*.rs"}
```

## Execution Tools

### shell

Execute shell commands.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `command` | string | Yes | Command to execute |
| `timeout` | number | No | Timeout in seconds (default: 30) |

**Example:**
```json
{"command": "cargo build --release", "timeout": 120}
```

**Safety Notes:**
- Commands run in a sandboxed environment
- Destructive commands require confirmation
- Network access may be restricted

## Web Tools

### web_fetch

Retrieve content from URLs.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `url` | string | Yes | URL to fetch |
| `download` | boolean | No | Save to file instead of returning |
| `output_path` | string | No | Path for downloaded file |

**Example:**
```json
{"url": "https://docs.rs/tokio/latest/tokio/"}
```

### web_search

Search the internet.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `query` | string | Yes | Search query |
| `limit` | number | No | Max results (default: 5) |

**Example:**
```json
{"query": "rust async trait implementation", "limit": 3}
```

## Memory Tools

### memory_search

Query persistent memory.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `query` | string | Yes | Search query |
| `limit` | number | No | Max results |
| `threshold` | number | No | Min confidence (0-1) |

**Example:**
```json
{"query": "user preferences", "limit": 5}
```

### note

Create session-scoped notes.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `title` | string | Yes | Note title |
| `content` | string | Yes | Note content |

**Example:**
```json
{"title": "TODO", "content": "- Fix auth bug\n- Update docs"}
```

### think

Record internal reasoning.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `thought` | string | Yes | Reasoning to record |

**Example:**
```json
{"thought": "The user prefers explicit error handling over unwrap()"}
```

## Orchestration Tools

### catalog

Browse available agents, skills, and tools.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `category` | string | No | Filter: `agents`, `skills`, `tools` |

**Example:**
```json
{"category": "agents"}
```

### delegate

Delegate to specialized subagents.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `agent` | string | Yes | Subagent name |
| `task` | string | Yes | Task description |
| `context` | string | No | Context from parent |
| `background` | boolean | No | Run in background (default: false) |
| `max_turns` | integer | No | Max turn limit for subagent |

**Example:**
```json
{
  "agent": "researcher",
  "task": "Find papers on transformer architectures",
  "context": "User building a RAG system",
  "background": false
}
```

### workflow

Execute defined pipelines.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `workflow` | string | Yes | Workflow name |
| `inputs` | object | No | Input parameters |

**Example:**
```json
{"workflow": "deploy", "inputs": {"environment": "staging"}}
```

## Tool Result Types

Tools return one of:

```rust
pub enum ToolResult {
    Text(String),           // Plain text output
    Json(Value),            // Structured data
    Error(String),          // Error message
    Binary { path: PathBuf, mime: String },  // File reference
}
```
