//! 选择提示模块

use crate::base::interactive::common::{OptionListRenderer, OptionRenderer};
use crate::base::interactive::error::Result;
use crate::base::interactive::style::get_theme;
use crate::base::interactive::terminal::Terminal;
use color_eyre::eyre;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

/// Select 选项渲染器
struct SelectOptionRenderer;

impl OptionRenderer for SelectOptionRenderer {
    fn render_option(
        &self,
        _index: usize,
        option_text: &str,
        is_current: bool,
        theme: &crate::base::interactive::style::Theme,
    ) -> String {
        if is_current {
            // 当前选中的选项：使用 "> " 前缀并应用 answer 样式（高亮）
            let prefix = theme.success.apply("> ", theme.enable_color);
            let option_styled = theme.answer.apply(option_text, theme.enable_color);
            format!("{}{}", prefix, option_styled)
        } else {
            // 其他选项：使用 "  " 前缀
            format!("  {}", option_text)
        }
    }
}

/// 选择提示构建器
pub struct SelectBuilder<T> {
    message: String,
    options: Vec<T>,
    default: Option<usize>,
}

impl<T> SelectBuilder<T>
where
    T: std::fmt::Display + Clone,
{
    pub fn new(message: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            message: message.into(),
            options,
            default: None,
        }
    }

    pub fn default(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    /// 执行提示
    pub fn prompt<TR: Terminal>(self, terminal: &mut TR) -> Result<T> {
        if self.options.is_empty() {
            return Err(eyre::eyre!("选项列表不能为空"));
        }

        let theme = get_theme();

        // 验证并调整默认索引
        let mut current_index = self.default.filter(|&idx| idx < self.options.len()).unwrap_or(0);

        // 显示提示信息（单独一行，使用 ? 前缀）
        let question_prefix = theme.warning.apply("? ", theme.enable_color);
        let message_text = theme.prompt.apply(&self.message, theme.enable_color);
        terminal.write_flush(&format!("{}{}\n", question_prefix, message_text))?;

        // 进入原始模式
        let _guard = terminal.enable_raw_mode()?;

        // 跟踪已渲染的行数（用于正确清除）
        let mut rendered_lines = 0;
        let renderer = SelectOptionRenderer;

        // 渲染初始状态
        rendered_lines = OptionListRenderer::render_options(
            &self.options,
            current_index,
            rendered_lines,
            &theme,
            &renderer,
            "使用 ↑/↓ 导航，回车确认",
        )?;

        loop {
            // 读取键盘事件
            match event::read() {
                Ok(Event::Key(KeyEvent {
                    code, modifiers, ..
                })) => {
                    match code {
                        KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                            if c == 'c' {
                                return Err(eyre::eyre!("User cancelled"));
                            }
                        }
                        KeyCode::Up => {
                            if current_index > 0 {
                                current_index -= 1;
                                rendered_lines = OptionListRenderer::render_options(
                                    &self.options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    "使用 ↑/↓ 导航，回车确认",
                                )?;
                            }
                        }
                        KeyCode::Down => {
                            if current_index < self.options.len() - 1 {
                                current_index += 1;
                                rendered_lines = OptionListRenderer::render_options(
                                    &self.options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    "使用 ↑/↓ 导航，回车确认",
                                )?;
                            }
                        }
                        KeyCode::Enter => {
                            // 清除选项列表和提示行，显示结果
                            let selected = self.options[current_index].clone();
                            let result_text = selected.to_string();
                            OptionListRenderer::clear_and_display_result(
                                self.options.len(),
                                &self.message,
                                &result_text,
                                &theme,
                            )?;
                            return Ok(selected);
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
}

/// 便捷函数
pub fn select<T: std::fmt::Display + Clone>(
    message: impl Into<String>,
    options: Vec<T>,
) -> SelectBuilder<T> {
    SelectBuilder::new(message, options)
}
