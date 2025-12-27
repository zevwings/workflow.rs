//! 敏感字符串处理工具
//!
//! 本模块提供了敏感字符串处理相关的工具 trait。

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
    /// use workflow::base::format::sensitive::Sensitive;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_basic() {
        // Basic validation of masking functionality
        assert_eq!("short".mask(), "***");
        assert_eq!("verylongapikey123456".mask(), "very***3456");
    }
}
