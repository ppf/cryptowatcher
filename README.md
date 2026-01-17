# Cryptowatcher

Rust TUI app for real-time cryptocurrency price monitoring with live charts.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

![Cryptowatcher](assets/screenshot.png)

## Features

- Real-time price tracking via Binance API
- Live price charts with 1-hour history
- Dynamic grid layout (up to 4 charts visible)
- 24h stats: high/low, volume, % change
- Configurable coin list
- Auto-refresh every 60 seconds

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Default (BTC, ETH)
cryptowatcher

# Custom coins
cryptowatcher --coins BTC,ETH,SOL,DOGE

# Custom refresh interval (seconds)
cryptowatcher --interval 30
```

## Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `r` | Force refresh |
| `←/→` | Page navigation (when >4 coins) |

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [tokio](https://tokio.rs) - Async runtime
- [reqwest](https://docs.rs/reqwest) - HTTP client

## License

MIT
