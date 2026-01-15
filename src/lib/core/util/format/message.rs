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
    use rstest::rstest;

    #[rstest]
    #[case(
        "read",
        "config.toml",
        "Permission denied",
        "Failed to read config.toml: Permission denied"
    )]
    #[case(
        "write",
        "data.json",
        "Disk full",
        "Failed to write data.json: Disk full"
    )]
    #[case(
        "delete",
        "temp.txt",
        "File not found",
        "Failed to delete temp.txt: File not found"
    )]
    #[case(
        "open",
        "database.db",
        "Connection timeout",
        "Failed to open database.db: Connection timeout"
    )]
    fn test_error_formatting(
        #[case] operation: &str,
        #[case] target: &str,
        #[case] error_msg: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(error(operation, target, error_msg), expected);
    }

    #[rstest]
    #[case("Creating", "new branch", "Creating new branch...")]
    #[case("Updating", "config file", "Updating config file...")]
    #[case("Deleting", "old files", "Deleting old files...")]
    #[case("Processing", "data", "Processing data...")]
    fn test_operation_formatting(
        #[case] action: &str,
        #[case] target: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(operation(action, target), expected);
    }

    #[rstest]
    #[case(3, 10, "files", "[3/10] Processing files")]
    #[case(0, 100, "items", "[0/100] Processing items")]
    #[case(50, 100, "tasks", "[50/100] Processing tasks")]
    #[case(1, 1, "file", "[1/1] Processing file")]
    #[case(99, 100, "operations", "[99/100] Processing operations")]
    fn test_progress_formatting(
        #[case] current: usize,
        #[case] total: usize,
        #[case] item: &str,
        #[case] expected: &str,
    ) {
        assert_eq!(progress(current, total, item), expected);
    }
}
