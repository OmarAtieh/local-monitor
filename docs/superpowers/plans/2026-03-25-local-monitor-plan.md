# LocalMonitor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a terminal-based system monitor that displays live hardware metrics with color-coded HUD and historical graphs, shipping as a single Windows executable.

**Architecture:** Modular trait-based Rust application. Collectors gather metrics → stored in SQLite → rendered by HUD panels (live) and graph views (historical). Event loop drives 1s sampling, user input, and 60s aggregation cycles.

**Tech Stack:** Rust, Ratatui, Crossterm, rusqlite (bundled), sysinfo, nvml-wrapper

**Spec:** `docs/superpowers/specs/2026-03-25-local-monitor-design.md`

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd C:/Users/omar-/Projects/LocalMonitor
cargo init --name localmonitor
```

- [ ] **Step 2: Set up Cargo.toml with all dependencies**

Replace `Cargo.toml` with:

```toml
[package]
name = "localmonitor"
version = "0.1.0"
edition = "2021"
description = "Lightweight terminal-based system monitor"

[dependencies]
ratatui = "0.29"
crossterm = "0.28"
rusqlite = { version = "0.32", features = ["bundled"] }
sysinfo = "0.33"
nvml-wrapper = "0.10"
dirs = "6"
chrono = "0.4"
anyhow = "1"

[profile.release]
opt-level = "z"
lto = true
strip = true
```

- [ ] **Step 3: Set up .gitignore**

```
/target
*.db
```

- [ ] **Step 4: Write minimal main.rs that compiles**

```rust
fn main() {
    println!("LocalMonitor starting...");
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully, downloads dependencies

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore
git commit -m "feat: project scaffolding with dependencies"
```

---

### Task 2: Core Types — Sample, Granularity, MetricStore

**Files:**
- Create: `src/metrics.rs`
- Modify: `src/main.rs` (add module declaration)

- [ ] **Step 1: Create `src/metrics.rs` with core types**

```rust
use std::time::SystemTime;

/// A single point-in-time snapshot of all system metrics.
#[derive(Debug, Clone, Default)]
pub struct Sample {
    pub ts: i64, // unix timestamp
    pub cpu_percent: f64,
    pub cpu_temp: Option<f64>,
    pub per_core_percent: Vec<f64>,
    pub cpu_freq_mhz: f64,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub ram_percent: f64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_percent: f64,
    pub gpu_percent: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub gpu_fan_rpm: Option<u32>,
    pub gpu_clock_mhz: Option<u32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub vram_percent: Option<f64>,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub process_count: usize,
    pub uptime_secs: u64,
}

/// Time granularities for graph views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    M1,
    M5,
    M15,
    M30,
    H1,
    H2,
    H4,
    H8,
    H24,
    D3,
    D7,
}

impl Granularity {
    pub const ALL: &[Granularity] = &[
        Self::M1, Self::M5, Self::M15, Self::M30,
        Self::H1, Self::H2, Self::H4, Self::H8, Self::H24,
        Self::D3, Self::D7,
    ];

    /// Total seconds this granularity covers.
    pub fn window_secs(&self) -> i64 {
        match self {
            Self::M1 => 60,
            Self::M5 => 300,
            Self::M15 => 900,
            Self::M30 => 1800,
            Self::H1 => 3600,
            Self::H2 => 7200,
            Self::H4 => 14400,
            Self::H8 => 28800,
            Self::H24 => 86400,
            Self::D3 => 259200,
            Self::D7 => 604800,
        }
    }

    /// Which DB table to query for this granularity.
    pub fn table_name(&self) -> &str {
        match self {
            Self::M1 => "samples_1s",
            Self::M5 | Self::M15 | Self::M30 => "samples_5s",
            Self::H1 | Self::H2 | Self::H4 => "samples_30s",
            Self::H8 | Self::H24 => "samples_5m",
            Self::D3 | Self::D7 => "samples_15m",
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::M1 => "1m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1h",
            Self::H2 => "2h",
            Self::H4 => "4h",
            Self::H8 => "8h",
            Self::H24 => "24h",
            Self::D3 => "3d",
            Self::D7 => "7d",
        }
    }

    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|g| g == self).unwrap();
        if idx + 1 < Self::ALL.len() { Self::ALL[idx + 1] } else { *self }
    }

    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|g| g == self).unwrap();
        if idx > 0 { Self::ALL[idx - 1] } else { *self }
    }
}

/// Stored historical data point (subset of Sample for graphing).
#[derive(Debug, Clone, Default)]
pub struct DataPoint {
    pub ts: i64,
    pub cpu_percent: f64,
    pub cpu_temp: Option<f64>,
    pub ram_percent: f64,
    pub swap_percent: f64,
    pub gpu_percent: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub vram_percent: Option<f64>,
    pub disk_read_rate: f64,  // bytes/sec
    pub disk_write_rate: f64, // bytes/sec
    pub net_rx_rate: f64,     // bytes/sec
    pub net_tx_rate: f64,     // bytes/sec
}
```

- [ ] **Step 2: Add module to main.rs**

```rust
mod metrics;

