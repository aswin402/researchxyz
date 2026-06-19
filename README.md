# ResearchXYZ 🚀

<p align="center">
  <img src="assets/logo.svg" alt="ResearchXYZ Logo" width="320" />
</p>

`researchxyz` is a high-performance, terminal-native AI research agent written entirely in Rust. 

Its sole purpose is to conduct deep, thorough research: finding relevant sources, reading webpages, parsing documents (PDF, Word, Excel), synthesizing findings, and producing formatted deliverables (.pdf, .docx, and .pptx slides) in a single keyboard-driven session.

---

## 🛠️ Key Features

*   **Sleek Terminal User Interface (TUI)**: Renders a conversation panel, multi-line editor, real-time logging viewport, and active status bar using `ratatui` & `crossterm`.
*   **Dual Execution Modes**:
    *   **TUI Mode (Default)**: Optimised for human-interactive research sessions.
    *   **JSON-Stream Mode**: Streams structured JSON to stdout and listens to stdin, allowing integration with other AI agents or MCP environments.
*   **Multi-Backend Search & Scraping (`web_search` / `web_fetch`)**: Attempts high-fidelity endpoints (Websurfx, Tavily, Exa) and falls back to keyless DuckDuckGo/Mojeek HTML scrapers. Walks DOM nodes natively via `scraper` and `ego-tree` to extract clean text.
*   **Rich Document Parser (`read_doc`)**: Programmatically extracts text from local files (PDFs via `pdf-extract`, Word DOCX files via `docx-rs`, and Excel spreadsheets via `calamine`).
*   **Structured Citations**: Feeds strict citation reference mappings into LLM context, automatically compiling inline source footnotes and bibliographies.
*   **Model Context Protocol (MCP)**: Features built-in stdio client integrations using `rmcp` to plug in external tool servers (such as presentation builders).

---

## 📂 Project Structure

```
researchxyz/
├── Cargo.toml                 # Cargo manifest
├── README.md                  # Project overview
├── config/
│   └── researchxyz.example.toml # Sample config.toml
├── onpkg.json                 # AI agent project metadata
├── onpkg_docs/                # AI agent specification documents
│   ├── prd.md                 # Product Requirements Document
│   ├── core.md                # System Architecture & Components
│   ├── spec.md                # Technical Schemas & Traits
│   ├── implementation.md      # Implementation Phases
│   ├── todo.md                # Task tracker checklist
│   └── INDEX.md               # AI Docs Index
├── src/
│   ├── main.rs                # Entrypoint, term setups, event router
│   ├── app.rs                 # State controller & event handlers
│   ├── config.rs              # TOML config loaders & path resolvers
│   ├── core/                  # Reasoning structures
│   │   ├── agent.rs           # Anthropic API ReAct execution loop
│   │   ├── registry.rs        # Tool trait & registry
│   │   └── types.rs           # Channel & event types
│   ├── tui/                   # Visual engine
│   │   ├── draw.rs            # Ratatui panel drawers
│   │   └── theme.rs           # Antigravity color theme
│   ├── tools/                 # Native tool controllers
│   │   ├── web.rs             # Search & scraping clients
│   │   ├── docreader.rs       # PDF, Word, Excel parsers
│   │   ├── docgen.rs          # PDF/Word/PPTX compilers
│   │   └── academic.rs        # Academic databases query
│   └── mcp/                   # MCP client connections
│       └── client.rs          # rmcp transport wrapper
└── workspace/                 # Local directory for generated documents
```

---

## 🚀 Quick Start

### 1. Installation & Build

Compile the project:
```bash
cargo build --release
```

### 2. Configuration

Create the configuration directory:
```bash
mkdir -p ~/.config/researchxyz
```
Copy the example config and add your environment variable settings:
```bash
cp config/researchxyz.example.toml ~/.config/researchxyz/config.toml
```

Configure `config.toml` to specify your preferred LLM provider, target model, search backend, and academic APIs.

### 3. Execution

*   **TUI Mode**:
    ```bash
    cargo run --release
    ```
    *   **Controls**: Type inside the prompt box. Press **`Ctrl+Enter`** to submit a research query. Press **`Esc`** to quit.
*   **Dual Mode Behavior**:
    *   If your **`RESEARCHXYZ_API_KEY`** is set in your environment: Runs the real Anthropic ReAct loop, calling web scrapers, reading files, and conducting live research.
    *   If your key is **not set**: Runs in **simulation mode** to showcase interface animations, tool spinners, and document updates offline.
