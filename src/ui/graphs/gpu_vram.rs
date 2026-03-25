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

    render_gpu_chart(f, chunks[0], data);
    render_vram_chart(f, chunks[1], data);
}

fn render_gpu_chart(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let gpu_data: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter_map(|(i, d)| d.gpu_percent.map(|v| (i as f64, v)))
        .collect();

    let temp_data: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter_map(|(i, d)| d.gpu_temp.map(|t| (i as f64, t)))
        .collect();

    let x_max = if data.is_empty() {
        1.0
    } else {
        (data.len() - 1) as f64
    };

    let mut datasets = vec![Dataset::default()
        .name("GPU %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme::GRAPH_PRIMARY))
        .data(&gpu_data)];

    if !temp_data.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Temp \u{00b0}C")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme::GRAPH_SECONDARY))
                .data(&temp_data),
        );
    }

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" GPU ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .x_axis(Axis::default().bounds([0.0, x_max]))
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .labels(vec!["0", "50", "100"]),
        );
    f.render_widget(chart, area);
}

fn render_vram_chart(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let vram_data: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter_map(|(i, d)| d.vram_percent.map(|v| (i as f64, v)))
        .collect();

    let x_max = if data.is_empty() {
        1.0
    } else {
        (data.len() - 1) as f64
    };

    let datasets = vec![Dataset::default()
        .name("VRAM %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme::GRAPH_PRIMARY))
        .data(&vram_data)];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(" VRAM ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BORDER_COLOR)),
        )
        .x_axis(Axis::default().bounds([0.0, x_max]))
        .y_axis(
            Axis::default()
                .bounds([0.0, 100.0])
                .labels(vec!["0", "50", "100"]),
        );
    f.render_widget(chart, area);
}
