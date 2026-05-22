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
        .title(" Projects ")
        .title_style(Style::default().fg(theme::PRIMARY).add_modifier(Modifier::BOLD))
        .borders(Borders::NONE)
        .padding(Padding::new(2, 2, 1, 0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.projects.is_empty() {
        let text = Paragraph::new(Line::from(Span::styled(
            "No projects found",
            theme::dim(),
        )));
        frame.render_widget(text, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let style = if i == app.project_selected {
                theme::selected()
            } else {
                Style::default()
            };
            let prefix = if i == app.project_selected {
                " ▸ "
            } else {
                "   "
            };
            let desc = if project.description.is_empty() {
                String::new()
            } else {
                format!("  {}", project.description)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}{}", prefix, project.name), style),
                Span::styled(desc, theme::dim()),
            ]))
        })
        .collect();

    let list = List::new(items);
    let mut state = ListState::default().with_selected(Some(app.project_selected));
    frame.render_stateful_widget(list, inner, &mut state);
}
