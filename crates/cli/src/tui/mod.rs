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
use futures::StreamExt;
use hypersdk::hypercore::{
    types::{Incoming, Subscription},
    ws::Event as WsEvent,
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
    // ── Connect WebSocket for live price streaming ──────────────
    let config = atlas_core::workspace::load_config().unwrap_or_default();
    let core = if config.network.testnet {
        hypersdk::hypercore::testnet()
    } else {
        hypersdk::hypercore::mainnet()
    };
    let mut ws = core.websocket();
    ws.subscribe(Subscription::AllMids { dex: None });

    loop {
        // ── Draw ────────────────────────────────────────────────
        terminal.draw(|frame| ui::render(frame, app))?;

        // ── Select: terminal events vs WebSocket messages ───────
        tokio::select! {
            // Poll terminal events (with timeout for ticking)
            term_event = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(200)).unwrap_or(false) {
                    event::read().ok()
                } else {
                    None
                }
            }) => {
                if let Ok(Some(Event::Key(key))) = term_event {
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
                        Action::CancelOrder => app.cancel_selected_order().await,
                    }
                }

                // ── Auto-refresh (REST) ─────────────────────────
                app.tick();
                if app.should_refresh() {
                    app.refresh().await;
                }
            }

            // Poll WebSocket messages for live price updates
            ws_event = ws.next() => {
                match ws_event {
                    Some(WsEvent::Message(Incoming::AllMids { dex: _, mids })) => {
                        app.on_ws_mids(mids);
                    }
                    Some(WsEvent::Connected) => {
                        app.on_ws_connected();
                    }
                    Some(WsEvent::Disconnected) => {
                        app.on_ws_disconnected();
                    }
                    Some(_) => {}
                    None => {
                        // WebSocket stream ended — reconnect
                        app.on_ws_disconnected();
                        ws = core.websocket();
                        ws.subscribe(Subscription::AllMids { dex: None });
                    }
                }
            }
        }
    }
}
