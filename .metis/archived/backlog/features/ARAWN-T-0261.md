---
id: create-config-schema-reference
level: task
title: "Create config schema reference documentation"
short_code: "ARAWN-T-0261"
created_at: 2026-03-04T13:24:09.922972+00:00
updated_at: 2026-03-05T03:26:37.697366+00:00
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

# Create config schema reference documentation

## Objective

Create comprehensive config schema reference documentation covering all TOML config options, their types, defaults, and examples.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Users must read source code to understand config options; a reference doc eliminates guesswork
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Documentation covers all `[llm]`, `[server]`, `[agent]`, `[memory]`, `[plugin]` config sections
- [ ] Each option shows type, default, and example value
- [ ] Published as part of the docs site (`angreal docs build`)
- [ ] Cross-referenced from README

## Status Updates

### Completed
- Rewrote `docs/src/configuration/reference.md` with comprehensive schema reference
- Every config section now has: TOML example + field table with type, default, and description
- Added missing sections:
  - **RLM Configuration** — exploration agent settings (model, max_turns, compaction, token budgets)
  - **Workstream compression** — session compression settings (backend, model, thresholds)
  - **Agent `max_tokens`** — was missing from agent field list
  - **Tool per-tool output overrides** — `shell`, `file_read`, `web_fetch`, `search` size limits
  - **Embedding local `model_url`/`tokenizer_url`** — auto-download URLs
  - **Memory indexing `ner_model_url`/`ner_tokenizer_url`** — auto-download URLs
  - **`ARAWN_SERVER_URL`** and **`ARAWN_AUTH_TOKEN`** env vars
- All 15 config sections documented: llm, agent, server, memory, embedding, tools, workstream, session, delegation, mcp, pipeline, plugins, rlm, logging, paths
- README already cross-references the doc at line 152 and 169
- `angreal docs build` succeeds