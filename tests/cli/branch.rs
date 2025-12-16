//! Branch CLI 命令测试
//!
//! 测试 Branch CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use pretty_assertions::assert_eq;
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
            assert_eq!(jira_id.into_option(), Some("PROJ-123".to_string()));
            assert!(from_default);
            assert!(dry_run.is_dry_run());
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
            assert_eq!(jira_id.into_option(), None);
            assert!(!from_default);
            assert!(!dry_run.is_dry_run());
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
            assert_eq!(jira_id.into_option(), Some("PROJ-456".to_string()));
            assert!(!from_default);
            assert!(!dry_run.is_dry_run());
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
            assert_eq!(jira_id.into_option(), None);
            assert!(from_default);
            assert!(!dry_run.is_dry_run());
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

// ==================== Rename 命令测试 ====================

#[test]
fn test_branch_rename_command() {
    // 测试 Rename 命令（无参数，完全交互式）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "rename"]).unwrap();

    match cli.command {
        BranchSubcommand::Rename => {
            // Rename 命令没有参数，完全交互式
            assert!(true, "Rename command should have no parameters");
        }
        _ => panic!("Expected Rename command"),
    }
}

// ==================== Sync 命令测试 ====================

#[test]
fn test_branch_sync_command_merge() {
    // 测试 Sync 命令（默认 merge 模式）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "sync", "master"]).unwrap();

    match cli.command {
        BranchSubcommand::Sync {
            source_branch,
            rebase,
            ff_only,
            squash,
        } => {
            assert_eq!(source_branch, "master");
            assert!(!rebase);
            assert!(!ff_only);
            assert!(!squash);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_branch_sync_command_rebase() {
    // 测试 Sync 命令（rebase 模式）
    let cli =
        TestBranchCli::try_parse_from(&["test-branch", "sync", "develop", "--rebase"]).unwrap();

    match cli.command {
        BranchSubcommand::Sync {
            source_branch,
            rebase,
            ..
        } => {
            assert_eq!(source_branch, "develop");
            assert!(rebase);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_branch_sync_command_ff_only() {
    // 测试 Sync 命令（fast-forward only 模式）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "sync", "main", "--ff-only"]).unwrap();

    match cli.command {
        BranchSubcommand::Sync {
            source_branch,
            ff_only,
            ..
        } => {
            assert_eq!(source_branch, "main");
            assert!(ff_only);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_branch_sync_command_squash() {
    // 测试 Sync 命令（squash 模式）
    let cli =
        TestBranchCli::try_parse_from(&["test-branch", "sync", "feature", "--squash"]).unwrap();

    match cli.command {
        BranchSubcommand::Sync {
            source_branch,
            squash,
            ..
        } => {
            assert_eq!(source_branch, "feature");
            assert!(squash);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_branch_sync_command_all_options() {
    // 测试 Sync 命令（所有选项，注意：rebase、ff_only、squash 互斥）
    // 在实际使用中，这些选项不应该同时使用，但测试参数解析
    let cli = TestBranchCli::try_parse_from(&[
        "test-branch",
        "sync",
        "source",
        "--rebase",
        "--ff-only",
        "--squash",
    ])
    .unwrap();

    match cli.command {
        BranchSubcommand::Sync {
            source_branch,
            rebase,
            ff_only,
            squash,
        } => {
            assert_eq!(source_branch, "source");
            // 所有选项都会被设置（虽然逻辑上互斥）
            assert!(rebase);
            assert!(ff_only);
            assert!(squash);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_branch_sync_command_required_source() {
    // 测试 Sync 命令需要 source_branch 参数
    let result = TestBranchCli::try_parse_from(&["test-branch", "sync"]);
    assert!(
        result.is_err(),
        "Sync should require source_branch parameter"
    );
}

// ==================== Delete 命令测试 ====================

#[test]
fn test_branch_delete_command() {
    // 测试 Delete 命令（提供 branch_name）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "delete", "feature/old"]).unwrap();

    match cli.command {
        BranchSubcommand::Delete {
            branch_name,
            local,
            remote,
            force,
            ..
        } => {
            assert_eq!(branch_name, Some("feature/old".to_string()));
            assert!(!local);
            assert!(!remote);
            assert!(!force);
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn test_branch_delete_command_local_only() {
    // 测试 Delete 命令（只删除本地分支）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "delete", "feature/old", "--local"])
        .unwrap();

    match cli.command {
        BranchSubcommand::Delete { local, remote, .. } => {
            assert!(local);
            assert!(!remote);
        }
        _ => panic!("Expected Delete command"),
    }
}

#[test]
fn test_branch_delete_command_force() {
    // 测试 Delete 命令（force 模式）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "delete", "feature/old", "--force"])
        .unwrap();

    match cli.command {
        BranchSubcommand::Delete { force, .. } => {
            assert!(force);
        }
        _ => panic!("Expected Delete command"),
    }
}

// ==================== Switch 命令测试 ====================

#[test]
fn test_branch_switch_command() {
    // 测试 Switch 命令（提供 branch_name）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "switch", "feature/new"]).unwrap();

    match cli.command {
        BranchSubcommand::Switch { branch_name } => {
            assert_eq!(branch_name, Some("feature/new".to_string()));
        }
        _ => panic!("Expected Switch command"),
    }
}

#[test]
fn test_branch_switch_command_interactive() {
    // 测试 Switch 命令（交互式模式，无参数）
    let cli = TestBranchCli::try_parse_from(&["test-branch", "switch"]).unwrap();

    match cli.command {
        BranchSubcommand::Switch { branch_name } => {
            assert_eq!(branch_name, None);
        }
        _ => panic!("Expected Switch command"),
    }
}
