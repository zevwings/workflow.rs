//! 输入对话框模块
//!
//! 提供文本输入功能，支持密码模式、验证器、占位符等

use crate::prompt::dialog::error::Result;
use crate::prompt::dialog::raw_mode::RawModeGuard;
use crate::prompt::style::get_theme;
use color_eyre::eyre;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

/// 光标所在的行
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorLine {
    /// 提示行（第1行）
    PromptLine,
    /// 输入行（第2行）
    InputLine,
}

/// 验证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationStatus {
    /// 初始状态（未输入）
    Initial,
    /// 验证通过
    Valid,
    /// 验证失败
    Invalid,
}

/// 输入编辑器，管理输入缓冲区和光标位置
struct InputEditor {
    buffer: String,
    cursor: usize, // 字节位置（不是字符位置）
    placeholder: Option<String>,
}

impl InputEditor {
    fn new(placeholder: Option<String>) -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            placeholder,
        }
    }

    fn insert(&mut self, ch: char) {
        let char_len = ch.len_utf8();
        self.buffer.insert(self.cursor, ch);
        self.cursor += char_len;
    }

    fn backspace(&mut self) -> bool {
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

    fn delete(&mut self) -> bool {
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

    fn move_left(&mut self) {
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

    fn move_right(&mut self) {
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

    fn as_str(&self) -> &str {
        &self.buffer
    }

    /// 获取光标位置的显示宽度（考虑 Unicode 字符宽度）
    /// 返回从字符串开始到光标位置的显示宽度
    fn cursor_display_width(&self) -> usize {
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
    fn display_width(&self) -> usize {
        self.buffer.width()
    }

    fn placeholder(&self) -> Option<&String> {
        self.placeholder.as_ref()
    }
}

/// 验证器 Trait
pub trait Validator: Send + Sync {
    /// 验证输入，返回错误消息（如果验证失败）
    fn validate(&self, input: &str) -> std::result::Result<(), String>;
}

/// 函数式验证器
impl<F> Validator for F
where
    F: Fn(&str) -> std::result::Result<(), String> + Send + Sync,
{
    fn validate(&self, input: &str) -> std::result::Result<(), String> {
        self(input)
    }
}

/// 内置验证器
pub mod validators {
    use super::Validator;

    pub fn required() -> impl Validator {
        move |input: &str| {
            if input.trim().is_empty() {
                Err("此字段为必填项".to_string())
            } else {
                Ok(())
            }
        }
    }

    pub fn email() -> impl Validator {
        move |input: &str| {
            if input.contains('@') && input.contains('.') {
                Ok(())
            } else {
                Err("请输入有效的邮箱地址".to_string())
            }
        }
    }

    pub fn min_length(min: usize) -> impl Validator {
        move |input: &str| {
            // 允许空输入（空输入由 required() 验证器处理）
            if input.is_empty() || input.len() >= min {
                Ok(())
            } else {
                Err(format!("长度至少为 {} 个字符", min))
            }
        }
    }

    pub fn max_length(max: usize) -> impl Validator {
        move |input: &str| {
            if input.len() <= max {
                Ok(())
            } else {
                Err(format!("长度不能超过 {} 个字符", max))
            }
        }
    }

    pub fn length(min: usize, max: usize) -> impl Validator {
        move |input: &str| {
            let len = input.len();
            if len >= min && len <= max {
                Ok(())
            } else {
                Err(format!("长度必须在 {} 到 {} 个字符之间", min, max))
            }
        }
    }

    pub fn url() -> impl Validator {
        move |input: &str| {
            if input.trim().is_empty() {
                return Err("请输入有效的 URL 地址".to_string());
            }
            // 检查是否包含空格（URL 不应该包含未编码的空格）
            if input.contains(' ') {
                return Err("请输入有效的 URL 地址".to_string());
            }
            // 简单的 URL 验证（不依赖外部 crate）
            // 检查是否以 http:// 或 https:// 开头
            let input_lower = input.to_lowercase();
            if !input_lower.starts_with("http://") && !input_lower.starts_with("https://") {
                return Err("请输入有效的 URL 地址（必须使用 http:// 或 https://）".to_string());
            }
            // 检查是否有 host（在 :// 之后至少有一个字符）
            if let Some(after_scheme) = input.split("://").nth(1) {
                if after_scheme.trim().is_empty() {
                    return Err("请输入有效的 URL 地址".to_string());
                }
                // 检查是否包含至少一个点（表示域名）
                if !after_scheme.contains('.') {
                    return Err("请输入有效的 URL 地址".to_string());
                }
            } else {
                return Err("请输入有效的 URL 地址".to_string());
            }
            Ok(())
        }
    }

    pub fn regex(pattern: &'static str, error_msg: Option<&'static str>) -> impl Validator {
        use regex::Regex;
        let re = Regex::new(pattern).expect("无效的正则表达式");
        let error_msg = error_msg
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("输入格式不正确，必须匹配: {}", pattern));
        move |input: &str| {
            if re.is_match(input) {
                Ok(())
            } else {
                Err(error_msg.clone())
            }
        }
    }
}

/// 输入提示构建器
pub struct InputBuilder {
    message: String,
    default: Option<String>,
    placeholder: Option<String>,
    validator: Option<Box<dyn Validator>>,
    password: bool,
    result_title: Option<String>,
}

impl InputBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            placeholder: None,
            validator: None,
            password: false,
            result_title: None,
        }
    }

    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        self.validator = Some(Box::new(validator));
        self
    }

    /// 设置验证器（接受已装箱的验证器）
    /// 用于从 FormField 传递验证器
    pub fn validator_boxed(mut self, validator: Box<dyn Validator + Send + Sync>) -> Self {
        // 转换类型：Box<dyn Validator + Send + Sync> -> Box<dyn Validator>
        // 这是安全的，因为 Validator trait 已经要求 Send + Sync
        self.validator = Some(validator);
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }

    /// 执行提示
    pub fn prompt(self) -> Result<String> {
        let theme = get_theme();

        // 显示提示信息（单独一行，使用 ? 前缀）
        // 注意：只显示 default，不显示 placeholder
        let (question_mark, prompt_text) = if let Some(ref default) = self.default {
            if self.password {
                // 密码模式：显示固定掩码
                ("? ", format!("{}[****]", self.message))
            } else {
                ("? ", format!("{}[{}]", self.message, default))
            }
        } else {
            ("? ", self.message.clone())
        };
        // 应用主题颜色：? 使用 yellow (warning)，文本使用 prompt
        let styled_question = theme.warning.apply(question_mark, theme.enable_color);
        let styled_text = theme.title.apply(&prompt_text, theme.enable_color);

        let mut stdout = std::io::stdout();
        writeln!(stdout, "{}{}", styled_question, styled_text)?;
        stdout.flush()?;

        // 进入原始模式
        let _guard = RawModeGuard::new()?;

        let mut editor = InputEditor::new(self.placeholder.clone());
        // 跟踪验证状态
        let mut validation_status = ValidationStatus::Initial;
        // 跟踪光标所在的行：writeln! 后光标应该在下一行（输入行）
        let mut cursor_line = CursorLine::InputLine;

        // 注意：default 不应该自动填充到输入框
        // default 只在标题行显示，如果用户直接按 Enter 才使用
        // 输入框应该显示 placeholder（如果有），而不是 default

        // 渲染初始状态
        // 确保光标在输入行（writeln! 后应该已经在输入行了，但为了安全，显式确保）
        self.ensure_cursor_on_input_line(&mut cursor_line)?;
        self.render_input(&editor, &theme, &mut cursor_line)?;

        loop {
            // 读取键盘事件
            match event::read() {
                Ok(Event::Key(KeyEvent {
                    code, modifiers, ..
                })) => {
                    match code {
                        KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                            if c == 'c' {
                                // Ctrl+C: 直接返回，不执行任何其他操作
                                // RawModeGuard 会在 drop 时自动恢复终端状态
                                return Err(eyre::eyre!("User cancelled"));
                            }
                        }
                        KeyCode::Char(c) => {
                            editor.insert(c);
                            // 从第一个字符开始，每次输入都进行实时验证
                            // 先验证并更新提示行状态（如果需要）
                            self.validate_and_update_prompt(
                                &editor,
                                &theme,
                                &mut validation_status,
                                &mut cursor_line,
                            )?;
                            // 渲染输入（render_input 会确保光标在输入行的正确位置）
                            // 如果提示行状态改变了，render_prompt_line 会更新提示行，
                            // 然后 render_input 会重新计算并设置正确的光标位置
                            self.render_input(&editor, &theme, &mut cursor_line)?;
                        }
                        KeyCode::Backspace => {
                            if editor.backspace() {
                                // 删除字符后，立即进行实时验证
                                // 先验证并更新提示行状态（如果需要）
                                self.validate_and_update_prompt(
                                    &editor,
                                    &theme,
                                    &mut validation_status,
                                    &mut cursor_line,
                                )?;
                                // 渲染输入（render_input 会确保光标在输入行的正确位置）
                                self.render_input(&editor, &theme, &mut cursor_line)?;
                            }
                        }
                        KeyCode::Delete => {
                            if editor.delete() {
                                // 删除字符后，立即进行实时验证
                                // 先验证并更新提示行状态（如果需要）
                                self.validate_and_update_prompt(
                                    &editor,
                                    &theme,
                                    &mut validation_status,
                                    &mut cursor_line,
                                )?;
                                // 渲染输入（render_input 会确保光标在输入行的正确位置）
                                self.render_input(&editor, &theme, &mut cursor_line)?;
                            }
                        }
                        KeyCode::Left => {
                            editor.move_left();
                            // 渲染输入（render_input 会确保光标在输入行）
                            self.render_input(&editor, &theme, &mut cursor_line)?;
                        }
                        KeyCode::Right => {
                            editor.move_right();
                            // 渲染输入（render_input 会确保光标在输入行）
                            self.render_input(&editor, &theme, &mut cursor_line)?;
                        }
                        KeyCode::Enter => {
                            let input = editor.as_str().to_string();
                            let final_input = if input.trim().is_empty() && self.default.is_some() {
                                // 如果输入为空且有默认值，使用默认值
                                self.default.as_ref().unwrap().clone()
                            } else {
                                input
                            };

                            // 验证输入
                            if let Some(ref validator) = self.validator {
                                match validator.validate(&final_input) {
                                    Ok(()) => {
                                        // 验证通过，清除输入区域并显示结果
                                        self.clear_and_display_result(
                                            &final_input,
                                            &mut cursor_line,
                                        )?;
                                        return Ok(final_input);
                                    }
                                    Err(_) => {
                                        // 验证失败，更新提示行状态并继续输入
                                        validation_status = ValidationStatus::Invalid;
                                        self.render_prompt_line(
                                            &theme,
                                            validation_status,
                                            &mut cursor_line,
                                        )?;
                                        self.render_input(&editor, &theme, &mut cursor_line)?;
                                    }
                                }
                            } else {
                                // 没有验证器，直接返回
                                self.clear_and_display_result(&final_input, &mut cursor_line)?;
                                return Ok(final_input);
                            }
                        }
                        KeyCode::Esc => {
                            return Err(eyre::eyre!("User cancelled"));
                        }
                        _ => {}
                    }
                }
                Ok(_) => continue,
                Err(e) => return Err(eyre::eyre!("IO error: {}", e)),
            }
        }
    }

    /// 确保光标在输入行
    fn ensure_cursor_on_input_line(&self, cursor_line: &mut CursorLine) -> Result<()> {
        if *cursor_line != CursorLine::InputLine {
            use crossterm::cursor;
            use crossterm::execute;
            let mut stdout = std::io::stdout();
            if *cursor_line == CursorLine::PromptLine {
                // 从提示行下移到输入行
                execute!(stdout, cursor::MoveDown(1))?;
            }
            *cursor_line = CursorLine::InputLine;
            stdout.flush()?;
        }
        Ok(())
    }

    /// 渲染提示行，根据验证状态显示不同的前缀
    fn render_prompt_line(
        &self,
        theme: &crate::prompt::style::Theme,
        validation_status: ValidationStatus,
        cursor_line: &mut CursorLine,
    ) -> Result<()> {
        use crossterm::cursor;
        use crossterm::execute;
        use crossterm::terminal::ClearType;
        use std::io::Write;

        let mut stdout = std::io::stdout();

        // 确保光标在提示行
        if *cursor_line != CursorLine::PromptLine {
            if *cursor_line == CursorLine::InputLine {
                execute!(stdout, cursor::MoveUp(1))?;
            }
            *cursor_line = CursorLine::PromptLine;
        }

        // 清除提示行
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::UntilNewLine))?;

        // 根据验证状态选择前缀和颜色
        let (prefix, prefix_style) = match validation_status {
            ValidationStatus::Initial => ("? ", &theme.warning),
            ValidationStatus::Valid => ("✓ ", &theme.success),
            ValidationStatus::Invalid => ("✗ ", &theme.error),
        };

        // 构建提示文本
        let prompt_text = if let Some(ref default) = self.default {
            if self.password {
                format!("{}[****]", self.message)
            } else {
                format!("{}[{}]", self.message, default)
            }
        } else {
            self.message.clone()
        };

        // 应用样式
        let styled_prefix = prefix_style.apply(prefix, theme.enable_color);
        let styled_text = theme.title.apply(&prompt_text, theme.enable_color);

        write!(stdout, "{}{}", styled_prefix, styled_text)?;
        stdout.flush()?;

        // 回到输入行
        // 注意：只下移一行，不重置列位置
        // 列位置会在 render_input 中重新计算并设置
        execute!(stdout, cursor::MoveDown(1))?;
        *cursor_line = CursorLine::InputLine;

        Ok(())
    }

    fn render_input(
        &self,
        editor: &InputEditor,
        theme: &crate::prompt::style::Theme,
        cursor_line: &mut CursorLine,
    ) -> Result<()> {
        // 在渲染前，确保光标在输入行
        self.ensure_cursor_on_input_line(cursor_line)?;

        let mut stdout = std::io::stdout();
        let debug_enabled = std::env::var("WORKFLOW_DEBUG_INPUT").is_ok();

        // 调试信息输出到文件，避免干扰终端显示
        if debug_enabled {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/workflow_debug.log")
            {
                use std::io::Write;
                let _ = writeln!(
                    file,
                    "[DEBUG] render_input: 开始渲染输入，输入长度: {}",
                    editor.as_str().len()
                );
            }
        }

        // 清除当前行（输入行）
        // 注意：调用此方法时，光标应该在输入行
        // 重要：只清除当前行，不要上移或下移，避免影响提示行或错误行
        // 使用 MoveToColumn(0) 确保光标在当前行的开头，然后清除当前行
        // 注意：MoveToColumn(0) 不会改变行，只改变列，所以不会影响提示行
        if debug_enabled {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/workflow_debug.log")
            {
                use std::io::Write;
                let _ = writeln!(file, "[DEBUG] render_input: 清除当前行（应该是输入行）");
            }
        }
        // 注意：不要使用 MoveUp 或 MoveDown，只使用 MoveToColumn(0) 来确保光标在当前行的开头
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::UntilNewLine))?;

        // 显示输入框前缀（应用主题颜色：> 使用 green (success)）
        let prefix = theme.success.apply("> ", theme.enable_color);
        write!(stdout, "{}", prefix)?;

        // 显示输入或 placeholder
        let display = if editor.as_str().is_empty() {
            // 如果输入为空
            if self.password {
                // 密码模式：不显示任何内容（包括 placeholder），保持空白
                String::new()
            } else if let Some(placeholder) = editor.placeholder() {
                // 普通模式：显示 placeholder（如果有）
                let mut hint_style = theme.hint.clone();
                hint_style.attributes.push(crossterm::style::Attribute::Italic);
                hint_style.apply(placeholder, theme.enable_color)
            } else {
                String::new()
            }
        } else if self.password {
            // 密码模式使用掩码，应用 answer 样式
            // 使用显示宽度而不是字符数量，以正确处理全角字符（中文、emoji 等）
            // 例如：输入 "你好" (显示宽度4) -> 显示 "****" (4个星号)
            let display_width = editor.display_width();
            let mask = "*".repeat(display_width);
            theme.answer.apply(&mask, theme.enable_color)
        } else {
            // 普通输入显示实际内容，应用 answer 样式
            theme.answer.apply(editor.as_str(), theme.enable_color)
        };
        write!(stdout, "{}", display)?;

        // 移动光标到正确位置
        // 注意：前缀 "> " 占2个显示宽度，光标位置需要加上这个偏移
        let prefix_len = 2; // "> " 的显示宽度
        let target_column = if editor.as_str().is_empty() {
            // 输入为空时，光标应该在 prefix 之后（即位置 prefix_len）
            // 无论是否有 placeholder，光标都应该在 prefix 之后
            prefix_len
        } else {
            // 普通模式下，光标位置 = prefix 显示宽度 + 光标位置的显示宽度
            // 使用 display_width 而不是字节位置，以正确处理 Unicode 字符（全角字符、emoji 等）
            prefix_len + editor.cursor_display_width()
        };

        // 使用 MoveToColumn 精确定位光标，避免移动到上一行
        if debug_enabled {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/workflow_debug.log")
            {
                use std::io::Write;
                let _ = writeln!(file, "[DEBUG] render_input: 移动光标到列 {}", target_column);
            }
        }
        execute!(stdout, cursor::MoveToColumn(target_column as u16))?;
        // 显示光标，因为这是输入模式，用户需要看到光标位置
        execute!(stdout, cursor::Show)?;

        stdout.flush()?;
        if debug_enabled {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/workflow_debug.log")
            {
                use std::io::Write;
                let _ = writeln!(file, "[DEBUG] render_input: 完成渲染");
            }
        }
        Ok(())
    }

    /// 实时验证输入并更新提示行状态
    ///
    /// 从第一个字符开始，每次输入/删除字符后都会调用此方法进行验证。
    /// - 如果验证通过，更新提示行为 ✓
    /// - 如果验证失败，更新提示行为 ✗
    /// - 提示行状态会实时更新
    ///
    /// 返回：状态是否改变
    fn validate_and_update_prompt(
        &self,
        editor: &InputEditor,
        theme: &crate::prompt::style::Theme,
        validation_status: &mut ValidationStatus,
        cursor_line: &mut CursorLine,
    ) -> Result<bool> {
        if let Some(ref validator) = self.validator {
            let current_input = editor.as_str();
            let new_status = match validator.validate(current_input) {
                Ok(()) => ValidationStatus::Valid,
                Err(_) => ValidationStatus::Invalid,
            };

            // 如果状态改变，更新提示行
            if *validation_status != new_status {
                *validation_status = new_status;
                self.render_prompt_line(theme, *validation_status, cursor_line)?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn clear_and_display_result(&self, value: &str, cursor_line: &mut CursorLine) -> Result<()> {
        use crate::prompt::style::get_theme;
        use crossterm::cursor;
        use crossterm::execute;
        use crossterm::terminal::ClearType;
        use std::io::Write;

        let mut stdout = std::io::stdout();
        let theme = get_theme();

        // 布局：提示行（第1行） -> 输入行（第2行）
        // 目标：清除提示行和输入行，在提示行位置显示结果

        // 确保光标在输入行
        if *cursor_line != CursorLine::InputLine {
            if *cursor_line == CursorLine::PromptLine {
                execute!(stdout, cursor::MoveDown(1))?;
            }
            *cursor_line = CursorLine::InputLine;
        }

        // 清除输入行
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::UntilNewLine))?;
        // 上移一行到提示行
        execute!(stdout, cursor::MoveUp(1))?;

        // 清除提示行
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::UntilNewLine))?;

        // 在提示行位置显示格式化的结果："> [title] [value]"
        let display_value = if self.password { "****" } else { value };

        // 应用主题颜色：> 使用 prefix（与 confirm 保持一致），标题和答案使用相应样式
        let prefix = theme.prefix.apply("> ", theme.enable_color);
        // 使用 result_title（如果存在），否则使用 message
        let title_text = self.result_title.as_ref().unwrap_or(&self.message);
        let title = theme.title.apply(title_text, theme.enable_color);
        let answer = theme.answer.apply(display_value, theme.enable_color);

        write!(stdout, "{}{} {}", prefix, title, answer)?;
        writeln!(stdout)?;
        // 确保光标在新行的开头，以便后续消息输出正确对齐
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }
}

