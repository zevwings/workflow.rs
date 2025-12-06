//! 对话框组件
//!
//! 提供基于 ratatui 的交互式对话框，包括输入、选择、多选和确认对话框。

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;

/// 输入验证器函数类型
type InputValidator = Box<dyn Fn(&String) -> Result<(), String> + Send + Sync>;

/// 输入对话框
pub struct InputDialog {
    prompt: String,
    placeholder: Option<String>,
    input: String,
    default: Option<String>,
    allow_empty: bool,
    validator: Option<InputValidator>,
    error_message: Option<String>,
}

impl InputDialog {
    /// 创建新的输入对话框
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            placeholder: None,
            input: String::new(),
            default: None,
            allow_empty: false,
            validator: None,
            error_message: None,
        }
    }

    /// 设置占位符文本
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// 设置默认值
    pub fn default(mut self, default: impl Into<String>) -> Self {
        let default_str = default.into();
        self.default = Some(default_str.clone());
        // 如果输入为空且没有初始文本，使用默认值作为初始输入
        if self.input.is_empty() {
            self.input = default_str;
        }
        self
    }

    /// 设置初始文本（类似 dialoguer 的 with_initial_text）
    pub fn with_initial_text(mut self, initial: impl Into<String>) -> Self {
        self.input = initial.into();
        self
    }

    /// 允许空值输入
    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }

    /// 设置验证函数
    pub fn validate_with<F>(mut self, validator: F) -> Self
    where
        F: Fn(&String) -> Result<(), &str> + Send + Sync + 'static,
    {
        self.validator = Some(Box::new(move |input: &String| {
            validator(input).map_err(|e| e.to_string())
        }));
        self
    }

    /// 显示对话框并返回用户输入
    pub fn show(&mut self) -> Result<String> {
        self.show_impl()
    }

    /// 交互式获取文本输入（兼容 dialoguer API）
    pub fn interact_text(&mut self) -> Result<String> {
        self.show_impl()
    }

    /// 内部实现
    fn show_impl(&mut self) -> Result<String> {
        // 检查是否为 TTY，如果不是则使用简单输入
        if !atty::is(atty::Stream::Stdout) {
            return self.show_simple();
        }

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        let result = loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(if self.error_message.is_some() { 3 } else { 0 }),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 输入框
                let input_text = if self.input.is_empty() {
                    self.placeholder
                        .as_deref()
                        .or(self.default.as_deref())
                        .unwrap_or("")
                        .to_string()
                } else {
                    self.input.clone()
                };
                let input = Paragraph::new(input_text.as_str())
                    .style(if self.input.is_empty() {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    })
                    .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(input, chunks[1]);

                // 错误消息
                if let Some(ref error) = self.error_message {
                    let error_para = Paragraph::new(error.as_str())
                        .style(Style::default().fg(Color::Red))
                        .block(Block::default().borders(Borders::ALL));
                    f.render_widget(error_para, chunks[2]);
                }
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                // 如果输入为空，使用默认值
                                let final_input = if self.input.is_empty() {
                                    self.default.clone().unwrap_or_default()
                                } else {
                                    self.input.clone()
                                };

                                // 检查是否允许空值
                                if final_input.is_empty() && !self.allow_empty {
                                    self.error_message = Some("Input cannot be empty".to_string());
                                    continue;
                                }

                                // 执行验证
                                if let Some(ref validator) = self.validator {
                                    match validator(&final_input) {
                                        Ok(()) => {
                                            self.error_message = None;
                                            break Ok(final_input);
                                        }
                                        Err(e) => {
                                            self.error_message = Some(e);
                                            continue;
                                        }
                                    }
                                } else {
                                    // 没有验证器，直接返回
                                    break Ok(final_input);
                                }
                            }
                            KeyCode::Char(c) => {
                                self.input.push(c);
                                // 清除错误消息
                                self.error_message = None;
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                                // 清除错误消息
                                self.error_message = None;
                            }
                            KeyCode::Esc => {
                                break Err(anyhow::anyhow!("User cancelled input"));
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal.clear()?;
        result
    }

    /// 非交互式终端的简单输入
    fn show_simple(&self) -> Result<String> {
        use std::io::{self, Write};
        print!("{}: ", self.prompt);
        if let Some(ref default) = self.default {
            print!("[{}] ", default);
        }
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();

        // 如果输入为空，使用默认值
        let final_input = if trimmed.is_empty() {
            self.default.clone().unwrap_or_default()
        } else {
            trimmed
        };

        // 检查是否允许空值
        if final_input.is_empty() && !self.allow_empty {
            return Err(anyhow::anyhow!("Input cannot be empty"));
        }

        // 执行验证
        if let Some(ref validator) = self.validator {
            validator(&final_input).map_err(|e| anyhow::anyhow!("{}", e))?;
        }

        Ok(final_input)
    }
}

/// 选择对话框
pub struct SelectDialog {
    prompt: String,
    items: Vec<String>,
    selected: usize,
    default: Option<usize>,
}

impl SelectDialog {
    /// 创建新的选择对话框
    pub fn new(prompt: impl Into<String>, items: &[impl ToString]) -> Self {
        Self {
            prompt: prompt.into(),
            items: items.iter().map(|i| i.to_string()).collect(),
            selected: 0,
            default: None,
        }
    }

