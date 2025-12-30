//! HTTP 方法枚举

use color_eyre::Result;
use std::fmt;
use std::str::FromStr;

/// HTTP 方法枚举
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl FromStr for HttpMethod {
    type Err = color_eyre::eyre::Report;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => color_eyre::eyre::bail!("Invalid HTTP method: {}", s),
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    /// 测试从字符串解析 HTTP 方法
    ///
    /// ## 测试目的
    /// 验证 HttpMethod::from_str() 能够正确解析有效的 HTTP 方法字符串，并拒绝无效的字符串。
    ///
    /// ## 测试场景
    /// 1. 测试解析有效的 HTTP 方法（"GET"）
    /// 2. 测试解析无效的字符串（"INVALID"）
    ///
    /// ## 预期结果
    /// - "GET" 被正确解析为 HttpMethod::Get
    /// - "INVALID" 返回错误
    #[test]
    fn test_from_str_basic() {
        // Arrange & Act & Assert: 验证 FromStr trait 的实现
        assert!(matches!(HttpMethod::from_str("GET"), Ok(HttpMethod::Get)));
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    /// 测试 HTTP 方法的 Display 实现
    ///
    /// ## 测试目的
    /// 验证 HttpMethod 的 Display trait 实现能够正确将枚举值转换为字符串表示。
    ///
    /// ## 测试场景
    /// 测试 Get 和 Post 方法的字符串表示
    ///
    /// ## 预期结果
    /// - HttpMethod::Get 转换为 "GET"
    /// - HttpMethod::Post 转换为 "POST"
    #[test]
    fn test_display_basic() {
        // Arrange & Act & Assert: 验证 Display trait 的实现
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    // ==================== HttpMethod Parsing Tests ====================

    /// 测试从字符串解析 HTTP 方法（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 HttpMethod::from_str() 能够正确解析所有有效的 HTTP 方法字符串。
    ///
    /// ## 测试场景
    /// 测试所有标准 HTTP 方法：GET, POST, PUT, DELETE, PATCH
    ///
    /// ## 预期结果
    /// - 所有有效的 HTTP 方法字符串都能正确解析为对应的枚举值
    #[rstest]
    #[case("GET", HttpMethod::Get)]
    #[case("POST", HttpMethod::Post)]
    #[case("PUT", HttpMethod::Put)]
    #[case("DELETE", HttpMethod::Delete)]
    #[case("PATCH", HttpMethod::Patch)]
    fn test_http_method_from_str_with_valid_methods_parses_correctly(
        #[case] input: &str,
        #[case] expected: HttpMethod,
    ) -> Result<()> {
        // Arrange: 准备HTTP方法字符串（通过参数传入）

        // Act: 从字符串解析HTTP方法
        let method = HttpMethod::from_str(input).map_err(|e| {
            color_eyre::eyre::eyre!("{} should be a valid HTTP method: {}", input, e)
        })?;

        // Assert: 验证解析为预期方法
        match (method, expected) {
            (HttpMethod::Get, HttpMethod::Get)
            | (HttpMethod::Post, HttpMethod::Post)
            | (HttpMethod::Put, HttpMethod::Put)
            | (HttpMethod::Delete, HttpMethod::Delete)
            | (HttpMethod::Patch, HttpMethod::Patch) => {
                // 验证方法匹配
            }
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "Failed to parse {} as {:?}",
                    input,
                    expected
                ))
            }
        }
        Ok(())
    }

    /// 测试从无效字符串解析 HTTP 方法
    ///
    /// ## 测试目的
    /// 验证 HttpMethod::from_str() 对无效字符串返回错误。
    ///
    /// ## 测试场景
    /// 1. 使用无效的 HTTP 方法字符串解析
    /// 2. 验证返回错误且错误消息包含 "Invalid HTTP method"
    ///
    /// ## 预期结果
    /// - 无效输入返回错误，错误消息包含 "Invalid HTTP method"
    #[test]
    fn test_http_method_from_str_invalid_with_invalid_string_returns_error() {
        // Arrange: 准备无效的HTTP方法字符串
        let invalid_method = "INVALID";

        // Act: 尝试解析无效方法
        let result = HttpMethod::from_str(invalid_method);

        // Assert: 验证返回错误且错误消息包含"Invalid HTTP method"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid HTTP method"));
    }

    /// 测试 HTTP 方法解析大小写敏感性
    ///
    /// ## 测试目的
    /// 验证 HttpMethod::from_str() 是大小写敏感的，小写输入返回错误。
    ///
    /// ## 测试场景
    /// 1. 使用小写和混合大小写的字符串解析
    /// 2. 验证返回错误
    ///
    /// ## 预期结果
    /// - 小写和混合大小写输入返回错误
    #[test]
    fn test_http_method_from_str_case_sensitive_with_lowercase_strings_returns_error() {
        // Arrange: 准备小写和混合大小写的字符串
        let lowercase_inputs = ["get", "post", "Get"];

        // Act & Assert: 验证大小写敏感，小写输入返回错误
        for input in lowercase_inputs.iter() {
            assert!(
                HttpMethod::from_str(input).is_err(),
                "Input '{}' should fail",
                input
            );
        }
    }

    // ==================== HttpMethod Display Tests ====================

    /// 测试 HTTP 方法显示格式（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证所有 HttpMethod 枚举值的 Display trait 实现是否正确。
    ///
    /// ## 测试场景
    /// 测试所有标准 HTTP 方法的显示格式：GET, POST, PUT, DELETE, PATCH
    ///
    /// ## 预期结果
    /// - 所有 HTTP 方法都能正确格式化为对应的字符串表示
    #[rstest]
    #[case(HttpMethod::Get, "GET")]
    #[case(HttpMethod::Post, "POST")]
    #[case(HttpMethod::Put, "PUT")]
    #[case(HttpMethod::Delete, "DELETE")]
    #[case(HttpMethod::Patch, "PATCH")]
    fn test_http_method_display_with_all_methods_formats_correctly(
        #[case] method: HttpMethod,
        #[case] expected: &str,
    ) {
        // Arrange: 准备HTTP方法（通过参数传入）

        // Act: 格式化显示
        let display = format!("{}", method);

        // Assert: 验证显示为预期字符串
        assert_eq!(
            display, expected,
            "Failed to format {:?} as {}",
            method, expected
        );
    }

    /// 测试 HTTP 方法 Debug 格式化
    ///
    /// ## 测试目的
    /// 验证 HttpMethod 的 Debug trait 实现正确。
    ///
    /// ## 测试场景
    /// 1. 格式化 HttpMethod 为 Debug 字符串
    /// 2. 验证输出包含方法名称
    ///
    /// ## 预期结果
    /// - Debug 输出包含 "Get"
    #[test]
    fn test_http_method_debug() {
        let debug_str = format!("{:?}", HttpMethod::Get);
        assert!(debug_str.contains("Get"));
    }

    /// 测试 HTTP 方法克隆功能
    ///
    /// ## 测试目的
    /// 验证 HttpMethod 的 Clone trait 实现正确。
    ///
    /// ## 测试场景
    /// 1. 创建 HttpMethod 实例
    /// 2. 克隆方法
    /// 3. 验证克隆后的值与原值相等
    ///
    /// ## 预期结果
    /// - 克隆后的值与原值相等
    #[test]
    fn test_http_method_clone() {
        let original = HttpMethod::Post;
        let cloned = original;
        assert_eq!(format!("{}", original), format!("{}", cloned));
    }

    /// 测试 HTTP 方法复制功能
    ///
    /// ## 测试目的
    /// 验证 HttpMethod 的 Copy trait 实现正确。
    ///
    /// ## 测试场景
    /// 1. 创建 HttpMethod 实例
    /// 2. 复制方法（通过赋值）
    /// 3. 验证复制后的值与原值相等
    ///
    /// ## 预期结果
    /// - 复制后的值与原值相等
    #[test]
    fn test_http_method_copy() {
        let original = HttpMethod::Put;
        let copied = original;
        assert_eq!(format!("{}", original), format!("{}", copied));
    }
}
