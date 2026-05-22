use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

use super::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    if app.loading {
        render_loading(frame, area);
        return;
    }

    match &app.running_entry {
        Some(entry) => render_running(frame, app, entry.clone(), area),
        None => render_idle(frame, area),
    }
}

fn render_loading(frame: &mut Frame, area: Rect) {
    let center = centered_rect(50, 40, area);
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("Loading...", theme::dim())),
    ];
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, center);
}

fn render_idle(frame: &mut Frame, area: Rect) {
    let center = centered_rect(44, 9, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MUTED));

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "No timer running",
            theme::dim(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press  n  to start a timer",
            Style::default().fg(theme::PRIMARY),
        )),
        Line::from(""),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, center);
}

fn render_running(frame: &mut Frame, app: &App, entry: crate::api::TimeEntry, area: Rect) {
    let center = centered_rect(50, 15, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(center);

    let elapsed = entry.elapsed_display();
    let time_style = if entry.is_paused() {
        theme::status_paused()
    } else {
        theme::status_running()
    };

    let time_line = Line::from(Span::styled(
        elapsed,
        time_style.add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(
        Paragraph::new(vec![Line::from(""), time_line, Line::from("")])
            .alignment(Alignment::Center),
        chunks[1],
    );

    let status_indicator = if entry.is_paused() { "⏸ Paused" } else { "● Running" };
    let status_style = if entry.is_paused() {
        theme::status_paused()
    } else {
        theme::status_running()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(status_indicator, status_style)))
            .alignment(Alignment::Center),
        chunks[3],
    );

    let project_name = app.project_name(entry.project_id);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  Project  ", theme::dim()),
            Span::styled(project_name, theme::bold()),
        ]))
        .alignment(Alignment::Center),
        chunks[5],
    );

    let task_label = app.task_name_from_entries(entry.task_id);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  Task     ", theme::dim()),
            Span::styled(task_label, theme::bold()),
        ]))
        .alignment(Alignment::Center),
        chunks[6],
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
