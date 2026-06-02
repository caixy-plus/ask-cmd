use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor, execute,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
};
use crossterm::{queue, style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickResult {
    Selected(usize),
    Retry,
    Cancel,
}

pub fn pick_command(items: &[String]) -> Result<PickResult> {
    if items.is_empty() {
        anyhow::bail!("no suggestions to pick");
    }
    if items.len() == 1 {
        return Ok(PickResult::Selected(0));
    }

    if !io::stderr().is_terminal() {
        return pick_fallback(items);
    }

    pick_interactive(items)
}

fn pick_interactive(items: &[String]) -> Result<PickResult> {
    let mut selected = 0usize;
    let mut stderr = io::stderr();

    terminal::enable_raw_mode().context("enable raw mode")?;
    execute!(stderr, cursor::Hide).ok();

    let result = (|| -> Result<PickResult> {
        render_list(&mut stderr, items, selected)?;
        loop {
            if !event::poll(std::time::Duration::from_millis(100))? {
                continue;
            }
            let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? else {
                continue;
            };

            match code {
                KeyCode::Up => {
                    selected = (selected + items.len() - 1) % items.len();
                    rerender(&mut stderr, items, selected)?;
                }
                KeyCode::Down => {
                    selected = (selected + 1) % items.len();
                    rerender(&mut stderr, items, selected)?;
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
    execute!(stderr, cursor::Show).ok();
    queue!(stderr, style::ResetColor, cursor::MoveToColumn(0))?;
    stderr.flush()?;

    result
}

fn rerender(stderr: &mut io::Stderr, items: &[String], selected: usize) -> Result<()> {
    let lines = items.len() + 2;
    queue!(stderr, cursor::MoveUp(lines as u16))?;
    render_list(stderr, items, selected)
}

fn render_list(stderr: &mut io::Stderr, items: &[String], selected: usize) -> Result<()> {
    queue!(stderr, terminal::Clear(ClearType::CurrentLine))?;
    for (i, item) in items.iter().enumerate() {
        if i == selected {
            queue!(
                stderr,
                style::SetAttribute(style::Attribute::Reverse),
                style::Print(format!("  ▸ {item}  ")),
                style::ResetColor,
                style::Print("\r\n"),
            )?;
        } else {
            queue!(stderr, style::Print(format!("    {item}\r\n")))?;
        }
    }
    queue!(
        stderr,
        style::Print("\r\n  ↑↓ select  ↵ confirm  r retry  q quit\r\n"),
    )?;
    stderr.flush()?;
    Ok(())
}

fn pick_fallback(items: &[String]) -> Result<PickResult> {
    let mut stderr = io::stderr();
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
        assert_eq!(pick_command(&items).unwrap(), PickResult::Selected(0));
    }
}
