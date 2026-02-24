use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, Tabs},
    Frame,
};

use atlas_utils::fmt::{self, Sign};

use super::state::App;

// ─── Color palette ──────────────────────────────────────────────────

const ACCENT: Color = Color::Cyan;
const GREEN: Color = Color::Green;
const RED: Color = Color::Red;
const DIM: Color = Color::DarkGray;
const YELLOW: Color = Color::Yellow;
const WHITE: Color = Color::White;
const BG_HEADER: Color = Color::Rgb(20, 20, 40);

// ─── Main render ────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Root layout: header(3) + tabs(3) + body(flex) + status(1)
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(8),   // Body
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    render_header(frame, app, root[0]);
    render_tabs(frame, app, root[1]);

    match app.tab {
        0 => render_dashboard(frame, app, root[2]),
        1 => render_positions(frame, app, root[2]),
        2 => render_orders(frame, app, root[2]),
        3 => render_markets(frame, app, root[2]),
        _ => {}
    }

    render_status_bar(frame, app, root[3]);

    // Help overlay on top
    if app.show_help {
        render_help(frame, area);
    }
}

// ─── Header ─────────────────────────────────────────────────────────

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(DIM))
        .style(Style::default().bg(BG_HEADER));

    let conn_indicator = if app.connected {
        Span::styled(" ● ", Style::default().fg(GREEN).bold())
    } else {
        Span::styled(" ● ", Style::default().fg(RED).bold())
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" ATLAS ", Style::default().fg(Color::Black).bg(ACCENT).bold()),
        Span::raw("  "),
        conn_indicator,
        Span::styled(&app.network, Style::default().fg(DIM)),
        Span::raw("  │  "),
        Span::styled(&app.profile_name, Style::default().fg(YELLOW).bold()),
        Span::raw("  │  "),
        Span::styled(
            fmt::truncate_address(&app.address),
            Style::default().fg(DIM),
        ),
    ]))
    .block(block);

    frame.render_widget(header, area);
}

// ─── Tab bar ────────────────────────────────────────────────────────

fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let tab_titles: Vec<Line> = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            Line::from(format!(" {} {} ", i + 1, t))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .select(app.tab)
        .style(Style::default().fg(DIM))
        .highlight_style(Style::default().fg(ACCENT).bold().underlined())
        .divider("│")
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(DIM)),
        );

    frame.render_widget(tabs, area);
}

// ─── Tab 1: Dashboard ───────────────────────────────────────────────

fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Left: Account overview
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Account card
            Constraint::Min(4),   // Quick positions
        ])
        .split(chunks[0]);

    render_account_card(frame, app, left[0]);
    render_quick_positions(frame, app, left[1]);

    // Right: Top markets
    render_top_markets(frame, app, chunks[1]);
}

