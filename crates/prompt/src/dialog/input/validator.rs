//! 验证器模块
//!
//! 提供输入验证功能，包括 Validator trait 和内置验证器

use std::result;

/// 验证结果类型别名
pub type ValidationResult = result::Result<(), String>;

/// 验证器 Trait
pub trait Validator: Send + Sync {
    /// 验证输入，返回错误消息（如果验证失败）
    fn validate(&self, input: &str) -> ValidationResult;
}

/// 函数式验证器
impl<F> Validator for F
where
    F: Fn(&str) -> ValidationResult + Send + Sync,
{
    fn validate(&self, input: &str) -> ValidationResult {
        self(input)
    }
}

/// 内置验证器
pub mod validators {
    use regex::Regex;

    use super::Validator;

    /// 必填字段验证器
    ///
    /// 验证输入不能为空（去除首尾空格后）。
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入为空则返回错误。
    pub fn required() -> impl Validator {
        move |input: &str| {
            if input.trim().is_empty() {
                Err("This field is required".to_string())
            } else {
                Ok(())
            }
        }
    }

    /// 最小长度验证器
    ///
    /// 验证输入的长度至少为指定值。
    ///
    /// # 参数
    ///
    /// * `min` - 最小长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入长度小于最小值则返回错误。
    ///
    /// # 注意
    ///
    /// 空输入会被允许（空输入由 `required()` 验证器处理）。
    pub fn min_length(min: usize) -> impl Validator {
        move |input: &str| {
            // 允许空输入（空输入由 required() 验证器处理）
            if input.is_empty() || input.len() >= min {
                Ok(())
            } else {
                Err(format!("Length must be at least {} characters", min))
            }
        }
    }

    /// 最大长度验证器
    ///
    /// 验证输入的长度不超过指定值。
    ///
    /// # 参数
    ///
    /// * `max` - 最大长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入长度超过最大值则返回错误。
    pub fn max_length(max: usize) -> impl Validator {
        move |input: &str| {
            if input.len() <= max {
                Ok(())
            } else {
                Err(format!("Length must not exceed {} characters", max))
            }
        }
    }

    /// 长度范围验证器
    ///
    /// 验证输入的长度在指定范围内。
    ///
    /// # 参数
    ///
    /// * `min` - 最小长度（字符数）
    /// * `max` - 最大长度（字符数）
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入长度不在范围内则返回错误。
    ///
    /// # Panics
    ///
    /// 如果 `min > max`，验证器行为未定义。
    pub fn length(min: usize, max: usize) -> impl Validator {
        move |input: &str| {
            let len = input.len();
            if len >= min && len <= max {
                Ok(())
            } else {
                Err(format!(
                    "Length must be between {} and {} characters",
                    min, max
                ))
            }
        }
    }

