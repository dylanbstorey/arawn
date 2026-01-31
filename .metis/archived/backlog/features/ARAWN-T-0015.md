---
id: implement-additional-built-in
level: task
title: "Implement additional built-in tools (web, glob, grep, memory)"
short_code: "ARAWN-T-0015"
created_at: 2026-01-28T04:00:42.448274+00:00
updated_at: 2026-01-28T04:07:04.686876+00:00
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

# Implement additional built-in tools (web, glob, grep, memory)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective

Implement additional built-in tools to complete the agent's core capabilities:
- Web tools for fetching URLs and searching the web
- File search tools (glob patterns and grep)
- Memory search tool (stub for future arawn-memory integration)

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `WebFetchTool`: Fetch URL content, extract text from HTML
- [x] `WebSearchTool`: Search the web (supports Brave, Serper, Tavily, DuckDuckGo)
- [x] `GlobTool`: Find files matching glob patterns
- [x] `GrepTool`: Search file contents with regex
- [x] `MemorySearchTool`: Stub for memory search (pending arawn-memory)
- [x] All tools have proper JSON Schema parameter definitions
- [x] Unit tests for each tool (23 new tests)
- [x] `cargo test -p arawn-agent` passes (106 tests total)

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates

### Session 1 - Completed
**Files created:**
- `crates/arawn-agent/src/tools/web.rs` - WebFetchTool, WebSearchTool
- `crates/arawn-agent/src/tools/search.rs` - GlobTool, GrepTool
- `crates/arawn-agent/src/tools/memory.rs` - MemorySearchTool (stub)

**Files modified:**
- `crates/arawn-agent/src/tools/mod.rs` - Added new tool exports
- `crates/arawn-agent/src/lib.rs` - Added new tool exports
- `crates/arawn-agent/Cargo.toml` - Added dependencies (reqwest, scraper, url, urlencoding, glob, regex, walkdir)

**Tools implemented:**
- `WebFetchTool`: HTTP client with HTML text extraction, title/description parsing
- `WebSearchTool`: Multi-provider search (Brave, Serper, Tavily, DuckDuckGo)
- `GlobTool`: File pattern matching with configurable depth and limits
- `GrepTool`: Regex search with file filtering and binary detection
- `MemorySearchTool`: Stub for future arawn-memory integration

**Tests:** 106 tests total (23 new tests for additional tools)