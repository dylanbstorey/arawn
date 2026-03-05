# Glossary

Domain-specific terms and acronyms used throughout the Arawn codebase.

## Acronyms

| Term | Expansion | Description |
|------|-----------|-------------|
| **GLiNER** | Generalist and Lightweight Model for NER | A span-based NER model used for local entity extraction. Arawn vendors it via `gline-rs`. |
| **MCP** | Model Context Protocol | Anthropic's protocol for connecting LLMs to external tool servers. |
| **NER** | Named Entity Recognition | NLP task of identifying and classifying entities (people, concepts, tools) in text. Arawn uses local NER for hybrid extraction — fast NER for entities/relationships, LLM for facts. See `arawn-agent/src/indexing/ner.rs`. |
| **ORP** | ONNX Runtime Pipelines | Lightweight framework for chaining ONNX model inference. Used internally by the GLiNER NER engine. See `crates/orp-vendored/`. |
| **PKCE** | Proof Key for Code Exchange | OAuth 2.0 extension for public clients. Used in the Claude MAX OAuth flow. See `arawn-oauth`. |
| **RLM** | Recursive Language Model | Arawn's exploration sub-agent that iteratively researches a topic using read-only tools and context compaction. See `arawn-agent/src/rlm/`. |

## Key Concepts

| Term | Description |
|------|-------------|
| **Agent loop** | The core turn-based cycle: receive user message, call LLM, execute tools, repeat until done. |
| **Compaction** | Summarizing conversation history to fit within context limits while preserving key information. |
| **Workstream** | An isolated, persistent environment for organizing related sessions and files. Named workstreams share state across sessions; scratch workstreams are session-isolated. |
| **Turn** | A single cycle of the agent loop: one LLM call plus any resulting tool executions. |
| **Tool registry** | The central registry of available tools that the agent can invoke during a turn. |
| **Session** | A conversation with message history, tied to a workstream. |
