//! 多选提示模块

use crate::dialog::selection::filter::FuzzyFilter;
use crate::dialog::selection::renderer::{OptionListRenderer, OptionRenderer, RenderOptionsParams};
use crate::dialog::{
    common::RawModeGuard, Result, PROMPT_PREFIX, SELECTED_PREFIX, UNSELECTED_PREFIX,
};
use crate::error::PromptError;
use crate::style::theme::get_theme;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;
use std::io::Write;

/// MultiSelect 选项渲染器
struct MultiSelectOptionRenderer<'a> {
    selected: &'a HashSet<usize>,
    // 原始索引到过滤后索引的映射（用于在过滤后的列表中正确显示选中状态）
    original_to_filtered: &'a [usize],
}

impl<'a> OptionRenderer for MultiSelectOptionRenderer<'a> {
    fn render_option(
        &self,
        index: usize,
        option_text: &str,
        is_current: bool,
        theme: &crate::style::theme::Theme,
    ) -> String {
        // index 是过滤后的索引，需要转换为原始索引
        let original_index = self.original_to_filtered[index];
        let is_selected = self.selected.contains(&original_index);

        // 构建前缀："> " 或 "  "
        let prefix = if is_current {
            theme.success.apply(SELECTED_PREFIX, theme.enable_color)
        } else {
            UNSELECTED_PREFIX.to_string()
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
    result_title: Option<String>,
    page_size: Option<usize>,
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
            result_title: None,
            page_size: None,
        }
    }

    pub fn default(mut self, indices: Vec<usize>) -> Self {
        self.default = indices;
        self
    }

    /// 设置输入完成后显示的 title
    pub fn result_title(mut self, title: impl Into<String>) -> Self {
        self.result_title = Some(title.into());
        self
    }

