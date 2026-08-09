use ratatui::style::{Color, Modifier, Style};

pub const PRIMARY: Color = Color::Cyan;
pub const ACCENT: Color = Color::Yellow;
pub const SUCCESS: Color = Color::Green;
pub const ERROR: Color = Color::Red;
pub const PAUSED: Color = Color::Magenta;
pub const MUTED: Color = Color::DarkGray;
pub const SECONDARY: Color = Color::White;
pub fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

pub fn selected() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn status_running() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn status_paused() -> Style {
    Style::default().fg(PAUSED).add_modifier(Modifier::BOLD)
}
