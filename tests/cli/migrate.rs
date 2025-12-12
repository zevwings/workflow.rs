//! Migrate CLI 命令测试
//!
//! 测试 Migrate CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::{Commands, DryRunArgs};

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-workflow")]
struct TestMigrateCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// ==================== Migrate 命令测试 ====================

#[test]
fn test_migrate_command_structure_with_dry_run() {
    // 测试 Migrate 命令结构（带 --dry-run 参数）
    let cli = TestMigrateCli::try_parse_from(&["test-workflow", "migrate", "--dry-run"]).unwrap();

    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(dry_run.is_dry_run(), "dry_run should be true");
            assert!(!keep_old, "keep_old should be false by default");
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_migrate_command_structure_with_keep_old() {
    // 测试 Migrate 命令结构（带 --keep-old 参数）
    let cli = TestMigrateCli::try_parse_from(&["test-workflow", "migrate", "--keep-old"]).unwrap();

    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(!dry_run.is_dry_run(), "dry_run should be false");
            assert!(keep_old, "keep_old should be true");
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_migrate_command_structure_with_both_flags() {
    // 测试 Migrate 命令结构（带 --dry-run 和 --keep-old 参数）
    let cli =
        TestMigrateCli::try_parse_from(&["test-workflow", "migrate", "--dry-run", "--keep-old"])
            .unwrap();

    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(dry_run.is_dry_run(), "dry_run should be true");
            assert!(keep_old, "keep_old should be true");
        }
        _ => panic!("Expected Migrate command"),
    }
}

#[test]
fn test_migrate_command_structure_minimal() {
    // 测试 Migrate 命令最小参数
    let cli = TestMigrateCli::try_parse_from(&["test-workflow", "migrate"]).unwrap();

    match cli.command {
        Some(Commands::Migrate { dry_run, keep_old }) => {
            assert!(!dry_run.is_dry_run(), "dry_run should be false by default");
            assert!(!keep_old, "keep_old should be false by default");
        }
        _ => panic!("Expected Migrate command"),
    }
}

// ==================== 命令解析完整性测试 ====================

#[test]
fn test_migrate_command_parsing() {
    // 测试 Migrate 命令可以正确解析
    let cli = TestMigrateCli::try_parse_from(&["test-workflow", "migrate"]).unwrap();
    assert!(matches!(cli.command, Some(Commands::Migrate { .. })));
}

#[test]
fn test_migrate_command_error_handling_extra_arguments() {
    // 测试 Migrate 命令不接受额外参数
    let result = TestMigrateCli::try_parse_from(&["test-workflow", "migrate", "extra-arg"]);
    assert!(result.is_err(), "Should fail on extra arguments");
}
