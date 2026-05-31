use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub gpu_index: u32,
    pub power_limit_watts: Option<f64>,
    pub core_clock_offset_mhz: Option<i32>,
    pub memory_clock_offset_mhz: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct ProfileStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl ProfileStore {
    pub fn new(app_name: &str) -> Self {
        let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push(app_name);
        Self {
            path: dir.join("profiles.json"),
            lock: Mutex::new(()),
        }
    }

    pub fn list(&self) -> Result<Vec<Profile>, String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Profile lock was poisoned.".to_string())?;
        self.read_unlocked()
    }

    pub fn load(&self, name: &str) -> Result<Profile, String> {
        self.list()?
            .into_iter()
            .find(|profile| profile.name == name)
            .ok_or_else(|| format!("Profile '{name}' was not found."))
    }

    pub fn save(&self, mut profile: Profile) -> Result<(), String> {
        if profile.name.trim().is_empty() {
            return Err("Profile name is required.".to_string());
        }
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Profile lock was poisoned.".to_string())?;
        let mut profiles = self.read_unlocked()?;
        let now = Utc::now().to_rfc3339();
        if profile.created_at.trim().is_empty() {
            profile.created_at = now.clone();
        }
        profile.updated_at = now;

        if let Some(existing) = profiles.iter_mut().find(|item| item.name == profile.name) {
            profile.created_at = existing.created_at.clone();
            *existing = profile;
        } else {
            profiles.push(profile);
        }
        self.write_unlocked(&profiles)
    }

    pub fn delete(&self, name: &str) -> Result<(), String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Profile lock was poisoned.".to_string())?;
        let mut profiles = self.read_unlocked()?;
        let original_len = profiles.len();
        profiles.retain(|profile| profile.name != name);
        if profiles.len() == original_len {
            return Err(format!("Profile '{name}' was not found."));
        }
        self.write_unlocked(&profiles)
    }

    fn read_unlocked(&self) -> Result<Vec<Profile>, String> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path).map_err(|error| {
            format!(
                "Failed to read profiles file {}: {error}",
                self.path.display()
            )
        })?;
        serde_json::from_str(&content)
            .map_err(|error| format!("Failed to parse profiles JSON: {error}"))
    }

    fn write_unlocked(&self, profiles: &[Profile]) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create profile directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let content = serde_json::to_string_pretty(profiles)
            .map_err(|error| format!("Failed to serialize profiles: {error}"))?;
        fs::write(&self.path, content).map_err(|error| {
            format!(
                "Failed to write profiles file {}: {error}",
                self.path.display()
            )
        })
    }
}
