//! 选择提示模块

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    backend::{Backend, TerminalBackend},
    dialog::{
        selection::{
            filter::FuzzyFilter,
            renderer::{
                clear_and_display_result_with_search, OptionListRenderer, OptionRenderer,
                RenderOptionsParams,
            },
        },
        Result, PROMPT_PREFIX, SELECTED_PREFIX, UNSELECTED_PREFIX,
    },
    error::PromptError,
    style::theme::get_theme,
};

/// Select 选项渲染器
struct SelectOptionRenderer;

impl OptionRenderer for SelectOptionRenderer {
    fn render_option(
        &self,
        _index: usize,
        option_text: &str,
        is_current: bool,
        theme: &crate::style::theme::Theme,
    ) -> String {
        if is_current {
            let prefix = theme.success.apply(SELECTED_PREFIX, theme.enable_color);
            let option_styled = theme.answer.apply(option_text, theme.enable_color);
            format!("{}{}", prefix, option_styled)
        } else {
            format!("{}{}", UNSELECTED_PREFIX, option_text)
        }
    }
}

/// 选择提示构建器
pub struct SelectBuilder<T> {
    message: String,
    options: Vec<T>,
    default: Option<usize>,
    result_title: Option<String>,
    page_size: Option<usize>,
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
            page_size: None,
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

    /// 设置分页大小
    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = Some(size);
        self
    }

    /// 执行提示（使用默认终端后端）
    pub fn prompt(self) -> Result<T> {
        use std::io::IsTerminal;
        if !std::io::stdin().is_terminal() {
            let default_idx = self.default.filter(|&idx| idx < self.options.len()).unwrap_or(0);
            return Ok(self.options[default_idx].clone());
        }
        let mut backend = TerminalBackend::default();
        self.prompt_with_backend(&mut backend)
    }

    /// 使用指定后端执行提示（内部使用）
    pub(crate) fn prompt_with_backend<B: Backend>(self, backend: &mut B) -> Result<T> {
        if self.options.is_empty() {
            return Err(PromptError::InvalidInput("The option list cannot be empty".to_string()));
        }

        let theme = get_theme();
        let filter = FuzzyFilter::new();

        let mut current_index = self.default.filter(|&idx| idx < self.options.len()).unwrap_or(0);
        let mut search_query = String::new();

        // 显示提示信息
        let question_prefix = theme.warning.apply(PROMPT_PREFIX, theme.enable_color);
        let styled_text = self.format_message(&theme);
        backend.writeln(&format!("{}{}", question_prefix, styled_text))?;
        backend.flush()?;

        // 进入原始模式
        backend.enable_raw_mode()?;

        let result = self.prompt_loop(backend, &filter, &mut current_index, &mut search_query);

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
        current_index: &mut usize,
        search_query: &mut String,
    ) -> Result<T> {
        let theme = get_theme();
        let renderer = SelectOptionRenderer;
        let mut rendered_lines = 0;

        let filter_options =
            |query: &str| -> (Vec<usize>, Vec<&T>) { filter.filter(&self.options, query) };

        let (mut filtered_indices, mut filtered_options) = filter_options(search_query);

        if !filtered_options.is_empty() {
            *current_index = (*current_index).min(filtered_options.len() - 1);
        } else {
            *current_index = 0;
        }

        // 渲染初始状态
        let hint_text = get_hint_text(search_query);
        rendered_lines = OptionListRenderer::render_options_with_search(
            backend,
            &RenderOptionsParams {
                options: &filtered_options,
                current_index: *current_index,
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
                })) => match code {
                    KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) => {
                        if c == 'c' {
                            print_cancelled_message(backend, rendered_lines)?;
                            return Err(PromptError::Cancelled);
                        }
                    }
                    KeyCode::Char(c) => {
                        search_query.push(c);
                        let (new_indices, new_filtered) = filter_options(search_query);
                        filtered_indices = new_indices;
                        filtered_options = new_filtered;
                        *current_index = 0;

                        let hint_text = get_hint_text(search_query);
                        rendered_lines = OptionListRenderer::render_options_with_search(
                            backend,
                            &RenderOptionsParams {
                                options: &filtered_options,
                                current_index: *current_index,
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
                                *current_index = (*current_index).min(filtered_options.len() - 1);
                            } else {
                                *current_index = 0;
                            }

                            let hint_text = get_hint_text(search_query);
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                backend,
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index: *current_index,
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
                        if !filtered_options.is_empty() && *current_index > 0 {
                            *current_index -= 1;
                            let hint_text = get_hint_text(search_query);
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                backend,
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index: *current_index,
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
                            && *current_index < filtered_options.len() - 1
                        {
                            *current_index += 1;
                            let hint_text = get_hint_text(search_query);
                            rendered_lines = OptionListRenderer::render_options_with_search(
                                backend,
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index: *current_index,
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
                        if filtered_options.is_empty() {
                            continue;
                        }
                        let original_index = filtered_indices[*current_index];
                        let selected = self.options[original_index].clone();
                        let result_text = selected.to_string();
                        let title_text = self.result_title.as_ref().unwrap_or(&self.message);
                        clear_and_display_result_with_search(
                            backend,
                            rendered_lines,
                            title_text,
                            &result_text,
                            &theme,
                        )?;
                        return Ok(selected);
                    }
                    KeyCode::Esc => {
                        if !search_query.is_empty() {
                            search_query.clear();
                            let (new_indices, new_filtered) = filter_options(search_query);
                            filtered_indices = new_indices;
                            filtered_options = new_filtered;
                            *current_index = 0;

                            rendered_lines = OptionListRenderer::render_options_with_search(
                                backend,
                                &RenderOptionsParams {
                                    options: &filtered_options,
                                    current_index: *current_index,
                                    rendered_lines,
                                    theme: &theme,
                                    renderer: &renderer,
                                    hint_text: "Use ↑/↓ to navigate, enter search, and press Enter to confirm",
                                    search_query: None,
                                    page_size: self.page_size,
                                },
                            )?;
                        } else {
                            print_cancelled_message(backend, rendered_lines)?;
                            return Err(PromptError::Cancelled);
                        }
                    }
                    _ => {}
                },
                Ok(_) => continue,
                Err(e) => return Err(PromptError::Io(e)),
            }
        }
    }
}

