# Task Tracker (todo.md) 📋

This checklist tracks progress on the five development phases of `researchxyz`.

## Phase 0 — Project Scaffolding
- [x] Initialize cargo package workspace configuration setup.
- [x] Structure the internal folder directory layout (`src/core/`, `src/tui/`, `src/tools/`, `src/mcp/`).
- [x] Add `thiserror` and `anyhow` error conversions in `core/types.rs`.
- [x] Configure `tracing` and `tracing-subscriber` file logging (logging to `~/.config/researchxyz/logs/`).
- [x] Create `config/researchxyz.example.toml` matching the config schema in `spec.md`.
- [x] Create the TOML config loader module.
- [x] Ensure cargo compilation runs check cleanly without workspace conflicts.

## Phase 1 — TUI Shell & Chat Loop
- [x] Define core structural types (`Role`, `Message`, `ContentBlock`, `AgentEvent`, `ToolResult`, `ToolError`) in `src/core/types.rs`.
- [x] Implement `LlmClient` trait and Anthropic Messages API integration (mocked loop, SSE parser setup).
- [x] Build the conversation reasoning loop (`run_turn`).
- [x] Implement `ratatui` `App` visual panels: conversation pane, status bar, input box.
- [x] Wire `crossterm` event loop for keyboard action mapping and window rendering.
- [x] Render streamed text deltas incrementally without full redraws.
- [x] Manual test: Verify a raw chat conversation runs end to end without tools.

## Phase 2 — Research Tools
- [x] Implement `WebFetch` tool (extract readable text using `scraper` selector logic from `openz`).
- [x] Implement `WebSearch` tool (support SearXNG, DuckDuckGo scraper, and Brave API from `openz`).
- [x] Implement `DocReader` tool (extract Word `.docx`, PDF `.pdf`, and Excel `.xlsx`/`.xls`/`.ods` documents from `openz`).
- [x] Implement arXiv client parser (`Paper` normalization).
- [x] Implement Crossref work search API (`Paper` normalization, mailto support).
- [x] Implement OpenAlex client work search (`Paper` normalization).
- [x] Implement Semantic Scholar search.
- [x] Implement merged `AcademicSearch` tool with DOI/title deduplication.
- [x] Implement session-scoped `SourceRef` registry for citations.
- [x] Manual test: "Find papers on X" produces cited sources in the chat history.

## Phase 3 — Document Generation
- [ ] Implement `CreateDocx` tool using `docx-rs` layout templates.
- [ ] Implement `CreatePdf` tool using `genpdf` / `printpdf` auto-pagination.
- [ ] Implement `CreatePptx` tool using `ppt-rs` slide builder.
- [ ] Implement filename slugification and timestamp suffixes.
- [ ] Emit `AgentEvent::FileWritten` and render absolute paths in the UI.
- [ ] Golden-file testing: Verify generated files can open correctly.
- [ ] Manual test: Generate Word and PDF documents from research session.

## Phase 4 — MCP Integration
- [ ] Wrap the `rmcp` client transport for stdio.
- [ ] Connect configured MCP servers at startup dynamically.
- [ ] Map MCP server tools into namespaced `Tool` trait objects (`mcp.<alias>.<tool>`).
- [ ] Add tool registry merge capability.
- [ ] Manual test: Connect `ppt_mcp` and test pptx generation.

## Phase 5 — Polish & Release
- [ ] Customize TUI Theme (default dark theme matching Antigravity color system).
- [ ] Show elapsed time, tokens, and active operations in status bar.
- [ ] Validate startup configurations with explicit error messaging.
- [x] Write detailed installation instructions and README documentation.
- [x] Implement interactive CLI configuration wizard (`researchxyz configure`).
- [x] Implement dynamic in-TUI model selection popup overlay via `/model` command.

