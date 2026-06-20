use crate::core::registry::Tool;
use crate::core::types::{ToolResult, ToolError, SourceRef};
use crate::core::memory::MemoryManager;
use serde_json::json;

pub struct MemorySearchTool;

#[async_trait::async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        "Search the local persistent research memory database for previously synthesized facts, paper abstracts, or key findings."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "max_results": { "type": "integer", "default": 5 }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let query = input["query"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing query parameter".to_string())
        })?;
        let max_results = input["max_results"].as_u64().unwrap_or(5) as usize;

        let manager = MemoryManager::load();
        let matches = manager.search(query, max_results);

        if matches.is_empty() {
            return Ok(ToolResult {
                content: format!("No matches found in persistent memory for query: '{}'", query),
                citations: vec![],
            });
        }

        let mut content = String::new();
        content.push_str("Found matching records in local persistent memory:\n\n");
        
        let mut citations = Vec::new();
        let mut id_counter = 1;

        for entry in matches {
            content.push_str(&format!("--- Record: {} (Type: {:?}, Saved: {}) ---\n", entry.query, entry.entry_type, entry.timestamp));
            content.push_str(&entry.summary);
            content.push_str("\n\n");

            if !entry.sources.is_empty() {
                content.push_str("Sources:\n");
                for src in &entry.sources {
                    content.push_str(&format!(" - {}\n", src));
                    
                    // Construct a SourceRef citation
                    citations.push(SourceRef {
                        id: id_counter,
                        url: Some(src.clone()),
                        doi: None,
                        title: format!("Memory Ref: {}", entry.query),
                    });
                    id_counter += 1;
                }
                content.push_str("\n");
            }
        }

        Ok(ToolResult {
            content,
            citations,
        })
    }
}

pub struct MemoryStoreTool;

#[async_trait::async_trait]
impl Tool for MemoryStoreTool {
    fn name(&self) -> &str {
        "memory_store"
    }

    fn description(&self) -> &str {
        "Store a newly synthesized query result, paper abstract, tool experience, or key facts in the local persistent research memory database."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "summary": { "type": "string" },
                "keywords": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "sources": {
                    "type": "array",
                    "items": { "type": "string" },
                    "default": []
                },
                "entry_type": {
                    "type": "string",
                    "enum": ["Fact", "ToolFailure", "LinkFailure", "UserCorrection"],
                    "default": "Fact"
                },
                "metadata": {
                    "type": "object",
                    "default": {}
                }
            },
            "required": ["query", "summary", "keywords"]
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let query = input["query"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing query parameter".to_string())
        })?;
        let summary = input["summary"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing summary parameter".to_string())
        })?;
        
        let keywords_array = input["keywords"].as_array().ok_or_else(|| {
            ToolError::InvalidInput("Missing keywords array".to_string())
        })?;
        let keywords: Vec<String> = keywords_array.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
            
        let sources_array = input["sources"].as_array();
        let sources: Vec<String> = match sources_array {
            Some(arr) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            None => Vec::new(),
        };

        let entry_type_str = input["entry_type"].as_str().unwrap_or("Fact");
        let entry_type = match entry_type_str {
            "ToolFailure" => crate::core::memory::EntryType::ToolFailure,
            "LinkFailure" => crate::core::memory::EntryType::LinkFailure,
            "UserCorrection" => crate::core::memory::EntryType::UserCorrection,
            _ => crate::core::memory::EntryType::Fact,
        };

        let metadata = input["metadata"].clone();

        let mut manager = MemoryManager::load();
        manager.add_detailed(query, summary, keywords, sources, entry_type, metadata)
            .map_err(|e| ToolError::Upstream(format!("Failed to save to memory file: {}", e)))?;

        Ok(ToolResult {
            content: "Successfully saved facts/takeaways to local persistent memory database.".to_string(),
            citations: vec![],
        })
    }
}
