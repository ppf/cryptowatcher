mod api;
mod app;
mod event;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use api::BinanceClient;
use app::App;
use event::{AppEvent, EventHandler};

const MAX_COINS: usize = 20;

#[derive(Parser, Debug)]
#[command(name = "cryptowatcher")]
#[command(about = "Real-time cryptocurrency price watcher with TUI charts")]
struct Args {
    #[arg(short, long, default_value = "BTC,ETH", value_delimiter = ',')]
    coins: Vec<String>,

    #[arg(short, long, default_value = "60")]
    interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut symbols: Vec<String> = Vec::new();
    for coin in args.coins.iter().take(MAX_COINS) {
        let trimmed = coin.trim().to_uppercase();
        if trimmed.chars().all(|ch| ch.is_alphanumeric()) {
            symbols.push(format!("{}USDT", trimmed));
        } else {
            eprintln!("Warning: Skipping invalid coin symbol: {}", coin);
        }
    }

    if symbols.is_empty() {
        eprintln!("Error: No valid coin symbols provided");
        std::process::exit(1);
    }

    let tick_rate = Duration::from_secs(args.interval);

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, symbols, tick_rate).await;

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    symbols: Vec<String>,
    tick_rate: Duration,
) -> Result<()> {
    let mut app = App::new(symbols);
    let client = BinanceClient::new()?;
    let mut events = EventHandler::new(tick_rate);

    // Load last hour's history on startup
    app.load_historical(&client).await;
    app.fetch_prices(&client).await;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        match events.next().await? {
            AppEvent::Tick => {
                app.status_message = "Fetching...".to_string();
                terminal.draw(|f| ui::render(f, &app))?;
                app.fetch_prices(&client).await;
            }
            AppEvent::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                KeyCode::Char('r') => {
                    app.status_message = "Refreshing...".to_string();
                    terminal.draw(|f| ui::render(f, &app))?;
                    app.fetch_prices(&client).await;
                }
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                _ => {}
            },
            AppEvent::Quit => app.quit(),
            AppEvent::Resize => {}
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