    /// 设置分页大小（每页显示的选项数量）
    ///
    /// 默认值为 10。当选项数量超过分页大小时，会启用滚动窗口。
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = Some(size);
        self
    }

    /// 执行提示
    pub fn prompt(self) -> Result<Vec<T>> {
        if self.options.is_empty() {
            return Err(PromptError::InvalidInput("选项列表不能为空".to_string()));
        }

        let theme = get_theme();
        let filter = FuzzyFilter::new();

        // 验证并清理默认选中项（使用原始索引）
        let mut selected: HashSet<usize> = self
            .default
            .iter()
            .copied()
            .filter(|&idx| idx < self.options.len())
            .collect();

        // 搜索查询
        let mut search_query = String::new();

        // 显示提示信息（单独一行，使用 ? 前缀）
        // 应用主题颜色：? 使用 yellow (warning)，message 使用 title，[value] 使用 hint
        let question_prefix = theme.warning.apply(PROMPT_PREFIX, theme.enable_color);

        // 分离 message 和 [current: ...] 部分，提取 value 显示为 [value]
        let styled_text = if let Some(current_start) = self.message.find("[current:") {
            let (base_message, current_part) = self.message.split_at(current_start);
            // 从 [current: value] 中提取 value，显示为 [value]
            // 格式："[current: value]"，提取 "value" 部分
            let default_value = if let Some(colon_pos) = current_part.find(": ") {
                let value_start = colon_pos + 2; // ": " 的长度是 2
                if let Some(bracket_end) = current_part[value_start..].find(']') {
                    format!(
                        "[{}]",
                        &current_part[value_start..value_start + bracket_end]
                    )
                } else {
                    format!("[{}]", &current_part[value_start..].trim_end_matches(']'))
                }
            } else {
                // 如果格式不对，保持原样
                current_part.to_string()
            };
            let styled_message = theme
                .title
                .apply(base_message.trim_end(), theme.enable_color);
            let styled_default = theme.hint.apply(&default_value, theme.enable_color);
            format!("{} {}", styled_message, styled_default)
        } else {
            theme.title.apply(&self.message, theme.enable_color)
        };

        let mut stdout = std::io::stdout();
        writeln!(stdout, "{}{}", question_prefix, styled_text)?;
        stdout.flush()?;

        // 进入原始模式
        let _guard = RawModeGuard::new()?;

        // 跟踪已渲染的行数（用于正确清除）
        let mut rendered_lines = 0;

        // 过滤选项的函数（使用 FuzzyFilter）
        let filter_options =
            |query: &str| -> (Vec<usize>, Vec<&T>) { filter.filter(&self.options, query) };

        // 初始过滤
        let (mut filtered_indices, mut filtered_options) = filter_options(&search_query);

        // 确定初始光标位置（使用过滤后的索引）
        let mut current_index = if !filtered_options.is_empty() {
            if !selected.is_empty() {
                // 尝试找到第一个已选中项在过滤后列表中的位置
                filtered_indices
                    .iter()
                    .position(|&idx| selected.contains(&idx))
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // 渲染初始状态
        let renderer = MultiSelectOptionRenderer {
            selected: &selected,
            original_to_filtered: &filtered_indices,
        };
        let hint_text = if search_query.is_empty() {
            "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
        } else {
            "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
        };
        rendered_lines = OptionListRenderer::render_options_with_search(&RenderOptionsParams {
            options: &filtered_options,
            current_index,
            rendered_lines,
            theme: &theme,
            renderer: &renderer,
            hint_text,
            search_query: if search_query.is_empty() {
                None
            } else {
                Some(&search_query)
            },
            page_size: self.page_size,
        })?;

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
                        KeyCode::Char(' ') => {
                            if filtered_options.is_empty() {
                                continue;
                            }
                            // 空格键切换选择状态（使用原始索引）
                            let original_index = filtered_indices[current_index];
                            if selected.contains(&original_index) {
                                selected.remove(&original_index);
                            } else {
                                selected.insert(original_index);
                            }
                            let hint_text = if search_query.is_empty() {
                                "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
                            } else {
                                "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
                            };
                            let renderer = MultiSelectOptionRenderer {
                                selected: &selected,
                                original_to_filtered: &filtered_indices,
                            };
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    theme: &theme,
                                    renderer: &renderer,
                                    hint_text,
                                    search_query: if search_query.is_empty() {
                                        None
                                    } else {
                                        Some(&search_query)
                                    },
                                    page_size: self.page_size,
                                },
                            )?;
                        }
                        KeyCode::Char(c) => {
                            // 输入字符，添加到搜索查询
                            search_query.push(c);
                            let (new_indices, new_filtered) = filter_options(&search_query);

                            filtered_indices = new_indices;
                            filtered_options = new_filtered;

                            // 重置当前索引
                            current_index = 0;

                            let hint_text = if search_query.is_empty() {
                                "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
                            } else {
                                "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
                            };
                            let renderer = MultiSelectOptionRenderer {
                                selected: &selected,
                                original_to_filtered: &filtered_indices,
                            };
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    theme: &theme,
                                    renderer: &renderer,
                                    hint_text,
                                    search_query: Some(&search_query),
                                    page_size: self.page_size,
                                },
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
                                    "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
                                } else {
                                    "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
                                };
                                let renderer = MultiSelectOptionRenderer {
                                    selected: &selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &RenderOptionsParams {
                                        options: &filtered_options,
                                        current_index,
                                        rendered_lines,
                                        theme: &theme,
                                        renderer: &renderer,
                                        hint_text,
                                        search_query: if search_query.is_empty() {
                                            None
                                        } else {
                                            Some(&search_query)
                                        },
                                        page_size: self.page_size,
                                    },
                                )?;
                            }
                        }
                        KeyCode::Up => {
                            if !filtered_options.is_empty() && current_index > 0 {
                                current_index -= 1;
                                let hint_text = if search_query.is_empty() {
                                    "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
                                } else {
                                    "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
                                };
                                let renderer = MultiSelectOptionRenderer {
                                    selected: &selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &RenderOptionsParams {
                                        options: &filtered_options,
                                        current_index,
                                        rendered_lines,
                                        theme: &theme,
                                        renderer: &renderer,
                                        hint_text,
                                        search_query: if search_query.is_empty() {
                                            None
                                        } else {
                                            Some(&search_query)
                                        },
                                        page_size: self.page_size,
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
                                    "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
                                } else {
                                    "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
                                };
                                let renderer = MultiSelectOptionRenderer {
                                    selected: &selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &RenderOptionsParams {
                                        options: &filtered_options,
                                        current_index,
                                        rendered_lines,
                                        theme: &theme,
                                        renderer: &renderer,
                                        hint_text,
                                        search_query: if search_query.is_empty() {
                                            None
                                        } else {
                                            Some(&search_query)
                                        },
                                        page_size: self.page_size,
                                    },
                                )?;
                            }
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

                            // 使用 result_title（如果存在），否则使用 message
                            let title_text = self.result_title.as_ref().unwrap_or(&self.message);
                            crate::dialog::selection::renderer::clear_and_display_result_with_search(
                                rendered_lines,
                                title_text,
                                &result_text,
                                &theme,
                            )?;
                            return Ok(selected_items);
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

                                let renderer = MultiSelectOptionRenderer {
                                    selected: &selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    &RenderOptionsParams {
                                        options: &filtered_options,
                                        current_index,
                                        rendered_lines,
                                        theme: &theme,
                                        renderer: &renderer,
                                        hint_text:
                                            "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认",
                                        search_query: None,
                                        page_size: self.page_size,
                                    },
                                )?;
                            } else {
                                // 没有搜索查询，取消操作：输出统一提示
                                if let Err(e) = crate::dialog::common::print_cancelled_message() {
                                    return Err(PromptError::Io(e));
                                }
                                return Err(PromptError::Cancelled);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(_) => continue,
                Err(e) => return Err(PromptError::Io(e)),
            }
        }
    }
}

