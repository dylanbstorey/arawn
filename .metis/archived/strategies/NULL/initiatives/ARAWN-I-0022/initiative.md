---
id: chat-interface
level: initiative
title: "Chat Interface"
short_code: "ARAWN-I-0022"
created_at: 2026-02-10T03:55:57.249208+00:00
updated_at: 2026-02-10T03:55:57.249208+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: chat-interface
---

# Chat Interface Initiative

## Context

The primary interaction surface for Arawn. Users need a conversational interface to interact with the agent, view streaming responses, observe tool executions, and manage message history within a session.

**Tech Stack:**
- Vue 3 + Composition API
- Tailwind CSS
- Pinia (state management)
- Tauri 2.0 (desktop + Android)
- WebSocket for streaming

**Backend Integration:**
- `POST /api/v1/chat` - synchronous
- `POST /api/v1/chat/stream` - SSE streaming
- `WS /ws` - WebSocket for bidirectional

## Goals & Non-Goals

**Goals:**
- Real-time streaming message display with markdown rendering
- Tool execution visibility (start/progress/result)
- Message input with multi-line support
- Copy, regenerate, and edit message actions
- Mobile-responsive layout
- Keyboard shortcuts for power users

**Non-Goals:**
- Session switching (separate initiative)
- Workstream management (separate initiative)
- Voice input/output (future consideration)
- File upload/attachment (future consideration)

## UI/UX Design

### Layout Structure

```
┌─────────────────────────────────────────────────────────┐
│ Header: [Session Title] [Status] [Settings]            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ User Message                                     │   │
│  │ "Explain async/await in Rust"                   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ Assistant Message                                │   │
│  │ ┌─────────────────────────────────┐             │   │
│  │ │ 🔧 Tool: shell                   │             │   │
│  │ │ Command: cargo doc --open       │ [Expand]    │   │
│  │ │ Status: ✓ Complete              │             │   │
│  │ └─────────────────────────────────┘             │   │
│  │                                                  │   │
│  │ Async/await in Rust allows you to write...      │   │
│  │ ████████░░ (streaming)                          │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐│
│ │ Message input...                              [Send]││
│ │                                                     ││
│ └─────────────────────────────────────────────────────┘│
│ [Attach] [Voice] [Stop]                                │
└─────────────────────────────────────────────────────────┘
```

### Message Components

1. **UserMessage** - Right-aligned or distinct styling, edit capability
2. **AssistantMessage** - Markdown rendered, streaming indicator
3. **ToolExecution** - Collapsible card showing tool name, params, result
4. **StreamingIndicator** - Typing indicator or progress bar

### Tool Execution Display

```
┌──────────────────────────────────────────┐
│ 🔧 shell                          ▼ Expand│
├──────────────────────────────────────────┤
│ Command: ls -la /src                     │
│ Duration: 0.3s                           │
│ ┌──────────────────────────────────────┐ │
│ │ total 42                             │ │
│ │ drwxr-xr-x  5 user staff  160 ...   │ │
│ │ -rw-r--r--  1 user staff 1234 ...   │ │
│ └──────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

### User Flows

1. **Send Message**: Type → Enter (or Cmd+Enter) → See streaming response
2. **View Tool**: Tool card appears inline → Click to expand details
3. **Copy Response**: Hover message → Click copy icon → Toast confirmation
4. **Stop Generation**: Click Stop button → Generation halts → Partial response kept

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message (single line mode) |
| `Cmd+Enter` | Send message (multi-line mode) |
| `Escape` | Stop generation |
| `Cmd+K` | Clear chat / new session |
| `Cmd+C` | Copy last response |

## Component Architecture

```
ChatView/
├── ChatHeader.vue          # Title, status, settings
├── MessageList.vue         # Scrollable message container
│   ├── UserMessage.vue     # User input display
│   ├── AssistantMessage.vue # Agent response with markdown
│   │   └── ToolCard.vue    # Collapsible tool execution
│   └── StreamingIndicator.vue
├── ChatInput.vue           # Multi-line input with actions
└── ChatActions.vue         # Attach, voice, stop buttons
```

## State Management (Pinia)

```typescript
interface ChatStore {
  messages: Message[]
  isStreaming: boolean
  currentToolExecution: ToolExecution | null
  error: string | null
  
  // Actions
  sendMessage(content: string): Promise<void>
  stopGeneration(): void
  clearMessages(): void
  regenerateLastResponse(): Promise<void>
}
```

## Alternatives Considered

1. **Polling vs WebSocket**: Chose WebSocket for true bidirectional streaming
2. **Markdown libraries**: Will evaluate marked, markdown-it, or remark
3. **Virtual scrolling**: May need for very long conversations

## Implementation Plan

1. Scaffold Vue component structure
2. Implement basic message display
3. Add WebSocket streaming integration
4. Build tool execution cards
5. Add markdown rendering
6. Implement input with keyboard shortcuts
7. Polish animations and transitions
8. Mobile responsive adjustments