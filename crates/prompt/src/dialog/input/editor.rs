//! 输入编辑器模块
//!
//! 管理输入缓冲区和光标位置，提供字符插入、删除、光标移动等功能。
//! 正确处理 Unicode 字符和显示宽度计算。

use unicode_width::UnicodeWidthStr;

/// 光标所在的行
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorLine {
    /// 提示行（第1行）
    PromptLine,
    /// 输入行（第2行）
    InputLine,
}

/// 验证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValidationStatus {
    /// 初始状态（未输入）
    Initial,
    /// 验证通过
    Valid,
    /// 验证失败
    Invalid,
}

/// 输入编辑器，管理输入缓冲区和光标位置
///
/// 提供文本编辑功能，包括：
/// - 字符插入和删除
/// - 光标移动（左/右）
/// - Unicode 字符和显示宽度的正确处理
///
/// # 实现细节
///
/// - 使用 `String` 存储缓冲区内容
/// - 光标位置使用字节索引（`usize`），不是字符索引
/// - 所有操作都确保光标位置在字符边界上
pub(crate) struct InputEditor {
    /// 输入缓冲区
    buffer: String,
    /// 光标位置（字节索引，不是字符索引）
    cursor: usize,
    /// 占位符文本（可选）
    placeholder: Option<String>,
}

