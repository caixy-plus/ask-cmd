use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default_provider: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_provider: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).context("read config")?;
        let config: Config = serde_json::from_str(&content).context("parse config")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("create config dir")?;
        }
        let content = serde_json::to_string_pretty(self).context("serialize config")?;
        fs::write(&path, content).context("write config")?;
        Ok(())
    }

    pub fn default_provider_name(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    pub fn set_default_provider(&mut self, name: String) {
        self.default_provider = Some(name);
    }
}

fn config_path() -> Result<PathBuf> {
    Ok(dirs::config_dir()
        .context("config dir")?
        .join("ask-cmd")
        .join("config.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn config_roundtrip() {
        let mut tmp = NamedTempFile::new().unwrap();
        let cfg = Config {
            default_provider: Some("claude".to_string()),
        };
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        tmp.write_all(json.as_bytes()).unwrap();
        tmp.flush().unwrap();

        let content = fs::read_to_string(tmp.path()).unwrap();
        let loaded: Config = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.default_provider, Some("claude".to_string()));
    }
}
