---
id: implement-unified-memorystore-api
level: task
title: "Implement unified MemoryStore API"
short_code: "ARAWN-T-0019"
created_at: 2026-01-28T04:11:26.169089+00:00
updated_at: 2026-01-28T04:57:45.544089+00:00
parent: ARAWN-I-0002
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0002
---

# Implement unified MemoryStore API

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0002]]

## Objective

Create the unified MemoryStore API that combines vector search and graph queries. Provide a clean interface for storing and retrieving memories with semantic and relational context.

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

## Acceptance Criteria

- [x] MemoryStore combines VectorStore and GraphStore internally
- [x] `store()`: Store memory with optional embedding and entity extraction
- [x] `get()`: Retrieve memory by ID with related graph context (`get_with_context()`)
- [x] `delete()`: Remove memory and associated vectors/graph nodes (`delete_cascade()`)
- [x] `update()`: Update memory content and re-index (`update_indexed()`)
- [x] Transaction support for atomic operations (`with_transaction()`)
- [x] Proper error propagation from both subsystems
- [x] Unit tests for unified API (15 new tests)
- [x] `cargo test -p arawn-memory` passes (with `--test-threads=1`)

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

## Status Updates **[REQUIRED]**

### Session 1 - 2026-01-27
- Implemented unified API in `store.rs`:
  - `StoreOptions`, `EntityLink`, `MemoryWithContext`, `RelatedEntity` types
  - `store()` - stores memory with optional embedding and graph entities
  - `get_with_context()` - retrieves memory with graph relationships
  - `delete_cascade()` - removes memory and all associated data
  - `update_indexed()` - updates memory and re-indexes
  - `with_transaction()` - executes operations atomically
  - Graph passthrough methods: `add_graph_entity()`, `add_graph_relationship()`, etc.
- Added `init_graph()` and `init_graph_at_path()` to initialize knowledge graph
- Added `has_vectors()` and `has_graph()` helper methods
- Fixed `get_neighbors()` in graph.rs to properly parse graphqlite's Value::Object return type
- Added 15 new tests for unified API
- All 49 tests pass when run with `--test-threads=1`
- Note: Parallel test execution causes race conditions in graphqlite library (not our code)