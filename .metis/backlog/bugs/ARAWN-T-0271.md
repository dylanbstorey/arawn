---
id: bug-agent-unaware-of-its-own
level: task
title: "Bug: Agent unaware of its own capabilities and workspace structure"
short_code: "ARAWN-T-0271"
created_at: 2026-03-06T03:14:16.461433+00:00
updated_at: 2026-03-06T03:14:16.461433+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Bug: Agent unaware of its own capabilities and workspace structure

## Objective

The agent doesn't know about its own internal structure — workspaces, scratch spaces, file system access, or that it can use `shell` + `file_write` to clone repos and manipulate files. It claimed "I don't have direct access to a writable file-system" despite having both `shell` and `file_write` tools. The system prompt needs to inform the agent about its environment and capabilities.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users — agent gives incorrect/misleading answers about its capabilities
- **Reproduction Steps**:
  1. Start a workstream session
  2. Ask the agent to clone a git repo or write files
  3. Agent claims it cannot access the filesystem or clone repos
- **Expected vs Actual**:
  - Expected: Agent uses `shell` to `git clone`, uses `file_write`/`file_read` to manage files, understands its workspace structure
  - Actual: Agent claims no filesystem access, suggests downloading ZIPs instead of cloning, provides tutorial-style responses instead of acting

### Evidence

From workstream `76ea4fdc`:
- User: "can you clone the repo at github.com/colliery-io/arawn"
- Agent: "I'm happy to fetch the contents for you, but I don't have direct access to a writable file-system in this environment"
- Agent has `shell`, `file_write`, `file_read`, `glob` tools — it absolutely can clone and work with repos

## Acceptance Criteria

- [ ] System prompt includes description of the agent's workspace structure (scratch space, workstream directories)
- [ ] System prompt clarifies the agent has full filesystem access via `shell`, `file_read`, `file_write`
- [ ] Agent can successfully clone a repo and navigate it when asked
- [ ] Agent understands workstream concept and can create/switch workstreams

## Implementation Notes

### Likely Areas
- `crates/arawn-agent/` — system prompt construction
- `crates/arawn-server/` — workspace/workstream context injection
- The system prompt likely needs a section describing:
  - Available workspace paths and what they're for
  - That `shell` gives full system access (git, build tools, etc.)
  - Workstream structure and how sessions relate
  - Tool capabilities summary with concrete examples

## Status Updates

*To be added during implementation*