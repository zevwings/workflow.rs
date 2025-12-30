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

    /// 测试错误消息格式化功能
    ///
    /// ## 测试目的
    /// 验证 MessageFormatter::error() 能够正确格式化错误消息，将操作名称、目标和错误信息组合成统一的错误消息格式。
    ///
    /// ## 测试场景
    /// 1. 使用操作名称 "read"、目标 "config.toml" 和错误信息 "Permission denied"
    /// 2. 调用 error() 方法格式化错误消息
    /// 3. 验证格式化结果符合预期格式
    ///
    /// ## 预期结果
    /// - 返回格式化的错误消息："Failed to read config.toml: Permission denied"
    /// - 消息格式为 "Failed to {operation} {target}: {error}"
    #[test]
    fn test_error_formatting() {
        // Arrange: 准备测试数据（操作名称、目标、错误信息）
        let operation = "read";
        let target = "config.toml";
        let error = "Permission denied";

        // Act: 格式化错误消息
        let msg = MessageFormatter::error(operation, target, error);

        // Assert: 验证格式化结果正确
        assert_eq!(msg, "Failed to read config.toml: Permission denied");
    }

    /// 测试操作消息格式化功能
    ///
    /// ## 测试目的
    /// 验证 MessageFormatter::operation() 能够正确格式化操作消息，将动作和目标组合成统一的操作消息格式。
    ///
    /// ## 测试场景
    /// 1. 使用动作 "Creating" 和目标 "new branch"
    /// 2. 调用 operation() 方法格式化操作消息
    /// 3. 验证格式化结果符合预期格式
    ///
    /// ## 预期结果
    /// - 返回格式化的操作消息："Creating new branch..."
    /// - 消息格式为 "{action} {target}..."
    #[test]
    fn test_operation_formatting() {
        // Arrange: 准备测试数据（动作、目标）
        let action = "Creating";
        let target = "new branch";

        // Act: 格式化操作消息
        let msg = MessageFormatter::operation(action, target);

        // Assert: 验证格式化结果正确
        assert_eq!(msg, "Creating new branch...");
    }

    /// 测试进度信息格式化功能
    ///
    /// ## 测试目的
    /// 验证 MessageFormatter::progress() 能够正确格式化进度信息，将当前进度、总进度和项目名称组合成统一的进度显示格式。
    ///
    /// ## 测试场景
    /// 1. 使用当前进度 3、总进度 10 和项目名称 "files"
    /// 2. 调用 progress() 方法格式化进度信息
    /// 3. 验证格式化结果符合预期格式
    ///
    /// ## 预期结果
    /// - 返回格式化的进度信息："[3/10] Processing files"
    /// - 消息格式为 "[{current}/{total}] Processing {item}"
    #[test]
    fn test_progress_formatting() {
        // Arrange: 准备测试数据（当前进度、总进度、项目名称）
        let current = 3;
        let total = 10;
        let item = "files";

        // Act: 格式化进度信息
        let msg = MessageFormatter::progress(current, total, item);

        // Assert: 验证格式化结果正确
        assert_eq!(msg, "[3/10] Processing files");
    }
}
