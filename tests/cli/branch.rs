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
    .expect("CLI args should parse successfully");

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
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create"]).expect("CLI args should parse successfully");

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
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "PROJ-456"]).expect("CLI args should parse successfully");

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
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "--from-default"]).expect("CLI args should parse successfully");

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
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", "--dry-run"]).expect("CLI args should parse successfully");

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

// ==================== 边界情况测试 ====================

#[test]
fn test_branch_create_command_empty_jira_id() {
    // 测试空字符串 JIRA ID（应该被验证器拒绝）
    // 这是正确的行为：JIRA ID 验证器不允许空字符串
    let result = TestBranchCli::try_parse_from(&["test-branch", "create", ""]);

    // 验证解析失败（空字符串被验证器拒绝）
    match result {
        Ok(_) => panic!("Empty JIRA ID should be rejected by validator"),
        Err(e) => {
            // 验证错误消息包含验证信息
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("JIRA ID")
                    || error_msg.contains("empty")
                    || error_msg.contains("Invalid")
                    || error_msg.contains("validation"),
                "Error message should indicate JIRA ID validation failure: {}",
                error_msg
            );
        }
    }
}

#[test]
fn test_branch_create_command_very_long_jira_id() {
    // 测试超长 JIRA ID（边界情况）
    let long_jira_id = "PROJ-".to_string() + &"1".repeat(100);
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", &long_jira_id]).expect("CLI args should parse successfully");

    match cli.command {
        BranchSubcommand::Create { jira_id, .. } => {
            assert_eq!(jira_id.into_option(), Some(long_jira_id));
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_branch_create_command_special_characters_in_jira_id() {
    // 测试 JIRA ID 中的特殊字符（边界情况）
    // 注意：实际业务逻辑可能会验证 JIRA ID 格式，但 CLI 解析应该接受任何字符串
    let special_jira_id = "PROJ-123_test@example.com";
    let cli = TestBranchCli::try_parse_from(&["test-branch", "create", special_jira_id]).expect("CLI args should parse successfully");

    match cli.command {
        BranchSubcommand::Create { jira_id, .. } => {
            assert_eq!(jira_id.into_option(), Some(special_jira_id.to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}
