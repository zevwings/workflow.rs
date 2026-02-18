//! 输入提示主逻辑

use std::io::Write as _;

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Attribute,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    backend::{Backend, TerminalBackend},
    dialog::{
        input::{
            builder::InputBuilder,
            editor::{CursorLine, InputEditor, ValidationStatus},
        },
        Result, PASSWORD_MASK, PROMPT_PREFIX, RESULT_PREFIX,
    },
    error::PromptError,
    style::theme::{get_theme, Theme},
};

fn normalize_paste_text(text: &str, multiline: bool) -> String {
    // 统一换行：\r\n / \r -> \n
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if multiline {
        normalized
    } else {
        // 单行输入不允许换行，避免破坏 UI
        normalized.replace('\n', " ")
    }
}

/// 获取调试日志文件路径（跨平台）
fn debug_log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("workflow_debug.log")
}

/// 渲染提示行，根据验证状态显示不同的前缀
fn render_prompt_line<B: Backend>(
    backend: &mut B,
    builder: &InputBuilder,
    theme: &Theme,
    validation_status: ValidationStatus,
    cursor_line: &mut CursorLine,
) -> Result<()> {
    // 确保光标在提示行
    if *cursor_line != CursorLine::PromptLine {
        if *cursor_line == CursorLine::InputLine {
            backend.move_up(1)?;
        }
        *cursor_line = CursorLine::PromptLine;
    }

    // 清除提示行
    backend.move_to_column(0)?;
    backend.clear_line()?;

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

    backend.write(&format!("{}{}", styled_prefix, styled_text))?;
    backend.flush()?;

    // 回到输入行
    backend.move_down(1)?;
    *cursor_line = CursorLine::InputLine;

    Ok(())
}

