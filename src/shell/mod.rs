use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

const PATH_MARKER: &str = "# ask (Rust CLI) — ensure cargo bin on PATH";

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

/// Remove legacy Oh My Zsh wrapper / ai-cmd plugin snippets from a shell rc file.
pub fn purge_legacy_integration(path: &PathBuf) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let mut content = fs::read_to_string(path)?;
    let original = content.clone();

    while let (Some(start), Some(end)) = (
        content.find("### ask-cmd shell integration"),
        content.find("### end ask-cmd shell integration"),
    ) {
        let remove_end = end + "### end ask-cmd shell integration".len();
        content.replace_range(start..remove_end, "");
    }

    while let (Some(start), Some(end)) = (
        content.find("### ai-cmd"),
        content.find("### end ai-cmd"),
    ) {
        let remove_end = end + "### end ai-cmd".len();
        content.replace_range(start..remove_end, "");
    }

    // Old zsh plugin: ask() { ai-cmd ... } or ask() wrapping ask-cmd path
    if content.contains("ai-cmd.plugin.zsh") || content.contains("noglob ai-cmd") {
        content = content
            .lines()
            .filter(|line| !line.contains("ai-cmd") && !line.contains("ask-cmd.plugin"))
            .collect::<Vec<_>>()
            .join("\n");
    }

    if content != original {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        fs::write(path, content)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn install_path(shell: ShellKind) -> Result<PathBuf> {
    let path = rc_path(shell)?;
    purge_legacy_integration(&path)?;
    ensure_cargo_bin_in_path(&path)?;
    Ok(path)
}

fn rc_path(shell: ShellKind) -> Result<PathBuf> {
    match shell {
        ShellKind::Bash => home_join(".bashrc"),
        ShellKind::Zsh => home_join(".zshrc"),
        ShellKind::Fish => {
            Ok(dirs::config_dir()
                .context("config dir")?
                .join("fish")
                .join("conf.d")
                .join("99-ask-path.fish"))
        }
        ShellKind::PowerShell => powershell_profile(),
    }
}

fn home_join(name: &str) -> Result<PathBuf> {
    Ok(dirs::home_dir().context("home dir")?.join(name))
}

#[cfg(not(windows))]
fn powershell_profile() -> Result<PathBuf> {
    bail!("PowerShell install is only supported on Windows")
}

#[cfg(windows)]
fn powershell_profile() -> Result<PathBuf> {
    home_join("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
}

fn cargo_bin_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".cargo").join("bin"))
        .context("home dir")
}

fn ensure_cargo_bin_in_path(path: &PathBuf) -> Result<()> {
    let cargo_bin = cargo_bin_dir()?;
    let line = format!(r#"export PATH="{}:$PATH" {PATH_MARKER}"#, cargo_bin.display());

    if path.exists() {
        let existing = fs::read_to_string(path)?;
        if existing.contains(PATH_MARKER) || existing.contains(".cargo/bin") {
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
    content.push_str(&line);
    content.push('\n');
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn purge_legacy_ask_wrapper() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            "plugins=(git ai-cmd)\n### ask-cmd shell integration\nask() {{ true; }}\n### end ask-cmd shell integration\n"
        )
        .unwrap();
        let path = tmp.path().to_path_buf();
        assert!(purge_legacy_integration(&path).unwrap());
        let text = fs::read_to_string(path).unwrap();
        assert!(!text.contains("ask-cmd shell integration"));
        assert!(!text.contains("ask()"));
    }
}
