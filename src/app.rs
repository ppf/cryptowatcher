use std::collections::VecDeque;
use std::time::Instant;

use chrono::{Local, TimeZone};

use crate::api::{BinanceClient, TickerData};

const MAX_HISTORY: usize = 60;

#[derive(Debug, Clone)]
pub struct CoinData {
    pub symbol: String,
    pub display_name: String,
    pub price: f64,
    pub change_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub volume_24h: f64,
    pub price_history: VecDeque<(i64, f64)>, // (timestamp_ms, price)
}

impl CoinData {
    pub fn new(symbol: &str) -> Self {
        let display_name = symbol.replace("USDT", "/USDT");
        Self {
            symbol: symbol.to_string(),
            display_name,
            price: 0.0,
            change_24h: 0.0,
            high_24h: 0.0,
            low_24h: 0.0,
            volume_24h: 0.0,
            price_history: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    pub fn update(&mut self, ticker: &TickerData) {
        self.price = ticker.last_price;
        self.change_24h = ticker.price_change_percent;
        self.high_24h = ticker.high_price;
        self.low_24h = ticker.low_price;
        self.volume_24h = ticker.volume;

        let now_ms = chrono::Utc::now().timestamp_millis();
        if self.price_history.len() >= MAX_HISTORY {
            self.price_history.pop_front();
        }
        self.price_history.push_back((now_ms, self.price));
    }

    pub fn history_data(&self) -> Vec<(f64, f64)> {
        // Convert to (index, price) for chart rendering
        self.price_history
            .iter()
            .enumerate()
            .map(|(i, (_, p))| (i as f64, *p))
            .collect()
    }

    pub fn price_bounds(&self) -> (f64, f64) {
        if self.price_history.is_empty() {
            return (0.0, 100.0);
        }
        let prices: Vec<f64> = self.price_history.iter().map(|(_, p)| *p).collect();
        let min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }

    pub fn time_labels(&self) -> Vec<String> {
        if self.price_history.is_empty() {
            return vec![
                "--:--".to_string(),
                "--:--".to_string(),
                "--:--".to_string(),
            ];
        }

        let format_time = |ts_ms: i64| -> String {
            match Local.timestamp_millis_opt(ts_ms).single() {
                Some(dt) => dt.format("%H:%M").to_string(),
                None => "--:--".to_string(),
            }
        };

        let first = self.price_history.front().map(|(ts, _)| *ts).unwrap_or(0);
        let last = self.price_history.back().map(|(ts, _)| *ts).unwrap_or(0);
        let mid = (first + last) / 2;

        vec![format_time(first), format_time(mid), format_time(last)]
    }

    pub fn load_history(&mut self, data: Vec<(i64, f64)>) {
        self.price_history.clear();
        for (ts, price) in data {
            self.price_history.push_back((ts, price));
        }
        if let Some((_, last_price)) = self.price_history.back() {
            self.price = *last_price;
        }
    }
}

pub struct App {
    pub coins: Vec<CoinData>,
    pub last_update: Option<Instant>,
    pub running: bool,
    pub scroll_offset: usize,
    pub status_message: String,
}

impl App {
    pub fn new(symbols: Vec<String>) -> Self {
        let coins = symbols.iter().map(|s| CoinData::new(s)).collect();
        Self {
            coins,
            last_update: None,
            running: true,
            scroll_offset: 0,
            status_message: "Starting...".to_string(),
        }
    }

    pub async fn load_historical(&mut self, client: &BinanceClient) {
        self.status_message = "Loading history...".to_string();
        let symbols: Vec<String> = self.coins.iter().map(|c| c.symbol.clone()).collect();
        let results = client.get_klines_batch(&symbols, MAX_HISTORY as u32).await;

        for (coin, result) in self.coins.iter_mut().zip(results.into_iter()) {
            match result {
                Ok(data) => coin.load_history(data),
                Err(e) => {
                    self.status_message =
                        format!("Error loading history for {}: {}", coin.symbol, e);
                }
            }
        }
    }

    pub async fn fetch_prices(&mut self, client: &BinanceClient) {
        let symbols: Vec<String> = self.coins.iter().map(|c| c.symbol.clone()).collect();
        let results = client.get_tickers(&symbols).await;

        for (coin, result) in self.coins.iter_mut().zip(results.into_iter()) {
            match result {
                Ok(ticker) => coin.update(&ticker),
                Err(e) => {
                    self.status_message = format!("Error fetching {}: {}", coin.symbol, e);
                }
            }
        }
        self.last_update = Some(Instant::now());
        self.status_message = "Updated".to_string();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.coins.len().saturating_sub(2) {
            self.scroll_offset += 1;
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn last_update_str(&self) -> String {
        match self.last_update {
            Some(instant) => {
                let secs = instant.elapsed().as_secs();
                if secs < 60 {
                    format!("{}s ago", secs)
                } else {
                    format!("{}m ago", secs / 60)
                }
            }
            None => "Never".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coin_data_new() {
        let coin = CoinData::new("BTCUSDT");
        assert_eq!(coin.symbol, "BTCUSDT");
        assert_eq!(coin.display_name, "BTC/USDT");
        assert_eq!(coin.price, 0.0);
        assert!(coin.price_history.is_empty());
    }

    #[test]
    fn test_coin_data_load_history() {
        let mut coin = CoinData::new("BTCUSDT");
        let history = vec![(1000, 100.0), (2000, 110.0), (3000, 105.0)];
        coin.load_history(history);

        assert_eq!(coin.price_history.len(), 3);
        assert_eq!(coin.price, 105.0);
    }

    #[test]
    fn test_coin_data_history_data() {
        let mut coin = CoinData::new("BTCUSDT");
        coin.load_history(vec![(1000, 100.0), (2000, 200.0)]);

        let data = coin.history_data();
        assert_eq!(data, vec![(0.0, 100.0), (1.0, 200.0)]);
    }

    #[test]
    fn test_coin_data_price_bounds() {
        let mut coin = CoinData::new("BTCUSDT");
        coin.load_history(vec![(1000, 100.0), (2000, 200.0)]);

        let (min, max) = coin.price_bounds();
        assert!(min < 100.0);
        assert!(max > 200.0);
    }

    #[test]
    fn test_coin_data_price_bounds_empty() {
        let coin = CoinData::new("BTCUSDT");
        let (min, max) = coin.price_bounds();
        assert_eq!(min, 0.0);
        assert_eq!(max, 100.0);
    }

    #[test]
    fn test_app_scroll() {
        let mut app = App::new(vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "SOLUSDT".to_string(),
        ]);
        assert_eq!(app.scroll_offset, 0);

        app.scroll_down();
        assert_eq!(app.scroll_offset, 1);

        app.scroll_up();
        assert_eq!(app.scroll_offset, 0);

        app.scroll_up();
        assert_eq!(app.scroll_offset, 0);
    }
}
