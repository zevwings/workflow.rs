//! Commit CLI 命令测试
//!
//! 测试 Commit CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::CommitSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-commit")]
struct TestCommitCli {
    #[command(subcommand)]
    command: CommitSubcommand,
}

// ==================== Amend 命令测试 ====================

#[test]
fn test_commit_amend_command_structure() {
    // 测试 Amend 命令结构（带所有参数）
    let cli = TestCommitCli::try_parse_from(&[
        "test-commit",
        "amend",
        "--message",
        "New commit message",
        "--no-verify",
    ])
    .unwrap();

    match cli.command {
        CommitSubcommand::Amend {
            message,
            no_edit,
            no_verify,
        } => {
            assert_eq!(message, Some("New commit message".to_string()));
            assert!(!no_edit);
            assert!(no_verify);
        }
        _ => panic!("Expected Amend command"),
    }
}

#[test]
fn test_commit_amend_command_minimal() {
    // 测试 Amend 命令最小参数
    let cli = TestCommitCli::try_parse_from(&["test-commit", "amend"]).unwrap();

    match cli.command {
        CommitSubcommand::Amend {
            message,
            no_edit,
            no_verify,
        } => {
            assert_eq!(message, None);
            assert!(!no_edit);
            assert!(!no_verify);
        }
        _ => panic!("Expected Amend command"),
    }
}

#[test]
fn test_commit_amend_command_with_no_edit() {
    // 测试 Amend 命令带 --no-edit 参数
    let cli = TestCommitCli::try_parse_from(&["test-commit", "amend", "--no-edit"]).unwrap();

    match cli.command {
        CommitSubcommand::Amend {
            message,
            no_edit,
            no_verify,
        } => {
            assert_eq!(message, None);
            assert!(no_edit);
            assert!(!no_verify);
        }
        _ => panic!("Expected Amend command"),
    }
}

#[test]
fn test_commit_amend_command_with_message_and_no_edit() {
    // 测试 Amend 命令同时带 --message 和 --no-edit（--no-edit 优先级更高）
    let cli = TestCommitCli::try_parse_from(&[
        "test-commit",
        "amend",
        "--message",
        "New message",
        "--no-edit",
    ])
    .unwrap();

    match cli.command {
        CommitSubcommand::Amend {
            message,
            no_edit,
            no_verify,
        } => {
            assert_eq!(message, Some("New message".to_string()));
            assert!(no_edit);
            assert!(!no_verify);
        }
        _ => panic!("Expected Amend command"),
    }
}

#[test]
fn test_commit_amend_command_with_all_flags() {
    // 测试 Amend 命令带所有标志
    let cli = TestCommitCli::try_parse_from(&[
        "test-commit",
        "amend",
        "--message",
        "New message",
        "--no-edit",
        "--no-verify",
    ])
    .unwrap();

    match cli.command {
        CommitSubcommand::Amend {
            message,
            no_edit,
            no_verify,
        } => {
            assert_eq!(message, Some("New message".to_string()));
            assert!(no_edit);
            assert!(no_verify);
        }
        _ => panic!("Expected Amend command"),
    }
}

// ==================== Reword 命令测试 ====================

#[test]
fn test_commit_reword_command_structure() {
    // 测试 Reword 命令结构（带 commit ID）
    let cli = TestCommitCli::try_parse_from(&[
        "test-commit",
        "reword",
        "abc1234",
    ])
    .unwrap();

    match cli.command {
        CommitSubcommand::Reword { commit_id } => {
            assert_eq!(commit_id, Some("abc1234".to_string()));
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_commit_reword_command_minimal() {
    // 测试 Reword 命令最小参数（默认 HEAD）
    let cli = TestCommitCli::try_parse_from(&["test-commit", "reword"]).unwrap();

    match cli.command {
        CommitSubcommand::Reword { commit_id } => {
            assert_eq!(commit_id, None);
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_commit_reword_command_with_head() {
    // 测试 Reword 命令显式指定 HEAD
    let cli = TestCommitCli::try_parse_from(&["test-commit", "reword", "HEAD"]).unwrap();

    match cli.command {
        CommitSubcommand::Reword { commit_id } => {
            assert_eq!(commit_id, Some("HEAD".to_string()));
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_commit_reword_command_with_head_tilde() {
    // 测试 Reword 命令使用 HEAD~n 格式
    let cli = TestCommitCli::try_parse_from(&["test-commit", "reword", "HEAD~2"]).unwrap();

    match cli.command {
        CommitSubcommand::Reword { commit_id } => {
            assert_eq!(commit_id, Some("HEAD~2".to_string()));
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_commit_reword_command_with_full_sha() {
    // 测试 Reword 命令使用完整 SHA
    let cli = TestCommitCli::try_parse_from(&[
        "test-commit",
        "reword",
        "abcdef1234567890abcdef1234567890abcdef12",
    ])
    .unwrap();

    match cli.command {
        CommitSubcommand::Reword { commit_id } => {
            assert_eq!(
                commit_id,
                Some("abcdef1234567890abcdef1234567890abcdef12".to_string())
            );
        }
        _ => panic!("Expected Reword command"),
    }
}

// ==================== 命令枚举测试 ====================

#[test]
fn test_commit_subcommand_enum() {
    // 测试 CommitSubcommand 枚举包含所有子命令
    let amend_cli = TestCommitCli::try_parse_from(&["test-commit", "amend"]).unwrap();
    let reword_cli = TestCommitCli::try_parse_from(&["test-commit", "reword"]).unwrap();

    match amend_cli.command {
        CommitSubcommand::Amend { .. } => {}
        _ => panic!("Expected Amend command"),
    }

    match reword_cli.command {
        CommitSubcommand::Reword { .. } => {}
        _ => panic!("Expected Reword command"),
    }
}
