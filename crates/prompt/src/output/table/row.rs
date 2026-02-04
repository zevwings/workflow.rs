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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_cell_left() {
        let result = align_cell("test".to_string(), 10, 4, Alignment::Left);
        assert_eq!(result, "test      ");
    }

    #[test]
    fn test_align_cell_right() {
        let result = align_cell("test".to_string(), 10, 4, Alignment::Right);
        assert_eq!(result, "      test");
    }

    #[test]
    fn test_align_cell_center() {
        let result = align_cell("test".to_string(), 10, 4, Alignment::Center);
        assert_eq!(result, "   test   ");
    }

    #[test]
    fn test_align_cell_center_odd_padding() {
        let result = align_cell("ab".to_string(), 7, 2, Alignment::Center);
        // 5 spaces to distribute: 2 left, 3 right
        assert_eq!(result, "  ab   ");
    }

    #[test]
    fn test_align_cell_exact_width() {
        let result = align_cell("test".to_string(), 4, 4, Alignment::Left);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_align_cell_exceeds_width() {
        let result = align_cell("toolong".to_string(), 4, 7, Alignment::Left);
        assert_eq!(result, "toolong"); // 不截断，原样返回
    }

    #[test]
    fn test_align_cell_empty_string() {
        let result = align_cell("".to_string(), 5, 0, Alignment::Center);
        assert_eq!(result, "     ");
    }

    #[test]
    fn test_align_cell_single_char() {
        let result = align_cell("X".to_string(), 5, 1, Alignment::Center);
        assert_eq!(result, "  X  ");
    }

    #[test]
    fn test_render_row_basic() {
        let builder = TableBuilder::new(vec!["Name", "Age"]).add_row(vec!["Alice", "30"]);
        let theme = crate::style::theme::get_theme();
        let col_widths = vec![5, 3];
        let row = render_row(
            &builder,
            &["Alice".to_string(), "30".to_string()],
            &col_widths,
            false,
            &theme,
        );
        assert!(row.contains("Alice"));
        assert!(row.contains("30"));
    }

    #[test]
    fn test_render_row_header() {
        let builder = TableBuilder::new(vec!["Name", "Age"]);
        let theme = crate::style::theme::get_theme();
        let col_widths = vec![4, 3];
        let row = render_row(
            &builder,
            &["Name".to_string(), "Age".to_string()],
            &col_widths,
            true,
            &theme,
        );
        assert!(row.contains("Name"));
        assert!(row.contains("Age"));
    }

    #[test]
    fn test_render_row_missing_cells() {
        let builder = TableBuilder::new(vec!["A", "B", "C"]);
        let theme = crate::style::theme::get_theme();
        let col_widths = vec![1, 1, 1];
        // 只提供 2 个单元格，但有 3 列
        let row = render_row(
            &builder,
            &["1".to_string(), "2".to_string()],
            &col_widths,
            false,
            &theme,
        );
        // 应该正常渲染，缺失的列用空字符串填充
        assert!(row.contains("1"));
        assert!(row.contains("2"));
    }

    #[test]
    fn test_render_row_with_column_alignments() {
        let builder = TableBuilder::new(vec!["A", "B", "C"]).with_column_alignments(vec![
            Alignment::Left,
            Alignment::Center,
            Alignment::Right,
        ]);
        let theme = crate::style::theme::get_theme();
        let col_widths = vec![5, 5, 5];
        let row = render_row(
            &builder,
            &["1".to_string(), "2".to_string(), "3".to_string()],
            &col_widths,
            false,
            &theme,
        );
        // 验证行被正确渲染
        assert!(row.contains("1"));
        assert!(row.contains("2"));
        assert!(row.contains("3"));
    }
}
