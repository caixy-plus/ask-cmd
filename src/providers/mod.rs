mod claude;
mod ollama;
mod openai;

use anyhow::{bail, Result};

use crate::config::Config;

pub enum Provider {
    Claude(claude::ClaudeProvider),
    OpenAi(openai::OpenAiProvider),
    Ollama(ollama::OllamaProvider),
}

impl Provider {
    pub fn from_config(config: &Config) -> Result<Self> {
        match config.provider.as_str() {
            "claude" => Ok(Self::Claude(claude::ClaudeProvider {
                timeout_secs: config.timeout_secs,
            })),
            "openai" => Ok(Self::OpenAi(openai::OpenAiProvider {
                api_key: config
                    .openai_api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| anyhow::anyhow!("set openai_api_key in config or OPENAI_API_KEY"))?,
                base_url: config.openai_base_url.clone(),
                model: config.openai_model.clone(),
                timeout_secs: config.timeout_secs,
            })),
            "ollama" => Ok(Self::Ollama(ollama::OllamaProvider {
                base_url: config.ollama_url.clone(),
                model: config.ollama_model.clone(),
                timeout_secs: config.timeout_secs,
            })),
            other => bail!("unknown provider '{other}' (claude|openai|ollama)"),
        }
    }

    pub async fn query(&self, query: &str, model_override: &Option<String>) -> Result<String> {
        match self {
            Self::Claude(p) => p.query(query, model_override).await,
            Self::OpenAi(p) => p.query(query, model_override).await,
            Self::Ollama(p) => p.query(query, model_override).await,
        }
    }
}

// Sync wrapper for lib — providers are async
impl Provider {
    pub fn query_sync(&self, query: &str, model: &Option<String>) -> Result<String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(self.query(query, model))
    }
}
