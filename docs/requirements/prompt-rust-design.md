# Prompt 模块 Rust 版本设计

## 设计原则

本文档设计一个完全基于 Rust 最佳实践的交互式提示库，遵循以下原则：

1. **类型安全**：充分利用 Rust 的类型系统，编译期保证正确性
2. **零成本抽象**：使用 Trait 和泛型，运行时无额外开销
3. **所有权清晰**：明确的所有权语义，避免不必要的克隆
4. **错误处理**：使用 `Result<T>` 进行错误处理，提供清晰的错误信息
5. **可组合性**：模块化设计，易于组合和扩展
6. **性能优先**：避免不必要的分配，使用零拷贝技术
7. **异步友好**：支持同步和异步两种模式（可选）

## 一、核心功能需求

### 1.1 输入提示（Input/Password）
- 文本输入和密码输入
- 默认值支持
- 占位符显示
- 实时验证
- 字符级编辑（光标移动、删除）
- 密码掩码显示

### 1.2 确认提示（Confirm）
- Yes/No 选择
- 默认值支持
- 单键响应（y/n）

### 1.3 选择提示（Select/MultiSelect）
- 单选和多选
- 方向键导航
- 默认选中
- 交互式渲染

### 1.4 表单提示（Form）
- 组合多个字段
- 条件显示
- 字段级和表单级验证

### 1.5 消息输出（Message）
- 多级别消息（Info、Success、Warning、Error）
- 格式化输出
- 分隔线

### 1.6 加载指示器（Spinner）
- 动画显示
- 自定义帧序列
- 完成后消息

### 1.7 表格显示（Table）
- 表头和行
- 边框和对齐
- 样式渲染

## 二、架构设计

### 2.1 模块结构（Rust 风格）

```
src/lib/base/prompt/
├── lib.rs                    # 公共 API 导出
├── error.rs                  # 错误类型定义
├── terminal/                 # 终端抽象
│   ├── mod.rs
│   ├── trait.rs              # Terminal Trait
│   ├── std.rs                # 标准终端实现
│   └── mock.rs               # Mock 终端（测试）
├── style/                    # 样式系统
│   ├── mod.rs
│   ├── theme.rs              # 主题定义
│   └── render.rs             # 渲染器
├── input/                    # 输入模块
│   ├── mod.rs
│   ├── editor.rs             # 输入编辑器
│   ├── validator.rs          # 验证器
│   └── prompt.rs             # Input/Password Prompt
├── confirm/                   # 确认模块
│   ├── mod.rs
│   └── prompt.rs
├── select/                   # 选择模块
│   ├── mod.rs
│   ├── prompt.rs             # Select/MultiSelect Prompt
│   └── render.rs             # 选项渲染
├── form/                     # 表单模块
│   ├── mod.rs
│   ├── builder.rs            # 表单构建器
│   ├── field.rs              # 字段定义
│   └── result.rs             # 表单结果
├── message/                  # 消息模块
│   ├── mod.rs
│   └── output.rs
├── spinner/                  # 加载指示器
│   ├── mod.rs
│   └── animation.rs
└── table/                    # 表格模块
    ├── mod.rs
    └── render.rs
```

### 2.2 核心 Trait 设计

#### 2.2.1 Terminal Trait
```rust
use std::io;

/// 终端抽象，支持同步和异步操作
pub trait Terminal: Send + Sync {
    /// 读取单个字节（用于交互式输入）
    fn read_byte(&mut self) -> io::Result<u8>;

    /// 读取一行（用于 fallback 模式）
    fn read_line(&mut self) -> io::Result<String>;

    /// 写入字符串
    fn write(&mut self, s: &str) -> io::Result<()>;

    /// 写入并刷新
    fn write_flush(&mut self, s: &str) -> io::Result<()>;

    /// 进入原始模式，返回 Guard
    fn enable_raw_mode(&mut self) -> io::Result<RawModeGuard>;

    /// 获取终端大小
    fn size(&self) -> io::Result<(u16, u16)>;

    /// 是否支持颜色
    fn supports_color(&self) -> bool;

    /// 是否在 TTY 中
    fn is_tty(&self) -> bool;
}

/// 原始模式 Guard，自动恢复终端状态
pub struct RawModeGuard {
    // 内部实现
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // 自动恢复终端状态
    }
}
```

