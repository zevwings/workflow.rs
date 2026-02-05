//! 确认提示模块
//!
//! 提供 Yes/No 确认对话框功能

use crate::backend::{Backend, TerminalBackend};
use crate::dialog::Result;
use crate::dialog::{PROMPT_PREFIX, RESULT_PREFIX};
use crate::error::PromptError;
use crate::style::theme::{get_theme, Theme};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

/// 确认提示构建器
///
/// 用于构建 Yes/No 确认对话框。
///
/// # 默认行为
///
/// 当用户按下 Enter 键时，如果未通过 `.default()` 设置默认值，
/// 则默认返回 `true`（即 Yes）。
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

    /// 设置默认值
    ///
    /// 当用户直接按 Enter 键时返回此值。
    /// 如果未设置，默认为 `true`。
    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }

    /// 执行提示（使用默认终端后端）
    pub fn prompt(self) -> Result<bool> {
        let mut backend = TerminalBackend::default();
        self.prompt_with_backend(&mut backend)
    }

    /// 使用指定后端执行提示（内部使用）
    pub(crate) fn prompt_with_backend<B: Backend>(self, backend: &mut B) -> Result<bool> {
        prompt_with_backend(self, backend)
    }
}

/// 清除并显示结果
fn clear_and_display_result<B: Backend>(
    backend: &mut B,
    builder: &ConfirmBuilder,
    value: bool,
    theme: &Theme,
) -> Result<()> {
    // 计算消息占用的行数
    let line_count = builder.message.chars().filter(|&c| c == '\n').count() + 1;

    // 向上移动并清除每一行
    for _ in 0..line_count {
        backend.move_up(1)?;
        backend.move_to_column(0)?;
        backend.clear_line()?;
    }

    // 显示结果
    let prefix = theme.prefix.apply(RESULT_PREFIX, theme.enable_color);
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

    backend.write(&format!("{}{} {}", prefix, title, result_text))?;
    backend.writeln("")?;
    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}

/// 打印取消消息
fn print_cancelled_message<B: Backend>(backend: &mut B, _builder: &ConfirmBuilder) -> Result<()> {
    let theme = get_theme();

    // 不清除提示行，直接在下方显示取消消息
    let prefix = theme.warning.apply("! ", theme.enable_color);
    let message = theme.hint.apply("Operation cancelled", theme.enable_color);
    backend.writeln(&format!("{}{}", prefix, message))?;
    backend.move_to_column(0)?;
    backend.flush()?;
    Ok(())
}

/// 使用指定后端执行提示
fn prompt_with_backend<B: Backend>(builder: ConfirmBuilder, backend: &mut B) -> Result<bool> {
    // 检查是否在交互式终端中运行
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        // 不是交互式终端，使用默认值或返回错误
        if let Some(default) = builder.default {
            return Ok(default);
        }
        return Err(PromptError::Io(std::io::Error::other(
            "Not running in an interactive terminal. Cannot prompt for user input.",
        )));
    }

    let theme = get_theme();

    // 显示提示信息
    let hint_text = match builder.default {
        Some(true) => "[Y/n]",
        Some(false) => "[y/N]",
        None => "[y/n]",
    };

    let question_prefix = theme.title.apply(PROMPT_PREFIX, theme.enable_color);
    let message_text = theme.title.apply(&builder.message, theme.enable_color);
    let hint_styled = theme.hint.apply(hint_text, theme.enable_color);

    backend.writeln(&format!(
        "{}{} {}",
        question_prefix, message_text, hint_styled
    ))?;
    backend.flush()?;

    // 强制同步 stderr 和 stdout，确保所有输出都已显示
    use std::io::Write;
    let _ = std::io::stderr().flush();
    let _ = std::io::stdout().flush();

    // 进入原始模式
    backend.enable_raw_mode()?;
    backend.hide_cursor()?;

    let result = prompt_loop(backend, &builder, &theme);

    // 恢复终端状态
    backend.show_cursor().ok();
    backend.disable_raw_mode().ok();

    result
}

/// 主事件循环
fn prompt_loop<B: Backend>(
    backend: &mut B,
    builder: &ConfirmBuilder,
    theme: &Theme,
) -> Result<bool> {
    let default_value = builder.default.unwrap_or(true);

    loop {
        match backend.read_event() {
            Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) => match code {
                KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                    if c == 'c' {
                        print_cancelled_message(backend, builder)?;
                        return Err(PromptError::Cancelled);
                    }
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    clear_and_display_result(backend, builder, true, theme)?;
                    return Ok(true);
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    clear_and_display_result(backend, builder, false, theme)?;
                    return Ok(false);
                }
                KeyCode::Enter => {
                    clear_and_display_result(backend, builder, default_value, theme)?;
                    return Ok(default_value);
                }
                KeyCode::Esc => {
                    print_cancelled_message(backend, builder)?;
                    return Err(PromptError::Cancelled);
                }
                _ => {}
            },
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
/// use prompt::confirm;
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
    use crate::backend::MockBackend;

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

    // ========================================================================
    // MockBackend 测试 - 测试实际交互逻辑
    // ========================================================================

    #[test]
    fn test_confirm_press_y() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result = ConfirmBuilder::new("Continue?")
            .default(false)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_press_capital_y() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('Y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result = ConfirmBuilder::new("Continue?")
            .default(false)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_press_n() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result =
            ConfirmBuilder::new("Continue?").default(true).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_confirm_press_capital_n() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('N'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result =
            ConfirmBuilder::new("Continue?").default(true).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_confirm_press_enter_with_default_true() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let result =
            ConfirmBuilder::new("Continue?").default(true).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_press_enter_with_default_false() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let result = ConfirmBuilder::new("Continue?")
            .default(false)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_confirm_press_enter_without_default() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        // 没有设置 default，默认值为 true
        let result = ConfirmBuilder::new("Continue?").prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_press_escape() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let result =
            ConfirmBuilder::new("Continue?").default(true).prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::Cancelled));
    }

    #[test]
    fn test_confirm_press_ctrl_c() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result =
            ConfirmBuilder::new("Continue?").default(true).prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::Cancelled));
    }

    #[test]
    fn test_confirm_ignore_invalid_keys() {
        // 测试无效按键被忽略，最终按 y 确认
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
        ];
        let mut backend = MockBackend::with_events(events);

        let result = ConfirmBuilder::new("Continue?")
            .default(false)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_confirm_with_result_title() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result = ConfirmBuilder::new("Delete all files?")
            .default(false)
            .result_title("Confirmed")
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(result.unwrap());

        // 验证输出中包含结果
        let output = backend.output_string();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_confirm_multiline_message() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        let result = ConfirmBuilder::new("This is a\nmultiline\nmessage")
            .default(true)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_confirm_terminal_modes() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        ))];
        let mut backend = MockBackend::with_events(events);

        // 验证初始状态
        assert!(!backend.is_raw_mode());
        assert!(backend.is_cursor_visible());

        let result = ConfirmBuilder::new("Test?").prompt_with_backend(&mut backend);

        assert!(result.is_ok());

        // 验证终端模式已恢复
        assert!(!backend.is_raw_mode());
        assert!(backend.is_cursor_visible());
    }
}
