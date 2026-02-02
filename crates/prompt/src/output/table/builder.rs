//! 表格构建器
//!
//! 提供表格构建和渲染功能

use crate::error::{PromptError, Result};
use std::io::Write;

use crate::output::table::render::render;

/// 对齐方式
#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

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

/// 表格构建器
pub struct TableBuilder {
    pub(crate) headers: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
    pub(crate) border: bool,
    pub(crate) row_line: bool,
    pub(crate) alignment: Alignment,
    pub(crate) title: Option<String>,
    pub(crate) max_width: Option<usize>,
    pub(crate) column_alignments: Vec<Alignment>,
}

impl TableBuilder {
    pub fn new(headers: Vec<impl Into<String>>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
            border: true,
            row_line: true,
            alignment: Alignment::Left,
            title: None,
            max_width: None,
            column_alignments: Vec::new(),
        }
    }

    pub fn add_row(mut self, row: Vec<impl Into<String>>) -> Self {
        self.rows.push(row.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn with_row_line(mut self, row_line: bool) -> Self {
        self.row_line = row_line;
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// 设置表格标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置表格样式
    pub fn with_style(mut self, style: TableStyle) -> Self {
        match style {
            TableStyle::Default | TableStyle::Modern | TableStyle::Grid => {
                self.border = true;
                self.row_line = true;
            }
            TableStyle::Compact => {
                self.border = false;
                self.row_line = false;
            }
            TableStyle::Minimal => {
                self.border = false;
                self.row_line = false;
            }
        }
        self
    }

    /// 设置最大宽度（自动换行）
    ///
    /// # 参数
    ///
    /// * `width` - 最大宽度，如果表格宽度超过此值，将按比例缩小各列
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }

    /// 设置每列的对齐方式
    ///
    /// # 参数
    ///
    /// * `alignments` - 每列的对齐方式，按列索引顺序。如果提供的对齐方式少于列数，剩余的列将使用默认对齐方式。
    pub fn with_column_alignments(mut self, alignments: Vec<Alignment>) -> Self {
        self.column_alignments = alignments;
        self
    }

    /// 渲染表格并返回字符串
    pub fn render(&self) -> String {
        render(self)
    }

    /// 渲染并打印表格到标准输出
    ///
    /// 这个方法会直接打印表格，不需要手动调用 `println!`。
    ///
    /// # 错误处理
    ///
    /// 如果写入标准输出时发生错误，会返回 `Result::Err`。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use prompt::{TableBuilder, TableStyle};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let table = TableBuilder::new(vec!["Name", "Age"])
    ///     .add_row(vec!["Alice", "30"])
    ///     .with_style(TableStyle::Modern);
    /// table.print()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn print(&self) -> Result<()> {
        let mut writer = std::io::stdout();
        writeln!(writer, "{}", self.render()).map_err(PromptError::Io)?;
        Ok(())
    }

    /// 渲染并显示表格到标准输出
    ///
    /// 这个方法会直接使用 `writeln!` 将表格输出到控制台。
    ///
    /// # 错误处理
    ///
    /// 如果写入标准输出时发生错误，会返回 `Result::Err`。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use prompt::TableBuilder;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let table = TableBuilder::new(vec!["Name", "Age"])
    ///     .add_row(vec!["Alice", "30"]);
    /// table.display()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn display(&self) -> Result<()> {
        let mut writer = std::io::stdout();
        writeln!(writer, "{}", self.render()).map_err(PromptError::Io)?;
        Ok(())
    }
}
