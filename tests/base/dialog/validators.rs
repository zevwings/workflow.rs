//! Base/Dialog 验证器测试
//!
//! 测试Dialog模块中输入验证逻辑的核心业务功能，包括：
//! - 验证器函数的逻辑正确性
//! - 输入验证规则和边界条件
//! - 错误消息生成和处理
//! - 空值处理和默认值逻辑
//!
//! 注意：我们不测试实际的UI交互，只测试验证逻辑本身

use std::sync::Arc;

use color_eyre::Result;
use rstest::rstest;

// 由于 ValidatorFn 是私有类型，我们在测试中自定义类型别名
type ValidatorFn = std::sync::Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// 常用验证器函数 - 数字验证
fn create_number_validator() -> ValidatorFn {
    Arc::new(|input: &str| -> Result<(), String> {
        if input.trim().is_empty() {
            return Err("Number cannot be empty".to_string());
        }

        input
            .trim()
            .parse::<i32>()
            .map(|_| ())
            .map_err(|_| "Please enter a valid number".to_string())
    })
}

/// 常用验证器函数 - 邮箱验证
fn create_email_validator() -> ValidatorFn {
    Arc::new(|input: &str| -> Result<(), String> {
        let email = input.trim();
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if !email.contains('@') || !email.contains('.') {
            return Err("Please enter a valid email address".to_string());
        }

        // 简单的邮箱格式检查
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email format".to_string());
        }

        Ok(())
    })
}

/// 常用验证器函数 - 长度验证
fn create_length_validator(min: usize, max: usize) -> ValidatorFn {
    Arc::new(move |input: &str| -> Result<(), String> {
        let len = input.trim().chars().count(); // 使用 chars().count() 来正确计算 Unicode 字符数
        if len < min {
            return Err(format!("Input must be at least {} characters", min));
        }
        if len > max {
            return Err(format!("Input must be no more than {} characters", max));
        }
        Ok(())
    })
}

/// 常用验证器函数 - 非空验证
fn create_non_empty_validator() -> ValidatorFn {
    Arc::new(|input: &str| -> Result<(), String> {
        if input.trim().is_empty() {
            Err("Input cannot be empty".to_string())
        } else {
            Ok(())
        }
    })
}

/// 常用验证器函数 - 正则表达式验证
fn create_regex_validator(pattern: &str, error_msg: &str) -> ValidatorFn {
    let regex = regex::Regex::new(pattern).expect("regex pattern should be valid");
    let error_message = error_msg.to_string();

    Arc::new(move |input: &str| -> Result<(), String> {
        if regex.is_match(input.trim()) {
            Ok(())
        } else {
            Err(error_message.clone())
        }
    })
}

/// 常用验证器函数 - 范围验证（数字）
fn create_range_validator(min: i32, max: i32) -> ValidatorFn {
    Arc::new(move |input: &str| -> Result<(), String> {
        let num = input
            .trim()
            .parse::<i32>()
            .map_err(|_| "Please enter a valid number".to_string())?;

        if num < min || num > max {
            Err(format!("Number must be between {} and {}", min, max))
        } else {
            Ok(())
        }
    })
}

