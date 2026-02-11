---
id: workflow-interface
level: initiative
title: "Workflow Interface"
short_code: "ARAWN-I-0024"
created_at: 2026-02-10T03:55:59.025698+00:00
updated_at: 2026-02-10T03:55:59.025698+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: workflow-interface
---

# Workflow Interface Initiative

## Context

Workstream management UI for persistent conversation contexts. Unlike sessions (ephemeral), workstreams persist across multiple sessions and maintain long-running context for projects or topics.

**Backend Integration:**
- `GET /api/v1/workstreams` - list workstreams
- `POST /api/v1/workstreams` - create workstream
- `GET /api/v1/workstreams/{id}` - get workstream details
- `WS /ws?workstream={id}` - connect to workstream context

## Goals & Non-Goals

**Goals:**
- Create and manage named workstreams
- Switch between workstream contexts
- Visual distinction between sessions and workstreams
- Workstream metadata (name, description, message count, last active)
- Resume workstreams with full context

**Non-Goals:**
- Chat functionality (separate initiative)
- Session management (separate initiative)
- Workstream sharing/collaboration (future consideration)

## UI/UX Design

### Main Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Workstreams                                         [+ Create]  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Active Workstreams                                             │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ 📁 Q4 Architecture Redesign                    ★ Pinned    ││
│  │ Last active: 2 hours ago  │  127 messages                   ││
│  │ [Resume] [Edit] [Archive]                                   ││
│  └─────────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────────┐│
│  │ 📁 Frontend Migration                                       ││
│  │ Last active: Yesterday  │  89 messages                      ││
│  │ [Resume] [Edit] [Archive]                                   ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                 │
│  Archived                                              [Show ▼] │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Workstream Card

```
┌─────────────────────────────────────────────────────────────────┐
│ 📁 Workstream Name                                    [★] [⋮]  │
│ Description or summary of the workstream context...             │
├─────────────────────────────────────────────────────────────────┤
│ 📅 Created: Jan 15  │  🕐 Last: 2h ago  │  💬 127 msgs         │
├─────────────────────────────────────────────────────────────────┤
│                    [Resume Conversation]                        │
└─────────────────────────────────────────────────────────────────┘
```

### Create Workstream Modal

```
┌─────────────────────────────────────────────────────┐
│ Create Workstream                              [×] │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Name *                                              │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Q1 Product Roadmap                              │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ Description                                         │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Planning and tracking for Q1 features...       │ │
│ │                                                 │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ Initial Context (optional)                          │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Working on a Vue 3 + Tauri app. Focus on...    │ │
│ │                                                 │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│              [Cancel]  [Create Workstream]          │
└─────────────────────────────────────────────────────┘
```

### Workstream Indicator in Chat

When in a workstream, show persistent indicator:

```
┌─────────────────────────────────────────────────────────────┐
│ 📁 Q4 Architecture Redesign                    [Switch] [×] │
├─────────────────────────────────────────────────────────────┤
│ ... chat messages ...                                       │
```

### User Flows

1. **Create Workstream**: Click Create → Fill form → Start chatting
2. **Resume Workstream**: Click Resume → Chat loads with full history context
3. **Switch Workstream**: Click Switch → Select from list → Context changes
4. **Pin Workstream**: Click star → Moves to top of list
5. **Archive Workstream**: Click Archive → Moves to archived section

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+W` | Open workstream list |
| `Cmd+Shift+W` | Create new workstream |
| `Cmd+1-9` | Quick switch to pinned workstreams |

## Component Architecture

```
WorkstreamView/
├── WorkstreamList.vue       # Main list container
│   ├── WorkstreamCard.vue   # Individual workstream display
│   ├── WorkstreamGroup.vue  # Active/Archived grouping
│   └── WorkstreamSearch.vue # Search/filter
├── WorkstreamCreate.vue     # Creation modal
├── WorkstreamEdit.vue       # Edit modal
├── WorkstreamIndicator.vue  # Header indicator when active
└── WorkstreamSwitcher.vue   # Quick switch dropdown
```

## State Management (Pinia)

```typescript
interface WorkstreamStore {
  workstreams: Workstream[]
  activeWorkstreamId: string | null
  isLoading: boolean
  
  // Computed
  activeWorkstreams: Workstream[]
  archivedWorkstreams: Workstream[]
  pinnedWorkstreams: Workstream[]
  currentWorkstream: Workstream | null
  
  // Actions
  fetchWorkstreams(): Promise<void>
  createWorkstream(data: CreateWorkstream): Promise<string>
  resumeWorkstream(id: string): Promise<void>
  archiveWorkstream(id: string): Promise<void>
  pinWorkstream(id: string): Promise<void>
  updateWorkstream(id: string, data: Partial<Workstream>): Promise<void>
}
```

## Alternatives Considered

1. **Tabs vs List**: List view scales better for many workstreams
2. **Modal vs Page**: Modal for create/edit, page for list
3. **Auto-archive**: Consider auto-archiving after inactivity period

## Implementation Plan

1. Build WorkstreamCard and WorkstreamList components
2. Implement workstream CRUD operations
3. Add workstream context switching
4. Build WorkstreamIndicator for chat header
5. Implement pin/archive functionality
6. Add quick switch keyboard shortcuts
7. Build creation/edit modals
8. Mobile responsive adjustments