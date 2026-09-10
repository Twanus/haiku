//! Rendering only — reads `App`, never mutates it.

use ratatui::layout::{Constraint, Layout};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use super::app::{App, BrowseState, ComposeState, ComposeStatus, Screen};
use super::theme;
use crate::style;

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Browse => draw_browse(frame, &app.browse),
        Screen::Compose => draw_compose(frame, &app.compose),
    }
}

fn draw_browse(frame: &mut Frame, browse: &BrowseState) {
    let area = frame.area();
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    let search_text = format!("{} {}", style::PROMPT, browse.query);
    let search = Paragraph::new(Span::styled(search_text, theme::accent()))
        .block(Block::bordered().title(" search "));
    frame.render_widget(search, rows[0]);
    let cursor_x = rows[0].x + 1 + 2 + browse.query.chars().count() as u16;
    frame.set_cursor_position((cursor_x, rows[0].y + 1));

    let cols = Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(rows[1]);

    // `matches` can hold every entry in the store (177k+) when the search is
    // empty. ratatui's `List` only *paints* the visible rows, but building a
    // `ListItem` (with its `format!` allocation) is O(n) regardless of how
    // many are ever shown — with a large enough store that cost alone made
    // every redraw (so every arrow-key press) noticeably slow. Only format
    // a window around the selection, sized to what the list area can
    // actually show, so the per-frame cost stays O(visible rows) no matter
    // how large `matches` gets.
    let list_area = cols[0];
    let visible_height = list_area.height.saturating_sub(2) as usize; // minus top/bottom border
    let total = browse.matches.len();
    let start = if total <= visible_height {
        0
    } else {
        browse
            .selected
            .saturating_sub(visible_height / 2)
            .min(total - visible_height)
    };
    let end = (start + visible_height).min(total);

    let items: Vec<ListItem> = browse.matches[start..end]
        .iter()
        .map(|&i| {
            let haiku = &browse.all[i];
            ListItem::new(format!(
                "{}  /  {}  /  {}",
                haiku.lines[0], haiku.lines[1], haiku.lines[2]
            ))
        })
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title(format!(
            " browse ({}/{}) ",
            browse.matches.len(),
            browse.all.len()
        )))
        .highlight_style(theme::accent().add_modifier(Modifier::REVERSED));
    let mut list_state = ListState::default();
    if end > start {
        // `items` is already just the visible window, so the selection
        // index needs to be relative to `start`, not to the full list.
        list_state.select(Some(browse.selected - start));
    }
    frame.render_stateful_widget(list, list_area, &mut list_state);

    let preview = match browse.selected_haiku() {
        Some(haiku) => {
            let [a, b, c] = haiku.syllable_counts();
            Text::from(vec![
                Line::from(Span::styled(haiku.lines[0].clone(), theme::muted())),
                Line::from(vec![
                    Span::styled(format!("{} ", style::DOT), theme::accent()),
                    Span::styled(haiku.lines[1].clone(), theme::fg()),
                ]),
                Line::from(Span::styled(haiku.lines[2].clone(), theme::muted())),
                Line::raw(""),
                Line::from(Span::styled(format!("{a}-{b}-{c}"), theme::cyan())),
            ])
        }
        None => Text::from(Span::styled(
            format!("{} no matches", style::FLOWER),
            theme::muted(),
        )),
    };
    frame.render_widget(
        Paragraph::new(preview).block(Block::bordered().title(" preview ")),
        cols[1],
    );

    let help = Line::from(Span::styled(
        "type to search  ·  ↑↓ move  ·  Tab compose  ·  Esc back/quit  ·  Ctrl+C quit",
        theme::muted(),
    ));
    frame.render_widget(Paragraph::new(help), rows[2]);
}

fn draw_compose(frame: &mut Frame, compose: &ComposeState) {
    let area = frame.area();
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    let hint = Line::from(Span::styled(
        format!("{} compose a haiku — 5, 7, 5 syllables", style::FLOWER),
        theme::muted(),
    ));
    frame.render_widget(Paragraph::new(hint), rows[0]);

    for i in 0..3 {
        let check = &compose.checks[i];
        let is_active = compose.active == i;
        let border_style = if is_active {
            theme::accent()
        } else if check.is_ok() {
            theme::success()
        } else if check.breakdown.is_empty() {
            theme::muted() // untouched, not yet attempted
        } else {
            theme::error() // attempted but still wrong
        };
        let title = format!(" {}{} {}/{} ", i + 1, style::PROMPT, check.count, check.target);
        let block = Block::bordered().title(title).border_style(border_style);

        let mut lines = vec![Line::from(Span::styled(
            compose.lines[i].text.clone(),
            theme::fg(),
        ))];
        if is_active && !check.breakdown.is_empty() {
            lines.push(Line::from(Span::styled(
                check.breakdown.clone(),
                theme::muted(),
            )));
        }
        let line_area = rows[i + 1];
        frame.render_widget(Paragraph::new(lines).block(block), line_area);

        if is_active {
            let cursor_x = line_area.x + 1 + compose.lines[i].cursor as u16;
            frame.set_cursor_position((cursor_x, line_area.y + 1));
        }
    }

    let status_line = match &compose.status {
        Some(ComposeStatus::Saved) => Line::from(Span::styled(
            format!("{} saved", style::OK),
            theme::success(),
        )),
        Some(ComposeStatus::Error(message)) => {
            Line::from(Span::styled(format!("{} {message}", style::BAD), theme::error()))
        }
        None => Line::from(Span::styled(
            "Enter advance/save  ·  Ctrl+S save  ·  Esc back  ·  Tab browse  ·  Ctrl+C quit",
            theme::muted(),
        )),
    };
    frame.render_widget(Paragraph::new(status_line), rows[5]);
}
