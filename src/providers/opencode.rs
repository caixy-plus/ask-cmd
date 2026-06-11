use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::process::Command;
use tokio::time::timeout;

use crate::prompt::{single_system_prompt, single_user_prompt, suggestions_system_prompt, suggestions_user_prompt};
use crate::providers::provider::Provider;

pub const DEFAULT_TIMEOUT_SECS: u64 = 45;

pub fn opencode_available() -> bool {
    resolve_opencode_binary().is_some()
}

fn resolve_opencode_binary() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in opencode_binary_names() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn opencode_binary_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["opencode.exe", "opencode.cmd", "opencode.bat", "opencode"]
    } else {
        &["opencode"]
    }
}

pub fn opencode_install_hint() -> &'static str {
    "\
未检测到 OpenCode CLI（opencode 命令）。

`ask`（Rust CLI）通过本机 OpenCode 将自然语言转换为 shell 命令，不支持自行接入 API Key。

请先安装并登录 OpenCode：

  npm install -g opencode-ai
  opencode auth login

安装文档：https://opencode.ai/docs/cli

安装完成后重新打开终端，运行 `opencode run hello` 验证是否正常。"
}

pub struct OpenCodeProvider {
    pub timeout_secs: u64,
}

impl Default for OpenCodeProvider {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

impl Provider for OpenCodeProvider {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn display_name(&self) -> &'static str {
        "OpenCode"
    }

    fn available(&self) -> bool {
        opencode_available()
    }

    fn suggest_sync(&self, query: &str) -> Result<String> {
        let prompt = format!("{}\n\n{}", suggestions_system_prompt(), suggestions_user_prompt(query));
        self.run_opencode(&prompt)
    }

    fn query_sync(&self, query: &str) -> Result<String> {
        let prompt = format!("{}\n\n{}", single_system_prompt(), single_user_prompt(query));
        self.run_opencode(&prompt)
    }

    fn install_hint(&self) -> &'static str {
        opencode_install_hint()
    }
}

impl OpenCodeProvider {
    fn run_opencode(&self, prompt: &str) -> Result<String> {
        if !opencode_available() {
            bail!("{}", opencode_install_hint())
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(self.run_opencode_async(prompt))
    }

    async fn run_opencode_async(&self, prompt: &str) -> Result<String> {
        let opencode = resolve_opencode_binary().expect("checked above");
        let mut cmd = Command::new(opencode);
        // 内置 plan agent：只读，不能改文件，权限最小
        cmd.args(["run", "--agent", "plan", prompt])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let dur = Duration::from_secs(self.timeout_secs);
        let output = timeout(dur, cmd.output())
            .await
            .context("opencode 请求超时，请稍后重试")?
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow::anyhow!("{}", opencode_install_hint())
                } else {
                    anyhow::anyhow!("无法运行 opencode：{e}")
                }
            })?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() && output.stdout.is_empty() {
            if stderr.contains("auth") || stderr.contains("login") || stderr.contains("API key") {
                bail!("opencode 未登录或认证失败。\n\n请运行：\n  opencode auth login\n\n然后重试 ask 命令。");
            }
            bail!("opencode 执行失败 ({}).\n\n{stderr}", output.status);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            bail!("opencode 返回为空。\n\n请确认已登录：opencode auth login");
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_hint_mentions_opencode() {
        assert!(opencode_install_hint().contains("OpenCode"));
    }

    #[test]
    fn provider_name() {
        let p = OpenCodeProvider::default();
        assert_eq!(p.name(), "opencode");
        assert_eq!(p.display_name(), "OpenCode");
    }
}
