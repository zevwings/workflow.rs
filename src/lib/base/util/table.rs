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
//! use crate::base::util::table::TableBuilder;
//!
//! #[derive(Tabled)]
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
//! TableBuilder::new(users)
//!     .with_title("Users")
//!     .with_style(TableStyle::Modern)
//!     .print();
//!
//! // 或者使用便捷方法
//! TableBuilder::new(users).print();
//! ```

use std::fmt;

use tabled::{
    settings::{object::Columns, Alignment, Modify, Style, Width},
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

/// 表格构建器
///
/// 提供链式配置和输出表格的功能。
///
/// # 示例
///
/// ```rust
/// use tabled::Tabled;
/// use crate::base::util::table::{TableBuilder, TableStyle};
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
/// // 链式配置
/// TableBuilder::new(users)
///     .with_title("Users List")
///     .with_style(TableStyle::Modern)
///     .with_max_width(80)
///     .print();
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
    /// use crate::base::util::table::TableBuilder;
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
    /// ```rust
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
    /// ```rust
    /// use crate::base::util::table::{TableBuilder, TableStyle};
    ///
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
    /// ```rust
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
    /// ```rust
    /// use tabled::settings::Alignment;
    ///
    /// TableBuilder::new(data)
    ///     .with_alignment(vec![Alignment::left(), Alignment::right()]);
    /// ```
    pub fn with_alignment(mut self, alignments: Vec<Alignment>) -> Self {
        self.alignments = alignments;
        self
    }

    /// 构建并打印表格
    ///
    /// # 示例
    ///
    /// ```rust
    /// TableBuilder::new(data).print();
    /// ```
    pub fn print(self) {
        if self.data.is_empty() {
            if let Some(ref title) = self.title {
                println!("{}\n(No data)", title);
            }
            return;
        }

        // 打印标题
        if let Some(ref title) = self.title {
            println!("{}\n", title);
        }

        // 构建表格
        let mut table = Table::new(&self.data);

        // 应用样式
        if let Some(style) = self.style {
            style.apply_to_table(&mut table);
        }

        // 应用最大宽度
        if let Some(width) = self.max_width {
            table.with(Width::wrap(width));
        }

        // 应用对齐
        for (col_idx, alignment) in self.alignments.iter().enumerate() {
            table.with(Modify::new(Columns::single(col_idx)).with(*alignment));
        }

        println!("{}", table);
    }
}

impl<T: Tabled> fmt::Display for TableBuilder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.is_empty() {
            return Ok(());
        }

        let mut table = Table::new(&self.data);

        // 应用样式
        if let Some(style) = self.style {
            style.apply_to_table(&mut table);
        }

        // 应用最大宽度
        if let Some(width) = self.max_width {
            table.with(Width::wrap(width));
        }

        // 应用对齐
        for (col_idx, alignment) in self.alignments.iter().enumerate() {
            table.with(Modify::new(Columns::single(col_idx)).with(*alignment));
        }

        write!(f, "{}", table)
    }
}
