//! 终端后端抽象模块
//!
//! 提供 Backend trait 用于抽象终端操作，支持：
//! - `TerminalBackend`: 真实终端后端（生产环境）
//! - `MockBackend`: Mock 后端（测试环境，需要 `testing` feature）

#[cfg(any(test, feature = "testing"))]
mod mock;
mod terminal;

// NOTE: 测试代码需要此导出，#[allow] 防止 clippy --fix 误删
#[allow(unused_imports)]
#[cfg(any(test, feature = "testing"))]
pub use mock::MockBackend;
pub use terminal::TerminalBackend;

use crossterm::event::Event;
use std::io::Result;

/// 终端后端抽象 trait
///
/// 抽象所有终端操作，使 prompt 组件可测试。
///
/// # 实现
///
/// - `TerminalBackend`: 委托给 crossterm，用于生产环境
/// - `MockBackend`: 支持预设事件和输出捕获，用于测试（需要 `testing` feature）
///
/// # 示例
///
/// ```rust,ignore
/// // `backend` 是内部模块（不属于公共 API），这里只演示内部使用方式。
/// use prompt::backend::{Backend, TerminalBackend};
///
/// // 生产环境使用真实终端
/// let mut backend = TerminalBackend::default();
/// ```
///
/// # 测试示例
///
/// ```rust,ignore
/// // 需要启用 testing feature
/// // Cargo.toml: prompt = { version = "...", features = ["testing"] }
/// use prompt::backend::{Backend, MockBackend};
///
/// let events = MockBackend::type_string("hello");
/// let mut mock = MockBackend::with_events(events);
/// ```
pub trait Backend {
    /// 读取下一个终端事件（阻塞）
    fn read_event(&mut self) -> Result<Event>;

    /// 写入字符串到终端
    fn write(&mut self, content: &str) -> Result<()>;

    /// 写入带换行的字符串
    fn writeln(&mut self, content: &str) -> Result<()>;

    /// 刷新输出缓冲区
    fn flush(&mut self) -> Result<()>;

    /// 移动光标到指定列
    fn move_to_column(&mut self, column: u16) -> Result<()>;

    /// 光标上移 n 行
    fn move_up(&mut self, n: u16) -> Result<()>;

    /// 光标下移 n 行
    fn move_down(&mut self, n: u16) -> Result<()>;

    /// 清除当前行（从光标到行尾）
    fn clear_line(&mut self) -> Result<()>;

    /// 显示光标
    fn show_cursor(&mut self) -> Result<()>;

    /// 隐藏光标
    fn hide_cursor(&mut self) -> Result<()>;

    /// 进入原始模式
    fn enable_raw_mode(&mut self) -> Result<()>;

    /// 退出原始模式
    fn disable_raw_mode(&mut self) -> Result<()>;

    /// 启用 bracketed paste 模式
    fn enable_bracketed_paste(&mut self) -> Result<()>;

    /// 禁用 bracketed paste 模式
    fn disable_bracketed_paste(&mut self) -> Result<()>;

    /// 启用增强键盘事件（kitty keyboard protocol）。
    ///
    /// 这能让兼容终端上报更丰富的修饰键信息（例如区分 `Enter` 与 `Shift+Enter`）。
    ///
    /// 默认实现为 no-op，便于 Mock/不支持的后端忽略该能力。
    fn enable_keyboard_enhancement(&mut self) -> Result<()> {
        Ok(())
    }

    /// 禁用增强键盘事件（与 `enable_keyboard_enhancement` 成对）。
    ///
    /// 默认实现为 no-op。
    fn disable_keyboard_enhancement(&mut self) -> Result<()> {
        Ok(())
    }
}
