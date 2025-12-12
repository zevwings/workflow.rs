//! Branch CLI 命令测试
//!
//! 测试 Branch CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::BranchSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-branch")]
struct TestBranchCli {
    #[command(subcommand)]
    command: BranchSubcommand,
}

// ==================== Create 命令测试 ====================

#[test]
fn test_branch_create_command_structure() {
    // 测试 Create 命令结构（带所有参数）
    let cli = TestBranchCli::try_parse_from(&[
        "test-branch",
        "create",
        "PROJ-123",
        "--from-default",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-123".to_string()));
            assert!(from_default);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_minimal() {
    // 测试 Create 命令最小参数
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(!from_default);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_with_jira_ticket_only() {
    // 测试 Create 命令只带 JIRA ticket
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "PROJ-456"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-456".to_string()));
            assert!(!from_default);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_with_from_default() {
    // 测试 Create 命令带 --from-default 参数
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "--from-default"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(from_default);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_with_dry_run() {
    // 测试 Create 命令带 --dry-run 参数
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "--dry-run"]).unwrap();

    match cli.command {
        BranchSubcommand::Create {
            jira_id,
            from_default,
            dry_run,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(!from_default);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}
