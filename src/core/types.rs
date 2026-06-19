use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlock {
    Text(String),
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        extra_content: Option<serde_json::Value>,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    TextDelta(String),
    ToolCallStarted {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolCallFinished {
        id: String,
        name: String,
        result: Result<ToolResult, ToolError>,
    },
    FileWritten {
        path: PathBuf,
        kind: DocKind,
    },
    TurnComplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocKind {
    Docx,
    Pdf,
    Pptx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub citations: Vec<SourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub id: u32,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ToolError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Rate limited. Retry after {retry_after_secs:?} seconds")]
    RateLimited { retry_after_secs: Option<u64> },
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Upstream error: {0}")]
    Upstream(String),
}
