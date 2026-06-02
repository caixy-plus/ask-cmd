pub mod extract;
pub mod prompt;
pub mod providers;
pub mod shell;
pub mod suggestions;
pub mod ui;
pub mod validate;

use anyhow::{Context, Result};

use crate::providers::{ensure_claude, ClaudeProvider};
use crate::suggestions::parse_suggestions;

pub fn resolve_suggestions(query: &str) -> Result<Vec<String>> {
    ensure_claude()?;
    let raw = ClaudeProvider::default()
        .suggest_sync(query)
        .context("Claude 请求失败")?;
    parse_suggestions(&raw)
}

/// Single command (used by `-n` fast path fallback).
pub fn resolve_command(query: &str) -> Result<String> {
    ensure_claude()?;
    let raw = ClaudeProvider::default()
        .query_sync(query)
        .context("Claude 请求失败")?;
    let cmd = extract::extract_command(&raw)
        .context("无法从 Claude 回复中提取 shell 命令，请说得更具体，例如：创建空文件 test.txt")?;
    validate::validate_command(&cmd).context(format!("无效命令: {cmd}"))?;
    Ok(cmd)
}

pub use providers::{claude_available, claude_install_hint};
pub use ui::picker::{pick_command, PickResult};
