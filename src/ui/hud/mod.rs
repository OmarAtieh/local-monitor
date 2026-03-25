pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod network;
pub mod ram;
pub mod system;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::metrics::Sample;

pub fn render_hud(f: &mut Frame, area: Rect, sample: &Sample) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // system header
            Constraint::Length(3), // CPU | GPU
            Constraint::Length(3), // RAM | VRAM
            Constraint::Length(2), // Disk | Network
        ])
        .split(area);

    // Row 0: System header
    system::render(f, rows[0], sample);

    // Row 1: CPU | GPU
    let row1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);
    cpu::render(f, row1[0], sample);
    gpu::render_gpu(f, row1[1], sample);

    // Row 2: RAM | VRAM
    let row2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);
    ram::render(f, row2[0], sample);
    gpu::render_vram(f, row2[1], sample);

    // Row 3: Disk | Network
    let row3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[3]);
    disk::render(f, row3[0], sample);
    network::render(f, row3[1], sample);
}
