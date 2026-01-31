# Tools Overview

Tools are the capabilities available to Arawn's agent loop.

## Tool Categories

| Category | Tools | Purpose |
|----------|-------|---------|
| **File System** | `file_read`, `file_write`, `glob`, `grep` | File operations and search |
| **Execution** | `shell` | Command execution |
| **Web** | `web_fetch`, `web_search` | Internet access |
| **Memory** | `memory_search`, `note`, `think` | Knowledge management |
| **Orchestration** | `delegate`, `workflow` | Task delegation and pipelines |
| **External** | MCP tools, CLI tools | Plugin-provided capabilities |

## How Tools Work

1. **Registration** — Tools register with the ToolRegistry at startup
2. **Documentation** — Tool schemas sent to LLM for understanding
3. **Selection** — LLM decides which tools to call
4. **Execution** — Agent executes tools with provided parameters
5. **Results** — Tool output returned to LLM for next decision

## Tool Interface

All tools implement the `Tool` trait:

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;  // JSON schema
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
}
```

## Section Contents

- [Built-in Tools](built-in.md) — Core tools shipped with Arawn
- [MCP Integration](mcp.md) — Model Context Protocol tools
- [Custom Tools](custom.md) — Plugin CLI tools and extensions