fn main() {
    println!("LocalMonitor starting...");
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add src/metrics.rs src/main.rs
git commit -m "feat: core types — Sample, Granularity, DataPoint"
```

---

### Task 3: Database Layer

**Files:**
- Create: `src/db.rs`
- Modify: `src/main.rs` (add module)

- [ ] **Step 1: Create `src/db.rs`**

```rust
use crate::metrics::{DataPoint, Granularity, Sample};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open or create the database at %LOCALAPPDATA%/LocalMonitor/localmonitor.db
    pub fn open() -> Result<Self> {
        let dir = dirs::data_local_dir()
            .context("cannot find LOCALAPPDATA")?
            .join("LocalMonitor");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("localmonitor.db");
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing or display-only fallback).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let tables = ["samples_1s", "samples_5s", "samples_30s", "samples_5m", "samples_15m"];
        for table in tables {
            self.conn.execute_batch(&format!(
                "CREATE TABLE IF NOT EXISTS {table} (
                    ts INTEGER NOT NULL,
                    cpu_percent REAL,
                    cpu_temp REAL,
                    ram_percent REAL,
                    swap_percent REAL,
                    gpu_percent REAL,
                    gpu_temp REAL,
                    vram_percent REAL,
                    disk_read_bytes REAL,
                    disk_write_bytes REAL,
                    net_rx_bytes REAL,
                    net_tx_bytes REAL
                );
                CREATE INDEX IF NOT EXISTS idx_{table}_ts ON {table}(ts);"
            ))?;
        }
        Ok(())
    }

    /// Insert a raw 1-second sample.
    pub fn insert_sample(&self, s: &Sample) -> Result<()> {
        self.conn.execute(
            "INSERT INTO samples_1s (ts, cpu_percent, cpu_temp, ram_percent, swap_percent,
             gpu_percent, gpu_temp, vram_percent, disk_read_bytes, disk_write_bytes,
             net_rx_bytes, net_tx_bytes)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                s.ts, s.cpu_percent, s.cpu_temp, s.ram_percent, s.swap_percent,
                s.gpu_percent, s.gpu_temp, s.vram_percent,
                s.disk_read_bytes as f64, s.disk_write_bytes as f64,
                s.net_rx_bytes as f64, s.net_tx_bytes as f64,
            ],
        )?;
        Ok(())
    }

    /// Query historical data points for a given granularity.
    pub fn query(&self, granularity: Granularity) -> Result<Vec<DataPoint>> {
        let table = granularity.table_name();
        let window = granularity.window_secs();
        let now = chrono::Utc::now().timestamp();
        let since = now - window;

        let mut stmt = self.conn.prepare(&format!(
            "SELECT ts, cpu_percent, cpu_temp, ram_percent, swap_percent,
                    gpu_percent, gpu_temp, vram_percent,
                    disk_read_bytes, disk_write_bytes, net_rx_bytes, net_tx_bytes
             FROM {table} WHERE ts >= ?1 ORDER BY ts"
        ))?;

        let rows = stmt.query_map(params![since], |row| {
            Ok(DataPoint {
                ts: row.get(0)?,
                cpu_percent: row.get(1)?,
                cpu_temp: row.get(2)?,
                ram_percent: row.get(3)?,
                swap_percent: row.get(4)?,
                gpu_percent: row.get(5)?,
                gpu_temp: row.get(6)?,
                vram_percent: row.get(7)?,
                disk_read_rate: row.get::<_, f64>(8).unwrap_or(0.0),
                disk_write_rate: row.get::<_, f64>(9).unwrap_or(0.0),
                net_rx_rate: row.get::<_, f64>(10).unwrap_or(0.0),
                net_tx_rate: row.get::<_, f64>(11).unwrap_or(0.0),
            })
        })?;

        let mut points = Vec::new();
        for row in rows {
            points.push(row?);
        }
        Ok(points)
    }

    /// Aggregate from a source table into a target table, averaging over `interval_secs` windows.
    pub fn aggregate(&self, source: &str, target: &str, interval_secs: i64) -> Result<()> {
        // Find the latest timestamp in target to avoid re-aggregating
        let last_ts: i64 = self.conn
            .query_row(&format!("SELECT COALESCE(MAX(ts), 0) FROM {target}"), [], |r| r.get(0))?;

        self.conn.execute(&format!(
            "INSERT INTO {target}
             SELECT (ts / ?1) * ?1 as bucket,
                    AVG(cpu_percent), AVG(cpu_temp), AVG(ram_percent), AVG(swap_percent),
                    AVG(gpu_percent), AVG(gpu_temp), AVG(vram_percent),
                    AVG(disk_read_bytes), AVG(disk_write_bytes),
                    AVG(net_rx_bytes), AVG(net_tx_bytes)
             FROM {source}
             WHERE ts > ?2
             GROUP BY bucket"
        ), params![interval_secs, last_ts])?;

        Ok(())
    }

    /// Run all aggregation steps and prune old data.
    pub fn aggregate_and_prune(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Aggregate: 1s → 5s, 5s → 30s, 30s → 5m, 5m → 15m
        self.aggregate("samples_1s", "samples_5s", 5)?;
        self.aggregate("samples_5s", "samples_30s", 30)?;
        self.aggregate("samples_30s", "samples_5m", 300)?;
        self.aggregate("samples_5m", "samples_15m", 900)?;

        // Prune: keep only the retention window for each table
        let retention = [
            ("samples_1s", 60),
            ("samples_5s", 1800),
            ("samples_30s", 14400),
            ("samples_5m", 86400),
            ("samples_15m", 604800),
        ];
        for (table, retain_secs) in retention {
            let cutoff = now - retain_secs;
            self.conn.execute(
                &format!("DELETE FROM {table} WHERE ts < ?1"),
                params![cutoff],
            )?;
        }

        Ok(())
    }
}
```

- [ ] **Step 2: Add module to main.rs**

Add `mod db;` to `src/main.rs`.

- [ ] **Step 3: Write a test to verify DB round-trip**

Add to bottom of `src/db.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let db = Db::open_in_memory().unwrap();
        let now = chrono::Utc::now().timestamp();
        let sample = Sample {
            ts: now,
            cpu_percent: 55.0,
            ram_percent: 62.0,
            ..Default::default()
        };
        db.insert_sample(&sample).unwrap();
        let points = db.query(Granularity::M1).unwrap();
        assert_eq!(points.len(), 1);
        assert!((points[0].cpu_percent - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregate_and_prune() {
        let db = Db::open_in_memory().unwrap();
        let now = chrono::Utc::now().timestamp();
        // Insert 10 samples at 1-second intervals
        for i in 0..10 {
            let sample = Sample {
                ts: now - 10 + i,
                cpu_percent: 50.0 + i as f64,
                ram_percent: 60.0,
                ..Default::default()
            };
            db.insert_sample(&sample).unwrap();
        }
        db.aggregate_and_prune().unwrap();
        let points_5s = db.query(Granularity::M5).unwrap();
        assert!(!points_5s.is_empty());
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/db.rs src/main.rs
git commit -m "feat: SQLite database layer with aggregation and pruning"
```

---

### Task 4: Collectors — CPU, RAM

**Files:**
- Create: `src/collectors/mod.rs`
- Create: `src/collectors/cpu.rs`
- Create: `src/collectors/ram.rs`
- Modify: `src/main.rs` (add module)

- [ ] **Step 1: Create `src/collectors/mod.rs` with Collector trait and registry**

```rust
pub mod cpu;
pub mod ram;

use crate::metrics::Sample;

/// Trait for all metric collectors.
pub trait Collector: Send {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    /// Collect current values and merge into the given sample.
    fn collect(&mut self, sample: &mut Sample);
}

/// Build the list of all available collectors.
pub fn build_collectors() -> Vec<Box<dyn Collector>> {
    let mut collectors: Vec<Box<dyn Collector>> = vec![
        Box::new(cpu::CpuCollector::new()),
        Box::new(ram::RamCollector::new()),
    ];
    collectors.retain(|c| c.is_available());
    collectors
}
```

- [ ] **Step 2: Create `src/collectors/cpu.rs`**

```rust
use crate::metrics::Sample;
use super::Collector;
use sysinfo::System;

pub struct CpuCollector {
    sys: System,
}

impl CpuCollector {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        Self { sys }
    }
}

impl Collector for CpuCollector {
    fn name(&self) -> &str { "CPU" }
    fn is_available(&self) -> bool { true }

    fn collect(&mut self, sample: &mut Sample) {
        self.sys.refresh_cpu_all();

        let cpus = self.sys.cpus();
        if !cpus.is_empty() {
            sample.per_core_percent = cpus.iter().map(|c| c.cpu_usage() as f64).collect();
            sample.cpu_percent = sample.per_core_percent.iter().sum::<f64>() / cpus.len() as f64;
            sample.cpu_freq_mhz = cpus[0].frequency() as f64;
        }

        // Temperature — best effort on Windows
        let components = sysinfo::Components::new_with_refreshed_list();
        sample.cpu_temp = components.iter()
            .find(|c| c.label().to_lowercase().contains("cpu"))
            .map(|c| c.temperature() as f64);
    }
}
```

- [ ] **Step 3: Create `src/collectors/ram.rs`**

```rust
use crate::metrics::Sample;
use super::Collector;
use sysinfo::System;

pub struct RamCollector {
    sys: System,
}

impl RamCollector {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }
}

impl Collector for RamCollector {
    fn name(&self) -> &str { "RAM" }
    fn is_available(&self) -> bool { true }

    fn collect(&mut self, sample: &mut Sample) {
        self.sys.refresh_memory();

        sample.ram_used_bytes = self.sys.used_memory();
        sample.ram_total_bytes = self.sys.total_memory();
        if sample.ram_total_bytes > 0 {
            sample.ram_percent = (sample.ram_used_bytes as f64 / sample.ram_total_bytes as f64) * 100.0;
        }

        sample.swap_used_bytes = self.sys.used_swap();
        sample.swap_total_bytes = self.sys.total_swap();
        if sample.swap_total_bytes > 0 {
            sample.swap_percent = (sample.swap_used_bytes as f64 / sample.swap_total_bytes as f64) * 100.0;
        }
    }
}
```

- [ ] **Step 4: Add module to main.rs**

Add `mod collectors;` to `src/main.rs`.

- [ ] **Step 5: Verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 6: Commit**

```bash
git add src/collectors/
git commit -m "feat: CPU and RAM collectors"
```

---

### Task 5: Collectors — GPU, Disk, Network, System

**Files:**
- Create: `src/collectors/gpu.rs`
- Create: `src/collectors/disk.rs`
- Create: `src/collectors/network.rs`
- Create: `src/collectors/system.rs`
- Modify: `src/collectors/mod.rs` (register new collectors)

- [ ] **Step 1: Create `src/collectors/gpu.rs`**

```rust
use crate::metrics::Sample;
use super::Collector;
use nvml_wrapper::Nvml;

pub struct GpuCollector {
    nvml: Option<Nvml>,
}

impl GpuCollector {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        Self { nvml }
    }
}

