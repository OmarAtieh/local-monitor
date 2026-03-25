use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let block = Block::default()
        .title(" RAM ")
        .title_style(Style::default().fg(theme::TITLE_COLOR))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // RAM gauge
    let pct = sample.ram_percent.clamp(0.0, 100.0);
    let used_gb = bytes_to_gb(sample.ram_used_bytes);
    let total_gb = bytes_to_gb(sample.ram_total_bytes);
    let label = format!("{pct:.0}%  {used_gb:.1}/{total_gb:.1} GB");
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(theme::utilization_color(pct)))
        .ratio(pct / 100.0)
        .label(label);
    f.render_widget(gauge, chunks[0]);

    // Swap gauge
    let swap_pct = sample.swap_percent.clamp(0.0, 100.0);
    let swap_used_gb = bytes_to_gb(sample.swap_used_bytes);
    let swap_total_gb = bytes_to_gb(sample.swap_total_bytes);
    let swap_label = format!("Swap {swap_pct:.0}%  {swap_used_gb:.1}/{swap_total_gb:.1} GB");
    let swap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(theme::utilization_color(swap_pct)))
        .ratio(swap_pct / 100.0)
        .label(swap_label);
    f.render_widget(swap_gauge, chunks[1]);
}
