---
id: catalog-tool-actions-list-register
level: task
title: "Catalog tool actions: list, register, inspect, remove"
short_code: "ARAWN-T-0086"
created_at: 2026-01-30T03:41:25.774677+00:00
updated_at: 2026-01-30T04:20:32.995131+00:00
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

# Catalog tool actions: list, register, inspect, remove

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Add catalog management actions to the MCP tool surface (either as new actions on `WorkflowTool` or as a dedicated `CatalogTool`): `catalog_list`, `catalog_register`, `catalog_inspect`, and `catalog_remove`. These actions let the agent discover available runtimes, register new custom runtimes it has compiled, inspect a runtime's expected config schema, and remove custom runtimes from the catalog.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `catalog_list` returns all runtimes (builtin + custom) with name, description, and category
- [ ] `catalog_register` adds a compiled `.wasm` file to `custom/` with name, description, and config schema metadata
- [ ] `catalog_inspect` returns the config schema and description for a given runtime name
- [ ] `catalog_remove` deletes a custom runtime entry and its `.wasm` file; refuses to remove builtin runtimes with a clear error
- [ ] All four actions are exposed as MCP tool actions callable by the agent
- [ ] Unit tests for each action including the builtin-removal guard

## Implementation Notes

### Dependencies
- ARAWN-T-0081 (catalog) — all actions delegate to RuntimeCatalog CRUD methods
- ARAWN-T-0085 (wiring) — catalog must be accessible from the tool layer

### Approach
Add a `CatalogTool` struct implementing the MCP `Tool` trait with four actions. Each action maps directly to `RuntimeCatalog` methods. `catalog_register` copies the `.wasm` file into `custom/`, updates `catalog.toml`, and returns the new entry. `catalog_remove` checks `entry.category == "builtin"` before proceeding. Register `CatalogTool` alongside `WorkflowTool` in server startup.

## Status Updates

*No updates yet.*