//! Migrate CLI 命令测试
//!
//! 测试 Migrate CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::Commands;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-workflow")]
struct TestMigrateCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// ==================== Migrate Command Tests ====================

#[test]
fn test_migrate_command_with_dry_run_flag_parses_correctly() {
    // Arrange: 准备带 --dry-run 参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate", "--dry-run"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证 --dry-run 参数解析正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(dry_run.is_dry_run(), "dry_run should be true");
            assert!(!keep_old, "keep_old should be false by default");
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_migrate_command_with_keep_old_flag_parses_correctly() {
    // Arrange: 准备带 --keep-old 参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate", "--keep-old"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证 --keep-old 参数解析正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(!dry_run.is_dry_run(), "dry_run should be false");
            assert!(keep_old, "keep_old should be true");
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_migrate_command_with_both_flags_parses_correctly() {
    // Arrange: 准备带 --dry-run 和 --keep-old 参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate", "--dry-run", "--keep-old"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证两个参数都解析正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(dry_run.is_dry_run(), "dry_run should be true");
            assert!(keep_old, "keep_old should be true");
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_migrate_command_with_minimal_args_parses_correctly() {
    // Arrange: 准备最小参数的 Migrate 命令输入
    let args = &["test-workflow", "migrate"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证默认值正确
    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(!dry_run.is_dry_run(), "dry_run should be false by default");
            assert!(!keep_old, "keep_old should be false by default");
        }
        _ => panic!("Expected Migrate command"),
    }
}

// ==================== Command Parsing Tests ====================

#[test]
fn test_migrate_command_with_valid_input_parses_successfully() {
    // Arrange: 准备有效的 Migrate 命令输入
    let args = &["test-workflow", "migrate"];

    // Act: 解析命令行参数
    let cli = TestMigrateCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证 Migrate 命令可以正确解析
    assert!(matches!(cli.command, Some(Commands::Migrate { .. })));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_migrate_command_with_extra_arguments_returns_error() {
    // Arrange: 准备包含额外参数的输入
    let args = &["test-workflow", "migrate", "extra-arg"];

    // Act: 尝试解析包含额外参数的命令
    let result = TestMigrateCli::try_parse_from(args);

    // Assert: 验证 Migrate 命令不接受额外参数，返回错误
    assert!(result.is_err(), "Should fail on extra arguments");
}
