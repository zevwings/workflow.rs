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

// ==================== Update Command Tests ====================

#[test]
fn test_update_command_with_version_flag_parses_correctly() {
    // Arrange: 准备带 --version 参数的 Update 命令输入
    let args = &["test-workflow", "update", "--version", "1.2.3"];

    // Act: 解析命令行参数
    let cli = TestLifecycleCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证版本参数解析正确
    match cli.command {
        Some(Commands::Update { version }) => {
            assert_eq!(version, Some("1.2.3".to_string()), "version should be set");
        }
        _ => panic!("Expected Update command"),
    }
}

#[test]
fn test_update_command_with_short_version_flag_parses_correctly() {
    // Arrange: 准备带 -v 参数的 Update 命令输入
    let args = &["test-workflow", "update", "-v", "1.2.3"];

    // Act: 解析命令行参数
    let cli = TestLifecycleCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证短版本参数解析正确
    match cli.command {
        Some(Commands::Update { version }) => {
            assert_eq!(version, Some("1.2.3".to_string()), "version should be set");
        }
        _ => panic!("Expected Update command"),
    }
}

#[test]
fn test_update_command_with_minimal_args_parses_correctly() {
    // Arrange: 准备最小参数的 Update 命令输入
    let args = &["test-workflow", "update"];

    // Act: 解析命令行参数
    let cli = TestLifecycleCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证版本参数默认为 None
    match cli.command {
        Some(Commands::Update { version }) => {
            assert_eq!(version, None, "version should be None by default");
        }
        _ => panic!("Expected Update command"),
    }
}

// ==================== Command Parsing Tests ====================

#[test]
fn test_lifecycle_commands_with_all_subcommands_parses_successfully() {
    // Arrange: 准备所有 lifecycle 命令的输入
    let setup_args = &["test-workflow", "setup"];
    let uninstall_args = &["test-workflow", "uninstall"];
    let version_args = &["test-workflow", "version"];
    let update_args = &["test-workflow", "update"];

    // Act: 解析所有命令
    let setup_cli = TestLifecycleCli::try_parse_from(setup_args)
        .expect("CLI args should parse successfully");
    let uninstall_cli = TestLifecycleCli::try_parse_from(uninstall_args)
        .expect("CLI args should parse successfully");
    let version_cli = TestLifecycleCli::try_parse_from(version_args)
        .expect("CLI args should parse successfully");
    let update_cli = TestLifecycleCli::try_parse_from(update_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证所有 lifecycle 命令都可以正确解析
    assert!(matches!(setup_cli.command, Some(Commands::Setup)));
    assert!(matches!(uninstall_cli.command, Some(Commands::Uninstall)));
    assert!(matches!(version_cli.command, Some(Commands::Version)));
    assert!(matches!(update_cli.command, Some(Commands::Update { .. })));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_lifecycle_commands_with_extra_arguments_return_error() {
    // Arrange: 准备所有命令和额外参数
    let commands = ["setup", "uninstall", "version"];

    // Act & Assert: 验证所有命令都不接受额外参数
    for cmd in commands.iter() {
        let args = &["test-workflow", cmd, "extra-arg"];
        let result = TestLifecycleCli::try_parse_from(args);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}
