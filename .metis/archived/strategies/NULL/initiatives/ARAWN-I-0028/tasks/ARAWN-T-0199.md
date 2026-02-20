---
id: clone-repo-operation
level: task
title: "Clone repo operation"
short_code: "ARAWN-T-0199"
created_at: 2026-02-18T19:03:15.635736+00:00
updated_at: 2026-02-18T21:14:11.676237+00:00
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

# Clone repo operation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement git repository cloning into workstream `production/` directory with API endpoint.

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

- [ ] `DirectoryManager.clone_repo()` clones git repo into `production/`
- [ ] `POST /api/v1/workstreams/:ws/clone` endpoint
- [ ] Uses system git (relies on user's SSH keys / credential helpers)
- [ ] Optional custom directory name
- [ ] Returns clone path and HEAD commit hash
- [ ] Error handling for auth failures, invalid URLs
- [ ] Integration test with public repo

## API Contract

```
POST /api/v1/workstreams/:ws/clone
Request:  { "url": "https://github.com/user/repo.git", "name": "my-repo" }
Response: { "path": "production/my-repo", "commit": "abc123def456" }
Error:    { "error": "clone_failed", "message": "Authentication failed" }
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/directory.rs` - add `clone_repo()` method
- `crates/arawn-server/src/routes/workstreams.rs` - add endpoint

### Core Logic

```rust
impl DirectoryManager {
    pub fn clone_repo(
        &self,
        workstream: &str,
        url: &str,
        name: Option<&str>,
    ) -> Result<CloneResult, DirectoryError> {
        let prod_path = self.workstream_path(workstream).join("production");
        
        // Derive directory name from URL if not provided
        let repo_name = name.unwrap_or_else(|| {
            url.rsplit('/').next()
                .and_then(|s| s.strip_suffix(".git"))
                .unwrap_or("repo")
        });
        
        let dest = prod_path.join(repo_name);
        
        // Check if already exists
        if dest.exists() {
            return Err(DirectoryError::AlreadyExists(dest));
        }
        
        // Run git clone
        let output = Command::new("git")
            .args(["clone", url, dest.to_str().unwrap()])
            .output()?;
        
        if !output.status.success() {
            return Err(DirectoryError::CloneFailed {
                url: url.to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        
        // Get HEAD commit
        let commit = self.get_head_commit(&dest)?;
        
        Ok(CloneResult { path: dest, commit })
    }
}
```

### Authentication
Relies on system git configuration:
- SSH keys in `~/.ssh`
- Git credential helpers
- `.netrc` file

No in-app credential management.

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- System `git` command

## Status Updates

### Session 1 (2026-02-18)
- Added error variants: `AlreadyExists`, `CloneFailed`, `GitNotFound`
- Implemented `CloneResult` struct with path and commit fields
- Implemented `DirectoryManager::clone_repo()` method with:
  - Workstream validation
  - Repository name derivation from URL (strips `.git` suffix)
  - Destination existence check
  - Git availability check
  - Git clone execution using system `git`
  - HEAD commit extraction
- Added helper methods: `repo_name_from_url()`, `is_git_available()`, `get_head_commit()`
- Added `Conflict` error variant to ServerError (HTTP 409)
- Added API endpoint `POST /api/v1/workstreams/:ws/clone`
- Created `CloneRepoRequest` and `CloneRepoResponse` types
- Added 8 unit tests + 2 integration tests (ignored by default)
- Integration test with `octocat/Hello-World` repo passes
- All 109 tests pass (48 in directory module, 5 doctests)

### Acceptance Criteria Status
- [x] `DirectoryManager.clone_repo()` clones git repo into `production/`
- [x] `POST /api/v1/workstreams/:ws/clone` endpoint
- [x] Uses system git (relies on user's SSH keys / credential helpers)
- [x] Optional custom directory name
- [x] Returns clone path and HEAD commit hash
- [x] Error handling for auth failures, invalid URLs
- [x] Integration test with public repo