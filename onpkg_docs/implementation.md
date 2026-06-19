# Technical Implementation Plan (implementation.md) 🛠️

This document describes the step-by-step code structure, file layouts, and crate allocations for implementing `researchxyz`.

---

## 1. Technical Stack Summary

| Layer | Choice |
|---|---|
| Language | Rust (stable channel, edition 2021) |
| Async runtime | tokio |
| TUI | ratatui + crossterm + tui-textarea |
| HTTP | reqwest (rustls-tls feature, no openssl dependency) |
| HTML parsing | scraper |
| Serialization | serde + serde_json + toml |
| Word generation | docx-rs |
| PDF generation | printpdf + genpdf |
| Slide generation | ppt-rs |
| MCP | rmcp (client role) |
| Errors | thiserror + anyhow |
| Logging | tracing + tracing-subscriber (file-based, never stdout, since stdout is the TUI) |

---

## 2. Code Structure (Single-Crate Modular Design)

To avoid Cargo workspace nesting conflicts with the parent VSCode workspace, `researchxyz` is structured as a single crate with clean, decoupled modules. This preserves compiling isolation and readability:

```
researchxyz/
├── Cargo.toml                 # package manifest
├── src/
│   ├── main.rs                # binary entrypoint and task coordinator
│   ├── app.rs                 # main UI app state loop management
│   ├── core/                  # agent loop, LLM client, tool registry, types
│   │   ├── mod.rs
│   │   ├── agent.rs
│   │   ├── types.rs
│   │   └── registry.rs
│   ├── tui/                   # ratatui rendering, event loop, themes
│   │   ├── mod.rs
│   │   ├── draw.rs
│   │   └── theme.rs
│   ├── tools/                 # web fetch + search + academic clients + docgen
│   │   ├── mod.rs
│   │   ├── web.rs             # web search & fetch
│   │   ├── academic.rs        # arXiv, Crossref, OpenAlex, Semantic Scholar clients
│   │   └── docgen.rs          # pdf, docx, pptx compilers
│   └── mcp/                   # rmcp client wrapper
│       ├── mod.rs
│       └── client.rs
├── config/
│   └── researchxyz.example.toml
└── README.md
```

---

## 3. Module Details

### `core`
- `Tool` trait definition (see `spec.md`).
- `Message`, `ToolCall`, `ToolResult`, `AgentEvent` types.
- `LlmClient` trait + one concrete implementation (Anthropic Messages API to start).
- The conversation loop (`run_turn`) that ties LLM calls to tool dispatch.
- `ToolRegistry`: a `HashMap<String, Box<dyn Tool>>` populated at startup.

### `tui`
- `App` struct holding ratatui state (scroll offset, input buffer, status fields).
- Render functions for the three panels (conversation, status, input).
- Event loop: reads crossterm events, translates them into `UserInput`, listens on the `AgentEvent` channel, re-renders on either.

### `tools/web`
- `WebFetch` tool: takes a URL, returns extracted readable text (via `scraper`, stripping nav/ads heuristically).
- `WebSearch` tool: swappable backends (SearXNG instance URL, DuckDuckGo HTML scrape, or Brave API key) selected by config.

### `tools/academic`
- One client module per source (`arxiv`, `crossref`, `openalex`, `semantic_scholar`), each returning a normalized `Paper` struct.
- A single `AcademicSearch` tool that fans out to enabled sources and merges/dedupes results.

### `tools/docgen`
- `CreateDocx`, `CreatePdf`, `CreatePptx` tools, each taking a structured content payload (title, sections, citation list) and writing to the output directory.
- `CreatePptx` uses `ppt-rs`. If not available, it can fallback to the MCP ppt_mcp server.

### `mcp`
- Wraps `rmcp`'s client transport, connects to configured servers over stdio on startup.
- Converts MCP tools into native `Tool` trait objects with a namespaced prefix (e.g. `mcp.ppt_mcp.create_slide`).

---

## 4. TUI Layout Details
- Split into a top conversation area (flex), a status bar (1 row), and a fixed-height input box (3 rows, grows to 6 using `tui-textarea` wrapping).
- Color values live in `Theme` (teal accent, near-black backgrounds, success-green, error-red, etc.).
- Streamed text from LLM renders incrementally using SSE parsing.

---

## 5. Implementation Phases

1. **Phase 1 — Shell**: TUI renders, connects to one LLM provider (Anthropic), no tools.
2. **Phase 2 — Research tools**: `tools/web` and `tools/academic` wired in; agent search & fetch.
3. **Phase 3 — Document generation**: `tools/docgen` wired in for PDF, DOCX, and PPTX slide creation.
4. **Phase 4 — MCP**: `mcp` client connection via stdio.
5. **Phase 5 — Polish**: configuration validation, themes, cross-compilation logs.
