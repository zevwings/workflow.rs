//! Commit CLI 命令测试
//!
//! 测试 Commit CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::cli::CommitSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-commit")]
struct TestCommitCli {
    #[command(subcommand)]
    command: CommitSubcommand,
}

// ==================== Amend Command Tests ====================

/// 测试Commit amend命令解析各种选项（参数化测试）
///
/// ## 测试目的
/// 验证 `CommitSubcommand::Amend` 命令能够正确解析各种选项组合（message, --no-edit, --no-verify）。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下组合：
/// - 无选项
/// - 只有message
/// - 只有--no-edit
/// - 只有--no-verify
/// - 所有选项组合
///
/// ## 预期结果
/// - 所有组合都能正确解析
/// - 参数值与预期一致
#[rstest]
#[case(None, false, false)]
#[case(Some("New commit message"), false, false)]
#[case(None, true, false)]
#[case(None, false, true)]
#[case(Some("New message"), true, true)]
fn test_commit_amend_command_with_various_options_parses_correctly(
    #[case] message: Option<&str>,
    #[case] no_edit: bool,
    #[case] no_verify: bool,
) -> Result<()> {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-commit", "amend"];
    if let Some(m) = message {
        args.push("--message");
        args.push(m);
    }
    if no_edit {
        args.push("--no-edit");
    }
    if no_verify {
        args.push("--no-verify");
    }

    // Act: 解析命令行参数
    let cli = TestCommitCli::try_parse_from(&args)?;

    // Assert: 验证参数解析正确
    match cli.command {
        CommitSubcommand::Amend {
            message: m,
            no_edit: ne,
            no_verify: nv,
        } => {
            assert_eq!(m, message.map(|s| s.to_string()));
            assert_eq!(ne, no_edit);
            assert_eq!(nv, no_verify);
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Amend command")),
    }

    Ok(())
}

// ==================== Reword Command Tests ====================

/// 测试Commit reword命令解析各种commit ID（参数化测试）
///
/// ## 测试目的
/// 验证 `CommitSubcommand::Reword` 命令能够正确解析各种格式的commit ID（可选参数）。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下情况：
/// - 无commit ID（None）
/// - 短commit ID（"abc1234"）
/// - HEAD引用（"HEAD"）
/// - 相对引用（"HEAD~2"）
/// - 完整SHA（40字符）
///
/// ## 预期结果
/// - 所有格式都能正确解析
/// - commit_id值与预期一致
#[rstest]
#[case(None)]
#[case(Some("abc1234"))]
#[case(Some("HEAD"))]
#[case(Some("HEAD~2"))]
#[case(Some("abcdef1234567890abcdef1234567890abcdef12"))]
fn test_commit_reword_command_with_various_commit_ids_parses_correctly(
    #[case] commit_id: Option<&str>,
) -> Result<()> {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-commit", "reword"];
    if let Some(id) = commit_id {
        args.push(id);
    }

    // Act: 解析命令行参数
    let cli = TestCommitCli::try_parse_from(&args)?;

    // Assert: 验证参数解析正确
    match cli.command {
        CommitSubcommand::Reword { commit_id: id } => {
            assert_eq!(id, commit_id.map(|s| s.to_string()));
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Reword command")),
    }

    Ok(())
}

// ==================== Subcommand Enum Tests ====================

/// 测试CommitSubcommand枚举包含所有子命令
///
/// ## 测试目的
/// 验证 `CommitSubcommand` 枚举包含所有预期的子命令（Amend, Reword等）。
///
/// ## 测试场景
/// 1. 解析所有子命令的输入
/// 2. 验证每个子命令都能正确解析
///
/// ## 预期结果
/// - 所有子命令都能正确解析
/// - 枚举类型匹配预期
#[test]
fn test_commit_subcommand_enum_contains_all_subcommands() -> Result<()> {
    // Arrange: 准备所有子命令的输入
    let amend_args = &["test-commit", "amend"];
    let reword_args = &["test-commit", "reword"];

    // Act: 解析所有子命令
    let amend_cli = TestCommitCli::try_parse_from(amend_args)?;
    let reword_cli = TestCommitCli::try_parse_from(reword_args)?;

    // Assert: 验证 CommitSubcommand 枚举包含所有子命令
    match amend_cli.command {
        CommitSubcommand::Amend { .. } => {}
        _ => return Err(color_eyre::eyre::eyre!("Expected Amend command")),
    }

    match reword_cli.command {
        CommitSubcommand::Reword { .. } => {}
        _ => return Err(color_eyre::eyre::eyre!("Expected Reword command")),
    }

    Ok(())
}
