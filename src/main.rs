use std::io::{self, Write};
use std::process::Command as StdCommand;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;

use ask_cmd::config::Config;
use ask_cmd::resolve_command;
use ask_cmd::shell::{self, ShellKind};
use ask_cmd::validate;

#[derive(Parser, Debug)]
#[command(name = "ask-cmd", about = "Natural language → shell command (cross-platform)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Print command only, do not confirm or execute
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Skip confirmation and execute immediately (use with care)
    #[arg(short = 'y', long = "yes")]
    yes: bool,

    /// Natural language request (when not using subcommands)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    query: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install shell wrapper `ask` into your rc file
    Install {
        #[arg(long, default_value = "auto")]
        shell: String,
    },
    /// Write default config to ~/.config/ask-cmd/config.toml
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { shell }) => {
            let kind = ShellKind::parse(&shell)?;
            let bin = std::env::current_exe().context("current exe")?;
            let target = match kind {
                ShellKind::Fish => shell::install_fish(&bin)?,
                ShellKind::PowerShell => shell::install_powershell(&bin)?,
                _ => shell::install(kind, &bin)?,
            };
            eprintln!(
                "{} Installed `ask` wrapper → {}",
                style("✓").green(),
                target.display()
            );
            eprintln!("Restart your terminal or run: source {}", target.display());
            return Ok(());
        }
        Some(Commands::Init) => {
            let path = Config::save_default()?;
            eprintln!("{} Wrote {}", style("✓").green(), path.display());
            return Ok(());
        }
        None => {}
    }

    if cli.query.is_empty() {
        eprintln!("Usage: ask-cmd [-n] <what you want to do>");
        eprintln!("       ask-cmd install [--shell auto|bash|zsh|fish|powershell]");
        std::process::exit(1);
    }

    let query = cli.query.join(" ");
    let config = Config::load()?;

    eprintln!(
        "{} Asking {}...",
        style("🤖").yellow(),
        config.provider
    );

    let cmd = resolve_command(&query, &config)?;

    if cli.dry_run {
        println!("{cmd}");
        return Ok(());
    }

    if cli.yes {
        run_command(&cmd)?;
        return Ok(());
    }

    confirm_and_run(&cmd)
}

fn confirm_and_run(cmd: &str) -> Result<()> {
    println!();
    println!("{}  {}", style("Suggested:").cyan().bold(), style(cmd).bold());

    if validate::is_dangerous(cmd) {
        println!("{}", style("⚠  Potentially destructive — review carefully").red());
    }

    loop {
        print!("Run? [Y/n/c(opy)/q]: ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let choice = line.trim().to_lowercase();

        match choice.as_str() {
            "" | "y" | "yes" => {
                run_command(cmd)?;
                return Ok(());
            }
            "n" | "no" => return Ok(()),
            "c" | "copy" => {
                copy_to_clipboard(cmd);
                println!("{}", style("Copied").green());
                return Ok(());
            }
            "q" | "quit" => return Ok(()),
            _ => eprintln!("Choose Y, n, c, or q."),
        }
    }
}

fn run_command(cmd: &str) -> Result<()> {
    println!("{cmd}");
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    let flag = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };
    let status = StdCommand::new(shell)
        .arg(flag)
        .arg(cmd)
        .status()
        .with_context(|| format!("execute: {cmd}"))?;
    if !status.success() {
        anyhow::bail!("command exited with {status}");
    }
    Ok(())
}

fn copy_to_clipboard(text: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = StdCommand::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()
            });
    }
    #[cfg(target_os = "linux")]
    {
        let _ = StdCommand::new("sh")
            .arg("-c")
            .arg(format!("printf %s | xclip -selection clipboard 2>/dev/null || printf %s | wl-copy 2>/dev/null", 
                shell_escape(text), shell_escape(text)))
            .status();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = StdCommand::new("powershell")
            .args(["-NoProfile", "-Command", &format!("Set-Clipboard -Value '{text}'")])
            .status();
    }
}

#[cfg(target_os = "linux")]
fn shell_escape(s: &str) -> String {
    format!("{s:?}")
}
