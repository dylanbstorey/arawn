---
id: plugin-subscription-config-and
level: task
title: "Plugin subscription config and storage"
short_code: "ARAWN-T-0125"
created_at: 2026-02-03T19:44:24.121313+00:00
updated_at: 2026-02-04T02:10:46.103385+00:00
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

# Plugin subscription config and storage

## Objective

Define configuration format and storage locations for plugin subscriptions, supporting both `arawn.toml` for initial subscriptions and a separate file for runtime additions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `[plugins.subscriptions]` section in `arawn.toml` for initial subscriptions
- [x] `~/.config/arawn/plugins.json` for runtime-added subscriptions (enabledPlugins)
- [x] Support project-local `.arawn/plugins.json` for project-specific plugins
- [x] Parse subscription sources: GitHub repos, git URLs, local paths
- [x] Merge subscriptions from all sources (global config + runtime + project)
- [x] Tests for config parsing and merging

## Implementation Notes

### arawn.toml Format (Initial Subscriptions)

```toml
[plugins]
enabled = true
hot_reload = true

[[plugins.subscriptions]]
source = "github"
repo = "dstorey/arawn-journal-plugin"
ref = "main"  # optional

[[plugins.subscriptions]]
source = "url"
url = "https://gitlab.com/team/plugin.git"
```

### plugins.json Format (Runtime Subscriptions)

```json
{
  "enabledPlugins": {
    "journal@local": true,
    "github-tools@github.com/author/repo": true
  },
  "subscriptions": [
    {
      "source": "github",
      "repo": "author/repo",
      "ref": "v1.0.0"
    }
  ]
}
```

### Storage Locations

- Global: `~/.config/arawn/plugins.json`
- Project: `.arawn/plugins.json`
- Cache: `~/.cache/arawn/plugins/<source>/<name>@<version>/`

### Files to Modify

- `crates/arawn-config/src/types.rs` - Add subscription types
- `crates/arawn-plugin/src/subscription.rs` - New file for subscription logic
- `crates/arawn-plugin/src/lib.rs` - Export subscription module

## Status Updates

### Completed 2026-02-04

**arawn-config/src/types.rs:**
- Added `PluginSubscription` struct with builder methods (`github()`, `url()`, `local()`, `with_ref()`)
- Added `PluginSource` enum (GitHub, Url, Local)
- Extended `PluginsConfig` with `auto_update` and `subscriptions` fields
- Subscription ID generation for cache directory naming
- Clone URL generation for git operations
- 12 new tests for subscription parsing and config

**arawn-plugin/src/subscription.rs (new file):**
- `RuntimePluginsConfig` - JSON format for runtime plugin state
- `SubscriptionManager` - merges subscriptions from all sources:
  - arawn.toml `[plugins.subscriptions]`
  - `~/.config/arawn/plugins.json` (global runtime)
  - `.arawn/plugins.json` (project-local)
- Enable/disable plugins by ID
- Deduplication and priority-based merging
- 14 tests for parsing, merging, and filtering

**arawn-plugin/Cargo.toml:**
- Added `arawn-config` dependency

All 120 plugin tests + 77 config tests pass.