use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
    Frame,
};

use crate::tui::app::App;

use super::theme;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Entries ")
        .title_style(Style::default().fg(theme::PRIMARY).add_modifier(Modifier::BOLD))
        .borders(Borders::NONE)
        .padding(Padding::new(2, 2, 1, 0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.entries.is_empty() {
        let msg = if app.loading {
            "Loading entries..."
        } else {
            "No entries yet  (r to refresh)"
        };
        let text = Paragraph::new(Line::from(Span::styled(msg, theme::dim())));
        frame.render_widget(text, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_sel = i == app.entry_selected;
            let style = if is_sel {
                theme::selected()
            } else {
                Style::default()
            };

            let project = app.project_name(entry.project_id);
            let elapsed = entry.elapsed_display();

            let status_span = match entry.status_label() {
                "Running" => Span::styled(
                    " ● ",
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
                "Paused" => Span::styled(
                    " ⏸ ",
                    Style::default().fg(theme::PAUSED),
                ),
                _ => Span::styled("   ", theme::dim()),
            };

            let started = &entry.started_at;
            let date_part = if started.len() >= 10 {
                &started[..10]
            } else {
                started.as_str()
            };

            let prefix = if is_sel { "▸ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                status_span,
                Span::styled(format!("{:<20}", project), style),
                Span::styled(format!("{:<12}", date_part), theme::dim()),
                Span::styled(elapsed, style),
            ]))
        })
        .collect();

    let list = List::new(items);
    let mut state = ListState::default().with_selected(Some(app.entry_selected));
    frame.render_stateful_widget(list, inner, &mut state);
}
