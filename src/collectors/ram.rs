use sysinfo::System;

use super::Collector;
use crate::metrics::Sample;

pub struct RamCollector {
    sys: System,
}

impl RamCollector {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        Self { sys }
    }
}

impl Collector for RamCollector {
    fn name(&self) -> &str {
        "ram"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn collect(&mut self, sample: &mut Sample) {
        self.sys.refresh_memory();

        sample.ram_used_bytes = self.sys.used_memory();
        sample.ram_total_bytes = self.sys.total_memory();
        sample.ram_percent = if sample.ram_total_bytes > 0 {
            (sample.ram_used_bytes as f64 / sample.ram_total_bytes as f64) * 100.0
        } else {
            0.0
        };

        sample.swap_used_bytes = self.sys.used_swap();
        sample.swap_total_bytes = self.sys.total_swap();
        sample.swap_percent = if sample.swap_total_bytes > 0 {
            (sample.swap_used_bytes as f64 / sample.swap_total_bytes as f64) * 100.0
        } else {
            0.0
        };
    }
}
