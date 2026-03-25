use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let block = Block::default()
        .title(" CPU ")
        .title_style(Style::default().fg(theme::TITLE_COLOR))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    // Gauge row — overall CPU with frequency and temp
    let pct = sample.cpu_percent.clamp(0.0, 100.0);
    let freq_ghz = sample.cpu_freq_mhz / 1000.0;
    let mut label_parts = vec![format!("{pct:.0}%  {freq_ghz:.2} GHz")];
    if let Some(temp) = sample.cpu_temp {
        label_parts.push(format!("{temp:.0}\u{00b0}C"));
    }
    let label = label_parts.join("  ");

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(theme::utilization_color(pct)))
        .ratio(pct / 100.0)
        .label(label);
    f.render_widget(gauge, chunks[0]);

    // Per-core row — mini bar per core: "C0:▇74 C1:▇82 ..."
    if chunks[1].height > 0 {
        let core_spans: Vec<Span> = sample
            .per_core_percent
            .iter()
            .enumerate()
            .flat_map(|(i, &p)| {
                let color = theme::utilization_color(p);
                let bar_len = ((p / 100.0) * 5.0).round() as usize;
                let bar: String = "\u{2587}".repeat(bar_len);
                let empty: String = " ".repeat(5_usize.saturating_sub(bar_len));
                let sep = if i > 0 { " " } else { "" };
                vec![
                    Span::raw(format!("{sep}C{i}:")),
                    Span::styled(bar, Style::default().fg(color)),
                    Span::raw(format!("{empty}{p:2.0}")),
                ]
            })
            .collect();

        let cores_line = Line::from(core_spans);
        let paragraph = Paragraph::new(cores_line);
        f.render_widget(paragraph, chunks[1]);
    }
}
