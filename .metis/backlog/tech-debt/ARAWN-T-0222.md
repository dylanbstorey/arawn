---
id: documentation-accuracy-audit-fixes
level: task
title: "Documentation accuracy audit fixes"
short_code: "ARAWN-T-0222"
created_at: 2026-02-25T05:29:05.899587+00:00
updated_at: 2026-02-25T05:29:05.899587+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Documentation accuracy audit fixes

## Objective

Bring all mdbook documentation (`docs/src/`) into alignment with the actual codebase. A full audit was performed on 2026-02-25 comparing every doc page against source code. This ticket captures all findings. Some pages were already fixed in the same session (configuration/reference.md, api.md, crate-structure.md, installation.md); remaining issues are listed below.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [ ] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: ~18 documentation pages contain inaccuracies ranging from wrong config syntax to entirely fictional CLI commands and API prefixes. Users following the docs will hit errors.
- **Benefits of Fixing**: Trustworthy docs, reduced support burden, new contributors can onboard from docs alone.
- **Risk Assessment**: Low risk — documentation-only changes, no code modifications.

## Acceptance Criteria

- [ ] Every doc page accurately reflects the current implementation
- [ ] No fictional CLI commands, config keys, or API formats remain
- [ ] `angreal docs build` succeeds with no broken internal links
- [ ] All pages already fixed (reference.md, api.md, crate-structure.md, installation.md) verified still accurate

---

## Findings by Priority

### CRITICAL — Functionality completely wrong

#### 1. `docs/src/configuration/secrets.md`
- **Source**: `crates/arawn-config/src/secret.rs`
- Documents `$keyring:service/key`, `$env:VAR`, `$file:/path` prefix syntax — **none of this exists in code**. Actual resolution order is: keyring lookup → env var fallback → raw config value. No prefix parsing.
- Documents non-existent CLI commands: `arawn secret set`, `arawn secret list`, `arawn secret delete`. Actual CLI has no `secret` subcommand.
- **Fix**: Rewrite to describe the real resolution chain. Remove fictional CLI commands.

#### 2. `docs/src/extensibility/plugins.md`
- **Source**: `crates/arawn-plugin/src/manifest.rs`, `crates/arawn-plugin/src/loader.rs`
- Wrong manifest location: docs say root `plugin.json`, actual is `.claude-plugin/plugin.json`
- Wrong directory names: docs say `cli-tools/`, actual is `tools/`
- Wrong hooks format: docs describe individual named scripts, actual uses `hooks.json` with event arrays
- **Fix**: Rewrite plugin structure section, manifest examples, and hooks format.

#### 3. `docs/src/extensibility/hooks.md`
- **Source**: `crates/arawn-plugin/src/hooks.rs`
- Missing 7 of 13 hook events: `PostToolUseFailure`, `PermissionRequest`, `UserPromptSubmit`, `Notification`, `SubagentStop`, `PreCompact`, `Stop`
- Configuration format completely wrong — should be `hooks.json` with event arrays, not individual named scripts
- **Fix**: Document all 13 events. Rewrite configuration format section.

#### 4. `docs/src/extensibility/custom.md`
- **Source**: `crates/arawn-plugin/src/loader.rs`
- Wrong directory name: says `cli-tools/`, actual is `tools/`
- Wrong manifest structure for custom tools
- **Fix**: Update directory name and manifest examples.

### HIGH — Significant inaccuracies

#### 5. `docs/src/getting-started/quickstart.md`
- **Source**: `crates/arawn/src/cli.rs`
- References `arawn session list` — command doesn't exist
- Missing documentation for 8+ CLI commands: `research`, `tasks`, `notes`, `auth`, `plugin`, `agent`, `mcp`, `tui`, `stop`, `status`
- **Fix**: Remove fake commands, add real command reference or link to one.