#### 2.2.2 Validator Trait
```rust
/// 验证器 Trait
pub trait Validator: Send + Sync {
    /// 验证输入，返回错误消息（如果验证失败）
    fn validate(&self, input: &str) -> Result<(), String>;
}

/// 函数式验证器
impl<F> Validator for F
where
    F: Fn(&str) -> Result<(), String> + Send + Sync,
{
    fn validate(&self, input: &str) -> Result<(), String> {
        self(input)
    }
}

/// 内置验证器
pub mod validators {
    use super::Validator;

    pub fn required() -> impl Validator {
        move |input: &str| {
            if input.trim().is_empty() {
                Err("此字段为必填项".to_string())
            } else {
                Ok(())
            }
        }
    }

    pub fn email() -> impl Validator {
        move |input: &str| {
            // 邮箱验证逻辑
            if input.contains('@') {
                Ok(())
            } else {
                Err("请输入有效的邮箱地址".to_string())
            }
        }
    }

    pub fn min_length(min: usize) -> impl Validator {
        move |input: &str| {
            if input.len() >= min {
                Ok(())
            } else {
                Err(format!("长度至少为 {} 个字符", min))
            }
        }
    }
}
```

#### 2.2.3 Prompt Trait（可选）
```rust
/// Prompt Trait，统一所有提示类型的接口
pub trait Prompt<T> {
    /// 执行提示并返回结果
    fn prompt(&mut self) -> Result<T, PromptError>;
}
```

### 2.3 错误处理

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("User cancelled")]
    Cancelled,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Terminal not supported")]
    TerminalNotSupported,
}
```

### 2.4 样式系统

```rust
use crossterm::style::{Color, Attribute, Stylize};

/// 样式定义
#[derive(Clone, Debug)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub attributes: Vec<Attribute>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            foreground: None,
            background: None,
            attributes: Vec::new(),
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.attributes.push(Attribute::Bold);
        self
    }

    pub fn apply(&self, text: &str) -> String {
        // 应用样式
        // 如果颜色未启用，返回原始文本
        // ...
    }
}

/// 主题配置
#[derive(Clone, Debug)]
pub struct Theme {
    pub info: Style,
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub prompt: Style,
    pub answer: Style,
    pub hint: Style,
    pub enable_color: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            info: Style::new().fg(Color::Cyan),
            success: Style::new().fg(Color::Green),
            warning: Style::new().fg(Color::Yellow),
            error: Style::new().fg(Color::Red).bold(),
            prompt: Style::new().fg(Color::White),
            answer: Style::new().fg(Color::Cyan),
            hint: Style::new().fg(Color::DarkGrey),
            enable_color: true,
        }
    }
}

/// 全局主题（线程安全）
static THEME: Lazy<RwLock<Theme>> = Lazy::new(|| RwLock::new(Theme::default()));

pub fn set_theme(theme: Theme) {
    *THEME.write().unwrap() = theme;
}

pub fn get_theme() -> Theme {
    THEME.read().unwrap().clone()
}
```

## 三、核心模块设计

### 3.1 输入模块（Input）

#### 3.1.1 设计思路
- 使用状态机管理输入状态
- 字符级编辑，支持光标移动
- 零拷贝字符串处理（尽可能使用 `&str`）

#### 3.1.2 API 设计
```rust
use crate::base::prompt::{Terminal, Validator, PromptError};

/// 输入提示构建器
pub struct InputBuilder {
    message: String,
    default: Option<String>,
    placeholder: Option<String>,
    validator: Option<Box<dyn Validator>>,
    password: bool,
    theme: Option<Theme>,
}

impl InputBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            placeholder: None,
            validator: None,
            password: false,
            theme: None,
        }
    }

    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        self.validator = Some(Box::new(validator));
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// 执行提示
    pub fn prompt<T: Terminal>(mut self, terminal: &mut T) -> Result<String, PromptError> {
        // 实现逻辑
        // 1. 显示提示和默认值
        // 2. 进入原始模式
        // 3. 字符级输入处理
        // 4. 验证
        // 5. 格式化显示结果
        // 6. 返回结果
        todo!()
    }
}

