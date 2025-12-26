//! Repo CLI 命令测试
//!
//! 测试 Repo CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
use workflow::cli::RepoSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-repo")]
struct TestRepoCli {
    #[command(subcommand)]
    command: RepoSubcommand,
}

// ==================== Command Parsing Tests ====================

/// 测试Repo命令解析所有子命令
///
/// ## 测试目的
/// 验证 `RepoSubcommand` 枚举的所有子命令（setup, show）都能够正确解析。
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
fn test_repo_command_with_all_subcommands_parses_successfully() -> Result<()> {
    // Arrange: 准备所有子命令的输入
    let setup_args = &["test-repo", "setup"];
    let show_args = &["test-repo", "show"];

    // Act: 解析所有子命令
    let setup_cli = TestRepoCli::try_parse_from(setup_args)?;
    let show_cli = TestRepoCli::try_parse_from(show_args)?;

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(setup_cli.command, RepoSubcommand::Setup));
    assert!(matches!(show_cli.command, RepoSubcommand::Show));

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试Repo命令使用无效子命令返回错误
///
/// ## 测试目的
/// 验证 `RepoSubcommand` 在使用无效子命令时能够正确返回错误。
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
fn test_repo_command_with_invalid_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备无效子命令的输入
    let args = &["test-repo", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestRepoCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");

    Ok(())
}

/// 测试Repo命令缺少子命令返回错误
///
/// ## 测试目的
/// 验证 `RepoSubcommand` 在缺少子命令时能够正确返回错误（Repo命令需要子命令）。
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
fn test_repo_command_with_missing_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-repo"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestRepoCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");

    Ok(())
}

/// 测试Repo所有命令使用额外参数返回错误
///
/// ## 测试目的
/// 验证 `RepoSubcommand` 的所有子命令（setup, show）都不接受额外参数。
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
fn test_repo_all_commands_with_extra_arguments_return_error() -> Result<()> {
    // Arrange: 准备所有命令和额外参数
    let commands = ["setup", "show"];

    // Act & Assert: 验证所有命令都不接受额外参数
    for cmd in commands.iter() {
        let args = &["test-repo", cmd, "extra-arg"];
        let result = TestRepoCli::try_parse_from(args);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }

    Ok(())
}
