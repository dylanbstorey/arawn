# Secret Management

Secure handling of API keys and credentials.

## Secret Sources

Arawn supports multiple secret sources with priority resolution:

| Priority | Source | Syntax | Example |
|----------|--------|--------|---------|
| 1 (highest) | OS Keyring | `$keyring:name` | `$keyring:anthropic_api_key` |
| 2 | Environment | `$env:VAR` | `$env:OPENAI_API_KEY` |
| 3 | File | `$file:path` | `$file:~/.secrets/api_key` |
| 4 (lowest) | Literal | Direct value | `sk-ant-...` |

## OS Keyring

The most secure option — secrets stored in your OS keychain.

### Setting Secrets

```bash
# macOS Keychain
security add-generic-password -s arawn -a anthropic_api_key -w "sk-ant-..."

# Linux (secret-tool)
secret-tool store --label="arawn" service arawn username anthropic_api_key

# Windows (Credential Manager)
# Use Windows Credential Manager UI
```

### Using in Config

```toml
[backends.anthropic]
api_key = "$keyring:anthropic_api_key"
```

### Arawn CLI

```bash
# Store a secret
arawn secret set anthropic_api_key

# List stored secrets
arawn secret list

# Delete a secret
arawn secret delete anthropic_api_key
```

## Environment Variables

Good for CI/CD and containerized deployments.

### Setting

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export GROQ_API_KEY="gsk_..."
```

### Using in Config

```toml
[backends.anthropic]
api_key = "$env:ANTHROPIC_API_KEY"

[backends.groq]
api_key = "$env:GROQ_API_KEY"
```

## File-Based Secrets

For secrets stored in files (e.g., mounted Kubernetes secrets).

### File Format

Secret files should contain only the secret value:

```bash
echo -n "sk-ant-..." > ~/.secrets/anthropic_key
chmod 600 ~/.secrets/anthropic_key
```

### Using in Config

```toml
[backends.anthropic]
api_key = "$file:~/.secrets/anthropic_key"
```

## Resolution Order

When a config value starts with `$`, Arawn resolves it:

1. Parse the reference type (`keyring:`, `env:`, `file:`)
2. Attempt to retrieve from that source
3. On failure, error with clear message

### Error Messages

```
Error: Secret "anthropic_api_key" not found in keyring
Hint: Run 'arawn secret set anthropic_api_key' to store it
```

## Security Best Practices

### Do

- Use keyring for personal machines
- Use environment variables for CI/CD
- Use file-based secrets for Kubernetes
- Restrict file permissions (`chmod 600`)

### Don't

- Commit secrets to version control
- Use literal values in shared configs
- Store secrets in world-readable files
- Echo secrets in shell scripts

## Project vs User Secrets

```
Project config (.arawn/arawn.toml):
  # Reference secrets, don't store them
  api_key = "$env:PROJECT_API_KEY"

User config (~/.arawn/arawn.toml):
  # Safe to use keyring references
  api_key = "$keyring:personal_api_key"
```

## Rotating Secrets

1. Generate new secret from provider
2. Update in keyring/env/file
3. Restart Arawn server
4. Verify with `arawn config show --secrets`

## Troubleshooting

### Keyring Not Available

```
Error: Keyring backend not available
```

**Solution:** Install keyring support for your OS:
- macOS: Built-in
- Linux: `apt install libsecret-1-0` or similar
- Windows: Built-in

### Environment Variable Not Set

```
Error: Environment variable "ANTHROPIC_API_KEY" not set
```

**Solution:** Export the variable in your shell profile.

### File Permission Denied

```
Error: Cannot read secret file: Permission denied
```

**Solution:** Check file ownership and permissions.
