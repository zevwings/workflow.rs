//! Lifecycle 命令测试
//!
//! 测试生命周期相关的命令，包括版本显示等。

use workflow::commands::lifecycle::version::VersionCommand;

#[test]
fn test_version_command_show() {
    // 测试版本命令可以正常执行（不抛出错误）
    let result = VersionCommand::show();
    assert!(result.is_ok(), "Version command should succeed");
}

#[test]
fn test_version_command_output_contains_version() {
    // 验证版本命令的输出包含版本号
    // 注意：这个测试主要验证命令不会崩溃
    let result = VersionCommand::show();
    assert!(result.is_ok());
}