/// 便捷函数
pub fn input(message: impl Into<String>) -> InputBuilder {
    InputBuilder::new(message)
}
```

#### 3.1.3 输入编辑器
```rust
/// 输入编辑器，管理输入缓冲区和光标位置
pub struct InputEditor {
    buffer: String,
    cursor: usize,
    placeholder: Option<String>,
}

impl InputEditor {
    pub fn new(placeholder: Option<String>) -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            placeholder,
        }
    }

    pub fn insert(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
            true
        } else {
            false
        }
    }

    pub fn delete(&mut self) -> bool {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
            true
        } else {
            false
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor
    }

    pub fn display_text(&self) -> &str {
        if self.buffer.is_empty() {
            self.placeholder.as_deref().unwrap_or("")
        } else {
            &self.buffer
        }
    }
}
```

### 3.2 确认模块（Confirm）

```rust
pub struct ConfirmBuilder {
    message: String,
    default: Option<bool>,
    theme: Option<Theme>,
}

impl ConfirmBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            default: None,
            theme: None,
        }
    }

    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    pub fn prompt<T: Terminal>(mut self, terminal: &mut T) -> Result<bool, PromptError> {
        // 实现逻辑
        // 1. 显示提示和默认值提示（Y/n 或 y/N）
        // 2. 读取单键输入
        // 3. 处理 y/n/Enter
        // 4. 返回结果
        todo!()
    }
}

pub fn confirm(message: impl Into<String>) -> ConfirmBuilder {
    ConfirmBuilder::new(message)
}
```

### 3.3 选择模块（Select/MultiSelect）

```rust
/// 单选提示
pub struct SelectBuilder<T> {
    message: String,
    options: Vec<T>,
    default: Option<usize>,
    theme: Option<Theme>,
}

impl<T: Display + Clone> SelectBuilder<T> {
    pub fn new(message: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            message: message.into(),
            options,
            default: None,
            theme: None,
        }
    }

    pub fn default(mut self, index: usize) -> Self {
        self.default = Some(index);
        self
    }

    pub fn prompt<TR: Terminal>(mut self, terminal: &mut TR) -> Result<T, PromptError> {
        // 实现逻辑
        // 1. 显示提示和选项列表
        // 2. 进入原始模式
        // 3. 处理方向键导航
        // 4. 处理 Enter 确认
        // 5. 返回选中的选项
        todo!()
    }
}

/// 多选提示
pub struct MultiSelectBuilder<T> {
    message: String,
    options: Vec<T>,
    default: Vec<usize>,
    theme: Option<Theme>,
}

impl<T: Display + Clone> MultiSelectBuilder<T> {
    pub fn new(message: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            message: message.into(),
            options,
            default: Vec::new(),
            theme: None,
        }
    }

    pub fn default(mut self, indices: Vec<usize>) -> Self {
        self.default = indices;
        self
    }

    pub fn prompt<TR: Terminal>(mut self, terminal: &mut TR) -> Result<Vec<T>, PromptError> {
        // 实现逻辑
        // 1. 显示提示和选项列表（带复选框）
        // 2. 进入原始模式
        // 3. 处理方向键导航
        // 4. 处理空格键切换选中
        // 5. 处理 Enter 确认
        // 6. 返回选中的选项列表
        todo!()
    }
}

pub fn select<T: Display + Clone>(message: impl Into<String>, options: Vec<T>) -> SelectBuilder<T> {
    SelectBuilder::new(message, options)
}

pub fn multi_select<T: Display + Clone>(message: impl Into<String>, options: Vec<T>) -> MultiSelectBuilder<T> {
    MultiSelectBuilder::new(message, options)
}
```

### 3.4 表单模块（Form）

```rust
use std::collections::HashMap;

/// 表单字段类型
pub enum FieldType {
    Input,
    Password,
    Confirm,
    Select,
    MultiSelect,
}

