---
id: migrate-journal-example-plugin-to
level: task
title: "Migrate journal example plugin to Claude format"
short_code: "ARAWN-T-0129"
created_at: 2026-02-03T19:44:27.576672+00:00
updated_at: 2026-02-04T13:39:43.562749+00:00
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

# Migrate journal example plugin to Claude format

## Objective

Migrate the existing journal example plugin from our TOML-based format to Claude Code's format, serving as a reference implementation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Move `plugin.toml` → `.claude-plugin/plugin.json`
- [ ] Migrate skills to `skills/<name>/SKILL.md` format
- [ ] Migrate hooks to `hooks/hooks.json` format
- [ ] Migrate agents to markdown format
- [ ] Move tool scripts to `scripts/` directory
- [ ] Use `${CLAUDE_PLUGIN_ROOT}` in all paths
- [ ] Update README with new structure
- [ ] Plugin loads and works with new format

## Implementation Notes

### Current Structure

```
examples/plugins/journal/
  plugin.toml
  skills/
    journal-entry.md
    journal-review.md
  hooks/
    session-end.sh
  agents/
    journal-assistant.toml
  tools/
    journal.sh
```

### Target Structure

```
examples/plugins/journal/
  .claude-plugin/
    plugin.json
  skills/
    journal-entry/
      SKILL.md
    journal-review/
      SKILL.md
  hooks/
    hooks.json
  agents/
    journal-assistant.md
  scripts/
    journal.sh
    session-end.sh
  README.md
```

### plugin.json Content

```json
{
  "name": "journal",
  "version": "0.1.0",
  "description": "Personal journaling and note-taking plugin",
  "author": {
    "name": "Arawn Team"
  },
  "license": "MIT",
  "skills": "./skills/",
  "agents": "./agents/",
  "hooks": "./hooks/hooks.json"
}
```

### hooks.json Content

```json
{
  "hooks": {
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/scripts/session-end.sh"
          }
        ]
      }
    ]
  }
}
```

### Dependencies

- ARAWN-T-0120 through ARAWN-T-0124 (format migrations must be complete)

## Status Updates

*To be added during implementation*