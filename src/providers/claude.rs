use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::time::timeout;

use crate::prompt::{system_prompt, user_prompt};

pub struct ClaudeProvider {
    pub timeout_secs: u64,
}

impl ClaudeProvider {
    pub async fn query(&self, query: &str, model: &Option<String>) -> Result<String> {
        let mut cmd = Command::new("claude");
        cmd.args([
            "--bare",
            "-p",
            &user_prompt(query),
            "--system-prompt",
            &system_prompt(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

        if let Some(m) = model {
            cmd.args(["--model", m]);
        }

        let dur = Duration::from_secs(self.timeout_secs);
        let output = timeout(dur, cmd.output())
            .await
            .context("claude timed out")?
            .context("failed to run claude — is it installed and logged in?")?;

        if !output.status.success() && output.stdout.is_empty() {
            anyhow::bail!("claude exited with {}", output.status);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            anyhow::bail!("empty response from claude");
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires claude CLI"]
    async fn live_claude() {
        let p = ClaudeProvider { timeout_secs: 60 };
        let out = p.query("create empty file /tmp/ask-cmd-test.txt", &None).await.unwrap();
        assert!(out.contains("touch") || out.contains("New-Item"));
    }
}
