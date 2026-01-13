//! 选项列表渲染器
//!
//! 提供 select 和 multiselect 共享的渲染逻辑

use crate::base::interactive::dialog::error::Result;
use crate::base::interactive::style::Theme;
use crossterm::cursor;
use crossterm::execute;
use crossterm::style::ResetColor;
use crossterm::terminal::ClearType;
use std::io::Write;

/// 选项渲染器 trait
///
/// 定义如何渲染单个选项，允许 select 和 multiselect 有不同的渲染方式
pub trait OptionRenderer {
    /// 渲染单个选项
    ///
    /// # 参数
    /// - `index`: 选项索引
    /// - `option_text`: 选项文本
    /// - `is_current`: 是否是当前光标位置
    /// - `theme`: 主题样式
    ///
    /// # 返回
    /// 渲染后的行文本（不包含换行符）
    fn render_option(
        &self,
        index: usize,
        option_text: &str,
        is_current: bool,
        theme: &Theme,
    ) -> String;
}

/// 选项列表渲染器
pub struct OptionListRenderer;

impl OptionListRenderer {
    /// 渲染选项列表
    ///
    /// # 参数
    /// - `options`: 选项列表
    /// - `current_index`: 当前光标位置
    /// - `rendered_lines`: 已渲染的行数（用于清除）
    /// - `theme`: 主题样式
    /// - `renderer`: 选项渲染器实现
    /// - `hint_text`: 提示文本
    ///
    /// # 返回
    /// 渲染的总行数（选项数 + 1 提示行）
    pub fn render_options<OR: OptionRenderer>(
        options: &[impl std::fmt::Display],
        current_index: usize,
        rendered_lines: usize,
        theme: &Theme,
        renderer: &OR,
        hint_text: &str,
    ) -> Result<usize> {
        let mut stdout = std::io::stdout();
        let total_lines = options.len() + 1;

        // 清除已渲染的行
        if rendered_lines > 0 {
            Self::clear_rendered_lines(rendered_lines)?;
        }

        // 渲染所有选项
        for (index, option) in options.iter().enumerate() {
            execute!(stdout, cursor::MoveToColumn(0))?;
            execute!(stdout, ResetColor)?;

            let is_current = index == current_index;
            let option_text = option.to_string();
            let rendered_line = renderer.render_option(index, &option_text, is_current, theme);

            write!(stdout, "{}", rendered_line)?;
            execute!(stdout, ResetColor)?;
            writeln!(stdout)?;
        }

        // 显示提示信息
        Self::render_hint(theme, hint_text)?;

        // 隐藏光标
        execute!(stdout, cursor::Hide)?;
        stdout.flush()?;
        Ok(total_lines)
    }

    /// 清除已渲染的行
    fn clear_rendered_lines(rendered_lines: usize) -> Result<()> {
        let mut stdout = std::io::stdout();

        // 上移到已渲染的第一行
        execute!(stdout, cursor::MoveUp(rendered_lines as u16))?;

        // 清除所有已渲染的行
        for i in 0..rendered_lines {
            write!(stdout, "\r")?;
            execute!(stdout, ResetColor)?;
            execute!(stdout, crossterm::terminal::Clear(ClearType::CurrentLine))?;
            if i < rendered_lines - 1 {
                execute!(stdout, cursor::MoveDown(1))?;
            }
        }

        // 清除后，光标在最后一个清除行（提示行）
        // 需要回到第一个选项行：上移 (rendered_lines - 1) 行
        if rendered_lines > 1 {
            execute!(stdout, cursor::MoveUp((rendered_lines - 1) as u16))?;
        }

        Ok(())
    }

    /// 渲染提示信息
    fn render_hint(theme: &Theme, hint_text: &str) -> Result<()> {
        let mut stdout = std::io::stdout();
        execute!(stdout, cursor::MoveToColumn(0))?;
        let hint_styled = theme.hint.apply(hint_text, theme.enable_color);
        writeln!(stdout, "{}", hint_styled)?;
        execute!(stdout, ResetColor)?;
        Ok(())
    }

    /// 清除并显示结果
    ///
    /// # 参数
    /// - `options_count`: 选项数量
    /// - `message`: 提示消息
    /// - `result_text`: 结果文本
    /// - `theme`: 主题样式
    pub fn clear_and_display_result(
        options_count: usize,
        message: &str,
        result_text: &str,
        theme: &Theme,
    ) -> Result<()> {
        let mut stdout = std::io::stdout();

        // 计算需要清除的行数：
        // - 选项行：options_count
        // - 提示信息行：1（"使用 ↑/↓ 导航，回车确认"）
        // - 提示行：1（"? 请选择一个选项"）
        // 总共：options_count + 2
        let lines_to_clear = options_count + 2;

        // 当前光标在提示信息行的下一行（因为 render_hint 输出了换行符）
        // 先向上移动一行回到提示信息行
        execute!(stdout, cursor::MoveUp(1))?;

        // 从提示信息行开始向上清除所有行
        // 先清除当前行（提示信息行）
        write!(stdout, "\r")?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::CurrentLine))?;

        // 向上移动并清除每一行（包括所有选项行和提示行）
        for _ in 0..(lines_to_clear - 1) {
            execute!(stdout, cursor::MoveUp(1))?;
            write!(stdout, "\r")?;
            execute!(stdout, ResetColor)?;
            execute!(stdout, crossterm::terminal::Clear(ClearType::CurrentLine))?;
        }

        // 此时光标在提示行位置（"? 请选择一个选项"），显示格式化的结果："> [title] [value]"
        let prefix = theme.success.apply("> ", theme.enable_color);
        let title = theme.title.apply(message, theme.enable_color);
        let answer = theme.answer.apply(result_text, theme.enable_color);

        write!(stdout, "{}{} {}", prefix, title, answer)?;
        writeln!(stdout)?;
        // 确保光标在新行的开头，以便后续消息输出正确对齐
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, ResetColor)?;
        execute!(stdout, cursor::Show)?;
        stdout.flush()?;
        Ok(())
    }
}
