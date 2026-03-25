use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
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

/// Render header content into the given area (no borders — parent draws the box).
pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let uptime = format_uptime(sample.uptime_secs);
    let now = chrono::Local::now();
    let offset_secs = now.offset().local_minus_utc();
    let offset_hours = offset_secs / 3600;
    let tz_str = if offset_hours >= 0 {
        format!("UTC+{offset_hours}")
    } else {
        format!("UTC{offset_hours}")
    };
    let datetime = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let right_text = format!(
        "Up: {} \u{2502} Procs: {} \u{2502} {} {}",
        uptime, sample.process_count, datetime, tz_str
    );

    // Build line: title left, right info right-aligned via padding
    let title_len = "LocalMonitor".len();
    let right_len = right_text.len();
    let inner_width = area.width as usize;
    let padding = inner_width.saturating_sub(title_len + right_len + 1);

    let line = Line::from(vec![
        Span::styled(
            "LocalMonitor",
            Style::default().fg(theme::TITLE_COLOR).bold(),
        ),
        Span::raw(" ".repeat(padding)),
        Span::styled(right_text, Style::default().fg(theme::DETAIL_COLOR)),
    ]);

    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
