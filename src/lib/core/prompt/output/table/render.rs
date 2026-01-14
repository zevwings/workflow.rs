//! 表格渲染主逻辑

use super::builder::TableBuilder;
use super::row::render_row;
use super::width::calculate_column_widths;
use crate::core::prompt::style::theme::get_theme;
use unicode_width::UnicodeWidthStr;

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

/// 渲染边框
fn render_border(
    col_widths: &[usize],
    horizontal: &str,
    left: &str,
    right: &str,
    cross: &str,
) -> String {
    let parts: Vec<String> = col_widths.iter().map(|width| horizontal.repeat(*width + 2)).collect();
    format!("{}{}{}", left, parts.join(cross), right)
}

/// 渲染分隔线
fn render_separator(
    col_widths: &[usize],
    horizontal: &str,
    cross: &str,
    left: &str,
    right: &str,
) -> String {
    render_border(col_widths, horizontal, left, right, cross)
}

/// 渲染表格并返回字符串
pub(super) fn render(builder: &TableBuilder) -> String {
    if builder.headers.is_empty() {
        return String::new();
    }

    let theme = get_theme();
    let col_widths = calculate_column_widths(builder);

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
    ) = if builder.border {
        ("│", "─", "┼", "┌", "┐", "└", "┘", "┬", "┴", "├", "┤")
    } else {
        (" ", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ")
    };

    let mut output = Vec::new();

    // 顶部边框
    if builder.border {
        let top_border = render_border(&col_widths, horizontal, top_left, top_right, top_cross);
        let styled = theme.hint.apply(&top_border, theme.enable_color);
        output.push(styled);
    }

    // 标题行（如果有）
    if let Some(ref title) = builder.title {
        let title_line = if builder.border {
            let v = theme.hint.apply(vertical, theme.enable_color);
            // 计算标题的显示宽度（去除 ANSI 代码）
            let clean_title = strip_ansi_codes(title);
            let title_width = clean_title.width();
            let total_width: usize = col_widths.iter().sum::<usize>() + (col_widths.len() - 1) * 3; // 列之间的分隔符宽度
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
        if builder.border {
            let separator = render_separator(
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
    let header_row = render_row(builder, &builder.headers, &col_widths, true, &theme);
    let header_line = if builder.border {
        let v = theme.hint.apply(vertical, theme.enable_color);
        format!("{} {} {}", v, header_row, v)
    } else {
        format!(" {} ", header_row)
    };
    output.push(header_line);

    // 表头分隔线
    if builder.border {
        let separator = render_separator(&col_widths, horizontal, cross, left_cross, right_cross);
        let styled = theme.hint.apply(&separator, theme.enable_color);
        output.push(styled);
    }

    // 渲染数据行
    for (i, row) in builder.rows.iter().enumerate() {
        let data_row = render_row(builder, row, &col_widths, false, &theme);
        let row_line = if builder.border {
            let v = theme.hint.apply(vertical, theme.enable_color);
            format!("{} {} {}", v, data_row, v)
        } else {
            format!(" {} ", data_row)
        };
        output.push(row_line);

        // 行分隔线（最后一行后不添加）
        if builder.row_line && i < builder.rows.len() - 1 && builder.border {
            let separator =
                render_separator(&col_widths, horizontal, cross, left_cross, right_cross);
            let styled = theme.hint.apply(&separator, theme.enable_color);
            output.push(styled);
        }
    }

    // 底部边框
    if builder.border {
        let bottom_border = render_border(
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
