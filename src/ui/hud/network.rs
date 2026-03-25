use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

use super::disk::format_bytes_rate;

fn format_total(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        format!("{gb:.1} GB")
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{mb:.1} MB")
    }
}

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let block = Block::default()
        .title(" Network ")
        .title_style(Style::default().fg(theme::TITLE_COLOR))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let dl_rate = format_bytes_rate(sample.net_rx_bytes as f64);
    let ul_rate = format_bytes_rate(sample.net_tx_bytes as f64);
    let dl_total = format_total(sample.net_rx_total);
    let ul_total = format_total(sample.net_tx_total);

    let text = format!("DL:{dl_rate} ({dl_total})  UL:{ul_rate} ({ul_total})");

    let paragraph = Paragraph::new(text).style(Style::default().fg(theme::LABEL_COLOR));
    f.render_widget(paragraph, inner);
}
