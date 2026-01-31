---
id: plugin-manifest-validation
level: task
title: "Plugin Manifest Validation"
short_code: "ARAWN-T-0146"
created_at: 2026-02-07T16:46:12.032107+00:00
updated_at: 2026-02-07T19:46:37.038828+00:00
parent: ARAWN-I-0021
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0021
---

# Plugin Manifest Validation

## Parent Initiative

[[ARAWN-I-0021]] - Interface Enforcement and Defensive Validation

## Objective

Add load-time validation for plugin manifests in `arawn-plugin` crate. Validate required fields, version formats, and capability declarations before plugins are loaded into the registry.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Define `ManifestValidationError` type with rich error context
- [x] Validate required manifest fields (name, version, description)
- [x] Validate version format follows semver
- [x] Validate capability declarations match actual exports
- [x] Add `validate()` method to `PluginManifest` struct
- [x] Validation runs automatically during plugin load
- [x] Clear error messages indicate exactly what's wrong and how to fix
- [x] Unit tests for valid and invalid manifests

## Implementation Notes

### Technical Approach

1. Create `validation.rs` module in `arawn-plugin`
2. Define `ManifestValidationError` enum with variants:
   - `MissingField { field: &'static str }`
   - `InvalidVersion { version: String, reason: String }`
   - `CapabilityMismatch { declared: Vec<String>, actual: Vec<String> }`
3. Implement `PluginManifest::validate(&self) -> Result<(), ManifestValidationError>`
4. Call validation in plugin loader before registering

### Files to Modify

- `crates/arawn-plugin/src/validation.rs` (new)
- `crates/arawn-plugin/src/manifest.rs`
- `crates/arawn-plugin/src/loader.rs`

### Dependencies

None - this is a foundational task

## Status Updates

### 2026-02-07: Implementation Complete

**Created `validation.rs` module:**
- `ManifestValidationError` enum with rich error context
  - `MissingField { field, hint }` - for required field violations
  - `InvalidField { field, message }` - for format violations  
  - `InvalidVersion { version, reason }` - for semver errors
  - `CapabilityMismatch { capability, declared, actual }` - for mismatched exports
  - `PathNotFound { field, path }` - for missing declared paths
- `validate_name()` - checks kebab-case, starts with letter, no consecutive hyphens
- `validate_version()` - checks semver format (MAJOR.MINOR.PATCH with optional prerelease)
- `validate_paths_exist()` - checks declared paths exist on disk
- `count_discovered_items()` - counts items at paths for capability matching

**Enhanced `manifest.rs`:**
- Made `validate()` public and comprehensive (now calls validation module)
- Added `validate_paths()` for checking declared paths exist
- Added `capability_summary()` for declared vs discovered matching
- Added `CapabilitySummary` struct with `has_errors()`, `warnings()`, `errors()`

**Tests added:**
- 22 new tests in `validation.rs` for name/version/path validation
- 14 new tests in `manifest.rs` for version validation and capability checking

**All 188 arawn-plugin tests pass.**

**Files Modified:**
- `crates/arawn-plugin/src/validation.rs` (new - 278 lines)
- `crates/arawn-plugin/src/manifest.rs` (enhanced validation)
- `crates/arawn-plugin/src/lib.rs` (exports)

**Note:** Also fixed `arawn-server` to handle new `StreamChunk::ToolOutput` variant (from shell streaming work):
- `crates/arawn-server/src/routes/ws.rs` - Added `ServerMessage::ToolOutput`
- `crates/arawn-server/src/routes/chat.rs` - Added `SseToolOutputEvent`

**Workspace builds and all tests pass.**