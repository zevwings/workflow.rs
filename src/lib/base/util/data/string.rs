//! 字符串处理工具函数
//!
//! 本模块提供了字符串处理相关的工具函数和扩展 trait。

/// 敏感字符串处理 trait
///
/// 为字符串类型提供敏感信息处理功能。
pub trait Sensitive {
    /// 隐藏敏感值（用于显示）
    ///
    /// 用于在日志或输出中隐藏敏感信息（如 API key、密码等）。
    /// - 短值（长度 ≤ 12）：完全隐藏，显示为 `***`
    /// - 长值（长度 > 12）：显示前 4 个字符和后 4 个字符，中间用 `***` 代替
    ///
    /// # 返回
    ///
    /// 返回隐藏后的字符串。
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::base::util::string::Sensitive;
    ///
    /// assert_eq!("short".mask(), "***");
    /// assert_eq!("verylongapikey123456".mask(), "very***3456");
    /// ```
    fn mask(&self) -> String;
}

/// 为 `str` 实现 `Sensitive` trait
impl Sensitive for str {
    fn mask(&self) -> String {
        if self.len() <= 12 {
            // 如果值较短，完全隐藏
            "***".to_string()
        } else {
            // 显示前4个字符和后4个字符，中间用 *** 代替
            let chars: Vec<char> = self.chars().collect();
            let len = chars.len();
            let start: String = chars.iter().take(4.min(len)).collect();
            let end: String = chars.iter().skip(len.saturating_sub(4)).collect();
            format!("{}***{}", start, end)
        }
    }
}

/// 为 `String` 实现 `Sensitive` trait
impl Sensitive for String {
    fn mask(&self) -> String {
        self.as_str().mask()
    }
}

/// 隐藏敏感值（用于显示）
///
/// 这是一个便利函数，内部调用 `Sensitive` trait 的方法。
/// 保留此函数是为了向后兼容。
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
/// use workflow::base::util::mask_sensitive_value;
///
/// assert_eq!(mask_sensitive_value("short"), "***");
/// assert_eq!(mask_sensitive_value("verylongapikey123456"), "very***3456");
/// ```
#[inline]
pub fn mask_sensitive_value(value: &str) -> String {
    value.mask()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_short_value() {
        assert_eq!("short".mask(), "***");
        assert_eq!("12345".mask(), "***");
    }

    #[test]
    fn test_mask_long_value() {
        assert_eq!("verylongapikey123456".mask(), "very***3456");
        assert_eq!("1234567890123".mask(), "1234***0123");
    }

    #[test]
    fn test_mask_edge_cases() {
        assert_eq!("".mask(), "***");
        assert_eq!("123456789012".mask(), "***"); // 恰好12个字符
        assert_eq!("1234567890123".mask(), "1234***0123"); // 13个字符
    }

    #[test]
    fn test_mask_with_string() {
        let s = String::from("verylongapikey123456");
        assert_eq!(s.mask(), "very***3456");
    }

    #[test]
    fn test_backward_compatibility() {
        // 测试原有函数仍然可用
        assert_eq!(mask_sensitive_value("short"), "***");
        assert_eq!(mask_sensitive_value("verylongapikey123456"), "very***3456");
    }
}
