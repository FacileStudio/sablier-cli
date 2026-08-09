use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::app::{App, Popup, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    if app.pending_g && !matches!(app.popup, Some(Popup::CreateTask { .. })) {
        app.pending_g = false;
        if key.code == KeyCode::Char('g') {
            if app.popup.is_some() {
                popup_jump_top(app);
            } else {
                match app.screen {
                    Screen::Projects => app.project_selected = 0,
                    Screen::Entries => app.entry_selected = 0,
                    _ => {}
                }
            }
            return;
        }
    }

    if app.popup.is_some() {
        handle_popup(app, key);
        return;
    }

    if key.code == KeyCode::Char('q') {
        app.should_quit = true;
        return;
    }

    if key.code == KeyCode::Tab {
        app.tab_next_screen();
        return;
    }
    if key.code == KeyCode::BackTab {
        app.tab_prev_screen();
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

    match app.screen {
        Screen::Timer => handle_timer(app, key),
        Screen::Projects => handle_projects(app, key),
        Screen::Entries => handle_entries(app, key),
    }
}

fn handle_timer(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('n') => {
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
            app.pending_g = true;
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
            app.pending_g = true;
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
    if matches!(app.popup, Some(Popup::CreateTask { .. })) {
        handle_create_task_input(app, key);
        return;
    }

    match key.code {
        KeyCode::Esc => {
            if matches!(app.popup, Some(Popup::PickTask { .. })) {
                app.popup = Some(Popup::PickProject {
                    projects: app.projects.clone(),
                    selected: 0,
                });
            } else {
                app.popup = None;
                app.popup_cancelled = true;
            }
        }
        KeyCode::Char('j') | KeyCode::Down => popup_move_down(app),
        KeyCode::Char('k') | KeyCode::Up => popup_move_up(app),
        KeyCode::Char('g') => app.pending_g = true,
        KeyCode::Char('G') => popup_jump_bottom(app),
        KeyCode::Enter => popup_select(app),
        _ => {}
    }
}

fn handle_create_task_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            if let Some(Popup::CreateTask { project, .. }) = app.popup.take() {
                app.popup = Some(Popup::PickTask {
                    project,
                    tasks: Vec::new(),
                    selected: 0,
                });
                app.needs_tasks_load = app.popup.as_ref().and_then(|p| match p {
                    Popup::PickTask { project, .. } => Some(project.clone()),
                    _ => None,
                });
            }
        }
        KeyCode::Enter => {
            if let Some(Popup::CreateTask { project, input }) = app.popup.take() {
                let name = input.trim().to_string();
                if name.is_empty() {
                    app.popup = Some(Popup::CreateTask { project, input });
                    return;
                }
                app.needs_create_task = Some((project, name));
                app.set_status("Creating task...");
            }
        }
        KeyCode::Backspace => {
            if let Some(Popup::CreateTask { ref mut input, .. }) = app.popup {
                input.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Some(Popup::CreateTask { ref mut input, .. }) = app.popup {
                input.push(c);
            }
        }
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
            let max = tasks.len(); // tasks.len() = "+ New Task" item
            if *selected < max {
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

fn popup_jump_top(app: &mut App) {
    match app.popup.as_mut() {
        Some(Popup::PickProject { selected, .. }) => *selected = 0,
        Some(Popup::PickTask { selected, .. }) => *selected = 0,
        _ => {}
    }
}

fn popup_jump_bottom(app: &mut App) {
    match app.popup.as_mut() {
        Some(Popup::PickProject {
            selected, projects, ..
        }) => *selected = projects.len().saturating_sub(1),
        Some(Popup::PickTask {
            selected, tasks, ..
        }) => *selected = tasks.len(), // tasks.len() = "+ New Task" item
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
            if selected == tasks.len() {
                app.popup = Some(Popup::CreateTask {
                    project,
                    input: String::new(),
                });
            } else if let Some(task) = tasks.get(selected) {
                app.needs_start = Some((project.id, task.id));
                app.set_status("Starting timer...");
            }
        }
        Some(Popup::CreateTask { .. }) | None => {}
    }
}
