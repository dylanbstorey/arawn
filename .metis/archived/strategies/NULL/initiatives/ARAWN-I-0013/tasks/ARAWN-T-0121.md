---
id: migrate-skills-to-skills-name
level: task
title: "Migrate skills to skills/<name>/SKILL.md format"
short_code: "ARAWN-T-0121"
created_at: 2026-02-03T19:44:20.589668+00:00
updated_at: 2026-02-03T21:54:15.608211+00:00
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

# Migrate skills to skills/<name>/SKILL.md format

## Objective

Update skill loading to use Claude Code's directory-based format where each skill lives in `skills/<name>/SKILL.md` instead of `skills/<name>.md`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Skills discovered from `skills/<name>/SKILL.md` path pattern (done in T-0120)
- [x] Skill frontmatter parsed from YAML (matching Claude's format)
- [x] Support for auxiliary files in skill directories (e.g., reference docs, scripts) - directory structure supports this
- [x] Skill namespacing: `/<plugin-name>:<skill-name>` format
- [x] Remove old flat-file skill loading (done in T-0120)
- [x] Tests updated for new format

## Implementation Notes

### Claude Skill Format

```
skills/
  code-review/
    SKILL.md        # Main skill file (required)
    reference.md    # Optional auxiliary files
    scripts/        # Optional scripts
```

### SKILL.md Format

```markdown
---
name: code-review
description: Reviews code for best practices
---

Review this code for:
1. Potential bugs
2. Security issues
3. Performance problems
```

### Files to Modify

- `crates/arawn-plugin/src/skill.rs` - Update discovery and loading logic
- `crates/arawn-plugin/src/manager.rs` - Update skill path resolution

### Dependencies

- ARAWN-T-0120 (manifest migration) - skills path comes from manifest

## Status Updates

### Session 1 - 2026-02-03

**Migrated skills to YAML frontmatter and added namespacing:**

1. **Cargo.toml** - Added `serde_yaml = "0.9"` dependency for YAML parsing

2. **skill.rs** - Complete updates:
   - Changed `parse_skill()` from TOML to YAML frontmatter parsing
   - Updated `SkillInvocation` to include optional `plugin` field for namespacing
   - Updated `detect_invocation()` to parse `/plugin:skill` namespaced format
   - Rewrote `SkillRegistry` to:
     - Store skills by qualified name (`plugin:skill`)
     - Index by simple name for non-namespaced lookups
     - Handle ambiguous lookups (same skill name in multiple plugins)
     - Added `get_by_invocation()` and `invoke_simple()` methods
   - Updated all tests to use YAML syntax
   - Added new tests for namespaced invocation and lookup

**Key changes:**
- Frontmatter now uses YAML: `name: skill-name` instead of `name = "skill-name"`
- Skills invocable as `/skill` (simple) or `/plugin:skill` (namespaced)
- Simple lookups fail gracefully when ambiguous (multiple plugins have same skill)

**Verification:**
- `angreal check all` passes
- `angreal test unit` - all 93 plugin tests pass