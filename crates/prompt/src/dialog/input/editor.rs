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
}