/// 表单字段
pub struct FormField {
    key: String,
    field_type: FieldType,
    message: String,
    // ... 其他配置
}

/// 表单结果
pub struct FormResult {
    data: HashMap<String, FormValue>,
}

#[derive(Clone, Debug)]
pub enum FormValue {
    String(String),
    Bool(bool),
    Int(i64),
    Vec(Vec<String>),
}

impl FormResult {
    pub fn get_string(&self, key: &str) -> Result<&String, PromptError> {
        match self.data.get(key) {
            Some(FormValue::String(s)) => Ok(s),
            _ => Err(PromptError::InvalidInput(format!("Key {} is not a string", key))),
        }
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, PromptError> {
        match self.data.get(key) {
            Some(FormValue::Bool(b)) => Ok(*b),
            _ => Err(PromptError::InvalidInput(format!("Key {} is not a bool", key))),
        }
    }

    // ... 其他 getter 方法
}

/// 表单构建器
pub struct FormBuilder {
    fields: Vec<FormField>,
    theme: Option<Theme>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            theme: None,
        }
    }

    pub fn add_input(mut self, key: impl Into<String>, message: impl Into<String>) -> Self {
        self.fields.push(FormField {
            key: key.into(),
            field_type: FieldType::Input,
            message: message.into(),
        });
        self
    }

    // ... 其他 add 方法

    pub fn prompt<TR: Terminal>(mut self, terminal: &mut TR) -> Result<FormResult, PromptError> {
        // 实现逻辑
        // 1. 遍历字段
        // 2. 根据字段类型调用相应的 prompt
        // 3. 收集结果
        // 4. 返回 FormResult
        todo!()
    }
}

pub fn form() -> FormBuilder {
    FormBuilder::new()
}
```

### 3.5 消息模块（Message）

```rust
use std::io::Write;

/// 消息输出器
pub struct Message {
    theme: Theme,
    writer: Box<dyn Write>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            theme: get_theme(),
            writer: Box::new(std::io::stdout()),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn info(&mut self, msg: impl AsRef<str>) -> Result<(), PromptError> {
        let styled = self.theme.info.apply(&format!("ℹ {}", msg.as_ref()));
        writeln!(self.writer, "{}", styled)?;
        Ok(())
    }

    pub fn success(&mut self, msg: impl AsRef<str>) -> Result<(), PromptError> {
        let styled = self.theme.success.apply(&format!("✓ {}", msg.as_ref()));
        writeln!(self.writer, "{}", styled)?;
        Ok(())
    }

    pub fn warning(&mut self, msg: impl AsRef<str>) -> Result<(), PromptError> {
        let styled = self.theme.warning.apply(&format!("⚠ {}", msg.as_ref()));
        writeln!(self.writer, "{}", styled)?;
        Ok(())
    }

    pub fn error(&mut self, msg: impl AsRef<str>) -> Result<(), PromptError> {
        let styled = self.theme.error.apply(&format!("✗ {}", msg.as_ref()));
        writeln!(self.writer, "{}", styled)?;
        Ok(())
    }

    pub fn break_line(&mut self) -> Result<(), PromptError> {
        writeln!(self.writer)?;
        Ok(())
    }

