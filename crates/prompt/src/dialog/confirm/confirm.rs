//! 确认提示模块
//!
//! 提供 Yes/No 确认对话框功能

use crate::dialog::{common::RawModeGuard, Result};
use crate::dialog::{PROMPT_PREFIX, RESULT_PREFIX};
use crate::error::PromptError;
use crate::style::theme::{get_theme, Theme};
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::Write;

/// 确认提示构建器
pub struct ConfirmBuilder {
    pub(crate) message: String,
    pub(crate) default: Option<bool>,
    pub(crate) result_title: Option<String>,
}

impl ConfirmBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            result_title: None,
        }
    }

    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }

    /// 执行提示
    pub fn prompt(self) -> Result<bool> {
        prompt(self)
    }
}

/// 清除并显示结果
fn clear_and_display_result(builder: &ConfirmBuilder, value: bool, theme: &Theme) -> Result<()> {
    let mut stdout = std::io::stdout();

    // 计算消息占用的行数（包含换行符的数量 + 1）
    let line_count = builder.message.chars().filter(|&c| c == '\n').count() + 1;

    // 当前光标在提示行的下一行（因为 writeln! 输出了换行符）
    // 需要向上移动 line_count 行回到提示行的第一行
    for _ in 0..line_count {
        execute!(stdout, cursor::MoveUp(1))?;
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::UntilNewLine))?;
    }

    // 显示格式化的结果："> [title] yes" 或 "> [title] no"
    let prefix = theme.prefix.apply(RESULT_PREFIX, theme.enable_color);
    // 使用 result_title（如果存在），否则使用 message 的第一行（避免多行消息导致输出错乱）
    let title_text = builder
        .result_title
        .as_deref()
        .unwrap_or_else(|| builder.message.lines().next().unwrap_or(&builder.message));
    let title = theme.title.apply(title_text, theme.enable_color);
    let result_text = if value {
        theme.answer.apply("yes", theme.enable_color)
    } else {
        theme.answer.apply("no", theme.enable_color)
    };

    write!(stdout, "{}{} {}", prefix, title, result_text)?;
    writeln!(stdout)?;
    // 确保光标在新行的开头，以便后续消息输出正确对齐
    execute!(stdout, cursor::MoveToColumn(0))?;

    // 显示光标
    execute!(stdout, cursor::Show)?;
    stdout.flush()?;
    Ok(())
}

/// 执行提示
fn prompt(builder: ConfirmBuilder) -> Result<bool> {
    let theme = get_theme();

    // 显示提示信息（单独一行，使用 ? 前缀）
    // 格式：? 是否继续操作？ [Y/n]
    let hint_text = match builder.default {
        Some(true) => "[Y/n]",
        Some(false) => "[y/N]",
        None => "[y/n]",
    };

    // 应用主题颜色：? 和消息使用 prompt 样式，hint 使用 hint 样式
    let question_prefix = theme.title.apply(PROMPT_PREFIX, theme.enable_color);
    let message_text = theme.title.apply(&builder.message, theme.enable_color);
    let hint_styled = theme.hint.apply(hint_text, theme.enable_color);

    let mut stdout = std::io::stdout();
    writeln!(
        stdout,
        "{}{} {}",
        question_prefix, message_text, hint_styled
    )?;
    stdout.flush()?;

    // 进入原始模式
    let _guard = RawModeGuard::new()?;

    // 隐藏光标
    {
        let mut stdout = std::io::stdout();
        execute!(stdout, cursor::Hide)?;
        stdout.flush()?;
    }

    let default_value = builder.default.unwrap_or(true);

    loop {
        // 读取键盘事件
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => {
                match code {
                    KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                        if c == 'c' {
                            // Ctrl+C：输出统一的取消提示，然后返回取消错误
                            if let Err(e) = crate::dialog::common::print_cancelled_message() {
                                return Err(PromptError::Io(e));
                            }
                            return Err(PromptError::Cancelled);
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // 清除提示行，显示结果
                        clear_and_display_result(&builder, true, &theme)?;
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        // 清除提示行，显示结果
                        clear_and_display_result(&builder, false, &theme)?;
                        return Ok(false);
                    }
                    KeyCode::Enter => {
                        // 清除提示行，显示结果（使用默认值）
                        clear_and_display_result(&builder, default_value, &theme)?;
                        return Ok(default_value);
                    }
                    KeyCode::Esc => {
                        // Esc：输出统一的取消提示，然后返回取消错误
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

// ============================================================================
// 宏定义
// ============================================================================

/// 确认提示宏
///
/// 提供格式化字符串的便捷方式，智能判断是否需要格式化：
/// - 简单字符串字面量：直接传递，不调用 `format!()`
/// - 格式化字符串：使用 `format!()` 进行格式化
/// - 变量或表达式：直接传递，不调用 `format!()`
///
/// # Examples
///
/// ```rust,no_run
/// use toolkit::confirm;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 简单字符串（直接传递，不格式化）
/// let result1 = confirm!("Continue?")
///     .default(true)
///     .prompt()?;
///
/// // 格式化字符串（使用 format!）
/// let branch = "feature-123";
/// let result2 = confirm!("Create PR for branch '{}'?", branch)
///     .default(true)
///     .prompt()?;
///
/// // 变量（直接传递，不格式化）
/// let msg = "Are you sure?";
/// let result3 = confirm!(msg)
///     .default(false)
///     .prompt()?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! confirm {
    // 格式化字符串：confirm!("Message {}", var) 或 confirm!("Message {}", var1, var2)
    ($fmt:literal, $($arg:expr),+ $(,)?) => {
        $crate::ConfirmBuilder::new(format!($fmt, $($arg),+))
    };
    // 简单字符串字面量：confirm!("Message") - 直接传递，不格式化
    ($msg:literal) => {
        $crate::ConfirmBuilder::new($msg)
    };
    // 变量或其他表达式：confirm!(var) - 直接传递，不格式化
    ($expr:expr) => {
        $crate::ConfirmBuilder::new($expr)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_builder_new() {
        let builder = ConfirmBuilder::new("Continue?");
        assert_eq!(builder.message, "Continue?");
        assert!(builder.default.is_none());
        assert!(builder.result_title.is_none());
    }

    #[test]
    fn test_confirm_builder_default() {
        let builder = ConfirmBuilder::new("Continue?").default(true);
        assert_eq!(builder.default, Some(true));

        let builder = ConfirmBuilder::new("Continue?").default(false);
        assert_eq!(builder.default, Some(false));
    }

    #[test]
    fn test_confirm_builder_result_title() {
        let builder = ConfirmBuilder::new("Continue?").result_title("Confirmation");
        assert_eq!(builder.result_title, Some("Confirmation".to_string()));
    }

    #[test]
    fn test_confirm_builder_chain() {
        let builder = ConfirmBuilder::new("Delete file?").default(false).result_title("Delete");

        assert_eq!(builder.message, "Delete file?");
        assert_eq!(builder.default, Some(false));
        assert_eq!(builder.result_title, Some("Delete".to_string()));
    }
}
