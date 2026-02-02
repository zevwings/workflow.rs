//! 输入提示主逻辑

use crate::dialog::input::builder::InputBuilder;
use crate::dialog::input::editor::{CursorLine, InputEditor, ValidationStatus};
use crate::dialog::{common::RawModeGuard, Result};
use crate::dialog::{PASSWORD_MASK, PROMPT_PREFIX, RESULT_PREFIX};
use crate::error::PromptError;
use crate::style::theme::{get_theme, Theme};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::Write;

/// 获取调试日志文件路径（跨平台）
fn debug_log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("workflow_debug.log")
}

/// 渲染提示行，根据验证状态显示不同的前缀
fn render_prompt_line(
    builder: &InputBuilder,
    theme: &Theme,
    validation_status: ValidationStatus,
    cursor_line: &mut CursorLine,
) -> Result<()> {
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

/// 实时验证输入并更新提示行状态
///
/// 从第一个字符开始，每次输入/删除字符后都会调用此方法进行验证。
/// - 如果验证通过，更新提示行为 ✓
/// - 如果验证失败，更新提示行为 ✗
/// - 提示行状态会实时更新
///
/// 返回：状态是否改变
fn validate_and_update_prompt(
    builder: &InputBuilder,
    editor: &InputEditor,
    theme: &Theme,
    validation_status: &mut ValidationStatus,
    cursor_line: &mut CursorLine,
) -> Result<bool> {
    if let Some(ref validator) = builder.validator {
        let current_input = editor.as_str();
        let new_status = match validator.validate(current_input) {
            Ok(()) => ValidationStatus::Valid,
            Err(_) => ValidationStatus::Invalid,
        };

        // 如果状态改变，更新提示行
        if *validation_status != new_status {
            *validation_status = new_status;
            render_prompt_line(builder, theme, *validation_status, cursor_line)?;
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// 清除输入区域并显示结果
fn clear_and_display_result(
    builder: &InputBuilder,
    value: &str,
    cursor_line: &mut CursorLine,
) -> Result<()> {
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
    let display_value = if builder.password {
        PASSWORD_MASK
    } else {
        value
    };

    // 应用主题颜色：> 使用 prefix（与 confirm 保持一致），标题和答案使用相应样式
    let prefix = theme.prefix.apply(RESULT_PREFIX, theme.enable_color);
    // 使用 result_title（如果存在），否则使用 message
    let title_text = builder.result_title.as_ref().unwrap_or(&builder.message);
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

/// 确保光标在输入行
pub(super) fn ensure_cursor_on_input_line(cursor_line: &mut CursorLine) -> Result<()> {
    if *cursor_line != CursorLine::InputLine {
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

/// 渲染输入行
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
            .open(debug_log_path())
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
            .open(debug_log_path())
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
            hint_style
                .attributes
                .push(crossterm::style::Attribute::Italic);
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
            .open(debug_log_path())
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
            .open(debug_log_path())
        {
            let _ = writeln!(file, "[DEBUG] render_input: 完成渲染");
        }
    }
    Ok(())
}

/// 执行提示
pub(super) fn prompt(builder: InputBuilder) -> Result<String> {
    let theme = get_theme();

    // 显示提示信息（单独一行，使用 ? 前缀）
    // 注意：只显示 default，不显示 placeholder
    // 应用主题颜色：? 使用 yellow (warning)，message 使用 title，default value 使用 hint
    let styled_question = theme.warning.apply(PROMPT_PREFIX, theme.enable_color);
    let styled_message = theme.title.apply(&builder.message, theme.enable_color);

    let styled_text = if let Some(ref default) = builder.default {
        if builder.password {
            // 密码模式：显示固定掩码
            let styled_default = theme
                .hint
                .apply(&format!("[{}]", PASSWORD_MASK), theme.enable_color);
            format!("{} {}", styled_message, styled_default)
        } else {
            let styled_default = theme
                .hint
                .apply(&format!("[{}]", default), theme.enable_color);
            format!("{} {}", styled_message, styled_default)
        }
    } else {
        styled_message
    };

    let mut stdout = std::io::stdout();
    writeln!(stdout, "{}{}", styled_question, styled_text)?;
    stdout.flush()?;

    // 进入原始模式
    let _guard = RawModeGuard::new()?;

    let mut editor = InputEditor::new(builder.placeholder.clone());
    // 跟踪验证状态
    let mut validation_status = ValidationStatus::Initial;
    // 跟踪光标所在的行：writeln! 后光标应该在下一行（输入行）
    let mut cursor_line = CursorLine::InputLine;

    // 注意：default 不应该自动填充到输入框
    // default 只在标题行显示，如果用户直接按 Enter 才使用
    // 输入框应该显示 placeholder（如果有），而不是 default

    // 渲染初始状态
    // 确保光标在输入行（writeln! 后应该已经在输入行了，但为了安全，显式确保）
    ensure_cursor_on_input_line(&mut cursor_line)?;
    render_input(&builder, &editor, &theme, &mut cursor_line)?;

    loop {
        // 读取键盘事件
        match event::read() {
            Ok(Event::Paste(text)) => {
                // 处理粘贴事件：使用 insert_str 批量插入文本，比逐个字符插入更高效
                editor.insert_str(&text);
                // 粘贴后，立即进行实时验证
                validate_and_update_prompt(
                    &builder,
                    &editor,
                    &theme,
                    &mut validation_status,
                    &mut cursor_line,
                )?;
                // 渲染输入（render_input 会确保光标在输入行的正确位置）
                render_input(&builder, &editor, &theme, &mut cursor_line)?;
            }
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => {
                match code {
                    KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                        if c == 'c' {
                            // Ctrl+C: 输出统一的取消提示，然后返回取消错误
                            // RawModeGuard 会在 drop 时自动恢复终端状态
                            if let Err(e) = crate::dialog::common::print_cancelled_message() {
                                return Err(PromptError::Io(e));
                            }
                            return Err(PromptError::Cancelled);
                        }
                    }
                    KeyCode::Char(c) => {
                        editor.insert(c);
                        // 从第一个字符开始，每次输入都进行实时验证
                        // 先验证并更新提示行状态（如果需要）
                        validate_and_update_prompt(
                            &builder,
                            &editor,
                            &theme,
                            &mut validation_status,
                            &mut cursor_line,
                        )?;
                        // 渲染输入（render_input 会确保光标在输入行的正确位置）
                        // 如果提示行状态改变了，render_prompt_line 会更新提示行，
                        // 然后 render_input 会重新计算并设置正确的光标位置
                        render_input(&builder, &editor, &theme, &mut cursor_line)?;
                    }
                    KeyCode::Backspace => {
                        if editor.backspace() {
                            // 删除字符后，立即进行实时验证
                            // 先验证并更新提示行状态（如果需要）
                            validate_and_update_prompt(
                                &builder,
                                &editor,
                                &theme,
                                &mut validation_status,
                                &mut cursor_line,
                            )?;
                            // 渲染输入（render_input 会确保光标在输入行的正确位置）
                            render_input(&builder, &editor, &theme, &mut cursor_line)?;
                        }
                    }
                    KeyCode::Delete => {
                        if editor.delete() {
                            // 删除字符后，立即进行实时验证
                            // 先验证并更新提示行状态（如果需要）
                            validate_and_update_prompt(
                                &builder,
                                &editor,
                                &theme,
                                &mut validation_status,
                                &mut cursor_line,
                            )?;
                            // 渲染输入（render_input 会确保光标在输入行的正确位置）
                            render_input(&builder, &editor, &theme, &mut cursor_line)?;
                        }
                    }
                    KeyCode::Left => {
                        editor.move_left();
                        // 渲染输入（render_input 会确保光标在输入行）
                        render_input(&builder, &editor, &theme, &mut cursor_line)?;
                    }
                    KeyCode::Right => {
                        editor.move_right();
                        // 渲染输入（render_input 会确保光标在输入行）
                        render_input(&builder, &editor, &theme, &mut cursor_line)?;
                    }
                    KeyCode::Enter => {
                        let input = editor.as_str().to_string();
                        let final_input = if input.trim().is_empty() {
                            // 如果输入为空且有默认值，使用默认值
                            builder.default.as_ref().cloned().unwrap_or(input)
                        } else {
                            input
                        };

                        // 验证输入
                        if let Some(ref validator) = builder.validator {
                            match validator.validate(&final_input) {
                                Ok(()) => {
                                    // 验证通过，清除输入区域并显示结果
                                    clear_and_display_result(
                                        &builder,
                                        &final_input,
                                        &mut cursor_line,
                                    )?;
                                    return Ok(final_input);
                                }
                                Err(_) => {
                                    // 验证失败，更新提示行状态并继续输入
                                    validation_status = ValidationStatus::Invalid;
                                    render_prompt_line(
                                        &builder,
                                        &theme,
                                        validation_status,
                                        &mut cursor_line,
                                    )?;
                                    render_input(&builder, &editor, &theme, &mut cursor_line)?;
                                }
                            }
                        } else {
                            // 没有验证器，直接返回
                            clear_and_display_result(&builder, &final_input, &mut cursor_line)?;
                            return Ok(final_input);
                        }
                    }
                    KeyCode::Esc => {
                        // Esc: 输出统一的取消提示，然后返回取消错误
                        if let Err(e) = crate::dialog::common::print_cancelled_message() {
                            return Err(PromptError::Io(e));
                        }
                        return Err(PromptError::Cancelled);
                    }
                    _ => {}
                }
            }
            Ok(_) => continue,
            Err(e) => return Err(PromptError::Io(e)),
        }
    }
}