// 宏定义（通过 #[macro_export] 在 crate 根级别导出）
/// 多选提示宏
///
/// 提供格式化字符串的便捷方式，智能判断是否需要格式化：
/// - 简单字符串字面量：直接传递，不调用 `format!()`
/// - 格式化字符串：使用 `format!()` 进行格式化
/// - 变量或表达式：直接传递，不调用 `format!()`
///
/// # Examples
///
/// ```rust,no_run
/// use toolkit::multiselect;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let options = vec!["Option 1", "Option 2", "Option 3"];
///
/// // 简单字符串（直接传递，不格式化）
/// let result1 = multiselect!("Select options", options.clone())
///     .default(vec![0])
///     .prompt()?;
///
/// // 格式化字符串（使用 format!）
/// let result2 = multiselect!("Select options for '{}':", "test", options.clone())
///     .default(vec![0])
///     .prompt()?;
///
/// // 变量（直接传递，不格式化）
/// let msg = "Select:";
/// let result3 = multiselect!(msg, options)
///     .default(vec![0])
///     .prompt()?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! multiselect {
    // 格式化字符串 + 单个参数 + 选项（用逗号分隔）- 必须放在多参数模式之前以避免歧义
    ($fmt:literal, $arg:expr, $options:expr) => {
        $crate::MultiSelectBuilder::new(format!($fmt, $arg), $options)
    };
    // 格式化字符串 + 多个参数（2个或更多）+ 选项（用逗号分隔）
    ($fmt:literal, $arg1:expr, $arg2:expr, $($arg:expr),+ $(,)?, $options:expr) => {
        $crate::MultiSelectBuilder::new(format!($fmt, $arg1, $arg2, $($arg),+), $options)
    };
    // 简单字符串字面量 + 选项列表
    ($msg:literal, $options:expr) => {
        $crate::MultiSelectBuilder::new($msg, $options)
    };
    // 变量或其他表达式 + 选项列表
    ($msg:expr, $options:expr) => {
        $crate::MultiSelectBuilder::new($msg, $options)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiselect_builder_new() {
        let options = vec!["Option 1", "Option 2", "Option 3"];
        let builder = MultiSelectBuilder::new("Choose options", options.clone());
        assert_eq!(builder.message, "Choose options");
        assert_eq!(builder.options.len(), 3);
        assert!(builder.default.is_empty());
        assert!(builder.result_title.is_none());
    }

    #[test]
    fn test_multiselect_builder_default() {
        let options = vec!["Option 1", "Option 2", "Option 3"];
        let builder = MultiSelectBuilder::new("Choose", options).default(vec![0, 2]);
        assert_eq!(builder.default, vec![0, 2]);
    }

    #[test]
    fn test_multiselect_builder_default_empty() {
        let options = vec!["Option 1", "Option 2"];
        let builder = MultiSelectBuilder::new("Choose", options).default(vec![]);
        assert!(builder.default.is_empty());
    }

    #[test]
    fn test_multiselect_builder_result_title() {
        let options = vec!["Option 1", "Option 2"];
        let builder = MultiSelectBuilder::new("Choose", options).result_title("Selected");
        assert_eq!(builder.result_title, Some("Selected".to_string()));
    }

    #[test]
    fn test_multiselect_builder_chain() {
        let options = vec!["A", "B", "C"];
        let builder = MultiSelectBuilder::new("Select", options)
            .default(vec![0, 1])
            .result_title("Choices");

        assert_eq!(builder.message, "Select");
        assert_eq!(builder.default, vec![0, 1]);
        assert_eq!(builder.result_title, Some("Choices".to_string()));
    }

    #[test]
    fn test_multiselect_builder_with_string_options() {
        let options: Vec<String> = vec!["Option 1".to_string(), "Option 2".to_string()];
        let builder = MultiSelectBuilder::new("Choose", options);
        assert_eq!(builder.options.len(), 2);
    }
}
