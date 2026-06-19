use crate::core::types::{Message, AgentEvent, Role, ContentBlock, ToolError};
use crate::core::registry::ToolRegistry;
use reqwest::Client;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn send_message(
        &self,
        messages: &[Message],
        registry: &ToolRegistry,
        event_tx: &Sender<AgentEvent>,
    ) -> Result<Vec<Message>, anyhow::Error>;
}

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .use_rustls_tls()
                .build()
                .unwrap_or_default(),
            api_key,
            model,
        }
    }
}

// Anthropic Request Types
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        #[serde(rename = "tool_use_id")]
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
    async fn send_message(
        &self,
        messages: &[Message],
        registry: &ToolRegistry,
        event_tx: &Sender<AgentEvent>,
    ) -> Result<Vec<Message>, anyhow::Error> {
        let system_prompt = "You are ResearchXYZ, a dedicated terminal-native research assistant written fully in Rust.\n\
        Your sole goal is to conduct deep, thorough research, scrape sources, evaluate data, and compile professional reports (PDF, Word DOCX, and PPTX slide decks) based on user instructions.\n\
        Always analyze the user's prompt and make a sequential step-by-step checklist plan before you start searching.\n\
        Use your tools to query academic papers, fetch web content, and format high-quality documents.\n\
        Be extremely rigorous and cite all of your assertions using the tool references. Avoid making unsupported claims.".to_string();

        let mut history = messages.to_vec();
        let mut loop_count = 0;
        const MAX_LOOPS: u32 = 10;

        loop {
            loop_count += 1;
            if loop_count > MAX_LOOPS {
                return Err(anyhow::anyhow!("Reached maximum tool execution loop iterations (10)."));
            }

            // 1. Map internal conversation history to Anthropic API formats
            let mut anthropic_messages = Vec::new();
            for msg in &history {
                let role = match msg.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "user".to_string(),
                };
                
                let mut content = Vec::new();
                for block in &msg.content {
                    match block {
                        ContentBlock::Text(text) => {
                            content.push(AnthropicContent::Text { text: text.clone() });
                        }
                        ContentBlock::ToolUse { id, name, input } => {
                            content.push(AnthropicContent::ToolUse {
                                id: id.clone(),
                                name: name.clone(),
                                input: input.clone(),
                            });
                        }
                        ContentBlock::ToolResult { tool_use_id, content: text, is_error } => {
                            content.push(AnthropicContent::ToolResult {
                                tool_use_id: tool_use_id.clone(),
                                content: text.clone(),
                                is_error: Some(*is_error),
                            });
                        }
                    }
                }
                anthropic_messages.push(AnthropicMessage { role, content });
            }

            // 2. Map registered tools to Anthropic API schemas
            let mut anthropic_tools = Vec::new();
            for tool in registry.list() {
                anthropic_tools.push(AnthropicTool {
                    name: tool.name().to_string(),
                    description: tool.description().to_string(),
                    input_schema: tool.input_schema(),
                });
            }

            let request_body = AnthropicRequest {
                model: self.model.clone(),
                max_tokens: 4096,
                system: system_prompt.clone(),
                messages: anthropic_messages,
                tools: anthropic_tools,
            };

            // 3. Dispatch POST request
            let res = self.client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("Anthropic API failed with HTTP {}: {}", status, err_text));
            }

            let resp_json: serde_json::Value = res.json().await?;
            
            // 4. Parse content block response
            let mut assistant_content = Vec::new();
            let mut text_response = String::new();
            let mut tool_calls = Vec::new();

            if let Some(content_array) = resp_json["content"].as_array() {
                for block in content_array {
                    let block_type = block["type"].as_str().unwrap_or_default();
                    if block_type == "text" {
                        let text = block["text"].as_str().unwrap_or_default().to_string();
                        text_response.push_str(&text);
                        assistant_content.push(ContentBlock::Text(text.clone()));
                        
                        // Stream text chunks to UI
                        let _ = event_tx.send(AgentEvent::TextDelta(text)).await;
                    } else if block_type == "tool_use" {
                        let id = block["id"].as_str().unwrap_or_default().to_string();
                        let name = block["name"].as_str().unwrap_or_default().to_string();
                        let input = block["input"].clone();
                        
                        tool_calls.push((id.clone(), name.clone(), input.clone()));
                        assistant_content.push(ContentBlock::ToolUse { id, name, input });
                    }
                }
            }

            // Append assistant response to history
            history.push(Message {
                role: Role::Assistant,
                content: assistant_content,
            });

            // 5. If no tool calls, turn is finished!
            if tool_calls.is_empty() {
                let _ = event_tx.send(AgentEvent::TurnComplete).await;
                return Ok(history);
            }

            // 6. Execute all tool calls in parallel/sequence
            let mut tool_results = Vec::new();
            for (id, name, input) in tool_calls {
                let _ = event_tx.send(AgentEvent::ToolCallStarted {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                }).await;

                let result = if let Some(tool) = registry.get(&name) {
                    tool.call(input).await
                } else {
                    Err(ToolError::InvalidInput(format!("Tool '{}' not found in registry.", name)))
                };

                let is_error = result.is_err();
                let content_str = match &result {
                    Ok(res) => res.content.clone(),
                    Err(err) => err.to_string(),
                };

                let _ = event_tx.send(AgentEvent::ToolCallFinished {
                    id: id.clone(),
                    name: name.clone(),
                    result: result.clone(),
                }).await;

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: id,
                    content: content_str,
                    is_error,
                });
            }

            // Append tool results to history
            history.push(Message {
                role: Role::Tool,
                content: tool_results,
            });
        }
    }
}

pub async fn run_turn(
    messages: &[Message],
    client: Arc<dyn LlmClient>,
    registry: Arc<ToolRegistry>,
    event_tx: Sender<AgentEvent>,
) -> Result<Vec<Message>, anyhow::Error> {
    client.send_message(messages, &registry, &event_tx).await
}
