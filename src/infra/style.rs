//! Terminal styling that follows the active Omarchy theme.
//!
//! Colors are loaded from `~/.local/state/omarchy/current/theme/colors.toml`
//! on each run, so `omarchy theme set …` is picked up immediately. When that
//! file is missing (non-Omarchy systems), we fall back to ANSI named colors,
//! which still track the terminal palette.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use owo_colors::{OwoColorize, Rgb, Stream};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Theme {
    pub accent: Rgb,
    pub muted: Rgb,
    pub foreground: Rgb,
    pub red: Rgb,
    pub orange: Rgb,
    pub yellow: Rgb,
    pub green: Rgb,
    pub cyan: Rgb,
}

impl Default for Theme {
    /// ANSI-ish defaults used when no Omarchy theme file is present.
    fn default() -> Self {
        Self {
            accent: Rgb(0x7a, 0xa2, 0xf7),
            muted: Rgb(0x6c, 0x70, 0x86),
            foreground: Rgb(0xc0, 0xca, 0xf5),
            red: Rgb(0xf7, 0x76, 0x8e),
            orange: Rgb(0xff, 0x9e, 0x64),
            yellow: Rgb(0xe0, 0xaf, 0x68),
            green: Rgb(0x9e, 0xce, 0x6a),
            cyan: Rgb(0x7d, 0xcf, 0xff),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ColorsToml {
    accent: Option<String>,
    muted: Option<String>,
    foreground: Option<String>,
    red: Option<String>,
    orange: Option<String>,
    yellow: Option<String>,
    green: Option<String>,
    cyan: Option<String>,
}

fn parse_hex(s: &str) -> Option<Rgb> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Rgb(r, g, b))
}

fn omarchy_colors_path() -> Option<PathBuf> {
    let mut path = PathBuf::from(env::var_os("HOME")?);
    path.push(".local/state/omarchy/current/theme/colors.toml");
    Some(path)
}

fn load_omarchy_theme() -> Option<Theme> {
    let path = omarchy_colors_path()?;
    let raw = fs::read_to_string(path).ok()?;
    let parsed: ColorsToml = toml::from_str(&raw).ok()?;
    let mut theme = Theme::default();
    if let Some(c) = parsed.accent.as_deref().and_then(parse_hex) {
        theme.accent = c;
    }
    if let Some(c) = parsed.muted.as_deref().and_then(parse_hex) {
        theme.muted = c;
    }
    if let Some(c) = parsed.foreground.as_deref().and_then(parse_hex) {
        theme.foreground = c;
    }
    if let Some(c) = parsed.red.as_deref().and_then(parse_hex) {
        theme.red = c;
    }
    if let Some(c) = parsed.orange.as_deref().and_then(parse_hex) {
        theme.orange = c;
    }
    if let Some(c) = parsed.yellow.as_deref().and_then(parse_hex) {
        theme.yellow = c;
    }
    if let Some(c) = parsed.green.as_deref().and_then(parse_hex) {
        theme.green = c;
    }
    if let Some(c) = parsed.cyan.as_deref().and_then(parse_hex) {
        theme.cyan = c;
    }
    Some(theme)
}

pub fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| load_omarchy_theme().unwrap_or_default())
}

fn paint(text: &str, color: Rgb) -> String {
    text.if_supports_color(Stream::Stdout, |t| t.color(color))
        .to_string()
}

pub fn accent(text: &str) -> String {
    paint(text, theme().accent)
}

pub fn muted(text: &str) -> String {
    paint(text, theme().muted)
}

pub fn fg(text: &str) -> String {
    paint(text, theme().foreground)
}

pub fn success(text: &str) -> String {
    paint(text, theme().green)
}

pub fn warn(text: &str) -> String {
    paint(text, theme().orange)
}

pub fn error(text: &str) -> String {
    paint(text, theme().red)
}

pub fn cyan(text: &str) -> String {
    paint(text, theme().cyan)
}

pub const OK: &str = "✓";
pub const BAD: &str = "✗";
pub const PROMPT: &str = "›";
pub const DOT: &str = "·";
pub const FLOWER: &str = "❀";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_accepts_hash_prefix() {
        assert_eq!(parse_hex("#509475"), Some(Rgb(0x50, 0x94, 0x75)));
        assert_eq!(parse_hex("FF5345"), Some(Rgb(0xFF, 0x53, 0x45)));
    }

    #[test]
    fn parse_hex_rejects_junk() {
        assert_eq!(parse_hex("zzz"), None);
        assert_eq!(parse_hex("#fff"), None);
    }
}
