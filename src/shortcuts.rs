use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub name: String,
    pub command: String,
    #[serde(default = "default_timestamp")]
    pub created_at: u64,
}

fn default_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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

    pub fn add(&mut self, name: String, command: String) -> Result<(), String> {
        // Enforce unique, slug-safe names
        let slug = name.trim().replace(" ", "-").to_lowercase();
        if self.items.iter().any(|s| s.name.to_lowercase() == slug) {
            return Err(format!("Shortcut '{}' already exists", slug));
        }
        
        self.items.push(Shortcut { 
            name: slug, 
            command,
            created_at: default_timestamp(),
        });
        self.save();
        Ok(())
    }

    pub fn remove_by_name(&mut self, name: &str) -> bool {
        if let Some(pos) = self.items.iter().position(|s| s.name == name) {
            self.items.remove(pos);
            self.save();
            return true;
        }
        false
    }



    pub fn rename(&mut self, old_name: &str, new_name: String, command: String) -> Result<(), String> {
        // Check if new name conflicts (unless it's the same name)
        if old_name != new_name && self.items.iter().any(|s| s.name == new_name) {
             return Err(format!("Shortcut '{}' already exists", new_name));
        }

        if let Some(existing) = self.items.iter_mut().find(|s| s.name == old_name) {
            existing.name = new_name;
            existing.command = command;
            self.save();
            return Ok(());
        }
        Err("Shortcut not found".to_string())
    }

    pub fn find(&self, name: &str) -> Option<Shortcut> {
        self.items.iter().find(|s| s.name == name).cloned()
    }
}
