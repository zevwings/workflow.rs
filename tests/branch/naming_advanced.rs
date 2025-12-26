//! Branch 命名高级功能测试
//!
//! 测试分支命名的高级功能，包括边界情况、Unicode 处理等。

use pretty_assertions::assert_eq;
use workflow::branch::naming::BranchNaming;

// ==================== Slugify Boundary Tests ====================

/// 测试slugify方法对长字符串的长度限制
///
/// ## 测试目的
/// 验证 `BranchNaming::slugify()` 方法能够正确处理超过长度限制的长字符串，结果不超过50个字符。
///
/// ## 测试场景
/// 1. 准备超过长度限制的长字符串（100个字符）
/// 2. 调用slugify方法
/// 3. 验证结果长度不超过50个字符
///
/// ## 预期结果
/// - 结果长度不超过50个字符
#[test]
fn test_slugify_with_long_string_enforces_length_limit() {
    // Arrange: 准备超过长度限制的长字符串（100个字符）
    let long_string = "a".repeat(100);

    // Act: 调用 slugify 方法
    let result = BranchNaming::slugify(&long_string);

    // Assert: 验证结果不超过50个字符
    assert!(result.len() <= 50);
}

/// 测试slugify方法处理边界情况
///
/// ## 测试目的
/// 验证 `BranchNaming::slugify()` 方法能够正确处理各种边界情况（空字符串、空格、连字符、单个字符等）。
///
/// ## 测试场景
/// 1. 测试空字符串
/// 2. 测试只有空格的字符串
/// 3. 测试只有连字符的字符串
/// 4. 测试单个小写字母
/// 5. 测试单个大写字母
///
/// ## 预期结果
/// - 空字符串和空格返回空字符串
/// - 连字符返回空字符串
/// - 单个字母返回小写字母
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

/// 测试slugify方法处理Unicode字符
///
/// ## 测试目的
/// 验证 `BranchNaming::slugify()` 方法能够正确处理包含Unicode字符的输入（保留ASCII部分）。
///
/// ## 测试场景
/// 1. 测试包含Unicode字符的输入（café, naïve）
/// 2. 调用slugify方法
/// 3. 验证Unicode字符处理正确
///
/// ## 预期结果
/// - Unicode字符被移除或转换
/// - ASCII部分被保留
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

/// 测试slugify方法保留数字
///
/// ## 测试目的
/// 验证 `BranchNaming::slugify()` 方法能够保留输入中的数字。
///
/// ## 测试场景
/// 1. 测试包含数字的输入（test123, 123test, test-123-branch）
/// 2. 调用slugify方法
/// 3. 验证数字被保留
///
/// ## 预期结果
/// - 数字被保留在结果中
/// - 格式正确（小写、连字符分隔）
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

/// 测试sanitize方法处理边界情况
///
/// ## 测试目的
/// 验证 `BranchNaming::sanitize()` 方法能够正确处理各种边界情况（空字符串、空格、连字符、单个字符等）。
///
/// ## 测试场景
/// 1. 测试空字符串
/// 2. 测试只有空格的字符串
/// 3. 测试只有连字符的字符串
/// 4. 测试单个小写字母
/// 5. 测试单个大写字母
///
/// ## 预期结果
/// - 空字符串和空格返回空字符串
/// - 连字符返回空字符串
/// - 单个字母返回小写字母
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

/// 测试sanitize方法移除非ASCII字符
///
/// ## 测试目的
/// 验证 `BranchNaming::sanitize()` 方法能够移除非ASCII字符，保留ASCII字符。
///
/// ## 测试场景
/// 1. 测试包含Unicode字符的输入（café, naïve, résumé）
/// 2. 调用sanitize方法
/// 3. 验证非ASCII字符被移除，ASCII字符被保留
///
/// ## 预期结果
/// - 非ASCII字符（é, ï等）被移除
/// - ASCII字符（caf, na, r等）被保留
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

/// 测试sanitize方法保留数字
///
/// ## 测试目的
/// 验证 `BranchNaming::sanitize()` 方法能够保留输入中的数字。
///
/// ## 测试场景
/// 1. 测试包含数字的输入（test123, 123test, test-123-branch）
/// 2. 调用sanitize方法
/// 3. 验证数字被保留
///
/// ## 预期结果
/// - 数字被保留在结果中
/// - 格式正确（小写、连字符分隔）
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
