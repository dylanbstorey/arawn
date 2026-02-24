# Changelog

## Unreleased

### Breaking Changes

#### API Response Changes
- **Notes endpoints**: `POST /api/v1/notes` now returns `Note` directly instead of `{ "note": Note }`. Response status changed from 200 to 201.
- **Notes endpoints**: `GET /api/v1/notes/{id}` now returns `Note` directly instead of `{ "note": Note }`.
- **Notes endpoints**: `PUT /api/v1/notes/{id}` now returns `Note` directly instead of `{ "note": Note }`.
- **MCP endpoints**: `POST /api/v1/mcp/servers` now returns 201 instead of 200.
- **Workstream file endpoints**: `POST /api/v1/workstreams/{id}/files/promote` now returns 201 instead of 200.
- **Workstream file endpoints**: `POST /api/v1/workstreams/{id}/files/export` now returns 201 instead of 200.

#### Pagination
- All list endpoints now support `limit` (default: 50, max: 100) and `offset` (default: 0) query parameters.
- List responses now include `total`, `limit`, and `offset` fields alongside the collection.
- **Notes listing**: The `limit` query parameter is now part of the standard pagination (`limit` + `offset`) replacing the previous notes-specific `limit`-only behavior.

#### Path Variable Naming
- Workstream sub-resource paths changed from `{ws}` to `{id}` for consistency:
  - `/api/v1/workstreams/{ws}/files/promote` → `/api/v1/workstreams/{id}/files/promote`
  - `/api/v1/workstreams/{ws}/files/export` → `/api/v1/workstreams/{id}/files/export`
  - `/api/v1/workstreams/{ws}/clone` → `/api/v1/workstreams/{id}/clone`
  - `/api/v1/workstreams/{ws}/usage` → `/api/v1/workstreams/{id}/usage`
  - `/api/v1/workstreams/{ws}/cleanup` → `/api/v1/workstreams/{id}/cleanup`

### Added
- Shared pagination module (`PaginationParams`, `PaginatedResponse<T>`) for consistent pagination across all list endpoints.
