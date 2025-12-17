//! PR CLI 命令测试
//!
//! 测试 PR CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use workflow::cli::{JiraIdArg, PRCommands};

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-pr")]
struct TestPRCli {
    #[command(subcommand)]
    command: PRCommands,
}

// ==================== Fixtures ====================

#[fixture]
fn test_pr_id() -> &'static str {
    "123"
}

#[fixture]
fn test_branch() -> &'static str {
    "feature/my-branch"
}

// ==================== Create 命令测试 ====================

#[rstest]
#[case(None, None, None, false)]
#[case(Some("PROJ-123"), None, None, false)]
#[case(Some("PROJ-123"), Some("Test PR"), Some("Test description"), true)]
fn test_pr_create_command(
    #[case] jira_ticket: Option<&str>,
    #[case] title: Option<&str>,
    #[case] description: Option<&str>,
    #[case] dry_run: bool,
) {
    let mut args = vec!["test-pr", "create"];
    if let Some(ticket) = jira_ticket {
        args.push(ticket);
    }
    if let Some(t) = title {
        args.push("--title");
        args.push(t);
    }
    if let Some(d) = description {
        args.push("--description");
        args.push(d);
    }
    if dry_run {
        args.push("--dry-run");
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Create {
            jira_id: JiraIdArg { jira_id: ticket },
            title: t,
            description: d,
            dry_run: dr,
        } => {
            assert_eq!(ticket, jira_ticket.map(|s| s.to_string()));
            assert_eq!(t, title.map(|s| s.to_string()));
            assert_eq!(d, description.map(|s| s.to_string()));
            assert_eq!(dr.dry_run, dry_run);
        }
        _ => panic!("Expected Create command"),
    }
}

// ==================== Merge 命令测试 ====================

#[rstest]
#[case(None, false)]
#[case(Some("123"), false)]
#[case(Some("123"), true)]
fn test_pr_merge_command(#[case] pull_request_id: Option<&str>, #[case] force: bool) {
    let mut args = vec!["test-pr", "merge"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }
    if force {
        args.push("--force");
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Merge {
            pull_request_id: id,
            force: f,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
            assert_eq!(f, force);
        }
        _ => panic!("Expected Merge command"),
    }
}

// ==================== Status 命令测试 ====================

#[rstest]
#[case(None)]
#[case(Some("123"))]
#[case(Some("feature/my-branch"))]
fn test_pr_status_command(#[case] pull_request_id_or_branch: Option<&str>) {
    let mut args = vec!["test-pr", "status"];
    if let Some(id) = pull_request_id_or_branch {
        args.push(id);
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Status {
            pull_request_id_or_branch: id,
        } => {
            assert_eq!(id, pull_request_id_or_branch.map(|s| s.to_string()));
        }
        _ => panic!("Expected Status command"),
    }
}

// ==================== List 命令测试 ====================

