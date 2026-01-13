//! Terminal Trait 定义

use crate::base::interactive::terminal::RawModeGuard;
use std::io;

/// 终端抽象 Trait，支持同步操作
pub trait Terminal: Send + Sync {
    /// 读取单个字节（用于交互式输入）
    fn read_byte(&mut self) -> io::Result<u8>;

    /// 读取一行（用于 fallback 模式）
    fn read_line(&mut self) -> io::Result<String>;

    /// 写入字符串
    fn write(&mut self, s: &str) -> io::Result<()>;

    /// 写入并刷新
    fn write_flush(&mut self, s: &str) -> io::Result<()>;

    /// 进入原始模式，返回 Guard
    fn enable_raw_mode(&mut self) -> io::Result<RawModeGuard>;

    /// 获取终端大小
    fn size(&self) -> io::Result<(u16, u16)>;

    /// 是否支持颜色
    fn supports_color(&self) -> bool;

    /// 是否在 TTY 中
    fn is_tty(&self) -> bool;
}
