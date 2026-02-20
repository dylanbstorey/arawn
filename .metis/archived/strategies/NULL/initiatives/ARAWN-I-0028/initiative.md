---
id: workstream-and-session-path
level: initiative
title: "Workstream and Session Path Management"
short_code: "ARAWN-I-0028"
created_at: 2026-02-18T03:03:38.703657+00:00
updated_at: 2026-02-19T18:10:56.022984+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: workstream-and-session-path
---

# Workstream and Session Path Management Initiative

## Context

Arawn server runs headless with TUI/clients connecting remotely. Currently:
- Agents have unrestricted filesystem access
- No convention for where workstream/session data lives
- No sandboxing between workstreams or sessions

Need a convention-based approach (similar to Claude) where the filesystem structure provides natural isolation.

## Goals & Non-Goals

**Goals:**
- Convention-based directory structure for workstreams and sessions
- Separate **production** (deliverables) from **working** (session scratch) areas
- Production = persistent outputs (repos, synced folders, finished products)
- Working = intermediate session-specific files
- **Promote** workflow: move finished work → production
- **Export** capability: publish production content externally
- Scratch sessions isolated; workstream sessions access full workstream

**Non-Goals:**
- User-specified arbitrary paths (convention over configuration)
- OS-level process sandboxing (path validation only)
- Cross-workstream access
- Real-time sync implementation (export is explicit, not live sync)

## Use Cases

### UC-1: Create Workstream
- **Actor**: User via TUI or API
- **Scenario**: User creates workstream "my-blog"
- **Outcome**: Directories created: `production/` and `work/`

### UC-2: Scratch Session (Isolated)
- **Actor**: User starts chatting without selecting workstream
- **Scenario**: New session in scratch workstream
- **Outcome**: Agent can only access `scratch/sessions/<id>/work/`

### UC-3: Workstream Session (Full Access)
- **Actor**: User starts session in named workstream
- **Scenario**: Session created, agent needs to work on blog
- **Outcome**: Agent can access `production/` + own `work/`

### UC-4: Clone Repo into Production
- **Actor**: Agent or user
- **Scenario**: Clone blog repo to start working
- **Outcome**: Repo cloned to `my-blog/production/blog-repo/`

### UC-5: Promote Work to Production
- **Actor**: Agent after completing task
- **Scenario**: Finished draft in `work/`, ready to commit
- **Outcome**: File moved to `production/`, committed to repo

### UC-6: Export Production Content
- **Actor**: User via command
- **Scenario**: Export finished report to Dropbox folder
- **Outcome**: File copied from `production/` to external path

### UC-7: Attach Scratch Session to Workstream
- **Actor**: User via TUI or API
- **Scenario**: Started in scratch, built something useful, want to continue in a real workstream
- **Outcome**: Session moved from `scratch/sessions/<id>/` to workstream, files migrated to `work/`, access expanded to full workstream

## Architecture

### Directory Convention

```
~/.arawn/
├── workstreams/
│   ├── scratch/                          # Special: isolated per-session
│   │   └── sessions/
│   │       ├── <session-id-1>/work/      # S1's isolated work
│   │       └── <session-id-2>/work/      # S2's isolated work
│   │
│   ├── my-blog/                          # Named workstream
│   │   ├── production/                   # Deliverables (shared)
│   │   │   ├── blog-repo/               # Git clone
│   │   │   └── assets/                  # Finished files
│   │   └── work/                         # Shared working area
│   │
│   └── data-project/
│       ├── production/
│       └── work/
│
└── config/                               # Server config (inaccessible)
```

### Area Semantics

| Area | Purpose | Lifecycle | Sharing |
|------|---------|-----------|---------|
| `production/` | Deliverables: repos, finished files | Persistent | All sessions in workstream |
| `work/` (named ws) | In-progress work | May be cleaned up | All sessions in workstream |
| `sessions/<id>/work/` (scratch) | Isolated scratch space | Ephemeral | Single session only |

### Full Access Matrix

**Directory Structure Reference:**
```
~/.arawn/workstreams/
├── scratch/sessions/
│   ├── S1/work/
│   └── S2/work/
├── blog/
│   ├── production/
│   └── work/
└── data/
    ├── production/
    └── work/
```

**Access Matrix (✓ = allowed, ✗ = denied):**

| Session | scratch/S1/work | scratch/S2/work | blog/production | blog/work | data/production | data/work |
|---------|:---:|:---:|:---:|:---:|:---:|:---:|
| **S1** (scratch) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| **S2** (scratch) | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ |
| **S3** (blog) | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ |
| **S4** (blog) | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ |
| **S5** (data) | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ |

