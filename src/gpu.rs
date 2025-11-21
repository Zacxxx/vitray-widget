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

        // Could add intel_gpu_top or radeontop parsing here in future

        None
    }
}
