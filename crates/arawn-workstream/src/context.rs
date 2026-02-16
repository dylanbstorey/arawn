use crate::Result;
use crate::manager::WorkstreamManager;
use crate::types::{MessageRole, WorkstreamMessage};

/// Assembled context ready for injection into an LLM request.
#[derive(Debug)]
pub struct AssembledContext {
    /// Workstream summary to inject as system-level context (if available).
    pub summary: Option<String>,
    /// Conversation messages in chronological order.
    pub messages: Vec<ContextMessage>,
}

/// A message prepared for LLM context, with role mapped to user/assistant.
#[derive(Debug, Clone)]
pub struct ContextMessage {
    /// "user" or "assistant"
    pub role: ContextRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRole {
    User,
    Assistant,
    System,
}

impl ContextRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

/// Assembles workstream history into LLM-ready context.
pub struct ContextAssembler<'a> {
    manager: &'a WorkstreamManager,
}

impl<'a> ContextAssembler<'a> {
    pub fn new(manager: &'a WorkstreamManager) -> Self {
        Self { manager }
    }

    /// Assemble context for a workstream, fitting within `max_chars` (approximate token budget).
    ///
    /// Uses char_count / 4 as a rough token estimate. Returns the workstream summary
    /// (if present) plus the most recent messages that fit.
    pub fn assemble(&self, workstream_id: &str, max_chars: usize) -> Result<AssembledContext> {
        let workstream = self.manager.get_workstream(workstream_id)?;
        let all_messages = self.manager.get_messages(workstream_id)?;

        let summary = workstream.summary.clone();
        let summary_chars = summary.as_ref().map_or(0, |s| s.len());

        // Budget for messages after reserving space for summary
        let message_budget = max_chars.saturating_sub(summary_chars);

        // Take most recent messages that fit within budget
        let context_messages = fit_messages(&all_messages, message_budget);

        Ok(AssembledContext {
            summary,
            messages: context_messages,
        })
    }
}

/// Map a WorkstreamMessage role to a ContextRole.
fn map_role(role: MessageRole) -> ContextRole {
    match role {
        MessageRole::User => ContextRole::User,
        MessageRole::Assistant | MessageRole::AgentPush => ContextRole::Assistant,
        MessageRole::System => ContextRole::System,
        // Tool use and tool results are sent as assistant context (part of the agent's actions)
        MessageRole::ToolUse => ContextRole::Assistant,
        MessageRole::ToolResult => ContextRole::User, // tool results sent as user context
    }
}

/// Select the most recent messages that fit within `budget` characters.
/// Walks backwards from the end, accumulating until budget is exceeded.
fn fit_messages(messages: &[WorkstreamMessage], budget: usize) -> Vec<ContextMessage> {
    let mut result = Vec::new();
    let mut used = 0;

    for msg in messages.iter().rev() {
        let cost = msg.content.len();
        if used + cost > budget && !result.is_empty() {
            break;
        }
        result.push(ContextMessage {
            role: map_role(msg.role),
            content: msg.content.clone(),
        });
        used += cost;
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_store::MessageStore;
    use crate::store::WorkstreamStore;

    fn test_manager() -> (tempfile::TempDir, WorkstreamManager) {
        let dir = tempfile::tempdir().unwrap();
        let store = WorkstreamStore::open_in_memory().unwrap();
        let msg_store = MessageStore::new(dir.path());
        let mgr = WorkstreamManager::from_parts(store, msg_store, 30);
        (dir, mgr)
    }

    #[test]
    fn test_empty_workstream() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Empty", None, &[]).unwrap();

        let assembler = ContextAssembler::new(&mgr);
        let ctx = assembler.assemble(&ws.id, 10000).unwrap();

        assert!(ctx.summary.is_none());
        assert!(ctx.messages.is_empty());
    }

    #[test]
    fn test_short_history_fits() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Chat", None, &[]).unwrap();

        mgr.send_message(Some(&ws.id), None, MessageRole::User, "hello", None)
            .unwrap();
        mgr.send_message(Some(&ws.id), None, MessageRole::Assistant, "hi there", None)
            .unwrap();

        let assembler = ContextAssembler::new(&mgr);
        let ctx = assembler.assemble(&ws.id, 10000).unwrap();

        assert_eq!(ctx.messages.len(), 2);
        assert_eq!(ctx.messages[0].role, ContextRole::User);
        assert_eq!(ctx.messages[0].content, "hello");
        assert_eq!(ctx.messages[1].role, ContextRole::Assistant);
    }

    #[test]
    fn test_long_history_truncated() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Long", None, &[]).unwrap();

        // Write 10 messages of 100 chars each = 1000 chars total
        for i in 0..10 {
            let role = if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };
            let content = format!("{:>100}", i); // 100 chars each
            mgr.send_message(Some(&ws.id), None, role, &content, None)
                .unwrap();
        }

        let assembler = ContextAssembler::new(&mgr);
        // Budget for ~3 messages (300 chars)
        let ctx = assembler.assemble(&ws.id, 300).unwrap();

        assert_eq!(ctx.messages.len(), 3);
        // Should be the last 3 messages
        assert!(ctx.messages[2].content.contains("9"));
    }

    #[test]
    fn test_summary_reduces_message_budget() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Summarized", None, &[]).unwrap();

        // Set a summary on the workstream
        mgr.store()
            .update_workstream(&ws.id, None, Some("A".repeat(200).as_str()), None, None)
            .unwrap();

        // Write 5 messages of 100 chars each
        for i in 0..5 {
            let role = if i % 2 == 0 {
                MessageRole::User
            } else {
                MessageRole::Assistant
            };
            let content = format!("{:>100}", i);
            mgr.send_message(Some(&ws.id), None, role, &content, None)
                .unwrap();
        }

        let assembler = ContextAssembler::new(&mgr);
        // 500 total budget, 200 for summary = 300 for messages = ~3 messages
        let ctx = assembler.assemble(&ws.id, 500).unwrap();

        assert!(ctx.summary.is_some());
        assert_eq!(ctx.summary.unwrap().len(), 200);
        assert!(ctx.messages.len() <= 3);
    }

    #[test]
    fn test_role_mapping() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Roles", None, &[]).unwrap();

        mgr.send_message(Some(&ws.id), None, MessageRole::User, "q", None)
            .unwrap();
        mgr.send_message(Some(&ws.id), None, MessageRole::Assistant, "a", None)
            .unwrap();
        mgr.push_agent_message(&ws.id, "update", None).unwrap();
        mgr.send_message(Some(&ws.id), None, MessageRole::System, "sys", None)
            .unwrap();
        mgr.send_message(Some(&ws.id), None, MessageRole::ToolResult, "result", None)
            .unwrap();

        let assembler = ContextAssembler::new(&mgr);
        let ctx = assembler.assemble(&ws.id, 10000).unwrap();

        assert_eq!(ctx.messages[0].role, ContextRole::User);
        assert_eq!(ctx.messages[1].role, ContextRole::Assistant);
        assert_eq!(ctx.messages[2].role, ContextRole::Assistant); // AgentPush → Assistant
        assert_eq!(ctx.messages[3].role, ContextRole::System);
        assert_eq!(ctx.messages[4].role, ContextRole::User); // ToolResult → User
    }
}
