//! Branch 命名工具函数测试
//!
//! 测试分支命名相关的工具函数，包括 slugify、sanitize 等。

use pretty_assertions::assert_eq;
use workflow::branch::naming::BranchNaming;

// ==================== Slugify Tests ====================

#[test]
fn test_slugify_with_basic_input_returns_slugified_string() {
    // Arrange: 准备基本输入字符串
    let input1 = "Hello World";
    let input2 = "test branch";
    let input3 = "Test Branch Name";

    // Act: 调用 slugify 方法
    let result1 = BranchNaming::slugify(input1);
    let result2 = BranchNaming::slugify(input2);
    let result3 = BranchNaming::slugify(input3);

    // Assert: 验证返回正确的 slugified 字符串
    assert_eq!(result1, "hello-world");
    assert_eq!(result2, "test-branch");
    assert_eq!(result3, "test-branch-name");
}

#[test]
fn test_slugify_with_underscores_preserves_underscores() {
    // Arrange: 准备包含下划线的输入字符串
    let input1 = "test_branch";
    let input2 = "test_branch_name";

    // Act: 调用 slugify 方法
    let result1 = BranchNaming::slugify(input1);
    let result2 = BranchNaming::slugify(input2);

    // Assert: 验证下划线被保留
    assert_eq!(result1, "test_branch");
    assert_eq!(result2, "test_branch_name");
}

#[test]
fn test_slugify_with_special_characters_removes_special_chars() {
    // Arrange: 准备包含特殊字符的输入字符串
    let input1 = "test@branch#123";
    let input2 = "test.branch";

    // Act: 调用 slugify 方法
    let result1 = BranchNaming::slugify(input1);
    let result2 = BranchNaming::slugify(input2);

    // Assert: 验证特殊字符被移除，不转换为连字符
    assert_eq!(result1, "testbranch123");
    assert_eq!(result2, "testbranch");
}

#[test]
fn test_slugify_with_empty_string_returns_empty_string() {
    // Arrange: 准备空字符串
    let input = "";

    // Act: 调用 slugify 方法
    let result = BranchNaming::slugify(input);

    // Assert: 验证返回空字符串
    assert_eq!(result, "");
}

#[test]
fn test_slugify_with_whitespace_normalizes_whitespace() {
    // Arrange: 准备包含多余空格的输入字符串
    let input1 = "  test  branch  ";
    let input2 = "test   branch";

    // Act: 调用 slugify 方法
    let result1 = BranchNaming::slugify(input1);
    let result2 = BranchNaming::slugify(input2);

    // Assert: 验证多余空格被规范化
    assert_eq!(result1, "test-branch");
    assert_eq!(result2, "test-branch");
}

// ==================== Sanitize Tests ====================

#[test]
fn test_sanitize_with_basic_input_returns_sanitized_string() {
    // Arrange: 准备基本输入字符串
    let input1 = "Hello World";
    let input2 = "test-branch";
    let input3 = "Test Branch";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);
    let result3 = BranchNaming::sanitize(input3);

    // Assert: 验证返回正确的 sanitized 字符串
    assert_eq!(result1, "hello-world");
    assert_eq!(result2, "test-branch");
    assert_eq!(result3, "test-branch");
}

#[test]
fn test_sanitize_with_special_characters_converts_to_hyphens() {
    // Arrange: 准备包含特殊字符的输入字符串
    let input1 = "test@branch#123";
    let input2 = "test.branch";
    let input3 = "test_branch";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);
    let result3 = BranchNaming::sanitize(input3);

    // Assert: 验证特殊字符被转换为连字符
    assert_eq!(result1, "test-branch-123");
    assert_eq!(result2, "test-branch");
    assert_eq!(result3, "test-branch");
}

#[test]
fn test_sanitize_with_non_ascii_characters_removes_non_ascii() {
    // Arrange: 准备包含非 ASCII 字符的输入字符串
    let input1 = "测试分支";
    let input2 = "test中文branch";
    let input3 = "test 中文 branch";
    let input4 = "Hello 世界";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);
    let result3 = BranchNaming::sanitize(input3);
    let result4 = BranchNaming::sanitize(input4);

    // Assert: 验证非 ASCII 字符被移除
    assert_eq!(result1, "");
    assert_eq!(result2, "testbranch");
    assert_eq!(result3, "test-branch");
    assert_eq!(result4, "hello");
}

#[test]
fn test_sanitize_with_duplicate_hyphens_removes_duplicates() {
    // Arrange: 准备包含重复连字符的输入字符串
    let input1 = "test---branch";
    let input2 = "test   branch";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);

    // Assert: 验证重复连字符被移除
    assert_eq!(result1, "test-branch");
    assert_eq!(result2, "test-branch");
}

#[test]
fn test_sanitize_with_leading_trailing_dashes_trims_dashes() {
    // Arrange: 准备包含前导和尾随连字符的输入字符串
    let input1 = "-test-branch-";
    let input2 = "--test--";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);

    // Assert: 验证前导和尾随连字符被移除
    assert_eq!(result1, "test-branch");
    assert_eq!(result2, "test");
}

#[test]
fn test_sanitize_with_empty_string_returns_empty_string() {
    // Arrange: 准备空字符串
    let input = "";

    // Act: 调用 sanitize 方法
    let result = BranchNaming::sanitize(input);

    // Assert: 验证返回空字符串
    assert_eq!(result, "");
}

#[test]
fn test_sanitize_with_only_special_chars_returns_empty_string() {
    // Arrange: 准备只包含特殊字符的输入字符串
    let input1 = "@#$%";
    let input2 = "---";

    // Act: 调用 sanitize 方法
    let result1 = BranchNaming::sanitize(input1);
    let result2 = BranchNaming::sanitize(input2);

    // Assert: 验证返回空字符串
    assert_eq!(result1, "");
    assert_eq!(result2, "");
}

