use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::widgets::{Axis, Block, Borders, Chart, Dataset, GraphType};
use ratatui::Frame;

use crate::metrics::DataPoint;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_rate_chart(f, chunks[0], data, "Download", |d| d.net_rx_rate);
    render_rate_chart(f, chunks[1], data, "Upload", |d| d.net_tx_rate);
}

fn render_rate_chart(
    f: &mut Frame,
    area: Rect,
    data: &[DataPoint],
    label: &str,
    extract: fn(&DataPoint) -> f64,
) {
    let chart_data: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, d)| (i as f64, extract(d)))
        .collect();

    let x_max = if data.is_empty() {
        1.0
    } else {
        (data.len() - 1) as f64
    };
    let y_max = chart_data.iter().map(|(_, v)| *v).fold(1.0_f64, f64::max);

    let y_label = format_y_label(y_max);

    let datasets = vec![Dataset::default()
        .name(label)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme::GRAPH_PRIMARY))
        .data(&chart_data)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(format!(" Net {label} "))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .x_axis(Axis::default().bounds([0.0, x_max]))
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(vec!["0", &y_label]),
        );
    f.render_widget(chart, area);
}

fn format_y_label(max_bytes: f64) -> String {
    if max_bytes >= 1_048_576.0 {
        format!("{:.1}MB/s", max_bytes / 1_048_576.0)
    } else {
        format!("{:.0}KB/s", max_bytes / 1024.0)
    }
}
