use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::process::Command;
use tokio::time::timeout;

use crate::prompt::{single_system_prompt, single_user_prompt, suggestions_system_prompt, suggestions_user_prompt};
use crate::providers::provider::Provider;

pub const DEFAULT_TIMEOUT_SECS: u64 = 45;

pub fn cursor_available() -> bool {
    resolve_cursor_binary().is_some()
}

fn resolve_cursor_binary() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in cursor_binary_names() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn cursor_binary_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["agent.exe", "cursor-agent.exe", "agent", "cursor-agent"]
    } else {
        &["agent", "cursor-agent"]
    }
}

pub fn cursor_install_hint() -> &'static str {
    "\
未检测到 Cursor Agent CLI（agent 或 cursor-agent 命令）。

`ask`（Rust CLI）通过本机 Cursor Agent 将自然语言转换为 shell 命令，不支持自行接入 API Key。

请先安装并登录 Cursor Agent：

  curl https://cursor.com/install -fsS | bash
  agent login

安装文档：https://cursor.com/docs/cli

安装完成后重新打开终端，运行 `agent -p hello` 验证是否正常。"
}

pub struct CursorProvider {
    pub timeout_secs: u64,
}

impl Default for CursorProvider {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

impl Provider for CursorProvider {
    fn name(&self) -> &'static str {
        "cursor"
    }

    fn display_name(&self) -> &'static str {
        "Cursor Agent"
    }

    fn available(&self) -> bool {
        cursor_available()
    }

    fn suggest_sync(&self, query: &str) -> Result<String> {
        let prompt = format!("{}\n\n{}", suggestions_system_prompt(), suggestions_user_prompt(query));
        self.run_cursor(&prompt)
    }

    fn query_sync(&self, query: &str) -> Result<String> {
        let prompt = format!("{}\n\n{}", single_system_prompt(), single_user_prompt(query));
        self.run_cursor(&prompt)
    }

    fn install_hint(&self) -> &'static str {
        cursor_install_hint()
    }
}

impl CursorProvider {
    fn run_cursor(&self, prompt: &str) -> Result<String> {
        if !cursor_available() {
            bail!("{}", cursor_install_hint())
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(self.run_cursor_async(prompt))
    }

    async fn run_cursor_async(&self, prompt: &str) -> Result<String> {
        let cursor = resolve_cursor_binary().expect("checked above");
        let mut cmd = Command::new(cursor);
        // --mode ask: 只读问答模式；--trust: headless 下跳过工作区信任弹窗（配合只读模式风险可控）
        cmd.args(["-p", "--mode", "ask", "--trust", "--output-format", "text", prompt])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let dur = Duration::from_secs(self.timeout_secs);
        let output = timeout(dur, cmd.output())
            .await
            .context("cursor 请求超时，请稍后重试")?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!("{}", cursor_install_hint())
                } else {
                    anyhow::anyhow!("无法运行 cursor agent：{e}")
                }
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && output.stdout.is_empty() {
            if stderr.contains("auth") || stderr.contains("login") || stderr.contains("API key") {
                bail!("cursor 未登录或认证失败。\n\n请运行：\n  agent login\n\n然后重试 ask 命令。");
            }
            bail!("cursor 执行失败 ({}).\n\n{stderr}", output.status);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            bail!("cursor 返回为空。\n\n请确认已登录：agent login");
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_hint_mentions_cursor() {
        assert!(cursor_install_hint().contains("Cursor"));
    }

    #[test]
    fn provider_name() {
        let p = CursorProvider::default();
        assert_eq!(p.name(), "cursor");
        assert_eq!(p.display_name(), "Cursor Agent");
    }
}