    /// 设置默认选择
    pub fn with_default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self.selected = default.min(self.items.len().saturating_sub(1));
        self
    }

    /// 显示对话框并返回选中的索引
    pub fn show(&mut self) -> Result<usize> {
        if self.items.is_empty() {
            return Err(anyhow::anyhow!("No items to select"));
        }

        // 检查是否为 TTY，如果不是则使用简单选择
        if !atty::is(atty::Stream::Stdout) {
            return Ok(self.default.unwrap_or(0));
        }

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected));

        let result = loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 列表
                let items: Vec<ListItem> = self
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let style = if i == self.selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        ListItem::new(item.as_str()).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Yellow));
                f.render_stateful_widget(list, chunks[1], &mut list_state);

                // 状态栏
                let status = Paragraph::new("↑↓ Navigate | Enter: Select | Esc: Cancel")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(status, chunks[2]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Up => {
                                self.selected = self.selected.saturating_sub(1);
                                list_state.select(Some(self.selected));
                            }
                            KeyCode::Down => {
                                self.selected =
                                    (self.selected + 1).min(self.items.len().saturating_sub(1));
                                list_state.select(Some(self.selected));
                            }
                            KeyCode::Enter => {
                                break Ok(self.selected);
                            }
                            KeyCode::Esc => {
                                break Err(anyhow::anyhow!("User cancelled selection"));
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal.clear()?;
        result
    }
}

/// 多选对话框
pub struct MultiSelectDialog {
    prompt: String,
    items: Vec<String>,
    selected: Vec<bool>,
}

impl MultiSelectDialog {
    /// 创建新的多选对话框
    pub fn new(prompt: impl Into<String>, items: &[impl ToString]) -> Self {
        Self {
            prompt: prompt.into(),
            items: items.iter().map(|i| i.to_string()).collect(),
            selected: vec![false; items.len()],
        }
    }

    /// 设置默认选中的项
    pub fn with_defaults(mut self, defaults: &[usize]) -> Self {
        for &idx in defaults {
            if idx < self.selected.len() {
                self.selected[idx] = true;
            }
        }
        self
    }

    /// 显示对话框并返回选中的索引列表
    pub fn show(&mut self) -> Result<Vec<usize>> {
        if self.items.is_empty() {
            return Ok(vec![]);
        }

        // 检查是否为 TTY，如果不是则返回默认值
        if !atty::is(atty::Stream::Stdout) {
            return Ok(self
                .selected
                .iter()
                .enumerate()
                .filter_map(|(i, &selected)| if selected { Some(i) } else { None })
                .collect());
        }

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        let mut cursor = 0;
        let mut list_state = ListState::default();
        list_state.select(Some(cursor));

        let result = loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 列表
                let items: Vec<ListItem> = self
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let prefix = if self.selected[i] { "[x]" } else { "[ ]" };
                        let text = format!("{} {}", prefix, item);
                        let style = if i == cursor {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        ListItem::new(text).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(Style::default().fg(Color::Yellow));
                f.render_stateful_widget(list, chunks[1], &mut list_state);

                // 状态栏
                let status =
                    Paragraph::new("↑↓ Navigate | Space: Toggle | Enter: Confirm | Esc: Cancel")
                        .style(Style::default().fg(Color::DarkGray))
                        .block(Block::default().borders(Borders::ALL));
                f.render_widget(status, chunks[2]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Up => {
                                cursor = cursor.saturating_sub(1);
                                list_state.select(Some(cursor));
                            }
                            KeyCode::Down => {
                                cursor = (cursor + 1).min(self.items.len().saturating_sub(1));
                                list_state.select(Some(cursor));
                            }
                            KeyCode::Char(' ') => {
                                if cursor < self.selected.len() {
                                    self.selected[cursor] = !self.selected[cursor];
                                }
                            }
                            KeyCode::Enter => {
                                let selected_indices: Vec<usize> = self
                                    .selected
                                    .iter()
                                    .enumerate()
                                    .filter_map(
                                        |(i, &selected)| if selected { Some(i) } else { None },
                                    )
                                    .collect();
                                break Ok(selected_indices);
                            }
                            KeyCode::Esc => {
                                break Err(anyhow::anyhow!("User cancelled selection"));
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal.clear()?;
        result
    }
}

/// 确认对话框
pub struct ConfirmDialog {
    prompt: String,
    default: bool,
}

impl ConfirmDialog {
    /// 创建新的确认对话框
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: true,
        }
    }

    /// 设置默认选择
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }

    /// 显示对话框并返回用户选择
    pub fn show(&mut self) -> Result<bool> {
        // 检查是否为 TTY，如果不是则使用默认值
        if !atty::is(atty::Stream::Stdout) {
            return Ok(self.default);
        }

        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        crossterm::terminal::enable_raw_mode()?;

        let mut selected = if self.default { 0 } else { 1 };

        let result = loop {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // 提示信息
                let prompt = Paragraph::new(self.prompt.as_str())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(prompt, chunks[0]);

                // 按钮
                let buttons = ["Yes", "No"];
                let button_text: Vec<Span> = buttons
                    .iter()
                    .enumerate()
                    .map(|(i, text)| {
                        let style = if i == selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        Span::styled(
                            if i == 0 {
                                format!("[{}]", text)
                            } else {
                                format!(" {} ", text)
                            },
                            style,
                        )
                    })
                    .collect();

                let buttons_widget = Paragraph::new(Line::from(button_text))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(buttons_widget, chunks[1]);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Left | KeyCode::Char('n') => {
                                selected = 1;
                            }
                            KeyCode::Right | KeyCode::Char('y') => {
                                selected = 0;
                            }
                            KeyCode::Enter => {
                                break Ok(selected == 0);
                            }
                            KeyCode::Esc => {
                                break Ok(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal.clear()?;
        result
    }
}
