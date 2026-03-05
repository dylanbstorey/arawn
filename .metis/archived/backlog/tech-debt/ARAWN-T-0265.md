---
id: document-unexplained-acronyms-rlm
level: task
title: "Document unexplained acronyms (RLM, NER, ORP)"
short_code: "ARAWN-T-0265"
created_at: 2026-03-04T13:24:14.983822+00:00
updated_at: 2026-03-05T04:41:54.076797+00:00
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

# Document unexplained acronyms (RLM, NER, ORP)

## Objective

Add glossary entries or module-level doc comments explaining domain-specific acronyms: RLM (Recursive Learning Model), NER (Named Entity Recognition), ORP (Observation-Reasoning-Planning).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: New contributors encounter RLM, NER, ORP in code without explanation
- **Benefits of Fixing**: Faster onboarding, self-documenting codebase
- **Risk Assessment**: None

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Each acronym defined in its module's doc comment (`//!`)
- [ ] Glossary section added to README or docs site
- [ ] `grep -r "RLM\|NER\|ORP"` — every occurrence near a definition or cross-reference

## Status Updates

### Audit
- **RLM**: Already defined in `rlm/mod.rs:1`, `types.rs:76`, `types.rs:1527`. Added expansion to `tools/explore.rs:1`.
- **NER**: Already defined in `indexing/ner.rs:1`. Config and indexer references are contextually clear.
- **ORP**: Already defined in `orp-vendored/src/lib.rs:1`. Only used as import in `gliner.rs`.
- No module-level doc comments were missing definitions for these acronyms.

### Implementation
- Created `docs/src/reference/glossary.md` with 6 acronyms (GLiNER, MCP, NER, ORP, PKCE, RLM) and 6 key concepts (agent loop, compaction, workstream, turn, tool registry, session)
- Added glossary to `docs/src/SUMMARY.md` and `docs/src/reference/README.md`
- Expanded RLM in `tools/explore.rs` module doc (only file where first mention lacked expansion)
- Docs build succeeds, `angreal check all` clean