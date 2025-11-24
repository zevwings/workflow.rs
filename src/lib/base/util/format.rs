//! 格式化工具模块
//!
//! 提供各种格式化函数，包括文件大小格式化等。

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
/// use workflow::base::util::format_size;
///
/// assert_eq!(format_size(0), "0 B");
/// assert_eq!(format_size(1024), "1.00 KB");
/// assert_eq!(format_size(1048576), "1.00 MB");
/// ```
pub fn format_size(bytes: u64) -> String {
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

