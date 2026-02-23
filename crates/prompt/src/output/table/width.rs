//! 列宽计算

use unicode_width::UnicodeWidthStr;

use crate::output::table::{builder::TableBuilder, strip_ansi_codes};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::table::builder::TableBuilder;

    #[test]
    fn test_calculate_column_widths_basic() {
        let builder = TableBuilder::new(vec!["Name", "Age"]).add_row(vec!["Alice", "30"]);
        let widths = calculate_column_widths(&builder);
        assert_eq!(widths.len(), 2);
        assert_eq!(widths[0], 5); // "Alice" 比 "Name" 长
        assert_eq!(widths[1], 3); // "Age" 比 "30" 长
    }

    #[test]
    fn test_calculate_column_widths_header_longer() {
        let builder = TableBuilder::new(vec!["Username", "ID"]).add_row(vec!["Bob", "1"]);
        let widths = calculate_column_widths(&builder);
        assert_eq!(widths[0], 8); // "Username" 最长
        assert_eq!(widths[1], 2); // "ID" 最长
    }

    #[test]
    fn test_calculate_column_widths_data_longer() {
        let builder =
            TableBuilder::new(vec!["A", "B"]).add_row(vec!["VeryLongValue", "AnotherLongValue"]);
        let widths = calculate_column_widths(&builder);
        assert_eq!(widths[0], 13); // "VeryLongValue"
        assert_eq!(widths[1], 16); // "AnotherLongValue"
    }

    #[test]
    fn test_calculate_column_widths_multiple_rows() {
        let builder = TableBuilder::new(vec!["Col"])
            .add_row(vec!["Short"])
            .add_row(vec!["VeryLongContent"])
            .add_row(vec!["Med"]);
        let widths = calculate_column_widths(&builder);
        assert_eq!(widths[0], 15); // "VeryLongContent" 最长
    }

    #[test]
    fn test_calculate_column_widths_empty_cells() {
        let builder = TableBuilder::new(vec!["A", "B"]).add_row(vec!["", ""]);
        let widths = calculate_column_widths(&builder);
        // 最小宽度为 1
        assert_eq!(widths[0], 1);
        assert_eq!(widths[1], 1);
    }

    #[test]
    fn test_calculate_column_widths_unicode() {
        let builder = TableBuilder::new(vec!["名字", "年龄"]).add_row(vec!["张三", "30"]);
        let widths = calculate_column_widths(&builder);
        // 中文字符宽度为 2
        assert_eq!(widths[0], 4); // "名字" = 2*2 = 4
        assert_eq!(widths[1], 4); // "年龄" = 2*2 = 4
    }

    #[test]
    fn test_calculate_column_widths_with_max_width() {
        let builder = TableBuilder::new(vec!["VeryLongHeaderOne", "VeryLongHeaderTwo"])
            .add_row(vec!["Data1", "Data2"])
            .with_max_width(30);
        let widths = calculate_column_widths(&builder);
        // 列宽应该被压缩以适应最大宽度
        let total: usize = widths.iter().sum();
        // 考虑边框和分隔符宽度
        assert!(total <= 30);
    }

    #[test]
    fn test_calculate_column_widths_minimum_width() {
        let builder = TableBuilder::new(vec!["", ""]).add_row(vec!["", ""]);
        let widths = calculate_column_widths(&builder);
        // 每列最小宽度为 1
        for width in &widths {
            assert!(*width >= 1);
        }
    }

    #[test]
    fn test_calculate_column_widths_single_column() {
        let builder = TableBuilder::new(vec!["Header"]).add_row(vec!["Data"]);
        let widths = calculate_column_widths(&builder);
        assert_eq!(widths.len(), 1);
        assert_eq!(widths[0], 6); // "Header"
    }

    #[test]
    fn test_calculate_column_widths_many_columns() {
        let builder =
            TableBuilder::new(vec!["A", "B", "C", "D", "E"]).add_row(vec!["1", "2", "3", "4", "5"]);
        let widths = calculate_column_widths(&builder);
        assert_eq!(widths.len(), 5);
        for width in &widths {
            assert_eq!(*width, 1);
        }
    }
}
