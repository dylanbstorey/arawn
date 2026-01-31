---
id: migrate-manifest-from-plugin-toml
level: task
title: "Migrate manifest from plugin.toml to .claude-plugin/plugin.json"
short_code: "ARAWN-T-0120"
created_at: 2026-02-03T19:44:19.665725+00:00
updated_at: 2026-02-03T21:48:41.785633+00:00
parent: ARAWN-I-0013
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0013
---

# Migrate manifest from plugin.toml to .claude-plugin/plugin.json

## Objective

Replace the current `plugin.toml` manifest format with Claude Code's `.claude-plugin/plugin.json` format for full compatibility with the Claude plugin ecosystem.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Add JSON schema types for `PluginManifest` matching Claude's format
- [x] Plugin discovery looks for `.claude-plugin/plugin.json` instead of `plugin.toml`
- [x] Parse all manifest fields: name, version, description, author, homepage, repository, license, keywords
- [x] Parse component paths: commands, agents, skills, hooks, mcpServers, lspServers, outputStyles
- [x] Remove TOML manifest parsing code
- [x] All existing tests updated for new format

## Implementation Notes

### Claude plugin.json Schema

```json
{
  "name": "plugin-name",
  "version": "1.0.0",
  "description": "Plugin description",
  "author": { "name": "Author", "email": "email@example.com" },
  "homepage": "https://docs.example.com",
  "repository": "https://github.com/author/plugin",
  "license": "MIT",
  "keywords": ["keyword1", "keyword2"],
  "commands": "./commands/",
  "agents": "./agents/",
  "skills": "./skills/",
  "hooks": "./hooks/hooks.json",
  "mcpServers": "./.mcp.json"
}
```

### Files to Modify

- `crates/arawn-plugin/src/manifest.rs` - Replace TOML with JSON parsing
- `crates/arawn-plugin/src/types.rs` - Update type definitions
- `crates/arawn-plugin/src/manager.rs` - Update discovery path
- `crates/arawn-plugin/Cargo.toml` - Remove toml dep if unused elsewhere

## Status Updates

### Session 1 - 2026-02-03

**Completed migration from TOML to JSON format:**

1. **manifest.rs** - Complete rewrite:
   - Replaced `PluginManifest` with Claude-compatible JSON schema
   - Added `PathOrPaths` enum for single/multiple path support
   - Added `PluginAuthor` struct with name/email/url
   - Added helper methods: `skills_paths()`, `agents_paths()`, `hooks_paths()`, `commands_paths()`
   - Added `PluginMeta` conversion for backward compatibility
   - Renamed `from_toml()` to `from_json()`
   - All fields now match Claude Code's plugin.json schema

2. **manager.rs** - Updated discovery and loading:
   - Changed `MANIFEST_PATH` to `.claude-plugin/plugin.json`
   - Updated `discover_skills()` for `skills/<name>/SKILL.md` format (Claude pattern)
   - Updated `discover_agents()` for `agents/<name>.md` format (Claude pattern)
   - Added `extract_frontmatter_field()` for YAML frontmatter parsing
   - Added `parse_agent_markdown()` for markdown agent configs
   - Updated all test fixtures to use `.claude-plugin/` directory and JSON

3. **watcher.rs** - Fixed references:
   - Updated all `manifest.plugin.name` → `manifest.name`
   - Updated all test fixtures to use Claude format

4. **start.rs** - Fixed manifest field references:
   - Removed `manifest.tools` reference (commands not yet implemented)
   - Removed `manifest.prompt` reference (not in Claude format)
   - Fixed `manifest.plugin.name` → `manifest.name`

**Verification:**
- `angreal check all` passes
- `angreal test unit` - all tests pass

**Deferred to future tasks:**
- CLI tools registration (commands/) - T-0128
- Prompt fragments - may need Claude format research