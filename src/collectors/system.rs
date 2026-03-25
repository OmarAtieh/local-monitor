use sysinfo::System;

use super::Collector;
use crate::metrics::Sample;

pub struct SystemCollector {
    sys: System,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }
}

impl Collector for SystemCollector {
    fn name(&self) -> &str {
        "system"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn collect(&mut self, sample: &mut Sample) {
        self.sys
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sample.process_count = self.sys.processes().len();
        sample.uptime_secs = System::uptime();
    }
}
