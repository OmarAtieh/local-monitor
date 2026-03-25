mod cpu;
mod disk;
mod gpu;
mod network;
mod ram;
mod system;

use crate::metrics::Sample;

/// Trait for all metric collectors. No `Send` bound — this is a single-threaded app.
pub trait Collector {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn collect(&mut self, sample: &mut Sample);
}

/// Build all collectors, filtering out any that report themselves as unavailable.
pub fn build_collectors() -> Vec<Box<dyn Collector>> {
    let candidates: Vec<Box<dyn Collector>> = vec![
        Box::new(cpu::CpuCollector::new()),
        Box::new(ram::RamCollector::new()),
        Box::new(gpu::GpuCollector::new()),
        Box::new(disk::DiskCollector::new()),
        Box::new(network::NetworkCollector::new()),
        Box::new(system::SystemCollector::new()),
    ];

    candidates
        .into_iter()
        .filter(|c| c.is_available())
        .collect()
}
