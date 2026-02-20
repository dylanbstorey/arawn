---
id: auto-generate-openapi-swagger
level: task
title: "Auto-generate OpenAPI/Swagger documentation"
short_code: "ARAWN-T-0192"
created_at: 2026-02-18T15:24:57.049167+00:00
updated_at: 2026-02-20T02:23:34.501887+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Auto-generate OpenAPI/Swagger documentation

## Objective

Add automatic OpenAPI/Swagger documentation generation for all arawn-server API endpoints, keeping docs in sync with code and enabling interactive API exploration.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: 
  - API documentation is manual and gets out of sync
  - No interactive way for developers to explore/test endpoints
  - Client SDK generation requires manual schema maintenance
- **Benefits of Fixing**: 
  - Always-accurate API docs generated from code
  - Swagger UI for interactive testing
  - Client SDK generation from OpenAPI spec
  - Better developer experience for API consumers
- **Risk Assessment**: Low risk - additive improvement, no behavioral changes

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] OpenAPI 3.x spec auto-generated from route definitions
- [x] All existing endpoints documented (workstreams, sessions, chat, memory, agents, etc.)
- [x] Request/response schemas derived from serde structs
- [x] Swagger UI served at `/api/docs` or similar
- [x] JSON spec available at `/api/openapi.json`
- [x] Auth requirements documented per endpoint

## Implementation Notes

### Technical Approach

Use `utoipa` crate with axum integration:
- `#[derive(ToSchema)]` on request/response structs
- `#[utoipa::path(...)]` macros on handlers
- `utoipa-swagger-ui` for interactive docs

### Candidate Crates
- `utoipa` - OpenAPI generation with derive macros
- `utoipa-swagger-ui` - Swagger UI integration
- `aide` - Alternative with axum-first design

### Dependencies
- None - can be added incrementally

### Risk Considerations
- Compile time increase from proc macros (acceptable)
- Need to maintain schema annotations as API evolves

## Status Updates

### 2026-02-20: Initial Implementation

Added OpenAPI/Swagger infrastructure using utoipa:

**Dependencies Added** (`arawn-server/Cargo.toml`):
- `utoipa = "5"` with features: axum_extras, chrono, uuid
- `utoipa-axum = "0.2"`
- `utoipa-swagger-ui = "9"` with axum feature

**Files Created**:
- `routes/openapi.rs` - ApiDoc struct aggregating all paths, SecurityAddon for bearer auth

**Files Modified**:
- `routes/sessions.rs` - Added `ToSchema` derives, `#[utoipa::path]` macros to all 6 handlers
- `routes/health.rs` - Added `ToSchema` derives, `#[utoipa::path]` macros to both handlers
- `routes/config.rs` - Added `ToSchema` derives, `#[utoipa::path]` macro to handler
- `routes/mod.rs` - Added openapi module export
- `lib.rs` - Mounted Swagger UI at `/api/docs`

**Endpoints Documented** (9 of ~30):
- `/health` - GET
- `/health/detailed` - GET
- `/api/v1/config` - GET
- `/api/v1/sessions` - GET, POST
- `/api/v1/sessions/{id}` - GET, PATCH, DELETE
- `/api/v1/sessions/{id}/messages` - GET

**Remaining** (to fully complete acceptance criteria):
- workstreams.rs (~12 endpoints)
- memory.rs (~7 endpoints)
- chat.rs (~2 endpoints)
- agents.rs (~2 endpoints)
- tasks.rs (~3 endpoints)
- mcp.rs (~6 endpoints)
- commands.rs (~3 endpoints)

The pattern is established - remaining endpoints follow the same approach.

### 2026-02-19: Full Implementation Complete

All API endpoints now have OpenAPI documentation.

**Files Modified in This Session**:
- `routes/workstreams.rs` - Added ToSchema to 17 structs, #[utoipa::path] to 14 handlers
- `routes/memory.rs` - Added ToSchema to 10 structs, #[utoipa::path] to 8 handlers
- `routes/agents.rs` - Added ToSchema to 5 structs, #[utoipa::path] to 2 handlers
- `routes/tasks.rs` - Added ToSchema to 3 structs, #[utoipa::path] to 3 handlers
- `routes/mcp.rs` - Added ToSchema to 7 structs, #[utoipa::path] to 6 handlers
- `routes/chat.rs` - Added ToSchema to 4 structs, #[utoipa::path] to 2 handlers
- `routes/commands.rs` - Added ToSchema to 4 structs, #[utoipa::path] to 3 handlers
- `routes/openapi.rs` - Registered all paths and schemas

**Total Documented Endpoints**: 47 endpoints across 10 tags
- health (2), config (1), sessions (6), workstreams (14), memory (8)
- agents (2), chat (2), tasks (3), mcp (6), commands (3)

**Swagger UI**: Available at `/api/docs`
**OpenAPI JSON**: Available at `/api/openapi.json`

All acceptance criteria met.