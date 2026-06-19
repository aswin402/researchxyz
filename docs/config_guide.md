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

## 4. Local Persistent Memory 🧠

ResearchXYZ automatically caches synthesized research facts, abstracts, takeaways, and sources in a local database located at:
```bash
~/.config/researchxyz/memory.json
```

This file is a flat JSON array of memory entries structured as follows:

```json
[
  {
    "id": "1718816823456_rust",
    "timestamp": "2026-06-19T14:00:00Z",
    "query": "Rust memory safety guarantees",
    "summary": "Rust guarantees memory safety at compile-time using ownership rules, lifetimes, and borrow checker. It prevents data races, double frees, and dangling pointers without a garbage collector...",
    "keywords": ["rust", "memory safety", "borrow checker"],
    "sources": ["https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html"]
  }
]
```

### Memory Retrieval and Storage
The agent uses the following guidelines automatically during a research loop:
1. **Query Overlap Search**: When a new query is submitted, the agent runs the `memory_search` tool first. It parses keywords from the query and performs a case-insensitive overlap scan on stored queries, keywords, and summaries to return matching records.
2. **Auto-Saving Takeaways**: Upon completing a research run, the agent invokes `memory_store` to record its synthesised answers, key keywords, and URLs referenced, making this context available to future sessions.

