use sysinfo::{Components, System};

use super::Collector;
use crate::metrics::Sample;

pub struct CpuCollector {
    sys: System,
    components: Components,
}

impl CpuCollector {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        let components = Components::new_with_refreshed_list();
        Self { sys, components }
    }

    /// Best-effort CPU temperature from sysinfo Components.
    /// Looks for a component whose label contains "cpu" (case-insensitive).
    fn cpu_temperature(&mut self) -> Option<f64> {
        self.components.refresh(true);
        self.components
            .iter()
            .filter(|c| {
                let label = c.label().to_lowercase();
                label.contains("cpu") || label.contains("core") || label.contains("tctl")
            })
            .filter_map(|c| c.temperature().map(|t| t as f64))
            .reduce(f64::max)
    }
}

impl Collector for CpuCollector {
    fn name(&self) -> &str {
        "cpu"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn collect(&mut self, sample: &mut Sample) {
        self.sys.refresh_cpu_all();

        sample.cpu_percent = self.sys.global_cpu_usage() as f64;

        sample.per_core_percent = self
            .sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .collect();

        // Average frequency across all cores.
        let cpus = self.sys.cpus();
        if !cpus.is_empty() {
            let total_freq: u64 = cpus.iter().map(|c| c.frequency()).sum();
            sample.cpu_freq_mhz = total_freq as f64 / cpus.len() as f64;
        }

        sample.cpu_temp = self.cpu_temperature();
    }
}
