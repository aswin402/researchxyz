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
            active_tab: ActiveTab::Chat, // Wait! Standard Rust path syntax is ActiveTab::Chat. Let's fix this in CodeContent.
            status: "Idle".to_string(),
            tokens_used: 0,
            model_name: "claude-sonnet-4-6".to_string(),
            running: true,
        }
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