/// 输入提示宏
///
/// 提供格式化字符串的便捷方式，智能判断是否需要格式化：
/// - 简单字符串字面量：直接传递，不调用 `format!()`
/// - 格式化字符串：使用 `format!()` 进行格式化
/// - 变量或表达式：直接传递，不调用 `format!()`
///
/// # Examples
///
/// ```rust,no_run
/// use workflow::input;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 简单字符串（直接传递，不格式化）
/// let name = input!("Enter your name")
///     .default("John Doe")
///     .prompt()?;
///
/// // 格式化字符串（使用 format!）
/// let value = input!("Enter {} name:", "branch")
///     .default("main")
///     .prompt()?;
///
/// // 变量（直接传递，不格式化）
/// let prompt = "Enter value:";
/// let value = input!(prompt)
///     .prompt()?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! input {
    // 格式化字符串：input!("Message {}", var) 或 input!("Message {}", var1, var2)
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::prompt::InputBuilder::new(format!($fmt, $($arg),+))
    };
    // 简单字符串字面量：input!("Message") - 直接传递，不格式化
    ($msg:literal) => {
        $crate::prompt::InputBuilder::new($msg)
    };
    // 变量或其他表达式：input!(var) - 直接传递，不格式化
    ($expr:expr) => {
        $crate::prompt::InputBuilder::new($expr)
    };
}
