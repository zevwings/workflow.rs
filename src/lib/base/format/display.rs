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

    #[test]
    fn test_basic_formatting() {
        // Basic validation of core formatting functions
        assert_eq!(DisplayFormatter::list_item("-", "test"), "- test");
        assert_eq!(
            DisplayFormatter::key_value("key", "value", None),
            "key: value"
        );
        assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
    }
}
