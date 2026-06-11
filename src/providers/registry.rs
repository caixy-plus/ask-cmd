use anyhow::{bail, Context, Result};

use crate::config::Config;
use crate::providers::claude::ClaudeProvider;
use crate::providers::codex::CodexProvider;
use crate::providers::cursor::CursorProvider;
use crate::providers::opencode::OpenCodeProvider;
use crate::providers::provider::Provider;

pub fn all_providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(ClaudeProvider::default()),
        Box::new(OpenCodeProvider::default()),
        Box::new(CodexProvider::default()),
        Box::new(CursorProvider::default()),
    ]
}

pub fn available_providers() -> Vec<Box<dyn Provider>> {
    all_providers().into_iter().filter(|p| p.available()).collect()
}

pub fn find_provider(name: &str) -> Option<Box<dyn Provider>> {
    all_providers().into_iter().find(|p| p.name() == name)
}

pub fn resolve_provider(preferred: Option<&str>) -> Result<Box<dyn Provider>> {
    // 1. CLI 参数指定
    if let Some(name) = preferred {
        if let Some(p) = find_provider(name) {
            if p.available() {
                return Ok(p);
            }
            bail!(
                "指定的智能体 '{}' 未安装或未配置。\n\n{}",
                p.name(),
                p.install_hint()
            );
        }
        bail!(
            "未知的智能体 '{}'。可用：{}",
            name,
            list_available_names()
        );
    }

    // 2. 环境变量
    if let Ok(env_name) = std::env::var("ASK_PROVIDER") {
        if let Some(p) = find_provider(&env_name) {
            if p.available() {
                return Ok(p);
            }
            bail!(
                "ASK_PROVIDER 指定的智能体 '{}' 未安装或未配置。\n\n{}",
                env_name,
                p.install_hint()
            );
        }
    }

    // 3. 配置文件
    let config = Config::load().context("load config")?;
    if let Some(name) = config.default_provider_name() {
        if let Some(p) = find_provider(name) {
            if p.available() {
                return Ok(p);
            }
            bail!(
                "配置文件默认智能体 '{}' 未安装或未配置。\n\n{}",
                name,
                p.install_hint()
            );
        }
    }

    // 4. 自动检测第一个可用的
    let available = available_providers();
    if let Some(p) = available.into_iter().next() {
        return Ok(p);
    }

    bail!(
        "未检测到任何支持的 AI CLI 智能体。请安装至少一个：\n\n{}\n\n安装后重新运行 ask。",
        all_install_hints()
    )
}

pub fn list_available_names() -> String {
    available_providers()
        .iter()
        .map(|p| format!("{} ({})", p.name(), p.display_name()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn all_install_hints() -> String {
    all_providers()
        .iter()
        .map(|p| format!("- {}: {}", p.display_name(), p.install_hint()))
        .collect::<Vec<_>>()
        .join("\n\n")
}
