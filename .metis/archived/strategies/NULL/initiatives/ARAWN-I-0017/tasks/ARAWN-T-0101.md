---
id: implement-llm-extraction-prompt
level: task
title: "Implement LLM extraction prompt and structured JSON parsing"
short_code: "ARAWN-T-0101"
created_at: 2026-01-31T04:09:07.141582+00:00
updated_at: 2026-02-01T03:51:15.053182+00:00
parent: ARAWN-I-0017
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0017
---

# Implement LLM extraction prompt and structured JSON parsing

## Objective

Create the extraction prompt that instructs an LLM to extract entities, facts, and relationships from a conversation history, returning structured JSON. Implement the JSON parser that validates and deserializes the LLM output into Rust types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ExtractionPrompt` struct that formats conversation history into an extraction prompt
- [ ] `ExtractionResult` struct: `entities: Vec<Entity>`, `facts: Vec<Fact>`, `relationships: Vec<Relationship>`
- [ ] `Entity` struct: `name`, `entity_type`, `context`
- [ ] `Fact` struct: `subject`, `predicate`, `object`, `confidence: ConfidenceSource`
- [ ] `Relationship` struct: `from`, `relation`, `to`
- [ ] JSON parser with error recovery (handle malformed LLM output gracefully — partial results OK)
- [ ] Prompt includes few-shot examples for consistent output format
- [ ] Tests: parse valid JSON, parse partial/malformed JSON, prompt formatting

## Implementation Notes

### Files
- `crates/arawn-agent/src/indexing/` — new module
- `crates/arawn-agent/src/indexing/extraction.rs` — prompt + parser
- `crates/arawn-agent/src/indexing/types.rs` — ExtractionResult, Entity, Fact, Relationship

### Dependencies
- ARAWN-T-0095 (ConfidenceSource enum)
- ARAWN-T-0100 (indexing config for backend/model selection)

## Status Updates

### Session — 2026-01-31
- Created `crates/arawn-agent/src/indexing/` module (mod.rs, types.rs, extraction.rs)
- `ExtractionResult` struct with `entities`, `facts`, `relationships` (all serde-default Vec)
- `ExtractedEntity`: name, entity_type, optional context
- `ExtractedFact`: subject, predicate, object, confidence (defaults to "inferred")
- `ExtractedRelationship`: from, relation, to
- `ExtractionPrompt::build()` formats conversation into extraction prompt with system instruction + few-shot example
- `parse_extraction()` with error recovery: strips markdown code fences, extracts JSON from surrounding text, returns empty on failure
- Prompt includes detailed entity_type vocabulary, confidence definitions, dot-notation convention for subjects
- 16 new tests: type deserialization (valid, partial, empty, defaults), prompt building, JSON parsing (valid, fenced, surrounded, malformed, partial), strip_code_fences, extract_json_object
- All checks + tests pass (732+ tests, 0 failures)