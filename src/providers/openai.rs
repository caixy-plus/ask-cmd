use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::prompt::{system_prompt, user_prompt};

pub struct OpenAiProvider {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

impl OpenAiProvider {
    pub async fn query(&self, query: &str, model_override: &Option<String>) -> Result<String> {
        let model = model_override.clone().unwrap_or_else(|| self.model.clone());
        let client = Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()?;

        let body = ChatRequest {
            model,
            messages: vec![
                Message {
                    role: "system".into(),
                    content: system_prompt(),
                },
                Message {
                    role: "user".into(),
                    content: user_prompt(query),
                },
            ],
            temperature: 0.1,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let dur = Duration::from_secs(self.timeout_secs);
        let resp = timeout(
            dur,
            client
                .post(&url)
                .bearer_auth(&self.api_key)
                .json(&body)
                .send(),
        )
        .await
        .context("openai request timed out")?
        .context("openai request failed")?
        .error_for_status()
        .context("openai API error")?
        .json::<ChatResponse>()
        .await
        .context("parse openai response")?;

        resp.choices
            .into_iter()
            .next()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("empty openai response"))
    }
}
