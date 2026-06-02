pub mod config;
pub mod extract;
pub mod prompt;
pub mod providers;
pub mod shell;
pub mod validate;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::providers::Provider;

pub fn resolve_command(query: &str, config: &Config) -> Result<String> {
    let provider = Provider::from_config(config)?;
    let raw = provider
        .query_sync(query, &config.model)
        .context("AI provider request failed")?;
    let cmd = extract::extract_command(&raw).context("could not extract a shell command from AI response")?;
    validate::validate_command(&cmd).context(format!("invalid command: {cmd}"))?;
    Ok(cmd)
}
