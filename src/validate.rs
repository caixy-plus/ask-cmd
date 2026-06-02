use anyhow::{bail, Result};
use regex::Regex;

pub fn looks_like_command(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() {
        return false;
    }
    if line.contains('：') || line.contains('。') || line.contains('！') || line.contains('？') {
        return false;
    }
    if !line
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '~' | '_' | '-'))
    {
        return false;
    }
    if contains_cjk(line) {
        return false;
    }
    true
}

pub fn validate_command(cmd: &str) -> Result<()> {
    if !looks_like_command(cmd) {
        bail!("not a valid shell command");
    }
    Ok(())
}

pub fn is_dangerous(cmd: &str) -> bool {
    static PATTERNS: &[&str] = &[
        r"(?i)(^|\s)sudo\s",
        r"rm\s+-\S*r",
        r"rm\s+-\S*f",
        r"(?i)(^|\s)mkfs(\s|$|\.)",
        r"(?i)(^|\s)dd\s",
        r">\s*/dev/",
        r"(?i)Remove-Item\s+.*-Recurse",
        r"(?i)Format-Volume",
    ];
    PATTERNS
        .iter()
        .any(|p| Regex::new(p).map(|re| re.is_match(cmd)).unwrap_or(false))
}

fn contains_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            '\u{4E00}'..='\u{9FFF}'
                | '\u{3400}'..='\u{4DBF}'
                | '\u{3000}'..='\u{303F}'
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good_commands() {
        assert!(looks_like_command("touch test.txt"));
        assert!(looks_like_command("ls -la"));
    }

    #[test]
    fn bad_commands() {
        assert!(!looks_like_command("touch 文件名"));
        assert!(!looks_like_command("创建文件有几种方式"));
    }

    #[test]
    fn dangerous_detection() {
        assert!(is_dangerous("sudo rm -rf /"));
        assert!(!is_dangerous("touch safe.txt"));
    }
}
