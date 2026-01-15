//! 文件大小显示格式化

/// 文件大小显示格式化 trait
///
/// 为数值类型提供文件大小格式化功能。
///
/// # 示例
///
/// ```
/// use workflow::util::format::SizeDisplay;
///
/// let size = 1024.to_size_string();
/// assert_eq!(size, "1.00 KB");
/// ```
pub trait SizeDisplay {
    /// 将字节数格式化为人类可读的格式（B, KB, MB, GB, TB）。
    ///
    /// # 返回值
    ///
    /// 格式化后的字符串，例如 "1.23 MB" 或 "1024 B"
    fn to_size_string(&self) -> String;
}

impl SizeDisplay for u64 {
    fn to_size_string(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = *self as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", self, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::format::DisplayFormatter;
    use rstest::rstest;

    // 基础格式化测试 - 覆盖所有单位
    #[rstest]
    #[case(0, "0 B")]
    #[case(1, "1 B")]
    #[case(512, "512 B")]
    #[case(1023, "1023 B")]
    #[case(1024, "1.00 KB")]
    #[case(1536, "1.50 KB")] // 1024 + 512
    #[case(2048, "2.00 KB")]
    #[case(1024 * 1023, "1023.00 KB")]
    #[case(1024 * 1024, "1.00 MB")]
    #[case(1024 * 1024 + 512 * 1024, "1.50 MB")]
    #[case(1024 * 1024 * 5, "5.00 MB")]
    #[case(1024 * 1024 * 1023, "1023.00 MB")]
    #[case(1024_u64.pow(3), "1.00 GB")]
    #[case(1024_u64.pow(3) + 512 * 1024_u64.pow(2), "1.50 GB")]
    #[case(1024_u64.pow(3) * 10, "10.00 GB")]
    #[case(1024_u64.pow(4), "1.00 TB")]
    #[case(1024_u64.pow(4) * 2, "2.00 TB")]
    #[case(1024_u64.pow(4) + 512 * 1024_u64.pow(3), "1.50 TB")]
    #[case(1048576, "1.00 MB")] // 1024^2
    #[case(1073741824, "1.00 GB")] // 1024^3
    #[case(1099511627776, "1.00 TB")] // 1024^4
    #[case(2147483648, "2.00 GB")] // 2 * 1024^3
    #[case(5368709120, "5.00 GB")] // 5 * 1024^3
    fn test_format_size_basic(#[case] bytes: u64, #[case] expected: &str) {
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    // 小数精度测试
    #[rstest]
    #[case(1024 + 256, "1.25 KB")] // 1.25 KB
    #[case(1024 + 102, "1.10 KB")] // 约1.10 KB
    #[case(1024 + 51, "1.05 KB")] // 约1.05 KB
    fn test_format_size_precision(#[case] bytes: u64, #[case] expected: &str) {
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    #[test]
    fn test_format_size_edge_cases() {
        // 测试边界值
        assert_eq!(
            DisplayFormatter::size(u64::MAX),
            format!("{:.2} TB", u64::MAX as f64 / 1024_f64.powi(4))
        );

        // 测试刚好达到下一个单位的值
        assert_eq!(DisplayFormatter::size(1024 - 1), "1023 B");
        assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
        assert_eq!(DisplayFormatter::size(1024 * 1024 - 1), "1024.00 KB");
        assert_eq!(DisplayFormatter::size(1024 * 1024), "1.00 MB");
    }
}
