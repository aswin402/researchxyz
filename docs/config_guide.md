# ResearchXYZ — Configuration Guide ⚙️

Configuration details for setting up the `researchxyz` parameters.

---

## 0. Quick Config Wizard 🧙‍♂️

Instead of editing `config.toml` manually, you can run the interactive CLI configuration wizard:
```bash
cargo run -- configure
```
This wizard will prompt you to select an LLM provider (OpenAI, Anthropic, DeepSeek, Groq, OpenRouter, Google AI Studio, or Auto), set a default model, paste your API key (masked input), and optionally configure a custom base URL. It will automatically save the settings to `~/.config/researchxyz/config.toml`.

---

## 1. File Locations

`researchxyz` expects a TOML file located at:
```bash
~/.config/researchxyz/config.toml
```

If this file is missing, the agent falls back to using DuckDuckGo scraping, standard Anthropic client presets, and directories under the home directory.

---

## 2. Configuration Settings

### `[llm]`
*   `provider`: API provider. Options: `"anthropic"` | `"openai"` | `"deepseek"` | `"groq"` | `"openrouter"` | `"google_ai_studio"` | `"auto"` (defaults to `"anthropic"`). If set to `"auto"`, the provider is auto-detected based on model keywords (e.g. `claude` routes to Anthropic, otherwise OpenAI).
*   `model`: The model name used for requests (e.g. `"claude-3-5-sonnet-latest"`, `"gpt-4o"`, `"deepseek-chat"`). You can also prefix models to override provider selection (e.g. `"openai/gpt-4o"` or `"anthropic/claude-3-5-sonnet"`).
*   `api_key_env`: Environment variable holding the LLM API key (defaults to `"RESEARCHXYZ_API_KEY"`). The resolver will also fall back to standard variables (like `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.) depending on the resolved provider.
*   `api_base`: (Optional) Base API endpoint URL (e.g. `"https://openrouter.ai/api/v1"` or a local Ollama endpoint like `"http://localhost:11434/v1"`). If omitted, standard endpoints for each provider are used.

### `[output]`
*   `dir`: The output folder where completed PDF briefs, Word files, and PowerPoint decks are written (e.g. `"~/researchxyz-output"`). Support `~/` home directory resolution.

### `[search]`
*   `backend`: The search provider. Options: `"searxng"` | `"duckduckgo"` | `"brave"`.
*   `searxng_url`: API URL of a self-hosted Websurfx / SearXNG instance.
*   `brave_api_key_env`: Environment variable name holding your Brave Search API key.

### `[academic]`
*   `sources`: Vector of active paper APIs. Options: `["arxiv", "crossref", "openalex", "semantic_scholar"]`.
*   `crossref_mailto`: Email address used to request Crossref works politely, putting you into their fast-response pool.

### `[[mcp.servers]]`
*   An array of external Model Context Protocol stdio servers.
*   `alias`: Target name (e.g., `"ppt_mcp"`).
*   `command`: Executable command (e.g., `"ppt-rs"`).
*   `args`: Command arguments.

---

## 3. Font Requirements for PDF Generation 📄

PDF generation uses the native `genpdf` crate, which requires TrueType fonts (`.ttf` files) to perform text formatting and layout calculations. ResearchXYZ has a built-in search sequence that checks the following standard paths on Linux:

1.  **FreeSans** in `/usr/share/fonts/truetype/freefont/`
2.  **DejaVuSans** in `/usr/share/fonts/truetype/dejavu/`
3.  **LiberationSans** in `/usr/share/fonts/truetype/liberation/`

To ensure PDF documents compile successfully, make sure at least one of these font families is installed on your Linux system. For example, on Debian/Ubuntu-based systems, you can install them using:
```bash
sudo apt-get install fonts-freefont-ttf fonts-dejavu fonts-liberation
```
If no fonts are found, the PDF creation tool will return an upstream compilation error explaining that no standard TrueType fonts could be loaded.

---

## 4. Local Persistent Memory & Self-Improvement System 🧠

ResearchXYZ automatically caches synthesized research facts, abstracts, takeaways, and sources in a local database.

### Hybrid Storage Engines & Dual-Write Fallback
To ensure high-performance, structured retrieval and robust backward compatibility, the memory manager operates with a dual-engine architecture:

1. **Native Cognitive Memory Server (`openmemory_rs`)**:
   - **Detection**: On startup, ResearchXYZ automatically scans for the `openmemory_rs` binary at `~/.local/bin/openmemory_rs`. If found, it integrates it as the primary cognitive storage engine.
   - **Protocol**: Spawns `openmemory_rs` as a child process and communicates via stdio-based JSON-RPC (Model Context Protocol).
   - **Cognitive Schemas**:
     - *Episodic Reflections*: Maps memories to reflections via `log_reflection` and queries them with `retrieve_episodic_reflections`.
     - *Knowledge Graph Entities*: Maps keyword tags and categories to semantic entities and observations via `create_entities` and `search_nodes`.
   - **Database Location**: Records are stored transactionally in a local SQLite file:
     ```bash
     ~/.config/researchxyz/openmemory.db
     ```
2. **Flat File Database (`memory.json`)**:
   - **Location**:
     ```bash
     ~/.config/researchxyz/memory.json
     ```
   - **Fallback and Dual-Write**: All new memory inserts are written to both `openmemory_rs` (if active) and `memory.json`. If the `openmemory_rs` server is not installed or encounters an error, the system seamlessly falls back to querying `memory.json` using keyword-overlap calculations.

---

### `memory.json` Structure
The flat-file database is structured as a JSON array of entries:

```json
[
  {
    "id": "1718816823456_rust",
    "timestamp": "2026-06-19T14:00:00Z",
    "entry_type": "Fact",
    "query": "Rust memory safety guarantees",
    "summary": "Rust guarantees memory safety at compile-time using ownership rules, lifetimes, and borrow checker. It prevents data races, double frees, and dangling pointers without a garbage collector...",
    "keywords": ["rust", "memory safety", "borrow checker"],
    "sources": ["https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html"],
    "metadata": {}
  },
  {
    "id": "1718821000123_correct",
    "timestamp": "2026-06-20T12:00:00Z",
    "entry_type": "UserCorrection",
    "query": "format PDF using standard font style and simple layout",
    "summary": "format PDF using standard font style and simple layout",
    "keywords": ["pdf", "format", "font", "style", "layout"],
    "sources": [],
    "metadata": null
  }
]
```

### Memory Entry Types
The memory system classifies records into four distinct types:
*   **`Fact`**: Standard synthesised summaries, facts, and paper references. (Default)
*   **`ToolFailure`**: Logged errors from API tools, rate limits, or failing queries, used to steer the agent away from repetitive failures.
*   **`LinkFailure`**: Discovered broken links, HTTP errors, or dead endpoints encountered during web research.
*   **`UserCorrection`**: Explicit formatting guidelines, user constraints, or workflow corrections.

### Self-Improvement Workflow & Adjustments
The ReAct research loop automatically retrieves and acts on historical experiences:
1.  **Memory Retrieval Boost**: At the start of a research task, the agent queries memory. Relevant records are matched based on keyword overlaps and boosted depending on their type:
    *   `UserCorrection` entries receive a **`+5`** boost.
    *   `ToolFailure` & `LinkFailure` entries receive a **`+2`** boost.
2.  **Workflow Adjustment**: The system prompts instruct the agent to:
    *   Strictly adhere to instructions found in `UserCorrection` blocks.
    *   Avoid using URLs or tool queries matched in `LinkFailure` or `ToolFailure` entries.
3.  **Active Correction Logging (`/correct` command)**: 
    *   In both the TUI interface and headless `--test-agent` mode, entering `/correct <instruction>` (e.g. `/correct format PDF reports using simple layouts and no decorative tables`) registers a `UserCorrection` memory entry in the background immediately, adjusting future agent outputs instantly.

---

## 5. Context Management and Compression System 📉

To handle long-running, multi-turn research tasks involving high-volume outputs (e.g., full web scraping text from `web_fetch`), ResearchXYZ features an automated **Context Management & History Compression** pipeline.

### Compression Trigger & Heuristics
1.  **Threshold Monitoring**: Prior to dispatching requests to the LLM backend, the agent estimates the character payload of the active conversation thread.
2.  **Trigger Point**: If the cumulative size of all tool results exceeds **40,000 characters** (~10,000 tokens), the compression pipeline triggers automatically.
3.  **Pruning Rules**:
    *   **Intact Window**: The **most recent 2 tool results** are kept completely intact so the agent can refer to them in granular detail for its current reasoning steps.
    *   **Pruning Target**: For all older tool results exceeding 1,500 characters, the content is compressed down by retaining the first 800 characters (headers/metadata) and the last 800 characters (conclusions/footers), substituting the middle content with a truncation notification.
    *   **Benefits**: This dramatically reduces input token footprint, lowers latency, avoids API context window overflows, and ensures the agent does not lose focus on the main query amid stale source data.

