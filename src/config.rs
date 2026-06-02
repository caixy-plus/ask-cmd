use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub provider: String,
    pub model: Option<String>,
    pub timeout_secs: u64,
    pub openai_api_key: Option<String>,
    pub openai_base_url: String,
    pub openai_model: String,
    pub ollama_url: String,
    pub ollama_model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "claude".into(),
            model: None,
            timeout_secs: 45,
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_base_url: "https://api.openai.com/v1".into(),
            openai_model: "gpt-4o-mini".into(),
            ollama_url: "http://127.0.0.1:11434".into(),
            ollama_model: "llama3.2".into(),
        }
    }
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let base = dirs::config_dir().context("cannot resolve config directory")?;
        Ok(base.join("ask-cmd").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let mut cfg: Config = toml::from_str(&text).context("parse config.toml")?;
        if cfg.openai_api_key.is_none() {
            cfg.openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        }
        Ok(cfg)
    }

    pub fn save_default() -> Result<PathBuf> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            let text = toml::to_string_pretty(&Config::default())?;
            fs::write(&path, text)?;
        }
        Ok(path)
    }
}
