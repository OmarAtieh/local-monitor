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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Gauge row
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

    // Per-core row
    let core_strs: Vec<Span> = sample
        .per_core_percent
        .iter()
        .enumerate()
        .flat_map(|(i, &p)| {
            let sep = if i > 0 { " " } else { "" };
            vec![
                Span::raw(sep),
                Span::styled(
                    format!("{p:.0}%"),
                    Style::default().fg(theme::utilization_color(p)),
                ),
            ]
        })
        .collect();

    let cores_line = Line::from(core_strs);
    let paragraph = Paragraph::new(cores_line);
    f.render_widget(paragraph, chunks[1]);
}
