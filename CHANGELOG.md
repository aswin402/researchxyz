# ResearchXYZ Changelog 📝

All notable changes to the **ResearchXYZ** project will be documented in this file.

---

## [v0.1.4] - 2026-06-20

### Added
- **Dynamic Context Compression**:
  - Implemented an automatic context history compression engine to prevent context window overflow during multi-turn ReAct loops.
  - Monitors total character payload of conversation history and triggers pruning when surpassing a 40,000-character threshold (~10,000 tokens).
  - Dynamically summarizes/truncates older `ToolResult` blocks (retaining only first/last 800 characters) while preserving the last 2 tool results completely intact to maintain conversational flow.
  - Implemented unit test suites verifying the truncation limits, boundaries, and correct preservation of recent context.

## [v0.1.3] - 2026-06-20

### Added
- **Self-Improvement / Procedural Memory System**:
  - Upgraded local flat-file persistent memory database (`memory.json`) to support detailed `EntryType` classifications: `Fact`, `ToolFailure`, `LinkFailure`, and `UserCorrection`.
  - Added a generic metadata field (`serde_json::Value`) for recording context-specific structured information.
  - Implemented entry-type-based relevance scoring boosts during keyword overlap searches (+5 boost for `UserCorrection`, +2 for `ToolFailure`/`LinkFailure`).
- **Workflow Corrections via `/correct` Prefix**:
  - Added `/correct <rule>` parser hook to intercept corrections in both TUI input and `--test-agent` CLI commands, automatically logging them as `UserCorrection` memory entries.
- **LLM System Prompt Self-Correction Guidance**:
  - Contextualised OpenAI/compatible and Anthropic client prompts to review retrieved memory entries, avoid querying historically failed endpoints/links, and strictly adjust output formatting or workflow behavior based on user corrections.
- **Robust Integration Testing**:
  - Added test suites for verification of memory schema migrations and relevance boost searches.

## [v0.1.1] - 2026-06-20

### Added
- **Local Development Environment (`.env` Integration)**:
  - Added support for loading environment variables from a local `.env` file automatically on application startup.
- **Headless Agent Testing Mode (`--test-agent`)**:
  - Implemented command-line interface argument support to run non-interactive agent queries.
  - Supports passing custom prompts directly from the CLI.
- **Thought Signature Propagation (Google AI Studio Compatibility)**:
  - Added parser and serializer mappings to capture and propagate internal reasoning `thought_signatures` for multi-turn Gemini compatibility.
- **Unit and Integration Tests**:
  - Added document generation unit tests verifying standard layouts for DOCX, PDF, and PPTX reports.

---

## [v0.0.1] - 2026-06-19

### Added
- **Local Persistent Memory Database (`memory_search`, `memory_store`)**:
  - Implemented lightweight flat-file JSON local database (`memory.json`) mapping queries to synthesized summaries, keyword tags, and source links.
  - Implemented automatic query keyword overlap scoring algorithm for sub-millisecond local context queries.
  - Wired LLM clients (Anthropic and OpenAI/compatible) system prompts to execute `memory_search` queries initially and call `memory_store` on completion.
- **Interactive Configuration Wizard (`researchxyz configure`)**:
  - Implemented interactive CLI setup using the `inquire` library.
  - Allows selecting LLM provider (Anthropic, OpenAI, DeepSeek, Groq, OpenRouter, Google AI Studio, or Auto).
  - Prompts for model names with intelligent defaults, masked API key inputs, and custom API base URLs.
  - Saves the resulting settings immediately to `~/.config/researchxyz/config.toml`.
- **In-TUI `/model` Selection Menu**:
  - Intercepts `/model` input query in the main prompt area to open a centered visual pop-up dialog box overlay.
  - Uses key listeners when active:
    - `Up` / `Down` Arrow keys to navigate providers and models.
    - `Enter` key to select/confirm the active option and advance steps.
    - `Esc` key to cancel selection and close the menu.
  - Dynamically updates active `llm.provider` and `llm.model` configurations in-memory and saves them to the disk configuration file for immediate, persistent application.
- **Native Document Compilers (`create_docx`, `create_pdf`, `create_pptx`)**:
  - Replaced stubs with production-ready Rust compilers inside `src/tools/docgen.rs`.
  - Implemented `.docx` formatting templates via `docx-rs`.
  - Implemented `.pdf` compilation layout engines via `genpdf` with robust Linux font fallback sequences.
  - Implemented presentation slides generation via `ppt-rs`.
  - Integrated dynamic filename slugification, timestamp suffixing, and automatic directory creation based on `config.toml` output properties.
- **Academic Search Engine (`academic_search`)**:
  - Replaced mock stubs with live REST API query logic in `src/tools/academic.rs`.
  - Implemented clients for **arXiv** (XML regex parser feed), **CrossRef** (JSON search), **OpenAlex** (JSON works API), and **Semantic Scholar** (Academic Graph search).
  - Merged and deduplicated responses by DOI/Title, dynamically indexing citation mappings from `1`.
- **TUI Modal Draw Engine**:
  - Added `centered_rect` calculations and `Clear` block layout drawing inside `src/tui/draw.rs` to render visual menu overlays on top of the main panels.
  - Styled with the custom Antigravity Teal accent palette.

### Documented
- Documented TUI key bindings and `/model` navigation in [docs/tui_manual.md](docs/tui_manual.md).
- Added quick config wizard setup instructions in [docs/config_guide.md](docs/config_guide.md).
- Updated [README.md](README.md) with quick start examples for both TUI and configure modes.
