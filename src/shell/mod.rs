use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

const MARKER_START: &str = "### ask-cmd shell integration";
const MARKER_END: &str = "### end ask-cmd shell integration";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl ShellKind {
    pub fn detect() -> Result<Self> {
        if let Ok(shell) = std::env::var("SHELL") {
            let shell = shell.to_lowercase();
            if shell.contains("fish") {
                return Ok(Self::Fish);
            }
            if shell.contains("zsh") {
                return Ok(Self::Zsh);
            }
            if shell.contains("bash") {
                return Ok(Self::Bash);
            }
        }
        if cfg!(target_os = "windows") {
            return Ok(Self::PowerShell);
        }
        Ok(Self::Bash)
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Self::detect(),
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "powershell" | "pwsh" => Ok(Self::PowerShell),
            _ => bail!("unknown shell '{s}' (auto|bash|zsh|fish|powershell)"),
        }
    }
}

pub fn install(shell: ShellKind, bin: &Path) -> Result<PathBuf> {
    let snippet = build_snippet(bin);
    let target = rc_path(shell)?;
    append_snippet(&target, &snippet)?;
    Ok(target)
}

fn rc_path(shell: ShellKind) -> Result<PathBuf> {
    match shell {
        ShellKind::Bash => home_join(".bashrc"),
        ShellKind::Zsh => home_join(".zshrc"),
        ShellKind::Fish => {
            let p = dirs::config_dir()
                .context("config dir")?
                .join("fish")
                .join("conf.d")
                .join("ask-cmd.fish");
            Ok(p)
        }
        ShellKind::PowerShell => powershell_profile(),
    }
}

fn home_join(name: &str) -> Result<PathBuf> {
    Ok(dirs::home_dir().context("home dir")?.join(name))
}

#[cfg(not(windows))]
fn powershell_profile() -> Result<PathBuf> {
    bail!("PowerShell profile install is only supported on Windows from this command; append integrations/ask.ps1 manually")
}

#[cfg(windows)]
fn powershell_profile() -> Result<PathBuf> {
    home_join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
}

fn build_snippet(bin: &Path) -> String {
    let bin_str = bin.display().to_string();
    format!(
        r#"{MARKER_START}
ask() {{
    "{bin_str}" "$@"
}}
{MARKER_END}
"#
    )
}

pub fn build_fish_snippet(bin: &Path) -> String {
    let bin_str = bin.display().to_string();
    format!(
        r#"# {MARKER_START}
function ask
    {bin_str} $argv
end
# {MARKER_END}
"#
    )
}

pub fn build_powershell_snippet(bin: &Path) -> String {
    let bin_str = bin.display().to_string();
    format!(
        r#"# {MARKER_START}
function ask {{
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)
    & "{bin_str}" @Args
}}
# {MARKER_END}
"#
    )
}

pub fn install_fish(bin: &Path) -> Result<PathBuf> {
    let target = rc_path(ShellKind::Fish)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let snippet = build_fish_snippet(bin);
    append_snippet(&target, &snippet)?;
    Ok(target)
}

pub fn install_powershell(bin: &Path) -> Result<PathBuf> {
    let target = rc_path(ShellKind::PowerShell)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let snippet = build_powershell_snippet(bin);
    append_snippet(&target, &snippet)?;
    Ok(target)
}

fn append_snippet(path: &Path, snippet: &str) -> Result<()> {
    if path.exists() {
        let existing = fs::read_to_string(path)?;
        if existing.contains(MARKER_START) {
            return Ok(());
        }
    } else if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(snippet);
    if !content.ends_with('\n') {
        content.push('\n');
    }
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn append_idempotent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rc");
        let snippet = build_snippet(Path::new("/usr/local/bin/ask-cmd"));
        append_snippet(&path, &snippet).unwrap();
        append_snippet(&path, &snippet).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches(MARKER_START).count(), 1);
    }
}
