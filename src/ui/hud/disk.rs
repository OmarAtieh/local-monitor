use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

/// Format a byte rate to a human-readable string, auto-scaling between KB/s and MB/s.
pub fn format_bytes_rate(bytes: f64) -> String {
    if bytes >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes / 1_048_576.0)
    } else {
        format!("{:.1} KB/s", bytes / 1024.0)
    }
}

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let block = Block::default()
        .title(" Disk ")
        .title_style(Style::default().fg(theme::TITLE_COLOR))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let capacity_pct = if sample.disk_total_bytes > 0 {
        (sample.disk_used_bytes as f64 / sample.disk_total_bytes as f64) * 100.0
    } else {
        0.0
    };
    let capacity_pct = capacity_pct.clamp(0.0, 100.0);

    let read_rate = format_bytes_rate(sample.disk_read_bytes as f64);
    let write_rate = format_bytes_rate(sample.disk_write_bytes as f64);
    let label = format!("{capacity_pct:.0}%  R:{read_rate}  W:{write_rate}");

    if inner.height >= 1 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1)])
            .split(inner);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(theme::utilization_color(capacity_pct)))
            .ratio(capacity_pct / 100.0)
            .label(label);
        f.render_widget(gauge, chunks[0]);
    }
}
