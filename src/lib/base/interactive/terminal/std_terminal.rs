//! 标准终端实现

use crate::base::interactive::terminal::Terminal;
use crossterm::terminal;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};

/// 原始模式 Guard，自动恢复终端状态
pub struct RawModeGuard {
    _private: (),
}

impl RawModeGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self { _private: () })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        terminal::disable_raw_mode().ok();
    }
}

/// 标准终端实现
pub struct StdTerminal {
    stdout: Arc<Mutex<io::Stdout>>,
    stdin: BufReader<io::Stdin>,
}

impl StdTerminal {
    /// 创建新的标准终端实例
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            stdout: Arc::new(Mutex::new(io::stdout())),
            stdin: BufReader::new(io::stdin()),
        })
    }
}

impl Terminal for StdTerminal {
    fn read_byte(&mut self) -> io::Result<u8> {
        use crossterm::event::{self, Event, KeyCode};
        loop {
            match event::read()? {
                Event::Key(key_event) => {
                    if let KeyCode::Char(c) = key_event.code {
                        return Ok(c as u8);
                    }
                    // 处理其他键码...
                }
                _ => continue,
            }
        }
    }

    fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        self.stdin.read_line(&mut line)?;
        Ok(line.trim_end().to_string())
    }

    fn write(&mut self, s: &str) -> io::Result<()> {
        let mut stdout = self.stdout.lock().unwrap();
        write!(stdout, "{}", s)?;
        Ok(())
    }

    fn write_flush(&mut self, s: &str) -> io::Result<()> {
        let mut stdout = self.stdout.lock().unwrap();
        write!(stdout, "{}", s)?;
        stdout.flush()?;
        Ok(())
    }

    fn enable_raw_mode(&mut self) -> io::Result<RawModeGuard> {
        RawModeGuard::new()
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    fn supports_color(&self) -> bool {
        // 简化实现：检查是否在 TTY 中
        self.is_tty()
    }

    fn is_tty(&self) -> bool {
        crossterm::tty::IsTty::is_tty(&io::stdout())
    }
}
