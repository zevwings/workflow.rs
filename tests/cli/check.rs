//! Check CLI 命令测试
//!
//! 测试 Check CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::Commands;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-workflow")]
struct TestCheckCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// ==================== Command Parsing Tests ====================

#[test]
fn test_check_command_with_valid_input_parses_successfully() {
    // Arrange: 准备有效的 Check 命令输入
    let args = &["test-workflow", "check"];

    // Act: 解析命令行参数
    let cli = TestCheckCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证 Check 命令可以正确解析
    assert!(matches!(cli.command, Some(Commands::Check)));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_check_command_with_extra_arguments_returns_error() {
    // Arrange: 准备包含额外参数的输入
    let args = &["test-workflow", "check", "extra-arg"];

    // Act: 尝试解析包含额外参数的命令
    let result = TestCheckCli::try_parse_from(args);

    // Assert: 验证 Check 命令不接受额外参数，返回错误
    assert!(result.is_err(), "Should fail on extra arguments");
}
