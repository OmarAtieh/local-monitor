/// A single point-in-time snapshot of all system metrics.
#[derive(Debug, Clone, Default)]
pub struct Sample {
    pub ts: i64,
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
    pub gpu_fan_percent: Option<u32>,
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
    pub net_rx_total: u64,
    pub net_tx_total: u64,
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
    #[allow(dead_code)]
    pub const ALL: &[Granularity] = &[
        Self::M1,
        Self::M5,
        Self::M15,
        Self::M30,
        Self::H1,
        Self::H2,
        Self::H4,
        Self::H8,
        Self::H24,
        Self::D3,
        Self::D7,
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
        match self {
            Self::M1 => Self::M5,
            Self::M5 => Self::M15,
            Self::M15 => Self::M30,
            Self::M30 => Self::H1,
            Self::H1 => Self::H2,
            Self::H2 => Self::H4,
            Self::H4 => Self::H8,
            Self::H8 => Self::H24,
            Self::H24 => Self::D3,
            Self::D3 => Self::D7,
            Self::D7 => Self::D7,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::M1 => Self::M1,
            Self::M5 => Self::M1,
            Self::M15 => Self::M5,
            Self::M30 => Self::M15,
            Self::H1 => Self::M30,
            Self::H2 => Self::H1,
            Self::H4 => Self::H2,
            Self::H8 => Self::H4,
            Self::H24 => Self::H8,
            Self::D3 => Self::H24,
            Self::D7 => Self::D3,
        }
    }
}

/// Stored historical data point for graphing.
#[derive(Debug, Clone, Default)]
pub struct DataPoint {
    #[allow(dead_code)]
    pub ts: i64,
    pub cpu_percent: f64,
    pub cpu_temp: Option<f64>,
    pub ram_percent: f64,
    pub swap_percent: f64,
    pub gpu_percent: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub vram_percent: Option<f64>,
    pub disk_read_rate: f64,
    pub disk_write_rate: f64,
    pub net_rx_rate: f64,
    pub net_tx_rate: f64,
}
