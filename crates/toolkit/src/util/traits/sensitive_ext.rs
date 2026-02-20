//! 敏感字符串扩展 trait
//!
//! 为字符串类型提供敏感信息处理相关的扩展方法。

/// 敏感字符串处理 trait
///
/// 为字符串类型提供敏感信息处理功能。
///
/// # 示例
///
/// ```
/// use toolkit::Sensitive;
///
/// assert_eq!("short".mask(), "***");
/// assert_eq!("verylongapikey123456".mask(), "very***3456");
/// ```
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
    /// use toolkit::Sensitive;
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
    use rstest::rstest;

    use super::*;

    // 测试 String 类型的 mask 方法
    #[test]
    fn test_mask_with_string_type() {
        let s = String::from("verylongapikey123456");
        assert_eq!(s.mask(), "very***3456");

        let short_string = String::from("short");
        assert_eq!(short_string.mask(), "***");
    }

    // 基础测试用例
    #[rstest]
    #[case("", "***")]
    #[case("a", "***")]
    #[case("abc", "***")]
    #[case("short", "***")]
    #[case("12345", "***")]
    #[case("123456789012", "***")] // 恰好12个字符
    #[case("1234567890123", "1234***0123")] // 13个字符
    #[case("verylongapikey123456", "very***3456")]
    #[case("abcdefghijklmnop", "abcd***mnop")] // 16 chars
    #[case("github_pat_1234567890abcdefghijklmnop", "gith***mnop")]
    #[case("very_long_api_key_with_underscores_123456", "very***3456")]
    #[case("ghp_1234567890abcdefghijklmnop", "ghp_***mnop")]
    #[case("sk-1234567890abcdefghijklmnopqrstuvwxyz", "sk-1***wxyz")]
    fn test_mask_basic(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(input.mask(), expected);
    }

    // 特殊字符测试用例
    #[rstest]
    #[case("key-with-dashes-123456789", "key-***6789")]
    #[case("key_with_underscores_123456", "key_***3456")]
    #[case("key.with.dots.123456789", "key.***6789")]
    #[case("key@with@symbols#123456", "key@***3456")]
    fn test_mask_special_characters(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(input.mask(), expected);
    }

    // Unicode 字符测试用例
    #[rstest]
    #[case("short string", "***")]
    #[case("a very long chinese string with numbers 123456", "a very***3456")]
    #[case("émoji🚀test123456789", "émoj***6789")]
    fn test_mask_unicode_strings(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(input.mask(), expected);
    }
}
