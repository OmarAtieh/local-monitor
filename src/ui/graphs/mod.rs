pub mod cpu_ram;
pub mod disk;
pub mod gpu_vram;
pub mod network;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::GraphFocus;
use crate::metrics::{DataPoint, Granularity};
use crate::ui::theme;

pub fn render_graphs(
    f: &mut Frame,
    area: Rect,
    focus: GraphFocus,
    granularity: Granularity,
    data: &[DataPoint],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(4),    // graph area
            Constraint::Length(1), // controls bar
        ])
        .split(area);

    // Title bar
    let title_line = Line::from(vec![
        Span::styled(
            format!(" \u{25b6} {} ", focus.label()),
            Style::default().fg(theme::TITLE_COLOR).bold(),
        ),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", granularity.label()),
            Style::default().fg(theme::LABEL_COLOR),
        ),
    ]);
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Graph area
    if data.is_empty() {
        let msg =
            Paragraph::new("Collecting data...").style(Style::default().fg(theme::LABEL_COLOR));
        f.render_widget(msg, chunks[1]);
    } else {
        match focus {
            GraphFocus::CpuRam => cpu_ram::render(f, chunks[1], data),
            GraphFocus::GpuVram => gpu_vram::render(f, chunks[1], data),
            GraphFocus::DiskIo => disk::render(f, chunks[1], data),
            GraphFocus::Network => network::render(f, chunks[1], data),
        }
    }

    // Controls bar
    let controls = Line::from(vec![Span::styled(
        " \u{25c0}\u{25b6}/AD: view   \u{25b2}\u{25bc}/WS: granularity   Q: quit",
        Style::default().fg(theme::LABEL_COLOR),
    )]);
    f.render_widget(Paragraph::new(controls), chunks[2]);
}
