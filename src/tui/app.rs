//! Pure, terminal-independent TUI state and key handling.
//!
//! Deliberately has no `ratatui` types and does no file I/O — `handle_key`
//! takes a `crossterm::event::KeyEvent` (plain data) and returns an
//! `AppEvent` for the impure event loop in `tui::mod` to act on (saving to
//! the store, quitting). That split is what makes this unit-testable
//! without a real terminal, mirroring how `store.rs` injects a directory
//! instead of touching the real filesystem in tests.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::haiku::Haiku;
use crate::line_check::{check_line, LineCheck};

const TARGETS: [u32; 3] = [5, 7, 5];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Browse,
    Compose,
}

pub struct App {
    pub screen: Screen,
    pub browse: BrowseState,
    pub compose: ComposeState,
}

pub struct BrowseState {
    pub all: Vec<Haiku>,
    pub query: String,
    pub matches: Vec<usize>,
    pub selected: usize,
}

#[derive(Default, Clone)]
pub struct LineBuffer {
    pub text: String,
    pub cursor: usize, // char index, not byte index
}

pub struct ComposeState {
    pub lines: [LineBuffer; 3],
    pub active: usize,
    pub checks: [LineCheck; 3],
    pub status: Option<ComposeStatus>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ComposeStatus {
    Saved,
    Error(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppEvent {
    Save(Haiku),
    Quit,
}

impl App {
    pub fn new(all: Vec<Haiku>) -> Self {
        let matches = (0..all.len()).collect();
        App {
            screen: Screen::Browse,
            browse: BrowseState {
                all,
                query: String::new(),
                matches,
                selected: 0,
            },
            compose: ComposeState::new(),
        }
    }

    /// After a successful save: fold the new haiku into the browse list
    /// (so it's visible without restarting), clear the compose draft, and
    /// leave a transient "saved" status for the compose screen to show.
    pub fn record_saved_haiku(&mut self, haiku: Haiku) {
        self.browse.all.push(haiku);
        self.browse.recompute_matches();
        self.compose = ComposeState::new();
        self.compose.status = Some(ComposeStatus::Saved);
    }
}

impl BrowseState {
    fn recompute_matches(&mut self) {
        let query = self.query.to_lowercase();
        self.matches = self
            .all
            .iter()
            .enumerate()
            .filter(|(_, haiku)| {
                query.is_empty()
                    || haiku
                        .lines
                        .iter()
                        .any(|line| line.to_lowercase().contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected >= self.matches.len() {
            self.selected = self.matches.len().saturating_sub(1);
        }
    }

    fn push_query_char(&mut self, c: char) {
        self.query.push(c);
        self.recompute_matches();
        self.selected = 0;
    }

    fn pop_query_char(&mut self) {
        self.query.pop();
        self.recompute_matches();
        self.selected = 0;
    }

    fn clear_query(&mut self) {
        self.query.clear();
        self.recompute_matches();
        self.selected = 0;
    }

    fn move_selection(&mut self, delta: isize) {
        if self.matches.is_empty() {
            return;
        }
        let max = (self.matches.len() - 1) as isize;
        self.selected = (self.selected as isize + delta).clamp(0, max) as usize;
    }

    fn move_to_start(&mut self) {
        self.selected = 0;
    }

    fn move_to_end(&mut self) {
        self.selected = self.matches.len().saturating_sub(1);
    }

    pub fn selected_haiku(&self) -> Option<&Haiku> {
        self.matches.get(self.selected).map(|&i| &self.all[i])
    }
}

impl LineBuffer {
    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    fn byte_index(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn insert_char(&mut self, c: char) {
        let idx = self.byte_index(self.cursor);
        self.text.insert(idx, c);
        self.cursor += 1;
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let idx = self.byte_index(self.cursor - 1);
        self.text.remove(idx);
        self.cursor -= 1;
    }

    fn delete(&mut self) {
        if self.cursor >= self.char_count() {
            return;
        }
        let idx = self.byte_index(self.cursor);
        self.text.remove(idx);
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.char_count());
    }

    fn move_home(&mut self) {
        self.cursor = 0;
    }

    fn move_end(&mut self) {
        self.cursor = self.char_count();
    }
}

impl ComposeState {
    fn new() -> Self {
        ComposeState {
            lines: Default::default(),
            active: 0,
            checks: [
                check_line("", TARGETS[0]),
                check_line("", TARGETS[1]),
                check_line("", TARGETS[2]),
            ],
            status: None,
        }
    }

    fn recompute_active_check(&mut self) {
        self.checks[self.active] = check_line(&self.lines[self.active].text, TARGETS[self.active]);
        self.status = None;
    }

    fn advance_if_valid(&mut self) {
        if self.active < 2 && self.checks[self.active].is_ok() {
            self.active += 1;
        }
    }

    fn go_back(&mut self) {
        self.active = self.active.saturating_sub(1);
    }

    fn try_build_haiku(&self) -> Option<Haiku> {
        if self.checks.iter().all(LineCheck::is_ok) {
            Haiku::new(
                self.lines[0].text.clone(),
                self.lines[1].text.clone(),
                self.lines[2].text.clone(),
            )
            .ok()
        } else {
            None
        }
    }
}

/// Handle one key event, mutating `app` in place. Returns `Some` only for
/// the two effects the impure event loop must act on (saving, quitting) —
/// everything else is a pure state change.
pub fn handle_key(app: &mut App, key: KeyEvent) -> Option<AppEvent> {
    // crossterm can report Release/Repeat kinds on some platforms; only
    // react to the initial press, or a held char key would double-insert.
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(AppEvent::Quit);
    }

    if key.code == KeyCode::Tab {
        app.screen = match app.screen {
            Screen::Browse => Screen::Compose,
            Screen::Compose => Screen::Browse,
        };
        return None;
    }

    match app.screen {
        Screen::Browse => handle_browse_key(app, key),
        Screen::Compose => handle_compose_key(app, key),
    }
}

fn handle_browse_key(app: &mut App, key: KeyEvent) -> Option<AppEvent> {
    let browse = &mut app.browse;
    match key.code {
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            browse.push_query_char(c);
        }
        KeyCode::Backspace => browse.pop_query_char(),
        KeyCode::Up => browse.move_selection(-1),
        KeyCode::Down => browse.move_selection(1),
        KeyCode::PageUp => browse.move_selection(-10),
        KeyCode::PageDown => browse.move_selection(10),
        KeyCode::Home => browse.move_to_start(),
        KeyCode::End => browse.move_to_end(),
        KeyCode::Esc => {
            if browse.query.is_empty() {
                return Some(AppEvent::Quit);
            }
            browse.clear_query();
        }
        _ => {}
    }
    None
}

fn handle_compose_key(app: &mut App, key: KeyEvent) -> Option<AppEvent> {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return app.compose.try_build_haiku().map(AppEvent::Save);
    }

    let compose = &mut app.compose;
    match key.code {
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            compose.lines[compose.active].insert_char(c);
            compose.recompute_active_check();
        }
        KeyCode::Backspace => {
            compose.lines[compose.active].backspace();
            compose.recompute_active_check();
        }
        KeyCode::Delete => {
            compose.lines[compose.active].delete();
            compose.recompute_active_check();
        }
        KeyCode::Left => compose.lines[compose.active].move_left(),
        KeyCode::Right => compose.lines[compose.active].move_right(),
        KeyCode::Home => compose.lines[compose.active].move_home(),
        KeyCode::End => compose.lines[compose.active].move_end(),
        KeyCode::BackTab | KeyCode::Up => compose.go_back(),
        KeyCode::Enter => {
            if compose.active == 2 {
                if let Some(haiku) = compose.try_build_haiku() {
                    return Some(AppEvent::Save(haiku));
                }
            } else {
                compose.advance_if_valid();
            }
        }
        KeyCode::Esc => {
            app.screen = Screen::Browse;
        }
        _ => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn char_key(c: char) -> KeyEvent {
        key(KeyCode::Char(c))
    }

    fn haiku(a: &str, b: &str, c: &str) -> Haiku {
        Haiku::new(a, b, c).unwrap()
    }

    fn sample_haikus() -> Vec<Haiku> {
        vec![
            haiku(
                "an old silent pond",
                "a frog jumps into the pond",
                "splash silence again",
            ),
            haiku(
                "i walked to the store",
                "wanted a little table",
                "whole apple loved much",
            ),
        ]
    }

    fn type_str(app: &mut App, s: &str) {
        for c in s.chars() {
            handle_key(app, char_key(c));
        }
    }

    // -- Browse --

    #[test]
    fn typing_narrows_matches_to_substring_hits() {
        let mut app = App::new(sample_haikus());
        type_str(&mut app, "table");
        assert_eq!(app.browse.matches, vec![1]);
    }

    #[test]
    fn search_is_case_insensitive() {
        let mut app = App::new(sample_haikus());
        type_str(&mut app, "POND");
        assert_eq!(app.browse.matches, vec![0]);
    }

    #[test]
    fn backspace_widens_matches_back() {
        let mut app = App::new(sample_haikus());
        type_str(&mut app, "table");
        handle_key(&mut app, key(KeyCode::Backspace));
        handle_key(&mut app, key(KeyCode::Backspace));
        handle_key(&mut app, key(KeyCode::Backspace));
        handle_key(&mut app, key(KeyCode::Backspace));
        handle_key(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.browse.matches, vec![0, 1]);
        assert!(app.browse.query.is_empty());
    }

    #[test]
    fn navigation_does_not_underflow_at_top() {
        let mut app = App::new(sample_haikus());
        handle_key(&mut app, key(KeyCode::Up));
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.browse.selected, 0);
    }

    #[test]
    fn navigation_does_not_overflow_at_bottom() {
        let mut app = App::new(sample_haikus());
        handle_key(&mut app, key(KeyCode::Down));
        handle_key(&mut app, key(KeyCode::Down));
        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.browse.selected, 1);
    }

    #[test]
    fn navigation_on_empty_matches_is_a_noop() {
        let mut app = App::new(sample_haikus());
        type_str(&mut app, "nonexistentword");
        assert!(app.browse.matches.is_empty());
        handle_key(&mut app, key(KeyCode::Down)); // must not panic
        assert_eq!(app.browse.selected, 0);
    }

    #[test]
    fn esc_clears_nonempty_query_before_quitting() {
        let mut app = App::new(sample_haikus());
        type_str(&mut app, "pond");
        let event = handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(event, None);
        assert!(app.browse.query.is_empty());
    }

    #[test]
    fn esc_on_empty_query_quits() {
        let mut app = App::new(sample_haikus());
        let event = handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(event, Some(AppEvent::Quit));
    }

    #[test]
    fn tab_switches_to_compose_and_back_preserves_browse_state() {
        let mut app = App::new(sample_haikus());
        type_str(&mut app, "table");
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.screen, Screen::Compose);
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.screen, Screen::Browse);
        assert_eq!(app.browse.query, "table");
        assert_eq!(app.browse.matches, vec![1]);
    }

    // -- Compose --

    fn goto_compose(app: &mut App) {
        handle_key(app, key(KeyCode::Tab));
    }

    #[test]
    fn valid_line_advances_to_next_line() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.compose.active, 1);
    }

    #[test]
    fn invalid_line_does_not_advance() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "too short");
        handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(app.compose.active, 0);
    }

    #[test]
    fn backspace_edits_active_line_and_recomputes_check() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        // "z" is an extra one-syllable word, bringing the line to 6.
        type_str(&mut app, "an old silent pond z");
        assert!(!app.compose.checks[0].is_ok());
        handle_key(&mut app, key(KeyCode::Backspace)); // remove "z"
        handle_key(&mut app, key(KeyCode::Backspace)); // remove the space
        assert!(app.compose.checks[0].is_ok());
    }

    #[test]
    fn cursor_movement_stays_in_bounds() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        handle_key(&mut app, key(KeyCode::Left)); // at 0, must not panic/underflow
        assert_eq!(app.compose.lines[0].cursor, 0);
        type_str(&mut app, "hi");
        handle_key(&mut app, key(KeyCode::Right));
        handle_key(&mut app, key(KeyCode::Right)); // past end, must clamp
        assert_eq!(app.compose.lines[0].cursor, 2);
    }

    #[test]
    fn up_moves_to_previous_line_even_if_current_is_invalid() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        handle_key(&mut app, key(KeyCode::Enter)); // advance to line 2 (valid)
        type_str(&mut app, "nope"); // line 2 now invalid
        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.compose.active, 0);
    }

    #[test]
    fn completing_all_three_valid_lines_then_enter_returns_save_event() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        handle_key(&mut app, key(KeyCode::Enter));
        type_str(&mut app, "a frog jumps into the pond");
        handle_key(&mut app, key(KeyCode::Enter));
        type_str(&mut app, "splash silence again");
        let event = handle_key(&mut app, key(KeyCode::Enter));
        assert_eq!(
            event,
            Some(AppEvent::Save(haiku(
                "an old silent pond",
                "a frog jumps into the pond",
                "splash silence again"
            )))
        );
    }

    #[test]
    fn ctrl_s_saves_when_all_lines_valid() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        handle_key(&mut app, key(KeyCode::Enter));
        type_str(&mut app, "a frog jumps into the pond");
        handle_key(&mut app, key(KeyCode::Enter));
        type_str(&mut app, "splash silence again");
        let event = handle_key(&mut app, ctrl_key(KeyCode::Char('s')));
        assert!(matches!(event, Some(AppEvent::Save(_))));
    }

    #[test]
    fn ctrl_s_is_a_noop_when_lines_incomplete() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        let event = handle_key(&mut app, ctrl_key(KeyCode::Char('s')));
        assert_eq!(event, None);
    }

    #[test]
    fn esc_returns_to_browse_without_discarding_draft() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Browse);
        assert_eq!(app.compose.lines[0].text, "an old silent pond");
    }

    #[test]
    fn record_saved_haiku_clears_draft_and_updates_browse() {
        let mut app = App::new(Vec::new());
        goto_compose(&mut app);
        type_str(&mut app, "an old silent pond");
        let saved = haiku(
            "an old silent pond",
            "a frog jumps into the pond",
            "splash silence again",
        );
        app.record_saved_haiku(saved.clone());
        assert_eq!(app.compose.lines[0].text, "");
        assert_eq!(app.compose.status, Some(ComposeStatus::Saved));
        assert!(app.browse.all.contains(&saved));
        assert!(app.browse.matches.contains(&0));
    }

    // -- Global --

    #[test]
    fn ctrl_c_quits_from_either_screen() {
        let mut app = App::new(sample_haikus());
        assert_eq!(
            handle_key(&mut app, ctrl_key(KeyCode::Char('c'))),
            Some(AppEvent::Quit)
        );

        let mut app = App::new(sample_haikus());
        goto_compose(&mut app);
        assert_eq!(
            handle_key(&mut app, ctrl_key(KeyCode::Char('c'))),
            Some(AppEvent::Quit)
        );
    }
}
