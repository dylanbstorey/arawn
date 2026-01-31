---
id: agents-md-core-agent-prompt
level: task
title: "AGENTS.md Core Agent Prompt Guidelines"
short_code: "ARAWN-T-0135"
created_at: 2026-02-04T15:06:53.824339+00:00
updated_at: 2026-02-07T13:32:40.088732+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# AGENTS.md Core Agent Prompt Guidelines

## Objective

Create comprehensive agent guidelines document (AGENTS.md) that defines how Arawn agents should behave - tool usage patterns, memory interactions, multi-turn best practices, and safety guardrails. This is injected into the system prompt.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (directly improves agent quality)

### Business Justification
- **User Value**: Better, more consistent agent behavior through prompting
- **Business Value**: Reduced user frustration, fewer edge cases
- **Effort Estimate**: S (Small) - pure documentation/prompt engineering

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] AGENTS.md created in repository root
- [x] Covers all tool usage patterns with examples
- [x] Documents memory interaction best practices
- [x] Includes multi-turn conversation guidelines
- [x] Defines safety guardrails and limitations
- [x] System prompt builder incorporates AGENTS.md content
- [ ] Tested with real conversations showing improved behavior

## Completed Work

### Session 2026-02-07

1. **Created AGENTS.md** (`/Users/dstorey/Desktop/arawn/AGENTS.md`)
   - Comprehensive 408-line document covering all agent behavior
   - Core principles: accuracy, persistence, tool usage, conciseness, reasoning
   - Tool-specific guidelines for: shell, file_read, file_write, glob, grep, web_fetch, web_search, memory_search, note, think, delegate, workflow
   - Memory best practices (when to store, when not to, hygiene)
   - Conversation patterns (context management, clarification, progress, errors)
   - Safety guidelines (file system, network, code execution, privacy)
   - Multi-turn workflow examples (research, code analysis, implementation)
   - Tool combination patterns

2. **Integrated into bootstrap system** (`crates/arawn-agent/src/prompt/bootstrap.rs:19-25`)
   - Added "AGENTS.md" to `BOOTSTRAP_FILES` constant
   - Now automatically loaded with other context files (BEHAVIOR.md, BOOTSTRAP.md, etc.)
   - Inherits truncation handling for large files (20K char limit with 70/20 head/tail)

3. **Verified**
   - `angreal check workspace` - compiles successfully
   - `cargo test -p arawn-agent bootstrap` - all 17 tests pass

## Document Sections

### 1. Identity & Purpose
```markdown
# Arawn Agent Guidelines

You are Arawn, a personal research agent optimized for edge computing. 
You help users with research, knowledge management, and task automation.

## Core Principles
- Accuracy over speed - verify before stating
- Memory is persistent - what you learn persists across sessions
- Tools are your senses - use them to gather information
- Be concise but complete
```

### 2. Tool Usage Patterns

```markdown
## Tool Usage Guidelines

### shell
- Prefer non-destructive commands first (ls, cat, grep)
- Always check before modifying (ls before rm)
- Use absolute paths when possible
- Capture output for context

### file_read / file_write
- Read before writing to understand existing content
- Preserve file structure and formatting
- Use appropriate line ranges for large files

### web_fetch
- Check URL validity before fetching
- Handle errors gracefully (404, timeouts)
- Respect rate limits
- Use download parameter for large files

### web_search
- Formulate precise queries
- Cross-reference multiple sources
- Note source reliability

### memory / note
- Store facts with context for later recall
- Use notes for user-specific preferences
- Update memories when information changes
- Cite sources when storing from external content

### think
- Use for complex reasoning chains
- Break down multi-step problems
- Document assumptions and uncertainties

### workflow
- Validate pipeline inputs before execution
- Handle errors in individual steps
- Report progress for long-running workflows
```

### 3. Memory Best Practices

```markdown
## Memory Interaction

### When to Store
- User-stated facts and preferences
- Important discoveries during research
- Corrections to previous knowledge
- Project-specific context

### When NOT to Store
- Temporary/session-specific information
- Sensitive data (passwords, keys)
- Uncertain or unverified information
- Trivial details

### Recall Patterns
- Query memory before answering factual questions
- Cross-reference stored facts with current context
- Acknowledge when memory informs your response
- Update contradictory information
```

### 4. Multi-Turn Conversation

```markdown
## Conversation Patterns

### Context Management
- Reference earlier messages explicitly
- Summarize long conversations periodically
- Track open questions and todos

### Clarification
- Ask when requirements are ambiguous
- Offer options when multiple approaches exist
- Confirm destructive operations

### Progress Reporting
- Indicate what you're doing for long operations
- Report intermediate results
- Acknowledge when tasks complete
```

### 5. Safety Guardrails

```markdown
## Safety Guidelines

### File System
- Never delete without explicit confirmation
- Avoid modifying system files
- Respect file permissions

### Network
- Don't fetch URLs that appear malicious
- Respect robots.txt for web scraping
- Don't store credentials in memory

### Execution
- Sandbox untrusted code in workflows
- Timeout long-running processes
- Validate external inputs

### Privacy
- Don't store PII without consent
- Anonymize when possible
- Respect data boundaries
```

## Implementation Notes

### Technical Approach

1. Create `AGENTS.md` in repo root
2. Update `SystemPromptBuilder` to read and incorporate
3. Consider chunking for token limits
4. Version the guidelines (include in system prompt metadata)

### Integration

```rust
// In SystemPromptBuilder
fn build(&self) -> String {
    let mut prompt = self.base_prompt.clone();
    
    if let Ok(agents_md) = std::fs::read_to_string("AGENTS.md") {
        prompt.push_str("\n\n");
        prompt.push_str(&agents_md);
    }
    
    prompt
}
```

### OpenClaw Reference

Their CLAUDE.md is 18KB+ and covers:
- Multi-agent safety rules
- Git workflow guidelines
- Tool schema guardrails
- Release procedures
- Shorthand commands

We should be more focused on research agent behavior.

### Dependencies
- None - pure documentation

### Risk Considerations
- Too verbose = wasted tokens
- Too prescriptive = reduced flexibility
- Need to iterate based on real usage

## Status Updates

*To be added during implementation*