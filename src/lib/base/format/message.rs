//! 消息格式化器模块
//!
//! 提供消息相关的格式化功能，包括错误消息、操作消息和进度信息。

/// 消息格式化器
///
/// 提供统一的消息格式化功能，包括错误消息、操作消息和进度信息的格式化。
///
/// # 示例
///
/// ```
/// use workflow::base::format::MessageFormatter;
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
    /// use workflow::base::format::MessageFormatter;
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
    /// use workflow::base::format::MessageFormatter;
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
    /// use workflow::base::format::MessageFormatter;
    ///
    /// let msg = MessageFormatter::progress(3, 10, "files");
    /// assert_eq!(msg, "[3/10] Processing files");
    /// ```
    pub fn progress(current: usize, total: usize, item: &str) -> String {
        format!("[{}/{}] Processing {}", current, total, item)
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
}