impl Collector for GpuCollector {
    fn name(&self) -> &str { "GPU" }

    fn is_available(&self) -> bool {
        self.nvml.is_some()
    }

    fn collect(&mut self, sample: &mut Sample) {
        let nvml = match &self.nvml {
            Some(n) => n,
            None => return,
        };

        let device = match nvml.device_by_index(0) {
            Ok(d) => d,
            Err(_) => return,
        };

        if let Ok(util) = device.utilization_rates() {
            sample.gpu_percent = Some(util.gpu as f64);
        }
        if let Ok(temp) = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu) {
            sample.gpu_temp = Some(temp as f64);
        }
        if let Ok(fan) = device.fan_speed(0) {
            // fan_speed returns percentage, but we'll display it as-is
            // For RPM we'd need fan_speed in RPM which isn't directly available
            // Store percentage for now
            sample.gpu_fan_rpm = Some(fan);
        }
        if let Ok(clocks) = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics) {
            sample.gpu_clock_mhz = Some(clocks);
        }
        if let Ok(mem) = device.memory_info() {
            sample.vram_used_bytes = Some(mem.used);
            sample.vram_total_bytes = Some(mem.total);
            if mem.total > 0 {
                sample.vram_percent = Some((mem.used as f64 / mem.total as f64) * 100.0);
            }
        }
    }
}
```

- [ ] **Step 2: Create `src/collectors/disk.rs`**

```rust
use crate::metrics::Sample;
use super::Collector;
use sysinfo::Disks;

pub struct DiskCollector {
    prev_read: u64,
    prev_write: u64,
    first_sample: bool,
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            prev_read: 0,
            prev_write: 0,
            first_sample: true,
        }
    }

    fn get_system_disk_usage(disks: &Disks) -> (u64, u64) {
        // Find C: drive
        for disk in disks.list() {
            let mount = disk.mount_point().to_string_lossy();
            if mount.starts_with("C:") || mount == "/" {
                return (disk.total_space(), disk.available_space());
            }
        }
        (0, 0)
    }
}

impl Collector for DiskCollector {
    fn name(&self) -> &str { "Disk" }
    fn is_available(&self) -> bool { true }

