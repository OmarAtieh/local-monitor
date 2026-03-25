# LocalMonitor

A lightweight, terminal-based system monitor for Windows. Single executable, zero configuration.

## What It Monitors

- **CPU** — per-core utilization, frequency, temperature
- **RAM** — usage, swap
- **GPU** — utilization, temperature, fan speed, clock (NVIDIA)
- **VRAM** — usage
- **Disk** — capacity, read/write IO rates
- **Network** — upload/download speeds, total transferred
- **System** — uptime, process count

## Preview

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

## Usage

```bash
# Just run it
localmonitor.exe
```

That's it. No flags, no config files. The database (`localmonitor.db`) is created automatically next to the executable.

## Controls

| Key | Action |
|-----|--------|
| `←` / `A` | Previous graph view |
| `→` / `D` | Next graph view |
| `↑` / `W` | Longer time granularity |
| `↓` / `S` | Shorter time granularity |
| `Q` | Quit |

## Graph Views

1. **CPU & RAM** — CPU utilization + temperature / RAM + swap
2. **GPU & VRAM** — GPU utilization + temperature / VRAM usage
3. **Disk IO** — Read rate / Write rate
4. **Network** — Download / Upload speeds

## Time Granularities

1m, 5m, 15m, 30m, 1h, 2h, 4h, 8h, 24h, 3d, 7d

History is available based on how long the monitor has been collecting data.

## Building from Source

Requires Rust toolchain and NVIDIA drivers (for GPU monitoring).

```bash
cargo build --release
```

Binary output: `target/release/localmonitor.exe`

## Requirements

- Windows 10/11
- NVIDIA GPU with drivers installed (for GPU/VRAM metrics)
