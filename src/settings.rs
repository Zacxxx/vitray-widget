use auto_launch;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

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
pub struct SectionStyle {
    pub opacity: f64,
    pub bg_color: String, // Hex code, e.g., "#12161e"
    pub font_size: f64,
    pub border_radius: f64,
    #[serde(default = "default_font")]
    pub font_family: String,
}

fn default_font() -> String {
    "Sans".to_string()
}

impl Default for SectionStyle {
    fn default() -> Self {
        Self {
            opacity: 0.78,
            bg_color: "#12161e".to_string(),
            font_size: 11.0,
            border_radius: 18.0,
            font_family: "Sans".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WidgetLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Default for WidgetLayout {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 200.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    // TODO(senior-ui): Persist window geometry, opacity, and monitor affinity to avoid the widget
    // jumping between workspaces each launch.
    pub theme: Theme,
    pub show_terminal: bool,
    pub show_monitoring: bool,
    pub show_shortcuts_panel: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_network: bool,
    pub monitor_style: MonitorStyle,
    pub launch_at_start: bool,
    pub lock_in_place: bool,
    pub lock_size: bool,
    #[serde(default = "default_shell")]
    pub shell: String,

    // New fields for detachable sections
    pub terminal_style: SectionStyle,
    pub monitoring_style: SectionStyle,
    pub shortcuts_style: SectionStyle,
    
    pub terminal_layout: WidgetLayout,
    pub monitoring_layout: WidgetLayout,
    pub shortcuts_layout: WidgetLayout,
}

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or("/bin/bash".to_string())
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            show_terminal: true,
            show_monitoring: true,
            show_shortcuts_panel: true,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_network: true,
            monitor_style: MonitorStyle::Text,
            launch_at_start: false,
            lock_in_place: true,
            lock_size: true,
            shell: default_shell(),

            terminal_style: SectionStyle {
                font_family: "Monospace".to_string(),
                ..SectionStyle::default()
            },
            monitoring_style: SectionStyle::default(),
            shortcuts_style: SectionStyle::default(),

            terminal_layout: WidgetLayout { x: 50.0, y: 50.0, width: 600.0, height: 400.0 },
            monitoring_layout: WidgetLayout { x: 50.0, y: 470.0, width: 600.0, height: 200.0 },
            shortcuts_layout: WidgetLayout { x: 670.0, y: 50.0, width: 250.0, height: 620.0 },
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
                // TODO(senior-ui): Surface JSON parsing errors in-app so users know why their
                // preferences reset instead of silently reverting to defaults.
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
                // TODO(senior-ui): Detect init system (systemd, gnome-session, sway) and register
                // the autostart entry appropriately instead of relying solely on launch agents.
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
