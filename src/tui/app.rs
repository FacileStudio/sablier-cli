use std::collections::HashMap;

use crate::api::{Project, Task, TimeEntry, User};
use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Timer,
    Projects,
    Entries,
}

#[derive(Debug, Clone)]
pub enum Popup {
    PickProject {
        projects: Vec<Project>,
        selected: usize,
    },
    PickTask {
        project: Project,
        tasks: Vec<Task>,
        selected: usize,
    },
}

pub const MENU_ITEMS: &[(&str, Screen)] = &[
    ("Timer", Screen::Timer),
    ("Projects", Screen::Projects),
    ("Entries", Screen::Entries),
];

pub struct App {
    pub config: Config,

    pub screen: Screen,
    pub menu_selected: usize,

    pub user: Option<User>,
    pub running_entry: Option<TimeEntry>,

    pub projects: Vec<Project>,
    pub project_selected: usize,

    pub task_names: HashMap<i64, String>,

    pub entries: Vec<TimeEntry>,
    pub entry_selected: usize,

    pub popup: Option<Popup>,
    pub popup_cancelled: bool,

    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub status_ttl: u8,

    pub loading: bool,
    pub should_quit: bool,

    pub needs_initial_load: bool,
    pub needs_timer_refresh: bool,
    pub needs_entries_load: bool,
    pub needs_tasks_load: Option<Project>,
    pub needs_start: Option<(i64, i64)>,
    pub needs_stop: bool,
    pub needs_pause: bool,
    pub needs_resume: bool,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            screen: Screen::Timer,
            menu_selected: 0,
            user: None,
            running_entry: None,
            projects: Vec::new(),
            project_selected: 0,
            task_names: HashMap::new(),
            entries: Vec::new(),
            entry_selected: 0,
            popup: None,
            popup_cancelled: false,
            status_message: None,
            error_message: None,
            status_ttl: 0,
            loading: false,
            should_quit: false,
            needs_initial_load: true,
            needs_timer_refresh: false,
            needs_entries_load: false,
            needs_tasks_load: None,
            needs_start: None,
            needs_stop: false,
            needs_pause: false,
            needs_resume: false,
        }
    }

    pub fn tab_next_screen(&mut self) {
        match self.screen {
            Screen::Timer => self.navigate_to(Screen::Projects),
            Screen::Projects => self.navigate_to(Screen::Entries),
            Screen::Entries => self.navigate_to(Screen::Timer),
        }
    }

    pub fn tab_prev_screen(&mut self) {
        match self.screen {
            Screen::Timer => self.navigate_to(Screen::Entries),
            Screen::Projects => self.navigate_to(Screen::Timer),
            Screen::Entries => self.navigate_to(Screen::Projects),
        }
    }

    pub fn navigate_to(&mut self, screen: Screen) {
        self.screen = screen;
        self.menu_selected = MENU_ITEMS
            .iter()
            .position(|(_, s)| *s == screen)
            .unwrap_or(0);
        match screen {
            Screen::Timer => self.needs_timer_refresh = true,
            Screen::Projects => {}
            Screen::Entries => {
                if self.entries.is_empty() {
                    self.needs_entries_load = true;
                }
            }
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.error_message = None;
        self.status_ttl = 30;
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.status_ttl = 0;
    }

    pub fn tick(&mut self) {
        if self.status_ttl > 0 {
            self.status_ttl -= 1;
            if self.status_ttl == 0 {
                self.status_message = None;
            }
        }
    }

    pub fn project_name(&self, project_id: i64) -> String {
        self.projects
            .iter()
            .find(|p| p.id == project_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("#{}", project_id))
    }

    pub fn task_name(&self, task_id: i64, entry_task_name: Option<&str>) -> String {
        if let Some(name) = entry_task_name {
            if !name.is_empty() {
                return name.to_string();
            }
        }
        self.task_names
            .get(&task_id)
            .cloned()
            .unwrap_or_else(|| format!("#{}", task_id))
    }

    pub fn cache_tasks(&mut self, tasks: &[Task]) {
        for t in tasks {
            self.task_names.insert(t.id, t.name.clone());
        }
    }
}
