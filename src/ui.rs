use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::{App, CoinData};

// Synthwave color palette
const PINK: Color = Color::Rgb(255, 46, 151); // #ff2e97
const CYAN: Color = Color::Rgb(0, 240, 255); // #00f0ff
const POSITIVE: Color = Color::Rgb(57, 255, 20); // #39ff14
const BORDER: Color = Color::Rgb(61, 26, 120); // #3d1a78
const MUTED: Color = Color::Rgb(107, 91, 149); // #6b5b95
const TEXT: Color = Color::Rgb(240, 240, 240); // #f0f0f0

const CHART_COLORS: [Color; 6] = [
    Color::Rgb(255, 46, 151), // Hot pink
    Color::Rgb(0, 240, 255),  // Cyan
    Color::Rgb(157, 78, 221), // Purple
    Color::Rgb(247, 37, 133), // Magenta
    Color::Rgb(76, 201, 240), // Light blue
    Color::Rgb(114, 9, 183),  // Deep violet
];

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let visible_coins = app.coins.len().min(area.height as usize / 8).max(1);
    let mut constraints: Vec<Constraint> = (0..visible_coins)
        .map(|_| Constraint::Ratio(1, visible_coins as u32))
        .collect();
    constraints.push(Constraint::Length(3));

    let chunks = Layout::vertical(constraints).split(area);

    for (i, coin) in app
        .coins
        .iter()
        .skip(app.scroll_offset)
        .take(visible_coins)
        .enumerate()
    {
        render_coin_chart(frame, chunks[i], coin, CHART_COLORS[i % CHART_COLORS.len()]);
    }

    render_status_bar(frame, chunks[visible_coins], app);
}

fn render_coin_chart(frame: &mut Frame, area: Rect, coin: &CoinData, color: Color) {
    let data = coin.history_data();
    let (y_min, y_max) = coin.price_bounds();

    let change_color = if coin.change_24h >= 0.0 {
        POSITIVE
    } else {
        PINK
    };

    let change_arrow = if coin.change_24h >= 0.0 { "▲" } else { "▼" };

    let title = Line::from(vec![
        Span::styled("◈ ", Style::default().fg(PINK)),
        Span::styled(
            coin.display_name.as_str(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(BORDER)),
        Span::styled(
            format_price(coin.price),
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(BORDER)),
        Span::styled(
            format!("{} {:.2}%", change_arrow, coin.change_24h.abs()),
            Style::default().fg(change_color),
        ),
        Span::styled(" │ ", Style::default().fg(BORDER)),
        Span::styled(
            format!(
                "H:{} L:{}",
                format_price_short(coin.high_24h),
                format_price_short(coin.low_24h)
            ),
            Style::default().fg(MUTED),
        ),
        Span::styled(" │ ", Style::default().fg(BORDER)),
        Span::styled(
            format!("Vol:{}", format_volume(coin.volume_24h)),
            Style::default().fg(MUTED),
        ),
        Span::styled(" ◈", Style::default().fg(PINK)),
    ]);

    let dataset = Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(&data);

    let x_max = coin.price_history.len().max(60) as f64;
    let time_labels = coin.time_labels();
    let x_labels: Vec<Span> = time_labels
        .iter()
        .map(|s| Span::styled(s.as_str(), Style::default().fg(MUTED)))
        .collect();

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER)),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(MUTED))
                .bounds([0.0, x_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(MUTED))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::styled(format_price_short(y_min), Style::default().fg(MUTED)),
                    Span::styled(format_price_short(y_max), Style::default().fg(MUTED)),
                ]),
        );

    frame.render_widget(chart, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let status = Line::from(vec![
        Span::raw(" "),
        Span::styled("Q", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("·Quit  ", Style::default().fg(MUTED)),
        Span::styled("R", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("·Refresh  ", Style::default().fg(MUTED)),
        Span::styled("↑↓", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("·Scroll", Style::default().fg(MUTED)),
        Span::raw("          "),
        Span::styled(
            format!("Updated {}", app.last_update_str()),
            Style::default().fg(MUTED),
        ),
        Span::raw("  "),
        Span::styled(&app.status_message, Style::default().fg(CYAN)),
    ]);

    let paragraph = Paragraph::new(status).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER)),
    );

    frame.render_widget(paragraph, area);
}

fn format_volume(vol: f64) -> String {
    if vol >= 1_000_000_000.0 {
        format!("{:.1}B", vol / 1_000_000_000.0)
    } else if vol >= 1_000_000.0 {
        format!("{:.1}M", vol / 1_000_000.0)
    } else if vol >= 1_000.0 {
        format!("{:.1}K", vol / 1_000.0)
    } else {
        format!("{:.0}", vol)
    }
}

fn format_price(price: f64) -> String {
    if price >= 1000.0 {
        // Round to cents first to handle edge cases like 99.999 → 100.00
        let rounded = (price * 100.0).round() / 100.0;
        let whole = rounded as i64;
        let frac = ((rounded - whole as f64) * 100.0).round().clamp(0.0, 99.0) as u8;
        let s = whole.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3);
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(c);
        }
        format!("${}.{:02}", result, frac)
    } else {
        format!("${:.2}", price)
    }
}

fn format_price_short(price: f64) -> String {
    if price >= 1_000_000.0 {
        format!("${:.1}M", price / 1_000_000.0)
    } else if price >= 1_000.0 {
        format!("${:.1}k", price / 1_000.0)
    } else {
        format!("${:.2}", price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_volume() {
        assert_eq!(format_volume(500.0), "500");
        assert_eq!(format_volume(1_500.0), "1.5K");
        assert_eq!(format_volume(1_500_000.0), "1.5M");
        assert_eq!(format_volume(1_500_000_000.0), "1.5B");
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(0.50), "$0.50");
        assert_eq!(format_price(99.99), "$99.99");
        assert_eq!(format_price(1000.00), "$1,000.00");
        assert_eq!(format_price(42069.42), "$42,069.42");
        assert_eq!(format_price(100000.00), "$100,000.00");
    }

    #[test]
    fn test_format_price_short() {
        assert_eq!(format_price_short(0.50), "$0.50");
        assert_eq!(format_price_short(999.99), "$999.99");
        assert_eq!(format_price_short(1500.0), "$1.5k");
        assert_eq!(format_price_short(1_500_000.0), "$1.5M");
    }
}
