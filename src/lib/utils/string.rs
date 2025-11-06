//! 字符串处理工具函数

/// 隐藏敏感值（用于显示）
/// 短值完全隐藏，长值显示前3个字符和后3个字符
pub fn mask_sensitive_value(value: &str) -> String {
    if value.len() <= 8 {
        // 如果值很短，完全隐藏
        "***".to_string()
    } else {
        // 显示前3个字符和后3个字符，中间用 *** 代替
        let start = &value[..3.min(value.len())];
        let end = &value[value.len().saturating_sub(3)..];
        format!("{}***{}", start, end)
    }
}
