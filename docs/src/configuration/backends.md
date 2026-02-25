# LLM Backends

Configuration for LLM providers.

## Supported Providers

| Provider | Backend Key | Capabilities |
|----------|-------------|--------------|
| Anthropic | `anthropic` | Tool calling, streaming, vision |
| OpenAI | `openai` | Tool calling, function_call, embeddings |
| Groq | `groq` | Fast inference, OpenAI-compatible |
| Ollama | `ollama` | Local LLMs, no rate limits |
| Custom | `custom` | OpenAI-compatible endpoint with custom base URL |
| Claude OAuth | `claude-oauth` | Anthropic via OAuth PKCE (Claude MAX) |

## Anthropic

Claude models from Anthropic.

```toml
[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000
```

API key resolved via keyring → `ANTHROPIC_API_KEY` env var → config file. See
[Secrets](secrets.md).

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
[llm.openai]
backend = "openai"
model = "gpt-4o"
max_context_tokens = 128000
```

API key resolved via keyring → `OPENAI_API_KEY` env var → config file.

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
[embedding]
provider = "openai"

[embedding.openai]
model = "text-embedding-3-small"
```

## Groq

Fast inference on open-source models (OpenAI-compatible API).

```toml
[llm.fast]
backend = "groq"
model = "llama-3.3-70b-versatile"
max_context_tokens = 32768
```

API key resolved via keyring → `GROQ_API_KEY` env var → config file.

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
[llm.local]
backend = "ollama"
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

## Custom

Any OpenAI-compatible endpoint.

```toml
[llm.custom]
backend = "custom"
base_url = "https://my-endpoint.example.com/v1"
model = "my-model"
max_context_tokens = 32768
```

API key resolved via keyring → `LLM_API_KEY` env var → config file.

## Claude OAuth

Authenticate via OAuth PKCE flow for Claude MAX subscriptions. No API key
needed — uses browser-based authentication.

```toml
[llm]
backend = "claude-oauth"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000
```

Run `arawn auth` to complete the OAuth flow.

## Multiple Backends

Configure named LLM profiles for different purposes:

```toml
# Default backend for all operations
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000

# Named profiles for specific uses
[llm.fast]
backend = "groq"
model = "llama-3.3-70b-versatile"
max_context_tokens = 32768

[llm.local]
backend = "ollama"
base_url = "http://localhost:11434"
model = "llama3"

# Assign profiles to agents
[agent.default]
llm = "claude"

[agent.summarizer]
llm = "fast"

# Memory indexing uses its own backend
[memory.indexing]
backend = "openai"
model = "gpt-4o-mini"

# Local embeddings
[embedding]
provider = "local"
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
1. Key is set: `echo $ANTHROPIC_API_KEY`
2. Keyring has entry (see [Secrets](secrets.md))
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
