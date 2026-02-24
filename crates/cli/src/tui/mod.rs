mod app;
mod state;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::Action;
use state::App;

/// `atlas tui` — launch the interactive terminal interface.
pub async fn run() -> Result<()> {
    // ── Setup terminal ──────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // ── Initialize app state ────────────────────────────────────
    let mut app = App::new().await;

    // ── Main loop ───────────────────────────────────────────────
    let result = run_loop(&mut terminal, &mut app).await;

    // ── Teardown (always runs) ──────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // ── Draw ────────────────────────────────────────────────
        terminal.draw(|frame| ui::render(frame, app))?;

        // ── Input ───────────────────────────────────────────────
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let action = app::handle_key(app, key.code, key.modifiers);
                match action {
                    Action::None => {}
                    Action::Quit => return Ok(()),
                    Action::Refresh => app.refresh().await,
                    Action::Tab(idx) => app.set_tab(idx),
                    Action::NextTab => app.next_tab(),
                    Action::PrevTab => app.prev_tab(),
                    Action::ScrollUp => app.scroll_up(),
                    Action::ScrollDown => app.scroll_down(),
                    Action::ToggleHelp => app.toggle_help(),
                }
            }
        }

        // ── Auto-refresh ─────────────────────────────────────────
        app.tick();
        if app.should_refresh() {
            app.refresh().await;
        }
    }
}