#### 6. `docs/src/reference/built-in.md`
- **Source**: `crates/arawn-agent/src/tools/`
- Missing `catalog` tool entirely
- `delegate` tool: docs say `mode` parameter (string), actual is `background` parameter (boolean)
- **Fix**: Add catalog tool, fix delegate parameters.

#### 7. `docs/src/architecture/behavior.md`
- **Source**: `crates/arawn-agent/src/tools/mod.rs`
- Missing 5 tools from the tool list: `memory_search`, `note`, `think`, `catalog`, `workflow`
- Shell tool blocked commands list not documented
- **Fix**: Add missing tools, document shell restrictions.

#### 8. `docs/src/architecture/agent-loop.md`
- **Source**: `crates/arawn-agent/src/agent.rs`
- Claims "Max tool calls per turn: 10" — no such limit exists in code
- Claims "Tool timeout: 30s" — this is configurable per-tool, not a universal limit
- **Fix**: Remove fictional limits or document actual configurable values.

#### 9. `docs/src/reference/backends.md`
- **Source**: `crates/arawn-llm/src/backends/mod.rs`
- Missing `Custom` and `ClaudeOauth` backend types from the Backend enum
- **Fix**: Add both backends with configuration examples.

#### 10. `docs/src/architecture/c4-model.md`
- **Source**: workspace `Cargo.toml`
- Missing 4 crates from the container diagram: `arawn-domain`, `arawn-oauth`, `arawn-sandbox`, `arawn-script-sdk`
- **Fix**: Add missing crates to the C4 container table.

### MEDIUM — Minor inaccuracies or gaps

#### 11. `docs/src/getting-started/README.md`
- Rust version listed as 1.75, should be 1.85
- **Fix**: Update version number.

#### 12. `docs/src/architecture/components.md`
- **Source**: `crates/arawn-agent/src/agent.rs`
- Missing 2 Agent struct fields: `interaction_logger`, `hook_dispatcher`
- **Fix**: Add missing fields to the Agent struct documentation.

#### 13. `docs/src/concepts/memory.md`
- **Source**: `crates/arawn-memory/src/types.rs`
- Entity struct uses `label` field, docs say `entity_type`
- Citation system exists in code but is undocumented
- **Fix**: Fix field name, add citation documentation.

#### 14. `docs/src/getting-started/configuration.md`
- `backend = "default"` in example should be `"openai"`
- **Fix**: Fix example value.

#### 15. `docs/src/concepts/workstreams.md`
- **Source**: `crates/arawn-workstream/src/types.rs`
- Missing `ToolUse`, `ToolResult`, `AgentPush` message roles from the WorkstreamRole enum
- **Fix**: Add missing roles.

#### 16. `docs/src/concepts/indexing.md`
- References `--no-index` CLI flag that doesn't exist in the CLI parser
- **Fix**: Remove or replace with actual mechanism.

#### 17. `docs/src/extensibility/subagents.md`
- `delegate` tool parameter docs say `mode` (string), actual is `background` (boolean)
- **Fix**: Same as #6, ensure consistency.

#### 18. `docs/src/concepts/identity.md`
- Doesn't mention that `system_prompt` is configurable per-agent via `[agent.<name>]` config
- **Fix**: Add note about per-agent system prompt override.

---

## Already Fixed (2026-02-25)

These pages were updated in the same session that produced this audit:

| File | What was fixed |
|------|---------------|
| `docs/src/configuration/reference.md` | Full rewrite to match all config structs in code |
| `docs/src/reference/api.md` | Added all missing endpoint groups, pagination, error codes |
| `docs/src/architecture/crate-structure.md` | Added all 18 crates, accurate dependency diagram |
| `docs/src/getting-started/installation.md` | Install script section, correct URLs, Rust 1.85, Linux deps |

## Approach

For each file, read the corresponding source code, then update the doc to match. Work through CRITICAL items first, then HIGH, then MEDIUM. Run `angreal docs build` after each batch to verify no broken links.

## Status Updates

*To be added during implementation*