    pub fn separator(&mut self, char: char, length: usize) -> Result<(), PromptError> {
        let line: String = std::iter::repeat(char).take(length).collect();
        writeln!(self.writer, "{}", line)?;
        Ok(())
    }
}
```

### 3.6 加载指示器（Spinner）

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 加载指示器
pub struct Spinner {
    message: String,
    frames: Vec<String>,
    interval: Duration,
    running: Arc<Mutex<bool>>,
    theme: Theme,
}

impl Spinner {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
                .into_iter()
                .map(String::from)
                .collect(),
            interval: Duration::from_millis(100),
            running: Arc::new(Mutex::new(false)),
            theme: get_theme(),
        }
    }

    pub fn with_frames(mut self, frames: Vec<String>) -> Self {
        self.frames = frames;
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn start(&mut self) {
        *self.running.lock().unwrap() = true;
        let running = Arc::clone(&self.running);
        let frames = self.frames.clone();
        let message = self.message.clone();
        let interval = self.interval;
        let theme = self.theme.clone();

        thread::spawn(move || {
            let mut frame_idx = 0;
            while *running.lock().unwrap() {
                let frame = &frames[frame_idx % frames.len()];
                let styled = theme.info.apply(&format!("{} {}", frame, message));
                print!("\r{}", styled);
                std::io::stdout().flush().ok();

                thread::sleep(interval);
                frame_idx += 1;
            }
        });
    }

    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
        print!("\r{}", " ".repeat(80)); // 清除行
        print!("\r");
        std::io::stdout().flush().ok();
    }

    pub fn with_success(mut self, msg: impl Into<String>) -> Result<(), PromptError> {
        self.stop();
        let mut message = Message::new();
        message.success(msg)?;
        Ok(())
    }

    pub fn with_error(mut self, msg: impl Into<String>) -> Result<(), PromptError> {
        self.stop();
        let mut message = Message::new();
        message.error(msg)?;
        Ok(())
    }

    pub fn do_work<F, E>(mut self, work: F) -> Result<(), PromptError>
    where
        F: FnOnce() -> Result<(), E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.start();
        let result = work();
        self.stop();
        result.map_err(|e| PromptError::Terminal(e.to_string()))
    }
}
```

### 3.7 表格模块（Table）

```rust
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    border: bool,
    alignment: Alignment,
}

#[derive(Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Table {
    pub fn new(headers: Vec<impl Into<String>>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
            border: true,
            alignment: Alignment::Left,
        }
    }

    pub fn add_row(&mut self, row: Vec<impl Into<String>>) {
        self.rows.push(row.into_iter().map(Into::into).collect());
    }

    pub fn with_border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn render(&self) -> Result<(), PromptError> {
        // 实现表格渲染逻辑
        // 计算列宽
        // 渲染边框
        // 渲染行
        todo!()
    }
}
```

## 四、技术实现细节

### 4.1 终端控制

使用 `crossterm` 库进行跨平台终端控制：

```rust
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

// 进入原始模式
let mut stdout = std::io::stdout();
terminal::enable_raw_mode()?;

// 读取键盘事件
match event::read()? {
    Event::Key(KeyEvent { code, .. }) => {
        match code {
            KeyCode::Char('y') => { /* ... */ }
            KeyCode::Up => { /* ... */ }
            KeyCode::Down => { /* ... */ }
            KeyCode::Enter => { /* ... */ }
            KeyCode::Esc => { /* ... */ }
            _ => {}
        }
    }
    _ => {}
}

// 恢复终端
terminal::disable_raw_mode()?;
```

### 4.2 键盘事件处理

```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
    Enter,
    Backspace,
    Delete,
    Tab,
    Esc,
    Ctrl(char),
    Unknown,
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        match event.code {
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    Key::Ctrl(c)
                } else {
                    Key::Char(c)
                }
            }
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Enter => Key::Enter,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Delete => Key::Delete,
            KeyCode::Tab => Key::Tab,
            KeyCode::Esc => Key::Esc,
            _ => Key::Unknown,
        }
    }
}
```

### 4.3 ANSI 转义序列

```rust
pub mod ansi {
    pub const CLEAR_LINE: &str = "\x1b[2K";
    pub const CLEAR_TO_END: &str = "\x1b[K";
    pub const MOVE_LEFT: &str = "\x1b[D";
    pub const MOVE_RIGHT: &str = "\x1b[C";
    pub const HIDE_CURSOR: &str = "\x1b[?25l";
    pub const SHOW_CURSOR: &str = "\x1b[?25h";
    pub const RESET: &str = "\x1b[0m";

    pub fn move_up(n: u16) -> String {
        format!("\x1b[{}A", n)
    }

    pub fn move_down(n: u16) -> String {
        format!("\x1b[{}B", n)
    }
}
```

### 4.4 错误处理最佳实践

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("User cancelled")]
    Cancelled,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Terminal not supported")]
    TerminalNotSupported,
}

pub type Result<T> = std::result::Result<T, PromptError>;
```

## 五、使用示例

### 5.1 基础使用

```rust
use workflow::base::prompt::*;

