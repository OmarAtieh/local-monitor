use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else {
        format!("{}h {}m", hours, minutes)
    }
}

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let uptime = format_uptime(sample.uptime_secs);
    let line = Line::from(vec![
        Span::styled(
            " LocalMonitor ",
            Style::default().fg(theme::TITLE_COLOR).bold(),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Up: {uptime}"),
            Style::default().fg(theme::LABEL_COLOR),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Procs: {}", sample.process_count),
            Style::default().fg(theme::LABEL_COLOR),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme::BORDER_COLOR));
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
