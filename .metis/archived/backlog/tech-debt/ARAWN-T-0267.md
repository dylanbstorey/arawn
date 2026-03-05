---
id: write-adrs-for-key-architectural
level: task
title: "Write ADRs for key architectural decisions"
short_code: "ARAWN-T-0267"
created_at: 2026-03-04T13:24:16.649662+00:00
updated_at: 2026-03-05T05:19:25.179287+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Write ADRs for key architectural decisions

## Objective

Write ADRs for key architectural decisions that are currently undocumented. Only 2 ADRs exist; several major decisions lack formal records.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Key decisions (why age over keyring, why WASM for plugins, why multi-crate workspace) are tribal knowledge
- **Benefits of Fixing**: New contributors understand the "why" behind architecture choices
- **Risk Assessment**: None

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] ADR for: age-encrypted secrets vs keyring
- [ ] ADR for: WASM plugin sandbox model
- [ ] ADR for: multi-crate workspace structure
- [ ] ADR for: FsGate centralized enforcement approach
- [ ] Each ADR follows existing format in `.metis/`

## Status Updates

### Completed
Created 4 ADRs, all transitioned to `decided`:

- **ARAWN-A-0003**: Age-Encrypted Secrets Over System Keyring
  - Covers: why age over keyring, handle-based resolution, agent isolation, 4-tier cascade
- **ARAWN-A-0004**: Claude Code-Compatible Plugin System
  - Covers: why Claude Code format, prompt-level vs code-level extensions, plugin structure, runtime model
- **ARAWN-A-0005**: Multi-Crate Workspace Architecture
  - Covers: 5-layer hierarchy, 18 crates, dependency flow rules, crate inventory table
- **ARAWN-A-0006**: FsGate Centralized Filesystem Enforcement
  - Covers: centralized gate vs per-tool, deny-by-default, defense in depth, workstream-scoped boundaries

All follow the existing ADR format established by ARAWN-A-0001 and ARAWN-A-0002.