**Rules Summary:**

| Rule | Description |
|------|-------------|
| **Scratch isolation** | Scratch sessions see ONLY their own work/ |
| **Workstream shared** | Named workstream sessions share both production/ AND work/ |
| **Workstream isolation** | No cross-workstream access ever |
| **Config exclusion** | ~/.arawn/config/ never accessible |

### Operations

| Operation | Description |
|-----------|-------------|
| **Promote** | Move files from `work/` → `production/` |
| **Export** | Copy/publish from `production/` → external destination |
| **Clone** | Clone repo into `production/` |
| **Attach** | Move scratch session to workstream, migrate files to shared `work/` |

### Filesystem Monitoring

| Event | Trigger | Action |
|-------|---------|--------|
| File change in production/ | inotify/FSEvents | Notify connected sessions, update file index |
| File change in work/ | inotify/FSEvents | Update session context |
| External change detection | Polling fallback | Sync state if monitoring missed events |

### Cleanup Policies

| Target | Policy | Trigger |
|--------|--------|---------|
| Scratch session work/ | Delete after session inactive for N days | Scheduled task |
| Named workstream work/ | Optional manual cleanup, warn if large | User command |
| Empty sessions/ dirs | Remove automatically | On session delete |
| Orphaned files | Detect files not tracked by any session | Audit command |

## Detailed Design

### Directory Manager

```rust
pub struct DirectoryManager {
    base_path: PathBuf,  // ~/.arawn
}

impl DirectoryManager {
    /// Get allowed paths for a session
    pub fn allowed_paths(&self, workstream: &str, session_id: &str) -> Vec<PathBuf> {
        let ws_path = self.base_path.join("workstreams").join(workstream);
        
        if workstream == "scratch" {
            // Scratch: isolated per-session work folder only
            vec![ws_path.join("sessions").join(session_id).join("work")]
        } else {
            // Named workstream: shared production + work
            vec![ws_path.join("production"), ws_path.join("work")]
        }
    }
    
    /// Create workstream directory structure
    pub fn create_workstream(&self, name: &str) -> Result<PathBuf> {
        let ws_path = self.base_path.join("workstreams").join(name);
        fs::create_dir_all(ws_path.join("production"))?;
        fs::create_dir_all(ws_path.join("work"))?;
        Ok(ws_path)
    }
    
    /// Create scratch session work directory
    pub fn create_scratch_session(&self, session_id: &str) -> Result<PathBuf> {
        let work_path = self.base_path
            .join("workstreams/scratch/sessions")
            .join(session_id).join("work");
        fs::create_dir_all(&work_path)?;
        Ok(work_path)
    }
    
    /// Promote file from work to production
    pub fn promote(&self, workstream: &str, src: &Path, dest: &Path) -> Result<()>;
    
    /// Export file from production to external location
    pub fn export(&self, workstream: &str, src: &Path, dest: &Path) -> Result<()>;
    
    /// Attach scratch session to a workstream
    /// Moves files from scratch/<session>/work/ to <workstream>/work/
    pub fn attach_session(&self, session_id: &str, target_workstream: &str) -> Result<()> {
        let src = self.base_path
            .join("workstreams/scratch/sessions")
            .join(session_id).join("work");
        let dest = self.base_path
            .join("workstreams").join(target_workstream).join("work");
        
        // Move all files from scratch work to workstream work
        for entry in fs::read_dir(&src)? {
            let entry = entry?;
            fs::rename(entry.path(), dest.join(entry.file_name()))?;
        }
        
        // Clean up empty scratch session dir
        fs::remove_dir_all(src.parent().unwrap())?;
        Ok(())
    }
}
```

### Path Validation

```rust
pub struct PathValidator {
    allowed_paths: Vec<PathBuf>,
}

impl PathValidator {
    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathError> {
        let canonical = path.canonicalize()?;
        for allowed in &self.allowed_paths {
            if canonical.starts_with(allowed) {
                return Ok(canonical);
            }
        }
        Err(PathError::NotAllowed { 
            path: path.to_path_buf(),
            allowed: self.allowed_paths.clone(),
        })
    }
}
```

### Agent Integration

Tool execution flow:
1. Agent receives tool call (read/write file, etc.)
2. Get allowed paths from DirectoryManager
3. Validate target path against allowed paths
4. Execute tool or return PathError

### Export Destinations (Future)

```rust
pub enum ExportDestination {
    LocalPath(PathBuf),
    // Future: cloud integrations
    // S3 { bucket: String, key: String },
    // Dropbox { path: String },
}
```

## Alternatives Considered

