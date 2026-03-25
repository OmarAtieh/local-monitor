# LocalMonitor — Design Spec

## Overview

A lightweight, terminal-based system monitor for Windows that displays real-time hardware metrics with historical graphing. Ships as a single executable with an auto-created SQLite database for persistence.

## Tech Stack

- **Language:** Rust
- **TUI:** Ratatui + Crossterm
- **Storage:** SQLite via rusqlite
- **System metrics:** sysinfo crate
- **GPU metrics:** nvml-wrapper crate (NVIDIA)

## Architecture

Modular, trait-based design for easy extensibility.

### Core Traits

```rust
/// A metric collector that samples hardware data
trait Collector {
    fn name(&self) -> &str;
    fn collect(&mut self) -> Vec<Sample>;
    fn is_available(&self) -> bool;
}

/// A HUD panel that renders live metrics
trait HudPanel {
    fn name(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect, data: &MetricStore);
}

/// A graph view (two stacked charts) for historical data
trait GraphView {
    fn name(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect, data: &MetricStore, granularity: Granularity);
}
```

### Module Structure

```
src/
  main.rs              — app entry, event loop
  app.rs               — app state, view/granularity cycling
  db.rs                — SQLite storage, aggregation, pruning
  metrics.rs           — Sample type, MetricStore trait
  collectors/
    mod.rs             — Collector trait, registry
    cpu.rs             — CPU collector (sysinfo)
    ram.rs             — RAM collector (sysinfo)
    gpu.rs             — GPU/VRAM collector (nvml-wrapper)
    disk.rs            — Disk IO collector (sysinfo)
    network.rs         — Network collector (sysinfo)
  ui/
    mod.rs             — layout, render loop
    hud/
      mod.rs           — HudPanel trait, panel registry
      cpu.rs           — CPU HUD panel
      ram.rs           — RAM HUD panel
      gpu.rs           — GPU/VRAM HUD panel
      disk.rs          — Disk HUD panel
      network.rs       — Network HUD panel
      system.rs        — Uptime/process count header
    graphs/
      mod.rs           — GraphView trait, view registry
      cpu_ram.rs       — CPU & RAM graph view
      gpu_vram.rs      — GPU & VRAM graph view
      disk.rs          — Disk IO graph view
      network.rs       — Network graph view
    theme.rs           — color scheme, thresholds
```

### Adding a New Monitor

To add a new metric (e.g., audio devices, battery):
1. Add a new collector in `collectors/` implementing `Collector`
2. Register it in the collector registry
3. Add a HUD panel in `ui/hud/` implementing `HudPanel`
4. Optionally add a graph view in `ui/graphs/` implementing `GraphView`
5. The DB schema auto-extends via the sample type

Collectors that fail `is_available()` at startup are silently skipped — their HUD panels show "N/A" and their graph views are omitted from the cycle.

## Metrics Collected

| Category | Metrics |
|----------|---------|
| CPU | Per-core utilization %, overall %, frequency (GHz), temperature (best-effort, see notes) |
| RAM | Used/total (GB), utilization %, pagefile used/total (GB) |
| GPU | Utilization %, temperature (°C), fan speed (RPM), clock speed (MHz) |
| VRAM | Used/total (GB), utilization % |
| Disk | Per-drive capacity used/total, read/write IO rates (MB/s, computed by diffing consecutive samples) |
| Network | Upload/download rates (auto-scaled: KB/s when <1 MB/s, MB/s otherwise), total bytes transferred |
| System | System uptime (time since last boot), process count |

**Sampling rate:** 1 second

### Notes

- **CPU temperature:** The `sysinfo` crate has limited temperature support on Windows. Attempt to read it; if unavailable, display "N/A" in the HUD and omit from graphs. No fallback crate — keep it simple.
- **GPU fallback:** If NVML initialization fails (no NVIDIA GPU, no drivers), GPU/VRAM sections in the HUD show "N/A" and the GPU & VRAM graph view is skipped.
- **Pagefile:** Windows uses a pagefile, not swap. The HUD label says "Swap" for brevity but reads the Windows pagefile stats.
- **Disk IO rates:** Computed as `(current_bytes - previous_bytes) / interval`. Not a direct API return.
- **Multiple disks:** HUD shows the system drive (C:) only. Other drives are omitted to keep the HUD compact.

## Layout

Single screen, two sections:

### Top: Always-Visible HUD

All metrics displayed simultaneously with colored progress bars and numeric values.

```
┌─────────────────────────────────────────────────────────────────┐
│  LocalMonitor                    Uptime: 3d 14h    Procs: 312  │
├─────────────────────────────────┬───────────────────────────────┤
│  CPU  ██████████░░░░░░ 74%      │  GPU  ████████░░░░░░░░ 52%   │
│  3.8 GHz  58°C                  │  45°C  Fan: 1200 RPM         │
│  Cores: 82 71 65 90 44 78 63 55 │  Clock: 1650 MHz             │
├─────────────────────────────────┼───────────────────────────────┤
│  RAM  ██████████░░░░░░ 62%      │  VRAM ██████░░░░░░░░░░ 38%   │
│  12.4 / 16.0 GB                 │  4.2 / 8.0 GB                │
│  Swap: 1.2 / 8.0 GB             │                               │
├─────────────────────────────────┼───────────────────────────────┤
│  Disk C: ████████████░░░░ 78%   │  Net  ↓ 2.4 MB/s  ↑ 0.3 MB/s│
│  R: 45 MB/s  W: 12 MB/s         │  Total: ↓1.2 GB  ↑340 MB    │
├─────────────────────────────────┴───────────────────────────────┤
```

