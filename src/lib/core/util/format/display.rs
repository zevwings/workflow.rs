//! 显示格式化工具函数

use crate::util::format::SizeDisplay;

/// 显示格式化器
///
/// 提供各种显示格式化功能。
///
/// # 示例
///
/// ```
/// use workflow::util::format::DisplayFormatter;
///
/// let size = DisplayFormatter::size(1024);
/// assert_eq!(size, "1.00 KB");
/// ```
pub struct DisplayFormatter;

impl DisplayFormatter {
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
    /// use workflow::util::format::DisplayFormatter;
    ///
    /// assert_eq!(DisplayFormatter::size(0), "0 B");
    /// assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
    /// assert_eq!(DisplayFormatter::size(1024 * 1024), "1.00 MB");
    /// ```
    pub fn size(bytes: u64) -> String {
        bytes.to_size_string()
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
/// use workflow::util::format::list_item;
///
/// let item = list_item("  -", "config.toml");
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
/// use workflow::util::format::key_value;
///
/// let kv = key_value("Version", "1.0.0", None);
/// assert_eq!(kv, "Version: 1.0.0");
///
/// let kv = key_value("Status", "Active", Some(" = "));
/// assert_eq!(kv, "Status = Active");
/// ```
pub fn key_value(key: &str, value: &str, separator: Option<&str>) -> String {
    let sep = separator.unwrap_or(": ");
    format!("{}{}{}", key, sep, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_item_formatting() {
        let item = list_item("  -", "config.toml");
        assert_eq!(item, "  - config.toml");
    }

    #[test]
    fn test_key_value_formatting() {
        let kv = key_value("Version", "1.0.0", None);
        assert_eq!(kv, "Version: 1.0.0");

        let kv = key_value("Status", "Active", Some(" = "));
        assert_eq!(kv, "Status = Active");
    }
}
