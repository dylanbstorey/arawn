# Installation

Arawn is designed for edge computing — a single binary with embedded storage.

## Building from Source

### Prerequisites

- Rust 1.75+ with cargo
- C compiler (for SQLite bindings)
- Optional: ONNX Runtime for local embeddings

### Basic Build

```bash
# Clone the repository
git clone https://github.com/your-org/arawn.git
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
