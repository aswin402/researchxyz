# ResearchXYZ — Configuration Guide ⚙️

Configuration details for setting up the `researchxyz` parameters.

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
*   `provider`: API provider. Currently defaults to `"anthropic"`.
*   `model`: The model name used for requests (e.g., `"claude-3-5-sonnet-latest"`).
*   `api_key_env`: Environment variable holding the LLM API key (defaults to `"RESEARCHXYZ_API_KEY"`).

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
