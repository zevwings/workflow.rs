//! Branch 命名高级功能测试
//!
//! 测试分支命名的高级功能，包括边界情况、Unicode 处理等。

use pretty_assertions::assert_eq;
use workflow::branch::naming::BranchNaming;

#[test]
fn test_slugify_length_limit() {
    // 测试 slugify 的长度限制（50字符）
    let long_string = "a".repeat(100);
    let result = BranchNaming::slugify(&long_string);

    // 验证结果不超过50个字符
    assert!(result.len() <= 50);
}

#[test]
fn test_sanitize_edge_cases() {
    // 测试边界情况
    assert_eq!(BranchNaming::sanitize(""), "");
    assert_eq!(BranchNaming::sanitize("   "), "");
    assert_eq!(BranchNaming::sanitize("---"), "");
    assert_eq!(BranchNaming::sanitize("a"), "a");
    assert_eq!(BranchNaming::sanitize("A"), "a");
}

#[test]
fn test_slugify_edge_cases() {
    // 测试边界情况
    assert_eq!(BranchNaming::slugify(""), "");
    assert_eq!(BranchNaming::slugify("   "), "");
    assert_eq!(BranchNaming::slugify("---"), "");
    assert_eq!(BranchNaming::slugify("a"), "a");
    assert_eq!(BranchNaming::slugify("A"), "a");
}

#[test]
fn test_sanitize_unicode_characters() {
    // 测试 Unicode 字符处理
    // 注意：sanitize 会移除非 ASCII 字符，但保留 ASCII 字符
    let result1 = BranchNaming::sanitize("café");
    assert!(result1.contains("caf"));
    assert!(!result1.contains("é"));

    let result2 = BranchNaming::sanitize("naïve");
    assert!(result2.contains("na"));
    assert!(!result2.contains("ï"));

    let result3 = BranchNaming::sanitize("résumé");
    assert!(result3.contains("r"));
    assert!(!result3.contains("é"));
}

#[test]
fn test_slugify_unicode_characters() {
    // 测试 Unicode 字符处理
    let result1 = BranchNaming::slugify("café");
    assert!(result1.contains("caf"));

    let result2 = BranchNaming::slugify("naïve");
    assert!(result2.contains("na"));
}

#[test]
fn test_sanitize_numbers() {
    // 测试数字处理
    assert_eq!(BranchNaming::sanitize("test123"), "test123");
    assert_eq!(BranchNaming::sanitize("123test"), "123test");
    assert_eq!(BranchNaming::sanitize("test-123-branch"), "test-123-branch");
}

#[test]
fn test_slugify_numbers() {
    // 测试数字处理
    assert_eq!(BranchNaming::slugify("test123"), "test123");
    assert_eq!(BranchNaming::slugify("123test"), "123test");
    assert_eq!(BranchNaming::slugify("test-123-branch"), "test-123-branch");
}

