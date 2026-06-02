use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::prompt::{system_prompt, user_prompt};

pub struct OllamaProvider {
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    stream: bool,
    format: String,
    messages: Vec<OllamaMessage>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessageOut,
}

#[derive(Deserialize)]
struct OllamaMessageOut {
    content: String,
}

impl OllamaProvider {
    pub async fn query(&self, query: &str, model_override: &Option<String>) -> Result<String> {
        let model = model_override.clone().unwrap_or_else(|| self.model.clone());
        let client = Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()?;

        let body = OllamaRequest {
            model,
            stream: false,
            format: "json".into(),
            messages: vec![
                OllamaMessage {
                    role: "system".into(),
                    content: system_prompt(),
                },
                OllamaMessage {
                    role: "user".into(),
                    content: user_prompt(query),
                },
            ],
        };

        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let dur = Duration::from_secs(self.timeout_secs);
        let resp = timeout(
            dur,
            client.post(&url).json(&body).send(),
        )
        .await
        .context("ollama request timed out")?
        .context("ollama request failed — is Ollama running?")?
        .error_for_status()
        .context("ollama API error")?
        .json::<OllamaResponse>()
        .await
        .context("parse ollama response")?;

        let text = resp.message.content.trim().to_string();
        if text.is_empty() {
            anyhow::bail!("empty ollama response");
        }
        Ok(text)
    }
}
