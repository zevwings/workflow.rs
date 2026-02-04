//! 输入提示主逻辑

use crate::backend::{Backend, TerminalBackend};
use crate::dialog::input::builder::InputBuilder;
use crate::dialog::input::editor::{CursorLine, InputEditor, ValidationStatus};
use crate::dialog::Result;
use crate::dialog::{PASSWORD_MASK, PROMPT_PREFIX, RESULT_PREFIX};
use crate::error::PromptError;
use crate::style::theme::{get_theme, Theme};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Attribute;
use std::io::Write as _;

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
                "[DEBUG] render_input: 开始渲染输入，输入长度: {}",
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
            let _ = writeln!(file, "[DEBUG] render_input: 移动光标到列 {}", target_column);
        }
    }

    backend.move_to_column(target_column as u16)?;
    backend.show_cursor()?;
    backend.flush()?;

    if debug_enabled {
        if let Ok(mut file) =
            std::fs::OpenOptions::new().create(true).append(true).open(debug_log_path())
        {
            let _ = writeln!(file, "[DEBUG] render_input: 完成渲染");
        }
    }
    Ok(())
}

/// 打印取消消息
fn print_cancelled_message<B: Backend>(backend: &mut B) -> Result<()> {
    let theme = get_theme();
    let prefix = theme.warning.apply("! ", theme.enable_color);
    let message = theme.hint.apply("Operation cancelled", theme.enable_color);
    backend.writeln(&format!("{}{}", prefix, message))?;
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

    // 使用 scopeguard 确保退出时恢复状态
    let result = prompt_loop(backend, &builder, &theme);

    // 恢复终端状态
    backend.disable_bracketed_paste().ok();
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

    ensure_cursor_on_input_line(backend, &mut cursor_line)?;
    render_input(backend, builder, &editor, theme, &mut cursor_line)?;

    loop {
        match backend.read_event() {
            Ok(Event::Paste(text)) => {
                editor.insert_str(&text);
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
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => {
                match code {
                    KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                        if c == 'c' {
                            print_cancelled_message(backend)?;
                            return Err(PromptError::Cancelled);
                        }
                    }
                    KeyCode::Char(c) => {
                        editor.insert(c);
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
                    KeyCode::Backspace => {
                        if editor.backspace() {
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
                    KeyCode::Delete => {
                        if editor.delete() {
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
                    KeyCode::Left => {
                        editor.move_left();
                        render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                    }
                    KeyCode::Right => {
                        editor.move_right();
                        render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                    }
                    KeyCode::Enter => {
                        let input = editor.as_str().to_string();
                        let final_input = if input.trim().is_empty() {
                            builder.default.as_ref().cloned().unwrap_or(input)
                        } else {
                            input
                        };

                        if let Some(ref validator) = builder.validator {
                            match validator.validate(&final_input) {
                                Ok(()) => {
                                    clear_and_display_result(
                                        backend,
                                        builder,
                                        &final_input,
                                        &mut cursor_line,
                                    )?;
                                    return Ok(final_input);
                                }
                                Err(_) => {
                                    validation_status = ValidationStatus::Invalid;
                                    render_prompt_line(
                                        backend,
                                        builder,
                                        theme,
                                        validation_status,
                                        &mut cursor_line,
                                    )?;
                                    render_input(backend, builder, &editor, theme, &mut cursor_line)?;
                                }
                            }
                        } else {
                            clear_and_display_result(
                                backend,
                                builder,
                                &final_input,
                                &mut cursor_line,
                            )?;
                            return Ok(final_input);
                        }
                    }
                    KeyCode::Esc => {
                        print_cancelled_message(backend)?;
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

/// 执行提示（使用默认终端后端）
pub(super) fn prompt(builder: InputBuilder) -> Result<String> {
    let mut backend = TerminalBackend::default();
    prompt_with_backend(builder, &mut backend)
}
