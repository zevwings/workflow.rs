//! 确认提示模块

use crate::base::interactive::dialog::error::Result;
use crate::base::interactive::dialog::raw_mode::RawModeGuard;
use crate::base::interactive::style::get_theme;
use color_eyre::eyre;
use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use std::io::Write;

/// 确认提示构建器
pub struct ConfirmBuilder {
    message: String,
    default: Option<bool>,
}

impl ConfirmBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
        }
    }

    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    /// 执行提示
    pub fn prompt(self) -> Result<bool> {
        let theme = get_theme();

        // 显示提示信息（单独一行，使用 ? 前缀）
        // 格式：? 是否继续操作？ [Y/n]
        let hint_text = match self.default {
            Some(true) => "[Y/n]",
            Some(false) => "[y/N]",
            None => "[y/n]",
        };

        // 应用主题颜色：? 和消息使用 prompt 样式，hint 使用 hint 样式
        let question_prefix = theme.prompt.apply("? ", theme.enable_color);
        let message_text = theme.prompt.apply(&self.message, theme.enable_color);
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

        let default_value = self.default.unwrap_or(true);

        loop {
            // 读取键盘事件
            match event::read() {
                Ok(Event::Key(KeyEvent { code, .. })) => {
                    match code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // 清除提示行，显示结果
                            self.clear_and_display_result(true, &theme)?;
                            return Ok(true);
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            // 清除提示行，显示结果
                            self.clear_and_display_result(false, &theme)?;
                            return Ok(false);
                        }
                        KeyCode::Enter => {
                            // 清除提示行，显示结果（使用默认值）
                            self.clear_and_display_result(default_value, &theme)?;
                            return Ok(default_value);
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

    fn clear_and_display_result(
        &self,
        value: bool,
        theme: &crate::base::interactive::style::Theme,
    ) -> Result<()> {
        use crossterm::terminal::ClearType;

        let mut stdout = std::io::stdout();

        // 当前光标在提示行的下一行（因为 write_flush 输出了换行符）
        // 先向上移动一行回到提示行
        execute!(stdout, cursor::MoveUp(1))?;

        // 清除提示行
        execute!(stdout, cursor::MoveToColumn(0))?;
        execute!(stdout, crossterm::terminal::Clear(ClearType::UntilNewLine))?;

        // 显示格式化的结果："> [title] yes" 或 "> [title] no"
        let prefix = theme.prefix.apply("> ", theme.enable_color);
        let title = theme.prompt.apply(&self.message, theme.enable_color);
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
}

/// 便捷函数
pub fn confirm(message: impl Into<String>) -> ConfirmBuilder {
    ConfirmBuilder::new(message)
}
