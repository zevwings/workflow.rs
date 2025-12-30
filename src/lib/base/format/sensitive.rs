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
    use rstest::rstest;

    /// 测试敏感信息掩码基本功能
    ///
    /// ## 测试目的
    /// 验证 Sensitive trait 的 mask() 方法能够正确掩码不同长度的字符串。
    ///
    /// ## 测试场景
    /// 1. 测试短字符串（≤12个字符）的掩码
    /// 2. 测试长字符串（>12个字符）的掩码
    ///
    /// ## 预期结果
    /// - 短字符串被完全掩码为 "***"
    /// - 长字符串显示前4个和后4个字符，中间用 "***" 掩码
    #[test]
    fn test_mask_basic() {
        // Arrange: 准备测试数据（短字符串和长字符串）

        // Act & Assert: 验证短字符串掩码（≤12个字符）
        assert_eq!("short".mask(), "***");

        // Act & Assert: 验证长字符串掩码（>12个字符）
        assert_eq!("verylongapikey123456".mask(), "very***3456");
    }

    // ==================== 敏感信息掩码测试 ====================

    /// 测试敏感信息掩码功能（短字符串）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Sensitive trait 的 mask() 方法能够正确掩码短字符串（≤12个字符）。
    ///
    /// ## 测试场景
    /// 测试空字符串、单字符、短字符串（≤12个字符）
    ///
    /// ## 预期结果
    /// - 所有短字符串都被掩码为 "***"
    #[rstest]
    #[case("", "***")]
    #[case("a", "***")]
    #[case("short", "***")]
    #[case("12345", "***")]
    #[case("123456789012", "***")] // 恰好12个字符
    fn test_mask_short_strings(#[case] input: &str, #[case] expected: &str) {
        // Arrange: 准备短字符串（通过参数提供）

        // Act & Assert: 验证短字符串被掩码
        assert_eq!(input.mask(), expected);
    }

    /// 测试敏感信息掩码功能（长字符串）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Sensitive trait 的 mask() 方法能够正确掩码长字符串（>12个字符）。
    ///
    /// ## 测试场景
    /// 测试各种长度的长字符串，包括API密钥格式
    ///
    /// ## 预期结果
    /// - 长字符串显示前4个和后4个字符，中间用 "***" 掩码
    #[rstest]
    #[case("1234567890123", "1234***0123")] // 13个字符
    #[case("verylongapikey123456", "very***3456")]
    #[case("ghp_1234567890abcdefghijklmnop", "ghp_***mnop")]
    #[case("sk-1234567890abcdefghijklmnopqrstuvwxyz", "sk-1***wxyz")]
    fn test_mask_long_strings(#[case] input: &str, #[case] expected: &str) {
        // Arrange: 准备长字符串（通过参数提供）

        // Act & Assert: 验证长字符串掩码正确
        assert_eq!(input.mask(), expected);
    }

    /// 测试敏感信息掩码功能（String类型）
    ///
    /// ## 测试目的
    /// 验证 Sensitive trait 的 mask() 方法能够正确处理 String 类型。
    ///
    /// ## 预期结果
    /// - String 类型能够正确掩码
    /// - 短字符串掩码为 "***"
    /// - 长字符串显示前后字符
    #[test]
    fn test_mask_with_string_type_with_string_inputs_returns_masked_string() {
        // Arrange: 准备String类型的输入
        let s = String::from("verylongapikey123456");
        let short_string = String::from("short");

        // Act & Assert: 验证String类型掩码正确
        assert_eq!(s.mask(), "very***3456");
        assert_eq!(short_string.mask(), "***");
    }

    /// 测试敏感信息掩码功能（基本场景）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Sensitive trait 的 mask() 方法的基本功能。
    ///
    /// ## 测试场景
    /// 测试空字符串、短字符串、长字符串
    ///
    /// ## 预期结果
    /// - 所有输入都能正确掩码
    #[rstest]
    #[case("short", "***")]
    #[case("verylongapikey123456", "very***3456")]
    #[case("", "***")]
    fn test_mask_basic_parametrized(#[case] input: &str, #[case] expected: &str) {
        // Arrange: 准备基本输入（通过参数提供）

        // Act & Assert: 验证基本掩码正确
        assert_eq!(input.mask(), expected);
    }

    /// 测试敏感信息掩码功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Sensitive trait 的 mask() 方法能够处理各种输入。
    ///
    /// ## 测试场景
    /// 测试从空字符串到长API密钥的各种输入
    ///
    /// ## 预期结果
    /// - 所有输入都能正确掩码
    #[rstest]
    #[case("", "***")]
    #[case("a", "***")]
    #[case("abc", "***")]
    #[case("123456789012", "***")] // 12 chars
    #[case("1234567890123", "1234***0123")] // 13 chars
    #[case("abcdefghijklmnop", "abcd***mnop")] // 16 chars
    #[case("github_pat_1234567890abcdefghijklmnop", "gith***mnop")]
    #[case("very_long_api_key_with_underscores_123456", "very***3456")]
    fn test_mask_parametrized_with_various_inputs_returns_masked_string(
        #[case] input: &str,
        #[case] expected: &str,
    ) {
        // Arrange: 准备输入和预期结果（通过参数提供）

        // Act: 掩码输入
        let result = input.mask();

        // Assert: 验证掩码结果与预期一致
        assert_eq!(result, expected);
    }

    /// 测试敏感信息掩码功能（特殊字符）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Sensitive trait 的 mask() 方法能够正确处理包含特殊字符的字符串。
    ///
    /// ## 测试场景
    /// 测试包含连字符、下划线、点号、@符号等的字符串
    ///
    /// ## 预期结果
    /// - 特殊字符被正确保留
    /// - 掩码格式正确
    #[rstest]
    #[case("key-with-dashes-123456789", "key-***6789")]
    #[case("key_with_underscores_123456", "key_***3456")]
    #[case("key.with.dots.123456789", "key.***6789")]
    #[case("key@with@symbols#123456", "key@***3456")]
    fn test_mask_special_characters(#[case] input: &str, #[case] expected: &str) {
        // Arrange: 准备包含特殊字符的字符串（通过参数提供）

        // Act & Assert: 验证特殊字符处理正确
        assert_eq!(input.mask(), expected);
    }

    /// 测试敏感信息掩码功能（Unicode字符串）（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 Sensitive trait 的 mask() 方法能够正确处理Unicode字符串。
    ///
    /// ## 测试场景
    /// 测试中文、emoji等Unicode字符
    ///
    /// ## 预期结果
    /// - Unicode字符被正确处理
    /// - 掩码格式正确
    #[rstest]
    #[case("短字符串", "***")]
    #[case("这是一个很长的中文字符串包含数字123456", "这是一个***3456")]
    #[case("émoji🚀test123456789", "émoj***6789")]
    fn test_mask_unicode_strings(#[case] input: &str, #[case] expected: &str) {
        // Arrange: 准备Unicode字符串（通过参数提供）

        // Act & Assert: 验证Unicode字符串处理正确
        assert_eq!(input.mask(), expected);
    }
}
