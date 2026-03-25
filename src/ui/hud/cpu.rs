use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

const BLOCK_CHARS: &[char] = &[
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
    '\u{2588}',
];
const LABEL_WIDTH: usize = 5; // indent for wrapped rows
const FIXED_OVERHEAD: usize = 24; // "CPU " + bar spacing + pct + freq (approx)

fn heatmap_char(pct: f64) -> char {
    let idx = ((pct.clamp(0.0, 100.0) / 100.0) * 7.0).round() as usize;
    BLOCK_CHARS[idx.min(7)]
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

/// How many characters to use per core so the heatmap fills ~half the row width.
fn chars_per_core(core_count: usize, target_width: usize) -> usize {
    if core_count == 0 {
        return 1;
    }
    (target_width / core_count).max(1)
}

/// Calculate how many extra rows the heatmap needs beyond the CPU bar row.
pub fn heatmap_extra_rows(core_count: usize, terminal_width: u16) -> u16 {
    if core_count == 0 {
        return 0;
    }
    let inner_width = (terminal_width as usize).saturating_sub(4);
    let half = inner_width / 2;
    let cpc = chars_per_core(core_count, half);
    let total_chars = core_count * cpc;

    // Inline space available after label + bar + pct + freq + 2 space separator
    let inline_space = inner_width.saturating_sub(FIXED_OVERHEAD + 2);

    if total_chars <= inline_space {
        return 0; // fits inline
    }

    // Wraps to dedicated rows
    let wrap_width = inner_width.saturating_sub(LABEL_WIDTH);
    if wrap_width == 0 {
        return 0;
    }
    total_chars.div_ceil(wrap_width) as u16
}

/// Build the heatmap spans for all cores, with `cpc` characters per core.
fn build_heatmap_spans(per_core: &[f64], cpc: usize) -> Vec<Span<'static>> {
    per_core
        .iter()
        .map(|&p| {
            let ch = heatmap_char(p);
            Span::styled(
                String::from(ch).repeat(cpc),
                Style::default().fg(theme::utilization_color(p)),
            )
        })
        .collect()
}

/// Render the CPU row content. May use multiple rows for heatmap wrapping.
pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height == 0 || area.width < 20 {
        return;
    }

    let pct = sample.cpu_percent.clamp(0.0, 100.0);
    let freq_ghz = sample.cpu_freq_mhz / 1000.0;
    let inner_w = area.width as usize;

    let label_str = "CPU ";
    let pct_str = format!("{:>3.0}%", pct);
    let freq_str = format!("{:.2} GHz", freq_ghz);
    let fixed_len = label_str.len() + 1 + pct_str.len() + 1 + freq_str.len();

    let core_count = sample.per_core_percent.len();
    let half = inner_w / 2;
    let cpc = chars_per_core(core_count, half);
    let total_heatmap_chars = core_count * cpc;

    // Build heatmap spans with multi-char per core
    let heatmap_spans = build_heatmap_spans(&sample.per_core_percent, cpc);

    // Inline space available after freq + 2 char separator
    let inline_space = inner_w.saturating_sub(fixed_len + 2);
    let fits_inline = total_heatmap_chars <= inline_space && core_count > 0;

    // Bar width: fill space between label and pct, accounting for inline heatmap if it fits
    let after_bar = 1
        + pct_str.len()
        + 1
        + freq_str.len()
        + if fits_inline {
            2 + total_heatmap_chars
        } else {
            0
        };
    let bar_width = inner_w.saturating_sub(label_str.len() + after_bar).max(4);

    // Build the main CPU row
    let mut spans: Vec<Span<'static>> = Vec::new();
    spans.push(Span::styled(
        label_str.to_string(),
        Style::default().fg(theme::LABEL_CPU),
    ));
    spans.extend(build_bar_spans(pct, bar_width));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        pct_str,
        Style::default().fg(theme::utilization_color(pct)),
    ));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        freq_str,
        Style::default().fg(theme::DETAIL_COLOR),
    ));

    // Inline heatmap
    if fits_inline {
        spans.push(Span::raw("  "));
        for s in &heatmap_spans {
            spans.push(s.clone());
        }
    }

    let row_rect = Rect::new(area.x, area.y, area.width, 1);
    f.render_widget(Paragraph::new(Line::from(spans)), row_rect);

    // Wrap heatmap to additional rows if it didn't fit inline
    if !fits_inline && core_count > 0 && area.height > 1 {
        let wrap_width = inner_w.saturating_sub(LABEL_WIDTH);
        if wrap_width == 0 {
            return;
        }
        // How many cores fit per wrapped row (in chars)
        let cores_per_row = wrap_width / cpc;
        if cores_per_row == 0 {
            return;
        }
        let indent = " ".repeat(LABEL_WIDTH);
        for (row_idx, chunk) in heatmap_spans.chunks(cores_per_row).enumerate() {
            let y = area.y + 1 + row_idx as u16;
            if y >= area.y + area.height {
                break;
            }
            let mut row_spans: Vec<Span<'static>> = vec![Span::raw(indent.clone())];
            for s in chunk {
                row_spans.push(s.clone());
            }
            let wrap_rect = Rect::new(area.x, y, area.width, 1);
            f.render_widget(Paragraph::new(Line::from(row_spans)), wrap_rect);
        }
    }
}
