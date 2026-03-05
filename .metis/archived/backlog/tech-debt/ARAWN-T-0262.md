---
id: remove-hardcoded-oauth-credentials
level: task
title: "Remove hardcoded OAuth credentials"
short_code: "ARAWN-T-0262"
created_at: 2026-03-04T13:24:10.841111+00:00
updated_at: 2026-03-05T03:41:38.086292+00:00
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

# Remove hardcoded OAuth credentials

## Objective

Move hardcoded OAuth client_id and other credentials out of source code into configuration. Currently embedded in `arawn-oauth` crate source.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: OAuth client credentials are hardcoded in source; rotating them requires a code change and rebuild
- **Benefits of Fixing**: Credentials manageable via config; easier to support multiple OAuth providers
- **Risk Assessment**: Low — straightforward config extraction

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] OAuth client_id read from config file or environment variable
- [ ] No credentials hardcoded in source code
- [ ] Fallback to current values if not configured (backward compat)
- [ ] Documentation updated with new config options

### Key Files
- `crates/arawn-oauth/src/`
- `crates/arawn-config/src/` (add OAuth config section)

## Status Updates

### Session 1 (Complete)
- Extracted 5 hardcoded values (client_id, authorize_url, token_url, redirect_uri, scope) to named constants on `OAuthConfig`
- Made `OAuthConfig::anthropic_max()` check `ARAWN_OAUTH_*` env vars with fallback to constants
- Added `OAuthConfig::with_overrides()` builder method for config-driven overrides
- Added `OAuthConfigOverride` struct to `arawn-config/src/types.rs` with `[oauth]` TOML section
- Added `oauth` field to `ArawnConfig`, `RawConfig`, `merge()`, and both `From` impls
- Added `FileTokenManager::with_config()` and `create_token_manager_with_config()` to `arawn-oauth`
- Updated `auth.rs` `cmd_login` to load `[oauth]` config and pass to both OAuthConfig and token manager
- Updated `start.rs` `create_backend()` to accept `oauth_overrides` param and wire through to ClaudeOauth backend
- Updated `docs/src/configuration/reference.md` with new `[oauth]` section and 5 new env vars
- All checks pass (`angreal check all`), all tests pass (`angreal test unit`)