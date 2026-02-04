//! 列宽计算

use crate::output::table::builder::TableBuilder;
use crate::output::table::strip_ansi_codes;
use unicode_width::UnicodeWidthStr;

/// 计算列宽
pub(super) fn calculate_column_widths(builder: &TableBuilder) -> Vec<usize> {
    let mut col_widths = vec![0; builder.headers.len()];

    // 计算表头宽度（去除 ANSI 代码后计算实际显示宽度）
    for (i, header) in builder.headers.iter().enumerate() {
        let clean_header = strip_ansi_codes(header);
        let width = clean_header.width();
        if width > col_widths[i] {
            col_widths[i] = width;
        }
    }

    // 计算数据行宽度
    for row in &builder.rows {
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
    if let Some(max_width) = builder.max_width {
        let border_width = if builder.border { 2 } else { 0 }; // 左右边框宽度
                                                               // 列之间的分隔符宽度（无论是否有边框，分隔符宽度都相同）
        let separator_width = (col_widths.len().saturating_sub(1)) * 3;
        let total_width: usize = col_widths.iter().sum::<usize>() + border_width + separator_width;

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
