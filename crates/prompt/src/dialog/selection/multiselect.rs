//! 多选提示模块

use crate::backend::{Backend, TerminalBackend};
use crate::dialog::selection::filter::FuzzyFilter;
use crate::dialog::selection::renderer::{
    clear_and_display_result_with_search, OptionListRenderer, OptionRenderer, RenderOptionsParams,
};
use crate::dialog::{Result, PROMPT_PREFIX, SELECTED_PREFIX, UNSELECTED_PREFIX};
use crate::error::PromptError;
use crate::style::theme::get_theme;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;

/// MultiSelect 选项渲染器
struct MultiSelectOptionRenderer<'a> {
    selected: &'a HashSet<usize>,
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
        let original_index = self.original_to_filtered[index];
        let is_selected = self.selected.contains(&original_index);

        let prefix = if is_current {
            theme.success.apply(SELECTED_PREFIX, theme.enable_color)
        } else {
            UNSELECTED_PREFIX.to_string()
        };

        let marker = if is_selected { "[x]" } else { "[ ]" };

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

    /// 设置分页大小
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = Some(size);
        self
    }

    /// 执行提示（使用默认终端后端）
    pub fn prompt(self) -> Result<Vec<T>> {
        let mut backend = TerminalBackend::default();
        self.prompt_with_backend(&mut backend)
    }

    /// 使用指定后端执行提示（内部使用）
    pub(crate) fn prompt_with_backend<B: Backend>(self, backend: &mut B) -> Result<Vec<T>> {
        if self.options.is_empty() {
            return Err(PromptError::InvalidInput("选项列表不能为空".to_string()));
        }

        let theme = get_theme();
        let filter = FuzzyFilter::new();

        let mut selected: HashSet<usize> =
            self.default.iter().copied().filter(|&idx| idx < self.options.len()).collect();
        let mut search_query = String::new();

        // 显示提示信息
        let question_prefix = theme.warning.apply(PROMPT_PREFIX, theme.enable_color);
        let styled_text = self.format_message(&theme);
        backend.writeln(&format!("{}{}", question_prefix, styled_text))?;
        backend.flush()?;

        // 进入原始模式
        backend.enable_raw_mode()?;

        let result =
            self.prompt_loop(backend, &filter, &mut selected, &mut search_query);

        // 恢复终端状态
        backend.show_cursor().ok();
        backend.disable_raw_mode().ok();

        result
    }

    fn format_message(&self, theme: &crate::style::theme::Theme) -> String {
        if let Some(current_start) = self.message.find("[current:") {
            let (base_message, current_part) = self.message.split_at(current_start);
            let default_value = if let Some(colon_pos) = current_part.find(": ") {
                let value_start = colon_pos + 2;
                if let Some(bracket_end) = current_part[value_start..].find(']') {
                    format!(
                        "[{}]",
                        &current_part[value_start..value_start + bracket_end]
                    )
                } else {
                    format!("[{}]", &current_part[value_start..].trim_end_matches(']'))
                }
            } else {
                current_part.to_string()
            };
            let styled_message = theme.title.apply(base_message.trim_end(), theme.enable_color);
            let styled_default = theme.hint.apply(&default_value, theme.enable_color);
            format!("{} {}", styled_message, styled_default)
        } else {
            theme.title.apply(&self.message, theme.enable_color)
        }
    }

    fn prompt_loop<B: Backend>(
        &self,
        backend: &mut B,
        filter: &FuzzyFilter,
        selected: &mut HashSet<usize>,
        search_query: &mut String,
    ) -> Result<Vec<T>> {
        let theme = get_theme();
        let mut rendered_lines = 0;

        let filter_options =
            |query: &str| -> (Vec<usize>, Vec<&T>) { filter.filter(&self.options, query) };

        let (mut filtered_indices, mut filtered_options) = filter_options(search_query);

        let mut current_index = if !filtered_options.is_empty() {
            if !selected.is_empty() {
                filtered_indices.iter().position(|&idx| selected.contains(&idx)).unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // 渲染初始状态
        let renderer = MultiSelectOptionRenderer {
            selected,
            original_to_filtered: &filtered_indices,
        };
        let hint_text = get_hint_text(search_query);
        rendered_lines = OptionListRenderer::render_options_with_search(
            backend,
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
                    Some(search_query)
                },
                page_size: self.page_size,
            },
        )?;

        loop {
            match backend.read_event() {
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
                        KeyCode::Char(' ') => {
                            if filtered_options.is_empty() {
                                continue;
                            }
                            let original_index = filtered_indices[current_index];
                            if selected.contains(&original_index) {
                                selected.remove(&original_index);
                            } else {
                                selected.insert(original_index);
                            }
                            let hint_text = get_hint_text(search_query);
                            let renderer = MultiSelectOptionRenderer {
                                selected,
                                original_to_filtered: &filtered_indices,
                            };
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                backend,
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
                                        Some(search_query)
                                    },
                                    page_size: self.page_size,
                                },
                            )?;
                        }
                        KeyCode::Char(c) => {
                            search_query.push(c);
                            let (new_indices, new_filtered) = filter_options(search_query);
                            filtered_indices = new_indices;
                            filtered_options = new_filtered;
                            current_index = 0;

                            let hint_text = get_hint_text(search_query);
                            let renderer = MultiSelectOptionRenderer {
                                selected,
                                original_to_filtered: &filtered_indices,
                            };
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                backend,
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index,
                                    rendered_lines,
                                    theme: &theme,
                                    renderer: &renderer,
                                    hint_text,
                                    search_query: Some(search_query),
                                    page_size: self.page_size,
                                },
                            )?;
                        }
                        KeyCode::Backspace => {
                            if !search_query.is_empty() {
                                search_query.pop();
                                let (new_indices, new_filtered) = filter_options(search_query);
                                filtered_indices = new_indices;
                                filtered_options = new_filtered;

                                if !filtered_options.is_empty() {
                                    current_index = current_index.min(filtered_options.len() - 1);
                                } else {
                                    current_index = 0;
                                }

                                let hint_text = get_hint_text(search_query);
                                let renderer = MultiSelectOptionRenderer {
                                    selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    backend,
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
                                            Some(search_query)
                                        },
                                        page_size: self.page_size,
                                    },
                                )?;
                            }
                        }
                        KeyCode::Up => {
                            if !filtered_options.is_empty() && current_index > 0 {
                                current_index -= 1;
                                let hint_text = get_hint_text(search_query);
                                let renderer = MultiSelectOptionRenderer {
                                    selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    backend,
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
                                            Some(search_query)
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
                                let hint_text = get_hint_text(search_query);
                                let renderer = MultiSelectOptionRenderer {
                                    selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    backend,
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
                                            Some(search_query)
                                        },
                                        page_size: self.page_size,
                                    },
                                )?;
                            }
                        }
                        KeyCode::Enter => {
                            let mut selected_indices: Vec<usize> =
                                selected.iter().copied().collect();
                            selected_indices.sort();
                            let selected_items: Vec<T> = selected_indices
                                .iter()
                                .map(|&idx| self.options[idx].clone())
                                .collect();

                            let result_text = if selected_items.is_empty() {
                                "(未选择)".to_string()
                            } else {
                                selected_items
                                    .iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };

                            let title_text = self.result_title.as_ref().unwrap_or(&self.message);
                            clear_and_display_result_with_search(
                                backend,
                                rendered_lines,
                                title_text,
                                &result_text,
                                &theme,
                            )?;
                            return Ok(selected_items);
                        }
                        KeyCode::Esc => {
                            if !search_query.is_empty() {
                                search_query.clear();
                                let (new_indices, new_filtered) = filter_options(search_query);
                                filtered_indices = new_indices;
                                filtered_options = new_filtered;
                                current_index = 0;

                                let renderer = MultiSelectOptionRenderer {
                                    selected,
                                    original_to_filtered: &filtered_indices,
                                };
                                rendered_lines = OptionListRenderer::render_options_with_search(
                                    backend,
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
                                print_cancelled_message(backend)?;
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

fn get_hint_text(search_query: &str) -> &'static str {
    if search_query.is_empty() {
        "使用 ↑/↓ 导航，输入搜索，空格键切换选择，回车确认"
    } else {
        "使用 ↑/↓ 导航，Esc 清除搜索，空格键切换选择，回车确认"
    }
}

fn print_cancelled_message<B: Backend>(backend: &mut B) -> Result<()> {
    let theme = get_theme();
    let prefix = theme.warning.apply("! ", theme.enable_color);
    let message = theme.hint.apply("Operation cancelled", theme.enable_color);
    backend.writeln(&format!("{}{}", prefix, message))?;
    backend.flush()?;
    Ok(())
}

#[macro_export]
macro_rules! multiselect {
    ($fmt:literal, $arg:expr, $options:expr) => {
        $crate::MultiSelectBuilder::new(format!($fmt, $arg), $options)
    };
    ($fmt:literal, $arg1:expr, $arg2:expr, $($arg:expr),+ $(,)?, $options:expr) => {
        $crate::MultiSelectBuilder::new(format!($fmt, $arg1, $arg2, $($arg),+), $options)
    };
    ($msg:literal, $options:expr) => {
        $crate::MultiSelectBuilder::new($msg, $options)
    };
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