fn main() -> Result<()> {
    let mut terminal = StdTerminal::new()?;

    // 输入提示
    let name = input("请输入您的姓名")
        .default("John Doe")
        .prompt(&mut terminal)?;

    // 密码输入
    let password = input("请输入密码")
        .password()
        .validator(validators::min_length(8))
        .prompt(&mut terminal)?;

    // 确认提示
    let confirmed = confirm("是否继续？")
        .default(true)
        .prompt(&mut terminal)?;

    // 选择提示
    let options = vec!["选项1", "选项2", "选项3"];
    let selected = select("请选择一个选项", options)
        .default(0)
        .prompt(&mut terminal)?;

    // 多选提示
    let selected = multi_select("请选择多个选项", options)
        .default(vec![0, 2])
        .prompt(&mut terminal)?;

    // 表单
    let result = form()
        .add_input("name", "姓名")
        .add_password("password", "密码")
        .add_select("role", "角色", vec!["开发者", "测试"])
        .prompt(&mut terminal)?;

    let name: &String = result.get_string("name")?;

    // 消息输出
    let mut msg = Message::new();
    msg.info("这是一条信息")?;
    msg.success("操作成功")?;
    msg.warning("这是一条警告")?;
    msg.error("这是一条错误")?;

    // 加载指示器
    let spinner = Spinner::new("正在处理...");
    spinner.start();
    // 执行操作
    thread::sleep(Duration::from_secs(2));
    spinner.with_success("处理完成")?;

    Ok(())
}
```

### 5.2 高级使用

```rust
// 自定义验证器
let email = input("请输入邮箱")
    .validator(|input: &str| {
        if input.contains('@') && input.contains('.') {
            Ok(())
        } else {
            Err("请输入有效的邮箱地址".to_string())
        }
    })
    .prompt(&mut terminal)?;

// 自定义主题
let theme = Theme {
    info: Style::new().fg(Color::Blue),
    success: Style::new().fg(Color::Green),
    // ...
    ..Theme::default()
};
set_theme(theme);

// 自定义 Spinner 帧
let spinner = Spinner::new("正在处理...")
    .with_frames(vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]
        .into_iter()
        .map(String::from)
        .collect())
    .with_interval(Duration::from_millis(50));
```

## 六、依赖库

### 6.1 必需依赖

```toml
[dependencies]
crossterm = "0.28"          # 跨平台终端控制
thiserror = "1.0"           # 错误类型定义
lazy_static = "1.4"         # 全局变量（或使用 std::sync::OnceLock）
```

### 6.2 可选依赖

```toml
[dependencies]
owo-colors = "4.0"          # 颜色格式化（可选，如果不用 crossterm 的样式）
unicode-width = "0.1"       # Unicode 字符宽度计算
```

## 七、测试策略

### 7.1 单元测试

- 使用 Mock Terminal 进行测试
- 测试验证器逻辑
- 测试输入编辑器逻辑
- 测试样式渲染

### 7.2 集成测试

- 测试完整的提示流程
- 测试错误处理
- 测试终端状态恢复

### 7.3 测试工具

```rust
pub struct MockTerminal {
    input: Vec<u8>,
    output: Vec<u8>,
}

impl Terminal for MockTerminal {
    // Mock 实现
}
```

## 八、性能考虑

1. **零拷贝**：尽可能使用 `&str` 而不是 `String`
2. **延迟分配**：只在需要时分配内存
3. **批量渲染**：减少终端写入次数
4. **缓存**：缓存样式计算结果

## 九、总结

本设计遵循 Rust 最佳实践：

1. **类型安全**：充分利用类型系统
2. **零成本抽象**：使用 Trait 和泛型
3. **清晰的错误处理**：使用 `Result<T>` 和 `thiserror`
4. **模块化设计**：清晰的模块划分
5. **可组合性**：易于组合和扩展
6. **性能优先**：避免不必要的分配

实现时需要注意：
- 终端原始模式的正确管理（使用 Guard 模式）
- 键盘事件的准确解析
- 错误处理和资源清理
- 跨平台兼容性
