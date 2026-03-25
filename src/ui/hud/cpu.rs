use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

const BLOCK_CHARS: &[char] = &[
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];
const LABEL_WIDTH: usize = 5; // "CPU " + space after bar info

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

/// Calculate how many cores fit inline on the CPU row after label + bar + pct + freq.
pub fn heatmap_extra_rows(core_count: usize, terminal_width: u16) -> u16 {
    if core_count == 0 {
        return 0;
    }
    let inner_width = (terminal_width as usize).saturating_sub(4); // box borders + padding
                                                                   // " CPU " (5) + bar (variable) + " XX% " (5) + " X.XX GHz  " (12) = 22 fixed chars minimum
                                                                   // bar takes up space but cores go after the freq text
                                                                   // Estimate: label(5) + pct(5) + freq(12) + 2 spaces = ~24 chars overhead
                                                                   // Available for cores inline = inner_width - 24
    let cores_per_row = inner_width.saturating_sub(LABEL_WIDTH + 19); // 19 = pct + freq + spacing
    if cores_per_row == 0 {
        // All cores wrap
        let per_row = inner_width.saturating_sub(LABEL_WIDTH);
        if per_row == 0 {
            return 0;
        }
        return core_count.div_ceil(per_row) as u16;
    }
    if core_count <= cores_per_row {
        return 0; // fits inline
    }
    let remaining = core_count - cores_per_row;
    let per_row = inner_width.saturating_sub(LABEL_WIDTH);
    if per_row == 0 {
        return 0;
    }
    remaining.div_ceil(per_row) as u16
}

/// Render the CPU row content (no borders). May use multiple rows for heatmap wrapping.
pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height == 0 || area.width < 20 {
        return;
    }

    let pct = sample.cpu_percent.clamp(0.0, 100.0);
    let freq_ghz = sample.cpu_freq_mhz / 1000.0;
    let inner_w = area.width as usize;

    // Fixed parts: "CPU " (4) + " " (1 after bar) + pct "XXX% " (5) + freq "X.XX GHz" (9)
    let label_str = "CPU ";
    let pct_str = format!("{:>3.0}%", pct);
    let freq_str = format!("{:.2} GHz", freq_ghz);
    let fixed_len = label_str.len() + 1 + pct_str.len() + 1 + freq_str.len();

    // Build core heatmap spans
    let core_spans: Vec<Span<'static>> = sample
        .per_core_percent
        .iter()
        .map(|&p| {
            let ch = heatmap_char(p);
            Span::styled(
                String::from(ch),
                Style::default().fg(theme::utilization_color(p)),
            )
        })
        .collect();

    let core_count = sample.per_core_percent.len();
    // Space available for cores inline (after 2 spaces separator)
    let inline_space = inner_w.saturating_sub(fixed_len + 2);
    let cores_inline = core_count.min(inline_space);

    // Determine bar width: fill the space between label and pct
    // Layout: "CPU " + bar + " " + pct + " " + freq + "  " + inline_cores
    let after_bar = 1
        + pct_str.len()
        + 1
        + freq_str.len()
        + if cores_inline > 0 {
            2 + cores_inline
        } else {
            0
        };
    let bar_width = inner_w.saturating_sub(label_str.len() + after_bar).max(4);

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

    // Inline cores
    if cores_inline > 0 {
        spans.push(Span::raw("  "));
        for s in core_spans.iter().take(cores_inline) {
            spans.push(s.clone());
        }
    }

    let line = Line::from(spans);
    let row_rect = Rect::new(area.x, area.y, area.width, 1);
    f.render_widget(Paragraph::new(line), row_rect);

    // Wrap remaining cores to additional rows
    if cores_inline < core_count && area.height > 1 {
        let remaining = &core_spans[cores_inline..];
        let wrap_width = inner_w.saturating_sub(LABEL_WIDTH);
        if wrap_width == 0 {
            return;
        }
        let indent = " ".repeat(LABEL_WIDTH);
        for (row_idx, chunk) in remaining.chunks(wrap_width).enumerate() {
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
