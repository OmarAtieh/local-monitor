use sysinfo::Disks;

use super::Collector;
use crate::metrics::Sample;

pub struct DiskCollector {
    disks: Disks,
    prev_read_bytes: Option<u64>,
    prev_write_bytes: Option<u64>,
}

impl DiskCollector {
    pub fn new() -> Self {
        let disks = Disks::new_with_refreshed_list();
        Self {
            disks,
            prev_read_bytes: None,
            prev_write_bytes: None,
        }
    }

    /// Find the C: drive (or root on other platforms) and return capacity info.
    fn find_primary_disk_capacity(&self) -> (u64, u64) {
        for disk in self.disks.iter() {
            let mount = disk.mount_point().to_string_lossy();
            // On Windows, look for C:\
            if mount.starts_with("C:") || mount == "/" {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                return (used, total);
            }
        }
        // Fallback: use the first disk if no C: found
        if let Some(disk) = self.disks.iter().next() {
            let total = disk.total_space();
            let available = disk.available_space();
            return (total.saturating_sub(available), total);
        }
        (0, 0)
    }

    /// Sum read/write bytes across all disks using the Disk usage API.
    fn total_io_bytes(&self) -> (u64, u64) {
        let mut total_read = 0u64;
        let mut total_write = 0u64;
        for disk in self.disks.iter() {
            let usage = disk.usage();
            total_read = total_read.saturating_add(usage.total_read_bytes);
            total_write = total_write.saturating_add(usage.total_written_bytes);
        }
        (total_read, total_write)
    }
}

impl Collector for DiskCollector {
    fn name(&self) -> &str {
        "disk"
    }

    fn is_available(&self) -> bool {
        self.disks.iter().next().is_some()
    }

    fn collect(&mut self, sample: &mut Sample) {
        self.disks.refresh(true);

        // Capacity
        let (used, total) = self.find_primary_disk_capacity();
        sample.disk_used_bytes = used;
        sample.disk_total_bytes = total;

        // IO rates via diff
        let (cur_read, cur_write) = self.total_io_bytes();

        if let (Some(prev_r), Some(prev_w)) = (self.prev_read_bytes, self.prev_write_bytes) {
            sample.disk_read_bytes = cur_read.saturating_sub(prev_r);
            sample.disk_write_bytes = cur_write.saturating_sub(prev_w);
        }
        // else: first sample, leave at 0 (default)

        self.prev_read_bytes = Some(cur_read);
        self.prev_write_bytes = Some(cur_write);
    }
}
