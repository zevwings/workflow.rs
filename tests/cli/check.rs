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

// ==================== Check 命令测试 ====================

// ==================== 命令解析完整性测试 ====================

#[test]
fn test_check_command_parsing() {
    // 测试 Check 命令可以正确解析
    let cli = TestCheckCli::try_parse_from(&["test-workflow", "check"]).expect("CLI args should parse successfully");
    assert!(matches!(cli.command, Some(Commands::Check)));
}

#[test]
fn test_check_command_error_handling_extra_arguments() {
    // 测试 Check 命令不接受额外参数
    let result = TestCheckCli::try_parse_from(&["test-workflow", "check", "extra-arg"]);
    assert!(result.is_err(), "Should fail on extra arguments");
}
