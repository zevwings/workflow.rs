//! Commit CLI 命令测试
//!
//! 测试 Commit CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
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

// ==================== Amend 命令测试 ====================

#[rstest]
#[case(None, false, false)]
#[case(Some("New commit message"), false, false)]
#[case(None, true, false)]
#[case(None, false, true)]
#[case(Some("New message"), true, true)]
fn test_commit_amend_command(
    #[case] message: Option<&str>,
    #[case] no_edit: bool,
    #[case] no_verify: bool,
) {
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

    let cli = TestCommitCli::try_parse_from(&args).expect("CLI args should parse successfully");

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
        _ => panic!("Expected Amend command"),
    }
}

// ==================== Reword 命令测试 ====================

#[rstest]
#[case(None)]
#[case(Some("abc1234"))]
#[case(Some("HEAD"))]
#[case(Some("HEAD~2"))]
#[case(Some("abcdef1234567890abcdef1234567890abcdef12"))]
fn test_commit_reword_command(#[case] commit_id: Option<&str>) {
    let mut args = vec!["test-commit", "reword"];
    if let Some(id) = commit_id {
        args.push(id);
    }

    let cli = TestCommitCli::try_parse_from(&args).expect("CLI args should parse successfully");

    match cli.command {
        CommitSubcommand::Reword { commit_id: id } => {
            assert_eq!(id, commit_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Reword command"),
    }
}

// ==================== 命令枚举测试 ====================

#[test]
fn test_commit_subcommand_enum() {
    // 测试 CommitSubcommand 枚举包含所有子命令
    let amend_cli = TestCommitCli::try_parse_from(&["test-commit", "amend"]).expect("CLI args should parse successfully");
    let reword_cli = TestCommitCli::try_parse_from(&["test-commit", "reword"]).expect("CLI args should parse successfully");

    match amend_cli.command {
        CommitSubcommand::Amend { .. } => {}
        _ => panic!("Expected Amend command"),
    }

    match reword_cli.command {
        CommitSubcommand::Reword { .. } => {}
        _ => panic!("Expected Reword command"),
    }
}
