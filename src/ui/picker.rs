use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use crossterm::{queue, style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickResult {
    Selected(usize),
    Retry,
    Cancel,
}

pub fn pick_command(items: &[String], title: &str) -> Result<PickResult> {
    if items.is_empty() {
        anyhow::bail!("no suggestions to pick");
    }
    if items.len() == 1 {
        return Ok(PickResult::Selected(0));
    }

    if !io::stderr().is_terminal() {
        return pick_fallback(items, title);
    }

    pick_interactive(items, title)
}

fn pick_interactive(items: &[String], title: &str) -> Result<PickResult> {
    let mut selected = 0usize;
    let mut stderr = io::stderr();

    terminal::enable_raw_mode().context("enable raw mode")?;
    execute!(stderr, EnterAlternateScreen, terminal::Clear(ClearType::All), cursor::Hide).ok();

    let result = (|| -> Result<PickResult> {
        render_list(&mut stderr, title, items, selected)?;
        loop {
            if !event::poll(std::time::Duration::from_millis(100))? {
                continue;
            }
            let Event::Key(KeyEvent { code, modifiers, kind, .. }) = event::read()? else {
                continue;
            };
            // Windows 上 Press/Release 各触发一次，只处理 Press 避免重复输入
            if kind != KeyEventKind::Press {
                continue;
            }

            match code {
                KeyCode::Up => {
                    selected = (selected + items.len() - 1) % items.len();
                    rerender(&mut stderr, title, items, selected)?;
                }
                KeyCode::Down => {
                    selected = (selected + 1) % items.len();
                    rerender(&mut stderr, title, items, selected)?;
                }
                KeyCode::Enter => return Ok(PickResult::Selected(selected)),
                KeyCode::Char('r') | KeyCode::Char('R') => return Ok(PickResult::Retry),
                KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(PickResult::Cancel),
                KeyCode::Esc => return Ok(PickResult::Cancel),
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(PickResult::Cancel);
                }
                _ => {}
            }
        }
    })();

    terminal::disable_raw_mode().ok();
    execute!(stderr, cursor::Show, LeaveAlternateScreen).ok();
    queue!(stderr, style::ResetColor)?;
    stderr.flush()?;

    result
}

fn rerender(stderr: &mut io::Stderr, title: &str, items: &[String], selected: usize) -> Result<()> {
    queue!(stderr, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
    render_list(stderr, title, items, selected)
}

fn render_list(stderr: &mut io::Stderr, title: &str, items: &[String], selected: usize) -> Result<()> {
    queue!(stderr, cursor::MoveTo(0, 0))?;
    queue!(
        stderr,
        style::SetAttribute(style::Attribute::Reset),
        style::Print(format!("Suggestions for: {title}\r\n\r\n"))
    )?;
    for (i, item) in items.iter().enumerate() {
        queue!(
            stderr,
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine)
        )?;
        if i == selected {
            queue!(
                stderr,
                style::SetAttribute(style::Attribute::Reverse),
                style::Print(format!("  ▸ {item}  ")),
                style::SetAttribute(style::Attribute::Reset),
                style::Print("\r\n"),
            )?;
        } else {
            queue!(
                stderr,
                style::SetAttribute(style::Attribute::Reset),
                style::Print(format!("    {item}\r\n"))
            )?;
        }
    }
    queue!(
        stderr,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        style::SetAttribute(style::Attribute::Reset),
        style::Print("\r\n↑↓ select  ↵ confirm  r retry  q quit\r\n"),
    )?;
    stderr.flush()?;
    Ok(())
}

fn pick_fallback(items: &[String], title: &str) -> Result<PickResult> {
    let mut stderr = io::stderr();
    eprintln!("Suggestions for: {title}");
    for (i, item) in items.iter().enumerate() {
        eprintln!("  [{}] {}", i + 1, item);
    }
    eprintln!("  [r] retry  [q] quit");
    eprint!("Select 1-{}: ", items.len());
    stderr.flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let line = line.trim();

    if line.eq_ignore_ascii_case("q") {
        return Ok(PickResult::Cancel);
    }
    if line.eq_ignore_ascii_case("r") {
        return Ok(PickResult::Retry);
    }

    let idx: usize = line
        .parse::<usize>()
        .context("enter a number")?
        .saturating_sub(1);
    if idx >= items.len() {
        anyhow::bail!("invalid selection");
    }
    Ok(PickResult::Selected(idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_item_skips_picker() {
        let items = vec!["touch a.txt".to_string()];
        assert_eq!(
            pick_command(&items, "create empty file a.txt").unwrap(),
            PickResult::Selected(0)
        );
    }
}
