---
id: runtime-catalog-format-loader-and
level: task
title: "Runtime catalog: format, loader, and CRUD operations"
short_code: "ARAWN-T-0081"
created_at: 2026-01-30T03:41:21.737944+00:00
updated_at: 2026-01-30T03:55:41.669851+00:00
parent: ARAWN-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0019
---

# Runtime catalog: format, loader, and CRUD operations

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Implement the runtime catalog system that manages discovery and persistence of WASM runtimes. This includes defining the `catalog.toml` format and serde types, building the `RuntimeCatalog` struct with full CRUD operations (load, save, add, remove, get, list), and establishing the `builtin/` and `custom/` directory layout for organizing runtime `.wasm` modules.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `catalog.toml` serde types defined with runtime name, description, path, and category (builtin/custom)
- [ ] `RuntimeCatalog` struct with `load()`, `save()`, `add()`, `remove()`, `get()`, and `list()` methods
- [ ] Auto-creates `builtin/` and `custom/` directory structure on first load if missing
- [ ] `list()` returns both built-in and custom runtimes with their metadata
- [ ] Unit tests covering CRUD operations, missing directory creation, and round-trip serialization

## Implementation Notes

### Dependencies
- ARAWN-T-0080 (RuntimeInput/RuntimeOutput protocol types) — catalog entries reference runtimes that conform to these types

### Approach
Define `CatalogEntry` and `RuntimeCatalog` in a `catalog` module within the script executor crate. Use `toml` + `serde` for persistence. The catalog file lives at `<data_dir>/runtimes/catalog.toml` with `builtin/` and `custom/` subdirectories alongside it. `RuntimeCatalog::load()` reads or initializes the file and ensures directories exist.

## Status Updates

### Session — completed
- Created `arawn-pipeline/src/catalog.rs` with `RuntimeCatalog`, `CatalogEntry`, `RuntimeCategory`
- CRUD: `load()`, `save()`, `add()`, `remove()`, `get()`, `list()`, `resolve_path()`
- `load()` auto-creates `builtin/` and `custom/` subdirs
- Persistence via `catalog.toml` (BTreeMap of name → entry)
- 9 unit tests: empty catalog, add/get, remove, overwrite, list, roundtrip persistence, resolve path, directory creation
- All pass, workspace clean