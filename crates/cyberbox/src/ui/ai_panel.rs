use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Focus};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Ai;
    let border_style = if focused { Style::default().fg(Color::Cyan) } else { Style::default() };

    if !app.ai_enabled {
        let placeholder = Paragraph::new("AI assistant disabled — press Ctrl+A to enable")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).border_style(border_style).title("AI Assistant"));
        f.render_widget(placeholder, area);
        return;
    }

    let mut lines: Vec<Line> = app.ai_lines.iter().map(|l| Line::from(l.as_str())).collect();
    if focused {
        lines.push(Line::from(format!("ask> {}_", app.ai_input)));
    } else if app.ai_busy {
        lines.push(Line::from("(thinking...)"));
    }

    let title = if app.ai_busy { "AI Assistant (thinking...)" } else { "AI Assistant (Tab/click to ask)" };
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(border_style).title(title))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
