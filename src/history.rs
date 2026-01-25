use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_HISTORY: usize = 100;

pub struct History {
    db: sled::Db,
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
        PathBuf::from(home).join(".knock").join("history")
    }

    pub fn load() -> Self {
        let db = sled::open(Self::history_path()).expect("Failed to open history database");
        Self { db }
    }

    pub fn add(&self, query: String, command: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = HistoryEntry {
            query,
            command,
            timestamp,
        };

        // Use timestamp as key (padded for proper ordering)
        let key = format!("{:020}", timestamp);
        if let Ok(value) = serde_json::to_vec(&entry) {
            let _ = self.db.insert(key.as_bytes(), value);
        }

        // Prune old entries if over limit
        self.prune();
    }

    fn prune(&self) {
        let count = self.db.len();
        if count > MAX_HISTORY {
            let to_remove = count - MAX_HISTORY;
            let keys: Vec<_> = self.db
                .iter()
                .take(to_remove)
                .filter_map(|r| r.ok().map(|(k, _)| k))
                .collect();

            for key in keys {
                let _ = self.db.remove(key);
            }
        }
    }

    pub fn search(&self, pattern: &str) -> Vec<HistoryEntry> {
        let pattern_lower = pattern.to_lowercase();
        self.db
            .iter()
            .filter_map(|r| r.ok())
            .filter_map(|(_, v)| serde_json::from_slice::<HistoryEntry>(&v).ok())
            .filter(|e| {
                e.query.to_lowercase().contains(&pattern_lower)
                    || e.command.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }

    pub fn recent(&self, count: usize) -> Vec<HistoryEntry> {
        self.db
            .iter()
            .rev()
            .take(count)
            .filter_map(|r| r.ok())
            .filter_map(|(_, v)| serde_json::from_slice::<HistoryEntry>(&v).ok())
            .collect()
    }
}
