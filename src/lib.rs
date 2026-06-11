pub mod config;
pub mod extract;
pub mod prompt;
pub mod providers;
pub mod shell;
pub mod suggestions;
pub mod ui;
pub mod validate;

use anyhow::{Context, Result};

use crate::suggestions::parse_suggestions;

pub fn resolve_suggestions(query: &str, preferred: Option<&str>) -> Result<Vec<String>> {
    let provider = providers::registry::resolve_provider(preferred)?;
    let raw = provider
        .suggest_sync(query)
        .with_context(|| format!("{} 请求失败", provider.display_name()))?;
    parse_suggestions(&raw)
}

/// Single command (used by `-n` fast path fallback).
pub fn resolve_command(query: &str, preferred: Option<&str>) -> Result<String> {
    let provider = providers::registry::resolve_provider(preferred)?;
    let raw = provider
        .query_sync(query)
        .with_context(|| format!("{} 请求失败", provider.display_name()))?;
    let cmd = extract::extract_command(&raw).with_context(|| {
        format!(
            "无法从 {} 回复中提取 shell 命令，请说得更具体，例如：创建空文件 test.txt",
            provider.display_name()
        )
    })?;
    validate::validate_command(&cmd).context(format!("无效命令: {cmd}"))?;
    Ok(cmd)
}

pub use providers::{
    claude_available, claude_install_hint, codex_available, codex_install_hint, cursor_available,
    cursor_install_hint, opencode_available, opencode_install_hint,
};
pub use ui::picker::{pick_command, PickResult};
