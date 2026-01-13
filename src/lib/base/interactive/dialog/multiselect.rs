//! 多选提示模块

use crate::base::interactive::common::{OptionListRenderer, OptionRenderer};
use crate::base::interactive::error::Result;
use crate::base::interactive::style::get_theme;
use crate::base::interactive::terminal::Terminal;
use color_eyre::eyre;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;

/// MultiSelect 选项渲染器
struct MultiSelectOptionRenderer<'a> {
    selected: &'a HashSet<usize>,
}

impl<'a> OptionRenderer for MultiSelectOptionRenderer<'a> {
    fn render_option(
        &self,
        index: usize,
        option_text: &str,
        is_current: bool,
        theme: &crate::base::interactive::style::Theme,
    ) -> String {
        let is_selected = self.selected.contains(&index);

        // 构建前缀："> " 或 "  "
        let prefix = if is_current {
            theme.success.apply("> ", theme.enable_color)
        } else {
            "  ".to_string()
        };

        // 构建标记："[x]" 或 "[ ]"
        let marker = if is_selected { "[x]" } else { "[ ]" };

        // 如果当前选中，应用高亮样式
        let option_styled = if is_current {
            theme.answer.apply(option_text, theme.enable_color)
        } else {
            option_text.to_string()
        };

        format!("{}{} {}", prefix, marker, option_styled)
    }
}

/// 多选提示构建器
pub struct MultiSelectBuilder<T> {
    message: String,
    options: Vec<T>,
    default: Vec<usize>,
}

impl<T> MultiSelectBuilder<T>
where
    T: std::fmt::Display + Clone,
{
    pub fn new(message: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            message: message.into(),
            options,
            default: Vec::new(),
        }
    }

    pub fn default(mut self, indices: Vec<usize>) -> Self {
        self.default = indices;
        self
    }

    /// 执行提示
    pub fn prompt<TR: Terminal>(self, terminal: &mut TR) -> Result<Vec<T>> {
        if self.options.is_empty() {
            return Err(eyre::eyre!("选项列表不能为空"));
        }

        let theme = get_theme();

        // 验证并清理默认选中项
        let mut selected: HashSet<usize> =
            self.default.iter().copied().filter(|&idx| idx < self.options.len()).collect();

        // 确定初始光标位置
        let mut current_index = if !selected.is_empty() {
            *selected.iter().next().unwrap()
        } else {
            0
        };

        // 显示提示信息（单独一行，使用 ? 前缀）
        let question_prefix = theme.warning.apply("? ", theme.enable_color);
        let message_text = theme.prompt.apply(&self.message, theme.enable_color);
        terminal.write_flush(&format!("{}{}\n", question_prefix, message_text))?;

        // 进入原始模式
        let _guard = terminal.enable_raw_mode()?;

        // 跟踪已渲染的行数（用于正确清除）
        let mut rendered_lines = 0;

        // 渲染初始状态
        let renderer = MultiSelectOptionRenderer {
            selected: &selected,
        };
        rendered_lines = OptionListRenderer::render_options(
            &self.options,
            current_index,
            rendered_lines,
            &theme,
            &renderer,
            "使用 ↑/↓ 导航，空格键切换选择，回车确认",
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
                                let renderer = MultiSelectOptionRenderer {
                                    selected: &selected,
                                };
                                rendered_lines = OptionListRenderer::render_options(
                                    &self.options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    "使用 ↑/↓ 导航，空格键切换选择，回车确认",
                                )?;
                            }
                        }
                        KeyCode::Down => {
                            if current_index < self.options.len() - 1 {
                                current_index += 1;
                                let renderer = MultiSelectOptionRenderer {
                                    selected: &selected,
                                };
                                rendered_lines = OptionListRenderer::render_options(
                                    &self.options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    "使用 ↑/↓ 导航，空格键切换选择，回车确认",
                                )?;
                            }
                        }
                        KeyCode::Char(' ') => {
                            // 空格键切换选择状态
                            if selected.contains(&current_index) {
                                selected.remove(&current_index);
                            } else {
                                selected.insert(current_index);
                            }
                            let renderer = MultiSelectOptionRenderer {
                                selected: &selected,
                            };
                            rendered_lines = OptionListRenderer::render_options(
                                &self.options,
                                current_index,
                                rendered_lines,
                                &theme,
                                &renderer,
                                "使用 ↑/↓ 导航，空格键切换选择，回车确认",
                            )?;
                        }
                        KeyCode::Enter => {
                            // 清除选项列表和提示行，显示结果
                            let mut selected_indices: Vec<usize> =
                                selected.iter().copied().collect();
                            selected_indices.sort();
                            let selected_items: Vec<T> = selected_indices
                                .iter()
                                .map(|&idx| self.options[idx].clone())
                                .collect();

                            // 格式化结果
                            let result_text = if selected_items.is_empty() {
                                "(未选择)".to_string()
                            } else {
                                selected_items
                                    .iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };

                            OptionListRenderer::clear_and_display_result(
                                self.options.len(),
                                &self.message,
                                &result_text,
                                &theme,
                            )?;
                            return Ok(selected_items);
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
pub fn multiselect<T: std::fmt::Display + Clone>(
    message: impl Into<String>,
    options: Vec<T>,
) -> MultiSelectBuilder<T> {
    MultiSelectBuilder::new(message, options)
}
