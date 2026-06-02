use std::io::{self, Write};
use std::process::Command as StdCommand;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;

use ask_cmd::shell::{self, ShellKind};
use ask_cmd::ui::picker::PickResult;
use ask_cmd::validate;
use ask_cmd::{pick_command, resolve_command, resolve_suggestions};

#[derive(Parser, Debug)]
#[command(name = "ask", about = "Natural language → shell command via Claude Code CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Print command(s) only, do not confirm or execute
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Skip picker/confirmation and execute the first suggestion
    #[arg(short = 'y', long = "yes")]
    yes: bool,

    /// Natural language request (when not using subcommands)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    query: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add ~/.cargo/bin to PATH in your shell rc (no zsh plugin / wrapper)
    Install {
        #[arg(long, default_value = "auto")]
        shell: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { shell }) => {
            let kind = ShellKind::parse(&shell)?;
            let zshrc = dirs::home_dir().map(|h| h.join(".zshrc"));
            if let Some(ref path) = zshrc {
                if shell::purge_legacy_integration(path)? {
                    eprintln!("{} Removed legacy ai-cmd / ask-cmd shell wrapper", style("✓").green());
                }
            }
            let target = shell::install_path(kind)?;
            eprintln!(
                "{} Rust `ask` CLI ready — updated {}",
                style("✓").green(),
                target.display()
            );
            eprintln!("Restart terminal or: source {}", target.display());
            eprintln!("Then run: ask -n 创建空文件 test.txt");
            return Ok(());
        }
        None => {}
    }

    if cli.query.is_empty() {
        eprintln!("Usage: ask [-n] <what you want to do>");
        eprintln!("       ask install [--shell auto|bash|zsh|fish|powershell]");
        std::process::exit(1);
    }

    let query = cli.query.join(" ");

    if cli.dry_run {
        return run_dry_run(&query);
    }

    if cli.yes {
        return run_first_suggestion(&query);
    }

    run_interactive(&query)
}

fn run_dry_run(query: &str) -> Result<()> {
    eprintln!("{} Asking Claude for suggestions...", style("🤖").yellow());
    match resolve_suggestions(query) {
        Ok(list) => {
            for cmd in &list {
                println!("{cmd}");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
    }
}

fn run_first_suggestion(query: &str) -> Result<()> {
    eprintln!("{} Asking Claude...", style("🤖").yellow());
    let cmd = resolve_suggestions(query)
        .map(|v| v.into_iter().next().expect("non-empty"))
        .or_else(|_| resolve_command(query))?;
    run_command(&cmd)
}

fn run_interactive(query: &str) -> Result<()> {
    loop {
        eprintln!("{} Asking Claude for suggestions...", style("🤖").yellow());
        let suggestions = match resolve_suggestions(query) {
            Ok(list) => list,
            Err(e) => {
                eprintln!("{e:#}");
                std::process::exit(1);
            }
        };

        match pick_command(&suggestions, query)? {
            PickResult::Selected(idx) => {
                let cmd = &suggestions[idx];
                confirm_and_run(cmd)?;
                return Ok(());
            }
            PickResult::Retry => continue,
            PickResult::Cancel => {
                eprintln!("Cancelled.");
                return Ok(());
            }
        }
    }
}

fn confirm_and_run(cmd: &str) -> Result<()> {
    println!();
    println!("{}  {}", style("Selected:").cyan().bold(), style(cmd).bold());

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
    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let flag = if cfg!(target_os = "windows") { "/C" } else { "-c" };
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
            .arg(format!(
                "printf {} | xclip -selection clipboard 2>/dev/null || printf {} | wl-copy 2>/dev/null",
                shell_escape(text),
                shell_escape(text)
            ))
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
