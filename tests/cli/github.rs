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

// ==================== 命令结构测试 ====================

#[test]
fn test_github_command_parsing_all_subcommands() {
    // 测试所有子命令都可以正确解析

    // List
    let cli = TestGitHubCli::try_parse_from(&["test-github", "list"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, GitHubSubcommand::List));

    // Current
    let cli = TestGitHubCli::try_parse_from(&["test-github", "current"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, GitHubSubcommand::Current));

    // Add
    let cli = TestGitHubCli::try_parse_from(&["test-github", "add"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, GitHubSubcommand::Add));

    // Remove
    let cli = TestGitHubCli::try_parse_from(&["test-github", "remove"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, GitHubSubcommand::Remove));

    // Switch
    let cli = TestGitHubCli::try_parse_from(&["test-github", "switch"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, GitHubSubcommand::Switch));

    // Update
    let cli = TestGitHubCli::try_parse_from(&["test-github", "update"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, GitHubSubcommand::Update));
}

#[test]
fn test_github_command_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestGitHubCli::try_parse_from(&["test-github", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_github_command_error_handling_missing_subcommand() {
    // 测试缺少子命令的错误处理
    let result = TestGitHubCli::try_parse_from(&["test-github"]);
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_github_all_commands_no_extra_arguments() {
    // 测试所有命令都不接受额外参数
    let commands = ["list", "current", "add", "remove", "switch", "update"];

    for cmd in commands.iter() {
        let result = TestGitHubCli::try_parse_from(&["test-github", cmd, "extra-arg"]);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}

#[test]
fn test_github_command_case_sensitivity() {
    // 测试命令大小写敏感性（clap 默认区分大小写）
    // 大写命令应该失败
    let result = TestGitHubCli::try_parse_from(&["test-github", "LIST"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );

    let result = TestGitHubCli::try_parse_from(&["test-github", "CURRENT"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );

    let result = TestGitHubCli::try_parse_from(&["test-github", "ADD"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
}

// 枚举变体完整性已通过 test_github_command_parsing_all_subcommands 测试验证
