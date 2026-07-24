use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Focus};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Output;
    let border_style = if focused { Style::default().fg(Color::Cyan) } else { Style::default() };

    let title = match &app.session {
        Some(s) => format!("Output — {} (j/k to scroll)", s.label),
        None => "Output — press Enter on a tool to run it".to_string(),
    };

    let lines: Vec<Line> = app
        .session
        .as_ref()
        .map(|s| s.lines.iter().map(|l| Line::from(l.as_str())).collect())
        .unwrap_or_default();

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(border_style).title(title))
        .wrap(Wrap { trim: false })
        .scroll((app.output_scroll, 0));

    f.render_widget(paragraph, area);
}
