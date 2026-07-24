mod ai_panel;
mod output_pane;
mod status_bar;
mod tool_browser;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(f.area());

    status_bar::draw(f, root[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(root[1]);

    tool_browser::draw(f, body[0], app);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(body[1]);

    output_pane::draw(f, right[0], app);
    ai_panel::draw(f, right[1], app);
}
