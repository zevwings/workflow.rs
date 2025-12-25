//! Stash CLI 命令测试
//!
//! 测试 Stash CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::StashSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-stash")]
struct TestStashCli {
    #[command(subcommand)]
    command: StashSubcommand,
}

// ==================== List Command Tests ====================

#[test]
fn test_stash_list_command_with_stat_flag_parses_correctly() {
    // Arrange: 准备带 --stat 参数的 List 命令输入
    let args = &["test-stash", "list", "--stat"];

    // Act: 解析命令行参数
    let cli = TestStashCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证 --stat 参数解析正确
    match cli.command {
        StashSubcommand::List { stat } => {
            assert!(stat, "stat should be true");
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_stash_list_command_with_minimal_args_parses_correctly() {
    // Arrange: 准备最小参数的 List 命令输入
    let args = &["test-stash", "list"];

    // Act: 解析命令行参数
    let cli = TestStashCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证默认值正确
    match cli.command {
        StashSubcommand::List { stat } => {
            assert!(!stat, "stat should be false by default");
        }
        _ => panic!("Expected List command"),
    }
}

// ==================== Command Parsing Tests ====================

#[test]
fn test_stash_command_with_all_subcommands_parses_successfully() {
    // Arrange: 准备所有子命令的输入
    let list_args = &["test-stash", "list"];
    let apply_args = &["test-stash", "apply"];
    let drop_args = &["test-stash", "drop"];
    let pop_args = &["test-stash", "pop"];
    let push_args = &["test-stash", "push"];

    // Act: 解析所有子命令
    let list_cli = TestStashCli::try_parse_from(list_args)
        .expect("CLI args should parse successfully");
    let apply_cli = TestStashCli::try_parse_from(apply_args)
        .expect("CLI args should parse successfully");
    let drop_cli = TestStashCli::try_parse_from(drop_args)
        .expect("CLI args should parse successfully");
    let pop_cli = TestStashCli::try_parse_from(pop_args)
        .expect("CLI args should parse successfully");
    let push_cli = TestStashCli::try_parse_from(push_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(list_cli.command, StashSubcommand::List { .. }));
    assert!(matches!(apply_cli.command, StashSubcommand::Apply));
    assert!(matches!(drop_cli.command, StashSubcommand::Drop));
    assert!(matches!(pop_cli.command, StashSubcommand::Pop));
    assert!(matches!(push_cli.command, StashSubcommand::Push));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_stash_command_with_invalid_subcommand_returns_error() {
    // Arrange: 准备无效子命令的输入
    let args = &["test-stash", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestStashCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_stash_command_with_missing_subcommand_returns_error() {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-stash"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestStashCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");
}
