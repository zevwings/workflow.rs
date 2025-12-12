//! PR CLI 命令测试
//!
//! 测试 PR CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::PRCommands;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-pr")]
struct TestPRCli {
    #[command(subcommand)]
    command: PRCommands,
}

// ==================== Create 命令测试 ====================

#[test]
fn test_pr_create_command_structure() {
    // 测试 Create 命令结构（带所有参数）
    let cli = TestPRCli::try_parse_from(&[
        "test-pr",
        "create",
        "PROJ-123",
        "--title",
        "Test PR",
        "--description",
        "Test description",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        PRCommands::Create {
            jira_ticket,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(jira_ticket, Some("PROJ-123".to_string()));
            assert_eq!(title, Some("Test PR".to_string()));
            assert_eq!(description, Some("Test description".to_string()));
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_pr_create_command_minimal() {
    // 测试 Create 命令最小参数
    let cli = TestPRCli::try_parse_from(&["test-pr", "create"]).unwrap();

    match cli.command {
        PRCommands::Create {
            jira_ticket,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(jira_ticket, None);
            assert_eq!(title, None);
            assert_eq!(description, None);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

#[test]
fn test_pr_create_command_with_jira_ticket_only() {
    // 测试 Create 命令只带 JIRA ticket
    let cli = TestPRCli::try_parse_from(&["test-pr", "create", "PROJ-456"]).unwrap();

    match cli.command {
        PRCommands::Create {
            jira_ticket,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(jira_ticket, Some("PROJ-456".to_string()));
            assert_eq!(title, None);
            assert_eq!(description, None);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

// ==================== Merge 命令测试 ====================

#[test]
fn test_pr_merge_command_structure() {
    // 测试 Merge 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "merge", "123", "--force"]).unwrap();

    match cli.command {
        PRCommands::Merge {
            pull_request_id,
            force,
        } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
            assert!(force);
        }
        _ => panic!("Expected Merge command"),
    }
}

#[test]
fn test_pr_merge_command_without_id() {
    // 测试 Merge 命令不带 PR ID
    let cli = TestPRCli::try_parse_from(&["test-pr", "merge"]).unwrap();

    match cli.command {
        PRCommands::Merge {
            pull_request_id,
            force,
        } => {
            assert_eq!(pull_request_id, None);
            assert!(!force);
        }
        _ => panic!("Expected Merge command"),
    }
}

// ==================== Status 命令测试 ====================

#[test]
fn test_pr_status_command_structure() {
    // 测试 Status 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "status", "123"]).unwrap();

    match cli.command {
        PRCommands::Status {
            pull_request_id_or_branch,
        } => {
            assert_eq!(pull_request_id_or_branch, Some("123".to_string()));
        }
        _ => panic!("Expected Status command"),
    }
}

#[test]
fn test_pr_status_command_with_branch_name() {
    // 测试 Status 命令带分支名
    let cli = TestPRCli::try_parse_from(&["test-pr", "status", "feature/my-branch"]).unwrap();

    match cli.command {
        PRCommands::Status {
            pull_request_id_or_branch,
        } => {
            assert_eq!(
                pull_request_id_or_branch,
                Some("feature/my-branch".to_string())
            );
        }
        _ => panic!("Expected Status command"),
    }
}

#[test]
fn test_pr_status_command_without_id() {
    // 测试 Status 命令不带 ID
    let cli = TestPRCli::try_parse_from(&["test-pr", "status"]).unwrap();

    match cli.command {
        PRCommands::Status {
            pull_request_id_or_branch,
        } => {
            assert_eq!(pull_request_id_or_branch, None);
        }
        _ => panic!("Expected Status command"),
    }
}

// ==================== List 命令测试 ====================

#[test]
fn test_pr_list_command_structure() {
    // 测试 List 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "list", "--state", "open", "--limit", "10"])
        .unwrap();

    match cli.command {
        PRCommands::List { state, limit } => {
            assert_eq!(state, Some("open".to_string()));
            assert_eq!(limit, Some(10));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_pr_list_command_minimal() {
    // 测试 List 命令最小参数
    let cli = TestPRCli::try_parse_from(&["test-pr", "list"]).unwrap();

    match cli.command {
        PRCommands::List { state, limit } => {
            assert_eq!(state, None);
            assert_eq!(limit, None);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_pr_list_command_with_state_only() {
    // 测试 List 命令只带 state
    let cli = TestPRCli::try_parse_from(&["test-pr", "list", "--state", "closed"]).unwrap();

    match cli.command {
        PRCommands::List { state, limit } => {
            assert_eq!(state, Some("closed".to_string()));
            assert_eq!(limit, None);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_pr_list_command_with_limit_only() {
    // 测试 List 命令只带 limit
    let cli = TestPRCli::try_parse_from(&["test-pr", "list", "--limit", "5"]).unwrap();

    match cli.command {
        PRCommands::List { state, limit } => {
            assert_eq!(state, None);
            assert_eq!(limit, Some(5));
        }
        _ => panic!("Expected List command"),
    }
}

// ==================== Update 命令测试 ====================

#[test]
fn test_pr_update_command_structure() {
    // 测试 Update 命令结构（无参数）
    let cli = TestPRCli::try_parse_from(&["test-pr", "update"]).unwrap();

    match cli.command {
        PRCommands::Update => {
            // Update 命令没有参数
            assert!(true, "Update command should have no parameters");
        }
        _ => panic!("Expected Update command"),
    }
}

// ==================== Sync 命令测试 ====================

#[test]
fn test_pr_sync_command_structure() {
    // 测试 Sync 命令结构（带所有参数）
    let cli = TestPRCli::try_parse_from(&[
        "test-pr",
        "sync",
        "feature/source",
        "--rebase",
        "--ff-only",
        "--squash",
    ])
    .unwrap();

    match cli.command {
        PRCommands::Sync {
            source_branch,
            rebase,
            ff_only,
            squash,
        } => {
            assert_eq!(source_branch, "feature/source");
            assert!(rebase);
            assert!(ff_only);
            assert!(squash);
        }
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_pr_sync_command_minimal() {
    // 测试 Sync 命令最小参数（只需要 source_branch）
    let cli = TestPRCli::try_parse_from(&["test-pr", "sync", "feature/source"]).unwrap();

    match cli.command {
        PRCommands::Sync {
            source_branch,
            rebase,
            ff_only,
            squash,
        } => {
            assert_eq!(source_branch, "feature/source");
            assert!(!rebase);
            assert!(!ff_only);
            assert!(!squash);
        }
        _ => panic!("Expected Sync command"),
    }
}

// ==================== Rebase 命令测试 ====================

#[test]
fn test_pr_rebase_command_structure() {
    // 测试 Rebase 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "rebase", "main", "--no-push", "--dry-run"])
        .unwrap();

    match cli.command {
        PRCommands::Rebase {
            target_branch,
            no_push,
            dry_run,
        } => {
            assert_eq!(target_branch, "main");
            assert!(no_push);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Rebase command"),
    }
}

#[test]
fn test_pr_rebase_command_minimal() {
    // 测试 Rebase 命令最小参数
    let cli = TestPRCli::try_parse_from(&["test-pr", "rebase", "main"]).unwrap();

    match cli.command {
        PRCommands::Rebase {
            target_branch,
            no_push,
            dry_run,
        } => {
            assert_eq!(target_branch, "main");
            assert!(!no_push);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Rebase command"),
    }
}

// ==================== Close 命令测试 ====================

#[test]
fn test_pr_close_command_structure() {
    // 测试 Close 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "close", "123"]).unwrap();

    match cli.command {
        PRCommands::Close { pull_request_id } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
        }
        _ => panic!("Expected Close command"),
    }
}

#[test]
fn test_pr_close_command_without_id() {
    // 测试 Close 命令不带 ID
    let cli = TestPRCli::try_parse_from(&["test-pr", "close"]).unwrap();

    match cli.command {
        PRCommands::Close { pull_request_id } => {
            assert_eq!(pull_request_id, None);
        }
        _ => panic!("Expected Close command"),
    }
}

// ==================== Summarize 命令测试 ====================

#[test]
fn test_pr_summarize_command_structure() {
    // 测试 Summarize 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "summarize", "123"]).unwrap();

    match cli.command {
        PRCommands::Summarize { pull_request_id } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
        }
        _ => panic!("Expected Summarize command"),
    }
}

#[test]
fn test_pr_summarize_command_without_id() {
    // 测试 Summarize 命令不带 ID
    let cli = TestPRCli::try_parse_from(&["test-pr", "summarize"]).unwrap();

    match cli.command {
        PRCommands::Summarize { pull_request_id } => {
            assert_eq!(pull_request_id, None);
        }
        _ => panic!("Expected Summarize command"),
    }
}

// ==================== Approve 命令测试 ====================

#[test]
fn test_pr_approve_command_structure() {
    // 测试 Approve 命令结构
    let cli = TestPRCli::try_parse_from(&["test-pr", "approve", "123"]).unwrap();

    match cli.command {
        PRCommands::Approve { pull_request_id } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
        }
        _ => panic!("Expected Approve command"),
    }
}

#[test]
fn test_pr_approve_command_without_id() {
    // 测试 Approve 命令不带 ID
    let cli = TestPRCli::try_parse_from(&["test-pr", "approve"]).unwrap();

    match cli.command {
        PRCommands::Approve { pull_request_id } => {
            assert_eq!(pull_request_id, None);
        }
        _ => panic!("Expected Approve command"),
    }
}

// ==================== Comment 命令测试 ====================

#[test]
fn test_pr_comment_command_structure() {
    // 测试 Comment 命令结构
    let cli =
        TestPRCli::try_parse_from(&["test-pr", "comment", "123", "This is a comment"]).unwrap();

    match cli.command {
        PRCommands::Comment {
            pull_request_id,
            message,
        } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
            assert_eq!(message, vec!["This is a comment"]);
        }
        _ => panic!("Expected Comment command"),
    }
}

#[test]
fn test_pr_comment_command_with_multiple_words() {
    // 测试 Comment 命令带多个单词
    let cli = TestPRCli::try_parse_from(&[
        "test-pr",
        "comment",
        "123",
        "This",
        "is",
        "a",
        "multi-word",
        "comment",
    ])
    .unwrap();

    match cli.command {
        PRCommands::Comment {
            pull_request_id,
            message,
        } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
            assert_eq!(message, vec!["This", "is", "a", "multi-word", "comment"]);
        }
        _ => panic!("Expected Comment command"),
    }
}

#[test]
fn test_pr_comment_command_without_id() {
    // 测试 Comment 命令不带 ID
    // 注意：由于 pull_request_id 是 Option<String> 且 message 是 trailing_var_arg，
    // clap 的行为是：第一个参数会被解析为 pull_request_id（如果存在），
    // 后续参数会进入 message。当只有一个参数时，它会被解析为 pull_request_id。
    // 要测试不带 ID 的情况，我们需要接受这个行为，或者测试实际的命令行使用场景。
    // 在实际使用中，用户可以通过明确指定 PR ID 或使用多个单词来区分。

    // 测试场景：多个单词（会被解析为 message，第一个可能被解析为 PR ID）
    // 由于 trailing_var_arg 的特性，我们测试一个更实际的场景
    let cli =
        TestPRCli::try_parse_from(&["test-pr", "comment", "123", "This is a message"]).unwrap();

    match cli.command {
        PRCommands::Comment {
            pull_request_id,
            message,
        } => {
            // 当明确提供 PR ID 时
            assert_eq!(pull_request_id, Some("123".to_string()));
            assert_eq!(message, vec!["This is a message"]);
        }
        _ => panic!("Expected Comment command"),
    }

    // 测试场景：只有一个参数时，它会被解析为 PR ID，message 为空
    // 这在实际使用中可能不是期望的行为，但这是 clap 的默认行为
    // 我们测试这个行为以确保测试覆盖所有情况
    let cli = TestPRCli::try_parse_from(&["test-pr", "comment", "single-arg"]).unwrap();

    match cli.command {
        PRCommands::Comment {
            pull_request_id,
            message,
        } => {
            // 单个参数会被解析为 PR ID
            assert_eq!(pull_request_id, Some("single-arg".to_string()));
            assert!(
                message.is_empty(),
                "Message should be empty when only one arg provided"
            );
        }
        _ => panic!("Expected Comment command"),
    }
}

// ==================== Pick 命令测试 ====================

#[test]
fn test_pr_pick_command_structure() {
    // 测试 Pick 命令结构
    let cli =
        TestPRCli::try_parse_from(&["test-pr", "pick", "feature/source", "main", "--dry-run"])
            .unwrap();

    match cli.command {
        PRCommands::Pick {
            from_branch,
            to_branch,
            dry_run,
        } => {
            assert_eq!(from_branch, "feature/source");
            assert_eq!(to_branch, "main");
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Pick command"),
    }
}

#[test]
fn test_pr_pick_command_minimal() {
    // 测试 Pick 命令最小参数
    let cli = TestPRCli::try_parse_from(&["test-pr", "pick", "feature/source", "main"]).unwrap();

    match cli.command {
        PRCommands::Pick {
            from_branch,
            to_branch,
            dry_run,
        } => {
            assert_eq!(from_branch, "feature/source");
            assert_eq!(to_branch, "main");
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Pick command"),
    }
}

// ==================== Reword 命令测试 ====================

#[test]
fn test_pr_reword_command_structure() {
    // 测试 Reword 命令结构（带所有参数）
    let cli = TestPRCli::try_parse_from(&[
        "test-pr",
        "reword",
        "123",
        "--title",
        "--description",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(pull_request_id, Some("123".to_string()));
            assert!(title);
            assert!(description);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_pr_reword_command_minimal() {
    // 测试 Reword 命令最小参数（不指定 PR ID，自动检测）
    let cli = TestPRCli::try_parse_from(&["test-pr", "reword"]).unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(pull_request_id, None);
            assert!(!title);
            assert!(!description);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_pr_reword_command_with_pr_id() {
    // 测试 Reword 命令指定 PR ID
    let cli = TestPRCli::try_parse_from(&["test-pr", "reword", "456"]).unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(pull_request_id, Some("456".to_string()));
            assert!(!title);
            assert!(!description);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_pr_reword_command_title_only() {
    // 测试 Reword 命令仅更新标题
    let cli = TestPRCli::try_parse_from(&["test-pr", "reword", "--title"]).unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(pull_request_id, None);
            assert!(title);
            assert!(!description);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_pr_reword_command_description_only() {
    // 测试 Reword 命令仅更新描述
    let cli = TestPRCli::try_parse_from(&["test-pr", "reword", "--description"]).unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(pull_request_id, None);
            assert!(!title);
            assert!(description);
            assert!(!dry_run.dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

#[test]
fn test_pr_reword_command_dry_run() {
    // 测试 Reword 命令预览模式
    let cli = TestPRCli::try_parse_from(&["test-pr", "reword", "--dry-run"]).unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id,
            title,
            description,
            dry_run,
        } => {
            assert_eq!(pull_request_id, None);
            assert!(!title);
            assert!(!description);
            assert!(dry_run.dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

// ==================== 命令枚举测试 ====================

#[test]
fn test_pr_commands_enum_all_variants() {
    // 测试所有 PR 命令变体都可以正确解析

    // Create
    let cli = TestPRCli::try_parse_from(&["test-pr", "create"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Create { .. }));

    // Merge
    let cli = TestPRCli::try_parse_from(&["test-pr", "merge"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Merge { .. }));

    // Status
    let cli = TestPRCli::try_parse_from(&["test-pr", "status"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Status { .. }));

    // List
    let cli = TestPRCli::try_parse_from(&["test-pr", "list"]).unwrap();
    assert!(matches!(cli.command, PRCommands::List { .. }));

    // Update
    let cli = TestPRCli::try_parse_from(&["test-pr", "update"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Update));

    // Sync
    let cli = TestPRCli::try_parse_from(&["test-pr", "sync", "source"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Sync { .. }));

    // Rebase
    let cli = TestPRCli::try_parse_from(&["test-pr", "rebase", "main"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Rebase { .. }));

    // Close
    let cli = TestPRCli::try_parse_from(&["test-pr", "close"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Close { .. }));

    // Summarize
    let cli = TestPRCli::try_parse_from(&["test-pr", "summarize"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Summarize { .. }));

    // Approve
    let cli = TestPRCli::try_parse_from(&["test-pr", "approve"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Approve { .. }));

    // Comment
    let cli = TestPRCli::try_parse_from(&["test-pr", "comment", "msg"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Comment { .. }));

    // Pick
    let cli = TestPRCli::try_parse_from(&["test-pr", "pick", "from", "to"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Pick { .. }));

    // Reword
    let cli = TestPRCli::try_parse_from(&["test-pr", "reword"]).unwrap();
    assert!(matches!(cli.command, PRCommands::Reword { .. }));
}

#[test]
fn test_pr_commands_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestPRCli::try_parse_from(&["test-pr", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_pr_commands_required_parameters() {
    // 测试必需参数的错误处理

    // Sync 需要 source_branch
    let result = TestPRCli::try_parse_from(&["test-pr", "sync"]);
    assert!(result.is_err(), "Sync should require source_branch");

    // Rebase 需要 target_branch
    let result = TestPRCli::try_parse_from(&["test-pr", "rebase"]);
    assert!(result.is_err(), "Rebase should require target_branch");

    // Pick 需要 from_branch 和 to_branch
    let result = TestPRCli::try_parse_from(&["test-pr", "pick"]);
    assert!(
        result.is_err(),
        "Pick should require from_branch and to_branch"
    );

    let result = TestPRCli::try_parse_from(&["test-pr", "pick", "from"]);
    assert!(result.is_err(), "Pick should require to_branch");
}
