mod entries;
mod footer;
mod popup;
mod projects;
mod sidebar;
pub mod theme;
mod timer;

use ratatui::{
    layout::{Constraint, Layout},
    Frame,
};

use crate::tui::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &mut App) {
    let outer = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let main = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(outer[0]);

    sidebar::render(frame, app, main[0]);

    match app.screen {
        Screen::Timer => timer::render(frame, app, main[1]),
        Screen::Projects => projects::render(frame, app, main[1]),
        Screen::Entries => entries::render(frame, app, main[1]),
    }

    footer::render(frame, app, outer[1]);

    popup::render(frame, app);
}
