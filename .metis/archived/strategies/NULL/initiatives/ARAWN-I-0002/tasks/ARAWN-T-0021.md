---
id: implement-combined-recall-queries
level: task
title: "Implement combined recall queries"
short_code: "ARAWN-T-0021"
created_at: 2026-01-28T04:11:26.722888+00:00
updated_at: 2026-01-28T05:05:32.072272+00:00
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

# Implement combined recall queries

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0002]]

## Objective

Implement combined recall queries that blend vector similarity search with knowledge graph context. This is the primary retrieval interface for the agent.

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

- [x] `recall()`: Combined query taking text + embedding + optional filters
- [x] Returns RecallResult with: similar memories, related entities, searched_count, query_time_ms
- [x] Configurable blend of vector vs graph results (vector_weight parameter)
- [x] Support time-range filtering (TimeRange: Today, Week, Month, All)
- [x] Support content-type filtering (ContentType filter)
- [x] Result ranking that considers both similarity and graph centrality (blended score)
- [x] Integration test: realistic recall scenario with mixed content (test_recall_mixed_content_integration)
- [x] Performance test: recall over 100 memories completes in <100ms (test_recall_performance_many_memories)
- [x] `cargo test -p arawn-memory` passes (71 tests with `--test-threads=1`)

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
- Added recall types to `store.rs`:
  - `TimeRange` enum: Today, Week, Month, All
  - `RecallQuery` - builder pattern with limit, time_range, content_types, vector_weight, graph_context
  - `RecallMatch` - memory + distance + score + related_entities
  - `RecallResult` - matches + all_entities + searched_count + query_time_ms
- Implemented `recall()` method:
  - Vector similarity search via sqlite-vec
  - Time range filtering
  - Content type filtering
  - Graph context retrieval for related entities
  - Blended scoring: vector_weight * vector_score + (1 - vector_weight) * graph_score
  - Results sorted by combined score
- Added text search fallbacks:
  - `search_memories()` - simple LIKE search
  - `search_memories_in_range()` - LIKE search with time filter
- Added 12 new tests including:
  - Basic recall
  - Content type filtering
  - Time range filtering
  - Graph context inclusion
  - Vector weight configuration
  - Result ordering
  - Performance test (100 memories < 100ms)
  - Mixed content integration test
- Exported new types from lib.rs
- All 71 tests pass with `--test-threads=1`