/// 实时验证输入并更新提示行状态
fn validate_and_update_prompt<B: Backend>(
    backend: &mut B,
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

        if *validation_status != new_status {
            *validation_status = new_status;
            render_prompt_line(backend, builder, theme, *validation_status, cursor_line)?;
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

/// 清除输入区域并显示结果
fn clear_and_display_result<B: Backend>(
    backend: &mut B,
    builder: &InputBuilder,
    value: &str,
    cursor_line: &mut CursorLine,
) -> Result<()> {
    let theme = get_theme();

    // 确保光标在输入行
    if *cursor_line != CursorLine::InputLine {
        if *cursor_line == CursorLine::PromptLine {
            backend.move_down(1)?;
        }
        *cursor_line = CursorLine::InputLine;
    }

    // 清除输入行
    backend.move_to_column(0)?;
    backend.clear_line()?;
    // 上移一行到提示行
    backend.move_up(1)?;

    // 清除提示行
    backend.move_to_column(0)?;
    backend.clear_line()?;

    // 在提示行位置显示格式化的结果
    let display_value = if builder.password {
        PASSWORD_MASK
    } else {
        value
    };

    let prefix = theme.prefix.apply(RESULT_PREFIX, theme.enable_color);
    let title_text = builder.result_title.as_ref().unwrap_or(&builder.message);
    let title = theme.title.apply(title_text, theme.enable_color);
    let answer = theme.answer.apply(display_value, theme.enable_color);

    backend.write(&format!("{}{} {}", prefix, title, answer))?;
    backend.writeln("")?;
    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}

/// 确保光标在输入行
fn ensure_cursor_on_input_line<B: Backend>(
    backend: &mut B,
    cursor_line: &mut CursorLine,
) -> Result<()> {
    if *cursor_line != CursorLine::InputLine {
        if *cursor_line == CursorLine::PromptLine {
            backend.move_down(1)?;
        }
        *cursor_line = CursorLine::InputLine;
        backend.flush()?;
    }
    Ok(())
}

/// 渲染输入行
fn render_input<B: Backend>(
    backend: &mut B,
    builder: &InputBuilder,
    editor: &InputEditor,
    theme: &Theme,
    cursor_line: &mut CursorLine,
) -> Result<()> {
    ensure_cursor_on_input_line(backend, cursor_line)?;

    let debug_enabled = std::env::var("WORKFLOW_DEBUG_INPUT").is_ok();

    if debug_enabled {
        if let Ok(mut file) =
            std::fs::OpenOptions::new().create(true).append(true).open(debug_log_path())
        {
            let _ = writeln!(
                file,
                "[DEBUG] render_input: Start rendering input, input length: {}",
                editor.as_str().len()
            );
        }
    }

    backend.move_to_column(0)?;
    backend.clear_line()?;

    // 显示输入框前缀
    let prefix = theme.success.apply("> ", theme.enable_color);
    backend.write(&prefix)?;

    // 显示输入或 placeholder
    let display = if editor.as_str().is_empty() {
        if builder.password {
            String::new()
        } else if let Some(placeholder) = editor.placeholder() {
            let mut hint_style = theme.hint.clone();
            hint_style.attributes.push(Attribute::Italic);
            hint_style.apply(placeholder, theme.enable_color)
        } else {
            String::new()
        }
    } else if builder.password {
        let display_width = editor.display_width();
        let mask = "*".repeat(display_width);
        theme.answer.apply(&mask, theme.enable_color)
    } else {
        theme.answer.apply(editor.as_str(), theme.enable_color)
    };
    backend.write(&display)?;

    // 移动光标到正确位置
    let prefix_len = 2;
    let target_column = if editor.as_str().is_empty() {
        prefix_len
    } else {
        prefix_len + editor.cursor_display_width()
    };

    if debug_enabled {
        if let Ok(mut file) =
            std::fs::OpenOptions::new().create(true).append(true).open(debug_log_path())
        {
            let _ = writeln!(file, "[DEBUG] render_input: Move cursor to column {}", target_column);
        }
    }

    backend.move_to_column(target_column as u16)?;
    backend.show_cursor()?;
    backend.flush()?;

    if debug_enabled {
        if let Ok(mut file) =
            std::fs::OpenOptions::new().create(true).append(true).open(debug_log_path())
        {
            let _ = writeln!(file, "[DEBUG] render_input: Completed rendering");
        }
    }
    Ok(())
}

fn revalidate_status(
    builder: &InputBuilder,
    editor: &InputEditor,
    current: ValidationStatus,
) -> ValidationStatus {
    let Some(ref validator) = builder.validator else {
        return current;
    };

    match validator.validate(editor.as_str()) {
        Ok(()) => ValidationStatus::Valid,
        Err(_) => ValidationStatus::Invalid,
    }
}

fn render_multiline<B: Backend>(
    backend: &mut B,
    builder: &InputBuilder,
    editor: &InputEditor,
    theme: &Theme,
    validation_status: ValidationStatus,
    cursor_row: &mut u16,
    rendered_input_lines: &mut u16,
) -> Result<()> {
    // 当前位置在输入区内：向上回到 prompt 行
    backend.move_up(cursor_row.saturating_add(1))?;
    backend.move_to_column(0)?;
    backend.clear_line()?;

    // 渲染 prompt 行（带校验状态）
    let (prefix, prefix_style) = match validation_status {
        ValidationStatus::Initial => (PROMPT_PREFIX, &theme.warning),
        ValidationStatus::Valid => ("✓ ", &theme.success),
        ValidationStatus::Invalid => ("✗ ", &theme.error),
    };

    let prompt_text = if let Some(ref default) = builder.default {
        if builder.password {
            format!("{}[{}]", builder.message, PASSWORD_MASK)
        } else {
            format!("{}[{}]", builder.message, default)
        }
    } else {
        builder.message.clone()
    };

    let styled_prefix = prefix_style.apply(prefix, theme.enable_color);
    let styled_text = theme.title.apply(&prompt_text, theme.enable_color);
    backend.write(&format!("{}{}", styled_prefix, styled_text))?;
    backend.flush()?;

    // 下移到输入区首行
    backend.move_down(1)?;

    // 清理旧输入区（多行）
    let old_lines = (*rendered_input_lines).max(1);
    for i in 0..old_lines {
        backend.move_to_column(0)?;
        backend.clear_line()?;
        if i + 1 < old_lines {
            backend.move_down(1)?;
        }
    }
    // 回到输入区首行，开始重绘
    backend.move_up(old_lines.saturating_sub(1))?;

    // =============================================================================
    // 计算“要渲染的可视行”
    //
    // 关键点：placeholder 可能包含 '\n'。在 raw mode 下直接写入包含 '\n' 的字符串，
    // 会出现换行不回到列 0 的情况，导致缩进越来越深、甚至和其他输出叠在同一行。
    // 因此这里必须将 placeholder 按行拆开渲染。
    // =============================================================================

    let (visual_lines, cursor_row_target, cursor_col_target): (Vec<String>, u16, u16) =
        if editor.as_str().is_empty() && !builder.password {
            if let Some(placeholder) = editor.placeholder() {
                let mut hint_style = theme.hint.clone();
                hint_style.attributes.push(Attribute::Italic);
                let raw_lines: Vec<&str> = placeholder.split('\n').collect();
                let lines = if raw_lines.is_empty() {
                    vec![String::new()]
                } else {
                    raw_lines.into_iter().map(|l| hint_style.apply(l, theme.enable_color)).collect()
                };
                // 空输入时，光标应在第一行起始位置（在 "> " 之后）
                (lines, 0, 0)
            } else {
                (vec![String::new()], 0, 0)
            }
        } else if builder.password {
            // 多行密码：按行掩码，保持换行结构与光标定位一致
            let raw_lines: Vec<&str> = editor.as_str().split('\n').collect();
            let lines = if raw_lines.is_empty() {
                vec![String::new()]
            } else {
                raw_lines
                    .into_iter()
                    .map(|l| {
                        let mask = "*".repeat(l.width());
                        theme.answer.apply(&mask, theme.enable_color)
                    })
                    .collect()
            };
            let (row, col) = editor.cursor_row_col_display_width();
            (
                lines,
                u16::try_from(row).unwrap_or(0),
                u16::try_from(col).unwrap_or(0),
            )
        } else {
            let raw_lines: Vec<&str> = editor.as_str().split('\n').collect();
            let lines = if raw_lines.is_empty() {
                vec![String::new()]
            } else {
                raw_lines
                    .into_iter()
                    .map(|l| theme.answer.apply(l, theme.enable_color))
                    .collect()
            };
            let (row, col) = editor.cursor_row_col_display_width();
            (
                lines,
                u16::try_from(row).unwrap_or(0),
                u16::try_from(col).unwrap_or(0),
            )
        };

    let new_lines = (visual_lines.len() as u16).max(1);

    for (i, display) in visual_lines.iter().enumerate() {
        backend.move_to_column(0)?;

        let prefix = if i == 0 {
            theme.success.apply("> ", theme.enable_color)
        } else {
            "  ".to_string()
        };

        if i + 1 < visual_lines.len() {
            backend.writeln(&format!("{}{}", prefix, display))?;
        } else {
            backend.write(&format!("{}{}", prefix, display))?;
        }
    }

    // 写完后光标在最后一行末尾，移动到目标行列
    if new_lines > 0 {
        let last_row = new_lines - 1;
        let target_row = cursor_row_target.min(last_row);
        let up = last_row.saturating_sub(target_row);
        backend.move_up(up)?;
    }

    // 每行前缀宽度固定为 2（"> " 或 "  "）
    backend.move_to_column(2 + cursor_col_target)?;
    backend.show_cursor()?;
    backend.flush()?;

    *cursor_row = cursor_row_target;
    *rendered_input_lines = new_lines;
    Ok(())
}

fn clear_and_display_result_multiline<B: Backend>(
    backend: &mut B,
    builder: &InputBuilder,
    value: &str,
    cursor_row: u16,
    rendered_input_lines: u16,
) -> Result<()> {
    let theme = get_theme();

    // 回到 prompt 行
    backend.move_up(cursor_row.saturating_add(1))?;
    backend.move_to_column(0)?;
    backend.clear_line()?;
    backend.move_down(1)?;

    // 清空输入区
    let lines = rendered_input_lines.max(1);
    for i in 0..lines {
        backend.move_to_column(0)?;
        backend.clear_line()?;
        if i + 1 < lines {
            backend.move_down(1)?;
        }
    }

    // 回到 prompt 行并输出结果
    backend.move_up(lines)?;
    backend.move_to_column(0)?;

    // 多行输入只显示第一行并添加省略号
    let display_value = if builder.password {
        PASSWORD_MASK
    } else {
        value
    };
    let prefix = theme.prefix.apply(RESULT_PREFIX, theme.enable_color);
    let title_text = builder.result_title.as_ref().unwrap_or(&builder.message);
    let title = theme.title.apply(title_text, theme.enable_color);

    // 只显示第一行，如果有多行则添加省略号
    let first_line = display_value.lines().next().unwrap_or("");
    let result_text = if display_value.contains('\n') {
        format!("{}...", first_line)
    } else {
        first_line.to_string()
    };
    let answer = theme.answer.apply(&result_text, theme.enable_color);
    backend.writeln(&format!("{}{} {}", prefix, title, answer))?;

    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}

/// 打印取消消息
fn print_cancelled_message<B: Backend>(
    backend: &mut B,
    editor: &InputEditor,
    cursor_line: &mut CursorLine,
) -> Result<()> {
    let theme = get_theme();

    // 确保光标在输入行
    ensure_cursor_on_input_line(backend, cursor_line)?;

    // 清除输入行并显示已输入的内容（如果有）
    backend.move_to_column(0)?;
    backend.clear_line()?;

    if !editor.as_str().is_empty() {
        let prefix = theme.success.apply("> ", theme.enable_color);
        let input_text = theme.answer.apply(editor.as_str(), theme.enable_color);
        backend.writeln(&format!("{}{}", prefix, input_text))?;
        backend.move_to_column(0)?;
    } else {
        // 如果没有输入，直接换行
        backend.writeln("")?;
        backend.move_to_column(0)?;
    }

    // 显示取消消息（不删除提示行）
    let cancel_prefix = theme.warning.apply("! ", theme.enable_color);
    let cancel_message = theme.hint.apply("Operation cancelled", theme.enable_color);
    backend.writeln(&format!("{}{}", cancel_prefix, cancel_message))?;
    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}

/// 多行输入模式下打印取消消息（需要先清除已渲染的输入内容）
fn print_cancelled_message_multiline<B: Backend>(
    backend: &mut B,
    editor: &InputEditor,
    cursor_row: u16,
    rendered_input_lines: u16,
) -> Result<()> {
    let theme = get_theme();

    // 回到输入区首行
    backend.move_up(cursor_row)?;
    backend.move_to_column(0)?;

    let lines = rendered_input_lines.max(1);

    if !editor.as_str().is_empty() {
        // 清空并在第一行显示输入内容
        backend.clear_line()?;
        let prefix = theme.success.apply("> ", theme.enable_color);
        let first_line = editor.as_str().lines().next().unwrap_or("");
        let input_text = if editor.as_str().contains('\n') {
            format!("{}...", first_line)
        } else {
            first_line.to_string()
        };
        let styled_input = theme.answer.apply(&input_text, theme.enable_color);
        backend.writeln(&format!("{}{}", prefix, styled_input))?;
        backend.move_to_column(0)?;

        // 清空剩余的输入行（第2行到第N行），但保持光标在第2行
        if lines > 1 {
            for i in 1..lines {
                backend.clear_line()?;
                if i + 1 < lines {
                    backend.move_down(1)?;
                    backend.move_to_column(0)?;
                }
            }
            // 移回到第2行（取消消息将显示的位置）
            if lines > 2 {
                backend.move_up(lines - 2)?;
                backend.move_to_column(0)?;
            }
        }
    } else {
        // 没有输入，清空所有输入行
        for i in 0..lines {
            backend.clear_line()?;
            if i + 1 < lines {
                backend.move_down(1)?;
                backend.move_to_column(0)?;
            }
        }
        // 换行到下一行
        backend.writeln("")?;
        backend.move_to_column(0)?;
    }

    // 显示取消消息（不删除提示行）
    let prefix = theme.warning.apply("! ", theme.enable_color);
    let message = theme.hint.apply("Operation cancelled", theme.enable_color);
    backend.writeln(&format!("{}{}", prefix, message))?;
    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}

/// 使用指定后端执行提示
pub(super) fn prompt_with_backend<B: Backend>(
    builder: InputBuilder,
    backend: &mut B,
) -> Result<String> {
    let theme = get_theme();

    // 显示提示信息
    let styled_question = theme.warning.apply(PROMPT_PREFIX, theme.enable_color);
    let styled_message = theme.title.apply(&builder.message, theme.enable_color);

    let styled_text = if let Some(ref default) = builder.default {
        if builder.password {
            let styled_default =
                theme.hint.apply(&format!("[{}]", PASSWORD_MASK), theme.enable_color);
            format!("{} {}", styled_message, styled_default)
        } else {
            let styled_default = theme.hint.apply(&format!("[{}]", default), theme.enable_color);
            format!("{} {}", styled_message, styled_default)
        }
    } else {
        styled_message
    };

    backend.writeln(&format!("{}{}", styled_question, styled_text))?;
    backend.flush()?;

    // 进入原始模式
    backend.enable_raw_mode()?;
    backend.enable_bracketed_paste()?;
    // 尝试启用增强键盘事件，以便兼容终端能区分 Enter/Shift+Enter 等组合键。
    backend.enable_keyboard_enhancement().ok();

    // 使用 scopeguard 确保退出时恢复状态
    let result = prompt_loop(backend, &builder, &theme);

    // 恢复终端状态
    backend.disable_bracketed_paste().ok();
    backend.disable_keyboard_enhancement().ok();
    backend.disable_raw_mode().ok();

    result
}

/// 主事件循环
fn prompt_loop<B: Backend>(
    backend: &mut B,
    builder: &InputBuilder,
    theme: &Theme,
) -> Result<String> {
    let mut editor = InputEditor::new(builder.placeholder.clone());
    let mut validation_status = ValidationStatus::Initial;
    let mut cursor_line = CursorLine::InputLine;
    let mut cursor_row: u16 = 0;
    let mut rendered_input_lines: u16 = 1;

    if builder.multiline {
        // 多行模式：由渲染器负责 prompt + 输入区整体重绘
        render_multiline(
            backend,
            builder,
            &editor,
            theme,
            validation_status,
            &mut cursor_row,
            &mut rendered_input_lines,
        )?;
    } else {
        ensure_cursor_on_input_line(backend, &mut cursor_line)?;
        render_input(backend, builder, &editor, theme, &mut cursor_line)?;
    }

    loop {
        match backend.read_event() {
            Ok(Event::Paste(text)) => {
                let pasted = normalize_paste_text(&text, builder.multiline);
                editor.insert_str(&pasted);

                if builder.multiline {
                    validation_status = revalidate_status(builder, &editor, validation_status);
                    render_multiline(
                        backend,
                        builder,
                        &editor,
                        theme,
                        validation_status,
                        &mut cursor_row,
                        &mut rendered_input_lines,
                    )?;
                } else {
                    validate_and_update_prompt(
                        backend,
                        builder,
                        &editor,
                        theme,
                        &mut validation_status,
                        &mut cursor_line,
                    )?;
                    render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                }
            }
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => match code {
                KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                    if c == 'c' {
                        if builder.multiline {
                            print_cancelled_message_multiline(
                                backend,
                                &editor,
                                cursor_row,
                                rendered_input_lines,
                            )?;
                        } else {
                            print_cancelled_message(backend, &editor, &mut cursor_line)?;
                        }
                        return Err(PromptError::Cancelled);
                    }
                    // 兼容：部分终端无法区分 Enter / Shift+Enter（两者都会上报为 Enter）。
                    // 在多行模式下，提供 Ctrl+J 作为“插入换行”的备用按键。
                    if builder.multiline && c == 'j' {
                        editor.insert('\n');
                        validation_status = revalidate_status(builder, &editor, validation_status);
                        render_multiline(
                            backend,
                            builder,
                            &editor,
                            theme,
                            validation_status,
                            &mut cursor_row,
                            &mut rendered_input_lines,
                        )?;
                        continue;
                    }
                }
                KeyCode::Char(c) => {
                    editor.insert(c);

                    if builder.multiline {
                        validation_status = revalidate_status(builder, &editor, validation_status);
                        render_multiline(
                            backend,
                            builder,
                            &editor,
                            theme,
                            validation_status,
                            &mut cursor_row,
                            &mut rendered_input_lines,
                        )?;
                    } else {
                        validate_and_update_prompt(
                            backend,
                            builder,
                            &editor,
                            theme,
                            &mut validation_status,
                            &mut cursor_line,
                        )?;
                        render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                    }
                }
                KeyCode::Backspace => {
                    if editor.backspace() {
                        if builder.multiline {
                            validation_status =
                                revalidate_status(builder, &editor, validation_status);
                            render_multiline(
                                backend,
                                builder,
                                &editor,
                                theme,
                                validation_status,
                                &mut cursor_row,
                                &mut rendered_input_lines,
                            )?;
                        } else {
                            validate_and_update_prompt(
                                backend,
                                builder,
                                &editor,
                                theme,
                                &mut validation_status,
                                &mut cursor_line,
                            )?;
                            render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                        }
                    }
                }
                KeyCode::Delete => {
                    if editor.delete() {
                        if builder.multiline {
                            validation_status =
                                revalidate_status(builder, &editor, validation_status);
                            render_multiline(
                                backend,
                                builder,
                                &editor,
                                theme,
                                validation_status,
                                &mut cursor_row,
                                &mut rendered_input_lines,
                            )?;
                        } else {
                            validate_and_update_prompt(
                                backend,
                                builder,
                                &editor,
                                theme,
                                &mut validation_status,
                                &mut cursor_line,
                            )?;
                            render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                        }
                    }
                }
                KeyCode::Left => {
                    editor.move_left();
                    if builder.multiline {
                        render_multiline(
                            backend,
                            builder,
                            &editor,
                            theme,
                            validation_status,
                            &mut cursor_row,
                            &mut rendered_input_lines,
                        )?;
                    } else {
                        render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                    }
                }
                KeyCode::Right => {
                    editor.move_right();
                    if builder.multiline {
                        render_multiline(
                            backend,
                            builder,
                            &editor,
                            theme,
                            validation_status,
                            &mut cursor_row,
                            &mut rendered_input_lines,
                        )?;
                    } else {
                        render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                    }
                }
                KeyCode::Up if builder.multiline => {
                    editor.move_up();
                    render_multiline(
                        backend,
                        builder,
                        &editor,
                        theme,
                        validation_status,
                        &mut cursor_row,
                        &mut rendered_input_lines,
                    )?;
                }
                KeyCode::Down if builder.multiline => {
                    editor.move_down();
                    render_multiline(
                        backend,
                        builder,
                        &editor,
                        theme,
                        validation_status,
                        &mut cursor_row,
                        &mut rendered_input_lines,
                    )?;
                }
                KeyCode::Enter => {
                    if builder.multiline && modifiers.contains(KeyModifiers::SHIFT) {
                        editor.insert('\n');
                        validation_status = revalidate_status(builder, &editor, validation_status);
                        render_multiline(
                            backend,
                            builder,
                            &editor,
                            theme,
                            validation_status,
                            &mut cursor_row,
                            &mut rendered_input_lines,
                        )?;
                        continue;
                    }

                    let input = editor.as_str().to_string();
                    let final_input = if input.trim().is_empty() {
                        builder.default.as_ref().cloned().unwrap_or(input)
                    } else {
                        input
                    };

                    if let Some(ref validator) = builder.validator {
                        match validator.validate(&final_input) {
                            Ok(()) => {
                                if builder.multiline {
                                    clear_and_display_result_multiline(
                                        backend,
                                        builder,
                                        &final_input,
                                        cursor_row,
                                        rendered_input_lines,
                                    )?;
                                } else {
                                    clear_and_display_result(
                                        backend,
                                        builder,
                                        &final_input,
                                        &mut cursor_line,
                                    )?;
                                }
                                return Ok(final_input);
                            }
                            Err(_) => {
                                validation_status = ValidationStatus::Invalid;
                                if builder.multiline {
                                    render_multiline(
                                        backend,
                                        builder,
                                        &editor,
                                        theme,
                                        validation_status,
                                        &mut cursor_row,
                                        &mut rendered_input_lines,
                                    )?;
                                } else {
                                    render_prompt_line(
                                        backend,
                                        builder,
                                        theme,
                                        validation_status,
                                        &mut cursor_line,
                                    )?;
                                    render_input(
                                        backend,
                                        builder,
                                        &editor,
                                        theme,
                                        &mut cursor_line,
                                    )?;
                                }
                            }
                        }
                    } else {
                        if builder.multiline {
                            clear_and_display_result_multiline(
                                backend,
                                builder,
                                &final_input,
                                cursor_row,
                                rendered_input_lines,
                            )?;
                        } else {
                            clear_and_display_result(
                                backend,
                                builder,
                                &final_input,
                                &mut cursor_line,
                            )?;
                        }
                        return Ok(final_input);
                    }
                }
                KeyCode::Esc => {
                    if builder.multiline {
                        print_cancelled_message_multiline(
                            backend,
                            &editor,
                            cursor_row,
                            rendered_input_lines,
                        )?;
                    } else {
                        print_cancelled_message(backend, &editor, &mut cursor_line)?;
                    }
                    return Err(PromptError::Cancelled);
                }
                _ => {}
            },
            Ok(_) => continue,
            Err(e) => return Err(PromptError::Io(e)),
        }
    }
}

/// 执行提示（使用默认终端后端）
pub(super) fn prompt(builder: InputBuilder) -> Result<String> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        if let Some(default) = builder.default {
            return Ok(default);
        }
        return Err(PromptError::Io(std::io::Error::other(
            "Not running in an interactive terminal. Cannot prompt for user input.",
        )));
    }
    let mut backend = TerminalBackend::default();
    prompt_with_backend(builder, &mut backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend::MockBackend, dialog::input::validator::validators};

    #[test]
    fn test_input_basic_input_and_enter() {
        let events = [
            MockBackend::type_string("hello"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_input_empty_with_default() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text")
            .default("default value")
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default value");
    }

    #[test]
    fn test_input_empty_without_default() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_input_override_default() {
        let events = [
            MockBackend::type_string("custom"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text")
            .default("default")
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "custom");
    }

    #[test]
    fn test_input_cancel_with_escape() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::Cancelled));
    }

    #[test]
    fn test_input_cancel_with_ctrl_c() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::Cancelled));
    }

    #[test]
    fn test_input_backspace() {
        let events = [
            MockBackend::type_string("hello"),
            vec![
                Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            ],
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hel");
    }

    #[test]
    fn test_input_delete() {
        let events = [
            MockBackend::type_string("hello"),
            vec![
                Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)),
            ],
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "helo");
    }

    #[test]
    fn test_input_cursor_movement() {
        let events = [
            MockBackend::type_string("ab"),
            vec![
                Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE)),
            ],
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "aXb");
    }

    #[test]
    fn test_input_cursor_left_at_start() {
        // 在开头按左键应该不改变位置
        let events = [
            vec![
                Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            ],
            MockBackend::type_string("test"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_input_cursor_right_at_end() {
        // 在末尾按右键应该不改变位置
        let events = [
            MockBackend::type_string("test"),
            vec![
                Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
                Event::Key(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::NONE)),
            ],
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "testX");
    }

    #[test]
    fn test_input_with_validator_valid() {
        let events = [
            MockBackend::type_string("hello"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text")
            .validator(validators::min_length(3))
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_input_with_validator_invalid_then_valid() {
        // 先输入无效的，然后按 backspace 删除再输入有效的
        let events = [
            MockBackend::type_string("ab"),   // 太短
            vec![MockBackend::press_enter()], // 验证失败，不会返回
            MockBackend::type_string("cde"),  // 继续输入使其有效
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text")
            .validator(validators::min_length(3))
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abcde");
    }

    #[test]
    fn test_input_password_mode() {
        let events = [
            MockBackend::type_string("secret"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result =
            InputBuilder::new("Enter password").password().prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "secret");
    }

    #[test]
    fn test_input_password_with_default() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter password")
            .password()
            .default("default_pass")
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default_pass");
    }

    #[test]
    fn test_input_with_result_title() {
        let events = [
            MockBackend::type_string("test"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text")
            .result_title("Your Input")
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_input_unicode_characters() {
        let events = [
            MockBackend::type_string("你好世界"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "你好世界");
    }

    #[test]
    fn test_input_paste_event() {
        let events = vec![
            Event::Paste("pasted text".to_string()),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pasted text");
    }

    #[test]
    fn test_input_terminal_modes_restored() {
        let events = [
            MockBackend::type_string("test"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        assert!(!backend.is_raw_mode());

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        // 终端模式应该已恢复
        assert!(!backend.is_raw_mode());
    }

    #[test]
    fn test_input_multiline_shift_enter_newline_enter_submit() {
        let events = [
            MockBackend::type_string("hello"),
            vec![MockBackend::press_shift_enter()],
            MockBackend::type_string("world"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").multiline().prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello\nworld");
    }

    #[test]
    fn test_input_multiline_password_shift_enter_newline_enter_submit() {
        let events = [
            MockBackend::type_string("secret"),
            vec![MockBackend::press_shift_enter()],
            MockBackend::type_string("line2"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter password")
            .multiline()
            .password()
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "secret\nline2");
    }

    #[test]
    fn test_input_multiline_placeholder_render_no_embedded_newlines() {
        // placeholder 自身包含多行时，渲染层必须按行输出，
        // 避免 raw mode 下 '\n' 不回到列 0 导致的缩进级联。
        let mut backend = MockBackend::default();
        let theme = get_theme();

        let builder = InputBuilder::new("Changes")
            .multiline()
            .placeholder("For example:\n- Fixed XXX\n- Optimized YYY\n- Added ZZZ");
        let editor = InputEditor::new(builder.placeholder.clone());

        let mut cursor_row = 0u16;
        let mut rendered_input_lines = 1u16;
        render_multiline(
            &mut backend,
            &builder,
            &editor,
            &theme,
            ValidationStatus::Initial,
            &mut cursor_row,
            &mut rendered_input_lines,
        )
        .unwrap();

        // 除了 writeln 追加的末尾换行外，不应出现嵌入的 '\n'
        for s in backend.output() {
            if let Some(stripped) = s.strip_suffix('\n') {
                assert!(
                    !stripped.contains('\n'),
                    "found embedded newline in output chunk: {s:?}"
                );
            } else {
                assert!(
                    !s.contains('\n'),
                    "found embedded newline in output chunk: {s:?}"
                );
            }
        }

        assert!(rendered_input_lines >= 1);
        assert_eq!(cursor_row, 0);
    }

    #[test]
    fn test_input_multiline_result_render_no_embedded_newlines() {
        // 多行结果回显也不能一次性 write 含 '\n' 的字符串
        let events = [
            MockBackend::type_string("hello"),
            vec![MockBackend::press_shift_enter()],
            MockBackend::type_string("world"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text")
            .multiline()
            .result_title("Changes")
            .prompt_with_backend(&mut backend);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello\nworld");

        for s in backend.output() {
            if let Some(stripped) = s.strip_suffix('\n') {
                assert!(
                    !stripped.contains('\n'),
                    "found embedded newline in output chunk: {s:?}"
                );
            } else {
                assert!(
                    !s.contains('\n'),
                    "found embedded newline in output chunk: {s:?}"
                );
            }
        }
    }

    #[test]
    fn test_input_backspace_at_start() {
        // 在开头按 backspace 应该什么都不做
        let events = [
            vec![Event::Key(KeyEvent::new(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            ))],
            MockBackend::type_string("test"),
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_input_delete_at_end() {
        // 在末尾按 delete 应该什么都不做
        let events = [
            MockBackend::type_string("test"),
            vec![Event::Key(KeyEvent::new(
                KeyCode::Delete,
                KeyModifiers::NONE,
            ))],
            vec![MockBackend::press_enter()],
        ]
        .concat();
        let mut backend = MockBackend::with_events(events);

        let result = InputBuilder::new("Enter text").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }
}
