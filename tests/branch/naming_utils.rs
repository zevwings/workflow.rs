//! Branch 命名工具函数测试
//!
//! 测试分支命名相关的工具函数，包括 slugify、sanitize 等。
//!
//! ## 测试策略
//!
//! - 使用参数化测试减少重复代码
//! - 覆盖各种边界情况和特殊输入

use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::branch::naming::BranchNaming;

// ==================== Slugify Tests ====================

/// 测试 slugify 功能（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 BranchNaming::slugify() 能够正确处理各种输入场景。
///
/// ## 测试场景
/// 测试以下场景：
/// - 基本输入（空格转换为连字符）
/// - 保留下划线
/// - 移除特殊字符
/// - 空字符串处理
/// - 规范化空白字符
///
/// ## 预期结果
/// - 所有输入都被正确处理并返回预期的 slugified 字符串
#[rstest]
#[case("Hello World", "hello-world")]  // 基本输入：空格转换为连字符
#[case("test branch", "test-branch")]  // 基本输入
#[case("Test Branch Name", "test-branch-name")]  // 多个单词
#[case("test_branch", "test_branch")]  // 保留下划线
#[case("test_branch_name", "test_branch_name")]  // 多个下划线
#[case("test@branch#123", "testbranch123")]  // 移除特殊字符
#[case("test.branch", "testbranch")]  // 移除点号
#[case("", "")]  // 空字符串
#[case("  test  branch  ", "test-branch")]  // 规范化前后空格
#[case("test   branch", "test-branch")]  // 规范化多个空格
fn test_slugify_with_various_inputs_returns_slugified_string(
    #[case] input: &str,
    #[case] expected: &str,
) {
    // Arrange: 准备输入字符串（通过参数传入）

    // Act: 调用 slugify 方法
    let result = BranchNaming::slugify(input);

    // Assert: 验证返回正确的 slugified 字符串
    assert_eq!(result, expected, "Failed to slugify '{}'", input);
}

// ==================== Sanitize Tests ====================

/// 测试 sanitize 功能（参数化测试）
///
/// ## 测试目的
/// 使用参数化测试验证 BranchNaming::sanitize() 能够正确处理各种输入场景。
///
/// ## 测试场景
/// 测试以下场景：
/// - 基本输入（空格转换为连字符）
/// - 转换特殊字符为连字符
/// - 移除非 ASCII 字符
/// - 移除重复连字符
/// - 修剪前导和尾随连字符
/// - 空字符串和只包含特殊字符的输入
///
/// ## 预期结果
/// - 所有输入都被正确处理并返回预期的 sanitized 字符串
#[rstest]
#[case("Hello World", "hello-world")]  // 基本输入：空格转换为连字符
#[case("test-branch", "test-branch")]  // 已包含连字符
#[case("Test Branch", "test-branch")]  // 多个单词
#[case("test@branch#123", "test-branch-123")]  // 转换特殊字符为连字符
#[case("test.branch", "test-branch")]  // 转换点号为连字符
#[case("test_branch", "test-branch")]  // 转换下划线为连字符
#[case("测试分支", "")]  // 移除非ASCII字符（纯中文）
#[case("test中文branch", "testbranch")]  // 移除非ASCII字符（混合）
#[case("test 中文 branch", "test-branch")]  // 移除非ASCII字符（带空格）
#[case("Hello 世界", "hello")]  // 移除非ASCII字符
#[case("test---branch", "test-branch")]  // 移除重复连字符
#[case("test   branch", "test-branch")]  // 多个空格转换为单个连字符
#[case("-test-branch-", "test-branch")]  // 修剪前后连字符
#[case("--test--", "test")]  // 修剪多个前后连字符
#[case("", "")]  // 空字符串
#[case("@#$%", "")]  // 只包含特殊字符
#[case("---", "")]  // 只包含连字符
fn test_sanitize_with_various_inputs_returns_sanitized_string(
    #[case] input: &str,
    #[case] expected: &str,
) {
    // Arrange: 准备输入字符串（通过参数传入）

    // Act: 调用 sanitize 方法
    let result = BranchNaming::sanitize(input);

    // Assert: 验证返回正确的 sanitized 字符串
    assert_eq!(result, expected, "Failed to sanitize '{}'", input);
}