#[rstest]
#[case(None, None)]
#[case(Some("open"), None)]
#[case(None, Some(10))]
#[case(Some("closed"), Some(5))]
fn test_pr_list_command(#[case] state: Option<&str>, #[case] limit: Option<usize>) {
    let mut args = vec!["test-pr", "list"];
    if let Some(s) = state {
        args.push("--state");
        args.push(s);
    }
    let limit_str = limit.map(|l| l.to_string());
    if let Some(ref l) = limit_str {
        args.push("--limit");
        args.push(l);
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::List {
            state: s,
            pagination,
        } => {
            assert_eq!(s, state.map(|s| s.to_string()));
            assert_eq!(pagination.limit, limit);
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

#[rstest]
#[case("feature/source", false, false, false)]
#[case("feature/source", true, false, false)]
#[case("feature/source", false, true, false)]
#[case("feature/source", false, false, true)]
#[case("feature/source", true, true, true)]
fn test_pr_sync_command(
    #[case] source_branch: &str,
    #[case] rebase: bool,
    #[case] ff_only: bool,
    #[case] squash: bool,
) {
    let mut args = vec!["test-pr", "sync", source_branch];
    if rebase {
        args.push("--rebase");
    }
    if ff_only {
        args.push("--ff-only");
    }
    if squash {
        args.push("--squash");
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Sync {
            source_branch: sb,
            rebase: r,
            ff_only: ff,
            squash: s,
        } => {
            assert_eq!(sb, source_branch);
            assert_eq!(r, rebase);
            assert_eq!(ff, ff_only);
            assert_eq!(s, squash);
        }
        _ => panic!("Expected Sync command"),
    }
}

// ==================== Rebase 命令测试 ====================

#[rstest]
#[case("main", false, false)]
#[case("main", true, false)]
#[case("main", false, true)]
#[case("main", true, true)]
fn test_pr_rebase_command(
    #[case] target_branch: &str,
    #[case] no_push: bool,
    #[case] dry_run: bool,
) {
    let mut args = vec!["test-pr", "rebase", target_branch];
    if no_push {
        args.push("--no-push");
    }
    if dry_run {
        args.push("--dry-run");
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Rebase {
            target_branch: tb,
            no_push: np,
            dry_run: dr,
        } => {
            assert_eq!(tb, target_branch);
            assert_eq!(np, no_push);
            assert_eq!(dr.dry_run, dry_run);
        }
        _ => panic!("Expected Rebase command"),
    }
}

// ==================== Close 命令测试 ====================

#[rstest]
#[case(None)]
#[case(Some("123"))]
fn test_pr_close_command(#[case] pull_request_id: Option<&str>) {
    let mut args = vec!["test-pr", "close"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Close {
            pull_request_id: id,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Close command"),
    }
}

// ==================== Summarize 命令测试 ====================

#[rstest]
#[case(None)]
#[case(Some("123"))]
fn test_pr_summarize_command(#[case] pull_request_id: Option<&str>) {
    let mut args = vec!["test-pr", "summarize"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Summarize {
            pull_request_id: id,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Summarize command"),
    }
}

// ==================== Approve 命令测试 ====================

#[rstest]
#[case(None)]
#[case(Some("123"))]
fn test_pr_approve_command(#[case] pull_request_id: Option<&str>) {
    let mut args = vec!["test-pr", "approve"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Approve {
            pull_request_id: id,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Approve command"),
    }
}

// ==================== Comment 命令测试 ====================

#[test]
fn test_pr_comment_command_structure() {
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
    // 测试场景：只有一个参数时，它会被解析为 PR ID，message 为空
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

#[rstest]
#[case("feature/source", "main", false)]
#[case("feature/source", "main", true)]
fn test_pr_pick_command(#[case] from_branch: &str, #[case] to_branch: &str, #[case] dry_run: bool) {
    let mut args = vec!["test-pr", "pick", from_branch, to_branch];
    if dry_run {
        args.push("--dry-run");
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Pick {
            from_branch: fb,
            to_branch: tb,
            dry_run: dr,
        } => {
            assert_eq!(fb, from_branch);
            assert_eq!(tb, to_branch);
            assert_eq!(dr.dry_run, dry_run);
        }
        _ => panic!("Expected Pick command"),
    }
}

// ==================== Reword 命令测试 ====================

#[rstest]
#[case(None, false, false, false)]
#[case(Some("456"), false, false, false)]
#[case(None, true, false, false)]
#[case(None, false, true, false)]
#[case(None, false, false, true)]
#[case(Some("123"), true, true, true)]
fn test_pr_reword_command(
    #[case] pull_request_id: Option<&str>,
    #[case] title: bool,
    #[case] description: bool,
    #[case] dry_run: bool,
) {
    let mut args = vec!["test-pr", "reword"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }
    if title {
        args.push("--title");
    }
    if description {
        args.push("--description");
    }
    if dry_run {
        args.push("--dry-run");
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();

    match cli.command {
        PRCommands::Reword {
            pull_request_id: id,
            title: t,
            description: d,
            dry_run: dr,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
            assert_eq!(t, title);
            assert_eq!(d, description);
            assert_eq!(dr.dry_run, dry_run);
        }
        _ => panic!("Expected Reword command"),
    }
}

// ==================== 命令枚举测试 ====================

#[rstest]
#[case("create", |cmd: &PRCommands| matches!(cmd, PRCommands::Create { .. }))]
#[case("merge", |cmd: &PRCommands| matches!(cmd, PRCommands::Merge { .. }))]
#[case("status", |cmd: &PRCommands| matches!(cmd, PRCommands::Status { .. }))]
#[case("list", |cmd: &PRCommands| matches!(cmd, PRCommands::List { .. }))]
#[case("update", |cmd: &PRCommands| matches!(cmd, PRCommands::Update))]
#[case("sync", |cmd: &PRCommands| matches!(cmd, PRCommands::Sync { .. }))]
#[case("rebase", |cmd: &PRCommands| matches!(cmd, PRCommands::Rebase { .. }))]
#[case("close", |cmd: &PRCommands| matches!(cmd, PRCommands::Close { .. }))]
#[case("summarize", |cmd: &PRCommands| matches!(cmd, PRCommands::Summarize { .. }))]
#[case("approve", |cmd: &PRCommands| matches!(cmd, PRCommands::Approve { .. }))]
#[case("comment", |cmd: &PRCommands| matches!(cmd, PRCommands::Comment { .. }))]
#[case("pick", |cmd: &PRCommands| matches!(cmd, PRCommands::Pick { .. }))]
#[case("reword", |cmd: &PRCommands| matches!(cmd, PRCommands::Reword { .. }))]
fn test_pr_commands_enum_all_variants(
    #[case] subcommand: &str,
    #[case] assert_fn: fn(&PRCommands) -> bool,
) {
    let mut args = vec!["test-pr", subcommand];
    // 为需要参数的命令添加最小参数
    match subcommand {
        "sync" => args.push("source"),
        "rebase" => args.push("main"),
        "pick" => {
            args.push("from");
            args.push("to");
        }
        _ => {}
    }

    let cli = TestPRCli::try_parse_from(&args).unwrap();
    assert!(
        assert_fn(&cli.command),
        "Command should match expected variant"
    );
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
