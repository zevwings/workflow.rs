//! Lifecycle CLI 命令测试
//!
//! 测试 Lifecycle CLI 命令的参数解析、命令执行流程和错误处理。
//! 包括 Setup, Uninstall, Version, Update 等命令。

use clap::Parser;
use pretty_assertions::assert_eq;
use workflow::cli::Commands;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-workflow")]
struct TestLifecycleCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// ==================== Update 命令测试 ====================

#[test]
fn test_update_command_structure_with_version() {
    // 测试 Update 命令结构（带 --version 参数）
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "update", "--version", "1.2.3"])
        .expect("CLI args should parse successfully");

    match cli.command {
        Some(Commands::Update { version }) => {
            assert_eq!(version, Some("1.2.3".to_string()), "version should be set");
        }
        _ => panic!("Expected Update command"),
    }
}

#[test]
fn test_update_command_structure_with_short_version() {
    // 测试 Update 命令结构（带 -v 参数）
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "update", "-v", "1.2.3"])
        .expect("CLI args should parse successfully");

    match cli.command {
        Some(Commands::Update { version }) => {
            assert_eq!(version, Some("1.2.3".to_string()), "version should be set");
        }
        _ => panic!("Expected Update command"),
    }
}

#[test]
fn test_update_command_structure_minimal() {
    // 测试 Update 命令最小参数
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "update"])
        .expect("CLI args should parse successfully");

    match cli.command {
        Some(Commands::Update { version }) => {
            assert_eq!(version, None, "version should be None by default");
        }
        _ => panic!("Expected Update command"),
    }
}

// ==================== 命令解析完整性测试 ====================

#[test]
fn test_lifecycle_commands_parsing() {
    // 测试所有 lifecycle 命令都可以正确解析

    // Setup
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "setup"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, Some(Commands::Setup)));

    // Uninstall
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "uninstall"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, Some(Commands::Uninstall)));

    // Version
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "version"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, Some(Commands::Version)));

    // Update
    let cli = TestLifecycleCli::try_parse_from(&["test-workflow", "update"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, Some(Commands::Update { .. })));
}

#[test]
fn test_lifecycle_commands_error_handling_extra_arguments() {
    // 测试 lifecycle 命令不接受额外参数
    let commands = ["setup", "uninstall", "version"];

    for cmd in commands.iter() {
        let result = TestLifecycleCli::try_parse_from(&["test-workflow", cmd, "extra-arg"]);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}
