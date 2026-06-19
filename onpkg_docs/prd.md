# Product Requirements Document (PRD) — ResearchXYZ 🚀

## 1. Summary
`researchxyz` is a terminal-native research agent, written entirely in Rust, that helps a person go from a question to a finished research artifact (a written report, a slide deck, or a literature review) inside a single fast, keyboard-driven session. It is scoped strictly to research workflows: finding sources, reading them, digesting/synthesizing them, and producing polished output documents (.pdf, .docx, .pptx) for other people. It is not a general-purpose coding agent, not a chat companion, and not an automation/ops tool.

## 2. Problem Statement
Existing AI research tools are either heavyweight desktop/web products with limited terminal access, or Python/TypeScript CLI agents that are slow to start, heavy on memory, and awkward to ship as a single binary. There is no fast, native, single-binary research agent built for people who live in a terminal and want a tool that starts instantly, never blocks on a runtime, and produces real deliverables (.docx, .pdf, .pptx) rather than just chat text.

## 3. Goals
- Provide a TUI-first research assistant that feels instant: sub-100ms startup, no runtime dependency, low memory footprint.
- Give the agent first-class research tools: general web search/fetch, academic paper search (arXiv, Crossref, OpenAlex, Semantic Scholar), and citation-aware synthesis.
- Let the agent produce real output documents — Word reports, PDF briefs, and PowerPoint decks — directly from a research session, without leaving the terminal.
- Keep the core implementation in native Rust crates wherever a mature option exists.
- Allow controlled extension via the Model Context Protocol (MCP) for capabilities that don't yet have a good native Rust crate, without making MCP a hard dependency for core functionality.
- Ship as a single static binary that runs on Linux and macOS with no external runtime installs.

## 4. Non-Goals
- Not a general coding agent (no code editing, no shell automation as a primary use case).
- Not a chatbot/companion product — every session should converge toward a deliverable.
- Not a multi-tenant SaaS product in v1 — this is a local, single-user CLI tool.
- Not committed to any single LLM vendor — the agent core should be provider-agnostic, but v1 only needs one provider integration working end to end.
- Not responsible for paid/enterprise search APIs in v1 — free or self-hosted sources only (SearXNG, DuckDuckGo, arXiv, Crossref, OpenAlex, Semantic Scholar).

## 5. Core Use Cases
1. **Literature Scan**: "Find recent papers on X, summarize the top 10, and give me a citation list." → agent queries arXiv/Crossref/OpenAlex/Semantic Scholar, synthesizes, returns formatted citations.
2. **Briefing Document**: "Research the current state of Y and write me a two-page brief." → agent searches the web, reads sources, drafts a `.docx` report with sections and citations.
3. **Presentation for Stakeholders**: "Turn this research into a 10-slide deck for the team." → agent uses prior session context (or fresh research) to produce a `.pptx`.
4. **Quick Fact-Check with Sources**: "Is claim Z still true? Show me where you got that." → agent searches, fetches, cites sources inline in the TUI before producing any file.
5. **Extending Capability**: A user connects an MCP server (e.g., a private knowledge base) and the agent treats it as just another tool, without any core code changes.

## 6. Functional Requirements

| Feature | Requirement |
|---|---|
| **TUI** | Scrollable conversation pane, persistent status bar (model, token usage, active tool/subagent), multi-line input box, keyboard-only navigation. |
| **Web Research** | Fetch and extract readable text from arbitrary URLs; pluggable search backend (SearXNG / Brave / DuckDuckGo). |
| **Academic Research** | Native clients for arXiv, Crossref, OpenAlex, and Semantic Scholar; results normalized into a common `Paper` type with title/authors/year/DOI/abstract. |
| **Document Generation** | Generate `.docx`, `.pdf`, and `.pptx` (via `ppt-rs`) files from agent-authored content, saved to a configurable output directory. |
| **Citations** | Every synthesized claim in a generated document is traceable to a source URL/DOI captured during the session. |
| **Tool Extensibility** | New native tools can be added by implementing one Rust trait; new external tools can be added by connecting an MCP server. |
| **Session Control** | User can interrupt a running tool call, review what the agent is about to do (for file-writing actions), and resume. |
| **Config** | A single TOML config file controls LLM provider/key, search backend, output directory, and connected MCP servers. Supports interactive wizard setup via `researchxyz configure` and dynamic provider/model selection popup overlay `/model` within the TUI. |


## 7. Success Metrics
- Cold start to first prompt rendered: under 150ms.
- Memory footprint at idle: under 30MB.
- A literature-scan use case (5–10 sources, citation list) completes without manual intervention in under 2 minutes of agent time.
- A new native tool can be added by a contributor in under 50 lines of code (trait implementation only).
- Zero crashes on malformed tool output (PDF/HTML/JSON parsing failures degrade gracefully, never panic).
