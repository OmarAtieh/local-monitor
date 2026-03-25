use ratatui::style::Color;

// Graph colors (unchanged)
pub const GRAPH_PRIMARY: Color = Color::Cyan;
pub const GRAPH_SECONDARY: Color = Color::Magenta;

// HUD box drawing
pub const BORDER_COLOR: Color = Color::Rgb(72, 79, 88);
pub const TITLE_COLOR: Color = Color::Rgb(230, 237, 243);
pub const DETAIL_COLOR: Color = Color::Rgb(125, 133, 150);
pub const BAR_EMPTY: Color = Color::Rgb(33, 38, 45);

// Label colors — fixed per subsystem, never green/yellow/red
pub const LABEL_CPU: Color = Color::Rgb(45, 212, 191);
pub const LABEL_GPU: Color = Color::Rgb(192, 132, 252);
pub const LABEL_RAM: Color = Color::Rgb(251, 146, 60);
pub const LABEL_DSK: Color = Color::Rgb(244, 114, 182);

// Legacy aliases used by graph views
pub const LABEL_COLOR: Color = Color::Gray;

pub fn utilization_color(percent: f64) -> Color {
    if percent > 80.0 {
        Color::Rgb(239, 68, 68)
    } else if percent >= 60.0 {
        Color::Rgb(234, 179, 8)
    } else {
        Color::Rgb(74, 222, 128)
    }
}

#[allow(dead_code)]
pub fn temp_color(temp: f64) -> Color {
    if temp > 80.0 {
        Color::Rgb(239, 68, 68)
    } else if temp >= 60.0 {
        Color::Rgb(234, 179, 8)
    } else {
        Color::Rgb(74, 222, 128)
    }
}
