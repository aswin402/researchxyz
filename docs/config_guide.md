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
