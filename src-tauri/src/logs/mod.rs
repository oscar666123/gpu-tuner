use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub action: String,
    pub success: bool,
    pub code: Option<String>,
    pub message: String,
}

pub struct LogStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl LogStore {
    pub fn new(app_name: &str) -> Self {
        let mut dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push(app_name);
        Self {
            path: dir.join("gpu-tuner.log"),
            lock: Mutex::new(()),
        }
    }

    pub fn write(
        &self,
        level: &str,
        action: &str,
        success: bool,
        code: Option<&str>,
        message: &str,
    ) -> Result<(), String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Log lock was poisoned.".to_string())?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create log directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            action: action.to_string(),
            success,
            code: code.map(ToOwned::to_owned),
            message: message.to_string(),
        };
        let line = serde_json::to_string(&entry)
            .map_err(|error| format!("Failed to serialize log entry: {error}"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| format!("Failed to open log file {}: {error}", self.path.display()))?;
        writeln!(file, "{line}")
            .map_err(|error| format!("Failed to write log file {}: {error}", self.path.display()))
    }

    pub fn read(&self) -> Result<Vec<LogEntry>, String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Log lock was poisoned.".to_string())?;
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|error| format!("Failed to read log file {}: {error}", self.path.display()))?;
        let mut entries = Vec::new();
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            if let Ok(entry) = serde_json::from_str::<LogEntry>(line) {
                entries.push(entry);
            }
        }
        entries.reverse();
        Ok(entries)
    }

    pub fn clear(&self) -> Result<(), String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| "Log lock was poisoned.".to_string())?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Failed to create log directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&self.path, "")
            .map_err(|error| format!("Failed to clear log file {}: {error}", self.path.display()))
    }
}
