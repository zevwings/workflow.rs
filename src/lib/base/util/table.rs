//! 表格输出工具
//!
//! 提供统一的表格输出接口，使用 tabled 库。
//!
//! ## 功能特性
//!
//! - 自动格式化表格
//! - 支持自定义样式和边框
//! - 支持列对齐和宽度控制
//! - 支持紧凑模式和完整模式
//! - 支持链式配置
//!
//! ## 使用示例
//!
//! ```rust
//! use tabled::Tabled;
//! use workflow::base::util::{TableBuilder, TableStyle};
//!
//! #[derive(Tabled, Clone)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let users = vec![
//!     User { name: "Alice".to_string(), age: 30 },
//!     User { name: "Bob".to_string(), age: 25 },
//! ];
//!
//! // 链式调用方式
//! let output = TableBuilder::new(users.clone())
//!     .with_title("Users")
//!     .with_style(TableStyle::Modern)
//!     .render();
//! println!("{}", output);
//!
//! // 或者使用 Display trait
//! println!("{}", TableBuilder::new(users));
//! ```

use std::fmt;

use tabled::{
    settings::{object::Columns, object::Rows, panel::Panel, Alignment, Modify, Style, Width},
    Table, Tabled,
};

/// 表格样式配置
#[derive(Debug, Clone, Copy)]
pub enum TableStyle {
    /// 默认样式（ASCII）
    Default,
    /// 现代样式（带边框）
    Modern,
    /// 紧凑样式（无边框）
    Compact,
    /// 最小样式（仅分隔符）
    Minimal,
    /// 网格样式（完整网格）
    Grid,
}

impl TableStyle {
    /// 将样式应用到 Table
    fn apply_to_table(&self, table: &mut Table) {
        match self {
            TableStyle::Default => {
                table.with(Style::ascii());
            }
            TableStyle::Modern => {
                table.with(Style::modern());
            }
            TableStyle::Compact => {
                table.with(Style::rounded());
            }
            TableStyle::Minimal => {
                table.with(Style::blank());
            }
            TableStyle::Grid => {
                table.with(Style::rounded());
            }
        }
    }
}

/// 修复表格边框格式
/// 1. 顶部边框：除了 ┌ 和 ┐ 之外，中间都应该是 ─
/// 2. 标题行下方的分隔线：从 ├─┼─┼─┤ 格式改为 ├─┬─┬─┤ 格式
fn fix_title_separator(table_output: String) -> String {
    let lines: Vec<&str> = table_output.lines().collect();
    if lines.len() < 4 {
        return table_output;
    }

    // 表格结构：
    // 第0行：顶部边框 ┌─────────────────────────────┐ (需要修复，确保中间都是 ─)
    // 第1行：标题行   │              title          │
    // 第2行：分隔线   ├─┼─┼─┤ (需要改为 ├─┬─┬─┤)
    // 第3行：列标题行
    // 第4行：列标题分隔线（保持原样）

    let mut result = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let mut fixed_line = line.to_string();

        // 修复顶部边框（第0行）：确保除了 ┌ 和 ┐ 之外，中间都是 ─
        if i == 0 && line.starts_with('┌') && line.ends_with('┐') {
            // 将除了第一个字符 ┌ 和最后一个字符 ┐ 之外的所有字符替换为 ─
            let chars: Vec<char> = line.chars().collect();
            if chars.len() >= 2 {
                // 保留第一个字符 ┌
                let first = chars[0];
                // 保留最后一个字符 ┐
                let last = chars[chars.len() - 1];
                // 中间全部替换为 ─
                fixed_line = format!("{}{}{}", first, "─".repeat(chars.len() - 2), last);
            }
        }
        // 修复标题行下方的分隔线（第2行）：将 ┼ 替换为 ┬
        else if i == 2 && line.starts_with('├') && line.ends_with('┤') && line.contains('┼')
        {
            fixed_line = fixed_line.replace('┼', "┬");
        }

        result.push(fixed_line);
    }

    result.join("\n")
}

/// 表格构建器
///
/// 提供链式配置和输出表格的功能。
///
/// # 示例
///
/// ```rust
/// use tabled::Tabled;
/// use workflow::base::util::{TableBuilder, TableStyle};
///
/// #[derive(Tabled)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let users = vec![
///     User { name: "Alice".to_string(), age: 30 },
/// ];
///
/// // 链式配置并渲染
/// let output = TableBuilder::new(users)
///     .with_title("Users List")
///     .with_style(TableStyle::Modern)
///     .with_max_width(80)
///     .render();
/// println!("{}", output);
/// ```
pub struct TableBuilder<T> {
    data: Vec<T>,
    title: Option<String>,
    style: Option<TableStyle>,
    max_width: Option<usize>,
    alignments: Vec<Alignment>,
}

