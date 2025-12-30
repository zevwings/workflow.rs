//! 显示格式化器模块
//!
//! 提供显示相关的格式化功能，包括路径、列表项、键值对和文件大小的格式化。

use std::path::Path;

/// 显示格式化器
///
/// 提供统一的显示格式化功能，包括路径、列表项、键值对和文件大小的格式化。
///
/// # 示例
///
/// ```
/// use workflow::base::format::DisplayFormatter;
/// use std::path::Path;
///
/// // 格式化路径
/// let path = Path::new("/home/user/project/src/main.rs");
/// let formatted_path = DisplayFormatter::path(path);
///
/// // 格式化列表项
/// let list_item = DisplayFormatter::list_item("  -", "config.toml");
/// assert_eq!(list_item, "  - config.toml");
///
/// // 格式化键值对
/// let kv = DisplayFormatter::key_value("Version", "1.0.0", None);
/// assert_eq!(kv, "Version: 1.0.0");
///
/// // 格式化文件大小
/// let size = DisplayFormatter::size(1024);
/// assert_eq!(size, "1.00 KB");
/// ```
pub struct DisplayFormatter;

impl DisplayFormatter {
    /// 格式化路径显示
    ///
    /// 将路径格式化为适合显示的字符串，优先显示相对路径。
    ///
    /// # 参数
    ///
    /// * `path` - 要格式化的路径
    ///
    /// # 返回值
    ///
    /// 格式化后的路径字符串
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::base::format::DisplayFormatter;
    /// use std::path::Path;
    ///
    /// let path = Path::new("/home/user/project/src/main.rs");
    /// let formatted = DisplayFormatter::path(path);
    /// // 返回相对路径或简化的路径表示
    /// ```
    pub fn path(path: &Path) -> String {
        if let Ok(relative) = path.strip_prefix(std::env::current_dir().unwrap_or_default()) {
            relative.display().to_string()
        } else {
            path.display().to_string()
        }
    }

    /// 格式化列表项
    ///
    /// 为列表项显示提供统一的格式化函数。
    ///
    /// # 参数
    ///
    /// * `prefix` - 前缀符号
    /// * `item` - 项目内容
    ///
    /// # 返回值
    ///
    /// 格式化后的列表项字符串
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::base::format::DisplayFormatter;
    ///
    /// let item = DisplayFormatter::list_item("  -", "config.toml");
    /// assert_eq!(item, "  - config.toml");
    /// ```
    pub fn list_item(prefix: &str, item: &str) -> String {
        format!("{} {}", prefix, item)
    }

    /// 格式化键值对
    ///
    /// 为配置或属性显示提供统一的格式化函数。
    ///
    /// # 参数
    ///
    /// * `key` - 键名
    /// * `value` - 值
    /// * `separator` - 分隔符（默认为 ": "）
    ///
    /// # 返回值
    ///
    /// 格式化后的键值对字符串
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::base::format::DisplayFormatter;
    ///
    /// let kv = DisplayFormatter::key_value("Version", "1.0.0", None);
    /// assert_eq!(kv, "Version: 1.0.0");
    ///
    /// let kv = DisplayFormatter::key_value("Status", "Active", Some(" = "));
    /// assert_eq!(kv, "Status = Active");
    /// ```
    pub fn key_value(key: &str, value: &str, separator: Option<&str>) -> String {
        let sep = separator.unwrap_or(": ");
        format!("{}{}{}", key, sep, value)
    }

