use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::Nvml;

use super::Collector;
use crate::metrics::Sample;

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
    fn name(&self) -> &str {
        "gpu"
    }

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

        // Utilization
        sample.gpu_percent = device.utilization_rates().ok().map(|u| u.gpu as f64);

        // Temperature
        sample.gpu_temp = device
            .temperature(TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as f64);

        // Fan speed (percentage). Try fan index 0.
        sample.gpu_fan_percent = device.fan_speed(0).ok();

        // Clock speed (Graphics)
        sample.gpu_clock_mhz = device.clock_info(Clock::Graphics).ok();

        // VRAM
        if let Ok(mem) = device.memory_info() {
            sample.vram_used_bytes = Some(mem.used);
            sample.vram_total_bytes = Some(mem.total);
            sample.vram_percent = if mem.total > 0 {
                Some((mem.used as f64 / mem.total as f64) * 100.0)
            } else {
                None
            };
        }
    }
}