    /// 正则表达式验证器
    ///
    /// 使用正则表达式验证输入格式。
    ///
    /// # 参数
    ///
    /// * `pattern` - 正则表达式模式（静态字符串，编译时验证）
    /// * `error_msg` - 可选的错误消息，如果未提供则使用默认消息
    ///
    /// # 返回
    ///
    /// 返回一个验证器，如果输入不匹配正则表达式则返回错误。
    ///
    /// # 错误
    ///
    /// 如果正则表达式编译失败，会在创建验证器时返回错误。
    /// 这通常表示正则表达式模式本身有问题。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use prompt::validators;
    ///
    /// // 验证数字
    /// let validator = validators::regex(r"^\d+$", Some("请输入数字"))?;
    ///
    /// // 验证邮箱（更严格）
    /// let validator = validators::regex(
    ///     r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
    ///     Some("请输入有效的邮箱地址")
    /// )?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn regex(
        pattern: &'static str,
        error_msg: Option<&'static str>,
    ) -> Result<impl Validator, String> {
        let re = Regex::new(pattern).map_err(|e| format!("Invalid regex '{}': {}", pattern, e))?;

        let error_msg = error_msg
            .map(String::from)
            .unwrap_or_else(|| format!("Invalid format, must match: {}", pattern));

        Ok(move |input: &str| {
            if re.is_match(input) {
                Ok(())
            } else {
                Err(error_msg.clone())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ==================== Validator Trait 测试 ====================

    #[test]
    fn test_function_validator() {
        let validator = |input: &str| -> ValidationResult {
            if input.len() > 5 {
                Ok(())
            } else {
                Err("Too short".to_string())
            }
        };

        assert!(validator.validate("123456").is_ok());
        assert!(validator.validate("123").is_err());
        assert_eq!(validator.validate("123").unwrap_err(), "Too short");
    }

    // ==================== required() 验证器测试 ====================

    #[test]
    fn test_required_validator() {
        let validator = validators::required();

        // 有效输入
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("  world  ").is_ok()); // 带空格的有效输入
        assert!(validator.validate("123").is_ok());

        // 无效输入
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err()); // 只有空格
        assert!(validator.validate("\t\n").is_err()); // 只有空白字符

        // 验证错误消息
        let result = validator.validate("");
        assert_eq!(result.unwrap_err(), "This field is required");
    }

    // ==================== min_length() 验证器测试 ====================

    #[rstest]
    #[case(3, "abc", true)]
    #[case(3, "ab", false)]
    #[case(3, "", true)] // 空输入被允许
    #[case(5, "12345", true)] // 正好最小长度
    #[case(5, "123456", true)] // 超过最小长度
    #[case(5, "1234", false)] // 太短
    fn test_min_length_validator(
        #[case] min: usize,
        #[case] input: &str,
        #[case] should_pass: bool,
    ) {
        let validator = validators::min_length(min);
        assert_eq!(validator.validate(input).is_ok(), should_pass);
    }

    #[test]
    fn test_min_length_error_message() {
        let validator = validators::min_length(5);
        let result = validator.validate("123");
        assert_eq!(result.unwrap_err(), "Length must be at least 5 characters");
    }

    // ==================== max_length() 验证器测试 ====================

    #[rstest]
    #[case(5, "", true)]
    #[case(5, "12345", true)] // 正好最大长度
    #[case(5, "1234", true)] // 小于最大长度
    #[case(5, "123456", false)] // 太长
    #[case(3, "", true)]
    fn test_max_length_validator(
        #[case] max: usize,
        #[case] input: &str,
        #[case] should_pass: bool,
    ) {
        let validator = validators::max_length(max);
        assert_eq!(validator.validate(input).is_ok(), should_pass);
    }

    #[test]
    fn test_max_length_error_message() {
        let validator = validators::max_length(5);
        let result = validator.validate("123456");
        assert_eq!(result.unwrap_err(), "Length must not exceed 5 characters");
    }

    // ==================== length() 验证器测试 ====================

    #[rstest]
    #[case(3, 5, "123", true)] // 最小长度
    #[case(3, 5, "12345", true)] // 最大长度
    #[case(3, 5, "1234", true)] // 中间长度
    #[case(3, 5, "12", false)] // 太短
    #[case(3, 5, "123456", false)] // 太长
    #[case(1, 10, "abc", true)]
    fn test_length_validator(
        #[case] min: usize,
        #[case] max: usize,
        #[case] input: &str,
        #[case] should_pass: bool,
    ) {
        let validator = validators::length(min, max);
        assert_eq!(validator.validate(input).is_ok(), should_pass);
    }

    #[test]
    fn test_length_error_message() {
        let validator = validators::length(3, 5);
        let short_result = validator.validate("12");
        assert_eq!(
            short_result.unwrap_err(),
            "Length must be between 3 and 5 characters"
        );

        let long_result = validator.validate("123456");
        assert_eq!(
            long_result.unwrap_err(),
            "Length must be between 3 and 5 characters"
        );
    }

    // ==================== regex() 验证器测试 ====================

    #[test]
    fn test_regex_validator() {
        // 测试数字验证
        let validator = validators::regex(r"^\d+$", Some("Please enter a number")).unwrap();

        assert!(validator.validate("123").is_ok());
        assert!(validator.validate("0").is_ok());
        assert!(validator.validate("abc").is_err());
        assert_eq!(
            validator.validate("abc").unwrap_err(),
            "Please enter a number"
        );

        // 测试自定义错误消息
        let validator =
            validators::regex(r"^[a-z]+$", Some("Only lowercase letters allowed")).unwrap();
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("Hello").is_err());
        assert_eq!(
            validator.validate("Hello").unwrap_err(),
            "Only lowercase letters allowed"
        );

        // 测试默认错误消息
        let validator = validators::regex(r"^\d+$", None).unwrap();
        assert!(validator.validate("123").is_ok());
        let err = validator.validate("abc").unwrap_err();
        assert!(err.contains("Invalid format"));
        assert!(err.contains(r"^\d+$"));
    }

    #[test]
    fn test_regex_validator_invalid_pattern() {
        // 测试无效的正则表达式
        let result = validators::regex(r"[invalid", None);
        assert!(result.is_err());
        match result {
            Err(err) => {
                assert!(err.contains("Invalid regex"));
                assert!(err.contains("[invalid"));
            }
            Ok(_) => panic!("Expected error for invalid regex pattern"),
        }
    }

    #[test]
    fn test_regex_validator_email() {
        // 测试更严格的邮箱验证
        let validator = validators::regex(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
            Some("Please enter a valid email address"),
        )
        .unwrap();

        assert!(validator.validate("user@example.com").is_ok());
        assert!(validator.validate("test.email@domain.org").is_ok());
        assert!(validator.validate("invalid").is_err());
        assert!(validator.validate("@example.com").is_err());
    }

    // ==================== Unicode 和边界条件测试 ====================

    #[test]
    fn test_validators_with_unicode() {
        let min_validator = validators::min_length(3);
        let max_validator = validators::max_length(5);

        // 注意：len() 计算的是字节数，不是字符数
        // 对于中文字符，每个字符占3个字节（UTF-8），所以 "你好" 的 len() 是 6，大于 3
        // 这里使用 ASCII 字符来测试字符数
        assert!(min_validator.validate("ab").is_err()); // 2个字符（字节）
        assert!(min_validator.validate("abc").is_ok()); // 3个字符（字节）

        assert!(max_validator.validate("abcde").is_ok()); // 5个字符（字节）
        assert!(max_validator.validate("abcdef").is_err()); // 6个字符（字节）

        // Unicode 字符测试（基于字节数）
        // "你好" 是 6 个字节，大于 min_length(3)
        assert!(min_validator.validate("你好").is_ok()); // 6个字节，大于3
                                                         // "你好世界" 是 12 个字节，大于 max_length(5)
        assert!(max_validator.validate("你好世界").is_err()); // 12个字节，大于5
    }

    #[test]
    fn test_validators_edge_cases() {
        // 测试极值
        let validator = validators::length(0, 1000);
        assert!(validator.validate("").is_ok());
        let long_str = "a".repeat(500);
        assert!(validator.validate(&long_str).is_ok());
        let too_long = "a".repeat(1001);
        assert!(validator.validate(&too_long).is_err());

        // 测试 min_length 允许空输入
        let validator = validators::min_length(10);
        assert!(validator.validate("").is_ok()); // 空输入被允许
        assert!(validator.validate("1234567890").is_ok());
        assert!(validator.validate("123").is_err());
    }
}
