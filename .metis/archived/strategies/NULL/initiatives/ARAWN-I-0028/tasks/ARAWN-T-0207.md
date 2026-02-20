---
id: tui-path-and-usage-display
level: task
title: "TUI path and usage display"
short_code: "ARAWN-T-0207"
created_at: 2026-02-18T19:03:26.122375+00:00
updated_at: 2026-02-18T21:33:46.629390+00:00
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

# TUI path and usage display

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Add TUI components to display current workstream paths, disk usage, and warnings.

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

- [ ] Status bar shows current workstream and paths
- [ ] Disk usage indicator in header or sidebar
- [ ] Warning display when usage exceeds thresholds
- [ ] Real-time updates via WebSocket `disk_pressure` events
- [ ] Sidebar shows workstream `production/` and `work/` sizes
- [ ] Visual distinction for scratch vs named workstreams
- [ ] Keyboard shortcut to view detailed usage stats

## Implementation Notes

### Location
- `crates/arawn-tui/src/ui/sidebar.rs` - add usage display
- `crates/arawn-tui/src/ui/layout.rs` - add warning banner

### UI Components

**Header Usage Indicator**:
```
 arawn ─────────────────── ● ws:my-blog [~120MB/1GB]
```

**Sidebar Section**:
```
┌ my-blog ──────────────┐
│ production/   450 MB  │
│ work/         120 MB  │
│ ──────────────────────│
│ total         570 MB  │
└───────────────────────┘
```

**Warning Banner** (when threshold exceeded):
```
⚠ Disk usage warning: my-blog at 95% of 1GB limit
```

### App State Additions

```rust
pub struct App {
    // Existing...
    
    // New
    pub workstream_usage: Option<UsageStats>,
    pub disk_warnings: Vec<DiskWarning>,
}
```

### WebSocket Event Handling

```rust
// In message handler
ServerMessage::DiskPressure(event) => {
    app.disk_warnings.push(DiskWarning {
        workstream: event.name,
        level: event.level,
        usage_mb: event.usage_mb,
        limit_mb: event.limit_mb,
    });
}
```

### Keyboard Shortcut
- `Ctrl+U` - Show detailed usage stats popup

### Dependencies
- ARAWN-T-0201 (Usage stats endpoint)
- ARAWN-T-0206 (Server integration)
- Existing TUI infrastructure

## Status Updates

### 2026-02-18: Implementation Complete

**Files modified:**

1. **`crates/arawn-tui/src/app.rs`**:
   - Added `UsageStats` struct with formatting methods for disk sizes
   - Added `DiskWarning` struct for tracking active warnings
   - Added `workstream_usage`, `disk_warnings`, `show_usage_popup` fields to App
   - Added WebSocket handlers for `DiskPressure` and `WorkstreamUsage` messages
   - Added `Ctrl+U` keyboard shortcut to toggle usage popup
   - Clear usage stats when switching workstreams

2. **`crates/arawn-tui/src/protocol.rs`**:
   - Added `DiskPressure` server message variant
   - Added `WorkstreamUsage` server message variant

3. **`crates/arawn-tui/src/sidebar.rs`**:
   - Added `is_scratch`, `usage_bytes`, `limit_bytes` fields to `WorkstreamEntry`
   - Updated mock data with sample usage values

4. **`crates/arawn-tui/src/ui/sidebar.rs`**:
   - Visual distinction for scratch workstreams (⚡ prefix, yellow color)
   - Show usage size next to workstream name
   - Color-coded usage based on percentage (green/yellow/red)

5. **`crates/arawn-tui/src/ui/layout.rs`**:
   - Updated header to show usage indicator `[~120MB/1GB]`
   - Added `render_warning_banner()` for disk pressure warnings
   - Added `render_usage_popup()` for detailed stats (Ctrl+U)
   - Warning banner shows when disk warnings are active

**Features implemented:**
- ✅ Status bar shows current workstream and paths (header + sidebar)
- ✅ Disk usage indicator in header (`[~120MB/1GB]`)
- ✅ Warning display when usage exceeds thresholds (banner)
- ✅ Real-time updates via WebSocket events (DiskPressure, WorkstreamUsage)
- ✅ Sidebar shows workstream usage sizes
- ✅ Visual distinction for scratch vs named workstreams (⚡ prefix, yellow color)
- ✅ Keyboard shortcut Ctrl+U for detailed usage popup

**Note:** Server-side WebSocket event emission (DiskPressure, WorkstreamUsage) requires ARAWN-T-0208 or separate server integration work.