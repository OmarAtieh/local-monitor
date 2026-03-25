use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
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

fn build_bar_spans(pct: f64, bar_width: usize) -> Vec<Span<'static>> {
    let filled = ((pct / 100.0) * bar_width as f64).round() as usize;
    let unfilled = bar_width.saturating_sub(filled);
    let color = theme::utilization_color(pct);

    vec![
        Span::styled("\u{2588}".repeat(filled), Style::default().fg(color)),
        Span::styled(
            "\u{2588}".repeat(unfilled),
            Style::default().fg(theme::BAR_EMPTY),
        ),
    ]
}

/// Render disk info into the left half of a paired row.
pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height == 0 || area.width < 10 {
        return;
    }

    let inner_w = area.width as usize;
    let label = "DSK ";

    let capacity_pct = if sample.disk_total_bytes > 0 {
        (sample.disk_used_bytes as f64 / sample.disk_total_bytes as f64) * 100.0
    } else {
        0.0
    };
    let pct = capacity_pct.clamp(0.0, 100.0);
    let pct_str = format!("{:>3.0}%", pct);

    let read_rate = format_bytes_rate(sample.disk_read_bytes as f64);
    let write_rate = format_bytes_rate(sample.disk_write_bytes as f64);
    let detail_str = format!("R:{read_rate} W:{write_rate}");

    let fixed = label.len() + 1 + pct_str.len() + 1 + detail_str.len();
    let bar_width = inner_w.saturating_sub(fixed).max(2);

    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled(
        label.to_string(),
        Style::default().fg(theme::LABEL_DSK),
    ));
    spans.extend(build_bar_spans(pct, bar_width));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        pct_str,
        Style::default().fg(theme::utilization_color(pct)),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        detail_str,
        Style::default().fg(theme::DETAIL_COLOR),
    ));

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}
