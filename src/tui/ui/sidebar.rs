use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};

use crate::tui::app::{App, Focus, MENU_ITEMS};

use super::theme;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Sidebar;

    let border_style = if is_focused {
        Style::default().fg(theme::PRIMARY)
    } else {
        Style::default().fg(theme::MUTED)
    };

    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(border_style)
        .padding(Padding::new(1, 1, 1, 0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let user_name = app
        .user
        .as_ref()
        .map(|u| {
            if u.name.is_empty() {
                u.email.clone()
            } else {
                u.name.clone()
            }
        })
        .unwrap_or_else(|| "...".to_string());

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            " sablier",
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(" {}", user_name),
            theme::dim(),
        )),
        Line::from(""),
    ];

    for (i, (label, _)) in MENU_ITEMS.iter().enumerate() {
        let is_sel = i == app.menu_selected;
        let prefix = if is_sel { " ▸ " } else { "   " };
        let style = if is_sel && is_focused {
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD)
        } else if is_sel {
            theme::bold()
        } else {
            theme::dim()
        };
        lines.push(Line::from(Span::styled(format!("{}{}", prefix, label), style)));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
