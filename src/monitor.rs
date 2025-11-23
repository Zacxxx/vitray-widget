use crate::gpu::GpuMonitor;

use std::thread;
use std::time::Duration;
use sysinfo::{Networks, System};

#[derive(Clone, Debug)]
pub struct MonitorData {
    pub cpu_usage: f32,
    pub ram_used: u64,
    pub ram_total: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub gpu_usage: Option<f32>,
}

struct SystemMonitor {
    sys: System,
    networks: Networks,
}

impl SystemMonitor {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        Self { sys, networks }
    }

    fn refresh(&mut self) -> MonitorData {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.networks.refresh();

        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        let (ram_used, ram_total) = (self.sys.used_memory(), self.sys.total_memory());

        let mut rx_bytes = 0;
        let mut tx_bytes = 0;
        for (_interface_name, data) in &self.networks {
            rx_bytes += data.received();
            tx_bytes += data.transmitted();
        }

        let gpu_usage = GpuMonitor::get_usage();

        MonitorData {
            cpu_usage,
            ram_used,
            ram_total,
            rx_bytes,
            tx_bytes,
            gpu_usage,
        }
    }
}

pub fn start_monitoring_service() -> async_channel::Receiver<MonitorData> {
    let (sender, receiver) = async_channel::unbounded();

    thread::spawn(move || {
        smol::block_on(async {
            let mut monitor = SystemMonitor::new();
            loop {
                let data = monitor.refresh();
                if sender.send(data).await.is_err() {
                    break; // Channel closed
                }
                smol::Timer::after(Duration::from_secs(1)).await;
            }
        });
    });

    receiver
}
