//! 输入行渲染

use super::builder::InputBuilder;
use super::editor::{CursorLine, InputEditor};
use super::prompt::ensure_cursor_on_input_line;
use crate::core::prompt::dialog::Result;
use crate::core::prompt::style::theme::Theme;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::Write;

pub(super) fn render_input(
    builder: &InputBuilder,
    editor: &InputEditor,
    theme: &Theme,
    cursor_line: &mut CursorLine,
) -> Result<()> {
    // 在渲染前，确保光标在输入行
    ensure_cursor_on_input_line(cursor_line)?;

    let mut stdout = std::io::stdout();
    let debug_enabled = std::env::var("WORKFLOW_DEBUG_INPUT").is_ok();

    // 调试信息输出到文件，避免干扰终端显示
    if debug_enabled {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/workflow_debug.log")
        {
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
        if builder.password {
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
    } else if builder.password {
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
            let _ = writeln!(file, "[DEBUG] render_input: 完成渲染");
        }
    }
    Ok(())
}
