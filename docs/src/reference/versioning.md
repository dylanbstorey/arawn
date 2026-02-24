# API Versioning

Arawn's REST API uses URL-based versioning with a clear policy for breaking changes and deprecation.

## Version Format

- **URL prefix:** `/api/v{major}/` (e.g., `/api/v1/sessions`)
- **API version** is independent of the package (crate) version
- The `/api/v1/config` endpoint reports both:
  - `api_version`: The API contract version (e.g., `"1.0"`)
  - `package_version`: The `Cargo.toml` version of `arawn-server`

## What Constitutes a Breaking Change

The following changes **require** a new major API version:

| Change | Example |
|--------|---------|
| Removing an endpoint | `DELETE /api/v1/foo` no longer exists |
| Removing a response field | `session.summary` field dropped |
| Changing a field's type | `id` from `string` to `integer` |
| Changing response structure | `{ "note": Note }` → bare `Note` |
| Changing HTTP method | `POST` → `PUT` for create |
| Changing error response format | Different JSON shape for errors |
| Making an optional request field required | `tags` now mandatory |

## Non-Breaking Changes

These do **not** require a version bump:

- Adding new endpoints
- Adding optional fields to request bodies
- Adding new fields to response bodies
- Adding optional query parameters
- Adding new enum values (when clients are expected to handle unknown values)
- Changing error messages (not structure)
- Performance improvements

## Deprecation Policy

When an endpoint or field is deprecated:

1. **Announce** via HTTP headers on responses:
   - `Deprecation: true` — this endpoint is deprecated
   - `Sunset: <RFC 7231 date>` — when the endpoint will be removed
   - `Link: </api/v2/replacement>; rel="successor-version"` — the replacement

2. **Minimum notice period:** 6 months from the first `Deprecation` header to removal.

3. **Documentation:** Deprecated endpoints are marked in the OpenAPI spec with `deprecated: true`.

4. **Removal:** After the sunset date, the old endpoint returns `410 Gone` for one additional release cycle, then is removed entirely.

### Example Deprecation Headers

```
HTTP/1.1 200 OK
Deprecation: true
Sunset: Sat, 01 Mar 2027 00:00:00 GMT
Link: </api/v2/sessions>; rel="successor-version"
Content-Type: application/json
```

## Version Negotiation

Arawn does **not** support content-type negotiation or `Accept-Version` headers. The version is always in the URL path.

Multiple API versions may run concurrently on the same server when a new major version is introduced. The previous version remains available until its sunset date.

## Current API Version

| Field | Value |
|-------|-------|
| API version | `1.0` |
| URL prefix | `/api/v1/` |
| Status | Stable |

## Changelog

Breaking changes are documented in the project's `CHANGELOG.md` under the "Breaking Changes" section. Clients should review the changelog before upgrading.
