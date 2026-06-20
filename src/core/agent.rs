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
        \n\
        === PERSISTENT MEMORY GUIDELINES ===\n\
        1. Always begin a new research topic by calling `memory_search` to see if we have previously saved facts, paper summaries, or findings related to the topic. Leverage existing memory to save time, tokens, and api calls.\n\
        2. Pay close attention to different Memory Entry Types returned by search:\n\
           - Fact: Standard research summaries and references. Use them directly.\n\
           - ToolFailure / LinkFailure: Historical logs of dead links, rate limits, or query patterns that failed. You MUST avoid querying these exact failed endpoints, URLs, or query keywords.\n\
           - UserCorrection: Explicit workflow instructions or formatting corrections previously requested by the user. You MUST adjust your research plan checklist to strictly satisfy these corrections.\n\
        3. If you encounter any tool call errors (e.g. RateLimited, or dead links), or if the user corrects your approach during a turn, you must invoke `memory_store` to cache that experience (setting `entry_type` to `ToolFailure`, `LinkFailure`, or `UserCorrection` accordingly with relevant keywords and metadata).\n\
        4. When you finish your research and synthesize the final takeaways, always call `memory_store` (with `entry_type` as \"Fact\") to save the summary, keywords, and sources in the local database. This helps you remember this context for future sessions.\n\
        \n\
        === ACADEMIC WORKFLOW GUIDELINES ===\n\
        1. Query formulation: Start by analyzing the user prompt and formulating a precise research plan checklist.\n\
        2. Search & Triangulation: Use `academic_search` to query arXiv, Crossref, OpenAlex, and Semantic Scholar. Deduplicate DOIs.\n\
        3. Web Fetch: Use `web_fetch` to retrieve full texts or summaries from relevant URLs.\n\
        4. Citations: Be extremely rigorous. Always cite assertions using explicit SourceRef IDs (e.g., [1], [2]). Avoid unsupported claims.\n\
        5. Compilation: Generate executive Word reports, PDF documents, or PowerPoint slide decks using formatting tools based on the user's requested deliverable format.".to_string();

        let mut history = messages.to_vec();
        let mut loop_count = 0;
        const MAX_LOOPS: u32 = 10;

        loop {
            loop_count += 1;
            compress_history(&mut history);
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
                        ContentBlock::ToolUse { id, name, input, .. } => {
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
                        assistant_content.push(ContentBlock::ToolUse { id, name, input, extra_content: None });
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

pub fn resolve_client(config: &crate::config::Config) -> Result<Arc<dyn LlmClient>, anyhow::Error> {
    let mut provider = config.llm.provider.to_lowercase();
    let mut model = config.llm.model.clone();
    
    // Parse model prefix to resolve provider if explicitly prefixed (e.g. openai/gpt-4o)
    let model_lower = model.to_lowercase();
    if model_lower.starts_with("openai/") {
        provider = "openai".to_string();
        model = model["openai/".len()..].to_string();
    } else if model_lower.starts_with("anthropic/") {
        provider = "anthropic".to_string();
        model = model["anthropic/".len()..].to_string();
    } else if model_lower.starts_with("deepseek/") {
        provider = "deepseek".to_string();
        model = model["deepseek/".len()..].to_string();
    } else if model_lower.starts_with("groq/") {
        provider = "groq".to_string();
        model = model["groq/".len()..].to_string();
    } else if model_lower.starts_with("openrouter/") {
        provider = "openrouter".to_string();
        model = model["openrouter/".len()..].to_string();
    } else if model_lower.starts_with("google_ai_studio/") || model_lower.starts_with("google-ai-studio/") {
        provider = "google_ai_studio".to_string();
        let prefix_len = if model_lower.starts_with("google_ai_studio/") { "google_ai_studio/".len() } else { "google-ai-studio/".len() };
        model = model[prefix_len..].to_string();
    } else if provider == "auto" {
        if model_lower.contains("claude") {
            provider = "anthropic".to_string();
        } else {
            provider = "openai".to_string();
        }
    }

    // Resolve API Key
    let api_key = config.llm.api_key.clone()
        .or_else(|| std::env::var(&config.llm.api_key_env).ok())
        .or_else(|| std::env::var("RESEARCHXYZ_API_KEY").ok())
        .or_else(|| {
            match provider.as_str() {
                "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
                "openai" => std::env::var("OPENAI_API_KEY").ok(),
                "deepseek" => std::env::var("DEEPSEEK_API_KEY").ok(),
                "groq" => std::env::var("GROQ_API_KEY").ok(),
                "openrouter" => std::env::var("OPENROUTER_API_KEY").ok(),
                "google_ai_studio" => std::env::var("GOOGLE_AI_STUDIO_API_KEY").ok(),
                _ => std::env::var("OPENAI_API_KEY").ok(),
            }
        })
        .unwrap_or_default();

    if api_key.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "No API key found for provider '{}'. Please set the environment variable '{}' or appropriate fallback key.",
            provider,
            config.llm.api_key_env
        ));
    }

    // Resolve Base URL
    let api_base = if let Some(base) = &config.llm.api_base {
        base.clone()
    } else {
        match provider.as_str() {
            "anthropic" => "https://api.anthropic.com".to_string(),
            "openai" => "https://api.openai.com/v1".to_string(),
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            "groq" => "https://api.groq.com/openai/v1".to_string(),
            "openrouter" => "https://openrouter.ai/api/v1".to_string(),
            "google_ai_studio" => "https://generativelanguage.googleapis.com/v1beta/openai/".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        }
    };

    if provider == "anthropic" {
        Ok(Arc::new(AnthropicClient::new(api_key, model)))
    } else {
        Ok(Arc::new(OpenAiClient::new(api_key, api_base, model)))
    }
}

