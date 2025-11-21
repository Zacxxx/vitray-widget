use std::process::Command;

pub struct GpuMonitor;

impl GpuMonitor {
    pub fn get_usage() -> Option<f32> {
        // Try nvidia-smi first
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=utilization.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if let Ok(val) = stdout.trim().parse::<f32>() {
                    return Some(val);
                }
            }
        }

        // Try intel_gpu_top (requires root usually, so maybe just check sysfs for some cards)
        // For now, let's check /sys/class/drm/card0/device/gpu_busy_percent if it exists (Intel)
        if let Ok(content) = std::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
             if let Ok(val) = content.trim().parse::<f32>() {
                 return Some(val);
             }
        }
        
        // AMD: /sys/class/drm/card0/device/gpu_busy_percent (amdgpu)
        // Some kernels expose it differently.
        
        None
    }
}
