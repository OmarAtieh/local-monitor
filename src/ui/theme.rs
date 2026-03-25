use ratatui::style::Color;

pub const GRAPH_PRIMARY: Color = Color::Cyan;
pub const GRAPH_SECONDARY: Color = Color::Magenta;
pub const BORDER_COLOR: Color = Color::DarkGray;
pub const TITLE_COLOR: Color = Color::White;
pub const LABEL_COLOR: Color = Color::Gray;

pub fn utilization_color(percent: f64) -> Color {
    if percent > 80.0 {
        Color::Red
    } else if percent >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

#[allow(dead_code)]
pub fn temp_color(temp: f64) -> Color {
    if temp > 80.0 {
        Color::Red
    } else if temp >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
