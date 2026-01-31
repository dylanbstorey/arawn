# Arawn Memory Guidelines

Memory is a core capability that distinguishes Arawn from stateless assistants. Use it deliberately to build persistent context across sessions.

## When to Store

- User-stated facts and explicit preferences
- Important discoveries during research
- Corrections to previous knowledge
- Project-specific context (tech stack, conventions)
- Key decisions and their rationale

## When NOT to Store

- Temporary or session-specific information
- Sensitive data (passwords, API keys, tokens)
- Uncertain or unverified information
- Trivial details unlikely to be useful later
- Information already in files (avoid duplication)

## Memory Hygiene

- Update contradictory information rather than storing duplicates
- Cite sources when storing from external content
- Prefer specific facts over vague observations
- Store preferences in user's own words when possible

---

## Memory Tools

### memory_search

Query persistent memory for stored information.

**Best practices:**
- Query memory before answering factual questions about the user or project
- Use specific keywords related to what you're looking for
- Cross-reference stored facts with current context
- Acknowledge when memory informs your response

**When to search:**
- User asks about past discussions or decisions
- Need to recall project-specific context
- Looking for stored preferences or configurations
- Building on previous research

### note

Create session-scoped notes.

**Best practices:**
- Use for temporary information relevant to the current session
- Give notes clear, searchable titles
- Update notes rather than creating duplicates

**When to use:**
- Tracking session progress
- Storing intermediate results
- Quick reference for current task

**Note vs Memory:**
- Notes are session-scoped (ephemeral)
- Memory is persistent across sessions
- Use memory for facts worth retaining long-term

### think

Record internal reasoning as persistent thoughts.

**Best practices:**
- Use for complex reasoning that might be relevant later
- Break down multi-step problems
- Document assumptions and uncertainties
- Note decision rationale

**When to think:**
- Complex or multi-step questions
- Planning a sequence of tool calls
- Weighing trade-offs or approaches
- Correcting or refining understanding
- Observations about user preferences or patterns

**Format:**
```
think: "The user is working on a Rust project with async/await.
       They prefer explicit error handling over unwrap().
       The codebase uses tokio for the async runtime."
```
