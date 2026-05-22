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
            let total_items = tasks.len() + 1; // +1 for "+ New Task"
            let area = centered_popup(60, total_items as u16 + 5, frame.area());
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

            let mut items: Vec<ListItem> = tasks
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

            let new_task_idx = tasks.len();
            let new_style = if selected == new_task_idx {
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::ACCENT)
            };
            let new_prefix = if selected == new_task_idx {
                " ▸ "
            } else {
                "   "
            };
            items.push(ListItem::new(Line::from(Span::styled(
                format!("{}+ New Task", new_prefix),
                new_style,
            ))));

            let list = List::new(items);
            let mut state = ListState::default().with_selected(Some(selected));
            frame.render_stateful_widget(list, inner, &mut state);
        }
        Popup::CreateTask { project, input } => {
            let area = centered_popup(60, 7, frame.area());
            frame.render_widget(Clear, area);

            let block = Block::default()
                .title(format!(" {} — New Task ", project.name))
                .title_style(
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                )
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::ACCENT));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let chunks = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner);

            let label = Paragraph::new(Line::from(Span::styled(
                " Task name:",
                Style::default().fg(theme::SECONDARY),
            )));
            frame.render_widget(label, chunks[0]);

            let cursor_display = format!(" {}_", input);
            let input_line = Paragraph::new(Line::from(Span::styled(
                cursor_display,
                theme::selected(),
            )));
            frame.render_widget(input_line, chunks[1]);

            let hint = Paragraph::new(Line::from(Span::styled(
                " Enter confirm · Esc cancel",
                theme::dim(),
            )));
            frame.render_widget(hint, chunks[2]);
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
