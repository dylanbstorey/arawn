---
id: hide-password-secret-input-in-cli
level: task
title: "Hide password/secret input in CLI prompts"
short_code: "ARAWN-T-0260"
created_at: 2026-03-04T13:24:04.951466+00:00
updated_at: 2026-03-05T03:01:34.534131+00:00
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

# Hide password/secret input in CLI prompts

## Objective

Use `rpassword` crate (or equivalent) to hide password/secret input when users type API keys or secret values in `arawn config set-secret` and `arawn secrets set`.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Secrets are currently visible on screen when typed, which is a security concern in shared environments
- **Effort Estimate**: XS

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add `rpassword` dependency to arawn CLI crate
- [ ] `arawn config set-secret` hides input
- [ ] `arawn secrets set` hides input
- [ ] Works on macOS and Linux terminals

### Key Files
- `crates/arawn/src/commands/config.rs`
- `crates/arawn/src/commands/secrets.rs`
- `crates/arawn/Cargo.toml`

## Status Updates

### Completed
- Added `rpassword = "7.3"` to `crates/arawn/Cargo.toml` (resolved to 7.4.0)
- `config.rs`: Replaced `stdin().read_line()` with `rpassword::prompt_password()` in `cmd_set_secret`
- `secrets.rs`: Replaced `stdin().read_line()` with `rpassword::prompt_password()` in `cmd_set`
- Both prompts now show "(input hidden)" and suppress terminal echo
- `rpassword` uses `libc::tcsetattr` on Unix — works on macOS and Linux
- All checks pass (`angreal check all`)