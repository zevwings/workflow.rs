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
