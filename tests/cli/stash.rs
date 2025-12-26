//! Stash CLI 命令测试
//!
//! 测试 Stash CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
use workflow::cli::StashSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-stash")]
struct TestStashCli {
    #[command(subcommand)]
    command: StashSubcommand,
}

// ==================== List Command Tests ====================

/// 测试Stash list命令解析--stat标志
///
/// ## 测试目的
/// 验证 `StashSubcommand::List` 命令能够正确解析 `--stat` 标志。
///
/// ## 测试场景
/// 1. 准备包含 `--stat` 标志的命令行输入
/// 2. 解析命令行参数
/// 3. 验证 `stat` 标志正确设置
///
/// ## 预期结果
/// - 解析成功
/// - stat为true
#[test]
fn test_stash_list_command_with_stat_flag_parses_correctly() -> Result<()> {
    // Arrange: 准备带 --stat 参数的 List 命令输入
    let args = &["test-stash", "list", "--stat"];

    // Act: 解析命令行参数
    let cli = TestStashCli::try_parse_from(args)?;

    // Assert: 验证 --stat 参数解析正确
    match cli.command {
        StashSubcommand::List { stat } => {
            assert!(stat, "stat should be true");
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected List command")),
    }

    Ok(())
}

/// 测试Stash list命令解析最小参数
///
/// ## 测试目的
/// 验证 `StashSubcommand::List` 命令在使用最小参数（只有命令名）时能够正确解析，`stat` 标志使用默认值（false）。
///
/// ## 测试场景
/// 1. 准备只包含命令名的命令行输入
/// 2. 解析命令行参数
/// 3. 验证 `stat` 标志为默认值
///
/// ## 预期结果
/// - 解析成功
/// - stat为false（默认值）
#[test]
fn test_stash_list_command_with_minimal_args_parses_correctly() -> Result<()> {
    // Arrange: 准备最小参数的 List 命令输入
    let args = &["test-stash", "list"];

    // Act: 解析命令行参数
    let cli = TestStashCli::try_parse_from(args)?;

    // Assert: 验证默认值正确
    match cli.command {
        StashSubcommand::List { stat } => {
            assert!(!stat, "stat should be false by default");
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected List command")),
    }

    Ok(())
}

// ==================== Command Parsing Tests ====================

/// 测试Stash命令解析所有子命令
///
/// ## 测试目的
/// 验证 `StashSubcommand` 枚举的所有子命令（list, apply, drop, pop, push）都能够正确解析。
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
fn test_stash_command_with_all_subcommands_parses_successfully() -> Result<()> {
    // Arrange: 准备所有子命令的输入
    let list_args = &["test-stash", "list"];
    let apply_args = &["test-stash", "apply"];
    let drop_args = &["test-stash", "drop"];
    let pop_args = &["test-stash", "pop"];
    let push_args = &["test-stash", "push"];

    // Act: 解析所有子命令
    let list_cli = TestStashCli::try_parse_from(list_args)?;
    let apply_cli = TestStashCli::try_parse_from(apply_args)?;
    let drop_cli = TestStashCli::try_parse_from(drop_args)?;
    let pop_cli = TestStashCli::try_parse_from(pop_args)?;
    let push_cli = TestStashCli::try_parse_from(push_args)?;

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(list_cli.command, StashSubcommand::List { .. }));
    assert!(matches!(apply_cli.command, StashSubcommand::Apply));
    assert!(matches!(drop_cli.command, StashSubcommand::Drop));
    assert!(matches!(pop_cli.command, StashSubcommand::Pop));
    assert!(matches!(push_cli.command, StashSubcommand::Push));

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试Stash命令使用无效子命令返回错误
///
/// ## 测试目的
/// 验证 `StashSubcommand` 在使用无效子命令时能够正确返回错误。
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
fn test_stash_command_with_invalid_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备无效子命令的输入
    let args = &["test-stash", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestStashCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");

    Ok(())
}

/// 测试Stash命令缺少子命令返回错误
///
/// ## 测试目的
/// 验证 `StashSubcommand` 在缺少子命令时能够正确返回错误（Stash命令需要子命令）。
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
fn test_stash_command_with_missing_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-stash"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestStashCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");

    Ok(())
}