1. **User-specified paths** - Rejected: Convention over configuration is simpler
2. **Symlink-based sandboxing** - Rejected: Complexity, cross-platform issues
3. **No scratch isolation** - Rejected: Scratch sessions shouldn't access each other

## Implementation Plan

### Phase 1: Directory Convention
- Implement DirectoryManager in arawn-workstream
- Create `production/` and `work/` on workstream creation
- Create `sessions/<id>/work/` for scratch sessions
- Configure base path from server config

### Phase 2: Path Validation
- Implement PathValidator struct
- Integrate with agent tool execution
- Clear error messages for path violations

### Phase 3: Promote Operation
- Implement `promote()` to move work → production
- Validate paths on both ends
- API endpoint for promote

### Phase 4: Export Operation
- Implement `export()` to copy production → external
- Local path support first
- API endpoint for export

### Phase 5: Attach Operation
- Implement `attach_session()` to move scratch → workstream
- Migrate files from isolated work/ to shared work/
- Update session record with new workstream_id
- API endpoint for attach

### Phase 6: Filesystem Monitoring
- Implement FileWatcher using notify crate (cross-platform)
- Watch production/ and work/ directories
- Emit events to connected sessions via WebSocket
- Polling fallback for edge cases

### Phase 7: Cleanup
- Scheduled task for scratch session cleanup (N days inactive)
- API endpoint for manual work/ cleanup
- Audit command to detect orphaned files
- Configurable retention policies

### Phase 8: Integration
- Wire DirectoryManager into server state
- Pass allowed paths to agent on session start
- Update existing workstream/session creation flows
- TUI display of current paths and disk usage

---

## Design Decisions

### Edge Case Handling

| Scenario | Decision | Rationale |
|----------|----------|-----------|
| **Attach file conflict** | Use session UUID as folder name; on collision, assign new session ID | UUIDs rarely collide; reassigning ID is simpler than merge logic |
| **Concurrent promote** | Append `(1)` to filename, alert session via WebSocket | User-friendly; alert lets them decide next steps |
| **In-flight write during promote** | Fail operation | Sessions are sequential; this shouldn't happen in practice |
| **Attach to non-existent workstream** | Defend at orchestration layer | Attach is part of existing user flow that validates target |
| **Path traversal (`../../../etc`)** | Full lockdown with VM-style isolation if needed | Critical security - cannot trust prompts alone |

### API Contracts

#### Existing Endpoints (no changes needed)

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/v1/workstreams` | Create workstream |
| GET | `/api/v1/workstreams` | List workstreams |
| GET | `/api/v1/workstreams/:id` | Get workstream details |
| PATCH | `/api/v1/workstreams/:id` | Update workstream |
| DELETE | `/api/v1/workstreams/:id` | Archive workstream |
| POST | `/api/v1/workstreams/:id/promote` | Promote scratch → named workstream |
| PATCH | `/api/v1/sessions/:id` | Update session (includes `workstream_id` for reassignment) |

#### New Endpoints

**File Promote** - Move file from `work/` to `production/`
```
POST /api/v1/workstreams/:ws/files/promote
Request:  { "source": "draft.md", "destination": "blog/final.md" }
Response: { "path": "production/blog/final.md", "bytes": 1234 }
Conflict: { "error": "file_conflict", "renamed_to": "final(1).md" }
```

**File Export** - Copy file from `production/` to external path
```
POST /api/v1/workstreams/:ws/files/export
Request:  { "source": "report.pdf", "destination": "/mnt/dropbox/reports/" }
Response: { "exported_to": "/mnt/dropbox/reports/report.pdf" }
```

**Clone Repo** - Clone git repository into `production/`
```
POST /api/v1/workstreams/:ws/clone
Request:  { "url": "https://github.com/user/repo.git", "name": "my-repo" }
Response: { "path": "production/my-repo", "commit": "abc123" }
```

**Usage Stats** - Get disk usage for workstream
```
GET /api/v1/workstreams/:ws/usage
Response: { 
  "production_mb": 450, 
  "work_mb": 120, 
  "sessions": [{ "id": "abc", "mb": 45 }],
  "warnings": ["work approaching 1GB limit"]
}
```

**Manual Cleanup** - Trigger cleanup of work area
```
POST /api/v1/workstreams/:ws/cleanup
Request:  { "target": "work", "older_than_days": 7 }
Response: { "deleted_files": 12, "freed_mb": 85 }
```

#### Enhanced Existing Endpoint

**Session Reassign** - Existing `PATCH /sessions/:id` with `workstream_id`
- **Enhancement**: When moving from scratch → named workstream, automatically migrate files from `scratch/sessions/<id>/work/` to `<workstream>/work/`
- Files placed in session-named subfolder to avoid conflicts

#### WebSocket Events

```json
// File system changes
{ "event": "fs_change", "workstream": "blog", "path": "production/post.md", "action": "modified" }

