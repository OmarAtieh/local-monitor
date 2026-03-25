use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
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

/// Render RAM info into the left half of a paired row.
pub fn render_ram(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height == 0 || area.width < 10 {
        return;
    }

    let inner_w = area.width as usize;
    let label = "RAM ";
    let pct = sample.ram_percent.clamp(0.0, 100.0);
    let pct_str = format!("{:>3.0}%", pct);
    let used_gb = bytes_to_gb(sample.ram_used_bytes);
    let total_gb = bytes_to_gb(sample.ram_total_bytes);
    let detail_str = format!("{used_gb:.1}/{total_gb:.1} GB");

    let fixed = label.len() + 1 + pct_str.len() + 1 + detail_str.len();
    let bar_width = inner_w.saturating_sub(fixed).max(2);

    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled(
        label.to_string(),
        Style::default().fg(theme::LABEL_RAM),
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

/// Render Swap info into the right half of a paired row.
pub fn render_swap(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height == 0 || area.width < 10 {
        return;
    }

    let inner_w = area.width as usize;
    let label = "SWP ";
    let pct = sample.swap_percent.clamp(0.0, 100.0);
    let pct_str = format!("{:>3.0}%", pct);
    let used_gb = bytes_to_gb(sample.swap_used_bytes);
    let total_gb = bytes_to_gb(sample.swap_total_bytes);
    let detail_str = format!("{used_gb:.1}/{total_gb:.1} GB");

    let fixed = label.len() + 1 + pct_str.len() + 1 + detail_str.len();
    let bar_width = inner_w.saturating_sub(fixed).max(2);

    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled(
        label.to_string(),
        Style::default().fg(theme::LABEL_RAM),
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
