//! Mock 后端实现
//!
//! 用于测试的 Mock 后端，支持预设事件和输出捕获。
//!
//! 此模块仅在 `testing` feature 启用时编译，供外部 crate 测试使用。

// NOTE: 这是测试工具 API，供外部 crate 使用，在本 crate 内不直接调用
#![allow(dead_code)]

use super::Backend;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Result};

/// Mock 后端
///
/// 用于测试的 Mock 后端，支持：
/// - 预设事件队列（模拟用户输入）
/// - 输出捕获（验证输出内容）
///
/// # 示例
///
/// ```rust
/// use prompt::backend::MockBackend;
///
/// // 模拟用户输入 "hello" 然后按 Enter
/// let events = [
///     MockBackend::type_string("hello"),
///     vec![MockBackend::press_enter()],
/// ].concat();
///
/// let mut backend = MockBackend::with_events(events);
/// ```
pub struct MockBackend {
    /// 预设的事件队列
    events: VecDeque<Event>,
    /// 捕获的输出
    output: Vec<String>,
    /// 当前光标位置 (column, row)
    cursor_position: (u16, u16),
    /// 是否处于原始模式
    raw_mode: bool,
    /// 是否启用了 bracketed paste
    bracketed_paste: bool,
    /// 光标是否可见
    cursor_visible: bool,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackend {
    /// 创建空的 Mock 后端
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            output: Vec::new(),
            cursor_position: (0, 0),
            raw_mode: false,
            bracketed_paste: false,
            cursor_visible: true,
        }
    }

    /// 创建带预设事件的 Mock 后端
    pub fn with_events(events: impl IntoIterator<Item = Event>) -> Self {
        Self {
            events: events.into_iter().collect(),
            output: Vec::new(),
            cursor_position: (0, 0),
            raw_mode: false,
            bracketed_paste: false,
            cursor_visible: true,
        }
    }

    /// 获取捕获的输出
    pub fn output(&self) -> &[String] {
        &self.output
    }

    /// 获取合并后的输出字符串
    pub fn output_string(&self) -> String {
        self.output.join("")
    }

    /// 获取当前光标位置
    pub fn cursor_position(&self) -> (u16, u16) {
        self.cursor_position
    }

    /// 检查是否处于原始模式
    pub fn is_raw_mode(&self) -> bool {
        self.raw_mode
    }

    /// 检查光标是否可见
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    // ========================================================================
    // 辅助方法：生成常用事件
    // ========================================================================

    /// 模拟用户输入字符串（转换为按键事件）
    pub fn type_string(input: &str) -> Vec<Event> {
        input
            .chars()
            .map(|c| Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .collect()
    }

    /// 模拟按下 Enter 键
    pub fn press_enter() -> Event {
        Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
    }

    /// 模拟按下 Shift+Enter 键（用于多行输入换行）
    pub fn press_shift_enter() -> Event {
        Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT))
    }

    /// 模拟按下 Escape 键
    pub fn press_escape() -> Event {
        Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
    }
}

impl Backend for MockBackend {
    fn read_event(&mut self) -> Result<Event> {
        self.events.pop_front().ok_or_else(|| {
            Error::new(
                ErrorKind::UnexpectedEof,
                "No more events in mock backend queue",
            )
        })
    }

    fn write(&mut self, content: &str) -> Result<()> {
        self.output.push(content.to_string());
        // 更新光标列位置（简化处理，不考虑换行）
        self.cursor_position.0 += content.len() as u16;
        Ok(())
    }

    fn writeln(&mut self, content: &str) -> Result<()> {
        self.output.push(format!("{}\n", content));
        // 换行后光标移到下一行开头
        self.cursor_position.0 = 0;
        self.cursor_position.1 += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        // Mock 不需要实际刷新
        Ok(())
    }

    fn move_to_column(&mut self, column: u16) -> Result<()> {
        self.cursor_position.0 = column;
        Ok(())
    }

    fn move_up(&mut self, n: u16) -> Result<()> {
        self.cursor_position.1 = self.cursor_position.1.saturating_sub(n);
        Ok(())
    }

    fn move_down(&mut self, n: u16) -> Result<()> {
        self.cursor_position.1 += n;
        Ok(())
    }

    fn clear_line(&mut self) -> Result<()> {
        // 记录清除操作（可选）
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<()> {
        self.cursor_visible = true;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<()> {
        self.cursor_visible = false;
        Ok(())
    }

    fn enable_raw_mode(&mut self) -> Result<()> {
        self.raw_mode = true;
        Ok(())
    }

    fn disable_raw_mode(&mut self) -> Result<()> {
        self.raw_mode = false;
        Ok(())
    }

    fn enable_bracketed_paste(&mut self) -> Result<()> {
        self.bracketed_paste = true;
        Ok(())
    }

    fn disable_bracketed_paste(&mut self) -> Result<()> {
        self.bracketed_paste = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_backend_with_events() {
        let events = vec![MockBackend::press_enter(), MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        assert!(matches!(
            backend.read_event().unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })
        ));
        assert!(matches!(
            backend.read_event().unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                ..
            })
        ));
        assert!(backend.read_event().is_err());
    }

    #[test]
    fn test_mock_backend_type_string() {
        let events = MockBackend::type_string("abc");
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_mock_backend_output() {
        let mut backend = MockBackend::new();
        backend.write("hello").unwrap();
        backend.writeln(" world").unwrap();

        assert_eq!(backend.output(), &["hello", " world\n"]);
        assert_eq!(backend.output_string(), "hello world\n");
    }

    #[test]
    fn test_mock_backend_cursor() {
        let mut backend = MockBackend::new();
        assert_eq!(backend.cursor_position(), (0, 0));

        backend.move_to_column(10).unwrap();
        assert_eq!(backend.cursor_position(), (10, 0));

        backend.move_down(2).unwrap();
        assert_eq!(backend.cursor_position(), (10, 2));

        backend.move_up(1).unwrap();
        assert_eq!(backend.cursor_position(), (10, 1));
    }

    #[test]
    fn test_mock_backend_modes() {
        let mut backend = MockBackend::new();

        assert!(!backend.is_raw_mode());
        backend.enable_raw_mode().unwrap();
        assert!(backend.is_raw_mode());
        backend.disable_raw_mode().unwrap();
        assert!(!backend.is_raw_mode());

        assert!(backend.is_cursor_visible());
        backend.hide_cursor().unwrap();
        assert!(!backend.is_cursor_visible());
        backend.show_cursor().unwrap();
        assert!(backend.is_cursor_visible());
    }
}
