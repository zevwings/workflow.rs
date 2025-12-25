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

/// 测试Update命令解析--version标志
///
/// ## 测试目的
/// 验证 `Commands::Update` 命令能够正确解析 `--version` 标志和版本号参数。
///
/// ## 测试场景
/// 1. 准备包含 `--version` 标志和版本号的命令行输入
/// 2. 解析命令行参数
/// 3. 验证版本号正确设置
///
/// ## 预期结果
/// - 解析成功
/// - version为指定的版本号（"1.2.3"）
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

/// 测试Update命令解析-v短标志
///
/// ## 测试目的
/// 验证 `Commands::Update` 命令能够正确解析 `-v` 短标志和版本号参数。
///
/// ## 测试场景
/// 1. 准备包含 `-v` 短标志和版本号的命令行输入
/// 2. 解析命令行参数
/// 3. 验证版本号正确设置
///
/// ## 预期结果
/// - 解析成功
/// - version为指定的版本号（"1.2.3"）
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

/// 测试Update命令解析最小参数
///
/// ## 测试目的
/// 验证 `Commands::Update` 命令在使用最小参数（只有命令名）时能够正确解析，version参数使用默认值（None）。
///
/// ## 测试场景
/// 1. 准备只包含命令名的命令行输入
/// 2. 解析命令行参数
/// 3. 验证version参数为默认值
///
/// ## 预期结果
/// - 解析成功
/// - version为None（默认值）
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

/// 测试Lifecycle命令解析所有子命令
///
/// ## 测试目的
/// 验证 `Commands` 枚举的所有lifecycle子命令（setup, uninstall, version, update）都能够正确解析。
///
/// ## 测试场景
/// 1. 准备所有lifecycle命令的输入
/// 2. 解析所有命令
/// 3. 验证每个命令都能正确解析
///
/// ## 预期结果
/// - 所有lifecycle命令都能正确解析
/// - 命令类型匹配预期
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

/// 测试Lifecycle命令使用额外参数返回错误
///
/// ## 测试目的
/// 验证 `Commands` 的lifecycle子命令（setup, uninstall, version）都不接受额外参数。
///
/// ## 测试场景
/// 1. 遍历所有lifecycle命令
/// 2. 为每个命令添加额外参数
/// 3. 验证所有命令都拒绝额外参数
///
/// ## 预期结果
/// - 所有命令在使用额外参数时都返回错误
/// - 错误消息明确指示不接受额外参数
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
