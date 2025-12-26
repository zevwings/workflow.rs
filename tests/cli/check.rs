//! Check CLI 命令测试
//!
//! 测试 Check CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
use workflow::cli::Commands;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-workflow")]
struct TestCheckCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// ==================== Command Parsing Tests ====================

/// 测试Check命令解析有效输入
///
/// ## 测试目的
/// 验证 `Commands::Check` 命令能够正确解析有效的命令行输入。
///
/// ## 测试场景
/// 1. 准备有效的Check命令输入（"check"）
/// 2. 解析命令行参数
/// 3. 验证命令解析成功
///
/// ## 预期结果
/// - 解析成功
/// - 命令类型为 `Commands::Check`
#[test]
fn test_check_command_with_valid_input_parses_successfully() -> Result<()> {
    // Arrange: 准备有效的 Check 命令输入
    let args = &["test-workflow", "check"];

    // Act: 解析命令行参数
    let cli = TestCheckCli::try_parse_from(args)?;

    // Assert: 验证 Check 命令可以正确解析
    assert!(matches!(cli.command, Some(Commands::Check)));

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试Check命令使用额外参数返回错误
///
/// ## 测试目的
/// 验证 `Commands::Check` 命令在使用额外参数时能够正确返回错误（Check命令不接受额外参数）。
///
/// ## 测试场景
/// 1. 准备包含额外参数的命令行输入（"check extra-arg"）
/// 2. 尝试解析命令行参数
/// 3. 验证解析失败
///
/// ## 预期结果
/// - 解析失败，返回错误
/// - 错误消息明确指示不接受额外参数
#[test]
fn test_check_command_with_extra_arguments_returns_error() -> Result<()> {
    // Arrange: 准备包含额外参数的输入
    let args = &["test-workflow", "check", "extra-arg"];

    // Act: 尝试解析包含额外参数的命令
    let result = TestCheckCli::try_parse_from(args);

    // Assert: 验证 Check 命令不接受额外参数，返回错误
    assert!(result.is_err(), "Should fail on extra arguments");

    Ok(())
}