pub struct OpenAiClient {
    client: Client,
    api_key: String,
    api_base: String,
    model: String,
}

impl OpenAiClient {
    pub fn new(api_key: String, api_base: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .use_rustls_tls()
                .build()
                .unwrap_or_default(),
            api_key,
            api_base,
            model,
        }
    }
}

#[async_trait::async_trait]
impl LlmClient for OpenAiClient {
    async fn send_message(
        &self,
        messages: &[Message],
        registry: &ToolRegistry,
        event_tx: &Sender<AgentEvent>,
    ) -> Result<Vec<Message>, anyhow::Error> {
        let system_prompt = "You are ResearchXYZ, a dedicated terminal-native research assistant written fully in Rust.\n\
        Your sole goal is to conduct deep, thorough research, scrape sources, evaluate data, and compile professional reports (PDF, Word DOCX, and PPTX slide decks) based on user instructions.\n\
        \n\
        === PERSISTENT MEMORY GUIDELINES ===\n\
        1. Always begin a new research topic by calling `memory_search` to see if we have previously saved facts, paper summaries, or findings related to the topic. Leverage existing memory to save time, tokens, and api calls.\n\
        2. Pay close attention to different Memory Entry Types returned by search:\n\
           - Fact: Standard research summaries and references. Use them directly.\n\
           - ToolFailure / LinkFailure: Historical logs of dead links, rate limits, or query patterns that failed. You MUST avoid querying these exact failed endpoints, URLs, or query keywords.\n\
           - UserCorrection: Explicit workflow instructions or formatting corrections previously requested by the user. You MUST adjust your research plan checklist to strictly satisfy these corrections.\n\
        3. If you encounter any tool call errors (e.g. RateLimited, or dead links), or if the user corrects your approach during a turn, you must invoke `memory_store` to cache that experience (setting `entry_type` to `ToolFailure`, `LinkFailure`, or `UserCorrection` accordingly with relevant keywords and metadata).\n\
        4. When you finish your research and synthesize the final takeaways, always call `memory_store` (with `entry_type` as \"Fact\") to save the summary, keywords, and sources in the local database. This helps you remember this context for future sessions.\n\
        \n\
        === ACADEMIC WORKFLOW GUIDELINES ===\n\
        1. Query formulation: Start by analyzing the user prompt and formulating a precise research plan checklist.\n\
        2. Search & Triangulation: Use `academic_search` to query arXiv, Crossref, OpenAlex, and Semantic Scholar. Deduplicate DOIs.\n\
        3. Web Fetch: Use `web_fetch` to retrieve full texts or summaries from relevant URLs.\n\
        4. Citations: Be extremely rigorous. Always cite assertions using explicit SourceRef IDs (e.g., [1], [2]). Avoid unsupported claims.\n\
        5. Compilation: Generate executive Word reports, PDF documents, or PowerPoint slide decks using formatting tools based on the user's requested deliverable format.".to_string();

        let mut history = messages.to_vec();
        let mut loop_count = 0;
        const MAX_LOOPS: u32 = 10;

        loop {
            loop_count += 1;
            compress_history(&mut history);
            if loop_count > MAX_LOOPS {
                return Err(anyhow::anyhow!("Reached maximum tool execution loop iterations (10)."));
            }

            // 1. Map internal conversation history to OpenAI messages
            let mut openai_messages = Vec::new();
            openai_messages.push(serde_json::json!({
                "role": "system",
                "content": system_prompt.clone()
            }));

            for msg in &history {
                match msg.role {
                    Role::User => {
                        let mut text = String::new();
                        for block in &msg.content {
                            if let ContentBlock::Text(t) = block {
                                text.push_str(t);
                            }
                        }
                        openai_messages.push(serde_json::json!({
                            "role": "user",
                            "content": text
                        }));
                    }
                    Role::Assistant => {
                        let mut text = String::new();
                        let mut tool_calls = Vec::new();
                        for block in &msg.content {
                            match block {
                                ContentBlock::Text(t) => {
                                    text.push_str(t);
                                }
                                ContentBlock::ToolUse { id, name, input, extra_content } => {
                                    let mut tc_val = serde_json::json!({
                                        "id": id.clone(),
                                        "type": "function",
                                        "function": {
                                            "name": name.clone(),
                                            "arguments": input.to_string()
                                        }
                                    });
                                    if let Some(extra) = extra_content {
                                        tc_val["extra_content"] = extra.clone();
                                    }
                                    tool_calls.push(tc_val);
                                }
                                _ => {}
                            }
                        }
                        
                        let mut assistant_msg = serde_json::json!({
                            "role": "assistant"
                        });
                        if !text.is_empty() {
                            assistant_msg["content"] = serde_json::Value::String(text);
                        }
                        if !tool_calls.is_empty() {
                            assistant_msg["tool_calls"] = serde_json::Value::Array(tool_calls);
                        }
                        openai_messages.push(assistant_msg);
                    }
                    Role::Tool => {
                        for block in &msg.content {
                            if let ContentBlock::ToolResult { tool_use_id, content: text, is_error: _ } = block {
                                openai_messages.push(serde_json::json!({
                                    "role": "tool",
                                    "tool_call_id": tool_use_id.clone(),
                                    "content": text.clone()
                                }));
                            }
                        }
                    }
                }
            }

            // 2. Map registered tools to OpenAI tool schemas
            let mut openai_tools = Vec::new();
            for tool in registry.list() {
                openai_tools.push(serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.input_schema()
                    }
                }));
            }

            let mut request_body = serde_json::json!({
                "model": self.model.clone(),
                "messages": openai_messages,
                "temperature": 0.2,
            });

            if !openai_tools.is_empty() {
                request_body["tools"] = serde_json::Value::Array(openai_tools);
            }

            // 3. Dispatch POST request
            let url = if self.api_base.ends_with('/') {
                format!("{}chat/completions", self.api_base)
            } else {
                format!("{}/chat/completions", self.api_base)
            };

            let res = self.client.post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("OpenAI-compatible API failed with HTTP {}: {}", status, err_text));
            }

            let resp_json: serde_json::Value = res.json().await?;

            // 4. Parse response Choices
            let mut assistant_content = Vec::new();
            let mut tool_calls = Vec::new();

            if let Some(choices) = resp_json["choices"].as_array() {
                if let Some(choice) = choices.first() {
                    if let Some(msg) = choice["message"].as_object() {
                        if let Some(content_str) = msg.get("content").and_then(|v| v.as_str()) {
                            if !content_str.is_empty() {
                                assistant_content.push(ContentBlock::Text(content_str.to_string()));
                                let _ = event_tx.send(AgentEvent::TextDelta(content_str.to_string())).await;
                            }
                        }
                        if let Some(tc_array) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                            for tc in tc_array {
                                if let (Some(id), Some(name)) = (tc["id"].as_str(), tc["function"]["name"].as_str()) {
                                    let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                                    let input: serde_json::Value = serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);
                                    let extra_content = tc.get("extra_content").cloned();
                                    tool_calls.push((id.to_string(), name.to_string(), input.clone()));
                                    assistant_content.push(ContentBlock::ToolUse {
                                        id: id.to_string(),
                                        name: name.to_string(),
                                        input,
                                        extra_content,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if assistant_content.is_empty() && tool_calls.is_empty() {
                return Err(anyhow::anyhow!("Received empty response from OpenAI-compatible provider. Payload: {}", resp_json));
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

            // 6. Execute all tool calls
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

fn compress_history(history: &mut [Message]) {
    // 1. Calculate total character count of all ToolResult blocks.
    let mut total_chars = 0;
    for msg in history.iter() {
        for block in &msg.content {
            if let ContentBlock::ToolResult { content, .. } = block {
                total_chars += content.len();
            }
        }
    }

    // Heuristic threshold: if total tool result content exceeds 40,000 characters (~10,000 tokens)
    const MAX_CHARS_THRESHOLD: usize = 40_000;
    if total_chars <= MAX_CHARS_THRESHOLD {
        return;
    }

    tracing::info!("Estimated message context size ({} chars) is over threshold ({} chars). Running history compression.", total_chars, MAX_CHARS_THRESHOLD);

    // 2. Locate all ToolResult indices in the history
    let mut tool_result_blocks = Vec::new();
    for (m_idx, msg) in history.iter().enumerate() {
        for (b_idx, block) in msg.content.iter().enumerate() {
            if let ContentBlock::ToolResult { .. } = block {
                tool_result_blocks.push((m_idx, b_idx));
            }
        }
    }

    let total_tool_results = tool_result_blocks.len();
    if total_tool_results <= 2 {
        return;
    }

    // Compress everything except the last 2 tool results
    let compress_limit = total_tool_results - 2;
    let mut compressed_count = 0;
    for i in 0..compress_limit {
        let (m_idx, b_idx) = tool_result_blocks[i];
        if let ContentBlock::ToolResult { content, .. } = &mut history[m_idx].content[b_idx] {
            if content.len() > 1500 {
                let original_len = content.len();
                let compressed_content = format!(
                    "{} ...\n\n[... Truncated {} characters of older raw tool output to prevent context overflow. Refer to earlier assistant reasoning for extracted facts ...] \n\n...{}",
                    &content[..800],
                    original_len - 1600,
                    &content[original_len - 800..]
                );
                *content = compressed_content;
                compressed_count += 1;
            }
        }
    }
    
    if compressed_count > 0 {
        tracing::info!("Successfully compressed {} older tool result blocks.", compressed_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Role;

    #[test]
    fn test_compress_history() {
        let mut history = vec![
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text("Query 1".to_string())],
            },
            Message {
                role: Role::Tool,
                content: vec![
                    ContentBlock::ToolResult {
                        tool_use_id: "1".to_string(),
                        content: "A".repeat(25000), // 25,000 characters
                        is_error: false,
                    },
                ],
            },
            Message {
                role: Role::Tool,
                content: vec![
                    ContentBlock::ToolResult {
                        tool_use_id: "2".to_string(),
                        content: "B".repeat(25000), // 25,000 characters
                        is_error: false,
                    },
                ],
            },
            Message {
                role: Role::Tool,
                content: vec![
                    ContentBlock::ToolResult {
                        tool_use_id: "3".to_string(),
                        content: "C".repeat(5000), // 5,000 characters (kept intact because it's last)
                        is_error: false,
                    },
                ],
            },
            Message {
                role: Role::Tool,
                content: vec![
                    ContentBlock::ToolResult {
                        tool_use_id: "4".to_string(),
                        content: "D".repeat(5000), // 5,000 characters (kept intact because it's last)
                        is_error: false,
                    },
                ],
            },
        ];

        compress_history(&mut history);

        // ToolResult 1 and 2 should be compressed because:
        // - Total character count of all ToolResults = 25000 + 25000 + 5000 + 5000 = 60,000 (which is > 40,000 threshold)
        // - ToolResult 3 and 4 are the last 2 tool results, so they are kept completely intact.
        // - ToolResult 1 and 2 are older tool results, and their lengths are > 1500 characters, so they are compressed.

        if let ContentBlock::ToolResult { content, .. } = &history[1].content[0] {
            assert!(content.contains("Truncated"));
            assert_eq!(content.len(), 1600 + " ...\n\n[... Truncated 23400 characters of older raw tool output to prevent context overflow. Refer to earlier assistant reasoning for extracted facts ...] \n\n...".len());
        } else {
            panic!("Expected ToolResult block");
        }

        if let ContentBlock::ToolResult { content, .. } = &history[2].content[0] {
            assert!(content.contains("Truncated"));
            assert_eq!(content.len(), 1600 + " ...\n\n[... Truncated 23400 characters of older raw tool output to prevent context overflow. Refer to earlier assistant reasoning for extracted facts ...] \n\n...".len());
        } else {
            panic!("Expected ToolResult block");
        }

        if let ContentBlock::ToolResult { content, .. } = &history[3].content[0] {
            assert_eq!(content, &"C".repeat(5000));
        } else {
            panic!("Expected ToolResult block");
        }

        if let ContentBlock::ToolResult { content, .. } = &history[4].content[0] {
            assert_eq!(content, &"D".repeat(5000));
        } else {
            panic!("Expected ToolResult block");
        }
    }
}

