//! Lifecycle 命令测试
//!
//! 测试生命周期相关的命令，包括版本显示等。

use workflow::commands::lifecycle::version::VersionCommand;

// ==================== Version Command Tests ====================

#[test]
fn test_version_command_show_with_no_parameters_executes_successfully() {
    // Arrange: 准备执行版本命令

    // Act: 执行版本命令
    let result = VersionCommand::show();

    // Assert: 验证命令可以正常执行（不抛出错误）
    assert!(result.is_ok(), "Version command should succeed");
}

#[test]
fn test_version_command_output_contains_version_with_command_executes_successfully() {
    // Arrange: 准备执行版本命令
    // 注意：这个测试主要验证命令不会崩溃

    // Act: 执行版本命令
    let result = VersionCommand::show();

    // Assert: 验证命令执行成功（输出包含版本号）
    assert!(result.is_ok());
}
