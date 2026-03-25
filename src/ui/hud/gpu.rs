use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

pub fn render_gpu(f: &mut Frame, area: Rect, sample: &Sample) {
    let block = Block::default()
        .title(" GPU ")
        .title_style(Style::default().fg(theme::TITLE_COLOR))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR));

    let inner = block.inner(area);
    f.render_widget(block, area);

    match sample.gpu_percent {
        Some(pct) => {
            let pct = pct.clamp(0.0, 100.0);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1)])
                .split(inner);

            let mut parts = vec![format!("{pct:.0}%")];
            if let Some(temp) = sample.gpu_temp {
                parts.push(format!("{temp:.0}\u{00b0}C"));
            }
            if let Some(fan) = sample.gpu_fan_percent {
                parts.push(format!("Fan {fan}%"));
            }
            if let Some(clock) = sample.gpu_clock_mhz {
                parts.push(format!("{clock} MHz"));
            }
            let label = parts.join("  ");

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(theme::utilization_color(pct)))
                .ratio(pct / 100.0)
                .label(label);
            f.render_widget(gauge, chunks[0]);
        }
        None => {
            let paragraph = Paragraph::new("N/A").style(Style::default().fg(theme::LABEL_COLOR));
            f.render_widget(paragraph, inner);
        }
    }
}

pub fn render_vram(f: &mut Frame, area: Rect, sample: &Sample) {
    let block = Block::default()
        .title(" VRAM ")
        .title_style(Style::default().fg(theme::TITLE_COLOR))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER_COLOR));

    let inner = block.inner(area);
    f.render_widget(block, area);

    match sample.vram_percent {
        Some(pct) => {
            let pct = pct.clamp(0.0, 100.0);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1)])
                .split(inner);

            let used_gb = sample.vram_used_bytes.map(bytes_to_gb).unwrap_or(0.0);
            let total_gb = sample.vram_total_bytes.map(bytes_to_gb).unwrap_or(0.0);
            let label = format!("{pct:.0}%  {used_gb:.1}/{total_gb:.1} GB");

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(theme::utilization_color(pct)))
                .ratio(pct / 100.0)
                .label(label);
            f.render_widget(gauge, chunks[0]);
        }
        None => {
            let paragraph = Paragraph::new("N/A").style(Style::default().fg(theme::LABEL_COLOR));
            f.render_widget(paragraph, inner);
        }
    }
}
