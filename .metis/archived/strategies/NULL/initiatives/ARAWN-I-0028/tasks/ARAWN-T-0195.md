---
id: pathvalidator-implementation
level: task
title: "PathValidator implementation"
short_code: "ARAWN-T-0195"
created_at: 2026-02-18T19:03:12.283762+00:00
updated_at: 2026-02-18T19:35:43.727368+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# PathValidator implementation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement path validation that ensures all file operations stay within allowed boundaries, with defense-in-depth against path traversal attacks.

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

- [x] `PathValidator` struct with configurable allowed paths
- [x] `validate(path)` canonicalizes and checks against allowed prefixes
- [x] Handles non-existent paths (canonicalize parent, append filename)
- [x] Rejects symlinks that escape allowed boundaries
- [x] Clear error messages with `PathError` enum
- [x] Blocks access to system paths (`/etc`, `/usr`, `~/.ssh`, etc.)
- [x] Unit tests for traversal attacks (`../`, symlinks, etc.)

## Implementation Notes

### Location
`crates/arawn-workstream/src/path_validator.rs` (new file)

### Core API

```rust
pub struct PathValidator {
    allowed_paths: Vec<PathBuf>,
    denied_paths: Vec<PathBuf>,  // ~/.ssh, /etc, etc.
}

impl PathValidator {
    pub fn new(allowed: Vec<PathBuf>) -> Self;
    
    /// Validate path is within allowed boundaries
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathError>;
    
    /// Validate for write (path may not exist yet)
    pub fn validate_write(&self, path: &Path) -> Result<PathBuf, PathError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("Path not allowed: {path}")]
    NotAllowed { path: PathBuf, allowed: Vec<PathBuf> },
    
    #[error("Path escapes boundary via symlink")]
    SymlinkEscape { path: PathBuf, target: PathBuf },
    
    #[error("Invalid path: {0}")]
    Invalid(PathBuf),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Validation Algorithm

```rust
pub fn validate_write(&self, path: &Path) -> Result<PathBuf, PathError> {
    // For new files: canonicalize parent, append filename
    let parent = path.parent().ok_or(PathError::Invalid(path.to_path_buf()))?;
    let parent_canonical = parent.canonicalize()?;
    
    // Check parent against allowed paths
    self.check_allowed(&parent_canonical)?;
    
    // Return full validated path
    Ok(parent_canonical.join(path.file_name().unwrap()))
}
```

### Default Denied Paths
- `~/.ssh`
- `~/.gnupg`
- `~/.aws`
- `/etc`
- `/usr`
- `/var`
- `~/.arawn/config`

### Dependencies
- ARAWN-T-0194 (DirectoryManager)

### Risk Considerations
- TOCTOU: Validate at operation time, not just at request time
- Symlink following: Use `O_NOFOLLOW` where possible

## Status Updates

### 2026-02-18 - Implementation Complete

**Created**: `crates/arawn-workstream/src/path_validator.rs`

**Core API implemented**:
- `PathValidator::new(allowed_paths)` - Create with allowed directories
- `PathValidator::with_denied(allowed, denied)` - Create with custom denied paths
- `PathValidator::for_session(dm, workstream, session_id)` - Create from DirectoryManager
- `validate(path)` - Validate for read operations (path must exist)
- `validate_write(path)` - Validate for write operations (parent must exist)
- `validate_for_shell(path)` - Extra strict validation (no shell metacharacters)

**PathError enum**:
- `NotAllowed` - Path outside allowed directories
- `DeniedPath` - Path in sensitive system directory
- `SymlinkEscape` - Symlink points outside allowed boundaries
- `Invalid` - Path has no parent or filename
- `ParentNotFound` - Parent directory doesn't exist (for write)
- `Io` - IO error during validation

**Security features**:
- Canonicalization resolves all symlinks and `..` components
- Symlink escape detection (compares original vs canonical paths)
- Default denied paths: `/etc`, `/usr`, `~/.ssh`, `~/.aws`, `~/.gnupg`, etc.
- Shell metacharacter rejection in `validate_for_shell`

**macOS compatibility**:
- Handles `/var` -> `/private/var` symlink correctly
- Uses canonicalized paths for all comparisons

**Integration with DirectoryManager**:
- `PathValidator::for_session()` creates validator from DM's allowed_paths

**Tests**: 17 unit tests + 1 doc test, all passing
- Thread-safety verified (`Send + Sync`)
- Traversal attack tests
- Symlink escape tests
- Denied path tests
- Write validation tests