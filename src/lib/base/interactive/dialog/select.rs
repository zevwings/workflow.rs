//! 选择提示模块

use crate::base::interactive::dialog::error::Result;
use crate::base::interactive::dialog::filter::FuzzyFilter;
use crate::base::interactive::dialog::raw_mode::RawModeGuard;
use crate::base::interactive::dialog::renderer::{OptionListRenderer, OptionRenderer};
use crate::base::interactive::style::get_theme;
use color_eyre::eyre;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::Write;

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
    result_title: Option<String>,
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
            result_title: None,
        }
    }

    pub fn default(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }

    /// 执行提示
    pub fn prompt(self) -> Result<T> {
        if self.options.is_empty() {
            return Err(eyre::eyre!("选项列表不能为空"));
        }

        let theme = get_theme();
        let filter = FuzzyFilter::new();

        // 验证并调整默认索引
        let mut current_index = self.default.filter(|&idx| idx < self.options.len()).unwrap_or(0);

        // 搜索查询
        let mut search_query = String::new();

        // 显示提示信息（单独一行，使用 ? 前缀）
        let question_prefix = theme.warning.apply("? ", theme.enable_color);
        let message_text = theme.title.apply(&self.message, theme.enable_color);

        let mut stdout = std::io::stdout();
        writeln!(stdout, "{}{}", question_prefix, message_text)?;
        stdout.flush()?;

        // 进入原始模式
        let _guard = RawModeGuard::new()?;

        // 跟踪已渲染的行数（用于正确清除）
        let mut rendered_lines = 0;
        let renderer = SelectOptionRenderer;

        // 过滤选项的函数（使用 FuzzyFilter）
        let filter_options =
            |query: &str| -> (Vec<usize>, Vec<&T>) { filter.filter(&self.options, query) };

        // 初始过滤
        let (mut filtered_indices, mut filtered_options) = filter_options(&search_query);

        // 调整当前索引（确保在过滤后的列表中有效）
        if !filtered_options.is_empty() {
            current_index = current_index.min(filtered_options.len() - 1);
        } else {
            current_index = 0;
        }

        // 渲染初始状态
        let hint_text = if search_query.is_empty() {
            "使用 ↑/↓ 导航，输入搜索，回车确认"
        } else {
            "使用 ↑/↓ 导航，Esc 清除搜索，回车确认"
        };
        rendered_lines = OptionListRenderer::render_options_with_search(
            &filtered_options,
            current_index,
            rendered_lines,
            &theme,
            &renderer,
            hint_text,
            if search_query.is_empty() {
                None
            } else {
                Some(&search_query)
            },
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
                        KeyCode::Char(c) => {
                            // 输入字符，添加到搜索查询
                            search_query.push(c);
                            let (new_indices, new_filtered) = filter_options(&search_query);

                            // 更新过滤后的选项和索引映射
                            filtered_indices = new_indices;
                            filtered_options = new_filtered;

                            // 重置当前索引
                            current_index = 0;

                            let hint_text = if search_query.is_empty() {
                                "使用 ↑/↓ 导航，输入搜索，回车确认"
                            } else {
                                "使用 ↑/↓ 导航，Esc 清除搜索，回车确认"
                            };
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                &filtered_options,
                                current_index,
                                rendered_lines,
                                &theme,
                                &renderer,
                                hint_text,
                                Some(&search_query),
                            )?;
                        }
                        KeyCode::Backspace => {
                            // 删除搜索查询的最后一个字符
                            if !search_query.is_empty() {
                                search_query.pop();
                                let (new_indices, new_filtered) = filter_options(&search_query);

                                filtered_indices = new_indices;
                                filtered_options = new_filtered;

                                // 重置当前索引
                                if !filtered_options.is_empty() {
                                    current_index = current_index.min(filtered_options.len() - 1);
                                } else {
                                    current_index = 0;
                                }

                                let hint_text = if search_query.is_empty() {
                                    "使用 ↑/↓ 导航，输入搜索，回车确认"
                                } else {
                                    "使用 ↑/↓ 导航，Esc 清除搜索，回车确认"
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    hint_text,
                                    if search_query.is_empty() {
                                        None
                                    } else {
                                        Some(&search_query)
                                    },
                                )?;
                            }
                        }
                        KeyCode::Up => {
                            if !filtered_options.is_empty() && current_index > 0 {
                                current_index -= 1;
                                let hint_text = if search_query.is_empty() {
                                    "使用 ↑/↓ 导航，输入搜索，回车确认"
                                } else {
                                    "使用 ↑/↓ 导航，Esc 清除搜索，回车确认"
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    hint_text,
                                    if search_query.is_empty() {
                                        None
                                    } else {
                                        Some(&search_query)
                                    },
                                )?;
                            }
                        }
                        KeyCode::Down => {
                            if !filtered_options.is_empty()
                                && current_index < filtered_options.len() - 1
                            {
                                current_index += 1;
                                let hint_text = if search_query.is_empty() {
                                    "使用 ↑/↓ 导航，输入搜索，回车确认"
                                } else {
                                    "使用 ↑/↓ 导航，Esc 清除搜索，回车确认"
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    hint_text,
                                    if search_query.is_empty() {
                                        None
                                    } else {
                                        Some(&search_query)
                                    },
                                )?;
                            }
                        }
                        KeyCode::Enter => {
                            if filtered_options.is_empty() {
                                continue;
                            }
                            // 获取原始索引
                            let original_index = filtered_indices[current_index];
                            // 清除选项列表和提示行，显示结果
                            let selected = self.options[original_index].clone();
                            let result_text = selected.to_string();
                            let has_search = !search_query.is_empty();
                            // 使用 result_title（如果存在），否则使用 message
                            let title_text = self.result_title.as_ref().unwrap_or(&self.message);
                            OptionListRenderer::clear_and_display_result_with_search(
                                filtered_options.len(),
                                title_text,
                                &result_text,
                                &theme,
                                has_search,
                            )?;
                            return Ok(selected);
                        }
                        KeyCode::Esc => {
                            if !search_query.is_empty() {
                                // 清除搜索查询
                                search_query.clear();
                                let (new_indices, new_filtered) = filter_options(&search_query);

                                filtered_indices = new_indices;
                                filtered_options = new_filtered;

                                // 重置当前索引
                                current_index = 0;

                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    &theme,
                                    &renderer,
                                    "使用 ↑/↓ 导航，输入搜索，回车确认",
                                    None,
                                )?;
                            } else {
                                // 没有搜索查询，取消操作
                                return Err(eyre::eyre!("User cancelled"));
                            }
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

/// 选择提示宏
///
/// 提供格式化字符串的便捷方式，智能判断是否需要格式化：
/// - 简单字符串字面量：直接传递，不调用 `format!()`
/// - 格式化字符串：使用 `format!()` 进行格式化
/// - 变量或表达式：直接传递，不调用 `format!()`
///
/// # Examples
///
/// ```rust,no_run
/// use workflow::select;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let options = vec!["Option 1", "Option 2", "Option 3"];
///
/// // 简单字符串（直接传递，不格式化）
/// let result1 = select!("Choose an option", options.clone())
///     .default(0)
///     .prompt()?;
///
/// // 格式化字符串（使用 format!）
/// let result2 = select!("Choose option for '{}':", "test", options.clone())
///     .default(0)
///     .prompt()?;
///
/// // 变量（直接传递，不格式化）
/// let msg = "Choose:";
/// let result3 = select!(msg, options)
///     .default(0)
///     .prompt()?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! select {
    // 格式化字符串 + 一个格式化参数 + 选项列表
    ($fmt:literal, $arg:expr, $options:expr) => {
        $crate::base::interactive::SelectBuilder::new(format!($fmt, $arg), $options)
    };
    // 格式化字符串 + 选项列表（无格式化参数）
    ($fmt:literal, $options:expr) => {
        $crate::base::interactive::SelectBuilder::new(format!($fmt), $options)
    };
    // 简单字符串字面量 + 选项列表
    ($msg:literal, $options:expr) => {
        $crate::base::interactive::SelectBuilder::new($msg, $options)
    };
    // 变量或其他表达式 + 选项列表
    ($msg:expr, $options:expr) => {
        $crate::base::interactive::SelectBuilder::new($msg, $options)
    };
}
