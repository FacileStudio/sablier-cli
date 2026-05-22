pub mod app;
mod events;
mod handlers;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::{bail, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::task::JoinHandle;

use crate::api::{ApiClient, Project, Task, TimeEntry, User};
use crate::config::Config;

use self::app::{App, Popup};
use self::events::{AppEvent, EventHandler};

enum ActionResult {
    Started(TimeEntry),
    Stopped,
    Paused(TimeEntry),
    Resumed(TimeEntry),
    TimerRefreshed(Option<TimeEntry>),
}

pub async fn run() -> Result<()> {
    let config = Config::load()?;
    if config.token.is_empty() {
        bail!(
            "No API token configured.\n\
             Generate one at your Sablier dashboard (Profile > API Token),\n\
             then add it to ~/.sablier.yml"
        );
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, config).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: Config,
) -> Result<()> {
    let mut app = App::new(config);
    let events = EventHandler::new(Duration::from_millis(500));

    let mut initial_task: Option<JoinHandle<Result<(User, Option<TimeEntry>, Vec<Project>)>>> =
        None;
    let mut action_task: Option<JoinHandle<Result<ActionResult>>> = None;
    let mut tasks_load: Option<JoinHandle<Result<(Project, Vec<Task>)>>> = None;
    let mut entries_task: Option<JoinHandle<Result<Vec<TimeEntry>>>> = None;
    let mut create_task_handle: Option<JoinHandle<Result<(Project, Task)>>> = None;

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        match events.next()? {
            AppEvent::Key(key) => handlers::handle_key(&mut app, key),
            AppEvent::Tick => app.tick(),
        }

        if app.needs_initial_load {
            app.needs_initial_load = false;
            app.loading = true;
            let api = new_api(&app);
            initial_task = Some(tokio::spawn(async move {
                let user = api.me().await?;
                let entry = api.running_entry().await?;
                let projects = api.projects().await?;
                Ok((user, entry, projects))
            }));
        }

        if app.needs_timer_refresh && action_task.is_none() {
            app.needs_timer_refresh = false;
            let api = new_api(&app);
            action_task = Some(tokio::spawn(async move {
                let entry = api.running_entry().await?;
                Ok(ActionResult::TimerRefreshed(entry))
            }));
        }

        if app.needs_stop && action_task.is_none() {
            app.needs_stop = false;
            let api = new_api(&app);
            action_task = Some(tokio::spawn(async move {
                api.stop().await?;
                Ok(ActionResult::Stopped)
            }));
        }

        if app.needs_pause && action_task.is_none() {
            app.needs_pause = false;
            let api = new_api(&app);
            action_task = Some(tokio::spawn(async move {
                let entry = api.pause().await?;
                Ok(ActionResult::Paused(entry))
            }));
        }

        if app.needs_resume && action_task.is_none() {
            app.needs_resume = false;
            let api = new_api(&app);
            action_task = Some(tokio::spawn(async move {
                let entry = api.resume().await?;
                Ok(ActionResult::Resumed(entry))
            }));
        }

        if let Some((project_id, task_id)) = app.needs_start.take() {
            if action_task.is_none() {
                let api = new_api(&app);
                action_task = Some(tokio::spawn(async move {
                    let entry = api.start(project_id, task_id).await?;
                    Ok(ActionResult::Started(entry))
                }));
            }
        }

        if let Some(project) = app.needs_tasks_load.take() {
            if tasks_load.is_none() {
                let api = new_api(&app);
                let p = project.clone();
                tasks_load = Some(tokio::spawn(async move {
                    let tasks = api.tasks(p.id).await?;
                    Ok((p, tasks))
                }));
            }
        }

        if let Some((project, name)) = app.needs_create_task.take() {
            if create_task_handle.is_none() {
                let api = new_api(&app);
                let p = project.clone();
                let n = name.clone();
                create_task_handle = Some(tokio::spawn(async move {
                    let task = api.create_task(p.id, &n).await?;
                    Ok((p, task))
                }));
            }
        }

        if app.needs_entries_load && entries_task.is_none() {
            app.needs_entries_load = false;
            let api = new_api(&app);
            let user_id = app.user.as_ref().map(|u| u.id);
            entries_task = Some(tokio::spawn(async move { api.entries(user_id).await }));
        }

        if let Some(ref handle) = initial_task {
            if handle.is_finished() {
                let handle = initial_task.take().unwrap();
                match handle.await? {
                    Ok((user, entry, projects)) => {
                        app.user = Some(user);
                        app.running_entry = entry;
                        app.projects = projects;
                        app.loading = false;
                    }
                    Err(e) => {
                        app.set_error(format!("Load failed: {}", e));
                        app.loading = false;
                    }
                }
            }
        }

        if let Some(ref handle) = action_task {
            if handle.is_finished() {
                let handle = action_task.take().unwrap();
                match handle.await? {
                    Ok(result) => match result {
                        ActionResult::Started(entry) => {
                            app.set_status("Timer started");
                            app.running_entry = Some(entry);
                        }
                        ActionResult::Stopped => {
                            app.set_status("Timer stopped");
                            app.running_entry = None;
                        }
                        ActionResult::Paused(entry) => {
                            app.set_status("Timer paused");
                            app.running_entry = Some(entry);
                        }
                        ActionResult::Resumed(entry) => {
                            app.set_status("Timer resumed");
                            app.running_entry = Some(entry);
                        }
                        ActionResult::TimerRefreshed(entry) => {
                            app.running_entry = entry;
                        }
                    },
                    Err(e) => {
                        app.set_error(format!("{}", e));
                    }
                }
            }
        }

        if let Some(ref handle) = tasks_load {
            if handle.is_finished() {
                let handle = tasks_load.take().unwrap();
                match handle.await? {
                    Ok((project, tasks)) => {
                        app.cache_tasks(&tasks);
                        if !app.popup_cancelled {
                            app.popup = Some(Popup::PickTask {
                                project,
                                tasks,
                                selected: 0,
                            });
                        }
                    }
                    Err(e) => {
                        app.set_error(format!("Failed to load tasks: {}", e));
                    }
                }
            }
        }

        if let Some(ref handle) = entries_task {
            if handle.is_finished() {
                let handle = entries_task.take().unwrap();
                match handle.await? {
                    Ok(entries) => {
                        app.entries = entries;
                        app.entry_selected = 0;
                    }
                    Err(e) => {
                        app.set_error(format!("Failed to load entries: {}", e));
                    }
                }
            }
        }

        if let Some(ref handle) = create_task_handle {
            if handle.is_finished() {
                let handle = create_task_handle.take().unwrap();
                match handle.await? {
                    Ok((project, task)) => {
                        app.cache_tasks(&[task.clone()]);
                        app.needs_start = Some((project.id, task.id));
                        app.set_status(format!("Created task \"{}\" — starting timer...", task.name));
                    }
                    Err(e) => {
                        app.set_error(format!("Failed to create task: {}", e));
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn new_api(app: &App) -> ApiClient {
    ApiClient::new(&app.config.server_url, &app.config.token)
}
