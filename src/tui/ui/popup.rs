use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::app::{App, Popup};

use super::theme;

pub fn render(frame: &mut Frame, app: &mut App) {
    let popup = match &app.popup {
        Some(p) => p,
        None => return,
    };

    match popup {
        Popup::PickProject { projects, selected } => {
            let selected = *selected;
            let area = centered_popup(60, projects.len() as u16 + 4, frame.area());
            frame.render_widget(Clear, area);

            let block = Block::default()
                .title(" Select Project ")
                .title_style(
                    Style::default()
                        .fg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                )
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::PRIMARY));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let items: Vec<ListItem> = projects
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let style = if i == selected {
                        theme::selected()
                    } else {
                        Style::default()
                    };
                    let prefix = if i == selected { " ▸ " } else { "   " };
                    ListItem::new(Line::from(Span::styled(
                        format!("{}{}", prefix, p.name),
                        style,
                    )))
                })
                .collect();

            let list = List::new(items);
            let mut state = ListState::default().with_selected(Some(selected));
            frame.render_stateful_widget(list, inner, &mut state);
        }
        Popup::PickTask {
            project,
            tasks,
            selected,
        } => {
            let selected = *selected;
            let area = centered_popup(60, tasks.len() as u16 + 5, frame.area());
            frame.render_widget(Clear, area);

            let block = Block::default()
                .title(format!(" {} — Select Task ", project.name))
                .title_style(
                    Style::default()
                        .fg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                )
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::PRIMARY));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            if tasks.is_empty() {
                let text = Paragraph::new(Line::from(Span::styled(
                    "No tasks in this project",
                    theme::dim(),
                )))
                .alignment(Alignment::Center);
                frame.render_widget(text, inner);
                return;
            }

            let items: Vec<ListItem> = tasks
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let style = if i == selected {
                        theme::selected()
                    } else {
                        Style::default()
                    };
                    let prefix = if i == selected { " ▸ " } else { "   " };
                    ListItem::new(Line::from(Span::styled(
                        format!("{}{}", prefix, t.name),
                        style,
                    )))
                })
                .collect();

            let list = List::new(items);
            let mut state = ListState::default().with_selected(Some(selected));
            frame.render_stateful_widget(list, inner, &mut state);
        }
    }
}

fn centered_popup(width: u16, height: u16, area: Rect) -> Rect {
    let height = height.min(area.height.saturating_sub(4));
    let width = width.min(area.width.saturating_sub(4));

    let vertical = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1])[1]
}
