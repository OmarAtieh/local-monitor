use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

use super::disk::format_bytes_rate;

fn format_total(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        format!("{gb:.1}G")
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{mb:.0}M")
    }
}

/// Render network info into the right half of a paired row.
pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height == 0 || area.width < 10 {
        return;
    }

    let label = "NET ";
    let dl_rate = format_bytes_rate(sample.net_rx_bytes as f64);
    let ul_rate = format_bytes_rate(sample.net_tx_bytes as f64);
    let dl_total = format_total(sample.net_rx_total);
    let ul_total = format_total(sample.net_tx_total);

    let detail_str =
        format!("\u{2193}{dl_rate} \u{2191}{ul_rate} \u{2193}{dl_total} \u{2191}{ul_total}");

    let line = Line::from(vec![
        Span::styled(label.to_string(), Style::default().fg(theme::LABEL_DSK)),
        Span::styled(detail_str, Style::default().fg(theme::DETAIL_COLOR)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
