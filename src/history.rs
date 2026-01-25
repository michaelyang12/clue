use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

const MAX_HISTORY: usize = 100;

#[derive(Serialize, Deserialize, Default)]
pub struct History {
    entries: Vec<HistoryEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub query: String,
    pub command: String,
    pub timestamp: u64,
}

impl History {
    fn history_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".knock").join("history.json")
    }

    pub fn load() -> Self {
        let path = Self::history_path();
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(e) if e.kind() == ErrorKind::NotFound => Self::default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn add(&mut self, query: String, command: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.entries.push(HistoryEntry {
            query,
            command,
            timestamp,
        });

        // Keep only the last MAX_HISTORY entries
        if self.entries.len() > MAX_HISTORY {
            self.entries = self.entries.split_off(self.entries.len() - MAX_HISTORY);
        }
    }

    pub fn search(&self, pattern: &str) -> Vec<&HistoryEntry> {
        let pattern_lower = pattern.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.query.to_lowercase().contains(&pattern_lower)
                    || e.command.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }

    pub fn recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(count).collect()
    }
}
