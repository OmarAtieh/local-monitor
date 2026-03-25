# HUD Redesign — Design Spec

## Overview

Redesign the HUD section of LocalMonitor to use a table-based layout with consistent alignment, adaptive core heatmap, distinct label/load color palettes, and local datetime in the header.

## Layout Structure

Rounded box (`╭╮╰╯`) with dotted internal dividers (`╌`). CPU gets a full-width row. GPU/VRM, RAM/SWP, and DSK/NET are paired side-by-side separated by `│`.

```
╭──────────────────────────────────────────────────────────────────────────────────────────────╮
│ LocalMonitor           Up: 3d 14h │ Procs: 312 │ 2026-03-25 12:34:56 UTC+3                  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ CPU ████████████████████████████████████████████ 55%  2.20 GHz  ▃▅▇█▆▃▁▂▅▇█▄▂▁▃▆            │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┬╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ GPU █████████████████░░ 92% 74°C 68% 1815M     │ VRM ████████████████░ 97% 7.8/8.0 GB        │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ RAM ███████████████░░░░ 78% 24.8/31.9 GB        │ SWP ███░░░░░░░░░░░░░ 15% 1.2/8.0 GB        │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ DSK ██████████████░░░░░░ 72% R:45 W:12 MB/s     │ NET ↓2.4 ↑0.3 MB/s ↓1.2G ↑340M            │
╰──────────────────────────────────────────────┴───────────────────────────────────────────────╯
```

## Rows

1. **Header** — "LocalMonitor" left, "Up: Xd Yh │ Procs: N │ YYYY-MM-DD HH:MM:SS TZ" right
2. **CPU** — Full width. Bar + percentage + frequency + inline core heatmap (if fits). Wraps heatmap to additional rows if core count exceeds available space.
3. **GPU │ VRM** — Paired. Each has bar + % + details (GPU: temp, fan%, clock; VRM: used/total).
4. **RAM │ SWP** — Paired. Each has bar + % + used/total.
5. **DSK │ NET** — Paired. DSK: bar + % + R/W rates. NET: download/upload rates + cumulative totals.

## CPU Core Heatmap

- One character per core using block elements: `▁▂▃▄▅▆▇█`
- Each character is colored by that core's load (green/yellow/red)
- **Inline behavior**: After the frequency value on the CPU row, if all cores fit in the remaining space
- **Wrap behavior**: If cores exceed remaining space on the CPU row, they wrap to dedicated row(s) below, indented to align with the bar start position
- The breakpoint is dynamic based on terminal width and number of cores
- Wrapping continues to as many rows as needed (e.g., 128 cores may need 3 rows on an 80-char terminal)

## Color System

### Label Colors (fixed, per-subsystem identity)

| Subsystem | Color | Hex |
|-----------|-------|-----|
| CPU | Teal | #2dd4bf |
| GPU / VRM | Purple | #c084fc |
| RAM / SWP | Orange | #fb923c |
| DSK / NET | Pink | #f472b6 |

These colors are **never** green, yellow, or red — they must remain visually distinct from load colors at all times.

### Load Colors (dynamic, based on utilization)

| Range | Color | Hex |
|-------|-------|-----|
| < 60% | Green | #4ade80 |
| 60–80% | Yellow | #eab308 |
| > 80% | Red | #ef4444 |

Used for: utilization bars, percentage text, core heatmap blocks, temperature values.

### Other Colors

| Element | Color | Hex |
|---------|-------|-----|
| Unused bar fill | Dark grey | #21262d |
| Detail text (GHz, GB) | Muted grey | #7d8590 |
| Box borders & dividers | Dim grey | #484f58 |
| App title | Bright white | #e6edf3 |

## Bar Rendering

- Bars use `█` (U+2588) for filled and `█` in dark grey for unfilled
- Bar width is dynamic — stretches to fill available space between label and percentage columns
- Percentage is displayed immediately after the bar, load-colored
- Detail text (GHz, GB, rates) follows after percentage, muted grey

## Box Drawing

- Corners: `╭` `╮` `╰` `╯` (rounded)
- Horizontal: `─` for top/bottom frame, `╌` for internal dotted dividers
- Vertical: `│` for frame sides and pair separators
- Junctions: `├` `┤` `┬` `┴` `┼` as needed

## Header

- Left: "LocalMonitor" in bright white bold
- Right: "Up: Xd Yh │ Procs: N │ YYYY-MM-DD HH:MM:SS UTC±N" in muted grey
- Uses `chrono::Local::now()` for local time with timezone offset

## Dynamic HUD Height

The HUD height adapts based on core count:
- **≤16 cores** (fits inline): 7 lines (header + CPU + GPU/VRM + RAM/SWP + DSK/NET + frame lines)
- **17–80 cores** (1 wrap row): 8 lines
- **81–160 cores** (2 wrap rows): 9 lines
- Pattern: 7 + ceil((cores - space_after_ghz) / cores_per_row) extra rows when wrapping

The main.rs layout constraint adjusts accordingly.

## Files Changed

- `src/ui/hud/mod.rs` — New table layout with box drawing, dynamic height
- `src/ui/hud/cpu.rs` — Core heatmap with inline/wrap logic
- `src/ui/hud/gpu.rs` — Simplified paired rendering (bar + details)
- `src/ui/hud/ram.rs` — Simplified paired rendering
- `src/ui/hud/disk.rs` — Simplified paired rendering
- `src/ui/hud/network.rs` — Rate + totals text
- `src/ui/hud/system.rs` — Header with datetime
- `src/ui/theme.rs` — New label colors, load colors remain same
- `src/main.rs` — Dynamic HUD height constraint

## Non-Changes

- Graph views remain unchanged
- Collectors remain unchanged
- Database layer remains unchanged
- Controls remain unchanged
