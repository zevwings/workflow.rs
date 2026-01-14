//! 格式化工具模块
//!
//! 提供统一的格式化功能，包括：
//! - 消息格式化（错误消息、操作消息、进度信息）
//! - 显示格式化（路径、列表项、键值对、文件大小）

use std::path::Path;

/// 消息格式化器
///
/// 提供统一的消息格式化功能，包括错误消息、操作消息和进度信息的格式化。
///
/// # 示例
///
/// ```
/// use workflow::util::format::MessageFormatter;
///
/// // 格式化错误消息
/// let error_msg = MessageFormatter::error("read", "config.toml", "Permission denied");
/// assert_eq!(error_msg, "Failed to read config.toml: Permission denied");
///
/// // 格式化操作消息
/// let operation_msg = MessageFormatter::operation("Creating", "new branch");
/// assert_eq!(operation_msg, "Creating new branch...");
///
/// // 格式化进度信息
/// let progress_msg = MessageFormatter::progress(3, 10, "files");
/// assert_eq!(progress_msg, "[3/10] Processing files");
/// ```
pub struct MessageFormatter;

impl MessageFormatter {
    /// 格式化错误消息
    ///
    /// 为常见的错误消息格式提供统一的格式化函数。
    ///
    /// # 参数
    ///
    /// * `operation` - 操作名称
    /// * `target` - 操作目标（文件、路径等）
    /// * `error` - 错误信息
    ///
    /// # 返回值
    ///
    /// 格式化后的错误消息字符串
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::util::format::MessageFormatter;
    ///
    /// let msg = MessageFormatter::error("read", "config.toml", "Permission denied");
    /// assert_eq!(msg, "Failed to read config.toml: Permission denied");
    /// ```
    pub fn error(operation: &str, target: &str, error: &str) -> String {
        format!("Failed to {} {}: {}", operation, target, error)
    }

    /// 格式化操作消息
    ///
    /// 为常见的操作消息格式提供统一的格式化函数。
    ///
    /// # 参数
    ///
    /// * `action` - 动作名称
    /// * `target` - 操作目标
    ///
    /// # 返回值
    ///
    /// 格式化后的操作消息字符串
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::util::format::MessageFormatter;
    ///
    /// let msg = MessageFormatter::operation("Creating", "new branch");
    /// assert_eq!(msg, "Creating new branch...");
    /// ```
    pub fn operation(action: &str, target: &str) -> String {
        format!("{} {}...", action, target)
    }

    /// 格式化进度信息
    ///
    /// 为进度显示提供统一的格式化函数。
    ///
    /// # 参数
    ///
    /// * `current` - 当前进度
    /// * `total` - 总进度
    /// * `item` - 进度项目名称
    ///
    /// # 返回值
    ///
    /// 格式化后的进度字符串
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::util::format::MessageFormatter;
    ///
    /// let msg = MessageFormatter::progress(3, 10, "files");
    /// assert_eq!(msg, "[3/10] Processing files");
    /// ```
    pub fn progress(current: usize, total: usize, item: &str) -> String {
        format!("[{}/{}] Processing {}", current, total, item)
    }
}

/// 显示格式化器
///
/// 提供统一的显示格式化功能，包括路径、列表项、键值对和文件大小的格式化。
///
/// # 示例
///
/// ```
/// use workflow::util::format::DisplayFormatter;
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
    /// use workflow::util::format::DisplayFormatter;
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
    /// use workflow::util::format::DisplayFormatter;
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
    /// use workflow::util::format::DisplayFormatter;
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
    /// use workflow::util::format::DisplayFormatter;
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
    fn test_error_formatting() {
        let msg = MessageFormatter::error("read", "config.toml", "Permission denied");
        assert_eq!(msg, "Failed to read config.toml: Permission denied");
    }

    #[test]
    fn test_operation_formatting() {
        let msg = MessageFormatter::operation("Creating", "new branch");
        assert_eq!(msg, "Creating new branch...");
    }

    #[test]
    fn test_progress_formatting() {
        let msg = MessageFormatter::progress(3, 10, "files");
        assert_eq!(msg, "[3/10] Processing files");
    }

    #[test]
    fn test_path_formatting() {
        let path = Path::new("/home/user/project/src/main.rs");
        let formatted = DisplayFormatter::path(path);
        // 根据当前工作目录，结果可能不同，但应该是一个有效的路径字符串
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_list_item_formatting() {
        let item = DisplayFormatter::list_item("  -", "config.toml");
        assert_eq!(item, "  - config.toml");
    }

    #[test]
    fn test_key_value_formatting() {
        let kv = DisplayFormatter::key_value("Version", "1.0.0", None);
        assert_eq!(kv, "Version: 1.0.0");

        let kv = DisplayFormatter::key_value("Status", "Active", Some(" = "));
        assert_eq!(kv, "Status = Active");
    }

    #[test]
    fn test_size_formatting() {
        assert_eq!(DisplayFormatter::size(0), "0 B");
        assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
        assert_eq!(DisplayFormatter::size(1048576), "1.00 MB");
    }
}
