//! Repo CLI 命令测试
//!
//! 测试 Repo CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::RepoSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-repo")]
struct TestRepoCli {
    #[command(subcommand)]
    command: RepoSubcommand,
}

// ==================== 命令解析完整性测试 ====================

#[test]
fn test_repo_command_parsing_all_subcommands() {
    // 测试所有子命令都可以正确解析

    // Setup
    let cli = TestRepoCli::try_parse_from(&["test-repo", "setup"]).unwrap();
    assert!(matches!(cli.command, RepoSubcommand::Setup));

    // Show
    let cli = TestRepoCli::try_parse_from(&["test-repo", "show"]).unwrap();
    assert!(matches!(cli.command, RepoSubcommand::Show));
}

#[test]
fn test_repo_command_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestRepoCli::try_parse_from(&["test-repo", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_repo_command_error_handling_missing_subcommand() {
    // 测试缺少子命令的错误处理
    let result = TestRepoCli::try_parse_from(&["test-repo"]);
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_repo_all_commands_no_extra_arguments() {
    // 测试所有命令都不接受额外参数
    let commands = ["setup", "show"];

    for cmd in commands.iter() {
        let result = TestRepoCli::try_parse_from(&["test-repo", cmd, "extra-arg"]);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}
