use anyhow::{bail, Context, Result};
use regex::Regex;
use serde_json::Value;

use crate::validate;

const MAX_SUGGESTIONS: usize = 5;

pub fn parse_suggestions(raw: &str) -> Result<Vec<String>> {
    let trimmed = strip_fences(raw.trim());
    let items = parse_json_array(&trimmed).or_else(|_| parse_embedded_array(&trimmed))?;

    let mut out = Vec::new();
    for item in items {
        let cmd = item.trim();
        if cmd.is_empty() {
            continue;
        }
        if validate::validate_command(cmd).is_err() {
            continue;
        }
        if out.iter().any(|existing| existing == cmd) {
            continue;
        }
        out.push(cmd.to_string());
        if out.len() >= MAX_SUGGESTIONS {
            break;
        }
    }

    if out.is_empty() {
        bail!("Claude 未返回可用的 shell 命令，请说得更具体，例如：创建空文件 test.txt");
    }
    Ok(out)
}

fn strip_fences(raw: &str) -> String {
    let mut s = raw.to_string();
    if s.starts_with("```") {
        if let Some(rest) = s.strip_prefix("```") {
            s = rest
                .trim_start_matches("json")
                .trim_start_matches("bash")
                .trim_start_matches("sh")
                .trim_start_matches('\n')
                .to_string();
        }
        if let Some(body) = s.strip_suffix("```") {
            s = body.trim().to_string();
        }
    }
    s
}

fn parse_json_array(raw: &str) -> Result<Vec<String>> {
    let value: Value = serde_json::from_str(raw).context("parse JSON array")?;
    json_value_to_strings(value)
}

fn parse_embedded_array(raw: &str) -> Result<Vec<String>> {
    let re = Regex::new(r"\[[\s\S]*?\]").context("regex")?;
    let Some(found) = re.find(raw) else {
        bail!("no JSON array in response");
    };
    parse_json_array(found.as_str())
}

fn json_value_to_strings(value: Value) -> Result<Vec<String>> {
    match value {
        Value::Array(items) => Ok(items
            .into_iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s),
                other => Some(other.to_string()),
            })
            .collect()),
        Value::String(s) => Ok(vec![s]),
        _ => bail!("expected JSON array of command strings"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_array() {
        let raw = r#"["touch a.txt", "echo '' > a.txt", "printf '' > a.txt"]"#;
        let cmds = parse_suggestions(raw).unwrap();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], "touch a.txt");
    }

    #[test]
    fn parses_fenced_json() {
        let raw = "```json\n[\"ls -la\", \"ls -l\"]\n```";
        let cmds = parse_suggestions(raw).unwrap();
        assert_eq!(cmds.len(), 2);
    }

    #[test]
    fn filters_invalid_commands() {
        let raw = r#"["touch ok.txt", "创建文件有几种方式", "ls"]"#;
        let cmds = parse_suggestions(raw).unwrap();
        assert_eq!(cmds, vec!["touch ok.txt", "ls"]);
    }

    #[test]
    fn caps_at_five() {
        let raw = r#"["touch 1", "touch 2", "touch 3", "touch 4", "touch 5", "touch 6"]"#;
        assert_eq!(parse_suggestions(raw).unwrap().len(), 5);
    }
}