    fn collect(&mut self, sample: &mut Sample) {
        let disks = Disks::new_with_refreshed_list();

        let (total, available) = Self::get_system_disk_usage(&disks);
        sample.disk_total_bytes = total;
        sample.disk_used_bytes = total.saturating_sub(available);

        // IO rates via process-level disk usage from sysinfo
        // sysinfo doesn't expose global disk IO counters directly on Windows.
        // We use the Windows performance counters approach instead.
        // For now, track cumulative bytes and diff them.
        use sysinfo::System;
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut total_read: u64 = 0;
        let mut total_write: u64 = 0;
        for (_, proc) in sys.processes() {
            let usage = proc.disk_usage();
            total_read += usage.total_read_bytes;
            total_write += usage.total_written_bytes;
        }

        if self.first_sample {
            self.first_sample = false;
        } else {
            sample.disk_read_bytes = total_read.saturating_sub(self.prev_read);
            sample.disk_write_bytes = total_write.saturating_sub(self.prev_write);
        }
        self.prev_read = total_read;
        self.prev_write = total_write;
    }
}
```

- [ ] **Step 3: Create `src/collectors/network.rs`**

```rust
use crate::metrics::Sample;
use super::Collector;
use sysinfo::Networks;

pub struct NetworkCollector {
    networks: Networks,
    prev_rx: u64,
    prev_tx: u64,
    first_sample: bool,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            prev_rx: 0,
            prev_tx: 0,
            first_sample: true,
        }
    }
}

impl Collector for NetworkCollector {
    fn name(&self) -> &str { "Network" }
    fn is_available(&self) -> bool { true }

    fn collect(&mut self, sample: &mut Sample) {
        self.networks.refresh(true);

        let mut total_rx: u64 = 0;
        let mut total_tx: u64 = 0;
        for (_name, data) in self.networks.iter() {
            total_rx += data.total_received();
            total_tx += data.total_transmitted();
        }

        if self.first_sample {
            self.first_sample = false;
        } else {
            sample.net_rx_bytes = total_rx.saturating_sub(self.prev_rx);
            sample.net_tx_bytes = total_tx.saturating_sub(self.prev_tx);
        }
        self.prev_rx = total_rx;
        self.prev_tx = total_tx;
    }
}
```

- [ ] **Step 4: Create `src/collectors/system.rs`**

```rust
use crate::metrics::Sample;
use super::Collector;
use sysinfo::System;

pub struct SystemCollector {
    sys: System,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }
}

impl Collector for SystemCollector {
    fn name(&self) -> &str { "System" }
    fn is_available(&self) -> bool { true }

    fn collect(&mut self, sample: &mut Sample) {
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
        sample.process_count = self.sys.processes().len();
        sample.uptime_secs = System::uptime();
    }
}
```

- [ ] **Step 5: Update `src/collectors/mod.rs` to register all collectors**

```rust
pub mod cpu;
pub mod ram;
pub mod gpu;
pub mod disk;
pub mod network;
pub mod system;

use crate::metrics::Sample;

/// Trait for all metric collectors.
pub trait Collector: Send {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn collect(&mut self, sample: &mut Sample);
}

/// Build the list of all available collectors.
pub fn build_collectors() -> Vec<Box<dyn Collector>> {
    let mut collectors: Vec<Box<dyn Collector>> = vec![
        Box::new(cpu::CpuCollector::new()),
        Box::new(ram::RamCollector::new()),
        Box::new(gpu::GpuCollector::new()),
        Box::new(disk::DiskCollector::new()),
        Box::new(network::NetworkCollector::new()),
        Box::new(system::SystemCollector::new()),
    ];
    collectors.retain(|c| c.is_available());
    collectors
}
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo build`
Expected: Compiles (GPU may warn if no NVIDIA drivers, but should compile)

- [ ] **Step 7: Commit**

```bash
git add src/collectors/
git commit -m "feat: GPU, disk, network, and system collectors"
```

---

### Task 6: App State & Event Loop

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs` (full rewrite with event loop)

- [ ] **Step 1: Create `src/app.rs` — app state and view management**

```rust
use crate::metrics::{Granularity, Sample};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFocus {
    CpuRam,
    GpuVram,
    DiskIo,
    Network,
}

impl GraphFocus {
    pub const ALL: &[GraphFocus] = &[
        Self::CpuRam, Self::GpuVram, Self::DiskIo, Self::Network,
    ];

    pub fn label(&self) -> &str {
        match self {
            Self::CpuRam => "CPU & RAM",
            Self::GpuVram => "GPU & VRAM",
            Self::DiskIo => "Disk IO",
            Self::Network => "Network",
        }
    }

    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|f| f == self).unwrap();
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|f| f == self).unwrap();
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

pub struct App {
    pub focus: GraphFocus,
    pub granularity: Granularity,
    pub latest_sample: Sample,
    pub gpu_available: bool,
    pub running: bool,
    pub db_warning: Option<String>,
}

impl App {
    pub fn new(gpu_available: bool) -> Self {
        Self {
            focus: GraphFocus::CpuRam,
            granularity: Granularity::M1,
            latest_sample: Sample::default(),
            gpu_available,
            running: true,
            db_warning: None,
        }
    }

    pub fn next_view(&mut self) {
        self.focus = self.focus.next();
        // Skip GPU view if not available
        if !self.gpu_available && self.focus == GraphFocus::GpuVram {
            self.focus = self.focus.next();
        }
    }

    pub fn prev_view(&mut self) {
        self.focus = self.focus.prev();
        if !self.gpu_available && self.focus == GraphFocus::GpuVram {
            self.focus = self.focus.prev();
        }
    }

    pub fn longer_granularity(&mut self) {
        self.granularity = self.granularity.next();
    }

    pub fn shorter_granularity(&mut self) {
        self.granularity = self.granularity.prev();
    }
}
```

- [ ] **Step 2: Rewrite `src/main.rs` with the event loop**

