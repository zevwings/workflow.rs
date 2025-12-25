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

// ==================== Command Parsing Tests ====================

#[test]
fn test_repo_command_with_all_subcommands_parses_successfully() {
    // Arrange: 准备所有子命令的输入
    let setup_args = &["test-repo", "setup"];
    let show_args = &["test-repo", "show"];

    // Act: 解析所有子命令
    let setup_cli = TestRepoCli::try_parse_from(setup_args)
        .expect("CLI args should parse successfully");
    let show_cli = TestRepoCli::try_parse_from(show_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(setup_cli.command, RepoSubcommand::Setup));
    assert!(matches!(show_cli.command, RepoSubcommand::Show));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_repo_command_with_invalid_subcommand_returns_error() {
    // Arrange: 准备无效子命令的输入
    let args = &["test-repo", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestRepoCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_repo_command_with_missing_subcommand_returns_error() {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-repo"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestRepoCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_repo_all_commands_with_extra_arguments_return_error() {
    // Arrange: 准备所有命令和额外参数
    let commands = ["setup", "show"];

    // Act & Assert: 验证所有命令都不接受额外参数
    for cmd in commands.iter() {
        let args = &["test-repo", cmd, "extra-arg"];
        let result = TestRepoCli::try_parse_from(args);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}
