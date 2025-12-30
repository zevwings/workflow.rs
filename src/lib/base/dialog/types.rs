use color_eyre::Result;
use std::sync::Arc;

/// Type alias for validator functions to reduce type complexity
pub(crate) type ValidatorFn = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::time::Instant;

    // ==================== 测试辅助函数 ====================

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
        let regex = regex::Regex::new(pattern)
            .map_err(|e| format!("regex pattern should be valid: {}", e))
            .expect("regex pattern should be valid");
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

    // ==================== 基础验证器测试 ====================

    /// 测试非空验证器功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证非空验证器能够正确识别空输入和有效输入。
    ///
    /// ## 测试场景
    /// 测试各种输入：有效输入（非空字符串）、无效输入（空字符串、只有空格、只有空白字符）
    ///
    /// ## 预期结果
    /// - 有效输入通过验证
    /// - 无效输入返回错误消息
    #[rstest]
    #[case("hello", true)] // 有效输入
    #[case("  world  ", true)] // 带空格的有效输入
    #[case("123", true)] // 数字字符串
    #[case("", false)] // 空字符串
    #[case("   ", false)] // 只有空格
    #[case("\t\n", false)] // 只有空白字符
    fn test_non_empty_validator_with_various_inputs_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
    ) {
        // Arrange: 准备非空验证器
        let validator = create_non_empty_validator();

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Input '{}' should {}",
            input,
            if should_be_valid {
                "be valid"
            } else {
                "be invalid"
            }
        );

        // 验证错误消息（仅对无效输入）
        if !should_be_valid {
            assert_eq!(result.unwrap_err(), "Input cannot be empty");
        }
    }

    /// 测试数字验证器功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证数字验证器能够正确识别有效数字和无效输入。
    ///
    /// ## 测试场景
    /// 测试各种输入：有效数字（整数、负数、零、带空格）、无效输入（非数字、浮点数、空字符串、混合字符）
    ///
    /// ## 预期结果
    /// - 有效数字通过验证
    /// - 无效输入返回错误消息
    #[rstest]
    #[case("123", true, None)] // 有效整数
    #[case("-456", true, None)] // 有效负数
    #[case("0", true, None)] // 零
    #[case("  789  ", true, None)] // 带空格的数字
    #[case("abc", false, Some("Please enter a valid number"))] // 非数字
    #[case("12.34", false, Some("Please enter a valid number"))] // 浮点数
    #[case("", false, Some("Number cannot be empty"))] // 空字符串
    #[case("123abc", false, Some("Please enter a valid number"))] // 混合字符
    fn test_number_validator_with_various_inputs_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
        #[case] expected_error: Option<&str>,
    ) {
        // Arrange: 准备数字验证器
        let validator = create_number_validator();

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Input '{}' should {}",
            input,
            if should_be_valid {
                "be valid"
            } else {
                "be invalid"
            }
        );

        // 验证错误消息（仅对无效输入）
        if !should_be_valid {
            if let Some(expected_msg) = expected_error {
                assert_eq!(result.unwrap_err(), expected_msg);
            }
        }
    }

    /// 测试邮箱验证器功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证邮箱验证器能够正确识别有效邮箱地址和无效输入。
    ///
    /// ## 测试场景
    /// 测试各种输入：有效邮箱（标准格式、带点、带空格）、无效邮箱（缺少@、缺少域名、缺少用户名、空字符串）
    ///
    /// ## 预期结果
    /// - 有效邮箱通过验证
    /// - 无效邮箱返回错误消息
    #[rstest]
    #[case("user@example.com", true, None)] // 标准格式
    #[case("test.email@domain.org", true, None)] // 带点
    #[case("  user@example.com  ", true, None)] // 带空格
    #[case("invalid-email", false, Some("Please enter a valid email address"))] // 无效格式（不包含@）
    #[case("@example.com", false, Some("Invalid email format"))] // 缺少用户名（包含@但格式不对）
    #[case("user@", false, Some("Please enter a valid email address"))] // 缺少域名（包含@但不包含.，实际返回"Please enter a valid email address"）
    #[case("user.example.com", false, Some("Please enter a valid email address"))] // 缺少@（不包含@）
    #[case("", false, Some("Email cannot be empty"))] // 空字符串
    fn test_email_validator_with_various_inputs_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
        #[case] expected_error: Option<&str>,
    ) {
        // Arrange: 准备邮箱验证器
        let validator = create_email_validator();

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Input '{}' should {}",
            input,
            if should_be_valid {
                "be valid"
            } else {
                "be invalid"
            }
        );

        // 验证错误消息（仅对无效输入）
        if !should_be_valid {
            if let Some(expected_msg) = expected_error {
                assert_eq!(result.unwrap_err(), expected_msg);
            }
        }
    }

    /// 测试长度验证器功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证长度验证器能够正确检查输入长度是否在指定范围内。
    ///
    /// ## 测试场景
    /// 测试各种输入：有效长度（最小长度、最大长度、中间长度）、无效长度（太短、太长、空字符串）
    ///
    /// ## 预期结果
    /// - 有效长度通过验证
    /// - 无效长度返回错误消息
    #[rstest]
    #[case("abc", true, None)] // 最小长度
    #[case("1234567890", true, None)] // 最大长度
    #[case("hello", true, None)] // 中间长度
    #[case("ab", false, Some("Input must be at least 3 characters"))] // 太短
    #[case("12345678901", false, Some("Input must be no more than 10 characters"))] // 太长
    #[case("", false, Some("Input must be at least 3 characters"))] // 空字符串
    fn test_length_validator_with_various_lengths_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
        #[case] expected_error: Option<&str>,
    ) {
        // Arrange: 准备长度验证器（最小3，最大10）
        let validator = create_length_validator(3, 10);

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Input '{}' should {}",
            input,
            if should_be_valid {
                "be valid"
            } else {
                "be invalid"
            }
        );

        // 验证错误消息（仅对无效输入）
        if !should_be_valid {
            if let Some(expected_msg) = expected_error {
                assert_eq!(result.unwrap_err(), expected_msg);
            }
        }
    }

    /// 测试范围验证器功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证范围验证器能够正确检查数字是否在指定范围内。
    ///
    /// ## 测试场景
    /// 测试各种输入：有效范围（最小值、最大值、中间值）、无效范围（小于最小值、大于最大值、负数、非数字）
    ///
    /// ## 预期结果
    /// - 有效范围通过验证
    /// - 无效范围返回错误消息
    #[rstest]
    #[case("1", true, None)] // 最小值
    #[case("100", true, None)] // 最大值
    #[case("50", true, None)] // 中间值
    #[case("0", false, Some("Number must be between 1 and 100"))] // 小于最小值
    #[case("101", false, Some("Number must be between 1 and 100"))] // 大于最大值
    #[case("-5", false, Some("Number must be between 1 and 100"))] // 负数
    #[case("abc", false, Some("Please enter a valid number"))] // 非数字输入
    fn test_range_validator_with_various_values_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
        #[case] expected_error: Option<&str>,
    ) {
        // Arrange: 准备范围验证器（1-100）
        let validator = create_range_validator(1, 100);

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Input '{}' should {}",
            input,
            if should_be_valid {
                "be valid"
            } else {
                "be invalid"
            }
        );

        // 验证错误消息（仅对无效输入）
        if !should_be_valid {
            if let Some(expected_msg) = expected_error {
                assert_eq!(result.unwrap_err(), expected_msg);
            }
        }
    }

    /// 测试正则表达式验证器功能（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证正则表达式验证器能够根据模式正确验证输入。
    ///
    /// ## 测试场景
    /// 测试各种输入：有效输入（符合正则模式）、无效输入（不符合正则模式）
    ///
    /// ## 预期结果
    /// - 符合模式的输入通过验证
    /// - 不符合模式的输入返回错误消息
    #[rstest]
    #[case("user123", true, None)] // 有效用户名
    #[case("test_user", true, None)] // 有效用户名（带下划线）
    #[case("UserName", true, None)] // 有效用户名（大小写混合）
    #[case(
        "user-123",
        false,
        Some("Username can only contain letters, numbers, and underscores")
    )] // 包含连字符
    #[case(
        "user@123",
        false,
        Some("Username can only contain letters, numbers, and underscores")
    )] // 包含特殊字符
    #[case(
        "user 123",
        false,
        Some("Username can only contain letters, numbers, and underscores")
    )] // 包含空格
    fn test_regex_validator_with_various_inputs_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
        #[case] expected_error: Option<&str>,
    ) {
        // Arrange: 准备正则验证器（用户名：只允许字母、数字、下划线）
        let validator = create_regex_validator(
            r"^[a-zA-Z0-9_]+$",
            "Username can only contain letters, numbers, and underscores",
        );

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(
            result.is_ok(),
            should_be_valid,
            "Input '{}' should {}",
            input,
            if should_be_valid {
                "be valid"
            } else {
                "be invalid"
            }
        );

        // 验证错误消息（仅对无效输入）
        if !should_be_valid {
            if let Some(expected_msg) = expected_error {
                assert_eq!(result.unwrap_err(), expected_msg);
            }
        }
    }

    // ==================== 参数化验证器测试 ====================

    /// 测试数字验证器参数化
    ///
    /// ## 测试目的
    /// 使用参数化测试验证数字验证器对各种输入的响应。
    ///
    /// ## 测试场景
    /// 1. 使用多种输入（有效数字、无效输入）进行测试
    /// 2. 验证结果与预期一致
    ///
    /// ## 预期结果
    /// - 有效数字通过验证，无效输入返回错误
    #[rstest]
    #[case("123", true)]
    #[case("-456", true)]
    #[case("0", true)]
    #[case("  789  ", true)]
    #[case("abc", false)]
    #[case("12.34", false)]
    #[case("", false)]
    #[case("123abc", false)]
    fn test_number_validator_parametrized_with_various_inputs_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
    ) {
        // Arrange: 准备数字验证器
        let validator = create_number_validator();

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(result.is_ok(), should_be_valid);
    }

    /// 测试邮箱验证器参数化
    ///
    /// ## 测试目的
    /// 使用参数化测试验证邮箱验证器对各种输入的响应。
    ///
    /// ## 测试场景
    /// 1. 使用多种输入（有效邮箱、无效邮箱）进行测试
    /// 2. 验证结果与预期一致
    ///
    /// ## 预期结果
    /// - 有效邮箱通过验证，无效邮箱返回错误
    #[rstest]
    #[case("user@example.com", true)]
    #[case("test.email@domain.org", true)]
    #[case("  user@example.com  ", true)]
    #[case("invalid-email", false)]
    #[case("@example.com", false)]
    #[case("user@", false)]
    #[case("user.example.com", false)]
    #[case("", false)]
    fn test_email_validator_parametrized_with_various_inputs_validates_correctly(
        #[case] input: &str,
        #[case] should_be_valid: bool,
    ) {
        // Arrange: 准备邮箱验证器
        let validator = create_email_validator();

        // Act: 验证输入
        let result = validator(input);

        // Assert: 验证结果与预期一致
        assert_eq!(result.is_ok(), should_be_valid);
    }

    /// 测试长度验证器参数化
    ///
    /// ## 测试目的
    /// 使用参数化测试验证长度验证器对不同长度范围的响应。
    ///
    /// ## 测试场景
    /// 1. 使用不同的最小/最大长度和输入进行测试
    /// 2. 验证结果与预期一致
    ///
    /// ## 预期结果
    /// - 在范围内的输入通过验证，超出范围返回错误
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

    /// 测试输入验证逻辑（有验证器）
    ///
    /// ## 测试目的
    /// 验证当提供验证器时，输入验证逻辑能够正确调用验证器。
    ///
    /// ## 测试场景
    /// 1. 使用验证器验证有效输入
    /// 2. 使用验证器验证无效输入
    /// 3. 验证 allow_empty 优先级
    ///
    /// ## 预期结果
    /// - 验证器被正确调用，allow_empty 优先于验证器
    #[test]
    fn test_validate_input_with_validator_with_validator_validates_correctly() {
        // Arrange: 准备验证器
        let validator = create_non_empty_validator();

        // Act & Assert: 验证有验证器的情况
        assert!(mock_validate_input("hello", Some(&validator), false).is_ok());
        assert!(mock_validate_input("", Some(&validator), false).is_err());
        assert!(mock_validate_input("", Some(&validator), true).is_ok()); // allow_empty优先
    }

    /// 测试输入验证逻辑（无验证器）
    ///
    /// ## 测试目的
    /// 验证当没有提供验证器时，输入验证逻辑能够正确处理空值检查。
    ///
    /// ## 测试场景
    /// 1. 无验证器且不允许空值的情况
    /// 2. 无验证器但允许空值的情况
    ///
    /// ## 预期结果
    /// - 根据 allow_empty 标志正确处理空值
    #[test]
    fn test_validate_input_without_validator_without_validator_validates_correctly() {
        // Arrange: 准备无验证器的情况

        // Act & Assert: 验证没有验证器的情况
        assert!(mock_validate_input("hello", None, false).is_ok());
        assert!(mock_validate_input("", None, false).is_err()); // 不允许空值
        assert!(mock_validate_input("", None, true).is_ok()); // 允许空值
        assert!(mock_validate_input("hello", None, true).is_ok());
    }

    /// 测试 allow_empty 优先级
    ///
    /// ## 测试目的
    /// 验证 allow_empty 标志的优先级高于验证器。
    ///
    /// ## 测试场景
    /// 1. 设置 allow_empty=true，即使验证器拒绝空值
    /// 2. 验证空值能够通过验证
    ///
    /// ## 预期结果
    /// - allow_empty=true 时，空值能够通过验证
    #[test]
    fn test_validate_input_allow_empty_priority_with_allow_empty_prioritizes_empty() {
        // Arrange: 准备验证器
        let validator = create_non_empty_validator();

        // Act & Assert: 验证allow_empty的优先级
        assert!(mock_validate_input("", Some(&validator), true).is_ok()); // allow_empty优先
        assert!(mock_validate_input("  ", Some(&validator), true).is_ok()); // 空格也算空
        assert!(mock_validate_input("hello", Some(&validator), true).is_ok());
    }

    /// 测试空白字符处理
    ///
    /// ## 测试目的
    /// 验证输入验证逻辑能够正确处理空白字符（空格、制表符、换行符）。
    ///
    /// ## 测试场景
    /// 1. 测试带空格的输入
    /// 2. 测试只有空白字符的输入
    /// 3. 验证空白字符被正确识别为空值
    ///
    /// ## 预期结果
    /// - 空白字符被正确识别和处理
    #[test]
    fn test_validate_input_whitespace_handling_with_whitespace_handles_correctly() {
        // Arrange: 准备验证器
        let validator = create_non_empty_validator();

        // Act & Assert: 验证空白字符处理
        assert!(mock_validate_input("  hello  ", Some(&validator), false).is_ok());
        assert!(mock_validate_input("  \t\n  ", Some(&validator), false).is_err());
        assert!(mock_validate_input("  \t\n  ", None, false).is_err());
        assert!(mock_validate_input("  \t\n  ", None, true).is_ok());
    }

    // ==================== 复合验证器测试 ====================

    /// 测试组合验证器功能
    ///
    /// ## 测试目的
    /// 验证多个验证器能够组合使用，依次验证输入。
    ///
    /// ## 测试场景
    /// 1. 创建组合验证器（数字验证 + 范围验证）
    /// 2. 测试有效和无效输入
    ///
    /// ## 预期结果
    /// - 所有验证器都被调用，只有全部通过才返回成功
    #[test]
    fn test_combined_validators_with_multiple_validators_validates_all() {
        // Arrange: 创建组合验证器：数字 + 范围
        let combined_validator: ValidatorFn = Arc::new(|input: &str| -> Result<(), String> {
            // 先验证是否为数字
            let number_validator = create_number_validator();
            number_validator(input)?;

            // 再验证范围
            let range_validator = create_range_validator(1, 100);
            range_validator(input)?;

            Ok(())
        });

        // Act & Assert: 验证组合验证正确
        assert!(combined_validator("50").is_ok());
        assert!(combined_validator("1").is_ok());
        assert!(combined_validator("100").is_ok());
        assert!(combined_validator("0").is_err()); // 超出范围
        assert!(combined_validator("101").is_err()); // 超出范围
        assert!(combined_validator("abc").is_err()); // 不是数字
    }

    /// 测试条件验证器功能
    ///
    /// ## 测试目的
    /// 验证验证器能够根据输入内容进行条件验证。
    ///
    /// ## 测试场景
    /// 1. 创建条件验证器（根据前缀应用不同规则）
    /// 2. 测试不同条件下的验证结果
    ///
    /// ## 预期结果
    /// - 根据条件应用不同的验证规则
    #[test]
    fn test_conditional_validator_with_conditions_validates_conditionally() {
        // Arrange: 创建条件验证器：如果输入以"admin_"开头，则需要至少10个字符
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

        // Act & Assert: 验证条件验证正确
        assert!(conditional_validator("admin_user123").is_ok()); // 12 chars
        assert!(conditional_validator("admin_usr").is_err()); // 9 chars, too short for admin
        assert!(conditional_validator("user").is_ok()); // 4 chars, ok for regular
        assert!(conditional_validator("us").is_err()); // 2 chars, too short for regular
    }

    // ==================== 错误处理和边界条件测试 ====================

    /// 测试验证器错误消息
    ///
    /// ## 测试目的
    /// 验证各种验证器能够返回正确的错误消息。
    ///
    /// ## 测试场景
    /// 1. 测试多个验证器的错误消息
    /// 2. 验证错误消息内容正确
    ///
    /// ## 预期结果
    /// - 所有验证器返回预期的错误消息
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

    /// 测试 Unicode 输入验证
    ///
    /// ## 测试目的
    /// 验证验证器能够正确处理 Unicode 字符（包括中文、emoji）。
    ///
    /// ## 测试场景
    /// 1. 测试中文字符验证
    /// 2. 测试包含重音符号的字符
    /// 3. 测试 emoji 字符
    ///
    /// ## 预期结果
    /// - Unicode 字符被正确验证
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

    /// 测试极端输入长度
    ///
    /// ## 测试目的
    /// 验证长度验证器能够处理极长和极短的输入。
    ///
    /// ## 测试场景
    /// 1. 测试极长输入（500字符）
    /// 2. 测试超长输入（1001字符）
    /// 3. 测试空输入（min=0）
    ///
    /// ## 预期结果
    /// - 长度验证器正确处理极端情况
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

    /// 测试特殊字符验证
    ///
    /// ## 测试目的
    /// 验证正则表达式验证器能够正确处理特殊字符。
    ///
    /// ## 测试场景
    /// 1. 测试允许的特殊字符（连字符、下划线）
    /// 2. 测试不允许的特殊字符（@、空格）
    ///
    /// ## 预期结果
    /// - 特殊字符被正确验证
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

    /// 测试验证器性能
    ///
    /// ## 测试目的
    /// 验证验证器在大量调用时能够保持良好性能。
    ///
    /// ## 测试场景
    /// 1. 执行1000次验证操作
    /// 2. 测量执行时间
    ///
    /// ## 预期结果
    /// - 1000次验证在100毫秒内完成
    #[test]
    fn test_validator_performance() {
        let validator = create_email_validator();
        let test_input = "user@example.com";

        // 1000次验证应该很快完成（< 100ms）
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = validator(test_input);
        }
        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100,
            "Performance test failed: {}ms > 100ms",
            duration.as_millis()
        );
    }

    /// 测试验证器一致性
    ///
    /// ## 测试目的
    /// 验证验证器在多次调用时返回一致的结果。
    ///
    /// ## 测试场景
    /// 1. 多次运行相同的验证操作
    /// 2. 验证结果一致
    ///
    /// ## 预期结果
    /// - 多次调用返回相同结果
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

    /// 测试验证器线程安全性
    ///
    /// ## 测试目的
    /// 验证验证器能够在多线程环境中安全使用。
    ///
    /// ## 测试场景
    /// 1. 在不同线程中使用验证器
    /// 2. 验证线程安全
    ///
    /// ## 预期结果
    /// - 验证器在多线程环境中正常工作
    #[test]
    fn test_validator_thread_safety() -> Result<()> {
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

        handle
            .join()
            .map_err(|e| color_eyre::eyre::eyre!("thread should join successfully: {:?}", e))?;
        Ok(())
    }

    /// 测试验证器内存效率
    ///
    /// ## 测试目的
    /// 验证创建大量验证器实例时的内存使用情况。
    ///
    /// ## 测试场景
    /// 1. 创建100个验证器实例
    /// 2. 验证所有验证器都能正常工作
    ///
    /// ## 预期结果
    /// - 所有验证器实例都能正常工作
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
