use ratatui::style::Color;

pub struct Theme {
    pub bg:         Color,  // Frame background
    pub surface:    Color,  // Status bar, input area background
    pub border:     Color,  // Very subtle separator lines
    pub text:       Color,  // Main readable text
    pub text_dim:   Color,  // Tool status lines, timestamps, labels
    pub text_faint: Color,  // Placeholder text in input box
    pub accent:     Color,  // Prompt symbol ›, cursor block, active tool name
    pub success:    Color,  // ✓ completed tool call
    pub error:      Color,  // ✗ failed tool call
    pub warn:       Color,  // Rate-limit / retry notices
}

impl Theme {
    pub fn default_dark() -> Self {
        Self {
            bg:         Color::Rgb(11,  14,  20),
            surface:    Color::Rgb(17,  20,  28),
            border:     Color::Rgb(38,  42,  54),
            text:       Color::Rgb(212, 212, 216),
            text_dim:   Color::Rgb(107, 114, 128),
            text_faint: Color::Rgb(60,  65,  80),
            accent:     Color::Rgb(125, 211, 192),  // Teal
            success:    Color::Rgb(95,  184, 138),  // Green
            error:      Color::Rgb(226, 87,  76),   // Red
            warn:       Color::Rgb(234, 179, 8),    // Amber
        }
    }
}