// Disk pressure alerts  
{ "event": "disk_pressure", "level": "warning", "scope": "workstream", "name": "blog", "usage_mb": 950, "limit_mb": 1024 }

// Operation alerts (conflicts, renames, etc.)
{ "event": "alert", "severity": "info", "message": "File renamed to draft(1).md due to conflict", "details": { "original": "draft.md", "actual": "draft(1).md" } }
```

### Error Handling

| Error Type | Response | Client Display |
|------------|----------|----------------|
| `PathNotAllowed` | 403 with allowed paths list | Show allowed paths for context |
| `FileConflict` | 409 with both paths | Let client decide resolution |
| `WorkstreamNotFound` | 404 with suggestion to create | Prompt to create workstream |
| `IOError` | 500 with context (operation, path) | Show what failed and where |

### Configuration

| Setting | Default | Configurable | Notes |
|---------|---------|--------------|-------|
| Base path | `~/.arawn` | Yes (env/config) | Support mounted drives, PVCs |
| Scratch cleanup interval | 7 days | Yes (cloacina) | Use cloacina for scheduling |
| Total usage warning | 10 GB | Yes | Warn on FS pressure |
| Workstream usage warning | 1 GB | Yes | Per-workstream limit |
| Session usage warning | 200 MB | Yes | Per-session limit |
| FS monitoring | On | Yes (on/off) | Enable/disable file watching |
| Polling fallback interval | 30s | Yes | When inotify/FSEvents unavailable |
| Event debounce | 500ms | Yes | Batch rapid file changes |

### Implementation Details

**Non-existent path validation** (write to new file):
```rust
// Canonicalize parent, then append filename
let parent = path.parent()
    .ok_or(PathError::NoParent)?
    .canonicalize()?;
validate_prefix(&parent, &allowed_paths)?;
let validated = parent.join(path.file_name().unwrap());
```

**Filesystem events broadcast**:
- All changes in `production/` and `work/` directories
- Clients filter as needed
- Debounced at 500ms to batch rapid changes

**Inactive session definition** (for scratch cleanup):
- Session is inactive when last turn completed > N days ago (default: 7 days)
- Based on actual usage, not connection state
- Cleanup scheduled via cloacina

### Shell Sandboxing

**Decision**: Shell execution requires sandbox (Option C - no shell without sandbox).

**Implementation**: Use `sandbox-runtime` crate (Rust port of Anthropic's sandbox-runtime)

| Platform | Mechanism | Dependencies |
|----------|-----------|--------------|
| macOS | sandbox-exec (Seatbelt) | None (built-in) |
| Linux | bubblewrap | `bubblewrap`, `socat` |
| WSL2 | bubblewrap | `bubblewrap`, `socat` |
| WSL1 | Not supported | - |

**Behavior**:
- On startup, check for sandbox availability
- If dependencies missing: shell commands disabled with clear error message
- No fallback to unsandboxed execution (security critical)

**Filesystem Model** (per sandbox-runtime):
- **Read**: Deny-only pattern (everything allowed except blocked paths like `~/.ssh`, `/etc`)
- **Write**: Allow-only pattern (nothing allowed except workstream `production/` and `work/`)

**Related**: ARAWN-T-0193 tracks easy dependency installation for distribution.

### Security: Path Traversal Prevention

Path validation must be **defense in depth**:

1. **Canonicalization** - Resolve symlinks, `..`, etc. before checking
2. **Prefix matching** - Canonical path must start with allowed prefix
3. **No symlink following outside boundary** - Reject if symlink target escapes
4. **Consider VM isolation** - If validation proves insufficient, escalate to container/VM isolation

```rust
pub fn validate_path(&self, path: &Path) -> Result<PathBuf, PathError> {
    // 1. Canonicalize (resolves .., symlinks)
    let canonical = path.canonicalize()
        .map_err(|_| PathError::InvalidPath(path.to_path_buf()))?;
    
    // 2. Check against allowed prefixes
    for allowed in &self.allowed_paths {
        let allowed_canonical = allowed.canonicalize()?;
        if canonical.starts_with(&allowed_canonical) {
            return Ok(canonical);
        }
    }
    
    Err(PathError::NotAllowed { 
        path: path.to_path_buf(),
        allowed: self.allowed_paths.clone(),
    })
}
```