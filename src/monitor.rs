use crate::gpu::GpuMonitor;
use sysinfo::{Networks, System};

pub struct SystemMonitor {
    sys: System,
    networks: Networks,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        Self { sys, networks }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.networks.refresh();
    }

    pub fn get_cpu_usage(&self) -> f32 {
        self.sys.global_cpu_info().cpu_usage()
    }

    pub fn get_ram_usage(&self) -> (u64, u64) {
        (self.sys.used_memory(), self.sys.total_memory())
    }

    pub fn get_network_stats(&self) -> (u64, u64) {
        let mut rx = 0;
        let mut tx = 0;
        for (_interface_name, data) in &self.networks {
            rx += data.received();
            tx += data.transmitted();
        }
        // TODO(senior-ui): Track per-interface deltas + smoothing so the UI can surface the
        // busiest link, rather than dumping aggregated spikes that flicker each second.
        (rx, tx)
    }

    pub fn get_gpu_usage(&self) -> Option<f32> {
        GpuMonitor::get_usage()
    }
}
