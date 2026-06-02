use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::process::Command;
use tokio::time::timeout;

use crate::prompt::{system_prompt, user_prompt};

pub const DEFAULT_TIMEOUT_SECS: u64 = 45;

/// Check whether `claude` is available on PATH.
pub fn claude_available() -> bool {
    resolve_claude_binary().is_some()
}

fn resolve_claude_binary() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in claude_binary_names() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn claude_binary_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["claude.exe", "claude.cmd", "claude.bat", "claude"]
    } else {
        &["claude"]
    }
}

pub fn claude_install_hint() -> &'static str {
    "\
未检测到 Claude Code CLI（claude 命令）。

ask-cmd 通过本机 Claude Code 将自然语言转换为 shell 命令，不支持自行接入 API Key。

请先安装并登录 Claude Code：

  npm install -g @anthropic-ai/claude-code
  claude login

安装文档：https://docs.anthropic.com/en/docs/claude-code

安装完成后重新打开终端，运行 `claude -p hello` 验证是否正常。"
}

pub fn ensure_claude() -> Result<()> {
    if claude_available() {
        return Ok(());
    }
    bail!("{}", claude_install_hint())
}

pub struct ClaudeProvider {
    pub timeout_secs: u64,
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

impl ClaudeProvider {
    pub async fn query(&self, query: &str) -> Result<String> {
        ensure_claude()?;

        let claude = resolve_claude_binary().expect("checked above");
        let mut cmd = Command::new(claude);
        cmd.args([
            "--bare",
            "-p",
            &user_prompt(query),
            "--system-prompt",
            &system_prompt(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        let dur = Duration::from_secs(self.timeout_secs);
        let output = timeout(dur, cmd.output())
            .await
            .context("claude 请求超时，请稍后重试")?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!("{}", claude_install_hint())
                } else {
                    anyhow::anyhow!("无法运行 claude：{e}")
                }
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && output.stdout.is_empty() {
            if stderr.contains("auth") || stderr.contains("login") || stderr.contains("API key") {
                bail!(
                    "claude 未登录或认证失败。\n\n请运行：\n  claude login\n\n然后重试 ask 命令。"
                );
            }
            bail!(
                "claude 执行失败 ({}).\n\n{stderr}",
                output.status
            );
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            bail!(
                "claude 返回为空。\n\n请确认已登录：claude login\n提示：claude -p \"hello\" 应能正常输出。"
            );
        }
        Ok(text)
    }

    pub fn query_sync(&self, query: &str) -> Result<String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(self.query(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_hint_mentions_claude_code() {
        assert!(claude_install_hint().contains("Claude Code"));
        assert!(claude_install_hint().contains("claude login"));
    }

    #[test]
    fn ensure_claude_fails_with_hint_when_missing() {
        if claude_available() {
            return;
        }
        let err = ensure_claude().unwrap_err().to_string();
        assert!(err.contains("Claude Code"));
    }

    #[tokio::test]
    #[ignore = "requires claude CLI"]
    async fn live_claude() {
        let p = ClaudeProvider::default();
        let out = p.query("create empty file /tmp/ask-cmd-test.txt").await.unwrap();
        assert!(out.contains("touch") || out.contains("New-Item"));
    }
}
