use sysinfo::Networks;

use super::Collector;
use crate::metrics::Sample;

pub struct NetworkCollector {
    networks: Networks,
    prev_rx: Option<u64>,
    prev_tx: Option<u64>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        Self {
            networks,
            prev_rx: None,
            prev_tx: None,
        }
    }

    /// Sum total_received across all interfaces.
    fn total_received(&self) -> u64 {
        self.networks
            .values()
            .map(|data| data.total_received())
            .sum()
    }

    /// Sum total_transmitted across all interfaces.
    fn total_transmitted(&self) -> u64 {
        self.networks
            .values()
            .map(|data| data.total_transmitted())
            .sum()
    }
}

impl Collector for NetworkCollector {
    fn name(&self) -> &str {
        "network"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn collect(&mut self, sample: &mut Sample) {
        self.networks.refresh(true);

        let cur_rx = self.total_received();
        let cur_tx = self.total_transmitted();

        // Cumulative totals
        sample.net_rx_total = cur_rx;
        sample.net_tx_total = cur_tx;

        // Delta (rate since last sample)
        if let (Some(prev_rx), Some(prev_tx)) = (self.prev_rx, self.prev_tx) {
            sample.net_rx_bytes = cur_rx.saturating_sub(prev_rx);
            sample.net_tx_bytes = cur_tx.saturating_sub(prev_tx);
        }
        // else: first sample, leave at 0 (default)

        self.prev_rx = Some(cur_rx);
        self.prev_tx = Some(cur_tx);
    }
}
