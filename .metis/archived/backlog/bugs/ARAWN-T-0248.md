---
id: add-websocket-origin-validation-in
level: task
title: "Add WebSocket Origin validation in chat routes"
short_code: "ARAWN-T-0248"
created_at: 2026-03-04T13:23:49.193089+00:00
updated_at: 2026-03-04T15:14:07.271519+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Add WebSocket Origin validation in chat routes

## Objective

Add Origin header validation to WebSocket upgrade requests in chat routes to prevent cross-site WebSocket hijacking (CSWSH) attacks.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P0 - Critical (blocks users/revenue)

### Impact Assessment
- **Affected Users**: All users connecting via WebSocket chat
- **Severity**: HIGH - A malicious website could connect to a user's local arawn server and issue commands
- **Expected vs Actual**: WebSocket upgrades should validate Origin header against allowed origins; currently no validation is performed

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] WebSocket upgrade handler checks Origin header
- [ ] Connections from non-allowed origins are rejected with 403
- [ ] Configurable allowed origins list (default: localhost variants)
- [ ] Unit test for valid and invalid Origin headers
- [ ] Existing localhost connections still work

## Implementation Notes

### Key Files
- `crates/arawn-server/src/routes/chat.rs`
- Server config for allowed origins

## Status Updates

### Session 1 — Investigation & Implementation

**Investigation:**
- WebSocket Origin validation was already partially implemented in `routes/ws/mod.rs`
- `validate_origin()` with exact and wildcard matching already existed
- Missing: default localhost origins, TOML config field, port-aware localhost matching, wiring through `start.rs`

**Changes:**

1. **`crates/arawn-config/src/types.rs`** — Added `ws_allowed_origins: Vec<String>` to `ServerConfig` (TOML config type) with `#[serde(default)]`

2. **`crates/arawn-server/src/routes/ws/mod.rs`** — Enhanced origin validation:
   - Added `is_localhost_origin()` — identifies bare localhost-class origins
   - Added `origin_matches_ignoring_port()` — matches `http://localhost:3000` against `http://localhost`
   - Updated `validate_origin()` to use port-ignoring match for localhost-class origins
   - Added `["*"]` wildcard support in `ws_handler()` to allow all origins when explicitly configured
   - 10 new tests covering: localhost with any port, 127.0.0.1 with port, IPv6 localhost with port, wrong scheme rejection, non-localhost port safety, default localhost variants (valid + invalid), helper function unit tests

3. **`crates/arawn/src/commands/start.rs`** — Wired `ws_allowed_origins` config:
   - Reads from TOML `[server]` section
   - When auth is enabled and no explicit origins: defaults to localhost variants (http/https for localhost, 127.0.0.1, [::1])
   - When auth is disabled: no origins = all allowed (dev mode)
   - `["*"]` in config = all allowed

**Verification:** `angreal check all` (clippy+fmt clean), `angreal test unit` (all pass, 16 WS origin tests)