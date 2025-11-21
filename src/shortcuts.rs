use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

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

    pub fn remove_by_name(&mut self, name: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|s| s.name == name) {
            self.items.remove(pos);
            self.save();
            return true;
        }
        false
    }

    pub fn upsert(&mut self, name: String, command: String) {
        if let Some(existing) = self.items.iter_mut().find(|s| s.name == name) {
            existing.command = command;
        } else {
            self.items.push(Shortcut { name, command });
        }
        self.save();
    }

    pub fn rename(&mut self, old_name: &str, new_name: String, command: String) -> bool {
        if let Some(existing) = self.items.iter_mut().find(|s| s.name == old_name) {
            existing.name = new_name;
            existing.command = command;
            self.save();
            return true;
        }
        false
    }

    pub fn find(&self, name: &str) -> Option<Shortcut> {
        self.items.iter().find(|s| s.name == name).cloned()
    }
}
