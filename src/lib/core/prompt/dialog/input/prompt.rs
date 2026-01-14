//! 输入提示主逻辑

use super::builder::InputBuilder;
use super::editor::{CursorLine, InputEditor, ValidationStatus};
use super::input_line::render_input;
use super::prompt_line::render_prompt_line;
use crate::core::prompt::dialog::{common::RawModeGuard, Result};
use crate::core::prompt::style::theme::{get_theme, Theme};
use color_eyre::eyre;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::Write;

use crate::core::prompt::dialog::{PASSWORD_MASK, PROMPT_PREFIX, RESULT_PREFIX};

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

/// 执行提示
pub(super) fn prompt(builder: InputBuilder) -> Result<String> {
    let theme = get_theme();

    // 显示提示信息（单独一行，使用 ? 前缀）
    // 注意：只显示 default，不显示 placeholder
    let (question_mark, prompt_text) = if let Some(ref default) = builder.default {
        if builder.password {
            // 密码模式：显示固定掩码
            (
                PROMPT_PREFIX,
                format!("{}[{}]", builder.message, PASSWORD_MASK),
            )
        } else {
            (PROMPT_PREFIX, format!("{}[{}]", builder.message, default))
        }
    } else {
        (PROMPT_PREFIX, builder.message.clone())
    };
    // 应用主题颜色：? 使用 yellow (warning)，文本使用 prompt
    let styled_question = theme.warning.apply(question_mark, theme.enable_color);
    let styled_text = theme.title.apply(&prompt_text, theme.enable_color);

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
