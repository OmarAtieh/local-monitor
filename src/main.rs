mod app;
mod collectors;
mod db;
mod metrics;
mod ui;

use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use app::App;
use collectors::build_collectors;
use db::Db;
use metrics::Sample;
use ui::theme;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

fn main() -> Result<()> {
    // Initialize database (fallback to in-memory with warning).
    let (database, db_warning) = match Db::open() {
        Ok(d) => (d, None),
        Err(e) => {
            let warning = format!("DB error: {e:#}. Running in-memory only.");
            let d = Db::open_in_memory()?;
            (d, Some(warning))
        }
    };

    // Initialize collectors and detect GPU availability.
    let mut collectors_list = build_collectors();
    let gpu_available = collectors_list.iter().any(|c| c.name() == "gpu");

    let mut app = App::new(gpu_available);
    app.db_warning = db_warning;

    // Setup terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;

    // Set panic hook to restore terminal on crash.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut tick_count: u64 = 0;

    // Main loop.
    while app.running {
        let tick_start = Instant::now();

        // Collect metrics.
        let mut sample = Sample {
            ts: chrono::Utc::now().timestamp(),
            ..Sample::default()
        };
        for collector in &mut collectors_list {
            collector.collect(&mut sample);
        }

        // Insert to DB (ignore errors to keep running).
        let _ = database.insert_sample(&sample);

        app.latest_sample = sample;

        // Aggregate every 60 seconds.
        tick_count += 1;
        if tick_count.is_multiple_of(60) {
            let _ = database.aggregate_and_prune();
        }

        // Query graph data.
        let graph_data = database.query(app.granularity).unwrap_or_default();

        // Render.
        terminal.draw(|f| {
            let size = f.area();

            if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
                let msg = format!(
                    "Terminal too small: {}x{}. Need at least {}x{}.",
                    size.width, size.height, MIN_WIDTH, MIN_HEIGHT
                );
                let paragraph = Paragraph::new(msg).style(Style::default().fg(theme::TITLE_COLOR));
                f.render_widget(paragraph, size);
                return;
            }

            // Show DB warning banner if present.
            let (warn_area, main_area) = if app.db_warning.is_some() {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(size);
                (Some(chunks[0]), chunks[1])
            } else {
                (None, size)
            };

            if let (Some(warn_rect), Some(warning)) = (warn_area, &app.db_warning) {
                let warn_text = Paragraph::new(warning.as_str())
                    .style(Style::default().fg(ratatui::style::Color::Yellow));
                f.render_widget(warn_text, warn_rect);
            }

            // Split main area into HUD (top) and graphs (bottom).
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(12), // HUD: 3+3+3+3
                    Constraint::Min(6),     // Graphs
                ])
                .split(main_area);

            ui::hud::render_hud(f, layout[0], &app.latest_sample);
            ui::graphs::render_graphs(f, layout[1], app.focus, app.granularity, &graph_data);
        })?;

        // Handle input — wait for remaining tick time.
        let elapsed = tick_start.elapsed();
        let timeout = Duration::from_secs(1).saturating_sub(elapsed);

        if event::poll(timeout)? {
            if let Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) = event::read()?
            {
                match (code, modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('Q'), _) => {
                        app.running = false;
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        app.running = false;
                    }
                    (KeyCode::Right, _) | (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) => {
                        app.next_view();
                    }
                    (KeyCode::Left, _) | (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) => {
                        app.prev_view();
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('w'), _) | (KeyCode::Char('W'), _) => {
                        app.shorter_granularity();
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('s'), _) | (KeyCode::Char('S'), _) => {
                        app.longer_granularity();
                    }
                    _ => {}
                }
            }
        }
    }

    // Cleanup.
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
