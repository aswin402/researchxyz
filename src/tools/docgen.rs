use crate::core::registry::Tool;
use crate::core::types::{ToolResult, ToolError};
use serde_json::json;

pub struct CreateDocxTool;

#[async_trait::async_trait]
impl Tool for CreateDocxTool {
    fn name(&self) -> &str {
        "create_docx"
    }

    fn description(&self) -> &str {
        "Generate a Microsoft Word document (.docx) of the research summary."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
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
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let title = input["title"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing title".to_string())
        })?;
        
        Ok(ToolResult {
            content: format!("Mock generated DOCX file for title: {}", title),
            citations: vec![],
        })
    }
}

pub struct CreatePdfTool;

#[async_trait::async_trait]
impl Tool for CreatePdfTool {
    fn name(&self) -> &str {
        "create_pdf"
    }

    fn description(&self) -> &str {
        "Generate a formatted PDF report of the research summary."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
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
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let title = input["title"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing title".to_string())
        })?;
        
        Ok(ToolResult {
            content: format!("Mock generated PDF file for title: {}", title),
            citations: vec![],
        })
    }
}

pub struct CreatePptxTool;

#[async_trait::async_trait]
impl Tool for CreatePptxTool {
    fn name(&self) -> &str {
        "create_pptx"
    }

    fn description(&self) -> &str {
        "Generate a slide deck presentation (.pptx) of the research summary."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
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
        })
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolResult, ToolError> {
        let title = input["title"].as_str().ok_or_else(|| {
            ToolError::InvalidInput("Missing title".to_string())
        })?;
        
        Ok(ToolResult {
            content: format!("Mock generated PPTX presentation for title: {}", title),
            citations: vec![],
        })
    }
}
