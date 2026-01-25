use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Ollama,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::OpenAI
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub provider: Provider,
    pub openai_model: Option<String>,
    pub anthropic_model: Option<String>,
    pub ollama_model: Option<String>,
    pub ollama_url: Option<String>,
}

impl Config {
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".knock").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(e) if e.kind() == ErrorKind::NotFound => Self::default(),
            Err(_) => Self::default(),
        }
    }

    pub fn openai_model(&self) -> &str {
        self.openai_model.as_deref().unwrap_or("gpt-4o-mini")
    }

    pub fn anthropic_model(&self) -> &str {
        self.anthropic_model.as_deref().unwrap_or("claude-sonnet-4-20250514")
    }

    pub fn ollama_model(&self) -> &str {
        self.ollama_model.as_deref().unwrap_or("llama3.2")
    }

    pub fn ollama_url(&self) -> &str {
        self.ollama_url.as_deref().unwrap_or("http://localhost:11434")
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).unwrap();
        std::fs::write(path, json)
    }
}
