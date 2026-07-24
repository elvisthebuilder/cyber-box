use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, InputMode, InstallStatus};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Browser;
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .flat_list
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = i == app.selected;
            match entry.tool_index {
                None => ListItem::new(Line::from(Span::styled(
                    format!(" {}", entry.category.to_uppercase()),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))),
                Some(idx) => {
                    let tool = app
                        .registry
                        .tools
                        .iter()
                        .filter(|t| t.category == entry.category)
                        .nth(idx);
                    let name = tool.map(|t| t.name.as_str()).unwrap_or("?");
                    let desc = tool.map(|t| t.description.as_str()).unwrap_or("");
                    let marker = tool
                        .and_then(|t| app.install_status.get(&t.name))
                        .map(|s| match s {
                            InstallStatus::Installed | InstallStatus::Unknown => "",
                            InstallStatus::NotInstalled => "[not installed, i to install] ",
                            InstallStatus::Installing => "[installing...] ",
                            InstallStatus::Failed(_) => "[install failed] ",
                        })
                        .unwrap_or("");
                    let base_style = if selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else if marker.is_empty() {
                        Style::default()
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    ListItem::new(Line::from(Span::styled(format!("   {marker}{name} — {desc}"), base_style)))
                }
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title("Tools (j/k, Enter to launch, i to install)"),
    );
    f.render_widget(list, area);

    if app.input_mode == InputMode::TargetPrompt {
        let prompt_area = Rect {
            x: area.x + 1,
            y: area.y + area.height.saturating_sub(2),
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let prompt = Paragraph::new(format!("target> {}_", app.target_input))
            .style(Style::default().fg(Color::Black).bg(Color::Yellow));
        f.render_widget(prompt, prompt_area);
    }
}