/// 模拟输入验证逻辑（从InputDialog中提取的核心逻辑）
fn mock_validate_input(
    input: &str,
    validator: Option<&ValidatorFn>,
    allow_empty: bool,
) -> Result<(), String> {
    // 如果允许空值且输入为空，直接通过（优先级最高）
    if allow_empty && input.trim().is_empty() {
        return Ok(());
    }

    // 如果有验证器，使用验证器
    if let Some(validator) = validator {
        return validator(input);
    }

    // 如果没有验证器但不允许空值，检查是否为空
    if !allow_empty && input.trim().is_empty() {
        return Err("Input cannot be empty".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 基础验证器测试 ====================

    #[test]
    fn test_non_empty_validator() {
        let validator = create_non_empty_validator();

        // 测试有效输入
        assert!(validator("hello").is_ok());
        assert!(validator("  world  ").is_ok()); // 带空格的有效输入
        assert!(validator("123").is_ok());

        // 测试无效输入
        assert!(validator("").is_err());
        assert!(validator("   ").is_err()); // 只有空格
        assert!(validator("\t\n").is_err()); // 只有空白字符

        // 验证错误消息
        let result = validator("");
        assert_eq!(result.unwrap_err(), "Input cannot be empty");
    }

    #[test]
    fn test_number_validator() {
        let validator = create_number_validator();

        // 测试有效数字
        assert!(validator("123").is_ok());
        assert!(validator("-456").is_ok());
        assert!(validator("0").is_ok());
        assert!(validator("  789  ").is_ok()); // 带空格的数字

        // 测试无效输入
        assert!(validator("abc").is_err());
        assert!(validator("12.34").is_err()); // 浮点数
        assert!(validator("").is_err());
        assert!(validator("123abc").is_err());

        // 验证错误消息
        let result = validator("abc");
        assert_eq!(result.unwrap_err(), "Please enter a valid number");

        let empty_result = validator("");
        assert_eq!(empty_result.unwrap_err(), "Number cannot be empty");
    }

    #[test]
    fn test_email_validator() {
        let validator = create_email_validator();

        // 测试有效邮箱
        assert!(validator("user@example.com").is_ok());
        assert!(validator("test.email@domain.org").is_ok());
        assert!(validator("  user@example.com  ").is_ok()); // 带空格

        // 测试无效邮箱
        assert!(validator("invalid-email").is_err());
        assert!(validator("@example.com").is_err()); // 缺少用户名
        assert!(validator("user@").is_err()); // 缺少域名
        assert!(validator("user.example.com").is_err()); // 缺少@
        assert!(validator("").is_err());

        // 验证错误消息
        let result = validator("invalid");
        assert_eq!(result.unwrap_err(), "Please enter a valid email address");

        let empty_result = validator("");
        assert_eq!(empty_result.unwrap_err(), "Email cannot be empty");
    }

    #[test]
    fn test_length_validator() {
        let validator = create_length_validator(3, 10);

        // 测试有效长度
        assert!(validator("abc").is_ok()); // 最小长度
        assert!(validator("1234567890").is_ok()); // 最大长度
        assert!(validator("hello").is_ok()); // 中间长度

        // 测试无效长度
        assert!(validator("ab").is_err()); // 太短
        assert!(validator("12345678901").is_err()); // 太长
        assert!(validator("").is_err()); // 空字符串

        // 验证错误消息
        let short_result = validator("ab");
        assert_eq!(
            short_result.unwrap_err(),
            "Input must be at least 3 characters"
        );

        let long_result = validator("12345678901");
        assert_eq!(
            long_result.unwrap_err(),
            "Input must be no more than 10 characters"
        );
    }

    #[test]
    fn test_range_validator() {
        let validator = create_range_validator(1, 100);

        // 测试有效范围
        assert!(validator("1").is_ok()); // 最小值
        assert!(validator("100").is_ok()); // 最大值
        assert!(validator("50").is_ok()); // 中间值

        // 测试无效范围
        assert!(validator("0").is_err()); // 小于最小值
        assert!(validator("101").is_err()); // 大于最大值
        assert!(validator("-5").is_err()); // 负数

        // 测试非数字输入
        assert!(validator("abc").is_err());

        // 验证错误消息
        let range_result = validator("0");
        assert_eq!(
            range_result.unwrap_err(),
            "Number must be between 1 and 100"
        );

        let invalid_result = validator("abc");
        assert_eq!(invalid_result.unwrap_err(), "Please enter a valid number");
    }

    #[test]
    fn test_regex_validator() {
        // 测试用户名验证（只允许字母、数字、下划线）
        let validator = create_regex_validator(
            r"^[a-zA-Z0-9_]+$",
            "Username can only contain letters, numbers, and underscores",
        );

        // 测试有效用户名
        assert!(validator("user123").is_ok());
        assert!(validator("test_user").is_ok());
        assert!(validator("UserName").is_ok());

        // 测试无效用户名
        assert!(validator("user-123").is_err()); // 包含连字符
        assert!(validator("user@123").is_err()); // 包含特殊字符
        assert!(validator("user 123").is_err()); // 包含空格

        // 验证错误消息
        let result = validator("user-123");
        assert_eq!(
            result.unwrap_err(),
            "Username can only contain letters, numbers, and underscores"
        );
    }

    // ==================== 参数化验证器测试 ====================

    #[rstest]
    #[case("123", true)]
    #[case("-456", true)]
    #[case("0", true)]
    #[case("  789  ", true)]
    #[case("abc", false)]
    #[case("12.34", false)]
    #[case("", false)]
    #[case("123abc", false)]
    fn test_number_validator_parametrized(#[case] input: &str, #[case] should_be_valid: bool) {
        let validator = create_number_validator();
        let result = validator(input);
        assert_eq!(result.is_ok(), should_be_valid);
    }

    #[rstest]
    #[case("user@example.com", true)]
    #[case("test.email@domain.org", true)]
    #[case("  user@example.com  ", true)]
    #[case("invalid-email", false)]
    #[case("@example.com", false)]
    #[case("user@", false)]
    #[case("user.example.com", false)]
    #[case("", false)]
    fn test_email_validator_parametrized(#[case] input: &str, #[case] should_be_valid: bool) {
        let validator = create_email_validator();
        let result = validator(input);
        assert_eq!(result.is_ok(), should_be_valid);
    }

    #[rstest]
    #[case(1, 5, "abc", true)] // 3 chars, within range
    #[case(1, 5, "a", true)] // 1 char, minimum
    #[case(1, 5, "abcde", true)] // 5 chars, maximum
    #[case(1, 5, "", false)] // 0 chars, too short
    #[case(1, 5, "abcdef", false)] // 6 chars, too long
    #[case(3, 3, "abc", true)] // exact length
    #[case(3, 3, "ab", false)] // too short
    #[case(3, 3, "abcd", false)] // too long
    fn test_length_validator_parametrized(
        #[case] min: usize,
        #[case] max: usize,
        #[case] input: &str,
        #[case] should_be_valid: bool,
    ) {
        let validator = create_length_validator(min, max);
        let result = validator(input);
        assert_eq!(result.is_ok(), should_be_valid);
    }

    // ==================== 输入验证逻辑测试 ====================

    #[test]
    fn test_validate_input_with_validator() {
        let validator = create_non_empty_validator();

        // 测试有验证器的情况
        assert!(mock_validate_input("hello", Some(&validator), false).is_ok());
        assert!(mock_validate_input("", Some(&validator), false).is_err());
        assert!(mock_validate_input("", Some(&validator), true).is_ok()); // allow_empty 优先
    }

    #[test]
    fn test_validate_input_without_validator() {
        // 测试没有验证器的情况
        assert!(mock_validate_input("hello", None, false).is_ok());
        assert!(mock_validate_input("", None, false).is_err()); // 不允许空值
        assert!(mock_validate_input("", None, true).is_ok()); // 允许空值
        assert!(mock_validate_input("hello", None, true).is_ok());
    }

    #[test]
    fn test_validate_input_allow_empty_priority() {
        let validator = create_non_empty_validator();

        // 测试 allow_empty 的优先级
        assert!(mock_validate_input("", Some(&validator), true).is_ok()); // allow_empty 优先
        assert!(mock_validate_input("  ", Some(&validator), true).is_ok()); // 空格也算空
        assert!(mock_validate_input("hello", Some(&validator), true).is_ok());
    }

    #[test]
    fn test_validate_input_whitespace_handling() {
        let validator = create_non_empty_validator();

        // 测试空白字符处理
        assert!(mock_validate_input("  hello  ", Some(&validator), false).is_ok());
        assert!(mock_validate_input("  \t\n  ", Some(&validator), false).is_err());
        assert!(mock_validate_input("  \t\n  ", None, false).is_err());
        assert!(mock_validate_input("  \t\n  ", None, true).is_ok());
    }

    // ==================== 复合验证器测试 ====================

    #[test]
    fn test_combined_validators() {
        // 创建一个组合验证器：数字 + 范围
        let combined_validator: ValidatorFn = Arc::new(|input: &str| -> Result<(), String> {
            // 先验证是否为数字
            let number_validator = create_number_validator();
            number_validator(input)?;

            // 再验证范围
            let range_validator = create_range_validator(1, 100);
            range_validator(input)?;

            Ok(())
        });

        // 测试组合验证
        assert!(combined_validator("50").is_ok());
        assert!(combined_validator("1").is_ok());
        assert!(combined_validator("100").is_ok());

        assert!(combined_validator("0").is_err()); // 超出范围
        assert!(combined_validator("101").is_err()); // 超出范围
        assert!(combined_validator("abc").is_err()); // 不是数字
    }

    #[test]
    fn test_conditional_validator() {
        // 创建条件验证器：如果输入以"admin_"开头，则需要至少10个字符
        let conditional_validator: ValidatorFn = Arc::new(|input: &str| -> Result<(), String> {
            let trimmed = input.trim();

            if trimmed.starts_with("admin_") {
                if trimmed.len() < 10 {
                    return Err("Admin usernames must be at least 10 characters".to_string());
                }
            } else if trimmed.len() < 3 {
                return Err("Regular usernames must be at least 3 characters".to_string());
            }

            Ok(())
        });

        // 测试条件验证
        assert!(conditional_validator("admin_user123").is_ok()); // 12 chars
        assert!(conditional_validator("admin_usr").is_err()); // 9 chars, too short for admin
        assert!(conditional_validator("user").is_ok()); // 4 chars, ok for regular
        assert!(conditional_validator("us").is_err()); // 2 chars, too short for regular
    }

    // ==================== 错误处理和边界条件测试 ====================

    #[test]
    fn test_validator_error_messages() {
        let validators = vec![
            (
                create_number_validator(),
                "abc",
                "Please enter a valid number",
            ),
            (
                create_email_validator(),
                "invalid",
                "Please enter a valid email address",
            ),
            (
                create_length_validator(5, 10),
                "ab",
                "Input must be at least 5 characters",
            ),
            (
                create_range_validator(1, 10),
                "20",
                "Number must be between 1 and 10",
            ),
        ];

        for (validator, input, expected_msg) in validators {
            let result = validator(input);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), expected_msg);
        }
    }

    #[test]
    fn test_unicode_input_validation() {
        let length_validator = create_length_validator(3, 10);

        // 测试Unicode字符
        assert!(length_validator("你好世界").is_ok()); // 4个中文字符
        assert!(length_validator("café").is_ok()); // 包含重音符号
        assert!(length_validator("🚀🌟").is_err()); // 2个emoji，太短

        let email_validator = create_email_validator();
        assert!(email_validator("用户@example.com").is_ok()); // 中文用户名
    }

    #[test]
    fn test_extreme_input_lengths() {
        let length_validator = create_length_validator(0, 1000);

        // 测试极长输入
        let long_input = "a".repeat(500);
        assert!(length_validator(&long_input).is_ok());

        let too_long_input = "a".repeat(1001);
        assert!(length_validator(&too_long_input).is_err());

        // 测试空输入
        assert!(length_validator("").is_ok()); // min = 0
    }

    #[test]
    fn test_special_characters_in_validation() {
        let regex_validator = create_regex_validator(
            r"^[a-zA-Z0-9_\-]+$",
            "Only letters, numbers, underscores and hyphens allowed",
        );

        // 测试有效字符
        assert!(regex_validator("password123").is_ok());
        assert!(regex_validator("user_name-123").is_ok());
        assert!(regex_validator("test_user").is_ok());

        // 测试不允许的字符
        assert!(regex_validator("test@user").is_err());
        assert!(regex_validator("test user").is_err());
    }

    // ==================== 性能和一致性测试 ====================

    #[test]
    fn test_validator_performance() {
        use std::time::Instant;

        let validator = create_email_validator();
        let test_input = "user@example.com";

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = validator(test_input);
        }
        let duration = start.elapsed();

        // 1000次验证应该很快完成
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_validator_consistency() {
        let validator = create_number_validator();
        let test_cases = vec![("123", true), ("abc", false), ("", false)];

        // 多次运行验证结果应该一致
        for _ in 0..10 {
            for (input, expected) in &test_cases {
                assert_eq!(validator(input).is_ok(), *expected);
            }
        }
    }

    #[test]
    fn test_validator_thread_safety() {
        use std::thread;

        let validator = create_number_validator();
        let validator_clone = validator.clone();

        // 测试在不同线程中使用验证器
        let handle = thread::spawn(move || {
            assert!(validator_clone("123").is_ok());
            assert!(validator_clone("abc").is_err());
        });

        // 主线程中也使用验证器
        assert!(validator("456").is_ok());
        assert!(validator("def").is_err());

        handle.join().expect("thread should join successfully");
    }

    #[test]
    fn test_validator_memory_efficiency() {
        // 创建多个验证器实例，测试内存使用
        let validators: Vec<ValidatorFn> =
            (0..100).map(|i| create_range_validator(i, i + 100)).collect();

        // 验证所有验证器都能正常工作
        for (i, validator) in validators.iter().enumerate() {
            let test_value = (i + 50).to_string();
            assert!(validator(&test_value).is_ok());
        }
    }
}
