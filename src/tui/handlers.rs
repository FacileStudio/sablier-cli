use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::app::{App, Focus, Popup, Screen, MENU_ITEMS};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    if app.popup.is_some() {
        handle_popup(app, key);
        return;
    }

    if key.code == KeyCode::Char('q') {
        app.should_quit = true;
        return;
    }

    if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
        app.focus = match app.focus {
            Focus::Sidebar => Focus::Content,
            Focus::Content => Focus::Sidebar,
        };
        return;
    }

    match key.code {
        KeyCode::Char('1') => {
            app.navigate_to(Screen::Timer);
            return;
        }
        KeyCode::Char('2') => {
            app.navigate_to(Screen::Projects);
            return;
        }
        KeyCode::Char('3') => {
            app.navigate_to(Screen::Entries);
            return;
        }
        _ => {}
    }

    match app.focus {
        Focus::Sidebar => handle_sidebar(app, key),
        Focus::Content => match app.screen {
            Screen::Timer => handle_timer(app, key),
            Screen::Projects => handle_projects(app, key),
            Screen::Entries => handle_entries(app, key),
        },
    }
}

fn handle_sidebar(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.menu_selected < MENU_ITEMS.len() - 1 {
                app.menu_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.menu_selected = app.menu_selected.saturating_sub(1);
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            let (_, screen) = MENU_ITEMS[app.menu_selected];
            app.navigate_to(screen);
        }
        _ => {}
    }
}

fn handle_timer(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') | KeyCode::Enter => {
            if app.projects.is_empty() {
                app.set_error("No projects loaded yet");
                return;
            }
            app.popup_cancelled = false;
            app.popup = Some(Popup::PickProject {
                projects: app.projects.clone(),
                selected: 0,
            });
        }
        KeyCode::Char('s') => {
            if app.running_entry.is_some() {
                app.needs_stop = true;
            }
        }
        KeyCode::Char('p') => {
            if let Some(ref entry) = app.running_entry {
                if !entry.is_paused() {
                    app.needs_pause = true;
                }
            }
        }
        KeyCode::Char('r') => {
            if let Some(ref entry) = app.running_entry {
                if entry.is_paused() {
                    app.needs_resume = true;
                }
            } else {
                app.needs_timer_refresh = true;
            }
        }
        _ => {}
    }
}

fn handle_projects(app: &mut App, key: KeyEvent) {
    if app.projects.is_empty() {
        return;
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.project_selected < app.projects.len().saturating_sub(1) {
                app.project_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.project_selected = app.project_selected.saturating_sub(1);
        }
        KeyCode::Char('g') => {
            app.project_selected = 0;
        }
        KeyCode::Char('G') => {
            app.project_selected = app.projects.len().saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_entries(app: &mut App, key: KeyEvent) {
    if app.entries.is_empty() {
        if key.code == KeyCode::Char('r') {
            app.needs_entries_load = true;
        }
        return;
    }
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.entry_selected < app.entries.len().saturating_sub(1) {
                app.entry_selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.entry_selected = app.entry_selected.saturating_sub(1);
        }
        KeyCode::Char('g') => {
            app.entry_selected = 0;
        }
        KeyCode::Char('G') => {
            app.entry_selected = app.entries.len().saturating_sub(1);
        }
        KeyCode::Char('r') => {
            app.needs_entries_load = true;
        }
        _ => {}
    }
}

fn handle_popup(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.popup = None;
            app.popup_cancelled = true;
        }
        KeyCode::Char('j') | KeyCode::Down => popup_move_down(app),
        KeyCode::Char('k') | KeyCode::Up => popup_move_up(app),
        KeyCode::Enter => popup_select(app),
        _ => {}
    }
}

fn popup_move_down(app: &mut App) {
    match app.popup.as_mut() {
        Some(Popup::PickProject {
            selected, projects, ..
        }) => {
            if *selected < projects.len().saturating_sub(1) {
                *selected += 1;
            }
        }
        Some(Popup::PickTask {
            selected, tasks, ..
        }) => {
            if *selected < tasks.len().saturating_sub(1) {
                *selected += 1;
            }
        }
        _ => {}
    }
}

fn popup_move_up(app: &mut App) {
    match app.popup.as_mut() {
        Some(Popup::PickProject { selected, .. }) => {
            *selected = selected.saturating_sub(1);
        }
        Some(Popup::PickTask { selected, .. }) => {
            *selected = selected.saturating_sub(1);
        }
        _ => {}
    }
}

fn popup_select(app: &mut App) {
    let popup = app.popup.take();
    match popup {
        Some(Popup::PickProject {
            projects, selected, ..
        }) => {
            if let Some(project) = projects.get(selected) {
                let project = project.clone();
                app.needs_tasks_load = Some(project);
                app.set_status("Loading tasks...");
            }
        }
        Some(Popup::PickTask {
            tasks,
            selected,
            project,
            ..
        }) => {
            if let Some(task) = tasks.get(selected) {
                app.needs_start = Some((project.id, task.id));
                app.set_status("Starting timer...");
            }
        }
        None => {}
    }
}
