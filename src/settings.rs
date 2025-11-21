use serde::{Deserialize, Serialize};
use std::fs;
use directories::ProjectDirs;
use auto_launch;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Theme {
    Dark,
    Light,
    Solarized,
    Tokyo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MonitorStyle {
    Bar,
    Chart,
    Text,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: Theme,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_network: bool,
    pub monitor_style: MonitorStyle,
    pub launch_at_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_network: true,
            monitor_style: MonitorStyle::Bar,
            launch_at_start: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("com", "moebius", "vitray-widget") {
            let config_dir = proj_dirs.config_dir();
            let config_file = config_dir.join("settings.json");
            
            if config_file.exists() {
                if let Ok(content) = fs::read_to_string(config_file) {
                    if let Ok(settings) = serde_json::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(proj_dirs) = ProjectDirs::from("com", "moebius", "vitray-widget") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            let config_file = config_dir.join("settings.json");
            let json = serde_json::to_string_pretty(self).unwrap_or_default();
            let _ = fs::write(config_file, json);
        }
    }

    pub fn set_auto_launch(&mut self, enable: bool) {
        self.launch_at_start = enable;
        self.save();

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_str) = exe_path.to_str() {
                let auto = auto_launch::AutoLaunchBuilder::new()
                    .set_app_name("vitray-widget")
                    .set_app_path(exe_str)
                    .set_use_launch_agent(true)
                    .build();

                if let Ok(auto) = auto {
                    if enable {
                        let _ = auto.enable();
                    } else {
                        let _ = auto.disable();
                    }
                }
            }
        }
    }
}
