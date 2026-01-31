---
id: extend-web-fetch-with-full-http
level: task
title: "Extend web_fetch with full HTTP method, headers, and body support"
short_code: "ARAWN-T-0071"
created_at: 2026-01-29T15:49:40.303228+00:00
updated_at: 2026-02-04T14:12:29.872865+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Extend web_fetch with full HTTP method, headers, and body support

## Objective

Extend the existing `web_fetch` tool in `crates/arawn-agent/src/tools/web.rs` to support full HTTP capabilities beyond simple GET requests.

### Priority
P2 — useful but not blocking other work.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `method` parameter: GET (default), POST, PUT, PATCH, DELETE (plus HEAD, OPTIONS)
- [x] `headers` parameter: key-value map of custom request headers
- [x] `body` parameter: string or JSON request body
- [x] `timeout_secs` parameter: per-request timeout override
- [x] Backward compatible — existing calls with just `url` work unchanged
- [x] Response includes status code and optionally response headers
- [x] Tests for each HTTP method and header/body combinations

## Implementation Notes

Extend the existing tool's JSON schema with optional parameters. The current `web_fetch` implementation already uses `reqwest` — adding method/headers/body is straightforward.

## Status Updates

### 2026-02-04: Completed

Extended `WebFetchTool` in `crates/arawn-agent/src/tools/web.rs`:

**New Parameters:**
- `method`: GET (default), POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- `headers`: Object with custom request headers (key-value pairs)
- `body`: Request body string (for POST/PUT/PATCH)
- `timeout_secs`: Per-request timeout override (1-300 seconds)
- `include_headers`: Boolean to include response headers in result

**Response Changes:**
- Always includes `status` (numeric status code)
- Always includes `status_text` (e.g., "OK", "Not Found")
- Always includes `method` used
- Includes `headers` object if `include_headers: true`
- Includes `error: true` for non-2xx responses (but still returns content)

**Backward Compatible:**
- Existing calls with just `url` work unchanged
- Default method is GET
- All new parameters are optional

**Tests added:**
- Parameter schema validation
- Unsupported method handling
- Custom headers parsing
- Body parsing
- Download parameter schema
- Max size config validation

### Additional Enhancements (same session)

**Download to file:**
- `download` parameter: File path to save response directly to disk
- Streams to disk, bypassing size limits
- Returns metadata: `{ downloaded: true, path: "...", size: N }`

**Auto-download on size exceeded:**
- If Content-Length > 10MB limit, automatically streams to temp file
- If response exceeds limit during read, saves to temp file
- Returns `auto_downloaded: true` with `reason` explaining why
- Temp files stored in `{temp_dir}/arawn_downloads/{uuid}_{filename}`

**Size limit increased:**
- In-memory limit raised from 5MB to 10MB
- No limit for explicit or auto downloads (streams to disk)