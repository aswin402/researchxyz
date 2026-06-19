# Core Architecture Specification (core.md) 🧠

This document details the architectural guidelines, component diagrams, state management, and thread communication rules for `researchxyz`.

## 1. Design Philosophy
- **Native first, MCP second**: Every capability gets a native Rust implementation if a mature crate exists. MCP is the escape hatch for capabilities that don't yet have one — not the default integration path.
- **One binary, one process**: The TUI, agent core, and native tools all live in one compiled binary. MCP servers are the only thing allowed to run as separate processes.
- **Tools are data, not control flow**: The agent core never special-cases a tool by name. Every tool — native or MCP — implements the same trait and is described to the LLM the same way (name, description, JSON schema).
- **The TUI is a renderer, not a brain**: All decisions (which tool to call, what to write) happen in the agent core. The TUI only renders `AgentEvent`s it receives over a channel; it has no business logic.
- **Fail loud, not silent**: A tool that fails (bad HTTP response, malformed PDF, rate limit) returns a structured error the LLM can see and react to. The agent never panics on external input.

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────┐
│  TUI layer  (ratatui + crossterm)            │
│  chat pane · status bar · subagent panel     │
└───────────────────▲───────────────┬─────────┘
                     │ AgentEvent    │ UserInput
                     │ (stream)      │
┌────────────────────┴───────────────▼─────────┐
│  Agent core                                   │
│  conversation loop · LLM client                │
│  tool dispatcher · memory/context              │
└───────────┬───────────────────────┬───────────┘
            │ Tool trait calls      │ Tool trait calls
┌───────────▼───────────┐  ┌────────▼───────────┐
│ Native tool layer      │  │ MCP tool layer       │
│ web fetch · arXiv       │  │ rmcp client           │
│ Crossref · OpenAlex     │  │ filesystem/browser/   │
│ Semantic Scholar        │  │ search servers         │
│ docx-rs · printpdf      │  │ ppt_mcp, others        │
│ ppt-rs                  │  │                        │
└─────────────────────────┘  └────────────────────────┘
```

## 3. Component Responsibilities

### 3.1 TUI Layer
Owns terminal rendering only. Maintains three panels: a scrollable conversation history, a status bar (active model, token usage, elapsed time, active tool name), and an input box. Receives a stream of `AgentEvent`s over an `mpsc` channel from the agent core and re-renders on each event. Sends `UserInput` events back (submitted prompt, interrupt key, approve/deny for file-writing actions).

### 3.2 Agent Core
Owns the conversation loop: builds the message history sent to the LLM, parses tool-use blocks out of the response, dispatches each tool call to either the native or MCP layer, feeds results back into the next LLM turn, and emits `AgentEvent`s for the TUI to render along the way. Also owns short-term memory (the running conversation) and session-scoped state (sources collected so far, for citation tracking).

### 3.3 Native Tool Layer
A set of Rust structs, each implementing the `Tool` trait (see `spec.md`). Grouped into three families: web research (fetch + extract), academic research (arXiv/Crossref/OpenAlex/Semantic Scholar clients, normalized into a common `Paper` type), and document generation (`.docx`/`.pdf`/`.pptx` writers). Every tool is synchronous from the LLM's point of view but implemented with `async fn` under `tokio`.

### 3.4 MCP Tool Layer
A thin wrapper around the official `rmcp` client. At startup, the agent core reads the list of configured MCP servers from the config file, connects to each over stdio, and merges their advertised tools into the same tool registry the native tools live in — with a namespaced prefix (e.g. `mcp.ppt_mcp.create_slide`) so name collisions with native tools are impossible.

## 4. Request Lifecycle
1. User types a prompt in the TUI; `UserInput::Prompt` is sent to the agent core.
2. Agent core appends the prompt to conversation history and calls the LLM client.
3. LLM response is parsed; if it contains tool-use blocks, the agent core emits `AgentEvent::ToolCallStarted` for each, then dispatches them concurrently (where independent) to the matching `Tool` implementation.
4. Each tool returns a `ToolResult` (success payload or structured error); the agent core emits `AgentEvent::ToolCallFinished` and appends the result to conversation history as a tool-result message.
5. The agent core calls the LLM again with the updated history. Steps 3–5 repeat until the LLM responds with plain text and no further tool calls.
6. Final text is emitted as `AgentEvent::TextDelta` chunks for streaming render in the TUI.
7. If the turn produced a file (via a document-generation tool), the agent core emits `AgentEvent::FileWritten { path }` so the TUI can surface it distinctly from ordinary tool output.

## 5. Concurrency Model
- Single `tokio` runtime, multi-threaded.
- The TUI render loop and the agent core run as separate tasks connected by channels; the TUI never blocks on network I/O.
- Independent tool calls within a single LLM turn (e.g. three arXiv queries) run concurrently via `tokio::join!` or a `JoinSet`; dependent calls (e.g. fetch-then-summarize) run sequentially as dictated by the LLM's own tool-call ordering.
- MCP client calls are awaited the same way as native tool calls — from the dispatcher's perspective there is no difference.

## 6. State Management
- **Conversation history**: an in-memory `Vec<Message>` for the session; optionally persisted to disk as JSON on exit for resume support.
- **Source registry**: every URL/DOI a tool touches during a session is recorded with a stable ID, so document-generation tools can emit numbered citations that map back to real sources rather than hallucinated ones.
- **Config**: loaded once at startup from a TOML file (see `spec.md` for schema); not mutated at runtime.

## 7. Crate Selection Rationale

| Concern | Crate | Why |
|---|---|---|
| TUI rendering | `ratatui` + `crossterm` | dominant Rust TUI ecosystem, immediate-mode, rich widget set |
| Multi-line input | `tui-textarea` | handles cursor/wrapping inside a ratatui widget |
| Async runtime | `tokio` | required by `reqwest`, `rmcp`, and most of the Rust async ecosystem |
| HTTP | `reqwest` | de facto standard async HTTP client |
| HTML extraction | `scraper` | CSS-selector based DOM querying for readable-text extraction |
| Word docs | `docx-rs` / `docx-rust` | most maintained native `.docx` writer |
| PDF | `printpdf` + `genpdf` | low-level PDF primitives plus automatic text-flow layout |
| Slides | `ppt-rs` | most complete native `.pptx` writer available; ships an optional MCP server as a fallback path |
| MCP | `rmcp` | official Rust SDK, client and server roles, stdio transport |
| Errors | `thiserror` (library code) / `anyhow` (binary glue) | standard pairing for typed vs. ad hoc errors |
| Config | `serde` + `toml` | standard deserialization for the config file |