impl<T: Tabled> TableBuilder<T> {
    /// 创建新的表格构建器
    ///
    /// # 参数
    ///
    /// * `data` - 要显示的数据，必须实现 `Tabled` trait
    ///
    /// # 示例
    ///
    /// ```rust
    /// use tabled::Tabled;
    /// use workflow::base::util::TableBuilder;
    ///
    /// #[derive(Tabled)]
    /// struct Item {
    ///     name: String,
    /// }
    ///
    /// let items = vec![Item { name: "Test".to_string() }];
    /// let builder = TableBuilder::new(items);
    /// ```
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data,
            title: None,
            style: None,
            max_width: None,
            alignments: Vec::new(),
        }
    }

    /// 设置表格标题
    ///
    /// # 参数
    ///
    /// * `title` - 表格标题
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::util::TableBuilder;
    /// # let data = vec![("name", "value")];
    /// TableBuilder::new(data).with_title("My Table");
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置表格样式
    ///
    /// # 参数
    ///
    /// * `style` - 表格样式
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::util::{TableBuilder, TableStyle};
    /// # let data = vec![("name", "value")];
    /// TableBuilder::new(data).with_style(TableStyle::Modern);
    /// ```
    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// 设置最大宽度（自动换行）
    ///
    /// # 参数
    ///
    /// * `width` - 最大宽度
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::util::TableBuilder;
    /// # let data = vec![("name", "value")];
    /// TableBuilder::new(data).with_max_width(80);
    /// ```
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    /// 设置列对齐方式
    ///
    /// # 参数
    ///
    /// * `alignments` - 每列的对齐方式，按列索引顺序
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use tabled::settings::Alignment;
    /// use workflow::base::util::TableBuilder;
    /// # let data = vec![("name", "value")];
    /// TableBuilder::new(data)
    ///     .with_alignment(vec![Alignment::left(), Alignment::right()]);
    /// ```
    pub fn with_alignment(mut self, alignments: Vec<Alignment>) -> Self {
        self.alignments = alignments;
        self
    }

    /// 构建并渲染表格为字符串
    ///
    /// # 返回
    ///
    /// 返回格式化后的表格字符串。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::util::TableBuilder;
    /// use workflow::log_message;
    /// # let data = vec![("name", "value")];
    /// let output = TableBuilder::new(data).render();
    /// log_message!("{}", output);
    /// ```
    pub fn render(self) -> String {
        if self.data.is_empty() {
            if let Some(ref title) = self.title {
                return format!("{}\n(No data)", title);
            }
            return String::new();
        }

        // 构建表格
        let mut table = Table::new(&self.data);

        // 应用样式（边框）
        if let Some(style) = self.style {
            style.apply_to_table(&mut table);
        }

        // 添加标题行（在边框内）
        if let Some(ref title) = self.title {
            table.with(Panel::header(title));
            // 设置标题行居中对齐
            table.with(Modify::new(Rows::first()).with(Alignment::center()));
        }

        // 应用最大宽度
        if let Some(width) = self.max_width {
            table.with(Width::wrap(width));
        }

        // 应用列对齐
        for (col_idx, alignment) in self.alignments.iter().enumerate() {
            table.with(Modify::new(Columns::single(col_idx)).with(*alignment));
        }

        // 渲染表格并修复标题行下方的分隔线
        let table_output = format!("{}", table);
        if self.title.is_some() {
            fix_title_separator(table_output)
        } else {
            table_output
        }
    }
}

impl<T: Tabled> fmt::Display for TableBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.is_empty() {
            return Ok(());
        }

        let mut table = Table::new(&self.data);

        // 应用样式（边框）
        if let Some(style) = self.style {
            style.apply_to_table(&mut table);
        }

        // 添加标题行（在边框内）
        if let Some(ref title) = self.title {
            table.with(Panel::header(title));
            // 设置标题行居中对齐
            table.with(Modify::new(Rows::first()).with(Alignment::center()));
        }

        // 应用最大宽度
        if let Some(width) = self.max_width {
            table.with(Width::wrap(width));
        }

        // 应用列对齐
        for (col_idx, alignment) in self.alignments.iter().enumerate() {
            table.with(Modify::new(Columns::single(col_idx)).with(*alignment));
        }

        // 渲染表格并修复标题行下方的分隔线
        let table_output = format!("{}", table);
        let fixed_output = if self.title.is_some() {
            fix_title_separator(table_output)
        } else {
            table_output
        };

        write!(f, "{}", fixed_output)
    }
}