impl InputEditor {
    /// 创建新的输入编辑器
    ///
    /// # 参数
    ///
    /// * `placeholder` - 可选的占位符文本，在输入为空时显示
    ///
    /// # 返回
    ///
    /// 返回一个新的 `InputEditor` 实例，光标位置在 0。
    pub(crate) fn new(placeholder: Option<String>) -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            placeholder,
        }
    }

    /// 在光标位置插入字符
    ///
    /// # 参数
    ///
    /// * `ch` - 要插入的字符
    ///
    /// # 性能
    ///
    /// 时间复杂度为 O(n)，其中 n 是光标位置之后的字符数。
    /// 这是因为需要移动光标后的所有字符。
    pub(crate) fn insert(&mut self, ch: char) {
        let char_len = ch.len_utf8();
        self.buffer.insert(self.cursor, ch);
        self.cursor += char_len;
    }

    /// 在光标位置插入字符串
    ///
    /// 用于批量插入文本（例如粘贴操作），比逐个字符插入更高效。
    ///
    /// # 参数
    ///
    /// * `text` - 要插入的文本
    ///
    /// # 性能
    ///
    /// 时间复杂度为 O(n + m)，其中 n 是光标位置之后的字符数，m 是要插入的文本长度。
    /// 批量插入比逐个字符插入更高效，因为只需要一次内存操作。
    ///
    /// # 注意
    ///
    /// 用于批量插入文本（例如粘贴操作），比逐个字符插入更高效。
    pub(crate) fn insert_str(&mut self, text: &str) {
        // 将文本插入到光标位置
        self.buffer.insert_str(self.cursor, text);
        // 更新光标位置到插入文本的末尾
        self.cursor += text.len();
    }

    /// 将光标移动到上一行（尽量保持列位置）
    pub(crate) fn move_up(&mut self) {
        let (row, col) = self.cursor_row_col_display_width();
        if row == 0 {
            return;
        }

        let safe_cursor = self.safe_cursor();
        let before = &self.buffer[..safe_cursor];

        // 当前行起点（上一行的 '\n' 之后）
        let current_line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);

        // 找到上一行起点与终点
        let prev_line_end = current_line_start.saturating_sub(1); // '\n' 的位置
        let prev_line_start = before[..prev_line_end].rfind('\n').map(|i| i + 1).unwrap_or(0);

        let prev_line = &self.buffer[prev_line_start..prev_line_end];
        let target_in_prev = byte_index_for_display_width(prev_line, col);
        self.cursor = prev_line_start + target_in_prev;
    }

    /// 将光标移动到下一行（尽量保持列位置）
    pub(crate) fn move_down(&mut self) {
        let (row, col) = self.cursor_row_col_display_width();
        let lines = self.buffer.split('\n').count();
        if row + 1 >= lines {
            return;
        }

        let safe_cursor = self.safe_cursor();
        let before = &self.buffer[..safe_cursor];

        // 当前行起点
        let current_line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        // 当前行终点（到下一处 '\n' 或字符串末尾）
        let current_line_end = self.buffer[current_line_start..]
            .find('\n')
            .map(|i| current_line_start + i)
            .unwrap_or(self.buffer.len());

        // 下一行起点与终点
        let next_line_start = (current_line_end + 1).min(self.buffer.len());
        let next_line_end = self.buffer[next_line_start..]
            .find('\n')
            .map(|i| next_line_start + i)
            .unwrap_or(self.buffer.len());

        let next_line = &self.buffer[next_line_start..next_line_end];
        let target_in_next = byte_index_for_display_width(next_line, col);
        self.cursor = next_line_start + target_in_next;
    }

    /// 删除光标前的字符（Backspace）
    ///
    /// # 返回
    ///
    /// 如果成功删除字符返回 `true`，如果光标已在开头返回 `false`。
    ///
    /// # 性能
    ///
    /// 时间复杂度为 O(n)，其中 n 是光标位置之后的字符数。
    pub(crate) fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            // 找到前一个字符的起始位置
            let prev_char_start = self.buffer[..self.cursor]
                .char_indices()
                .last()
                .map(|(pos, _)| pos)
                .unwrap_or(0);

            // 移除字符
            self.buffer.remove(prev_char_start);
            self.cursor = prev_char_start;
            true
        } else {
            false
        }
    }

    /// 删除光标位置的字符（Delete）
    ///
    /// # 返回
    ///
    /// 如果成功删除字符返回 `true`，如果光标已在末尾返回 `false`。
    ///
    /// # 性能
    ///
    /// 时间复杂度为 O(n)，其中 n 是光标位置之后的字符数。
    pub(crate) fn delete(&mut self) -> bool {
        if self.cursor < self.buffer.len() {
            // 找到当前字符的结束位置
            let char_end = self.buffer[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(pos, _)| self.cursor + pos)
                .unwrap_or(self.buffer.len());

            // 移除字符
            self.buffer.drain(self.cursor..char_end);
            true
        } else {
            false
        }
    }

    /// 将光标向左移动一个字符
    ///
    /// 如果光标已在开头，则不执行任何操作。
    ///
    /// # 性能
    ///
    /// 时间复杂度为 O(m)，其中 m 是光标位置之前的字符数（需要遍历找到前一个字符边界）。
    pub(crate) fn move_left(&mut self) {
        if self.cursor > 0 {
            // 找到前一个字符的起始位置
            let prev_char_start = self.buffer[..self.cursor]
                .char_indices()
                .last()
                .map(|(pos, _)| pos)
                .unwrap_or(0);
            self.cursor = prev_char_start;
        }
    }

    /// 将光标向右移动一个字符
    ///
    /// 如果光标已在末尾，则不执行任何操作。
    ///
    /// # 性能
    ///
    /// 时间复杂度为 O(1)，因为只需要找到下一个字符边界。
    pub(crate) fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            // 找到下一个字符的结束位置
            let next_char_end = self.buffer[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(pos, _)| self.cursor + pos)
                .unwrap_or(self.buffer.len());
            self.cursor = next_char_end;
        }
    }

    /// 获取光标所在的（行、列）位置。
    ///
    /// - 行：以 `\n` 分隔，0-based
    /// - 列：当前行内的显示宽度（考虑 Unicode 宽度），0-based
    pub(crate) fn cursor_row_col_display_width(&self) -> (usize, usize) {
        let safe_cursor = self.safe_cursor();
        let before = &self.buffer[..safe_cursor];
        let row = before.as_bytes().iter().filter(|&&b| b == b'\n').count();
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = before[line_start..].width();
        (row, col)
    }

    /// 获取缓冲区内容的字符串切片
    ///
    /// # 返回
    ///
    /// 返回缓冲区内容的 `&str` 引用。
    pub(crate) fn as_str(&self) -> &str {
        &self.buffer
    }

    /// 获取光标位置的显示宽度（考虑 Unicode 字符宽度）
    ///
    /// 计算从字符串开始到光标位置的显示宽度。
    /// 这对于正确显示光标位置很重要，因为某些 Unicode 字符（如中文）占用 2 个显示宽度。
    ///
    /// # 返回
    ///
    /// 返回从字符串开始到光标位置的显示宽度（列数）。
    ///
    /// # 实现细节
    ///
    /// - 如果光标不在字符边界上，会自动调整到最近的字符边界
    /// - 使用 `unicode_width` crate 计算显示宽度
    pub(crate) fn cursor_display_width(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }
        // 确保 cursor 在字符边界上，如果不在则调整到最近的边界
        let safe_cursor = if self.cursor > self.buffer.len() {
            self.buffer.len()
        } else if !self.buffer.is_char_boundary(self.cursor) {
            // 如果不在字符边界上，找到前一个字符边界
            self.buffer[..self.cursor]
                .char_indices()
                .last()
                .map(|(pos, _)| pos)
                .unwrap_or(0)
        } else {
            self.cursor
        };
        let text_before_cursor = &self.buffer[..safe_cursor];
        text_before_cursor.width()
    }

    /// 获取整个缓冲区的显示宽度
    ///
    /// 计算整个缓冲区内容的显示宽度。
    ///
    /// # 返回
    ///
    /// 返回整个缓冲区的显示宽度（列数）。
    pub(crate) fn display_width(&self) -> usize {
        self.buffer.width()
    }

    /// 获取占位符文本
    ///
    /// # 返回
    ///
    /// 如果设置了占位符，返回 `Some(&String)`，否则返回 `None`。
    pub(crate) fn placeholder(&self) -> Option<&String> {
        self.placeholder.as_ref()
    }

    fn safe_cursor(&self) -> usize {
        if self.cursor > self.buffer.len() {
            return self.buffer.len();
        }
        if self.buffer.is_char_boundary(self.cursor) {
            return self.cursor;
        }
        self.buffer[..self.cursor]
            .char_indices()
            .last()
            .map(|(pos, _)| pos)
            .unwrap_or(0)
    }
}

