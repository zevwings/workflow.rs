//! 消息格式化

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
/// use workflow::util::format::error;
///
/// let msg = error("read", "config.toml", "Permission denied");
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
/// use workflow::util::format::operation;
///
/// let msg = operation("Creating", "new branch");
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
/// use workflow::util::format::progress;
///
/// let msg = progress(3, 10, "files");
/// assert_eq!(msg, "[3/10] Processing files");
/// ```
pub fn progress(current: usize, total: usize, item: &str) -> String {
    format!("[{}/{}] Processing {}", current, total, item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_formatting() {
        let msg = error("read", "config.toml", "Permission denied");
        assert_eq!(msg, "Failed to read config.toml: Permission denied");
    }

    #[test]
    fn test_operation_formatting() {
        let msg = operation("Creating", "new branch");
        assert_eq!(msg, "Creating new branch...");
    }

    #[test]
    fn test_progress_formatting() {
        let msg = progress(3, 10, "files");
        assert_eq!(msg, "[3/10] Processing files");
    }
}
