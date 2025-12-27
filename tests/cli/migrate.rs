//! Migrate CLI 命令测试
//!
//! 测试 Migrate CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
use workflow::cli::Commands;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-workflow")]
struct TestMigrateCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// ==================== Migrate Command Tests ====================

/// 测试Migrate命令解析--dry-run标志
///
/// ## 测试目的
/// 验证 `Commands::Migrate` 命令能够正确解析 `--dry-run` 标志。
///
/// ## 测试场景
/// 1. 准备包含 `--dry-run` 标志的命令行输入
/// 2. 解析命令行参数
/// 3. 验证 `dry_run` 标志正确设置，`keep_old` 为默认值
///
/// ## 预期结果
/// - 解析成功
/// - dry_run为true
/// - keep_old为false（默认值）
#[test]
fn test_migrate_command_with_dry_run_flag_parses_correctly() -> Result<()> {
    // Arrange: 准备带 --dry-run 参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate", "--dry-run"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)?;

    // Assert: 验证 --dry-run 参数解析正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(dry_run.is_dry_run(), "dry_run should be true");
            assert!(!keep_old, "keep_old should be false by default");
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Migrate command")),
    }

    Ok(())
}

/// 测试Migrate命令解析--keep-old标志
///
/// ## 测试目的
/// 验证 `Commands::Migrate` 命令能够正确解析 `--keep-old` 标志。
///
/// ## 测试场景
/// 1. 准备包含 `--keep-old` 标志的命令行输入
/// 2. 解析命令行参数
/// 3. 验证 `keep_old` 标志正确设置，`dry_run` 为默认值
///
/// ## 预期结果
/// - 解析成功
/// - keep_old为true
/// - dry_run为false（默认值）
#[test]
fn test_migrate_command_with_keep_old_flag_parses_correctly() -> Result<()> {
    // Arrange: 准备带 --keep-old 参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate", "--keep-old"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)?;

    // Assert: 验证 --keep-old 参数解析正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(!dry_run.is_dry_run(), "dry_run should be false");
            assert!(keep_old, "keep_old should be true");
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Migrate command")),
    }

    Ok(())
}

/// 测试Migrate命令解析两个标志组合
///
/// ## 测试目的
/// 验证 `Commands::Migrate` 命令能够同时解析 `--dry-run` 和 `--keep-old` 两个标志。
///
/// ## 测试场景
/// 1. 准备包含两个标志的命令行输入
/// 2. 解析命令行参数
/// 3. 验证两个标志都正确设置
///
/// ## 预期结果
/// - 解析成功
/// - dry_run为true
/// - keep_old为true
#[test]
fn test_migrate_command_with_both_flags_parses_correctly() -> Result<()> {
    // Arrange: 准备带 --dry-run 和 --keep-old 参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate", "--dry-run", "--keep-old"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)?;

    // Assert: 验证两个参数都解析正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(dry_run.is_dry_run(), "dry_run should be true");
            assert!(keep_old, "keep_old should be true");
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Migrate command")),
    }

    Ok(())
}

/// 测试Migrate命令解析最小参数
///
/// ## 测试目的
/// 验证 `Commands::Migrate` 命令在使用最小参数（只有命令名）时能够正确解析，所有标志使用默认值。
///
/// ## 测试场景
/// 1. 准备只包含命令名的命令行输入
/// 2. 解析命令行参数
/// 3. 验证所有标志为默认值
///
/// ## 预期结果
/// - 解析成功
/// - dry_run为false（默认值）
/// - keep_old为false（默认值）
#[test]
fn test_migrate_command_with_minimal_args_parses_correctly() -> Result<()> {
    // Arrange: 准备最小参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)?;

    // Assert: 验证默认值正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(!dry_run.is_dry_run(), "dry_run should be false by default");
            assert!(!keep_old, "keep_old should be false by default");
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Migrate command")),
    }

    Ok(())
}

// ==================== Command Parsing Tests ====================

/// 测试Migrate命令解析有效输入
///
/// ## 测试目的
/// 验证 `Commands::Migrate` 命令能够正确解析有效的命令行输入。
///
/// ## 测试场景
/// 1. 准备有效的Migrate命令输入
/// 2. 解析命令行参数
/// 3. 验证命令解析成功
///
/// ## 预期结果
/// - 解析成功
/// - 命令类型为 `Commands::Migrate`
#[test]
fn test_migrate_command_with_valid_input_parses_successfully() -> Result<()> {
    // Arrange: 准备有效的 Migrate 命令输入
    let args = &["test-workflow", "migrate"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)?;

    // Assert: 验证 Migrate 命令可以正确解析
    assert!(matches!(cli.command, Some(Commands::Migrate { .. })));

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试Migrate命令使用额外参数返回错误
///
/// ## 测试目的
/// 验证 `Commands::Migrate` 命令在使用额外参数时能够正确返回错误（Migrate命令不接受额外参数）。
///
/// ## 测试场景
/// 1. 准备包含额外参数的命令行输入
/// 2. 尝试解析命令行参数
/// 3. 验证解析失败
///
/// ## 预期结果
/// - 解析失败，返回错误
/// - 错误消息明确指示不接受额外参数
#[test]
fn test_migrate_command_with_extra_arguments_returns_error() -> Result<()> {
    // Arrange: 准备包含额外参数的输入
    let args = &["test-workflow", "migrate", "extra-arg"];

    // Act: 尝试解析包含额外参数的命令
    let result = TestMigrateCli::try_parse_from(args);

    // Assert: 验证 Migrate 命令不接受额外参数，返回错误
    assert!(result.is_err(), "Should fail on extra arguments");

    Ok(())
}
