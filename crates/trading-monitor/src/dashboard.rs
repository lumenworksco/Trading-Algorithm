//! TUI Dashboard using ratatui.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};
use rust_decimal::Decimal;
use std::io;
use std::time::Duration;
use trading_core::types::Portfolio;

/// Dashboard state.
pub struct DashboardState {
    pub portfolio: Portfolio,
    pub strategy_name: String,
    pub signals_today: usize,
    pub trades_today: usize,
    pub daily_pnl: Decimal,
    pub messages: Vec<String>,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            portfolio: Portfolio::new(Decimal::ZERO),
            strategy_name: String::new(),
            signals_today: 0,
            trades_today: 0,
            daily_pnl: Decimal::ZERO,
            messages: Vec::new(),
        }
    }
}

/// TUI Dashboard.
pub struct Dashboard {
    refresh_ms: u64,
}

impl Dashboard {
    /// Create a new dashboard.
    pub fn new(refresh_ms: u64) -> Self {
        Self { refresh_ms }
    }

    /// Run the dashboard.
    pub fn run<F>(&self, mut get_state: F) -> io::Result<()>
    where
        F: FnMut() -> DashboardState,
    {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_loop(&mut terminal, &mut get_state);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        res
    }

    fn run_loop<F>(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        get_state: &mut F,
    ) -> io::Result<()>
    where
        F: FnMut() -> DashboardState,
    {
        loop {
            let state = get_state();
            terminal.draw(|f| self.ui(f, &state))?;

            if event::poll(Duration::from_millis(self.refresh_ms))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn ui(&self, frame: &mut Frame, state: &DashboardState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(5), // Stats
                Constraint::Min(10),   // Positions
                Constraint::Length(8), // Messages
            ])
            .split(frame.area());

        self.render_header(frame, chunks[0], state);
        self.render_stats(frame, chunks[1], state);
        self.render_positions(frame, chunks[2], state);
        self.render_messages(frame, chunks[3], state);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect, state: &DashboardState) {
        let header = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "Trading Dashboard",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(&state.strategy_name, Style::default().fg(Color::Cyan)),
            Span::raw(" | Press 'q' to quit"),
        ])])
        .block(Block::default().borders(Borders::ALL).title("System"));
        frame.render_widget(header, area);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect, state: &DashboardState) {
        let pnl_color = if state.daily_pnl >= Decimal::ZERO {
            Color::Green
        } else {
            Color::Red
        };

        let stats = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Equity: "),
                Span::styled(
                    format!("${:.2}", state.portfolio.equity),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  |  Cash: "),
                Span::styled(format!("${:.2}", state.portfolio.cash), Style::default()),
                Span::raw("  |  Daily P&L: "),
                Span::styled(
                    format!("${:.2}", state.daily_pnl),
                    Style::default().fg(pnl_color),
                ),
            ]),
            Line::from(vec![
                Span::raw("Positions: "),
                Span::styled(
                    format!("{}", state.portfolio.position_count()),
                    Style::default(),
                ),
                Span::raw("  |  Signals: "),
                Span::styled(format!("{}", state.signals_today), Style::default()),
                Span::raw("  |  Trades: "),
                Span::styled(format!("{}", state.trades_today), Style::default()),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title("Statistics"));
        frame.render_widget(stats, area);
    }

    fn render_positions(&self, frame: &mut Frame, area: Rect, state: &DashboardState) {
        let header_cells = ["Symbol", "Qty", "Entry", "Current", "P&L", "P&L %"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows = state.portfolio.positions.values().map(|pos| {
            let pnl_color = if pos.unrealized_pnl >= Decimal::ZERO {
                Color::Green
            } else {
                Color::Red
            };

            Row::new(vec![
                Cell::from(pos.symbol.clone()),
                Cell::from(format!("{}", pos.quantity)),
                Cell::from(format!("${:.2}", pos.avg_entry_price)),
                Cell::from(format!("${:.2}", pos.current_price)),
                Cell::from(format!("${:.2}", pos.unrealized_pnl))
                    .style(Style::default().fg(pnl_color)),
                Cell::from(format!("{:.2}%", pos.unrealized_pnl_percent))
                    .style(Style::default().fg(pnl_color)),
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Positions"));

        frame.render_widget(table, area);
    }

    fn render_messages(&self, frame: &mut Frame, area: Rect, state: &DashboardState) {
        let messages: Vec<Line> = state
            .messages
            .iter()
            .rev()
            .take(5)
            .map(|m| Line::from(m.as_str()))
            .collect();

        let paragraph =
            Paragraph::new(messages).block(Block::default().borders(Borders::ALL).title("Log"));
        frame.render_widget(paragraph, area);
    }
}
