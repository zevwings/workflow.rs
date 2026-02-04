//! 真实终端后端实现
//!
//! 委托给 crossterm 进行实际的终端操作。

use super::Backend;
use crossterm::cursor;
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{self, ClearType};
use std::io::{Result, Stdout, Write};

/// 真实终端后端
///
/// 委托给 crossterm 进行终端操作，用于生产环境。
pub struct TerminalBackend {
    stdout: Stdout,
    /// 是否处于原始模式
    raw_mode: bool,
    /// 是否启用了 bracketed paste
    bracketed_paste: bool,
    /// 是否启用了增强键盘事件（kitty keyboard protocol）
    keyboard_enhancement: bool,
}

impl Default for TerminalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalBackend {
    /// 创建新的终端后端
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout(),
            raw_mode: false,
            bracketed_paste: false,
            keyboard_enhancement: false,
        }
    }
}

impl Drop for TerminalBackend {
    fn drop(&mut self) {
        // 确保退出时恢复终端状态
        if self.bracketed_paste {
            let _ = execute!(self.stdout, DisableBracketedPaste);
        }
        if self.keyboard_enhancement {
            let _ = execute!(self.stdout, PopKeyboardEnhancementFlags);
        }
        if self.raw_mode {
            let _ = terminal::disable_raw_mode();
        }
    }
}

impl Backend for TerminalBackend {
    fn read_event(&mut self) -> Result<Event> {
        crossterm::event::read()
    }

    fn write(&mut self, content: &str) -> Result<()> {
        write!(self.stdout, "{}", content)
    }

    fn writeln(&mut self, content: &str) -> Result<()> {
        writeln!(self.stdout, "{}", content)
    }

    fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }

    fn move_to_column(&mut self, column: u16) -> Result<()> {
        execute!(self.stdout, cursor::MoveToColumn(column))
    }

    fn move_up(&mut self, n: u16) -> Result<()> {
        if n > 0 {
            execute!(self.stdout, cursor::MoveUp(n))
        } else {
            Ok(())
        }
    }

    fn move_down(&mut self, n: u16) -> Result<()> {
        if n > 0 {
            execute!(self.stdout, cursor::MoveDown(n))
        } else {
            Ok(())
        }
    }

    fn clear_line(&mut self) -> Result<()> {
        execute!(self.stdout, terminal::Clear(ClearType::UntilNewLine))
    }

    fn show_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::Show)
    }

    fn hide_cursor(&mut self) -> Result<()> {
        execute!(self.stdout, cursor::Hide)
    }

    fn enable_raw_mode(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        self.raw_mode = true;
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        self.raw_mode = false;
        Ok(())
    }

    fn enable_bracketed_paste(&mut self) -> Result<()> {
        execute!(self.stdout, EnableBracketedPaste)?;
        self.bracketed_paste = true;
        Ok(())
    }

    fn disable_bracketed_paste(&mut self) -> Result<()> {
        execute!(self.stdout, DisableBracketedPaste)?;
        self.bracketed_paste = false;
        Ok(())
    }

    fn enable_keyboard_enhancement(&mut self) -> Result<()> {
        // 仅在兼容终端上有效；不兼容终端通常会忽略该控制序列。
        execute!(
            self.stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
        self.keyboard_enhancement = true;
        Ok(())
    }

    fn disable_keyboard_enhancement(&mut self) -> Result<()> {
        execute!(self.stdout, PopKeyboardEnhancementFlags)?;
        self.keyboard_enhancement = false;
        Ok(())
    }
}
