use crate::core::types::{DocKind, AgentEvent};
use std::path::PathBuf;

#[derive(Debug)]
pub enum ChatLine {
    UserPrompt(String),
    TextDelta { text: String, complete: bool },
    ToolStatus(ToolStatusLine),
    FileWritten { path: PathBuf, kind: DocKind },
    Separator,
}

#[derive(Debug, Clone)]
pub struct ToolStatusLine {
    pub id: String,
    pub name: String,
    pub state: ToolState,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolState {
    Running,
    Done,
    Failed,
    RateLimited,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActiveTab {
    Chat,
    Logs,
    Documents,
}

pub struct App {
    pub chat_lines: Vec<ChatLine>,
    pub input_buffer: String,
    pub active_tab: ActiveTab,
    pub status: String,
    pub tokens_used: usize,
    pub model_name: String,
    pub running: bool,
    
    // Model selection menu state
    pub model_menu_active: bool,
    pub model_menu_step: u8, // 0 = select provider, 1 = select model
    pub selected_provider_idx: usize,
    pub selected_model_idx: usize,
    pub providers: Vec<String>,
    pub models: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            chat_lines: vec![
                ChatLine::TextDelta {
                    text: "Welcome to researchxyz! Enter a research topic or paper query to begin.".to_string(),
                    complete: true,
                }
            ],
            input_buffer: String::new(),
            active_tab: ActiveTab::Chat,
            status: "Idle".to_string(),
            tokens_used: 0,
            model_name: "claude-sonnet-4-6".to_string(),
            running: true,
            
            model_menu_active: false,
            model_menu_step: 0,
            selected_provider_idx: 0,
            selected_model_idx: 0,
            providers: vec![
                "anthropic".to_string(),
                "openai".to_string(),
                "deepseek".to_string(),
                "groq".to_string(),
                "openrouter".to_string(),
                "google_ai_studio".to_string(),
                "auto".to_string(),
            ],
            models: Vec::new(),
        }
    }

    pub fn update_models_list(&mut self) {
        let provider = &self.providers[self.selected_provider_idx];
        self.models = match provider.as_str() {
            "anthropic" => vec![
                "claude-3-5-sonnet-latest".to_string(),
                "claude-3-5-haiku-latest".to_string(),
                "claude-3-opus-latest".to_string(),
            ],
            "openai" => vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "o1-mini".to_string(),
                "o3-mini".to_string(),
            ],
            "deepseek" => vec![
                "deepseek-chat".to_string(),
                "deepseek-reasoner".to_string(),
            ],
            "groq" => vec![
                "llama-3.3-70b-versatile".to_string(),
                "mixtral-8x7b-32768".to_string(),
            ],
            "openrouter" => vec![
                "google/gemini-2.0-flash-exp:free".to_string(),
                "meta-llama/llama-3.3-70b-instruct:free".to_string(),
                "deepseek/deepseek-chat".to_string(),
            ],
            "google_ai_studio" => vec![
                "gemini-2.5-flash".to_string(),
                "gemini-2.5-pro".to_string(),
            ],
            _ => vec![
                "auto-detect".to_string(),
            ],
        };
        self.selected_model_idx = 0;
    }

    pub fn handle_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::TextDelta(text) => {
                if let Some(ChatLine::TextDelta { text: ref mut t, ref mut complete }) = self.chat_lines.last_mut() {
                    if !*complete {
                        t.push_str(&text);
                        return;
                    }
                }
                self.chat_lines.push(ChatLine::TextDelta { text, complete: false });
            }
            AgentEvent::ToolCallStarted { id, name, input } => {
                self.status = format!("Running tool: {}", name);
                self.chat_lines.push(ChatLine::ToolStatus(ToolStatusLine {
                    id,
                    name,
                    state: ToolState::Running,
                    detail: Some(input.to_string()),
                }));
            }
            AgentEvent::ToolCallFinished { id, name: _, result } => {
                self.status = "Idle".to_string();
                if let Some(pos) = self.chat_lines.iter().position(|l| {
                    if let ChatLine::ToolStatus(ts) = l {
                        ts.id == id
                    } else {
                        false
                    }
                }) {
                    if let ChatLine::ToolStatus(ref mut ts) = self.chat_lines[pos] {
                        match result {
                            Ok(res) => {
                                ts.state = ToolState::Done;
                                ts.detail = Some(format!("Citations: {}", res.citations.len()));
                            }
                            Err(err) => {
                                ts.state = ToolState::Failed;
                                ts.detail = Some(err.to_string());
                            }
                        }
                    }
                }
            }
            AgentEvent::FileWritten { path, kind } => {
                self.chat_lines.push(ChatLine::FileWritten { path, kind });
            }
            AgentEvent::TurnComplete => {
                if let Some(ChatLine::TextDelta { complete, .. }) = self.chat_lines.last_mut() {
                    *complete = true;
                }
                self.status = "Idle".to_string();
            }
        }
    }
}
