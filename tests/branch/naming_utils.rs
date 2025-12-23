//! Branch 命名工具函数测试
//!
//! 测试分支命名相关的工具函数，包括 slugify、sanitize 等。

use pretty_assertions::assert_eq;
use workflow::branch::naming::BranchNaming;

#[test]
fn test_slugify_basic() {
    assert_eq!(BranchNaming::slugify("Hello World"), "hello-world");
    assert_eq!(BranchNaming::slugify("test branch"), "test-branch");
    assert_eq!(BranchNaming::slugify("Test Branch Name"), "test-branch-name");
}

#[test]
fn test_slugify_preserves_underscores() {
    assert_eq!(BranchNaming::slugify("test_branch"), "test_branch");
    assert_eq!(BranchNaming::slugify("test_branch_name"), "test_branch_name");
}

#[test]
fn test_slugify_special_characters() {
    // slugify 会移除特殊字符，不转换为连字符
    assert_eq!(BranchNaming::slugify("test@branch#123"), "testbranch123");
    // 点号也会被移除
    assert_eq!(BranchNaming::slugify("test.branch"), "testbranch");
}

#[test]
fn test_slugify_empty() {
    assert_eq!(BranchNaming::slugify(""), "");
}

#[test]
fn test_slugify_whitespace() {
    assert_eq!(BranchNaming::slugify("  test  branch  "), "test-branch");
    assert_eq!(BranchNaming::slugify("test   branch"), "test-branch");
}

#[test]
fn test_sanitize_basic() {
    assert_eq!(BranchNaming::sanitize("Hello World"), "hello-world");
    assert_eq!(BranchNaming::sanitize("test-branch"), "test-branch");
    assert_eq!(BranchNaming::sanitize("Test Branch"), "test-branch");
}

#[test]
fn test_sanitize_special_characters() {
    assert_eq!(BranchNaming::sanitize("test@branch#123"), "test-branch-123");
    assert_eq!(BranchNaming::sanitize("test.branch"), "test-branch");
    assert_eq!(BranchNaming::sanitize("test_branch"), "test-branch");
}

#[test]
fn test_sanitize_non_ascii() {
    // 非 ASCII 字符应该被忽略
    assert_eq!(BranchNaming::sanitize("测试分支"), "");
    assert_eq!(BranchNaming::sanitize("test中文branch"), "testbranch");
    assert_eq!(BranchNaming::sanitize("test 中文 branch"), "test-branch");
    assert_eq!(BranchNaming::sanitize("Hello 世界"), "hello");
}

#[test]
fn test_sanitize_duplicate_hyphens() {
    assert_eq!(BranchNaming::sanitize("test---branch"), "test-branch");
    assert_eq!(BranchNaming::sanitize("test   branch"), "test-branch");
}

#[test]
fn test_sanitize_trim_dashes() {
    assert_eq!(BranchNaming::sanitize("-test-branch-"), "test-branch");
    assert_eq!(BranchNaming::sanitize("--test--"), "test");
}

#[test]
fn test_sanitize_empty() {
    assert_eq!(BranchNaming::sanitize(""), "");
}

#[test]
fn test_sanitize_only_special_chars() {
    assert_eq!(BranchNaming::sanitize("@#$%"), "");
    assert_eq!(BranchNaming::sanitize("---"), "");
}