```rust
mod app;
mod collectors;
mod db;
mod metrics;

use anyhow::Result;
use app::App;
use collectors::build_collectors;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use db::Db;
use metrics::Sample;
use ratatui::prelude::*;
use std::io::stdout;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    // Initialize DB (fallback to in-memory on failure)
    let (db, db_warning) = match Db::open() {
        Ok(db) => (db, None),
        Err(e) => (
            Db::open_in_memory()?,
            Some(format!("DB error: {e} — running without history")),
        ),
    };

    // Initialize collectors
    let mut collectors = build_collectors();
    let gpu_available = collectors.iter().any(|c| c.name() == "GPU");

    let mut app = App::new(gpu_available);
    app.db_warning = db_warning;

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let tick_rate = Duration::from_secs(1);
    let mut last_tick = Instant::now();
    let mut last_aggregate = Instant::now();
    let aggregate_interval = Duration::from_secs(60);

    while app.running {
        // Render
        let size = terminal.size()?;
        if size.width < 80 || size.height < 24 {
            terminal.draw(|f| {
                let msg = "Terminal too small — resize to at least 80x24";
                let area = f.area();
                let x = area.width.saturating_sub(msg.len() as u16) / 2;
                let y = area.height / 2;
                f.render_widget(
                    ratatui::widgets::Paragraph::new(msg)
                        .style(Style::default().fg(Color::Yellow)),
                    Rect::new(x, y, msg.len() as u16, 1),
                );
            })?;
        } else {
            // Query historical data for the current view
            let history = db.query(app.granularity).unwrap_or_default();
            terminal.draw(|f| {
                // Placeholder — will be replaced in Task 7
                let area = f.area();
                f.render_widget(
                    ratatui::widgets::Paragraph::new("LocalMonitor — UI loading..."),
                    area,
                );
            })?;
        }

        // Handle input (non-blocking with timeout)
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => app.running = false,
                        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => app.prev_view(),
                        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => app.next_view(),
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => app.longer_granularity(),
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => app.shorter_granularity(),
                        _ => {}
                    }
                }
            }
        }

        // Collect metrics every tick
        if last_tick.elapsed() >= tick_rate {
            let mut sample = Sample::default();
            sample.ts = chrono::Utc::now().timestamp();
            for collector in &mut collectors {
                collector.collect(&mut sample);
            }
            let _ = db.insert_sample(&sample);
            app.latest_sample = sample;
            last_tick = Instant::now();
        }

        // Aggregate every 60 seconds
        if last_aggregate.elapsed() >= aggregate_interval {
            let _ = db.aggregate_and_prune();
            last_aggregate = Instant::now();
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 4: Quick smoke test — run it, press Q to quit**

Run: `cargo run`
Expected: Shows "LocalMonitor — UI loading..." text, Q exits cleanly

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: app state and event loop with input handling"
```

---

### Task 7: UI — Theme and HUD Layout

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/theme.rs`
- Create: `src/ui/hud/mod.rs`
- Create: `src/ui/hud/cpu.rs`
- Create: `src/ui/hud/ram.rs`
- Create: `src/ui/hud/gpu.rs`
- Create: `src/ui/hud/disk.rs`
- Create: `src/ui/hud/network.rs`
- Create: `src/ui/hud/system.rs`

- [ ] **Step 1: Create `src/ui/theme.rs`**

```rust
use ratatui::style::Color;

/// Returns a color based on utilization percentage.
pub fn utilization_color(percent: f64) -> Color {
    if percent >= 80.0 {
        Color::Red
    } else if percent >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Returns a color based on temperature.
pub fn temp_color(temp: f64) -> Color {
    if temp >= 80.0 {
        Color::Red
    } else if temp >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

/// Graph line colors for different metrics.
pub const GRAPH_PRIMARY: Color = Color::Cyan;
pub const GRAPH_SECONDARY: Color = Color::Magenta;
pub const GRAPH_TERTIARY: Color = Color::Yellow;
pub const BORDER_COLOR: Color = Color::DarkGray;
pub const TITLE_COLOR: Color = Color::White;
pub const LABEL_COLOR: Color = Color::Gray;
```

- [ ] **Step 2: Create `src/ui/hud/system.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::Sample;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let uptime = format_uptime(sample.uptime_secs);
    let text = format!(
        "  LocalMonitor                    Uptime: {}    Procs: {}  ",
        uptime, sample.process_count
    );
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(theme::TITLE_COLOR).bold())
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)));
    f.render_widget(paragraph, area);
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
```

- [ ] **Step 3: Create `src/ui/hud/cpu.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::Sample;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let pct = sample.cpu_percent;
    let color = theme::utilization_color(pct);

    let freq = format!("{:.1} GHz", sample.cpu_freq_mhz / 1000.0);
    let temp = match sample.cpu_temp {
        Some(t) => {
            let tc = theme::temp_color(t);
            format!("  {:.0}°C", t)
        }
        None => "".to_string(),
    };

    let cores: String = sample.per_core_percent.iter()
        .map(|c| format!("{:.0}", c))
        .collect::<Vec<_>>()
        .join(" ");

    let gauge_ratio = (pct / 100.0).clamp(0.0, 1.0);
    let gauge = Gauge::default()
        .block(Block::default().title(" CPU ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .ratio(gauge_ratio)
        .label(format!("{:.0}%  {}{}", pct, freq, temp));
    f.render_widget(gauge, Rect::new(area.x, area.y, area.width, 2));

    if area.height > 2 && !cores.is_empty() {
        let cores_text = Paragraph::new(format!("  Cores: {}", cores))
            .style(Style::default().fg(theme::LABEL_COLOR));
        f.render_widget(cores_text, Rect::new(area.x, area.y + 2, area.width, 1));
    }
}
```

- [ ] **Step 4: Create `src/ui/hud/ram.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::Sample;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let pct = sample.ram_percent;
    let color = theme::utilization_color(pct);
    let used_gb = sample.ram_used_bytes as f64 / 1_073_741_824.0;
    let total_gb = sample.ram_total_bytes as f64 / 1_073_741_824.0;
    let swap_used_gb = sample.swap_used_bytes as f64 / 1_073_741_824.0;
    let swap_total_gb = sample.swap_total_bytes as f64 / 1_073_741_824.0;

    let gauge = Gauge::default()
        .block(Block::default().title(" RAM ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .ratio((pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.0}%  {:.1} / {:.1} GB", pct, used_gb, total_gb));
    f.render_widget(gauge, Rect::new(area.x, area.y, area.width, 2));

    if area.height > 2 {
        let swap_text = Paragraph::new(format!("  Swap: {:.1} / {:.1} GB", swap_used_gb, swap_total_gb))
            .style(Style::default().fg(theme::LABEL_COLOR));
        f.render_widget(swap_text, Rect::new(area.x, area.y + 2, area.width, 1));
    }
}
```

- [ ] **Step 5: Create `src/ui/hud/gpu.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::Sample;
use crate::ui::theme;

