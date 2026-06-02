use anyhow::{bail, Result};
use serde_json::Value;

use crate::validate::looks_like_command;

pub fn extract_command(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("empty response");
    }

    if let Some(cmd) = extract_from_json(trimmed) {
        if looks_like_command(&cmd) {
            return Ok(cmd);
        }
    }

    if let Some(cmd) = extract_from_markdown(trimmed) {
        return Ok(cmd);
    }

    if let Some(cmd) = extract_first_command_line(trimmed) {
        return Ok(cmd);
    }

    bail!("no command-like line found")
}

fn extract_from_json(raw: &str) -> Option<String> {
    let value: Value = serde_json::from_str(raw).ok()?;
    value
        .get("command")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn extract_from_markdown(raw: &str) -> Option<String> {
    let mut in_block = false;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("```") {
            in_block = !in_block;
            continue;
        }
        if in_block && looks_like_command(line) {
            return Some(line.to_string());
        }
    }

    if let Some(start) = raw.find('`') {
        let rest = &raw[start + 1..];
        if let Some(end) = rest.find('`') {
            let inner = rest[..end].trim();
            if !inner.contains('\n') && looks_like_command(inner) {
                return Some(inner.to_string());
            }
        }
    }

    None
}

fn extract_first_command_line(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let line = line.trim();
        if looks_like_command(line) {
            return Some(line.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_block() {
        let raw = "```bash\ntouch demo.txt\n```";
        assert_eq!(extract_command(raw).unwrap(), "touch demo.txt");
    }

    #[test]
    fn json_command_field() {
        let raw = r#"{"command":"touch from.json"}"#;
        assert_eq!(extract_command(raw).unwrap(), "touch from.json");
    }

    #[test]
    fn rejects_prose() {
        let raw = "创建文件有几种方式：";
        assert!(extract_command(raw).is_err());
    }
}
