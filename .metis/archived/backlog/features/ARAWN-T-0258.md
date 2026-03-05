---
id: add-command-examples-to-cli-help
level: task
title: "Add command examples to CLI --help text"
short_code: "ARAWN-T-0258"
created_at: 2026-03-04T13:24:03.194167+00:00
updated_at: 2026-03-05T02:39:12.757188+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Add command examples to CLI --help text

## Objective

Add usage examples to all CLI subcommands via clap's `#[command(after_help = ...)]` or `#[arg(help = ...)]` so users can see concrete invocation patterns in `--help` output.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Users must guess at argument syntax; examples reduce trial-and-error
- **Effort Estimate**: S

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every subcommand (`start`, `ask`, `chat`, `config`, `memory`, `notes`, `secrets`, `plugin`, `agent`, `mcp`, `status`, `tui`) has at least one example in help text
- [ ] Examples use `# Examples` section via clap's `after_help`
- [ ] `arawn --help` shows a top-level example

## Status Updates

### Implementation Complete

Added `after_help` examples to all 13 CLI subcommands plus the top-level `arawn --help`:

| Command | Examples | File |
|---------|----------|------|
| `arawn` (top-level) | 5 | main.rs |
| `start` | 5 | commands/start.rs |
| `status` | 3 | commands/status.rs |
| `ask` | 3 | commands/ask.rs |
| `chat` | 3 | commands/chat.rs |
| `memory` | 6 | commands/memory.rs |
| `notes` | 5 | commands/notes.rs |
| `config` | 9 | commands/config.rs |
| `auth` | 4 | commands/auth.rs |
| `plugin` | 7 | commands/plugin.rs |
| `agent` | 3 | commands/agent.rs |
| `mcp` | 7 | commands/mcp.rs |
| `secrets` | 3 | commands/secrets.rs |
| `tui` | 2 | commands/tui.rs |

- Used `\x1b[1m...\x1b[0m` ANSI bold for "Examples:" header (matches clap's style)
- `angreal check all` passes
- Help output verified with `cargo run --package arawn -- --help` and subcommands