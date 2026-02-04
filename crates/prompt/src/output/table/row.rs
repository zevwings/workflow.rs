//! 表格行渲染

use crate::output::table::builder::{Alignment, TableBuilder};
use crate::output::table::strip_ansi_codes;
use crate::style::theme::Theme;
use unicode_width::UnicodeWidthStr;

/// 渲染表格行
pub(super) fn render_row(
    builder: &TableBuilder,
    row: &[String],
    col_widths: &[usize],
    is_header: bool,
    theme: &Theme,
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
        let alignment = if i < builder.column_alignments.len() {
            builder.column_alignments[i]
        } else {
            builder.alignment
        };

        // 对齐处理（基于实际显示宽度）
        let aligned_cell = align_cell(cell, col_widths[i], actual_width, alignment);

        // 表头样式（在对齐后应用，这样不会影响宽度计算）
        let final_cell = if is_header && theme.enable_color {
            theme.title.apply(&aligned_cell, theme.enable_color)
        } else {
            aligned_cell
        };

        cells.push(final_cell);
    }

    // 根据是否有边框选择分隔符
    let separator = if !builder.border {
        " │ ".to_string()
    } else if theme.enable_color {
        let v = theme.hint.apply("│", theme.enable_color);
        format!(" {} ", v)
    } else {
        " │ ".to_string()
    };

    cells.join(&separator)
}

/// 对齐单元格
pub(super) fn align_cell(
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
