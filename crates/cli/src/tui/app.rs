use crossterm::event::{KeyCode, KeyModifiers};

use super::state::App;

/// Actions the TUI can perform in response to input.
pub enum Action {
    None,
    Quit,
    Refresh,
    Tab(usize),
    NextTab,
    PrevTab,
    ScrollUp,
    ScrollDown,
    ToggleHelp,
}

/// Map a key press to an Action.
pub fn handle_key(app: &App, code: KeyCode, modifiers: KeyModifiers) -> Action {
    // Help overlay captures all keys except close
    if app.show_help {
        return match code {
            KeyCode::Char('?') | KeyCode::Esc => Action::ToggleHelp,
            _ => Action::None,
        };
    }

    match code {
        // ── Quit ────────────────────────────────────────────
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

        // ── Refresh ─────────────────────────────────────────
        KeyCode::Char('r') => Action::Refresh,

        // ── Tab switching (number keys) ─────────────────────
        KeyCode::Char('1') => Action::Tab(0),
        KeyCode::Char('2') => Action::Tab(1),
        KeyCode::Char('3') => Action::Tab(2),
        KeyCode::Char('4') => Action::Tab(3),

        // ── Tab cycling ─────────────────────────────────────
        KeyCode::Tab => Action::NextTab,
        KeyCode::BackTab => Action::PrevTab, // Shift+Tab
        KeyCode::Char('l') | KeyCode::Right => Action::NextTab,
        KeyCode::Char('h') | KeyCode::Left => Action::PrevTab,

        // ── Scroll ──────────────────────────────────────────
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,

        // ── Help ────────────────────────────────────────────
        KeyCode::Char('?') => Action::ToggleHelp,

        _ => Action::None,
    }
}