pub fn render_gpu(f: &mut Frame, area: Rect, sample: &Sample) {
    match sample.gpu_percent {
        Some(pct) => {
            let color = theme::utilization_color(pct);
            let temp_str = sample.gpu_temp
                .map(|t| format!("  {:.0}°C", t))
                .unwrap_or_default();
            let fan_str = sample.gpu_fan_rpm
                .map(|r| format!("  Fan: {}%", r))
                .unwrap_or_default();
            let clock_str = sample.gpu_clock_mhz
                .map(|c| format!("  {}MHz", c))
                .unwrap_or_default();

            let gauge = Gauge::default()
                .block(Block::default().title(" GPU ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
                .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
                .ratio((pct / 100.0).clamp(0.0, 1.0))
                .label(format!("{:.0}%{}{}", pct, temp_str, fan_str));
            f.render_widget(gauge, Rect::new(area.x, area.y, area.width, 2));

            if area.height > 2 {
                let detail = Paragraph::new(format!("  Clock:{}", clock_str))
                    .style(Style::default().fg(theme::LABEL_COLOR));
                f.render_widget(detail, Rect::new(area.x, area.y + 2, area.width, 1));
            }
        }
        None => {
            let na = Paragraph::new("  GPU: N/A")
                .block(Block::default().title(" GPU ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(na, area);
        }
    }
}

pub fn render_vram(f: &mut Frame, area: Rect, sample: &Sample) {
    match sample.vram_percent {
        Some(pct) => {
            let color = theme::utilization_color(pct);
            let used_gb = sample.vram_used_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;
            let total_gb = sample.vram_total_bytes.unwrap_or(0) as f64 / 1_073_741_824.0;

            let gauge = Gauge::default()
                .block(Block::default().title(" VRAM ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
                .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
                .ratio((pct / 100.0).clamp(0.0, 1.0))
                .label(format!("{:.0}%  {:.1} / {:.1} GB", pct, used_gb, total_gb));
            f.render_widget(gauge, Rect::new(area.x, area.y, area.width, 2));
        }
        None => {
            let na = Paragraph::new("  VRAM: N/A")
                .block(Block::default().title(" VRAM ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(na, area);
        }
    }
}
```

- [ ] **Step 6: Create `src/ui/hud/disk.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::Sample;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let pct = if sample.disk_total_bytes > 0 {
        (sample.disk_used_bytes as f64 / sample.disk_total_bytes as f64) * 100.0
    } else {
        0.0
    };
    let color = theme::utilization_color(pct);

    let read_rate = format_bytes_rate(sample.disk_read_bytes as f64);
    let write_rate = format_bytes_rate(sample.disk_write_bytes as f64);

    let gauge = Gauge::default()
        .block(Block::default().title(" Disk C: ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
        .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
        .ratio((pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.0}%  R: {}  W: {}", pct, read_rate, write_rate));
    f.render_widget(gauge, area);
}

pub fn format_bytes_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else {
        format!("{:.0} KB/s", bytes_per_sec / 1024.0)
    }
}
```

- [ ] **Step 7: Create `src/ui/hud/network.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::Sample;
use crate::ui::theme;
use crate::ui::hud::disk::format_bytes_rate;

pub fn render(f: &mut Frame, area: Rect, sample: &Sample) {
    let dl = format_bytes_rate(sample.net_rx_bytes as f64);
    let ul = format_bytes_rate(sample.net_tx_bytes as f64);

    let text = format!("  Net  ↓ {}  ↑ {}", dl, ul);
    let paragraph = Paragraph::new(text)
        .block(Block::default().title(" Network ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
        .style(Style::default().fg(theme::LABEL_COLOR));
    f.render_widget(paragraph, area);
}
```

- [ ] **Step 8: Create `src/ui/hud/mod.rs`**

```rust
pub mod cpu;
pub mod ram;
pub mod gpu;
pub mod disk;
pub mod network;
pub mod system;

use ratatui::prelude::*;
use crate::metrics::Sample;

/// Render the full HUD (top section of the dashboard).
pub fn render_hud(f: &mut Frame, area: Rect, sample: &Sample) {
    // Header row: 2 lines
    // CPU/GPU row: 3 lines
    // RAM/VRAM row: 3 lines
    // Disk/Net row: 2 lines
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(3),  // CPU | GPU
            Constraint::Length(3),  // RAM | VRAM
            Constraint::Length(2),  // Disk | Net
        ])
        .split(area);

    // Header
    system::render(f, rows[0], sample);

    // CPU | GPU
    let cols_1 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);
    cpu::render(f, cols_1[0], sample);
    gpu::render_gpu(f, cols_1[1], sample);

    // RAM | VRAM
    let cols_2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);
    ram::render(f, cols_2[0], sample);
    gpu::render_vram(f, cols_2[1], sample);

    // Disk | Net
    let cols_3 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[3]);
    disk::render(f, cols_3[0], sample);
    network::render(f, cols_3[1], sample);
}
```

- [ ] **Step 9: Create `src/ui/mod.rs`**

```rust
pub mod hud;
pub mod theme;
```

- [ ] **Step 10: Add `mod ui;` to `src/main.rs` and verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 11: Commit**

```bash
git add src/ui/
git commit -m "feat: theme and HUD panels — CPU, RAM, GPU, VRAM, disk, network, system"
```

---

### Task 8: UI — Graph Views

**Files:**
- Create: `src/ui/graphs/mod.rs`
- Create: `src/ui/graphs/cpu_ram.rs`
- Create: `src/ui/graphs/gpu_vram.rs`
- Create: `src/ui/graphs/disk.rs`
- Create: `src/ui/graphs/network.rs`
- Modify: `src/ui/mod.rs` (add graphs module)

- [ ] **Step 1: Create `src/ui/graphs/cpu_ram.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::DataPoint;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // CPU graph
    let cpu_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.cpu_percent))
        .collect();
    let cpu_temp_data: Vec<(f64, f64)> = data.iter().enumerate()
        .filter_map(|(i, d)| d.cpu_temp.map(|t| (i as f64, t)))
        .collect();

    let mut datasets = vec![
        Dataset::default()
            .name("CPU %")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_PRIMARY))
            .data(&cpu_data),
    ];
    if !cpu_temp_data.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Temp °C")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme::GRAPH_SECONDARY))
                .data(&cpu_temp_data),
        );
    }

    let x_max = data.len().max(1) as f64;
    let chart = Chart::new(datasets)
        .block(Block::default().title(" CPU ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
        .x_axis(Axis::default().bounds([0.0, x_max]))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .labels(["0", "25", "50", "75", "100"]));
    f.render_widget(chart, rows[0]);

    // RAM graph
    let ram_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.ram_percent))
        .collect();
    let swap_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.swap_percent))
        .collect();

    let ram_chart = Chart::new(vec![
        Dataset::default()
            .name("RAM %")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_PRIMARY))
            .data(&ram_data),
        Dataset::default()
            .name("Swap %")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_SECONDARY))
            .data(&swap_data),
    ])
    .block(Block::default().title(" RAM ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
    .x_axis(Axis::default().bounds([0.0, x_max]))
    .y_axis(Axis::default()
        .bounds([0.0, 100.0])
        .labels(["0", "25", "50", "75", "100"]));
    f.render_widget(ram_chart, rows[1]);
}
```

- [ ] **Step 2: Create `src/ui/graphs/gpu_vram.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::DataPoint;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let x_max = data.len().max(1) as f64;

    // GPU graph
    let gpu_data: Vec<(f64, f64)> = data.iter().enumerate()
        .filter_map(|(i, d)| d.gpu_percent.map(|p| (i as f64, p)))
        .collect();
    let gpu_temp_data: Vec<(f64, f64)> = data.iter().enumerate()
        .filter_map(|(i, d)| d.gpu_temp.map(|t| (i as f64, t)))
        .collect();

    let mut datasets = vec![];
    if !gpu_data.is_empty() {
        datasets.push(
            Dataset::default()
                .name("GPU %")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme::GRAPH_PRIMARY))
                .data(&gpu_data),
        );
    }
    if !gpu_temp_data.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Temp °C")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme::GRAPH_SECONDARY))
                .data(&gpu_temp_data),
        );
    }

    let chart = Chart::new(datasets)
        .block(Block::default().title(" GPU ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
        .x_axis(Axis::default().bounds([0.0, x_max]))
        .y_axis(Axis::default()
            .bounds([0.0, 100.0])
            .labels(["0", "25", "50", "75", "100"]));
    f.render_widget(chart, rows[0]);

    // VRAM graph
    let vram_data: Vec<(f64, f64)> = data.iter().enumerate()
        .filter_map(|(i, d)| d.vram_percent.map(|p| (i as f64, p)))
        .collect();

    let vram_chart = Chart::new(vec![
        Dataset::default()
            .name("VRAM %")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_PRIMARY))
            .data(&vram_data),
    ])
    .block(Block::default().title(" VRAM ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
    .x_axis(Axis::default().bounds([0.0, x_max]))
    .y_axis(Axis::default()
        .bounds([0.0, 100.0])
        .labels(["0", "25", "50", "75", "100"]));
    f.render_widget(vram_chart, rows[1]);
}
```

- [ ] **Step 3: Create `src/ui/graphs/disk.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::DataPoint;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let x_max = data.len().max(1) as f64;

    // Find max rate for Y-axis scaling
    let max_read = data.iter().map(|d| d.disk_read_rate).fold(0.0f64, f64::max);
    let max_write = data.iter().map(|d| d.disk_write_rate).fold(0.0f64, f64::max);
    let y_max_read = (max_read * 1.2).max(1024.0); // At least 1 KB/s
    let y_max_write = (max_write * 1.2).max(1024.0);

    let read_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.disk_read_rate))
        .collect();
    let write_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.disk_write_rate))
        .collect();

    let read_label = format_rate_label(y_max_read);
    let read_chart = Chart::new(vec![
        Dataset::default()
            .name("Read")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_PRIMARY))
            .data(&read_data),
    ])
    .block(Block::default().title(" Disk Read ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
    .x_axis(Axis::default().bounds([0.0, x_max]))
    .y_axis(Axis::default()
        .bounds([0.0, y_max_read])
        .labels(["0", &read_label]));
    f.render_widget(read_chart, rows[0]);

    let write_label = format_rate_label(y_max_write);
    let write_chart = Chart::new(vec![
        Dataset::default()
            .name("Write")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_SECONDARY))
            .data(&write_data),
    ])
    .block(Block::default().title(" Disk Write ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
    .x_axis(Axis::default().bounds([0.0, x_max]))
    .y_axis(Axis::default()
        .bounds([0.0, y_max_write])
        .labels(["0", &write_label]));
    f.render_widget(write_chart, rows[1]);
}

fn format_rate_label(bytes: f64) -> String {
    if bytes >= 1_048_576.0 {
        format!("{:.0} MB/s", bytes / 1_048_576.0)
    } else {
        format!("{:.0} KB/s", bytes / 1024.0)
    }
}
```

- [ ] **Step 4: Create `src/ui/graphs/network.rs`**

```rust
use ratatui::{prelude::*, widgets::*};
use crate::metrics::DataPoint;
use crate::ui::theme;

pub fn render(f: &mut Frame, area: Rect, data: &[DataPoint]) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let x_max = data.len().max(1) as f64;

    let max_rx = data.iter().map(|d| d.net_rx_rate).fold(0.0f64, f64::max);
    let max_tx = data.iter().map(|d| d.net_tx_rate).fold(0.0f64, f64::max);
    let y_max_rx = (max_rx * 1.2).max(1024.0);
    let y_max_tx = (max_tx * 1.2).max(1024.0);

    let rx_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.net_rx_rate))
        .collect();
    let tx_data: Vec<(f64, f64)> = data.iter().enumerate()
        .map(|(i, d)| (i as f64, d.net_tx_rate))
        .collect();

    let rx_label = format_rate_label(y_max_rx);
    let rx_chart = Chart::new(vec![
        Dataset::default()
            .name("Download")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_PRIMARY))
            .data(&rx_data),
    ])
    .block(Block::default().title(" Download ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
    .x_axis(Axis::default().bounds([0.0, x_max]))
    .y_axis(Axis::default()
        .bounds([0.0, y_max_rx])
        .labels(["0", &rx_label]));
    f.render_widget(rx_chart, rows[0]);

    let tx_label = format_rate_label(y_max_tx);
    let tx_chart = Chart::new(vec![
        Dataset::default()
            .name("Upload")
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme::GRAPH_SECONDARY))
            .data(&tx_data),
    ])
    .block(Block::default().title(" Upload ").borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)))
    .x_axis(Axis::default().bounds([0.0, x_max]))
    .y_axis(Axis::default()
        .bounds([0.0, y_max_tx])
        .labels(["0", &tx_label]));
    f.render_widget(tx_chart, rows[1]);
}

