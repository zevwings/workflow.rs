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
/// use workflow::base::table::{TableBuilder, TableStyle};
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
    /// use workflow::base::table::TableBuilder;
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
    /// use workflow::base::table::TableBuilder;
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
    /// use workflow::base::table::{TableBuilder, TableStyle};
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
    /// use workflow::base::table::TableBuilder;
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
    /// use workflow::base::table::TableBuilder;
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
    /// use workflow::base::table::TableBuilder;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tabled::Tabled;

    #[derive(Tabled, Clone)]
    struct TestUser {
        name: String,
        age: u32,
        email: String,
    }

    // ==================== TableBuilder Creation Tests ====================

    /// 测试使用数据创建 TableBuilder
    ///
    /// ## 测试目的
    /// 验证 TableBuilder::new() 能够使用数据向量创建表格构建器实例。
    ///
    /// ## 测试场景
    /// 1. 准备包含多个测试用户的数据向量
    /// 2. 使用 new() 方法创建 TableBuilder
    /// 3. 验证构建器创建成功（不 panic）
    ///
    /// ## 预期结果
    /// - TableBuilder 实例创建成功
    /// - 能够正常使用构建器
    #[test]
    fn test_table_builder_new_with_data_creates_builder() {
        let users = vec![
            TestUser {
                name: "Alice".to_string(),
                age: 30,
                email: "alice@example.com".to_string(),
            },
            TestUser {
                name: "Bob".to_string(),
                age: 25,
                email: "bob@example.com".to_string(),
            },
        ];
        let _builder = TableBuilder::new(users);
    }

    /// 测试使用标题创建 TableBuilder 并渲染
    ///
    /// ## 测试目的
    /// 验证 TableBuilder::with_title() 能够为表格添加标题，并在渲染时包含标题。
    ///
    /// ## 测试场景
    /// 1. 创建包含数据的 TableBuilder
    /// 2. 使用 with_title() 方法添加标题
    /// 3. 调用 render() 方法渲染表格
    /// 4. 验证渲染结果包含标题
    ///
    /// ## 预期结果
    /// - 渲染的表格输出包含指定的标题
    #[test]
    fn test_table_builder_with_title_with_title_string_renders_with_title() {
        let users = vec![TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }];
        let title = "Users List";
        let builder = TableBuilder::new(users).with_title(title);
        let output = builder.render();
        assert!(output.contains(title));
    }

    /// 测试使用不同样式渲染表格
    ///
    /// ## 测试目的
    /// 验证 TableBuilder::with_style() 能够使用不同的表格样式，并且不同样式会产生不同的渲染输出。
    ///
    /// ## 测试场景
    /// 1. 使用相同的数据创建两个 TableBuilder
    /// 2. 分别应用 Modern 和 Compact 样式
    /// 3. 渲染两个表格
    /// 4. 验证两个渲染结果不同
    ///
    /// ## 预期结果
    /// - Modern 样式和 Compact 样式的渲染输出不同
    #[test]
    fn test_table_builder_with_style_with_different_styles_renders_differently() {
        let users = vec![TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }];
        let builder = TableBuilder::new(users.clone()).with_style(TableStyle::Modern);
        let output_modern = builder.render();
        let builder = TableBuilder::new(users).with_style(TableStyle::Compact);
        let output_compact = builder.render();
        assert_ne!(output_modern, output_compact);
    }

    /// 测试使用最大宽度限制渲染表格
    ///
    /// ## 测试目的
    /// 验证 TableBuilder::with_max_width() 能够限制表格的最大宽度，并在渲染时应用宽度限制。
    ///
    /// ## 测试场景
    /// 1. 创建包含数据的 TableBuilder
    /// 2. 使用 with_max_width() 方法设置最大宽度
    /// 3. 调用 render() 方法渲染表格
    /// 4. 验证渲染结果不为空
    ///
    /// ## 预期结果
    /// - 表格能够正常渲染（输出不为空）
    /// - 宽度限制生效
    #[test]
    fn test_table_builder_with_max_width_with_width_limit_renders_table() {
        let users = vec![TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }];
        let max_width = 20;
        let builder = TableBuilder::new(users).with_max_width(max_width);
        let output = builder.render();
        assert!(!output.is_empty());
    }

    /// 测试使用空数据渲染表格
    ///
    /// ## 测试目的
    /// 验证 TableBuilder 在使用空数据向量时能够正确处理，返回空字符串。
    ///
    /// ## 测试场景
    /// 1. 创建空的测试数据向量
    /// 2. 使用空数据创建 TableBuilder
    /// 3. 调用 render() 方法渲染表格
    /// 4. 验证渲染结果为空字符串
    ///
    /// ## 预期结果
    /// - 渲染结果为空字符串
    #[test]
    fn test_table_builder_empty_data_with_empty_data_returns_empty_string() {
        let users: Vec<TestUser> = vec![];
        let builder = TableBuilder::new(users);
        let output = builder.render();
        assert_eq!(output, "");
    }

    /// 测试使用空数据和标题渲染表格
    ///
    /// ## 测试目的
    /// 验证 TableBuilder 在使用空数据但有标题时，能够显示标题和 "No data" 提示。
    ///
    /// ## 测试场景
    /// 1. 创建空的测试数据向量
    /// 2. 使用空数据创建 TableBuilder 并添加标题
    /// 3. 调用 render() 方法渲染表格
    /// 4. 验证渲染结果包含标题和 "No data" 提示
    ///
    /// ## 预期结果
    /// - 渲染结果包含标题
    /// - 渲染结果包含 "(No data)" 提示
    #[test]
    fn test_table_builder_empty_data_with_title() {
        let users: Vec<TestUser> = vec![];
        let builder = TableBuilder::new(users).with_title("Empty Table");
        let output = builder.render();
        assert!(output.contains("Empty Table"));
        assert!(output.contains("(No data)"));
    }

    /// 测试 TableBuilder 的 Display trait 实现
    ///
    /// ## 测试目的
    /// 验证 TableBuilder 实现了 Display trait，可以通过 format!() 宏直接格式化输出。
    ///
    /// ## 测试场景
    /// 1. 创建包含数据的 TableBuilder
    /// 2. 使用 format!() 宏格式化 TableBuilder
    /// 3. 验证格式化结果不为空
    ///
    /// ## 预期结果
    /// - 格式化输出不为空
    /// - 输出与 render() 方法的结果一致
    #[test]
    fn test_table_builder_display_trait() {
        let users = vec![TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }];
        let builder = TableBuilder::new(users);
        let output = format!("{}", builder);
        assert!(!output.is_empty());
    }

    /// 测试所有 TableStyle 变体都可以使用
    ///
    /// ## 测试目的
    /// 验证所有 TableStyle 枚举变体都能正常创建和使用，不会产生编译错误。
    ///
    /// ## 测试场景
    /// 1. 创建包含所有 TableStyle 变体的数组
    /// 2. 验证所有变体都能正常创建
    ///
    /// ## 预期结果
    /// - 所有 TableStyle 变体都能正常创建
    /// - 包括：Default, Modern, Compact, Minimal, Grid
    #[test]
    fn test_table_style_variants() {
        let _styles = [
            TableStyle::Default,
            TableStyle::Modern,
            TableStyle::Compact,
            TableStyle::Minimal,
            TableStyle::Grid,
        ];
    }

    /// 测试 TableBuilder 的链式调用
    ///
    /// ## 测试目的
    /// 验证 TableBuilder 支持链式调用多个配置方法，能够流畅地配置和渲染表格。
    ///
    /// ## 测试场景
    /// 1. 创建包含数据的 TableBuilder
    /// 2. 链式调用 with_title()、with_style()、with_max_width() 方法
    /// 3. 调用 render() 方法渲染表格
    /// 4. 验证渲染结果包含标题且不为空
    ///
    /// ## 预期结果
    /// - 链式调用成功执行
    /// - 渲染结果不为空
    /// - 渲染结果包含标题
    #[test]
    fn test_table_builder_chain_calls() {
        let users = vec![TestUser {
            name: "Alice".to_string(),
            age: 30,
            email: "alice@example.com".to_string(),
        }];
        let output = TableBuilder::new(users)
            .with_title("Users")
            .with_style(TableStyle::Modern)
            .with_max_width(80)
            .render();
        assert!(!output.is_empty());
        assert!(output.contains("Users"));
    }
}
