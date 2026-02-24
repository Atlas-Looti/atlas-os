use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

use super::state::{App, SwapFocus, TradeFocus};

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
    CancelOrder,
    ToggleTrade,
    ToggleSwap,
    SubmitTrade,
    SubmitSwap,
}

/// Map a cross-term event to an Action.
pub fn handle_event(app: &mut App, event: Event) -> Action {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return Action::None;
        }

        // ── Active Trade Popup ──────────────────────────────────────
        if app.trade_popup.visible {
            match key.code {
                KeyCode::Esc => {
                    app.trade_popup.visible = false;
                    return Action::None;
                }
                KeyCode::Tab => {
                    app.trade_popup.focus = match app.trade_popup.focus {
                        TradeFocus::Coin => TradeFocus::Size,
                        TradeFocus::Size => TradeFocus::Price,
                        TradeFocus::Price => TradeFocus::Side,
                        TradeFocus::Side => TradeFocus::Coin,
                    };
                    return Action::None;
                }
                KeyCode::BackTab => {
                    app.trade_popup.focus = match app.trade_popup.focus {
                        TradeFocus::Coin => TradeFocus::Side,
                        TradeFocus::Size => TradeFocus::Coin,
                        TradeFocus::Price => TradeFocus::Size,
                        TradeFocus::Side => TradeFocus::Price,
                    };
                    return Action::None;
                }
                KeyCode::Enter => {
                    return Action::SubmitTrade;
                }
                _ => {
                    match app.trade_popup.focus {
                        TradeFocus::Coin => app.trade_popup.coin.handle_event(&event),
                        TradeFocus::Size => app.trade_popup.size.handle_event(&event),
                        TradeFocus::Price => app.trade_popup.price.handle_event(&event),
                        TradeFocus::Side => app.trade_popup.side.handle_event(&event),
                    };
                    return Action::None;
                }
            }
        }

        // ── Active Swap Popup ───────────────────────────────────────
        if app.swap_popup.visible {
            match key.code {
                KeyCode::Esc => {
                    app.swap_popup.visible = false;
                    return Action::None;
                }
                KeyCode::Tab => {
                    app.swap_popup.focus = match app.swap_popup.focus {
                        SwapFocus::SellToken => SwapFocus::BuyToken,
                        SwapFocus::BuyToken => SwapFocus::SellAmount,
                        SwapFocus::SellAmount => SwapFocus::SellToken,
                    };
                    return Action::None;
                }
                KeyCode::BackTab => {
                    app.swap_popup.focus = match app.swap_popup.focus {
                        SwapFocus::SellToken => SwapFocus::SellAmount,
                        SwapFocus::BuyToken => SwapFocus::SellToken,
                        SwapFocus::SellAmount => SwapFocus::BuyToken,
                    };
                    return Action::None;
                }
                KeyCode::Enter => {
                    return Action::SubmitSwap;
                }
                _ => {
                    match app.swap_popup.focus {
                        SwapFocus::SellToken => app.swap_popup.sell_token.handle_event(&event),
                        SwapFocus::BuyToken => app.swap_popup.buy_token.handle_event(&event),
                        SwapFocus::SellAmount => app.swap_popup.sell_amount.handle_event(&event),
                    };
                    return Action::None;
                }
            }
        }

        // ── Help overlay ────────────────────────────────────────────
        if app.show_help {
            return match key.code {
                KeyCode::Char('?') | KeyCode::Esc => Action::ToggleHelp,
                _ => Action::None,
            };
        }

        match key.code {
            // ── Quit ────────────────────────────────────────────
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

            // ── Refresh ─────────────────────────────────────────
            KeyCode::Char('r') => Action::Refresh,

            // ── Cancel order (on Orders tab) ────────────────────
            KeyCode::Char('c') => {
                if app.tab == 2 {
                    Action::CancelOrder
                } else {
                    Action::None
                }
            }

            // ── Popups ──────────────────────────────────────────
            KeyCode::Char('t') => Action::ToggleTrade,
            KeyCode::Char('s') => Action::ToggleSwap,

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

            // ── Scroll / Selection ──────────────────────────────
            KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
            KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,

            // ── Help ────────────────────────────────────────────
            KeyCode::Char('?') => Action::ToggleHelp,

            _ => Action::None,
        }
    } else {
        Action::None
    }
}
