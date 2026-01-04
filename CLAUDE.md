# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build              # Build
cargo run                # Run with defaults (BTC, ETH)
cargo run -- --coins BTC,ETH,SOL --interval 30  # Custom coins/refresh
cargo test               # Run tests
cargo clippy -- -D warnings  # Lint (CI enforced)
cargo fmt                # Format (CI enforced)
```

## Architecture

Async TUI app using tokio + ratatui with synthwave color theme.

```
main.rs     Entry point, CLI (clap), terminal setup, main event loop
app.rs      App state + CoinData (price history as VecDeque<(timestamp, price)>)
api.rs      BinanceClient - ticker/24h and klines endpoints
ui.rs       Rendering: coin charts with 24h stats, status bar (synthwave palette)
event.rs    EventHandler - keyboard input + tick timer via tokio channels
```

**Data flow**: EventHandler emits Tick/Key → main loop calls `app.fetch_prices()` → BinanceClient fetches → CoinData updates → ui::render draws charts

**Price history**: 60 data points max (15-min klines on startup, then live updates appended)

## Synthwave Theme (ui.rs)

Colors defined as RGB constants: PINK (#ff2e97), CYAN (#00f0ff), POSITIVE (neon green), BORDER (deep purple), MUTED, TEXT. Chart colors cycle through 6-color palette.