fn get_hint_text(search_query: &str) -> &'static str {
    if search_query.is_empty() {
        "Use ↑/↓ to navigate, enter search, and press Enter to confirm"
    } else {
        "Use ↑/↓ to navigate, press Esc to clear search, and press Enter to confirm"
    }
}

fn print_cancelled_message<B: Backend>(backend: &mut B, rendered_lines: usize) -> Result<()> {
    let theme = get_theme();

    // 清除已渲染的选项列表
    if rendered_lines > 0 {
        // 向上移动到已渲染区域的第一行
        backend.move_up(rendered_lines as u16)?;

        // 清除所有已渲染的行
        for i in 0..rendered_lines {
            backend.move_to_column(0)?;
            backend.clear_line()?;
            if i < rendered_lines - 1 {
                backend.move_down(1)?;
            }
        }

        // 回到第一行
        if rendered_lines > 1 {
            backend.move_up((rendered_lines - 1) as u16)?;
        }
    }

    // 显示取消消息（不删除提示行）
    let prefix = theme.warning.apply("! ", theme.enable_color);
    let message = theme.hint.apply("Operation cancelled", theme.enable_color);
    backend.writeln(&format!("{}{}", prefix, message))?;
    backend.move_to_column(0)?;
    backend.show_cursor()?;
    backend.flush()?;
    Ok(())
}

