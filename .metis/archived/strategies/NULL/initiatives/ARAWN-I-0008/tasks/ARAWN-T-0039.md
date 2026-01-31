---
id: add-bootstrapcontext-loader
level: task
title: "Add BootstrapContext Loader"
short_code: "ARAWN-T-0039"
created_at: 2026-01-28T15:50:13.298584+00:00
updated_at: 2026-01-28T15:55:30.290586+00:00
parent: ARAWN-I-0008
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0008
---

# Add BootstrapContext Loader

## Parent Initiative
[[ARAWN-I-0008]]

## Objective

Implement `BootstrapContext` for loading workspace context files (SOUL.md, BOOTSTRAP.md, etc.) with smart truncation for large files.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create `crates/arawn-agent/src/prompt/bootstrap.rs`
- [ ] `BootstrapContext::load(path)` loads files from workspace
- [ ] Support files: SOUL.md, BOOTSTRAP.md, MEMORY.md, IDENTITY.md
- [ ] Truncation: 70% head + 20% tail when exceeding max chars (default 20,000)
- [ ] Warning callback when truncation occurs
- [ ] `to_prompt_section()` formats loaded files for prompt

## Implementation Notes

### Constants (from moltbot)
```rust
const DEFAULT_MAX_CHARS: usize = 20_000;
const HEAD_RATIO: f64 = 0.7;
const TAIL_RATIO: f64 = 0.2;
```

### API Design
```rust
pub struct BootstrapFile {
    pub filename: String,
    pub content: String,
    pub truncated: bool,
}

pub struct BootstrapContext {
    files: Vec<BootstrapFile>,
}

impl BootstrapContext {
    pub fn load(workspace: impl AsRef<Path>) -> io::Result<Self>;
    pub fn load_with_options(
        workspace: impl AsRef<Path>,
        max_chars: usize,
        warn_fn: Option<Box<dyn Fn(&str)>>,
    ) -> io::Result<Self>;
    pub fn files(&self) -> &[BootstrapFile];
    pub fn to_prompt_section(&self) -> String;
}
```

### Truncation Logic
When file exceeds max_chars:
1. Calculate head_len = (max_chars * 0.7) as usize
2. Calculate tail_len = (max_chars * 0.2) as usize
3. Take first head_len chars + "\n...[truncated]...\n" + last tail_len chars

### Dependencies
- Depends on ARAWN-T-0038 (core module structure)

## Status Updates

### 2026-01-28
- Implemented as part of ARAWN-T-0038 (core module)
- Created `crates/arawn-agent/src/prompt/bootstrap.rs` with:
  - `BootstrapFile` struct for individual loaded files
  - `BootstrapContext` struct with `load()` and `load_with_options()` methods
  - Smart truncation with 70% head + 20% tail strategy
  - Unicode-safe char boundary handling
  - Support for SOUL.md, BOOTSTRAP.md, MEMORY.md, IDENTITY.md
- 12 tests for bootstrap module all passing