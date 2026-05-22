use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::{App, Screen};

use super::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref err) = app.error_message {
        let line = Line::from(Span::styled(
            format!(" {}", err),
            Style::default().fg(theme::ERROR),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    if let Some(ref msg) = app.status_message {
        let line = Line::from(Span::styled(
            format!(" {}", msg),
            Style::default().fg(theme::SUCCESS),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let hints = match app.screen {
        Screen::Timer => {
            if app.running_entry.is_some() {
                " s stop · p pause · r resume · n new · q quit"
            } else {
                " n new timer · r refresh · q quit"
            }
        }
        Screen::Projects => " j/k navigate · q quit",
        Screen::Entries => " j/k navigate · r refresh · q quit",
    };

    let line = Line::from(Span::styled(hints, theme::dim()));
    frame.render_widget(Paragraph::new(line), area);
}
