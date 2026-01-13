//! 表格渲染模块

use crate::base::interactive::style::get_theme;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

/// 对齐方式
#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

/// 表格构建器
pub struct TableBuilder {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    border: bool,
    row_line: bool,
    alignment: Alignment,
}

impl TableBuilder {
    pub fn new(headers: Vec<impl Into<String>>) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
            border: true,
            row_line: true,
            alignment: Alignment::Left,
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

    /// 渲染表格
    pub fn render(&self) {
        if self.headers.is_empty() {
            return;
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

        let mut stdout = std::io::stdout();

        // 顶部边框
        if self.border {
            let top_border =
                self.render_border(&col_widths, horizontal, top_left, top_right, top_cross);
            let styled = theme.hint.apply(&top_border, theme.enable_color);
            writeln!(stdout, "{}", styled).ok();
        }

        // 表头
        let header_row = self.render_row(&self.headers, &col_widths, true, &theme);
        let header_line = if self.border {
            let v = theme.hint.apply(vertical, theme.enable_color);
            format!("{} {} {}", v, header_row, v)
        } else {
            format!(" {} ", header_row)
        };
        writeln!(stdout, "{}", header_line).ok();

        // 表头分隔线
        if self.border {
            let separator =
                self.render_separator(&col_widths, horizontal, cross, left_cross, right_cross);
            let styled = theme.hint.apply(&separator, theme.enable_color);
            writeln!(stdout, "{}", styled).ok();
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
            writeln!(stdout, "{}", row_line).ok();

            // 行分隔线（最后一行后不添加）
            if self.row_line && i < self.rows.len() - 1 && self.border {
                let separator =
                    self.render_separator(&col_widths, horizontal, cross, left_cross, right_cross);
                let styled = theme.hint.apply(&separator, theme.enable_color);
                writeln!(stdout, "{}", styled).ok();
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
            writeln!(stdout, "{}", styled).ok();
        }

        stdout.flush().ok();
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

            // 对齐处理（基于实际显示宽度）
            let aligned_cell = self.align_cell(cell, col_widths[i], actual_width);

            // 表头样式（在对齐后应用，这样不会影响宽度计算）
            let final_cell = if is_header && theme.enable_color {
                theme.prompt.apply(&aligned_cell, theme.enable_color)
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

    fn align_cell(&self, cell: String, target_width: usize, actual_width: usize) -> String {
        if actual_width >= target_width {
            return cell;
        }

        let padding = target_width - actual_width;
        match self.alignment {
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
