# Installation

Arawn is designed for edge computing — a single binary with embedded storage.

## Install Script

The quickest way to install Arawn and its dependencies:

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/arawn/main/scripts/install.sh | sh
```

This will:
- Download the latest release binary from GitHub
- Install sandbox dependencies on Linux (bubblewrap, socat)
- macOS sandbox support is built-in — no extra dependencies needed

Options:

```bash
# Install a specific version
./install.sh --version v0.1.0

# Install to a custom directory
./install.sh --install-dir /usr/local/bin

# Skip sandbox dependency installation
./install.sh --skip-deps

# Preview what would happen
./install.sh --dry-run
```

## Building from Source

### Prerequisites

- Rust 1.85+ with cargo
- C compiler (for SQLite bindings)
- Linux only: bubblewrap and socat for sandboxed shell execution
- Optional: ONNX Runtime for local embeddings

### Basic Build

```bash
# Clone the repository
git clone https://github.com/colliery-io/arawn.git
cd arawn

# Build release binary
cargo build --release

# Binary location
./target/release/arawn
```

### With Local Embeddings

Enable ONNX features for local embedding generation:

```bash
# Build with GLiNER for local NER + embeddings
cargo build --release --features gliner

# Models downloaded on first use:
# - all-MiniLM-L6-v2 (embeddings, ~80MB)
# - GLiNER-small (NER, ~400MB)
```

### Linux Sandbox Dependencies

Arawn requires bubblewrap and socat on Linux for sandboxed shell execution:

```bash
# Ubuntu/Debian
sudo apt-get install bubblewrap socat

# Fedora
sudo dnf install bubblewrap socat

# Arch
sudo pacman -S bubblewrap socat

# Alpine
sudo apk add bubblewrap socat

# openSUSE
sudo zypper install bubblewrap socat
```

## Directory Structure

Arawn uses the following data layout:

```
~/.arawn/
├── arawn.toml           # User configuration
├── arawn.log            # Rotating logs
└── data/
    ├── memory.db        # SQLite (memories, sessions, notes)
    ├── memory.db-wal    # Write-ahead log
    ├── workstreams.db   # Workstream cache
    └── workstreams/     # JSONL message history
```

## Resource Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 512 MB | 2 GB (with local embeddings) |
| Disk | 100 MB | 1 GB (for memories + models) |
| CPU | 2 cores | 4 cores (parallel tool execution) |

## Verifying Installation

```bash
# Check version
arawn --version

# Verify configuration
arawn config show

# Test with a simple question
arawn ask "Hello, are you working?"
```

## Next Steps

- [Quick Start](quickstart.md) — Your first conversation
- [Configuration](configuration.md) — Set up LLM backends
