//! Full-screen TUI entry point: terminal lifecycle + event loop. This is the
//! only file that touches `crossterm::terminal`/`ratatui::Terminal` directly
//! and the only one that calls `store::list`/`store::save` — everything else
//! in `tui/` is either pure state (`app`) or pure rendering (`ui`).

use crossterm::event::{self, Event};

use crate::infra::store;

mod app;
mod theme;
mod ui;

use app::{App, AppEvent, ComposeStatus};

/// Restores the terminal (raw mode off, leave the alternate screen) when
/// dropped — covers normal return and early `?`-returns. `ratatui::try_init`
/// below also installs its own panic hook that restores on panic, so a crash
/// mid-session doesn't leave the terminal broken either.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Load before entering the terminal guard, so a StoreError (e.g.
    // corrupt JSON) prints normally instead of inside an alternate screen.
    let all = store::list()?;
    let mut app = App::new(all);

    let mut terminal = ratatui::try_init()?;
    let _guard = TerminalGuard;

    event_loop(&mut terminal, &mut app)
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Event::Key(key) = event::read()? {
            match app::handle_key(app, key) {
                Some(AppEvent::Quit) => return Ok(()),
                Some(AppEvent::Save(haiku)) => match store::save(haiku.clone()) {
                    Ok(()) => app.record_saved_haiku(haiku),
                    Err(err) => app.compose.status = Some(ComposeStatus::Error(err.to_string())),
                },
                None => {}
            }
        }
    }
}
