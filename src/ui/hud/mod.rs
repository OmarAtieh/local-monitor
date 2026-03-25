pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod ram;
pub mod system;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::metrics::Sample;
use crate::ui::theme;

/// Calculate total HUD height based on core count and terminal width.
pub fn hud_height(core_count: usize, terminal_width: u16) -> u16 {
    // top(1) + header(1) + div(1) + cpu(1) + div(1) + gpu/vrm(1) + div(1)
    // + ram/swp(1) + div(1) + dsk/net(1) + bottom(1) = 11
    let base_height: u16 = 11;
    let extra = cpu::heatmap_extra_rows(core_count, terminal_width);
    base_height + extra
}

/// Draw a horizontal line of a given character across the buffer.
fn draw_hline(buf: &mut Buffer, x: u16, y: u16, width: u16, ch: &str, style: Style) {
    for col in 0..width {
        if x + col < buf.area().right() && y < buf.area().bottom() {
            buf.set_string(x + col, y, ch, style);
        }
    }
}

pub fn render_hud(f: &mut Frame, area: Rect, sample: &Sample) {
    if area.height < 7 || area.width < 20 {
        return;
    }

    let buf = f.buffer_mut();
    let border_style = Style::default().fg(theme::BORDER_COLOR);
    let x = area.x;
    let w = area.width;
    let inner_w = w.saturating_sub(2); // inside the box (between │ ... │)

    // Calculate row Y positions
    let extra_cpu_rows = cpu::heatmap_extra_rows(sample.per_core_percent.len(), area.width);
    let mut y = area.y;

    // ╭─────╮ top border
    buf.set_string(x, y, "\u{256D}", border_style);
    draw_hline(buf, x + 1, y, inner_w, "\u{2500}", border_style);
    buf.set_string(x + w - 1, y, "\u{256E}", border_style);
    y += 1;

    // │ header │
    buf.set_string(x, y, "\u{2502}", border_style);
    buf.set_string(x + w - 1, y, "\u{2502}", border_style);
    let header_rect = Rect::new(x + 2, y, inner_w.saturating_sub(2), 1);
    y += 1;

    // ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤ dotted divider
    buf.set_string(x, y, "\u{251C}", border_style);
    draw_hline(buf, x + 1, y, inner_w, "\u{254C}", border_style);
    buf.set_string(x + w - 1, y, "\u{2524}", border_style);
    y += 1;

    // │ CPU row(s) │
    let cpu_rows = 1 + extra_cpu_rows;
    for row in 0..cpu_rows {
        buf.set_string(x, y + row, "\u{2502}", border_style);
        buf.set_string(x + w - 1, y + row, "\u{2502}", border_style);
    }
    let cpu_rect = Rect::new(x + 2, y, inner_w.saturating_sub(2), cpu_rows);
    y += cpu_rows;

    // ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┬╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤ paired divider
    let left_half = inner_w / 2;
    let right_half = inner_w - left_half;

    buf.set_string(x, y, "\u{251C}", border_style);
    draw_hline(buf, x + 1, y, left_half, "\u{254C}", border_style);
    buf.set_string(x + 1 + left_half, y, "\u{252C}", border_style);
    draw_hline(
        buf,
        x + 2 + left_half,
        y,
        right_half.saturating_sub(1),
        "\u{254C}",
        border_style,
    );
    buf.set_string(x + w - 1, y, "\u{2524}", border_style);
    y += 1;

    // Draw 3 paired rows: GPU/VRM, RAM/SWP, DSK/NET
    for i in 0..3u16 {
        // │ left │ right │
        buf.set_string(x, y, "\u{2502}", border_style);
        buf.set_string(x + 1 + left_half, y, "\u{2502}", border_style);
        buf.set_string(x + w - 1, y, "\u{2502}", border_style);
        y += 1;

        // Divider between paired rows (not after the last)
        if i < 2 {
            buf.set_string(x, y, "\u{251C}", border_style);
            draw_hline(buf, x + 1, y, left_half, "\u{254C}", border_style);
            buf.set_string(x + 1 + left_half, y, "\u{253C}", border_style);
            draw_hline(
                buf,
                x + 2 + left_half,
                y,
                right_half.saturating_sub(1),
                "\u{254C}",
                border_style,
            );
            buf.set_string(x + w - 1, y, "\u{2524}", border_style);
            y += 1;
        }
    }

    // ╰──────┴──────╯ bottom border
    buf.set_string(x, y, "\u{2570}", border_style);
    draw_hline(buf, x + 1, y, left_half, "\u{2500}", border_style);
    buf.set_string(x + 1 + left_half, y, "\u{2534}", border_style);
    draw_hline(
        buf,
        x + 2 + left_half,
        y,
        right_half.saturating_sub(1),
        "\u{2500}",
        border_style,
    );
    buf.set_string(x + w - 1, y, "\u{256F}", border_style);

    // Now render content via Frame (drop buf reference by ending the block above).
    // We need to recalculate positions since we can't hold buf and call panel fns.
    // Recalculate content rects:
    system::render(f, header_rect, sample);
    cpu::render(f, cpu_rect, sample);

    let left_content_w = left_half.saturating_sub(1);
    let right_content_w = right_half.saturating_sub(2);

    // GPU/VRM row
    let gpu_y = area.y + 3 + cpu_rows + 1;
    let gpu_left = Rect::new(x + 2, gpu_y, left_content_w, 1);
    let gpu_right = Rect::new(x + 2 + left_half, gpu_y, right_content_w, 1);
    gpu::render_gpu(f, gpu_left, sample);
    gpu::render_vram(f, gpu_right, sample);

    // RAM/SWP row
    let ram_y = gpu_y + 2;
    let ram_left = Rect::new(x + 2, ram_y, left_content_w, 1);
    let ram_right = Rect::new(x + 2 + left_half, ram_y, right_content_w, 1);
    ram::render_ram(f, ram_left, sample);
    ram::render_swap(f, ram_right, sample);

    // DSK/NET row
    let dsk_y = ram_y + 2;
    let dsk_left = Rect::new(x + 2, dsk_y, left_content_w, 1);
    let net_right = Rect::new(x + 2 + left_half, dsk_y, right_content_w, 1);
    disk::render(f, dsk_left, sample);
    network::render(f, net_right, sample);
}
