---
id: claude-oauth-proxy-vendored-claude
level: initiative
title: "Claude OAuth Proxy: Vendored Claude MAX Authentication"
short_code: "ARAWN-I-0010"
created_at: 2026-01-28T18:46:45.436890+00:00
updated_at: 2026-01-28T18:55:09.966584+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: claude-oauth-proxy-vendored-claude
---

# Claude OAuth Proxy: Vendored Claude MAX Authentication Initiative

## Context

Arawn needs to support Claude MAX subscription authentication via OAuth 2.0 PKCE flow. This allows users with Claude MAX plans to use their subscription credits without needing a separate API key. The implementation is modeled after muninn's proxy at `~/Desktop/muninn/crates/muninn-rlm/`.

Key components from muninn:
- **OAuth PKCE flow** (`oauth.rs`): Browser-based auth with Anthropic's hardcoded client ID (`9d1c250a-e61b-44d9-88ed-5944d1962f5e`), token exchange, refresh
- **Token management** (`token_manager.rs`): File-based token persistence with auto-refresh (5 min buffer before expiry)
- **Request mangling** (`passthrough.rs`): System prompt injection ("You are Claude Code..."), field stripping (only valid Anthropic fields), `anthropic-beta` header injection
- **Proxy server** (`proxy.rs`): Axum-based localhost proxy exposing `/v1/messages` endpoint

## Goals & Non-Goals

**Goals:**
- Create `arawn-oauth` crate with OAuth PKCE flow, token management, and passthrough logic
- Integrate as a backend option in `arawn-config` (backend = "claude-oauth")
- Enable `arawn start` to work with Claude MAX without any API key
- CLI command `arawn oauth` for initial browser-based authentication

**Non-Goals:**
- RLM routing logic (muninn-specific)
- OpenAI-compatible endpoint translation
- Agentic tracing infrastructure

## Detailed Design

New crate `arawn-oauth` with modules:
1. `oauth.rs` — PKCE challenge generation, authorization URL building, code exchange, token refresh
2. `token_manager.rs` — `TokenManager` trait, `FileTokenManager`, token expiry checking
3. `passthrough.rs` — Request preparation (system prompt injection, field stripping, anthropic-beta headers), auth mode handling
4. `proxy.rs` — Axum proxy server on localhost, `/v1/messages` + `/health` endpoints

Integration points:
- `arawn-config`: Add `Backend::ClaudeOAuth` variant, token storage path config
- `arawn/commands/start.rs`: When backend is claude-oauth, start proxy + point AnthropicBackend at proxy URL
- `arawn/commands/oauth.rs`: New CLI command for browser-based OAuth flow

## Implementation Plan

1. Create `arawn-oauth` crate with OAuth PKCE + token management (port from muninn)
2. Add passthrough client with request mangling
3. Add proxy server (Axum, localhost)
4. Add `arawn oauth` CLI command for authentication
5. Wire into `arawn start` as a backend option
6. Integration testing