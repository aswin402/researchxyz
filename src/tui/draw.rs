use crate::app::{App, ChatLine, ToolState};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App, theme: &Theme, textarea: &tui_textarea::TextArea<'_>) {
    // 1. Draw background
    let bg_block = Block::default().style(Style::default().bg(theme.bg));
    frame.render_widget(bg_block, frame.area());

    // 2. Define Layout (3 rows: Chat pane, Status line, Input area)
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(5), // input box height
    ])
    .split(frame.area());

    // 3. Render Conversation Pane
    draw_conversation_pane(frame, chunks[0], app, theme);

    // 4. Render Status Bar
    draw_status_bar(frame, chunks[1], app, theme);

    // 5. Render Input Box
    draw_input_box(frame, chunks[2], theme, textarea);
}

fn draw_conversation_pane(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mut lines = Vec::new();

    for chat_line in &app.chat_lines {
        match chat_line {
            ChatLine::UserPrompt(prompt) => {
                lines.push(Line::from(vec![
                    Span::styled("› ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(prompt, Style::default().fg(theme.text)),
                ]));
                lines.push(Line::from("")); // Separator empty line
            }
            ChatLine::TextDelta { text, complete } => {
                let mut spans = vec![
                    Span::styled("    ", Style::default()), // 4 space indentation
                    Span::styled(text, Style::default().fg(theme.text)),
                ];
                if !complete {
                    spans.push(Span::styled("█", Style::default().fg(theme.accent)));
                }
                lines.push(Line::from(spans));
                lines.push(Line::from(""));
            }
            ChatLine::ToolStatus(ts) => {
                let (icon, color) = match ts.state {
                    ToolState::Running => ("⠋", theme.text_dim),
                    ToolState::Done => ("✓", theme.success),
                    ToolState::Failed => ("✗", theme.error),
                    ToolState::RateLimited => ("↻", theme.warn),
                };
                
                let detail = ts.detail.as_deref().unwrap_or("");
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ", ts.name), Style::default().fg(theme.accent)),
                    Span::styled(format!("· {}", detail), Style::default().fg(theme.text_dim)),
                ]));
            }
            ChatLine::FileWritten { path, kind } => {
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled("📁 Written ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:?} ", kind), Style::default().fg(theme.text)),
                    Span::styled(format!("-> {}", path.display()), Style::default().fg(theme.text_dim).add_modifier(Modifier::UNDERLINED)),
                ]));
                lines.push(Line::from(""));
            }
            ChatLine::Separator => {
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled("──────────────────────────────────────────────────", Style::default().fg(theme.border)),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::NONE));
    
    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let status_style = Style::default().bg(theme.surface).fg(theme.text_dim);
    
    let status_text = format!(
        " Model: {}  |  Tokens: {}  |  Status: {}",
        app.model_name, app.tokens_used, app.status
    );
    
    let paragraph = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::NONE));
        
    frame.render_widget(paragraph, area);
}

fn draw_input_box(frame: &mut Frame, area: Rect, theme: &Theme, textarea: &tui_textarea::TextArea<'_>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.surface))
        .title(Span::styled(" Research Prompt ", Style::default().fg(theme.accent)));

    let mut inner_area = area;
    inner_area.x += 1;
    inner_area.y += 1;
    inner_area.width -= 2;
    inner_area.height -= 2;

    frame.render_widget(block, area);
    frame.render_widget(textarea.widget(), inner_area);
}
