# Secret Management

Secure handling of API keys and credentials.

## Resolution Chain

Arawn resolves API keys using a priority chain. The first source that returns
a value wins:

| Priority | Source | How |
|----------|--------|-----|
| 1 (highest) | System keyring | Looked up automatically by backend name |
| 2 | Environment variable | Backend-specific env var (e.g., `ANTHROPIC_API_KEY`) |
| 3 (lowest) | Config file | Plaintext value in TOML (not recommended) |

There is no prefix syntax — Arawn checks each source in order and uses the
first match. You do not need to annotate values with `$keyring:` or `$env:`.

## System Keyring

The most secure option. Arawn uses the OS keychain via the `keyring` crate:

- **macOS**: Keychain Access (built-in)
- **Linux**: Secret Service API (`libsecret`)
- **Windows**: Windows Credential Manager

### Storing a Key

Keyring entries use service `"arawn"` and a user derived from the backend name.

**macOS:**

```bash
security add-generic-password -s arawn -a anthropic_api_key -w "sk-ant-..."
```

**Linux (secret-tool):**

```bash
secret-tool store --label="arawn" service arawn username anthropic_api_key
```

Arawn will find these automatically — no config file entry needed.

### Keyring Feature

Keyring support requires the `keyring` Cargo feature (enabled by default in
release builds). Without it, step 1 is skipped and resolution falls through to
environment variables.

## Environment Variables

Standard for CI/CD, containers, and serverless environments.

| Backend | Environment Variable |
|---------|---------------------|
| Anthropic | `ANTHROPIC_API_KEY` |
| OpenAI | `OPENAI_API_KEY` |
| Groq | `GROQ_API_KEY` |
| Ollama | `OLLAMA_API_KEY` |
| Custom | `LLM_API_KEY` |
| Claude OAuth | `ANTHROPIC_API_KEY` |

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export GROQ_API_KEY="gsk_..."
```

## Config File (Fallback)

If neither keyring nor environment variable provides a key, Arawn falls back to
the value in the TOML config file. This is **not recommended** for shared or
version-controlled configs.

```toml
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
api_key = "sk-ant-..."   # Plaintext — avoid if possible
```

Arawn logs a warning when a plaintext API key is loaded from the config file.

## Security Best Practices

### Do

- Use the system keyring on personal machines
- Use environment variables in CI/CD and containers
- Keep config files out of version control (or use `.gitignore`)
- Restrict file permissions on any config containing keys (`chmod 600`)

### Don't

- Commit API keys to version control
- Use plaintext values in shared configs
- Store secrets in world-readable files

## Troubleshooting

### Keyring Not Available

If the keyring crate can't access the OS keychain, Arawn silently falls through
to environment variables. To verify keyring is working:

```bash
# macOS — list Arawn entries
security find-generic-password -s arawn

# Linux — check secret-service
secret-tool search service arawn
```

### No API Key Found

If none of the three sources provides a key, Arawn returns an error when the
LLM backend is invoked. Verify your setup:

1. Check if the key is in the keyring
2. Check if the environment variable is set: `echo $ANTHROPIC_API_KEY`
3. Check `~/.arawn/arawn.toml` for a fallback value
