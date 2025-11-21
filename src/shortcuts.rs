use serde::{Deserialize, Serialize};
use std::fs;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Shortcuts {
    pub items: Vec<Shortcut>,
}

impl Shortcuts {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("com", "moebius", "vitray-widget") {
            let config_dir = proj_dirs.config_dir();
            let file_path = config_dir.join("shortcuts.json");
            
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(file_path) {
                    if let Ok(shortcuts) = serde_json::from_str(&content) {
                        return shortcuts;
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
            let file_path = config_dir.join("shortcuts.json");
            let json = serde_json::to_string_pretty(self).unwrap_or_default();
            let _ = fs::write(file_path, json);
        }
    }

    pub fn add(&mut self, name: String, command: String) {
        self.items.push(Shortcut { name, command });
        self.save();
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            self.save();
        }
    }
}
