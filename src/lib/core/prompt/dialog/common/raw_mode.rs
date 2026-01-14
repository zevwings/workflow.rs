//! 原始模式 Guard，自动恢复终端状态
//!
//! 使用 RAII 模式，当 guard 被 drop 时自动恢复终端状态

use crossterm::terminal;
use std::io;

/// 原始模式 Guard，自动恢复终端状态
///
/// 使用 RAII 模式，当 guard 被 drop 时自动恢复终端状态
pub struct RawModeGuard {
    _private: (),
}

impl RawModeGuard {
    /// 创建新的 RawModeGuard，启用原始模式
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self { _private: () })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        terminal::disable_raw_mode().ok();
    }
}
