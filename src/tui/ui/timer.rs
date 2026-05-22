use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use tui_big_text::{BigText, PixelSize};

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
        Line::from(Span::styled(
            "Loading...",
            Style::default().fg(theme::PRIMARY),
        )),
    ];
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);
    frame.render_widget(paragraph, center);
}

fn render_idle(frame: &mut Frame, area: Rect) {
    let center = centered_rect(60, 40, area);

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(center);

    let lines = vec![
        Line::from(Span::styled(
            "No timer running",
            Style::default().fg(theme::SECONDARY),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme::SECONDARY)),
            Span::styled(
                " n ",
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to start a new timer", Style::default().fg(theme::SECONDARY)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("or use ", Style::default().fg(theme::SECONDARY)),
            Span::styled(
                "sablier start",
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" from CLI", Style::default().fg(theme::SECONDARY)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, chunks[1]);
}

fn pick_pixel_size(height: u16, width: u16) -> Option<PixelSize> {
    if height >= 16 && width >= 64 {
        Some(PixelSize::Full)
    } else if height >= 12 && width >= 64 {
        Some(PixelSize::HalfHeight)
    } else if height >= 10 && width >= 32 {
        Some(PixelSize::Quadrant)
    } else {
        None
    }
}

fn big_text_height(px: PixelSize) -> u16 {
    match px {
        PixelSize::Full => 8,
        PixelSize::HalfHeight => 4,
        PixelSize::HalfWidth => 8,
        PixelSize::Quadrant => 4,
        PixelSize::ThirdHeight => 3,
        PixelSize::Sextant => 3,
    }
}

fn render_running(frame: &mut Frame, app: &App, entry: crate::api::TimeEntry, area: Rect) {
    let elapsed = entry.elapsed_display();
    let is_paused = entry.is_paused();

    let time_color = if is_paused { theme::PAUSED } else { theme::SUCCESS };

    let pixel_size = pick_pixel_size(area.height, area.width);

    let timer_height = pixel_size.map_or(1, big_text_height);
    let info_lines: u16 = 4;
    let total_content = timer_height + info_lines;
    let v_pad = area.height.saturating_sub(total_content) / 2;

    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(v_pad),
            Constraint::Length(timer_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    if let Some(px_size) = pixel_size {
        let parts: Vec<&str> = elapsed.split(':').collect();
        let time_line = if parts.len() == 3 {
            Line::from(vec![
                Span::styled(parts[0], Style::default().fg(time_color)),
                Span::styled(":", Style::default().fg(theme::MUTED)),
                Span::styled(parts[1], Style::default().fg(time_color)),
                Span::styled(":", Style::default().fg(theme::MUTED)),
                Span::styled(parts[2], Style::default().fg(time_color)),
            ])
        } else {
            Line::from(Span::styled(
                &elapsed,
                Style::default().fg(time_color),
            ))
        };

        let big_text = BigText::builder()
            .pixel_size(px_size)
            .lines(vec![time_line])
            .centered()
            .build();

        frame.render_widget(big_text, chunks[1]);
    } else {
        let time_style = if is_paused {
            theme::status_paused()
        } else {
            theme::status_running()
        };
        let line = Line::from(Span::styled(
            &elapsed,
            time_style.add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            chunks[1],
        );
    }

    let (status_text, status_style) = if is_paused {
        ("\u{23f8}  Paused", theme::status_paused())
    } else {
        ("\u{25cf}  Running", theme::status_running())
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(status_text, status_style)))
            .alignment(Alignment::Center),
        chunks[3],
    );

    let project_name = app.project_name(entry.project_id);
    let task_name = app.task_name(entry.task_id, Some(&entry.task_name));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Project ", Style::default().fg(theme::SECONDARY)),
            Span::styled(project_name, theme::bold()),
        ]))
        .alignment(Alignment::Center),
        chunks[4],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Task    ", Style::default().fg(theme::SECONDARY)),
            Span::styled(task_name, theme::bold()),
        ]))
        .alignment(Alignment::Center),
        chunks[5],
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
