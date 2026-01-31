# Agent Loop

The agentic reasoning cycle that drives Arawn's behavior.

## Overview

The agent loop is a repeated cycle of:
1. Build context (system prompt + history + recall)
2. Call LLM with available tools
3. Execute any tool calls
4. Repeat until LLM responds with text only

## Turn Flow

```rust
pub async fn turn(&self, session: &mut Session, message: &str) -> Result<AgentResponse> {
    // 1. Add user message to history
    session.add_message(Message::user(message));

    // 2. Recall relevant memories
    let recall = self.recall(message).await?;

    // 3. Build request with context
    let request = self.build_request(session, recall);

    // 4. Enter tool loop
    loop {
        let response = self.backend.complete(request).await?;

        match response.content {
            Content::Text(text) => {
                // Done - return final response
                return Ok(AgentResponse { text, tools, usage });
            }
            Content::ToolUse(calls) => {
                // Execute tools, append results, continue loop
                for call in calls {
                    let result = self.execute_tool(call).await?;
                    request.messages.push(result);
                }
            }
        }
    }
}
```

## Context Building

### System Prompt

The system prompt is assembled from:

1. **Bootstrap prompt** — Core identity and behavior
2. **Tool documentation** — Descriptions of available tools
3. **Workspace context** — Current directory, project info
4. **Context preamble** — Optional injected context (e.g., from parent agent)

### Conversation History

Session history includes:
- Previous user messages
- Previous assistant responses
- Tool call records with results

### Memory Recall

Before each turn, relevant memories are retrieved:

```rust
let query = RecallQuery {
    embedding: self.embed(message).await?,
    limit: 5,
    threshold: 0.6,
};
let memories = self.memory_store.recall(query).await?;
```

Recalled memories are injected as context before the user message.

## Tool Execution

When the LLM returns tool calls:

1. **Parse** — Extract tool name and parameters from response
2. **Validate** — Check tool exists and parameters match schema
3. **Execute** — Run tool with parameters and context
4. **Format** — Convert result to message format
5. **Append** — Add tool call and result to conversation

### Tool Context

Tools receive context about their execution environment:

```rust
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: PathBuf,
    pub config: Arc<Config>,
    pub memory_store: Option<Arc<MemoryStore>>,
}
```

## Iteration Limits

The agent loop has safety limits:

| Limit | Default | Purpose |
|-------|---------|---------|
| Max iterations | 25 | Prevent runaway loops |
| Max tool calls per turn | 10 | Rate limit tool usage |
| Tool timeout | 30s | Kill hung tools |

## Streaming

For streaming responses, the loop yields events:

```rust
pub enum AgentEvent {
    Text(String),          // Partial text
    ToolStart { name, id },  // Tool execution starting
    ToolEnd { id, result },  // Tool execution complete
    Done,                    // Turn complete
}
```

## Error Handling

| Error Type | Behavior |
|------------|----------|
| Tool execution failure | Return error message to LLM |
| LLM API error | Bubble up to caller |
| Timeout | Cancel tool, return timeout message |
| Max iterations | Stop loop, return partial response |
