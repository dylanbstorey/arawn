---
id: git-clone-and-update-for
level: task
title: "Git clone and update for subscribed plugins"
short_code: "ARAWN-T-0126"
created_at: 2026-02-03T19:44:25.173125+00:00
updated_at: 2026-02-04T03:18:02.988886+00:00
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

# Git clone and update for subscribed plugins

## Objective

Implement git clone and pull operations for subscribed plugins, fetching them to the cache directory.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Clone GitHub repos: `github.com/<owner>/<repo>` format
- [x] Clone arbitrary git URLs (https, git@)
- [x] Support branch/tag/commit ref specification
- [x] Clone to `~/.cache/arawn/plugins/<source>/<name>/`
- [x] Pull updates for existing clones
- [x] Handle clone failures gracefully (log warning, continue)
- [x] Tests for clone operations (mocked git)

## Implementation Notes

### Clone Destination Structure

```
~/.cache/arawn/plugins/
  github.com/
    dstorey/
      arawn-journal-plugin/
        .claude-plugin/
          plugin.json
        skills/
        ...
  gitlab.com/
    team/
      other-plugin/
        ...
```

### Source Types

1. **GitHub shorthand**: `owner/repo` → `https://github.com/owner/repo.git`
2. **Full git URL**: `https://gitlab.com/team/repo.git`
3. **SSH URL**: `git@github.com:owner/repo.git`

### Implementation Approach

Use `git2` crate or shell out to `git` command:
- `git clone --depth 1 --branch <ref> <url> <dest>` for initial clone
- `git pull --ff-only` for updates

Prefer shelling out to git for simplicity and credential handling (SSH keys, credential helpers).

### Files to Modify

- `crates/arawn-plugin/src/subscription.rs` - Add clone/update functions
- `crates/arawn-plugin/Cargo.toml` - Add git2 if using library approach

### Dependencies

- ARAWN-T-0125 (subscription config) - knows what to clone

## Status Updates

### Completed 2026-02-04

**arawn-plugin/src/subscription.rs:**
- Added `GitOps` struct with static methods for git operations:
  - `clone()` - shallow clone with `--depth 1 --branch <ref>`
  - `pull()` - fetch all + checkout + pull --ff-only
  - `is_available()` - check if git command exists
  - `current_commit()` / `current_branch()` - repo inspection
- Added `SyncResult` and `SyncAction` types for tracking sync outcomes
- Extended `SubscriptionManager` with:
  - `sync_all()` - sync all subscriptions
  - `sync_subscription()` - sync single subscription
  - `plugin_dir_for()` - get plugin directory (local or cache)
  - `plugin_dirs()` - get all available plugin directories

**Design decisions:**
- Shell out to git instead of git2 crate for better credential handling (SSH keys, credential helpers)
- Shallow clones for faster downloads
- Graceful failure handling - failures logged but don't stop other syncs
- Local subscriptions return `Skipped` action

**Tests:** 10 new tests (+ 2 ignored integration tests for real git operations)
All 130 tests pass.