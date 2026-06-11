use std::io::{self, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{self, Attribute},
    terminal::{self, ClearType},
};

pub enum EditResult {
    Confirmed(String),
    Cancelled,
}

/// Show `initial` in an editable input line. Returns the (possibly modified) command or Cancelled.
pub fn edit_command(initial: &str, warning: Option<&str>) -> Result<EditResult> {
    let mut buf: Vec<char> = initial.chars().collect();
    let mut cursor_pos = buf.len();

    let mut stderr = io::stderr();
    terminal::enable_raw_mode().context("enable raw mode")?;

    if let Some(warn) = warning {
        queue!(
            stderr,
            style::SetForegroundColor(style::Color::Red),
            style::Print(format!("⚠  {warn}\r\n")),
            style::SetAttribute(Attribute::Reset),
        )?;
    }

    let prompt = "  Edit: ";
    queue!(
        stderr,
        style::SetAttribute(Attribute::Dim),
        style::Print("  ↵ run  Esc cancel\r\n"),
        style::SetAttribute(Attribute::Reset),
        style::Print(prompt),
        style::SetAttribute(Attribute::Bold),
        style::Print(initial),
        style::SetAttribute(Attribute::Reset),
    )?;
    stderr.flush()?;

    let result = (|| -> Result<EditResult> {
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
                KeyCode::Enter => {
                    return Ok(EditResult::Confirmed(buf.iter().collect()));
                }
                KeyCode::Esc => {
                    return Ok(EditResult::Cancelled);
                }
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(EditResult::Cancelled);
                }
                KeyCode::Backspace => {
                    if cursor_pos > 0 {
                        buf.remove(cursor_pos - 1);
                        cursor_pos -= 1;
                    }
                }
                KeyCode::Delete => {
                    if cursor_pos < buf.len() {
                        buf.remove(cursor_pos);
                    }
                }
                KeyCode::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if cursor_pos < buf.len() {
                        cursor_pos += 1;
                    }
                }
                KeyCode::Home => {
                    cursor_pos = 0;
                }
                KeyCode::End => {
                    cursor_pos = buf.len();
                }
                KeyCode::Char(c) => {
                    buf.insert(cursor_pos, c);
                    cursor_pos += 1;
                }
                _ => continue,
            }

            let line: String = buf.iter().collect();
            let col = (prompt.len() + cursor_pos) as u16;
            queue!(
                stderr,
                cursor::MoveToColumn(0),
                terminal::Clear(ClearType::CurrentLine),
                style::Print(prompt),
                style::SetAttribute(Attribute::Bold),
                style::Print(&line),
                style::SetAttribute(Attribute::Reset),
                cursor::MoveToColumn(col),
            )?;
            stderr.flush()?;
        }
    })();

    terminal::disable_raw_mode().ok();
    execute!(stderr, style::Print("\r\n")).ok();

    result
}