    /// 格式化文件大小
    ///
    /// 将字节数格式化为人类可读的格式（B, KB, MB, GB, TB）。
    ///
    /// # 参数
    ///
    /// * `bytes` - 字节数
    ///
    /// # 返回值
    ///
    /// 格式化后的字符串，例如 "1.23 MB" 或 "1024 B"
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::base::format::DisplayFormatter;
    ///
    /// assert_eq!(DisplayFormatter::size(0), "0 B");
    /// assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
    /// assert_eq!(DisplayFormatter::size(1048576), "1.00 MB");
    /// ```
    pub fn size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// 测试基本格式化功能
    ///
    /// ## 测试目的
    /// 验证 DisplayFormatter 的基本格式化功能，包括列表项、键值对和文件大小的格式化。
    ///
    /// ## 测试场景
    /// 1. 测试列表项格式化（`list_item`）
    /// 2. 测试键值对格式化（`key_value`）
    /// 3. 测试文件大小格式化（`size`）
    ///
    /// ## 预期结果
    /// - 列表项格式化正确：前缀和项目内容正确组合
    /// - 键值对格式化正确：使用默认分隔符 ": "
    /// - 文件大小格式化正确：1024 字节格式化为 "1.00 KB"
    #[test]
    fn test_basic_formatting() {
        // Arrange: 准备测试数据（直接在断言中使用）

        // Act & Assert: 验证列表项格式化
        assert_eq!(DisplayFormatter::list_item("-", "test"), "- test");

        // Act & Assert: 验证键值对格式化
        assert_eq!(
            DisplayFormatter::key_value("key", "value", None),
            "key: value"
        );

        // Act & Assert: 验证文件大小格式化
        assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
    }

    // ==================== 文件大小格式化测试 ====================

