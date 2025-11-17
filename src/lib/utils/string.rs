//! 字符串处理工具函数
//!
//! 本模块提供了字符串处理相关的工具函数。

/// 隐藏敏感值（用于显示）
///
/// 用于在日志或输出中隐藏敏感信息（如 API key、密码等）。
/// - 短值（长度 ≤ 12）：完全隐藏，显示为 `***`
/// - 长值（长度 > 12）：显示前 4 个字符和后 4 个字符，中间用 `***` 代替
///
/// # 参数
///
/// * `value` - 要隐藏的敏感值
///
/// # 返回
///
/// 返回隐藏后的字符串。
///
/// # 示例
///
/// ```
/// use workflow::utils::mask_sensitive_value;
///
/// assert_eq!(mask_sensitive_value("short"), "***");
/// assert_eq!(mask_sensitive_value("verylongapikey123456"), "very***3456");
/// ```
pub fn mask_sensitive_value(value: &str) -> String {
    if value.len() <= 12 {
        // 如果值较短，完全隐藏
        "***".to_string()
    } else {
        // 显示前4个字符和后4个字符，中间用 *** 代替
        let start = &value[..4.min(value.len())];
        let end = &value[value.len().saturating_sub(4)..];
        format!("{}***{}", start, end)
    }
}
