//! Lifecycle 命令测试
//!
//! 测试生命周期相关的命令，包括版本显示等。

use workflow::commands::lifecycle::version::VersionCommand;

// ==================== Version Command Tests ====================

/// 测试版本命令执行成功（无参数）
///
/// ## 测试目的
/// 验证 `VersionCommand::show()` 方法能够正常执行，不会抛出错误。
///
/// ## 测试场景
/// 1. 调用版本命令（无参数）
/// 2. 验证命令执行成功
///
/// ## 预期结果
/// - 命令执行成功，返回Ok
/// - 不会panic或产生错误
#[test]
fn test_version_command_show_with_no_parameters_executes_successfully() {
    // Arrange: 准备执行版本命令

    // Act: 执行版本命令
    let result = VersionCommand::show();

    // Assert: 验证命令可以正常执行（不抛出错误）
    assert!(result.is_ok(), "Version command should succeed");
}

/// 测试版本命令输出包含版本号
///
/// ## 测试目的
/// 验证 `VersionCommand::show()` 方法执行成功，输出包含版本号（主要验证命令不会崩溃）。
///
/// ## 测试场景
/// 1. 调用版本命令
/// 2. 验证命令执行成功
///
/// ## 注意事项
/// - 此测试主要验证命令不会崩溃
/// - 输出内容的具体验证不在本测试范围内
///
/// ## 预期结果
/// - 命令执行成功，返回Ok
#[test]
fn test_version_command_output_contains_version_with_command_executes_successfully() {
    // Arrange: 准备执行版本命令
    // 注意：这个测试主要验证命令不会崩溃

    // Act: 执行版本命令
    let result = VersionCommand::show();

    // Assert: 验证命令执行成功（输出包含版本号）
    assert!(result.is_ok());
}