#[macro_export]
macro_rules! select {
    ($fmt:literal, $arg:expr, $options:expr) => {
        $crate::SelectBuilder::new(format!($fmt, $arg), $options)
    };
    ($fmt:literal, $options:expr) => {
        $crate::SelectBuilder::new(format!($fmt), $options)
    };
    ($msg:literal, $options:expr) => {
        $crate::SelectBuilder::new($msg, $options)
    };
    ($msg:expr, $options:expr) => {
        $crate::SelectBuilder::new($msg, $options)
    };
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyEvent, KeyModifiers};

    use super::*;
    use crate::backend::MockBackend;

    #[test]
    fn test_select_builder_new() {
        let options = vec!["Option 1", "Option 2", "Option 3"];
        let builder = SelectBuilder::new("Choose an option", options.clone());
        assert_eq!(builder.message, "Choose an option");
        assert_eq!(builder.options.len(), 3);
        assert!(builder.default.is_none());
        assert!(builder.result_title.is_none());
    }

    #[test]
    fn test_select_builder_default() {
        let options = vec!["Option 1", "Option 2", "Option 3"];
        let builder = SelectBuilder::new("Choose", options).default(1);
        assert_eq!(builder.default, Some(1));
    }

    #[test]
    fn test_select_builder_result_title() {
        let options = vec!["Option 1", "Option 2"];
        let builder = SelectBuilder::new("Choose", options).result_title("Selected");
        assert_eq!(builder.result_title, Some("Selected".to_string()));
    }

    #[test]
    fn test_select_builder_chain() {
        let options = vec!["A", "B", "C"];
        let builder = SelectBuilder::new("Select", options).default(0).result_title("Choice");

        assert_eq!(builder.message, "Select");
        assert_eq!(builder.default, Some(0));
        assert_eq!(builder.result_title, Some("Choice".to_string()));
    }

    #[test]
    fn test_select_builder_with_string_options() {
        let options: Vec<String> = vec!["Option 1".to_string(), "Option 2".to_string()];
        let builder = SelectBuilder::new("Choose", options);
        assert_eq!(builder.options.len(), 2);
    }

    // ========================================================================
    // MockBackend 测试 - 测试实际交互逻辑
    // ========================================================================

    #[test]
    fn test_select_empty_options() {
        let options: Vec<&str> = vec![];
        let mut backend = MockBackend::new();

        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::InvalidInput(_)));
    }

    #[test]
    fn test_select_first_option_with_enter() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result =
            SelectBuilder::new("Choose a fruit", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Apple");
    }

    #[test]
    fn test_select_with_default_index() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose a fruit", options)
            .default(1)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Banana");
    }

    #[test]
    fn test_select_default_out_of_bounds() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        // default 超出范围，应该回退到 0
        let result = SelectBuilder::new("Choose", options)
            .default(100)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Apple");
    }

    #[test]
    fn test_select_navigate_down_and_select() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cherry");
    }

    #[test]
    fn test_select_navigate_up_and_select() {
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Banana");
    }

    #[test]
    fn test_select_navigate_up_at_top() {
        // 在顶部按 Up 应该保持在顶部
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Apple");
    }

    #[test]
    fn test_select_navigate_down_at_bottom() {
        // 在底部按 Down 应该保持在底部
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cherry");
    }

    #[test]
    fn test_select_cancel_with_escape() {
        let events = vec![MockBackend::press_escape()];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::Cancelled));
    }

    #[test]
    fn test_select_cancel_with_ctrl_c() {
        let events = vec![Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        ))];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PromptError::Cancelled));
    }

    #[test]
    fn test_select_search_filter() {
        // 输入 "b" 过滤出 Banana，然后按 Enter 选择
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Banana");
    }

    #[test]
    fn test_select_search_no_match_then_backspace() {
        // 输入 "xyz" 无匹配，然后删除，再输入 "a" 选择
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            MockBackend::press_enter(),
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Cherry");
    }

    #[test]
    fn test_select_escape_clears_search() {
        // 输入搜索词，然后按 Escape 清除搜索，再选择第一项
        let events = vec![
            Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
            MockBackend::press_escape(), // 清除搜索
            MockBackend::press_enter(),  // 选择第一项 Apple
        ];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Apple");
    }

    #[test]
    fn test_select_with_result_title() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let options = vec!["Apple", "Banana", "Cherry"];
        let result = SelectBuilder::new("Choose a fruit", options)
            .result_title("Selected fruit")
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Apple");
    }

    #[test]
    fn test_select_with_page_size() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        let options: Vec<String> = (1..=20).map(|i| format!("Option {}", i)).collect();
        let result = SelectBuilder::new("Choose", options)
            .page_size(5)
            .prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Option 1");
    }

    #[test]
    fn test_select_terminal_modes() {
        let events = vec![MockBackend::press_enter()];
        let mut backend = MockBackend::with_events(events);

        assert!(!backend.is_raw_mode());

        let options = vec!["Apple", "Banana"];
        let result = SelectBuilder::new("Choose", options).prompt_with_backend(&mut backend);

        assert!(result.is_ok());
        assert!(!backend.is_raw_mode());
        assert!(backend.is_cursor_visible());
    }
}
