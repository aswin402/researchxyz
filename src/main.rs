pub mod app;
pub mod config;
pub mod core;
pub mod mcp;
pub mod tools;
pub mod tui;

use crate::app::{App, ChatLine};
use crate::config::Config;
use crate::core::types::AgentEvent;
use crate::tui::Theme;
use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tui_textarea::TextArea;

fn init_logging() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_dir = PathBuf::from(home).join(".config/researchxyz/logs");
    fs::create_dir_all(&log_dir)?;
    
    let file_appender = tracing_appender::rolling::daily(log_dir, "researchxyz.log");
    tracing_subscriber::fmt()
        .with_writer(move || file_appender.clone())
        .with_env_filter("info")
        .init();
        
    Ok(())
}

// Simple rolling file appender implementation to avoid adding extra dependency
mod tracing_appender {
    pub mod rolling {
        use std::fs::{File, OpenOptions};
        use std::io::Write;
        use std::path::PathBuf;
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        pub struct DailyAppender {
            file: Arc<Mutex<File>>,
        }

        impl Write for DailyAppender {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.file.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.file.lock().unwrap().flush()
            }
        }

        pub fn daily<P: Into<PathBuf>>(dir: P, filename: &str) -> DailyAppender {
            let path = dir.into().join(filename);
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            DailyAppender {
                file: Arc::new(Mutex::new(file)),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("researchxyz v0.0.1");
        return Ok(());
    }

    if args.contains(&"configure".to_string()) {
        crate::config::run_configure_wizard()?;
        return Ok(());
    }

    // 1. Init configuration and logging
    let _ = init_logging();
    
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(home).join(".config/researchxyz/config.toml");
    let mut config = if config_path.exists() {
        Config::load_from_path(&config_path).unwrap_or_else(|_| Config::default_config())
    } else {
        Config::default_config()
    };
    
    // Create output directory if it doesn't exist
    let output_dir = config.resolve_output_dir();
    fs::create_dir_all(&output_dir)?;

    // 2. Setup raw terminal mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Initialize UI State & Input Box
    let mut app = App::new();
    app.model_name = config.llm.model.clone();
    let theme = Theme::default_dark();
    let mut textarea = TextArea::default();
    
    // Initialize ToolRegistry and register tools
    let mut registry = crate::core::registry::ToolRegistry::new();
    registry.register(std::sync::Arc::new(crate::tools::WebSearchTool::new()));
    registry.register(std::sync::Arc::new(crate::tools::WebFetchTool::new()));
    registry.register(std::sync::Arc::new(crate::tools::AcademicSearchTool));
    registry.register(std::sync::Arc::new(crate::tools::CreateDocxTool));
    registry.register(std::sync::Arc::new(crate::tools::CreatePdfTool));
    registry.register(std::sync::Arc::new(crate::tools::CreatePptxTool));
    registry.register(std::sync::Arc::new(crate::tools::DocReaderTool));
    let registry = std::sync::Arc::new(registry);
    
    let mut history: Vec<crate::core::types::Message> = Vec::new();

    textarea.set_placeholder_text("Ask a research query... (Enter to send, Esc to exit)");

    // 4. Communication Channels
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(100);

    // 5. Main Event Loop
    let mut reader = EventStream::new();
    
    while app.running {
        terminal.draw(|f| {
            tui::draw(f, &app, &theme, &textarea);
        })?;

        tokio::select! {
            // Background agent events
            Some(agent_event) = event_rx.recv() => {
                app.handle_event(agent_event);
            }
            
            // Terminal user inputs
            Some(Ok(event)) = reader.next() => {
                if let Event::Key(key) = event {
                    if app.model_menu_active {
                        match key.code {
                            KeyCode::Up => {
                                if app.model_menu_step == 0 {
                                    if app.selected_provider_idx > 0 {
                                        app.selected_provider_idx -= 1;
                                    } else {
                                        app.selected_provider_idx = app.providers.len() - 1;
                                    }
                                } else {
                                    if app.selected_model_idx > 0 {
                                        app.selected_model_idx -= 1;
                                    } else {
                                        app.selected_model_idx = app.models.len().saturating_sub(1);
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if app.model_menu_step == 0 {
                                    if app.selected_provider_idx + 1 < app.providers.len() {
                                        app.selected_provider_idx += 1;
                                    } else {
                                        app.selected_provider_idx = 0;
                                    }
                                } else {
                                    if !app.models.is_empty() {
                                        if app.selected_model_idx + 1 < app.models.len() {
                                            app.selected_model_idx += 1;
                                        } else {
                                            app.selected_model_idx = 0;
                                        }
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if app.model_menu_step == 0 {
                                    app.update_models_list();
                                    app.model_menu_step = 1;
                                } else {
                                    if !app.models.is_empty() {
                                        let provider = app.providers[app.selected_provider_idx].clone();
                                        let model = app.models[app.selected_model_idx].clone();
                                        config.llm.provider = provider.clone();
                                        config.llm.model = model.clone();
                                        let _ = config.save_to_path(&config_path);
                                        app.model_name = model;
                                    }
                                    app.model_menu_active = false;
                                }
                            }
                            KeyCode::Esc => {
                                app.model_menu_active = false;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            // Exit application
                            KeyCode::Esc => {
                                app.running = false;
                            }
                            
                            // Submit prompt on Enter
                            KeyCode::Enter => {
                                let input_lines = textarea.lines().join("\n");
                                let trimmed = input_lines.trim();
                                if !trimmed.is_empty() {
                                    if trimmed == "/model" {
                                        textarea = TextArea::default();
                                        textarea.set_placeholder_text("Ask a research query... (Enter to send, Esc to exit)");
                                        app.model_menu_active = true;
                                        app.model_menu_step = 0;
                                        app.selected_provider_idx = 0;
                                        app.selected_model_idx = 0;
                                    } else {
                                        // Add user prompt to screen
                                        app.chat_lines.push(ChatLine::UserPrompt(trimmed.to_string()));
                                        app.chat_lines.push(ChatLine::TextDelta {
                                            text: String::new(),
                                            complete: false,
                                        });
                                        app.status = "Thinking...".to_string();
                                        
                                        // Reset text input
                                        textarea = TextArea::default();
                                        textarea.set_placeholder_text("Ask a research query... (Enter to send, Esc to exit)");
                                        
                                        let tx = event_tx.clone();
                                        let prompt = trimmed.to_string();
                                        
                                        // Convert prompt to message history
                                        let user_msg = crate::core::types::Message {
                                            role: crate::core::types::Role::User,
                                            content: vec![crate::core::types::ContentBlock::Text(prompt.clone())],
                                        };
                                        history.push(user_msg.clone());
                                        
                                        let history_clone = history.clone();
                                        let registry_clone = registry.clone();
                                        let config_clone = config.clone();
                                        
                                        tokio::spawn(async move {
                                            match crate::core::agent::resolve_client(&config_clone) {
                                                Ok(client) => {
                                                    match crate::core::agent::run_turn(&history_clone, client, registry_clone, tx.clone()).await {
                                                        Ok(_updated_history) => {
                                                            // Turn completed successfully
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(AgentEvent::TextDelta(format!("\nError running turn: {}\n", e))).await;
                                                            let _ = tx.send(AgentEvent::TurnComplete).await;
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    // Run simulation mode
                                                    let _ = tx.send(AgentEvent::TextDelta(format!("Warning: Client resolution failed: {}. Running in simulation mode...\n", err))).await;
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                    let _ = tx.send(AgentEvent::TextDelta(format!("Beginning simulated literature scan on: {}\n", prompt))).await;
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                                                    
                                                    // 1. Search tool call
                                                    let tool_id = "call_search_1".to_string();
                                                    let _ = tx.send(AgentEvent::ToolCallStarted {
                                                        id: tool_id.clone(),
                                                        name: "academic_search".to_string(),
                                                        input: serde_json::json!({ "query": prompt }),
                                                    }).await;
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                                    
                                                    let _ = tx.send(AgentEvent::ToolCallFinished {
                                                        id: tool_id,
                                                        name: "academic_search".to_string(),
                                                        result: Ok(crate::core::types::ToolResult {
                                                            content: "Successfully fetched papers.".to_string(),
                                                            citations: vec![crate::core::types::SourceRef {
                                                                id: 1,
                                                                url: Some("https://arxiv.org/abs/2103.00001".to_string()),
                                                                doi: Some("10.48550/arXiv.2103.00001".to_string()),
                                                                title: "Literature Survey Paper".to_string(),
                                                            }],
                                                        }),
                                                    }).await;
                                                    
                                                    // 2. Summarization
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                                                    let _ = tx.send(AgentEvent::TextDelta("Synthesizing results... Found 1 highly relevant paper. Generating reports.\n".to_string())).await;
                                                    
                                                    // 3. File compilation tool call
                                                    let doc_tool_id = "call_pdf_1".to_string();
                                                    let _ = tx.send(AgentEvent::ToolCallStarted {
                                                        id: doc_tool_id.clone(),
                                                        name: "create_pdf".to_string(),
                                                        input: serde_json::json!({ "title": prompt }),
                                                    }).await;
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                                                    
                                                    let file_path = PathBuf::from("workspace/documents/literature-scan.pdf");
                                                    let _ = tx.send(AgentEvent::ToolCallFinished {
                                                        id: doc_tool_id,
                                                        name: "create_pdf".to_string(),
                                                        result: Ok(crate::core::types::ToolResult {
                                                            content: "PDF written.".to_string(),
                                                            citations: vec![],
                                                        }),
                                                    }).await;
                                                    
                                                    let _ = tx.send(AgentEvent::FileWritten {
                                                        path: file_path,
                                                        kind: crate::core::types::DocKind::Pdf,
                                                    }).await;
                                                    
                                                    // 4. Completion
                                                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                    let _ = tx.send(AgentEvent::TurnComplete).await;
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            
                            // Pass other keystrokes to the active text box
                            _ => {
                                textarea.input(key);
                            }
                        }
                    }
                }
            }
        }
    }

    // 6. Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}