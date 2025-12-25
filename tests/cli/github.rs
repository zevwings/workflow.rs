//! GitHub CLI 命令测试
//!
//! 测试 GitHub CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::GitHubSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-github")]
struct TestGitHubCli {
    #[command(subcommand)]
    command: GitHubSubcommand,
}

// ==================== Command Parsing Tests ====================

#[test]
fn test_github_command_with_all_subcommands_parses_successfully() {
    // Arrange: 准备所有子命令的输入
    let list_args = &["test-github", "list"];
    let current_args = &["test-github", "current"];
    let add_args = &["test-github", "add"];
    let remove_args = &["test-github", "remove"];
    let switch_args = &["test-github", "switch"];
    let update_args = &["test-github", "update"];

    // Act: 解析所有子命令
    let list_cli = TestGitHubCli::try_parse_from(list_args)
        .expect("CLI args should parse successfully");
    let current_cli = TestGitHubCli::try_parse_from(current_args)
        .expect("CLI args should parse successfully");
    let add_cli = TestGitHubCli::try_parse_from(add_args)
        .expect("CLI args should parse successfully");
    let remove_cli = TestGitHubCli::try_parse_from(remove_args)
        .expect("CLI args should parse successfully");
    let switch_cli = TestGitHubCli::try_parse_from(switch_args)
        .expect("CLI args should parse successfully");
    let update_cli = TestGitHubCli::try_parse_from(update_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(list_cli.command, GitHubSubcommand::List));
    assert!(matches!(current_cli.command, GitHubSubcommand::Current));
    assert!(matches!(add_cli.command, GitHubSubcommand::Add));
    assert!(matches!(remove_cli.command, GitHubSubcommand::Remove));
    assert!(matches!(switch_cli.command, GitHubSubcommand::Switch));
    assert!(matches!(update_cli.command, GitHubSubcommand::Update));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_github_command_with_invalid_subcommand_returns_error() {
    // Arrange: 准备无效子命令的输入
    let args = &["test-github", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestGitHubCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_github_command_with_missing_subcommand_returns_error() {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-github"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestGitHubCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_github_all_commands_with_extra_arguments_return_error() {
    // Arrange: 准备所有命令和额外参数
    let commands = ["list", "current", "add", "remove", "switch", "update"];

    // Act & Assert: 验证所有命令都不接受额外参数
    for cmd in commands.iter() {
        let args = &["test-github", cmd, "extra-arg"];
        let result = TestGitHubCli::try_parse_from(args);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}

#[test]
fn test_github_command_with_uppercase_subcommand_returns_error() {
    // Arrange: 准备大写子命令的输入（clap 默认区分大小写）
    let list_args = &["test-github", "LIST"];
    let current_args = &["test-github", "CURRENT"];
    let add_args = &["test-github", "ADD"];

    // Act: 尝试解析大写子命令
    let list_result = TestGitHubCli::try_parse_from(list_args);
    let current_result = TestGitHubCli::try_parse_from(current_args);
    let add_result = TestGitHubCli::try_parse_from(add_args);

    // Assert: 验证大写命令返回错误
    assert!(
        list_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
    assert!(
        current_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
    assert!(
        add_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
}

// 枚举变体完整性已通过 test_github_command_parsing_all_subcommands 测试验证
