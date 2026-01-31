# Extensibility

Arawn provides multiple extension points for customization.

## Extension Mechanisms

| Mechanism | Purpose | Scope |
|-----------|---------|-------|
| **Plugins** | Bundle skills, agents, hooks, tools | Full extensibility |
| **Subagents** | Specialized child agents | Task delegation |
| **Hooks** | Event-driven automation | Cross-cutting concerns |
| **MCP** | External tool servers | Tool integration |

## Plugin Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ PluginManager                                                    │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────────────────────────┐ │
│  │ plugins: Vec<>   │  │ Plugin                                │ │
│  │                  │──│                                       │ │
│  │ load_directory() │  │ name: String                         │ │
│  │ get_skill()      │  │ root: PathBuf                        │ │
│  │ get_agents()     │  │ skills: Vec<Skill>                   │ │
│  │                  │  │ hooks: Vec<HookConfig>               │ │
│  └──────────────────┘  │ agents: Vec<PluginAgentConfig>       │ │
│                        │ cli_tools: Vec<CliTool>              │ │
│                        └──────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Section Contents

- [Plugin System](plugins.md) — Creating and managing plugins
- [Subagent Delegation](subagents.md) — Spawning specialized agents
- [Hooks](hooks.md) — Event-driven extensibility