    /// 测试文件大小格式化（字节单位）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确格式化字节值（< 1024 字节）。
    ///
    /// ## 测试场景
    /// 测试多种字节值：0、1、512、1023
    ///
    /// ## 预期结果
    /// - 所有值都格式化为 "X B" 格式
    #[rstest]
    #[case(0, "0 B")]
    #[case(1, "1 B")]
    #[case(512, "512 B")]
    #[case(1023, "1023 B")]
    fn test_format_size_bytes_with_byte_values(#[case] bytes: u64, #[case] expected: &str) {
        // Arrange: 准备字节值（通过参数提供）

        // Act & Assert: 验证字节值格式化正确
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    /// 测试文件大小格式化（KB单位）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确格式化KB值（1024 字节到 1023 KB）。
    ///
    /// ## 测试场景
    /// 测试多种KB值：1 KB、1.5 KB、2 KB、1023 KB
    ///
    /// ## 预期结果
    /// - 所有值都格式化为 "X.XX KB" 格式
    #[rstest]
    #[case(1024, "1.00 KB")]
    #[case(1536, "1.50 KB")] // 1024 + 512
    #[case(2048, "2.00 KB")]
    #[case(1024 * 1023, "1023.00 KB")]
    fn test_format_size_kilobytes_with_kb_values(#[case] bytes: u64, #[case] expected: &str) {
        // Arrange: 准备KB值（通过参数提供）

        // Act & Assert: 验证KB值格式化正确
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    /// 测试文件大小格式化（MB单位）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确格式化MB值（1 MB到 1023 MB）。
    ///
    /// ## 测试场景
    /// 测试多种MB值：1 MB、1.5 MB、5 MB、1023 MB
    ///
    /// ## 预期结果
    /// - 所有值都格式化为 "X.XX MB" 格式
    #[rstest]
    #[case(1024 * 1024, "1.00 MB")]
    #[case(1024 * 1024 + 512 * 1024, "1.50 MB")]
    #[case(1024 * 1024 * 5, "5.00 MB")]
    #[case(1024 * 1024 * 1023, "1023.00 MB")]
    fn test_format_size_megabytes_with_mb_values(#[case] bytes: u64, #[case] expected: &str) {
        // Arrange: 准备MB值（通过参数提供）

        // Act & Assert: 验证MB值格式化正确
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    /// 测试文件大小格式化（GB单位）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确格式化GB值（1 GB及以上）。
    ///
    /// ## 测试场景
    /// 测试多种GB值：1 GB、1.5 GB、10 GB
    ///
    /// ## 预期结果
    /// - 所有值都格式化为 "X.XX GB" 格式
    #[rstest]
    #[case(1024_u64.pow(3), "1.00 GB")]
    #[case(1024_u64.pow(3) + 512 * 1024_u64.pow(2), "1.50 GB")]
    #[case(1024_u64.pow(3) * 10, "10.00 GB")]
    fn test_format_size_gigabytes_with_gb_values(#[case] bytes: u64, #[case] expected: &str) {
        // Arrange: 准备GB值（通过参数提供）

        // Act & Assert: 验证GB值格式化正确
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    /// 测试文件大小格式化（TB单位）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确格式化TB值（1 TB及以上）。
    ///
    /// ## 测试场景
    /// 测试多种TB值：1 TB、2 TB、1.5 TB
    ///
    /// ## 预期结果
    /// - 所有值都格式化为 "X.XX TB" 格式
    #[rstest]
    #[case(1024_u64.pow(4), "1.00 TB")]
    #[case(1024_u64.pow(4) * 2, "2.00 TB")]
    #[case(1024_u64.pow(4) + 512 * 1024_u64.pow(3), "1.50 TB")]
    fn test_format_size_terabytes_with_tb_values(#[case] bytes: u64, #[case] expected: &str) {
        // Arrange: 准备TB值（通过参数提供）

        // Act & Assert: 验证TB值格式化正确
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    /// 测试文件大小格式化（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确处理各种大小的文件。
    ///
    /// ## 测试场景
    /// 测试从字节到TB的各种大小值
    ///
    /// ## 预期结果
    /// - 所有值都格式化为正确的单位格式
    #[rstest]
    #[case(0, "0 B")]
    #[case(1, "1 B")]
    #[case(1023, "1023 B")]
    #[case(1024, "1.00 KB")]
    #[case(1536, "1.50 KB")]
    #[case(1048576, "1.00 MB")] // 1024^2
    #[case(1073741824, "1.00 GB")] // 1024^3
    #[case(1099511627776, "1.00 TB")] // 1024^4
    #[case(2147483648, "2.00 GB")] // 2 * 1024^3
    #[case(5368709120, "5.00 GB")] // 5 * 1024^3
    fn test_format_size_parametrized_with_various_bytes_returns_formatted_string(
        #[case] bytes: u64,
        #[case] expected: &str,
    ) {
        // Arrange: 准备字节值和预期结果（通过参数提供）

        // Act: 格式化文件大小
        let result = DisplayFormatter::size(bytes);

        // Assert: 验证格式化结果与预期一致
        assert_eq!(result, expected);
    }

    /// 测试文件大小格式化的精度（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 DisplayFormatter::size() 能够正确处理带小数的文件大小。
    ///
    /// ## 测试场景
    /// 测试1.25 KB、1.10 KB、1.05 KB等带小数的值
    ///
    /// ## 预期结果
    /// - 小数精度正确（保留两位小数）
    #[rstest]
    #[case(1024 + 256, "1.25 KB")] // 1.25 KB
    #[case(1024 + 102, "1.10 KB")] // 约1.10 KB
    #[case(1024 + 51, "1.05 KB")] // 约1.05 KB
    fn test_format_size_precision_with_decimal_values(#[case] bytes: u64, #[case] expected: &str) {
        // Arrange: 准备带小数的字节值（通过参数提供）

        // Act & Assert: 验证小数精度正确
        assert_eq!(DisplayFormatter::size(bytes), expected);
    }

    /// 测试文件大小格式化的边界情况
    ///
    /// ## 测试目的
    /// 验证 DisplayFormatter::size() 能够正确处理边界值（如单位转换点、最大值等）。
    ///
    /// ## 测试场景
    /// 测试1023 B、1024 B、1024 KB、1 MB等边界值以及最大值
    ///
    /// ## 预期结果
    /// - 边界值格式化正确
    /// - 最大值能够正确处理
    #[test]
    fn test_format_size_edge_cases_with_boundary_values_handles_correctly() {
        // Arrange: 准备边界值
        let max_value = u64::MAX;
        let boundary_values = vec![
            (1024 - 1, "1023 B"),
            (1024, "1.00 KB"),
            (1024 * 1024 - 1, "1024.00 KB"),
            (1024 * 1024, "1.00 MB"),
        ];

        // Act & Assert: 验证边界值处理正确
        assert_eq!(
            DisplayFormatter::size(max_value),
            format!("{:.2} TB", max_value as f64 / 1024_f64.powi(4))
        );
        for (bytes, expected) in boundary_values {
            assert_eq!(DisplayFormatter::size(bytes), expected);
        }
    }
}
