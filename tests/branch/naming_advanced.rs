//! Branch 命名高级功能测试
//!
//! 测试分支命名的高级功能，包括边界情况、Unicode 处理等。

use pretty_assertions::assert_eq;
use workflow::branch::naming::BranchNaming;

// ==================== Slugify Boundary Tests ====================

#[test]
fn test_slugify_with_long_string_enforces_length_limit() {
    // Arrange: 准备超过长度限制的长字符串（100个字符）
    let long_string = "a".repeat(100);

    // Act: 调用 slugify 方法
    let result = BranchNaming::slugify(&long_string);

    // Assert: 验证结果不超过50个字符
    assert!(result.len() <= 50);
}

#[test]
fn test_slugify_with_edge_cases_handles_correctly() {
    // Arrange: 准备边界情况输入
    let empty_input = "";
    let whitespace_input = "   ";
    let hyphens_input = "---";
    let single_lowercase = "a";
    let single_uppercase = "A";

    // Act: 调用 slugify 方法
    let result_empty = BranchNaming::slugify(empty_input);
    let result_whitespace = BranchNaming::slugify(whitespace_input);
    let result_hyphens = BranchNaming::slugify(hyphens_input);
    let result_lowercase = BranchNaming::slugify(single_lowercase);
    let result_uppercase = BranchNaming::slugify(single_uppercase);

    // Assert: 验证边界情况处理正确
    assert_eq!(result_empty, "");
    assert_eq!(result_whitespace, "");
    assert_eq!(result_hyphens, "");
    assert_eq!(result_lowercase, "a");
    assert_eq!(result_uppercase, "a");
}

#[test]
fn test_slugify_with_unicode_characters_handles_correctly() {
    // Arrange: 准备包含 Unicode 字符的输入
    let input1 = "café";
    let input2 = "naïve";

    // Act: 调用 slugify 方法
    let result1 = BranchNaming::slugify(input1);
    let result2 = BranchNaming::slugify(input2);

    // Assert: 验证 Unicode 字符处理正确（保留 ASCII 部分）
    assert!(result1.contains("caf"));
    assert!(result2.contains("na"));
}

#[test]
fn test_slugify_with_numbers_preserves_numbers() {
    // Arrange: 准备包含数字的输入
    let input1 = "test123";
    let input2 = "123test";
    let input3 = "test-123-branch";

    // Act: 调用 slugify 方法
    let result1 = BranchNaming::slugify(input1);
    let result2 = BranchNaming::slugify(input2);
    let result3 = BranchNaming::slugify(input3);

    // Assert: 验证数字被保留
    assert_eq!(result1, "test123");
    assert_eq!(result2, "123test");
    assert_eq!(result3, "test-123-branch");
}

// ==================== Sanitize Boundary Tests ====================

#[test]
fn test_sanitize_with_edge_cases_handles_correctly() {
    // Arrange: 准备边界情况输入
    let empty_input = "";
    let whitespace_input = "   ";
    let hyphens_input = "---";
    let single_lowercase = "a";
    let single_uppercase = "A";

    // Act: 调用 sanitize 方法
    let result_empty = BranchNaming::sanitize(empty_input);
    let result_whitespace = BranchNaming::sanitize(whitespace_input);
    let result_hyphens = BranchNaming::sanitize(hyphens_input);
    let result_lowercase = BranchNaming::sanitize(single_lowercase);
    let result_uppercase = BranchNaming::sanitize(single_uppercase);

    // Assert: 验证边界情况处理正确
    assert_eq!(result_empty, "");
    assert_eq!(result_whitespace, "");
    assert_eq!(result_hyphens, "");
    assert_eq!(result_lowercase, "a");
    assert_eq!(result_uppercase, "a");
}

#[test]
fn test_sanitize_with_unicode_characters_removes_non_ascii() {
    // Arrange: 准备包含 Unicode 字符的输入
    let input1 = "café";
    let input2 = "naïve";
    let input3 = "résumé";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);
    let result3 = BranchNaming::sanitize(input3);

    // Assert: 验证非 ASCII 字符被移除，ASCII 字符被保留
    assert!(result1.contains("caf"));
    assert!(!result1.contains("é"));
    assert!(result2.contains("na"));
    assert!(!result2.contains("ï"));
    assert!(result3.contains("r"));
    assert!(!result3.contains("é"));
}

#[test]
fn test_sanitize_with_numbers_preserves_numbers() {
    // Arrange: 准备包含数字的输入
    let input1 = "test123";
    let input2 = "123test";
    let input3 = "test-123-branch";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);
    let result3 = BranchNaming::sanitize(input3);

    // Assert: 验证数字被保留
    assert_eq!(result1, "test123");
    assert_eq!(result2, "123test");
    assert_eq!(result3, "test-123-branch");
}

