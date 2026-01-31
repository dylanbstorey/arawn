---
id: 001-sandboxed-script-execution-via
level: adr
title: "Sandboxed Script Execution via Wasmtime + Rust-to-WASM"
number: 1
short_code: "ARAWN-A-0002"
created_at: 2026-01-29T17:49:49.964758+00:00
updated_at: 2026-01-29T17:53:00.723715+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: Sandboxed Script Execution via Wasmtime + Rust-to-WASM

## Context

Arawn's Cloacina-based workflow engine (I-0012) needs the ability to execute agent-generated scripts as workflow task actions. The agent must be able to define and run arbitrary code at runtime — for data processing, API interactions, text manipulation, and other tasks within scheduled or event-driven workflows.

This code runs on the user's machine (laptop/desktop), potentially with access to the filesystem, network, and OS. Agent-generated code carries real risks:
- **Destructive actions**: accidental file/database deletion
- **Data exfiltration**: unintended network access leaking sensitive data
- **System compromise**: scripts escaping their intended scope

We need a sandboxing strategy that provides strong isolation while being:
- **Cross-platform**: macOS and Linux (no Linux-only kernel features)
- **Lightweight**: no containers, no VMs, no Docker dependency
- **Laptop-friendly**: minimal resource overhead, fast startup
- **Embeddable**: runs as a library within the Arawn process

## Decision

Use **Wasmtime** as the sandbox runtime, with **Rust as the scripting language** compiled to `wasm32-wasip1`.

The execution model:
1. Agent generates a Rust script implementing a pre-defined task trait/SDK
2. Arawn compiles the script to WASM targeting `wasm32-wasip1` (with cached dependencies)
3. Wasmtime executes the WASM module with capability-scoped WASI permissions
4. Context flows in via stdin/args, output captured via stdout
5. Compile errors are fed back to the agent for self-correction

A pre-compiled SDK crate provides common operations (JSON parsing, HTTP client, regex, etc.) so agent scripts don't need to pull in dependencies, keeping compile times at 3-8 seconds cold.

## Alternatives Analysis

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **Wasmtime + Rust** | Strong sandbox, cross-platform, Rust-native embedding, compiler catches agent errors before execution, WASI capability model | 3-8s compile per attempt, WASI networking still maturing | Low | Medium |
| **Wasmtime + JS (QuickJS)** | <100ms startup, no compile step, familiar language | Weak typing leads to more runtime failures, agent needs more iterations to get correct code, no compile-time safety | Medium | Low |
| **WasmEdge + Python plugin** | Native Python in sandbox, familiar to LLMs | Dynamic typing = runtime errors, no pip/C extensions, limited stdlib, less mature Rust embedding API | Medium | Medium |
| **Firecracker microVM** | Strongest possible isolation, runs any language | Linux-only (no macOS), requires KVM, heavy for script execution | Low | High |
| **Container (bubblewrap/nsjail)** | Good isolation, any language | Linux-only for full features, macOS limited, container overhead | Medium | Medium |
| **Subprocess + seccomp/sandbox-exec** | Simple, cross-platform, fast | Weakest isolation, platform-specific restrictions, hard to truly sandbox | High | Low |

## Rationale

**Why Wasmtime:**
- Only option providing strong isolation on both macOS and Linux without containers or VMs
- Capability-based WASI security model — no filesystem/network access unless explicitly granted
- Embeds as a Rust library (`wasmtime` crate) — no external runtime dependency
- Sub-millisecond module instantiation once compiled
- Bytecode Alliance backed, most actively maintained WASM runtime

**Why Rust as the scripting language:**
- LLMs generate more correct Rust than Python because the compiler provides precise, actionable error feedback. A type error is caught at compile time with an exact message, not at runtime three layers deep.
- The type system acts as guardrails — `Result<T, E>` forces error handling, `Option<T>` prevents null errors, strong typing prevents silent coercion bugs.
- Fewer iterations to correct code. The compile-fix loop is: write → compile error (precise) → fix → compile succeeds → runs correctly. Python's loop is: write → runs → runtime error (ambiguous) → fix → runs → different runtime error → fix → works maybe.
- Net wall-clock time is comparable despite compile overhead, because fewer attempts are needed.
- Arawn is a Rust project — the SDK, types, and toolchain are already available.

**Why not Python/JS:**
- Dynamic typing means errors surface at runtime, not compile time. Agent-generated code is more likely to fail silently or with ambiguous errors.
- Python-on-WASI has no pip, limited stdlib, no C extensions — most agent-generated Python would fail due to missing `import requests`, `import json` edge cases, etc.
- JS is viable (QuickJS-on-WASM) but lacks the compile-time safety net. Could be added as a secondary option later.

## Consequences

### Positive
- Strong, cross-platform sandboxing with no external dependencies
- Compiler-enforced correctness reduces agent iteration loops
- Fine-grained capability control (filesystem paths, network endpoints, memory limits)
- Pre-compiled SDK keeps common operations fast without dependency compilation
- Same language as the host project — shared types, familiar toolchain
- WASM modules are cacheable — identical scripts skip recompilation

### Negative
- 3-8 second compile time per new script (mitigated by caching and pre-compiled SDK)
- Agents must generate Rust, which is less common in LLM training data than Python/JS (mitigated by the SDK reducing boilerplate and the compiler guiding corrections)
- WASI networking proposals are still stabilizing — HTTP client in the SDK may need updating as specs evolve
- Adds `wasmtime` as a dependency to Arawn (significant but well-maintained crate)

### Neutral
- Does not preclude adding JS/Python support later via QuickJS-WASM or language-specific WASM runtimes
- Cloacina workflows that don't need script execution are unaffected
- Built-in Arawn tools (web_fetch, memory operations) remain native Rust and don't go through the sandbox

## Review Triggers

- WASI networking proposals reach stable — review whether SDK HTTP client needs updating
- If agent Rust generation proves unreliable in practice — consider adding QuickJS as a secondary runtime
- Cloacina plugin system stabilizes — evaluate whether it provides an alternative execution model