//! Adapts `crate::style`'s Omarchy-aware theme colors into `ratatui` styles,
//! so the TUI matches the CLI's existing palette (`cmd_list`, `cmd_new`,
//! etc.) instead of hand-rolling its own.

use ratatui::style::{Color, Style};

use crate::style;

fn to_color(rgb: owo_colors::Rgb) -> Color {
    Color::Rgb(rgb.0, rgb.1, rgb.2)
}

pub fn accent() -> Style {
    Style::default().fg(to_color(style::theme().accent))
}

pub fn muted() -> Style {
    Style::default().fg(to_color(style::theme().muted))
}

pub fn fg() -> Style {
    Style::default().fg(to_color(style::theme().foreground))
}

pub fn success() -> Style {
    Style::default().fg(to_color(style::theme().green))
}

pub fn error() -> Style {
    Style::default().fg(to_color(style::theme().red))
}

pub fn cyan() -> Style {
    Style::default().fg(to_color(style::theme().cyan))
}