fn byte_index_for_display_width(line: &str, target_col: usize) -> usize {
    if target_col == 0 {
        return 0;
    }

    let mut col = 0usize;
    for (idx, ch) in line.char_indices() {
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if col + w > target_col {
            return idx;
        }
        col += w;
    }
    line.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // CursorLine 和 ValidationStatus 测试
    // ========================================================================

    #[test]
    fn test_cursor_line_equality() {
        assert_eq!(CursorLine::PromptLine, CursorLine::PromptLine);
        assert_eq!(CursorLine::InputLine, CursorLine::InputLine);
        assert_ne!(CursorLine::PromptLine, CursorLine::InputLine);
    }

    #[test]
    fn test_validation_status_equality() {
        assert_eq!(ValidationStatus::Initial, ValidationStatus::Initial);
        assert_eq!(ValidationStatus::Valid, ValidationStatus::Valid);
        assert_eq!(ValidationStatus::Invalid, ValidationStatus::Invalid);
        assert_ne!(ValidationStatus::Initial, ValidationStatus::Valid);
        assert_ne!(ValidationStatus::Valid, ValidationStatus::Invalid);
    }

    // ========================================================================
    // InputEditor 基本操作测试
    // ========================================================================

    #[test]
    fn test_editor_new_empty() {
        let editor = InputEditor::new(None);
        assert_eq!(editor.as_str(), "");
        assert!(editor.placeholder().is_none());
    }

    #[test]
    fn test_editor_new_with_placeholder() {
        let editor = InputEditor::new(Some("Enter text...".to_string()));
        assert_eq!(editor.as_str(), "");
        assert_eq!(editor.placeholder(), Some(&"Enter text...".to_string()));
    }

    #[test]
    fn test_editor_insert_single_char() {
        let mut editor = InputEditor::new(None);
        editor.insert('a');
        assert_eq!(editor.as_str(), "a");
    }

    #[test]
    fn test_editor_insert_multiple_chars() {
        let mut editor = InputEditor::new(None);
        editor.insert('h');
        editor.insert('e');
        editor.insert('l');
        editor.insert('l');
        editor.insert('o');
        assert_eq!(editor.as_str(), "hello");
    }

    #[test]
    fn test_editor_insert_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert('你');
        editor.insert('好');
        assert_eq!(editor.as_str(), "你好");
    }

    #[test]
    fn test_editor_insert_str() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello world");
        assert_eq!(editor.as_str(), "hello world");
    }

    #[test]
    fn test_editor_insert_str_at_cursor() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("ac");
        editor.move_left();
        editor.insert_str("b");
        assert_eq!(editor.as_str(), "abc");
    }

    // ========================================================================
    // Backspace 测试
    // ========================================================================

    #[test]
    fn test_editor_backspace_at_end() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        assert!(editor.backspace());
        assert_eq!(editor.as_str(), "hell");
    }

    #[test]
    fn test_editor_backspace_at_start() {
        let editor = InputEditor::new(None);
        let mut editor = editor;
        assert!(!editor.backspace());
        assert_eq!(editor.as_str(), "");
    }

    #[test]
    fn test_editor_backspace_empty() {
        let mut editor = InputEditor::new(None);
        assert!(!editor.backspace());
    }

    #[test]
    fn test_editor_backspace_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("你好");
        assert!(editor.backspace());
        assert_eq!(editor.as_str(), "你");
        assert!(editor.backspace());
        assert_eq!(editor.as_str(), "");
    }

    #[test]
    fn test_editor_backspace_in_middle() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("abc");
        editor.move_left();
        assert!(editor.backspace());
        assert_eq!(editor.as_str(), "ac");
    }

    // ========================================================================
    // Delete 测试
    // ========================================================================

    #[test]
    fn test_editor_delete_at_start() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        // 移动到开头
        for _ in 0..5 {
            editor.move_left();
        }
        assert!(editor.delete());
        assert_eq!(editor.as_str(), "ello");
    }

    #[test]
    fn test_editor_delete_at_end() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        assert!(!editor.delete());
        assert_eq!(editor.as_str(), "hello");
    }

    #[test]
    fn test_editor_delete_empty() {
        let mut editor = InputEditor::new(None);
        assert!(!editor.delete());
    }

    #[test]
    fn test_editor_delete_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("你好世界");
        // 移动到开头
        for _ in 0..4 {
            editor.move_left();
        }
        assert!(editor.delete());
        assert_eq!(editor.as_str(), "好世界");
    }

    #[test]
    fn test_editor_delete_in_middle() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("abcd");
        editor.move_left();
        editor.move_left();
        assert!(editor.delete());
        assert_eq!(editor.as_str(), "abd");
    }

    // ========================================================================
    // 光标移动测试
    // ========================================================================

    #[test]
    fn test_editor_move_left() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("abc");
        editor.move_left();
        editor.insert('X');
        assert_eq!(editor.as_str(), "abXc");
    }

    #[test]
    fn test_editor_move_left_at_start() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("abc");
        for _ in 0..10 {
            // 移动超过文本长度
            editor.move_left();
        }
        editor.insert('X');
        assert_eq!(editor.as_str(), "Xabc");
    }

    #[test]
    fn test_editor_move_right() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("abc");
        editor.move_left();
        editor.move_left();
        editor.move_right();
        editor.insert('X');
        assert_eq!(editor.as_str(), "abXc");
    }

    #[test]
    fn test_editor_move_right_at_end() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("abc");
        for _ in 0..10 {
            // 移动超过文本长度
            editor.move_right();
        }
        editor.insert('X');
        assert_eq!(editor.as_str(), "abcX");
    }

    #[test]
    fn test_editor_move_left_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("你好");
        editor.move_left();
        editor.insert('X');
        assert_eq!(editor.as_str(), "你X好");
    }

    #[test]
    fn test_editor_move_right_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("你好");
        editor.move_left();
        editor.move_left();
        editor.move_right();
        editor.insert('X');
        assert_eq!(editor.as_str(), "你X好");
    }

    #[test]
    fn test_editor_move_up_down_multiline() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello\nworld");
        // 光标默认在末尾（第二行末尾）
        let (row, _col) = editor.cursor_row_col_display_width();
        assert_eq!(row, 1);

        editor.move_up();
        let (row, _col) = editor.cursor_row_col_display_width();
        assert_eq!(row, 0);

        editor.move_down();
        let (row, _col) = editor.cursor_row_col_display_width();
        assert_eq!(row, 1);
    }

    // ========================================================================
    // 显示宽度测试
    // ========================================================================

    #[test]
    fn test_editor_display_width_ascii() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        assert_eq!(editor.display_width(), 5);
    }

    #[test]
    fn test_editor_display_width_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("你好");
        // 中文字符通常占 2 个显示宽度
        assert_eq!(editor.display_width(), 4);
    }

    #[test]
    fn test_editor_display_width_mixed() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hi你好");
        // 2 ASCII + 2*2 中文 = 6
        assert_eq!(editor.display_width(), 6);
    }

    #[test]
    fn test_editor_display_width_empty() {
        let editor = InputEditor::new(None);
        assert_eq!(editor.display_width(), 0);
    }

    #[test]
    fn test_editor_cursor_display_width_at_start() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        for _ in 0..5 {
            editor.move_left();
        }
        assert_eq!(editor.cursor_display_width(), 0);
    }

    #[test]
    fn test_editor_cursor_display_width_at_end() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        assert_eq!(editor.cursor_display_width(), 5);
    }

    #[test]
    fn test_editor_cursor_display_width_in_middle() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello");
        editor.move_left();
        editor.move_left();
        assert_eq!(editor.cursor_display_width(), 3);
    }

    #[test]
    fn test_editor_cursor_display_width_unicode() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("你好世界");
        editor.move_left();
        editor.move_left();
        // 前两个中文字符 = 4 个显示宽度
        assert_eq!(editor.cursor_display_width(), 4);
    }

    #[test]
    fn test_editor_cursor_display_width_empty() {
        let editor = InputEditor::new(None);
        assert_eq!(editor.cursor_display_width(), 0);
    }

    // ========================================================================
    // 综合测试
    // ========================================================================

    #[test]
    fn test_editor_complex_editing() {
        let mut editor = InputEditor::new(None);

        // 输入 "hello"
        editor.insert_str("hello");
        assert_eq!(editor.as_str(), "hello");

        // 删除最后一个字符
        editor.backspace();
        assert_eq!(editor.as_str(), "hell");

        // 移动到开头并删除第一个字符
        for _ in 0..4 {
            editor.move_left();
        }
        editor.delete();
        assert_eq!(editor.as_str(), "ell");

        // 在开头插入 "w"
        editor.insert('w');
        assert_eq!(editor.as_str(), "well");

        // 移动到末尾并追加
        for _ in 0..3 {
            editor.move_right();
        }
        editor.insert_str(" done");
        assert_eq!(editor.as_str(), "well done");
    }

    #[test]
    fn test_editor_emoji() {
        let mut editor = InputEditor::new(None);
        editor.insert_str("hello 👋");
        assert_eq!(editor.as_str(), "hello 👋");

        editor.backspace();
        assert_eq!(editor.as_str(), "hello ");
    }
}
