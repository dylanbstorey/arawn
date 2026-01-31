# LLM Backends

Configuration for LLM providers.

## Supported Providers

| Provider | Backend Key | Capabilities |
|----------|-------------|--------------|
| Anthropic | `anthropic` | Tool calling, streaming, vision |
| OpenAI | `openai` | Tool calling, function_call, embeddings |
| Groq | `groq` | Fast inference, OpenAI-compatible |
| Ollama | `ollama` | Local LLMs, no rate limits |

## Anthropic

Claude models from Anthropic.

```toml
[backends.anthropic]
api_key = "$keyring:anthropic_api_key"
model = "claude-sonnet-4-20250514"
max_tokens = 4096
```

### Available Models

| Model | Best For |
|-------|----------|
| `claude-sonnet-4-20250514` | Balanced capability and cost |
| `claude-3-haiku-20240307` | Fast, cost-effective tasks |
| `claude-opus-4-20250514` | Complex reasoning |

### Features

- Native tool calling
- Streaming responses
- Vision (image analysis)
- Long context (200k tokens)

## OpenAI

GPT models from OpenAI.

```toml
[backends.openai]
api_key = "$env:OPENAI_API_KEY"
model = "gpt-4o"
max_tokens = 4096
```

### Available Models

| Model | Best For |
|-------|----------|
| `gpt-4o` | Balanced multimodal |
| `gpt-4o-mini` | Cost-effective tasks |
| `gpt-4-turbo` | High capability |

### Features

- Function calling
- Streaming responses
- Vision (with vision models)
- Embeddings (separate endpoint)

### Embeddings

```toml
[embeddings.openai]
api_key = "$env:OPENAI_API_KEY"
model = "text-embedding-3-small"
```

## Groq

Fast inference on open-source models.

```toml
[backends.groq]
api_key = "$env:GROQ_API_KEY"
model = "llama-3.3-70b-versatile"
max_tokens = 4096
```

### Available Models

| Model | Best For |
|-------|----------|
| `llama-3.3-70b-versatile` | General purpose |
| `mixtral-8x7b-32768` | Long context tasks |
| `llama-3.1-8b-instant` | Fast responses |

### Features

- OpenAI-compatible API
- Very fast inference
- Tool calling support
- Streaming responses

## Ollama

Local LLM inference.

```toml
[backends.ollama]
base_url = "http://localhost:11434"
model = "llama3"
```

### Setup

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama3

# Start server (usually automatic)
ollama serve
```

### Available Models

Any model available in Ollama:
- `llama3`, `llama3:70b`
- `codellama`
- `mistral`
- `mixtral`
- Custom fine-tuned models

### Features

- No API key required
- No rate limits
- Full privacy (local)
- Custom models supported

## Multiple Backends

Configure multiple backends for different purposes:

```toml
# Primary for chat
[llm]
backend = "groq"
model = "llama-3.3-70b"

# Fast model for indexing
[memory.indexing]
backend = "anthropic"
model = "claude-3-haiku-20240307"

# Local embeddings
[embeddings]
backend = "local"
model = "all-MiniLM-L6-v2"

# Backend definitions
[backends.anthropic]
api_key = "$keyring:anthropic"
model = "claude-sonnet-4-20250514"

[backends.groq]
api_key = "$env:GROQ_API_KEY"
model = "llama-3.3-70b-versatile"
```

## Switching Backends

### Via Config

Change `[llm].backend` in your config file.

### Via CLI

```bash
arawn --backend anthropic chat
arawn --backend ollama ask "Hello"
```

### Via API

```json
{
  "message": "Hello",
  "backend": "anthropic"
}
```

## Backend Selection Strategy

| Use Case | Recommended |
|----------|-------------|
| Development | Ollama (free, private) |
| Quick tasks | Groq (fast) |
| Complex reasoning | Anthropic Claude |
| Cost optimization | OpenAI gpt-4o-mini |
| Indexing | Anthropic Haiku |
| Embeddings | Local ONNX |

## Troubleshooting

### API Key Issues

```
Error: Authentication failed for backend "anthropic"
```

Check:
1. Key is set correctly: `arawn config show --secrets`
2. Key has correct permissions
3. Key hasn't expired

### Model Not Available

```
Error: Model "claude-opus-4-20250514" not available
```

Check:
1. Model name is spelled correctly
2. Your API plan includes the model
3. Model is available in your region

### Ollama Connection Failed

```
Error: Cannot connect to Ollama at http://localhost:11434
```

Check:
1. Ollama is running: `ollama list`
2. Port is correct
3. No firewall blocking