fn render_account_card(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Account ")
        .title_style(Style::default().fg(ACCENT).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM));

    let pnl_color = color_for_value(&app.total_ntl_pos);

    let text = vec![
        Line::from(vec![
            Span::styled(" Account Value  ", Style::default().fg(DIM)),
            Span::styled(
                fmt::format_usd(&app.account_value),
                Style::default().fg(WHITE).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Margin Used    ", Style::default().fg(DIM)),
            Span::styled(fmt::format_usd(&app.total_margin_used), Style::default().fg(YELLOW)),
        ]),
        Line::from(vec![
            Span::styled(" Net Position   ", Style::default().fg(DIM)),
            Span::styled(fmt::format_usd(&app.total_ntl_pos), Style::default().fg(pnl_color)),
        ]),
        Line::from(vec![
            Span::styled(" Raw USD        ", Style::default().fg(DIM)),
            Span::styled(fmt::format_usd(&app.total_raw_usd), Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled(" Withdrawable   ", Style::default().fg(DIM)),
            Span::styled(fmt::format_usd(&app.withdrawable), Style::default().fg(GREEN)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(" Positions: ", Style::default().fg(DIM)),
            Span::styled(
                format!("{}", app.positions.len()),
                Style::default().fg(WHITE).bold(),
            ),
            Span::styled("  │  Orders: ", Style::default().fg(DIM)),
            Span::styled(
                format!("{}", app.open_orders.len()),
                Style::default().fg(WHITE).bold(),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_quick_positions(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Positions ")
        .title_style(Style::default().fg(GREEN).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM));

    if app.positions.is_empty() {
        let p = Paragraph::new(" No open positions")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let header = Row::new(vec!["Coin", "Size", "Entry", "uPnL", "Lev"])
        .style(Style::default().fg(ACCENT).bold())
        .bottom_margin(0);

    let rows: Vec<Row> = app
        .positions
        .iter()
        .map(|p| {
            let pnl_color = color_for_value(&p.upnl);
            let side_color = if p.size.starts_with('-') { RED } else { GREEN };
            Row::new(vec![
                Cell::from(p.coin.clone()).style(Style::default().fg(WHITE).bold()),
                Cell::from(p.size.clone()).style(Style::default().fg(side_color)),
                Cell::from(fmt::truncate_number(&p.entry_px)),
                Cell::from(fmt::truncate_number(&p.upnl)).style(Style::default().fg(pnl_color)),
                Cell::from(p.leverage.clone()).style(Style::default().fg(YELLOW)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn render_top_markets(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Markets ")
        .title_style(Style::default().fg(YELLOW).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM));

    if app.all_mids.is_empty() {
        let p = Paragraph::new(" Loading market data...")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let header = Row::new(vec!["Coin", "Mid Price"])
        .style(Style::default().fg(ACCENT).bold());

    // Show first N that fit
    let max_rows = (area.height as usize).saturating_sub(3);
    let rows: Vec<Row> = app
        .all_mids
        .iter()
        .take(max_rows)
        .map(|(coin, mid)| {
            Row::new(vec![
                Cell::from(coin.clone()).style(Style::default().fg(WHITE)),
                Cell::from(fmt::truncate_number(mid)).style(Style::default().fg(GREEN)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(12), Constraint::Min(16)],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

// ─── Tab 2: Positions (full detail) ─────────────────────────────────

fn render_positions(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Positions ")
        .title_style(Style::default().fg(GREEN).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM));

    if app.positions.is_empty() {
        let p = Paragraph::new("\n  No open positions.\n\n  Open a position on Hyperliquid to see it here.")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let header = Row::new(vec![
        "Coin", "Size", "Entry", "Mark", "Liq", "uPnL", "ROE%", "Lev", "Margin",
    ])
    .style(Style::default().fg(ACCENT).bold())
    .bottom_margin(0);

    let rows: Vec<Row> = app
        .positions
        .iter()
        .map(|p| {
            let pnl_color = color_for_value(&p.upnl);
            let roe_color = color_for_value(&p.roe);
            let side_color = if p.size.starts_with('-') { RED } else { GREEN };

            Row::new(vec![
                Cell::from(p.coin.clone()).style(Style::default().fg(WHITE).bold()),
                Cell::from(p.size.clone()).style(Style::default().fg(side_color)),
                Cell::from(fmt::truncate_number(&p.entry_px)),
                Cell::from(fmt::truncate_number(&p.mark_px)),
                Cell::from(fmt::truncate_number(&p.liq_px)).style(Style::default().fg(RED)),
                Cell::from(fmt::truncate_number(&p.upnl)).style(Style::default().fg(pnl_color)),
                Cell::from(fmt::format_pct(&p.roe)).style(Style::default().fg(roe_color)),
                Cell::from(p.leverage.clone()).style(Style::default().fg(YELLOW)),
                Cell::from(fmt::truncate_number(&p.margin_used)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // Coin
            Constraint::Length(12), // Size
            Constraint::Length(12), // Entry
            Constraint::Length(12), // Mark
            Constraint::Length(12), // Liq
            Constraint::Length(12), // uPnL
            Constraint::Length(9),  // ROE
            Constraint::Length(6),  // Lev
            Constraint::Length(12), // Margin
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

// ─── Tab 3: Open orders ─────────────────────────────────────────────

fn render_orders(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Open Orders ")
        .title_style(Style::default().fg(YELLOW).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM));

    if app.open_orders.is_empty() {
        let p = Paragraph::new("\n  No open orders.")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let header = Row::new(vec!["Coin", "Side", "Size", "Price", "Type", "OID"])
        .style(Style::default().fg(ACCENT).bold());

    let rows: Vec<Row> = app
        .open_orders
        .iter()
        .map(|o| {
            let side_color = if o.side == "B" { GREEN } else { RED };
            let side_label = if o.side == "B" { "BUY" } else { "SELL" };
            Row::new(vec![
                Cell::from(o.coin.clone()).style(Style::default().fg(WHITE).bold()),
                Cell::from(side_label).style(Style::default().fg(side_color).bold()),
                Cell::from(o.size.clone()),
                Cell::from(fmt::truncate_number(&o.price)),
                Cell::from(o.order_type.clone()).style(Style::default().fg(DIM)),
                Cell::from(format!("{}", o.oid)).style(Style::default().fg(DIM)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // Coin
            Constraint::Length(6),  // Side
            Constraint::Length(12), // Size
            Constraint::Length(14), // Price
            Constraint::Length(8),  // Type
            Constraint::Min(10),   // OID
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

// ─── Tab 4: All markets ─────────────────────────────────────────────

fn render_markets(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(" Markets ({}) ", app.all_mids.len()))
        .title_style(Style::default().fg(YELLOW).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM));

    if app.all_mids.is_empty() {
        let p = Paragraph::new("\n  Loading...")
            .style(Style::default().fg(DIM))
            .block(block);
        frame.render_widget(p, area);
        return;
    }

    let header = Row::new(vec!["#", "Coin", "Mid Price"])
        .style(Style::default().fg(ACCENT).bold());

    let scroll = app.scroll as usize;
    let rows: Vec<Row> = app
        .all_mids
        .iter()
        .enumerate()
        .skip(scroll)
        .map(|(i, (coin, mid))| {
            Row::new(vec![
                Cell::from(format!("{}", i + 1)).style(Style::default().fg(DIM)),
                Cell::from(coin.clone()).style(Style::default().fg(WHITE).bold()),
                Cell::from(fmt::truncate_number(mid)).style(Style::default().fg(GREEN)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(14),
            Constraint::Min(16),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

// ─── Status bar ─────────────────────────────────────────────────────

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let conn = if app.connected {
        Span::styled("CONNECTED", Style::default().fg(GREEN).bold())
    } else {
        Span::styled("DISCONNECTED", Style::default().fg(RED).bold())
    };

    let error_span = if let Some(ref err) = app.last_error {
        Span::styled(
            format!("  │  {}", fmt::truncate_str(err, 40)),
            Style::default().fg(RED),
        )
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled(" ", Style::default()),
        conn,
        error_span,
        Span::styled(
            format!("  │  Last: {}  ", app.last_refresh),
            Style::default().fg(DIM),
        ),
        Span::styled("?", Style::default().fg(Color::Black).bg(ACCENT).bold()),
        Span::styled(" Help ", Style::default().fg(DIM)),
        Span::styled("q", Style::default().fg(Color::Black).bg(RED).bold()),
        Span::styled(" Quit", Style::default().fg(DIM)),
    ]);

    let bar = Paragraph::new(line).style(Style::default().bg(BG_HEADER));
    frame.render_widget(bar, area);
}

// ─── Help overlay ───────────────────────────────────────────────────

fn render_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 60, area);

    frame.render_widget(Clear, popup);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", Style::default().fg(ACCENT).bold())),
        Line::from(""),
        Line::from("  1-4           Switch tab"),
        Line::from("  Tab / l / →   Next tab"),
        Line::from("  S-Tab / h / ← Previous tab"),
        Line::from("  j / ↓         Scroll down"),
        Line::from("  k / ↑         Scroll up"),
        Line::from(""),
        Line::from(Span::styled("  Actions", Style::default().fg(ACCENT).bold())),
        Line::from(""),
        Line::from("  r             Refresh data"),
        Line::from("  ?             Toggle help"),
        Line::from("  q / Ctrl+C    Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press ? or Esc to close",
            Style::default().fg(DIM),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .title_style(Style::default().fg(ACCENT).bold())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(ACCENT)),
        )
        .style(Style::default().bg(Color::Rgb(15, 15, 30)));

    frame.render_widget(help, popup);
}

// ─── Helpers ────────────────────────────────────────────────────────

/// Determine color based on numeric string sign.
fn color_for_value(s: &str) -> Color {
    match fmt::sign_of(s) {
        Sign::Negative => RED,
        Sign::Zero => DIM,
        Sign::Positive => GREEN,
    }
}

/// Create a centered rectangle for overlay popups.
fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
