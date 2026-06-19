use crate::core::registry::Tool;
use crate::core::types::{ToolResult, ToolError, SourceRef};
use serde_json::json;

pub struct AcademicSearchTool;

#[async_trait::async_trait]
impl Tool for AcademicSearchTool {
    fn name(&self) -> &str {
        "academic_search"
    }

    fn description(&self) -> &str {
        "Search academic databases (arXiv, Crossref, OpenAlex, Semantic Scholar) for literature research papers."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
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
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let query = input["query"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing query parameter".to_string())
        })?;
        
        Ok(ToolResult {
            content: format!("Found mock academic paper for query: {}", query),
            citations: vec![SourceRef {
                id: 3,
                url: Some("https://arxiv.org/abs/2103.00001".to_string()),
                doi: Some("10.48550/arXiv.2103.00001".to_string()),
                title: "Example Academic Research Paper".to_string(),
            }],
        })
    }
}
