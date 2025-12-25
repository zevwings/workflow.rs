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

/// 测试GitHub命令解析所有子命令
///
/// ## 测试目的
/// 验证 `GitHubSubcommand` 枚举的所有子命令（list, current, add, remove, switch, update）都能够正确解析。
///
/// ## 测试场景
/// 1. 准备所有子命令的输入
/// 2. 解析所有子命令
/// 3. 验证每个子命令都能正确解析
///
/// ## 预期结果
/// - 所有子命令都能正确解析
/// - 命令类型匹配预期
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

/// 测试GitHub命令使用无效子命令返回错误
///
/// ## 测试目的
/// 验证 `GitHubSubcommand` 在使用无效子命令时能够正确返回错误。
///
/// ## 测试场景
/// 1. 准备无效子命令的输入（"invalid"）
/// 2. 尝试解析命令行参数
/// 3. 验证解析失败
///
/// ## 预期结果
/// - 解析失败，返回错误
/// - 错误消息明确指示无效子命令
#[test]
fn test_github_command_with_invalid_subcommand_returns_error() {
    // Arrange: 准备无效子命令的输入
    let args = &["test-github", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestGitHubCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

/// 测试GitHub命令缺少子命令返回错误
///
/// ## 测试目的
/// 验证 `GitHubSubcommand` 在缺少子命令时能够正确返回错误（GitHub命令需要子命令）。
///
/// ## 测试场景
/// 1. 准备缺少子命令的输入（只有命令名）
/// 2. 尝试解析命令行参数
/// 3. 验证解析失败
///
/// ## 预期结果
/// - 解析失败，返回错误
/// - 错误消息明确指示缺少子命令
#[test]
fn test_github_command_with_missing_subcommand_returns_error() {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-github"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestGitHubCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

/// 测试GitHub所有命令使用额外参数返回错误
///
/// ## 测试目的
/// 验证 `GitHubSubcommand` 的所有子命令都不接受额外参数。
///
/// ## 测试场景
/// 1. 遍历所有子命令
/// 2. 为每个命令添加额外参数
/// 3. 验证所有命令都拒绝额外参数
///
/// ## 预期结果
/// - 所有命令在使用额外参数时都返回错误
/// - 错误消息明确指示不接受额外参数
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

/// 测试GitHub命令使用大写子命令返回错误
///
/// ## 测试目的
/// 验证 `GitHubSubcommand` 在使用大写子命令时能够正确返回错误（clap默认区分大小写）。
///
/// ## 测试场景
/// 1. 准备大写子命令的输入（LIST, CURRENT, ADD等）
/// 2. 尝试解析命令行参数
/// 3. 验证解析失败
///
/// ## 注意事项
/// - clap默认区分大小写
/// - 大写命令应该返回错误
///
/// ## 预期结果
/// - 所有大写命令都返回错误
/// - 错误消息明确指示命令无效
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
