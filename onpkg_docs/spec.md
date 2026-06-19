# Technical Specification (spec.md) 📋

This document specifies the concrete interfaces, schemas, and contracts for `researchxyz`.

## 1. Core Structural Types

```rust
pub enum Role { User, Assistant, Tool }

pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

pub enum ContentBlock {
    Text(String),
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

pub enum AgentEvent {
    TextDelta(String),
    ToolCallStarted { id: String, name: String, input: serde_json::Value },
    ToolCallFinished { id: String, name: String, result: Result<ToolResult, ToolError> },
    FileWritten { path: std::path::PathBuf, kind: DocKind },
    TurnComplete,
}

pub enum DocKind { Docx, Pdf, Pptx }

pub struct ToolResult {
    pub content: String,           // Text returned to the LLM
    pub citations: Vec<SourceRef>, // Captured sources for citation mapping
}

pub struct SourceRef {
    pub id: u32,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub title: String,
}

pub enum ToolError {
    Network(String),
    RateLimited { retry_after_secs: Option<u64> },
    InvalidInput(String),
    Upstream(String),
}
```

---

## 2. The `Tool` Trait Contract

```rust
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

- `name()` must be unique in the registry. MCP tools are prefixed as `mcp.<server_alias>.<tool_name>`.
- `input_schema()` must return a valid JSON Schema (Draft 2020-12 subset).
- `call()` must never panic. Inputs must be validated, and errors returned as `ToolError`.

---

## 3. TUI Visual Theme

```rust
pub struct Theme {
    pub bg:         Color,  // Frame background
    pub surface:    Color,  // Status bar, input area background
    pub border:     Color,  // Subtle separators
    pub text:       Color,  // Main readable text
    pub text_dim:   Color,  // Tool status lines, timestamps, labels
    pub text_faint: Color,  // Placeholder text
    pub accent:     Color,  // Prompt symbol ›, cursor block, active tool name (teal)
    pub success:    Color,  // ✓ completed tool call (green)
    pub error:      Color,  // ✗ failed tool call (red)
    pub warn:       Color,  // Rate-limit/retry notices (amber)
}
```

---

## 4. Native Tool Schemas

### `web_fetch`
```json
{
  "type": "object",
  "properties": {
    "url": { "type": "string" }
  },
  "required": ["url"]
}
```

### `web_search`
```json
{
  "type": "object",
  "properties": {
    "query": { "type": "string" },
    "max_results": { "type": "integer", "default": 5 }
  },
  "required": ["query"]
}
```

### `academic_search`
```json
{
  "type": "object",
  "properties": {
    "query": { "type": "string" },
    "sources": {
      "type": "array",
      "items": { "type": "string" },
      "default": ["arxiv", "crossref", "openalex", "semantic_scholar"]
    },
    "max_results": { "type": "integer", "default": 10 }
  },
  "required": ["query"]
}
```

### `create_docx` / `create_pdf` / `create_pptx`
```json
{
  "type": "object",
  "properties": {
    "title": { "type": "string" },
    "sections": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "heading": { "type": "string" },
          "body": { "type": "string" }
        },
        "required": ["heading", "body"]
      }
    },
    "citations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "title": { "type": "string" },
          "url_or_doi": { "type": "string" }
        },
        "required": ["id", "title", "url_or_doi"]
      }
    },
    "filename": { "type": "string" }
  },
  "required": ["title", "sections", "citations"]
}
```

---

## 5. Config File Schema (`config.toml`)

```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-6"
api_key_env = "RESEARCHXYZ_API_KEY"

[output]
dir = "~/researchxyz-output"

[search]
backend = "searxng"        # "searxng" | "duckduckgo" | "brave"
searxng_url = "http://localhost:8080"
brave_api_key_env = "BRAVE_API_KEY"

[academic]
sources = ["arxiv", "crossref", "openalex", "semantic_scholar"]
crossref_mailto = "you@example.com"

[[mcp.servers]]
alias = "ppt_mcp"
command = "ppt-rs"
args = ["mcp"]
```
