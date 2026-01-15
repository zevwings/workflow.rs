//! 原始模式 Guard，自动恢复终端状态
//!
//! 使用 RAII 模式，当 guard 被 drop 时自动恢复终端状态

use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::execute;
use crossterm::terminal;
use std::io;

/// 原始模式 Guard，自动恢复终端状态
///
/// 使用 RAII 模式，当 guard 被 drop 时自动恢复终端状态
/// 同时启用 bracketed paste mode，以便正确处理粘贴事件
pub struct RawModeGuard {
    _private: (),
}

impl RawModeGuard {
    /// 创建新的 RawModeGuard，启用原始模式和 bracketed paste mode
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        // 启用 bracketed paste mode，以便区分用户输入和粘贴内容
        execute!(std::io::stdout(), EnableBracketedPaste)?;
        Ok(Self { _private: () })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // 禁用 bracketed paste mode
        execute!(std::io::stdout(), DisableBracketedPaste).ok();
        // 禁用原始模式
        terminal::disable_raw_mode().ok();
    }
}
