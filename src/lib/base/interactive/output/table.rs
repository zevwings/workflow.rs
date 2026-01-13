//! 表格渲染模块

use crate::base::interactive::style::get_theme;
use unicode_width::UnicodeWidthStr;

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
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    border: bool,
    row_line: bool,
    alignment: Alignment,
    title: Option<String>,
    max_width: Option<usize>,
    column_alignments: Vec<Alignment>,
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
        if self.headers.is_empty() {
            return String::new();
        }

        let theme = get_theme();
        let col_widths = self.calculate_column_widths();

        // 边框字符
        let (
            vertical,
            horizontal,
            cross,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            top_cross,
            bottom_cross,
            left_cross,
            right_cross,
        ) = if self.border {
            ("│", "─", "┼", "┌", "┐", "└", "┘", "┬", "┴", "├", "┤")
        } else {
            (" ", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ")
        };

        let mut output = Vec::new();

        // 顶部边框
        if self.border {
            let top_border =
                self.render_border(&col_widths, horizontal, top_left, top_right, top_cross);
            let styled = theme.hint.apply(&top_border, theme.enable_color);
            output.push(styled);
        }

        // 标题行（如果有）
        if let Some(ref title) = self.title {
            let title_line = if self.border {
                let v = theme.hint.apply(vertical, theme.enable_color);
                // 计算标题的显示宽度（去除 ANSI 代码）
                let clean_title = strip_ansi_codes(title);
                let title_width = clean_title.width();
                let total_width: usize =
                    col_widths.iter().sum::<usize>() + (col_widths.len() - 1) * 3; // 列之间的分隔符宽度
                let padding = if total_width > title_width {
                    (total_width - title_width) / 2
                } else {
                    0
                };
                let centered_title = format!("{}{}", " ".repeat(padding), title);
                let styled_title = if theme.enable_color {
                    theme.title.apply(&centered_title, theme.enable_color)
                } else {
                    centered_title
                };
                format!("{} {} {}", v, styled_title, v)
            } else {
                format!(" {}", title)
            };
            output.push(title_line);

            // 标题行下方的分隔线
            if self.border {
                let separator = self.render_separator(
                    &col_widths,
                    horizontal,
                    "┬", // 使用 ┬ 而不是 ┼
                    left_cross,
                    right_cross,
                );
                let styled = theme.hint.apply(&separator, theme.enable_color);
                output.push(styled);
            }
        }

        // 表头
        let header_row = self.render_row(&self.headers, &col_widths, true, &theme);
        let header_line = if self.border {
            let v = theme.hint.apply(vertical, theme.enable_color);
            format!("{} {} {}", v, header_row, v)
        } else {
            format!(" {} ", header_row)
        };
        output.push(header_line);

        // 表头分隔线
        if self.border {
            let separator =
                self.render_separator(&col_widths, horizontal, cross, left_cross, right_cross);
            let styled = theme.hint.apply(&separator, theme.enable_color);
            output.push(styled);
        }

        // 渲染数据行
        for (i, row) in self.rows.iter().enumerate() {
            let data_row = self.render_row(row, &col_widths, false, &theme);
            let row_line = if self.border {
                let v = theme.hint.apply(vertical, theme.enable_color);
                format!("{} {} {}", v, data_row, v)
            } else {
                format!(" {} ", data_row)
            };
            output.push(row_line);

            // 行分隔线（最后一行后不添加）
            if self.row_line && i < self.rows.len() - 1 && self.border {
                let separator =
                    self.render_separator(&col_widths, horizontal, cross, left_cross, right_cross);
                let styled = theme.hint.apply(&separator, theme.enable_color);
                output.push(styled);
            }
        }

        // 底部边框
        if self.border {
            let bottom_border = self.render_border(
                &col_widths,
                horizontal,
                bottom_left,
                bottom_right,
                bottom_cross,
            );
            let styled = theme.hint.apply(&bottom_border, theme.enable_color);
            output.push(styled);
        }

        output.join("\n")
    }

    fn calculate_column_widths(&self) -> Vec<usize> {
        let mut col_widths = vec![0; self.headers.len()];

        // 计算表头宽度（去除 ANSI 代码后计算实际显示宽度）
        for (i, header) in self.headers.iter().enumerate() {
            let clean_header = strip_ansi_codes(header);
            let width = clean_header.width();
            if width > col_widths[i] {
                col_widths[i] = width;
            }
        }

        // 计算数据行宽度
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    let clean_cell = strip_ansi_codes(cell);
                    let width = clean_cell.width();
                    if width > col_widths[i] {
                        col_widths[i] = width;
                    }
                }
            }
        }

        // 确保最小宽度为 1
        for width in &mut col_widths {
            if *width < 1 {
                *width = 1;
            }
        }

        // 应用最大宽度限制
        if let Some(max_width) = self.max_width {
            let border_width = if self.border { 2 } else { 0 }; // 左右边框宽度
                                                                // 列之间的分隔符宽度（无论是否有边框，分隔符宽度都相同）
            let separator_width = (col_widths.len().saturating_sub(1)) * 3;
            let total_width: usize =
                col_widths.iter().sum::<usize>() + border_width + separator_width;

            if total_width > max_width {
                // 计算需要减少的宽度
                let available_width = max_width.saturating_sub(border_width + separator_width);

                if available_width >= col_widths.len() {
                    // 按比例缩小各列
                    let current_total: usize = col_widths.iter().sum();
                    let scale = available_width as f64 / current_total as f64;

                    // 先按比例分配
                    let mut new_widths: Vec<usize> =
                        col_widths.iter().map(|&w| (w as f64 * scale).floor() as usize).collect();

                    // 确保每列至少宽度为 1
                    for w in &mut new_widths {
                        if *w < 1 {
                            *w = 1;
                        }
                    }

                    // 调整总宽度以匹配可用宽度
                    let mut new_total: usize = new_widths.iter().sum();
                    let mut diff = available_width as i64 - new_total as i64;

                    // 按比例分配剩余的宽度
                    while diff != 0 && new_total > 0 {
                        for w in &mut new_widths {
                            if diff > 0 {
                                *w += 1;
                                diff -= 1;
                                if diff == 0 {
                                    break;
                                }
                            } else if diff < 0 && *w > 1 {
                                *w -= 1;
                                diff += 1;
                                if diff == 0 {
                                    break;
                                }
                            }
                        }
                        new_total = new_widths.iter().sum();
                        diff = available_width as i64 - new_total as i64;
                    }

                    col_widths = new_widths;
                } else {
                    // 如果可用宽度太小，每列至少为 1
                    col_widths = vec![1; col_widths.len()];
                }
            }
        }

        col_widths
    }

    fn render_row(
        &self,
        row: &[String],
        col_widths: &[usize],
        is_header: bool,
        theme: &crate::base::interactive::style::Theme,
    ) -> String {
        let mut cells = Vec::new();

        for i in 0..col_widths.len() {
            let cell = if i < row.len() {
                row[i].clone()
            } else {
                String::new()
            };

            // 去除 ANSI 代码计算实际宽度，然后对齐
            let clean_cell = strip_ansi_codes(&cell);
            let actual_width = clean_cell.width();

            // 获取该列的对齐方式（如果指定了每列对齐，使用每列对齐；否则使用全局对齐）
            let alignment = if i < self.column_alignments.len() {
                self.column_alignments[i]
            } else {
                self.alignment
            };

            // 对齐处理（基于实际显示宽度）
            let aligned_cell = self.align_cell(cell, col_widths[i], actual_width, alignment);

            // 表头样式（在对齐后应用，这样不会影响宽度计算）
            let final_cell = if is_header && theme.enable_color {
                theme.title.apply(&aligned_cell, theme.enable_color)
            } else {
                aligned_cell
            };

            cells.push(final_cell);
        }

        // 根据是否有边框选择分隔符
        let separator = if !self.border {
            " │ ".to_string()
        } else if theme.enable_color {
            let v = theme.hint.apply("│", theme.enable_color);
            format!(" {} ", v)
        } else {
            " │ ".to_string()
        };

        cells.join(&separator)
    }

    fn align_cell(
        &self,
        cell: String,
        target_width: usize,
        actual_width: usize,
        alignment: Alignment,
    ) -> String {
        if actual_width >= target_width {
            return cell;
        }

        let padding = target_width - actual_width;
        match alignment {
            Alignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!("{}{}{}", " ".repeat(left_pad), cell, " ".repeat(right_pad))
            }
            Alignment::Right => {
                format!("{}{}", " ".repeat(padding), cell)
            }
            Alignment::Left => {
                format!("{}{}", cell, " ".repeat(padding))
            }
        }
    }

    fn render_border(
        &self,
        col_widths: &[usize],
        horizontal: &str,
        left: &str,
        right: &str,
        cross: &str,
    ) -> String {
        let parts: Vec<String> =
            col_widths.iter().map(|width| horizontal.repeat(*width + 2)).collect();
        format!("{}{}{}", left, parts.join(cross), right)
    }

    fn render_separator(
        &self,
        col_widths: &[usize],
        horizontal: &str,
        cross: &str,
        left: &str,
        right: &str,
    ) -> String {
        self.render_border(col_widths, horizontal, left, right, cross)
    }
}