fn format_rate_label(bytes: f64) -> String {
    if bytes >= 1_048_576.0 {
        format!("{:.0} MB/s", bytes / 1_048_576.0)
    } else {
        format!("{:.0} KB/s", bytes / 1024.0)
    }
}
```

- [ ] **Step 5: Create `src/ui/graphs/mod.rs`**

```rust
pub mod cpu_ram;
pub mod gpu_vram;
pub mod disk;
pub mod network;

use ratatui::{prelude::*, widgets::*};
use crate::app::GraphFocus;
use crate::metrics::{DataPoint, Granularity};
use crate::ui::theme;

/// Render the graph section (bottom of dashboard).
pub fn render_graphs(
    f: &mut Frame,
    area: Rect,
    focus: GraphFocus,
    granularity: Granularity,
    data: &[DataPoint],
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // view title + granularity
            Constraint::Min(6),   // graphs
            Constraint::Length(1), // controls bar
        ])
        .split(area);

    // Title bar
    let title = format!("  ► {}                                              [{}]", focus.label(), granularity.label());
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(theme::TITLE_COLOR).bold());
    f.render_widget(title_widget, rows[0]);

    // Graphs
    if data.is_empty() {
        let msg = Paragraph::new("  Collecting data...")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme::BORDER_COLOR)));
        f.render_widget(msg, rows[1]);
    } else {
        match focus {
            GraphFocus::CpuRam => cpu_ram::render(f, rows[1], data),
            GraphFocus::GpuVram => gpu_vram::render(f, rows[1], data),
            GraphFocus::DiskIo => disk::render(f, rows[1], data),
            GraphFocus::Network => network::render(f, rows[1], data),
        }
    }

    // Controls bar
    let controls = Paragraph::new("  ◀▶/AD: view   ▲▼/WS: granularity   Q: quit")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(controls, rows[2]);
}
```

- [ ] **Step 6: Update `src/ui/mod.rs`**

```rust
pub mod hud;
pub mod graphs;
pub mod theme;
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 8: Commit**