### Bottom: Focus Graphs

Two vertically stacked, full-width graphs that change based on the selected view. Each graph has a legend.

```
│  ► CPU & RAM                                              [1m] │
├─────────────────────────────────────────────────────────────────┤
│  CPU %                                                          │
│ 100│                 ╭╮                                         │
│  75│  ╭──────╮  ╭───╯╰──╮                                      │
│  50│╭─╯      ╰──╯       ╰──╮    ╭──╮                           │
│  25│╯                      ╰────╯  ╰──────                     │
│   0│────────────────────────────────────────────────            │
│  ── CPU %  ── Temp                                              │
├─────────────────────────────────────────────────────────────────┤
│  RAM %                                                          │
│ 100│                                                            │
│  75│         ╭────╮                                             │
│  50│─────────╯    ╰───╮  ╭──────────────────                   │
│  25│                   ╰──╯                                     │
│   0│────────────────────────────────────────────────            │
│  ── RAM %  ── Swap %                                            │
├─────────────────────────────────────────────────────────────────┤
│  ◀▶/AD: view   ▲▼/WS: granularity   Q: quit                    │
└─────────────────────────────────────────────────────────────────┘
```

**4 graph views** (cycle with ←→ or A/D):
1. CPU & RAM — CPU utilization + temperature overlay / RAM + swap utilization
2. GPU & VRAM — GPU utilization + temperature overlay / VRAM utilization (skipped if no NVIDIA GPU)
3. Disk IO — Read rate / Write rate
4. Network — Download rate / Upload rate

## Time Granularities

11 levels (cycle with ↑↓ or W/S):
1m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 24h, 3d, 7d

Only granularities with available data are selectable (e.g., 7d is only available after 7 days of collection).

## Color Scheme

**Utilization bars and graph lines:**
- Green: <60%
- Yellow: 60–80%
- Red: >80%

**Temperature:**
- Green: <60°C
- Yellow: 60–80°C
- Red: >80°C

**Graph lines:** Distinct colors per metric within each graph, identified by legend.

## Data Storage

SQLite database auto-created at `%LOCALAPPDATA%\LocalMonitor\localmonitor.db` (avoids write permission issues in Program Files).

### Schema

```sql
-- Raw 1-second samples (retained: 1 minute)
CREATE TABLE samples_1s (
    ts INTEGER NOT NULL,           -- unix timestamp
    cpu_percent REAL,
    cpu_temp REAL,
    ram_percent REAL,
    swap_percent REAL,
    gpu_percent REAL,
    gpu_temp REAL,
    vram_percent REAL,
    disk_read_bytes INTEGER,
    disk_write_bytes INTEGER,
    net_rx_bytes INTEGER,
    net_tx_bytes INTEGER
);

-- Downsampled tables share the same schema
-- samples_5s  (retained: 30 minutes)
-- samples_30s (retained: 4 hours)
-- samples_5m  (retained: 24 hours)
-- samples_15m (retained: 7 days)
```

### Resolution-to-Granularity Mapping

| Graph Granularity | Query Resolution | Data Points (approx) |
|-------------------|-----------------|---------------------|
| 1m | 1s | 60 |
| 5m | 5s | 60 |
| 15m | 5s | 180 |
| 30m | 5s | 360 |
| 1h | 30s | 120 |
| 2h | 30s | 240 |
| 4h | 30s | 480 |
| 8h | 5m | 96 |
| 24h | 5m | 288 |
| 3d | 15m | 288 |
| 7d | 15m | 672 |

Data points are downsampled to fit the available graph width (one point per column).

Background aggregation and pruning runs every 60 seconds.

## Controls

| Key | Action |
|-----|--------|
| ←/A | Previous graph view |
| →/D | Next graph view |
| ↑/W | Longer time granularity |
| ↓/S | Shorter time granularity |
| Q / Ctrl+C | Quit (restores terminal state cleanly) |

## Edge Cases

- **First launch / empty database:** Graphs show "Collecting data..." until enough samples exist for the selected granularity.
- **Terminal too small:** Show a centered message "Terminal too small — resize to at least 80x24" instead of the dashboard.
- **Minimum terminal size:** 80 columns x 24 rows.
- **SQLite errors:** If the database cannot be opened or written to, run in display-only mode (live metrics, no history graphs) and show a warning banner.
- **Ctrl+C handling:** Crossterm's cleanup restores the terminal to its original state on any exit path.

## Non-Goals

- No remote monitoring or network server
- No alerting or notifications
- No configuration file (zero-config)
- No process-level breakdown (just system totals)

## Build Output

Single `.exe` binary via `cargo build --release`. No installer needed.