/// 去除 ANSI 转义代码
fn strip_ansi_codes(s: &str) -> String {
    // 简单的 ANSI 代码去除（匹配 ESC[ ... m 格式）
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // 跳过 ANSI 转义序列
            if chars.peek() == Some(&'[') {
                chars.next(); // 跳过 '['
                              // 跳过数字和分号，直到找到 'm'
                while let Some(&ch) = chars.peek() {
                    if ch == 'm' {
                        chars.next();
                        break;
                    } else if ch.is_ascii_digit() || ch == ';' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// 便捷函数
pub fn table(headers: Vec<impl Into<String>>) -> TableBuilder {
    TableBuilder::new(headers)
}

// 支持 Tabled trait 的适配器
impl TableBuilder {
    /// 从实现了 Tabled trait 的数据创建表格构建器
    ///
    /// # 参数
    ///
    /// * `data` - 要显示的数据，必须实现 `Tabled` trait
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use tabled::Tabled;
    /// use workflow::base::interactive::output::TableBuilder;
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
    /// let table = TableBuilder::from_tabled(users)
    ///     .with_title("Users")
    ///     .with_style(TableStyle::Modern)
    ///     .render();
    /// ```
    pub fn from_tabled<T: tabled::Tabled>(data: Vec<T>) -> Self {
        if data.is_empty() {
            return Self {
                headers: Vec::new(),
                rows: Vec::new(),
                border: true,
                row_line: true,
                alignment: Alignment::Left,
                title: None,
                max_width: None,
                column_alignments: Vec::new(),
            };
        }

        // 优化：只创建一次完整的表格，而不是为每个项目创建临时表格
        // 使用 ASCII 样式以确保解析的一致性
        let mut full_table = tabled::Table::new(&data);
        full_table.with(tabled::settings::Style::ascii());
        let table_str = format!("{}", full_table);
        let lines: Vec<&str> = table_str.lines().collect();

        // 从表头行提取列名（通常是第二行，第一行是顶部边框）
        let headers = if lines.len() >= 2 {
            let header_line = lines[1];
            header_line
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        // 从完整的表格中提取所有数据行
        // 表格结构（ASCII 样式）：
        // - 第0行：顶部边框（如 "+---+---+"）
        // - 第1行：表头（如 "| Header1 | Header2 |"）
        // - 第2行：表头分隔线（如 "+---+---+"）
        // - 第3行开始：数据行（如 "| Data1 | Data2 |"）
        // - 行分隔线（如 "+---+---+"）
        // - 最后一行：底部边框（如 "+---+---+"）
        let rows: Vec<Vec<String>> = lines
            .iter()
            .skip(3) // 跳过顶部边框、表头、表头分隔线
            .filter_map(|line| {
                // 跳过边框和分隔线（ASCII 样式使用 '+', '-', '|'）
                // 检查是否是边框/分隔线：只包含 '+', '-', '|', 空格，且包含 '+'
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }
                // 如果包含 '+'，说明是边框或分隔线
                if trimmed.contains('+') {
                    return None;
                }
                // 如果只包含 '-' 和 '|'，也是分隔线
                if trimmed.chars().all(|c| c == '-' || c == '|' || c.is_whitespace()) {
                    return None;
                }
                // 解析数据行（应该包含 '|' 分隔符）
                if !trimmed.contains('|') {
                    return None;
                }
                let row: Vec<String> = trimmed
                    .split('|')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if row.is_empty() {
                    None
                } else {
                    Some(row)
                }
            })
            .collect();

        Self {
            headers,
            rows,
            border: true,
            row_line: true,
            alignment: Alignment::Left,
            title: None,
            max_width: None,
            column_alignments: Vec::new(),
        }
    }
}