```bash
git add src/ui/
git commit -m "feat: graph views — CPU/RAM, GPU/VRAM, disk IO, network"
```

---

### Task 9: Wire Everything Together — Full Dashboard Rendering

**Files:**
- Modify: `src/main.rs` (replace placeholder render with full UI)

- [ ] **Step 1: Update the render section in `src/main.rs`**

Replace the placeholder `terminal.draw` block (inside the `else` branch where terminal is large enough) with:

```rust
            let history = db.query(app.granularity).unwrap_or_default();
            terminal.draw(|f| {
                let main_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(11), // HUD (header 3 + cpu/gpu 3 + ram/vram 3 + disk/net 2)
                        Constraint::Min(8),     // Graphs
                    ])
                    .split(f.area());

                // DB warning banner if applicable
                if let Some(ref warning) = app.db_warning {
                    // Render warning as part of HUD area
                }

                ui::hud::render_hud(f, main_layout[0], &app.latest_sample);
                ui::graphs::render_graphs(
                    f,
                    main_layout[1],
                    app.focus,
                    app.granularity,
                    &history,
                );
            })?;
```

Also add `mod ui;` and `use ui;` if not already present.

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles

- [ ] **Step 3: Run the app and validate the full dashboard renders**

Run: `cargo run`
Expected: Full HUD with gauges + graph area showing "Collecting data..." initially. Metrics start populating after 1 second. Arrow keys cycle views, W/S cycles granularity. Q quits.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire full dashboard — HUD + graphs rendering"
```

---

### Task 10: Polish and Edge Cases

**Files:**
- Modify: `src/main.rs` (Ctrl+C handling, DB warning banner)
- Modify: Various files for compilation fixes found during integration

- [ ] **Step 1: Add Ctrl+C handling**

In `src/main.rs`, add to the key event match:

```rust
KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
    app.running = false;
}
```

- [ ] **Step 2: Add panic hook for terminal cleanup**

At the start of `main()`, before `enable_raw_mode`:

```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic| {
    let _ = disable_raw_mode();
    let _ = stdout().execute(LeaveAlternateScreen);
    original_hook(panic);
}));
```

- [ ] **Step 3: Build release binary**

Run: `cargo build --release`
Expected: Single `.exe` in `target/release/localmonitor.exe`

- [ ] **Step 4: Test release binary runs standalone**

Run: `./target/release/localmonitor.exe`
Expected: Dashboard launches, all metrics visible, Q quits cleanly

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: polish — Ctrl+C handling, panic recovery, release build"
```

---

### Task 11: Final Integration Test & Push

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 2: Run clippy for lint check**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Fix any issues found by tests or clippy**

- [ ] **Step 4: Final commit if any fixes**

- [ ] **Step 5: Push to remote**

```bash
git push origin main
```
