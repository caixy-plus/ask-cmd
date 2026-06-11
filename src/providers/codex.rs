use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::process::Command;
use tokio::time::timeout;

use crate::prompt::{single_system_prompt, single_user_prompt, suggestions_system_prompt, suggestions_user_prompt};
use crate::providers::provider::Provider;

pub const DEFAULT_TIMEOUT_SECS: u64 = 45;

pub fn codex_available() -> bool {
    resolve_codex_binary().is_some()
}

fn resolve_codex_binary() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in codex_binary_names() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn codex_binary_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["codex.exe", "codex.cmd", "codex.bat", "codex"]
    } else {
        &["codex"]
    }
}

pub fn codex_install_hint() -> &'static str {
    "\
未检测到 Codex CLI（codex 命令）。

`ask`（Rust CLI）通过本机 Codex 将自然语言转换为 shell 命令，不支持自行接入 API Key。

请先安装并登录 Codex：

  npm install -g @openai/codex
  codex login

安装文档：https://developers.openai.com/codex/cli

安装完成后重新打开终端，运行 `codex exec hello` 验证是否正常。"
}

pub struct CodexProvider {
    pub timeout_secs: u64,
}

impl Default for CodexProvider {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

impl Provider for CodexProvider {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    fn available(&self) -> bool {
        codex_available()
    }

    fn suggest_sync(&self, query: &str) -> Result<String> {
        let prompt = format!("{}\n\n{}", suggestions_system_prompt(), suggestions_user_prompt(query));
        self.run_codex(&prompt)
    }

    fn query_sync(&self, query: &str) -> Result<String> {
        let prompt = format!("{}\n\n{}", single_system_prompt(), single_user_prompt(query));
        self.run_codex(&prompt)
    }

    fn install_hint(&self) -> &'static str {
        codex_install_hint()
    }
}

impl CodexProvider {
    fn run_codex(&self, prompt: &str) -> Result<String> {
        if !codex_available() {
            bail!("{}", codex_install_hint())
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(self.run_codex_async(prompt))
    }

    async fn run_codex_async(&self, prompt: &str) -> Result<String> {
        let codex = resolve_codex_binary().expect("checked above");
        let mut cmd = Command::new(codex);
        // read-only 沙箱 + 跳过 git 仓库检查（非 git 目录下 codex exec 会拒绝运行）+ 不落盘 session
        cmd.args(["exec", "--sandbox", "read-only", "--skip-git-repo-check", "--ephemeral", prompt])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let dur = Duration::from_secs(self.timeout_secs);
        let output = timeout(dur, cmd.output())
            .await
            .context("codex 请求超时，请稍后重试")?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!("{}", codex_install_hint())
                } else {
                    anyhow::anyhow!("无法运行 codex：{e}")
                }
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && output.stdout.is_empty() {
            if stderr.contains("auth") || stderr.contains("login") || stderr.contains("API key") {
                bail!("codex 未登录或认证失败。\n\n请运行：\n  codex login\n\n然后重试 ask 命令。");
            }
            bail!("codex 执行失败 ({}).\n\n{stderr}", output.status);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            bail!("codex 返回为空。\n\n请确认已登录：codex login");
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_hint_mentions_codex() {
        assert!(codex_install_hint().contains("Codex"));
    }

    #[test]
    fn provider_name() {
        let p = CodexProvider::default();
        assert_eq!(p.name(), "codex");
        assert_eq!(p.display_name(), "Codex");
    }
}
