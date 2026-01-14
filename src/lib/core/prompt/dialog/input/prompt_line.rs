//! 提示行渲染

use super::builder::InputBuilder;
use super::editor::{CursorLine, ValidationStatus};
use crate::core::prompt::dialog::{PASSWORD_MASK, PROMPT_PREFIX};
use crate::core::prompt::style::theme::Theme;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::Write;

/// 渲染提示行，根据验证状态显示不同的前缀
pub(super) fn render_prompt_line(
    builder: &InputBuilder,
    theme: &Theme,
    validation_status: ValidationStatus,
    cursor_line: &mut CursorLine,
) -> Result<(), std::io::Error> {
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
        ValidationStatus::Initial => (PROMPT_PREFIX, &theme.warning),
        ValidationStatus::Valid => ("✓ ", &theme.success),
        ValidationStatus::Invalid => ("✗ ", &theme.error),
    };

    // 构建提示文本
    let prompt_text = if let Some(ref default) = builder.default {
        if builder.password {
            format!("{}[{}]", builder.message, PASSWORD_MASK)
        } else {
            format!("{}[{}]", builder.message, default)
        }
    } else {
        builder.message.clone()
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
