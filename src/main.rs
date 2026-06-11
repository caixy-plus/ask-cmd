use std::process::Command as StdCommand;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;

use ask_cmd::shell::{self, ShellKind};
use ask_cmd::ui::editor::{edit_command, EditResult};
use ask_cmd::ui::picker::PickResult;
use ask_cmd::validate;
use ask_cmd::{pick_command, resolve_command, resolve_suggestions};

#[derive(Parser, Debug)]
#[command(name = "ask", about = "Natural language → shell command via AI CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// AI provider to use (claude|opencode|codex|cursor)
    #[arg(short = 'p', long = "provider")]
    provider: Option<String>,

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
    /// Switch default AI provider interactively
    Switch,
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
        Some(Commands::Switch) => {
            return run_switch();
        }
        None => {}
    }

    if cli.query.is_empty() {
        eprintln!("Usage: ask [-n] [-p <provider>] <what you want to do>");
        eprintln!("       ask install [--shell auto|bash|zsh|fish|powershell]");
        eprintln!("       ask switch");
        std::process::exit(1);
    }

    let query = cli.query.join(" ");
    let preferred = cli.provider.as_deref();

    if cli.dry_run {
        return run_dry_run(&query, preferred);
    }

    if cli.yes {
        return run_first_suggestion(&query, preferred);
    }

    run_interactive(&query, preferred)
}

fn run_dry_run(query: &str, preferred: Option<&str>) -> Result<()> {
    let provider = ask_cmd::providers::registry::resolve_provider(preferred)?;
    eprintln!(
        "{} Asking {} for suggestions...",
        style("🤖").yellow(),
        provider.display_name()
    );
    match resolve_suggestions(query, preferred) {
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

fn run_first_suggestion(query: &str, preferred: Option<&str>) -> Result<()> {
    let provider = ask_cmd::providers::registry::resolve_provider(preferred)?;
    eprintln!("{} Asking {}...", style("🤖").yellow(), provider.display_name());
    let cmd = resolve_suggestions(query, preferred)
        .map(|v| v.into_iter().next().expect("non-empty"))
        .or_else(|_| resolve_command(query, preferred))?;
    run_command(&cmd)
}

fn run_interactive(query: &str, preferred: Option<&str>) -> Result<()> {
    let provider = ask_cmd::providers::registry::resolve_provider(preferred)?;
    loop {
        eprintln!(
            "{} Asking {} for suggestions...",
            style("🤖").yellow(),
            provider.display_name()
        );
        let suggestions = match resolve_suggestions(query, preferred) {
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

    let warning = if validate::is_dangerous(cmd) {
        Some("Potentially destructive — review carefully")
    } else {
        None
    };

    match edit_command(cmd, warning)? {
        EditResult::Confirmed(final_cmd) => {
            run_command(&final_cmd)?;
        }
        EditResult::Cancelled => {
            eprintln!("Cancelled.");
        }
    }
    Ok(())
}

fn run_command(cmd: &str) -> Result<()> {
    println!("{cmd}");
    // Windows 用 PowerShell 执行，与提示词生成的命令风格保持一致（cmd.exe 无法运行 Get-ChildItem 等 cmdlet）
    let (shell, flags): (&str, &[&str]) = if cfg!(target_os = "windows") {
        ("powershell", &["-NoProfile", "-Command"])
    } else {
        ("sh", &["-c"])
    };
    let status = StdCommand::new(shell)
        .args(flags)
        .arg(cmd)
        .status()
        .with_context(|| format!("execute: {cmd}"))?;
    if !status.success() {
        anyhow::bail!("command exited with {status}");
    }
    Ok(())
}


fn run_switch() -> Result<()> {
    use ask_cmd::config::Config;
    use ask_cmd::providers::registry::available_providers;

    let providers = available_providers();
    if providers.is_empty() {
        anyhow::bail!("未检测到任何可用的 AI CLI 智能体。");
    }

    let names: Vec<String> = providers
        .iter()
        .map(|p| format!("{} - {}", p.display_name(), p.name()))
        .collect();

    match pick_command(&names, "选择默认智能体")? {
        PickResult::Selected(idx) => {
            let selected = &providers[idx];
            let mut config = Config::load()?;
            config.set_default_provider(selected.name().to_string());
            config.save()?;
            eprintln!(
                "{} 默认智能体已切换为: {} ({})",
                style("✓").green(),
                selected.display_name(),
                selected.name()
            );
            Ok(())
        }
        PickResult::Retry => run_switch(),
        PickResult::Cancel => {
            eprintln!("已取消。");
            Ok(())
        }
    }
}
