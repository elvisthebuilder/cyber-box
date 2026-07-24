use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Focus};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let container = if app.container_running {
        Span::styled("● container up", Style::default().fg(Color::Green))
    } else {
        Span::styled("○ container down", Style::default().fg(Color::Red))
    };

    let tor = if app.tor_enabled {
        Span::styled(" TOR: ON ", Style::default().fg(Color::Black).bg(Color::Magenta))
    } else {
        Span::styled(" TOR: OFF ", Style::default().fg(Color::Gray))
    };

    let ai = if app.ai_enabled {
        Span::styled(" AI: ON ", Style::default().fg(Color::Black).bg(Color::Cyan))
    } else {
        Span::styled(" AI: OFF ", Style::default().fg(Color::Gray))
    };

    let focus_label = match app.focus {
        Focus::Browser => "Browser",
        Focus::Output => "Output",
        Focus::Ai => "AI",
    };

    let mut spans = vec![
        Span::raw(" cyber-box  "),
        container,
        Span::raw("  "),
        tor,
        Span::raw("  "),
        ai,
        Span::raw(format!("  focus: {focus_label}  ")),
        Span::styled(
            "Tab: switch | i: install | Ctrl+T: tor | Ctrl+A: ai | q: quit",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    if let Some(msg) = &app.status_message {
        spans.push(Span::raw("  |  "));
        spans.push(Span::styled(msg, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}
