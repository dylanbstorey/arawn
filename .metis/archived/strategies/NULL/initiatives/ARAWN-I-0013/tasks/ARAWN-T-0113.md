---
id: skill-loading-and-invocation
level: task
title: "Skill loading and invocation"
short_code: "ARAWN-T-0113"
created_at: 2026-02-02T01:54:19.232639+00:00
updated_at: 2026-02-02T03:50:01.518377+00:00
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

# Skill loading and invocation

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Implement skill loading (parse markdown with TOML frontmatter), argument substitution, and invocation. When a user message starts with `/skill-name args`, the agent loads the skill template, substitutes arguments, and injects the skill content as a system-level instruction for the current turn.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Skill markdown parser: extract TOML frontmatter (name, description, uses_tools, args) and body content
- [ ] `SkillRegistry` holding all loaded skills, queryable by name
- [ ] Argument substitution: replace `{arg_name}` placeholders in skill body with provided values
- [ ] Validation: required args must be provided, error if missing
- [ ] Skill invocation detection: parse `/skill-name arg1 arg2` from user messages
- [ ] Skill content injection: prepend skill body to the system prompt or inject as a system message for the turn
- [ ] `uses_tools` field available for optional tool constraining (stored but not enforced in this task)
- [ ] Tests: parse skill markdown, substitute args, detect invocation, handle missing args
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- Skill parser in `arawn-plugin/src/skill.rs`
- `SkillRegistry` in `arawn-plugin/src/skill_registry.rs`
- Use a simple TOML frontmatter parser: split on `---` delimiters, parse first section as TOML, rest is markdown body
- Invocation parsing: regex `^/([a-z][a-z0-9-]*)\s*(.*)$` on user message
- Argument substitution: simple string replace `{name}` → value. Positional args map to declared arg names in order.
- Injection point: the agent turn loop will need a hook (wired in T-0118), but this task provides the `SkillRegistry::invoke(name, args) -> String` method

### Dependencies
- ARAWN-T-0110 (Skill type definition)
- ARAWN-T-0111 (skills loaded from disk)

## Status Updates

*To be added during implementation*