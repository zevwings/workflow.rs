//! PR CLI 命令测试
//!
//! 测试 PR CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::cli::{JiraIdArg, PRCommands};

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-pr")]
struct TestPRCli {
    #[command(subcommand)]
    command: PRCommands,
}

// ==================== Create Command Tests ====================

/// 测试PR创建命令的参数解析（各种选项组合）
#[rstest]
#[case(None, None, None, false)]
#[case(Some("PROJ-123"), None, None, false)]
#[case(Some("PROJ-123"), Some("Test PR"), Some("Test description"), true)]
fn test_pr_create_command_with_various_options_parses_correctly(
    #[case] jira_ticket: Option<&str>,
    #[case] title: Option<&str>,
    #[case] description: Option<&str>,
    #[case] dry_run: bool,
) {
    // Arrange: 准备命令行参数
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

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
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

// ==================== Merge Command Tests ====================

/// 测试PR合并命令的参数解析（包含force选项）
#[rstest]
#[case(None, false)]
#[case(Some("123"), false)]
#[case(Some("123"), true)]
fn test_pr_merge_command_with_various_options_parses_correctly(
    #[case] pull_request_id: Option<&str>,
    #[case] force: bool,
) {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-pr", "merge"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }
    if force {
        args.push("--force");
    }

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
    match cli.command {
        PRCommands::Merge {
            pull_request_id: id,
            force: f,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
            assert_eq!(f.is_force(), force);
        }
        _ => panic!("Expected Merge command"),
    }
}

// ==================== Status Command Tests ====================

/// 测试PR状态命令的参数解析（支持PR ID或分支名）
#[rstest]
#[case(None)]
#[case(Some("123"))]
#[case(Some("feature/my-branch"))]
fn test_pr_status_command_with_various_inputs_parses_correctly(
    #[case] pull_request_id_or_branch: Option<&str>,
) {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-pr", "status"];
    if let Some(id) = pull_request_id_or_branch {
        args.push(id);
    }

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
    match cli.command {
        PRCommands::Status {
            pull_request_id_or_branch: id,
        } => {
            assert_eq!(id, pull_request_id_or_branch.map(|s| s.to_string()));
        }
        _ => panic!("Expected Status command"),
    }
}

// ==================== List Command Tests ====================

/// 测试PR列表命令的参数解析（包含state和limit选项）
#[rstest]
#[case(None, None)]
#[case(Some("open"), None)]
#[case(None, Some(10))]
#[case(Some("closed"), Some(5))]
fn test_pr_list_command_with_various_options_parses_correctly(
    #[case] state: Option<&str>,
    #[case] limit: Option<usize>,
) {
    // Arrange: 准备命令行参数
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

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
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

// ==================== Update Command Tests ====================

/// 测试PR更新命令的参数解析（无参数）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_update_command_with_valid_input_parses_successfully() {
    // Arrange: 准备有效的 Update 命令输入（无参数）
    let args = &["test-pr", "update"];

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证 Update 命令可以正确解析
    assert!(matches!(cli.command, PRCommands::Update));
}

// ==================== Sync Command Tests ====================

/// 测试PR同步命令的参数解析（包含rebase、ff_only、squash选项）
#[rstest]
#[case("feature/source", false, false, false)]
#[case("feature/source", true, false, false)]
#[case("feature/source", false, true, false)]
#[case("feature/source", false, false, true)]
#[case("feature/source", true, true, true)]
fn test_pr_sync_command_with_various_options_parses_correctly(
    #[case] source_branch: &str,
    #[case] rebase: bool,
    #[case] ff_only: bool,
    #[case] squash: bool,
) {
    // Arrange: 准备命令行参数
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

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
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

// ==================== Rebase Command Tests ====================

/// 测试PR变基命令的参数解析（包含no_push和dry_run选项）
#[rstest]
#[case("main", false, false)]
#[case("main", true, false)]
#[case("main", false, true)]
#[case("main", true, true)]
fn test_pr_rebase_command_with_various_options_parses_correctly(
    #[case] target_branch: &str,
    #[case] no_push: bool,
    #[case] dry_run: bool,
) {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-pr", "rebase", target_branch];
    if no_push {
        args.push("--no-push");
    }
    if dry_run {
        args.push("--dry-run");
    }

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
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

// ==================== Close Command Tests ====================

/// 测试PR关闭命令的参数解析（可选PR ID）
#[rstest]
#[case(None)]
#[case(Some("123"))]
fn test_pr_close_command_with_various_inputs_parses_correctly(
    #[case] pull_request_id: Option<&str>,
) {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-pr", "close"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
    match cli.command {
        PRCommands::Close {
            pull_request_id: id,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Close command"),
    }
}

// ==================== Summarize Command Tests ====================

/// 测试PR摘要命令的参数解析（可选PR ID）
#[rstest]
#[case(None)]
#[case(Some("123"))]
fn test_pr_summarize_command_with_various_inputs_parses_correctly(
    #[case] pull_request_id: Option<&str>,
) {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-pr", "summarize"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
    match cli.command {
        PRCommands::Summarize {
            pull_request_id: id,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Summarize command"),
    }
}

// ==================== Approve Command Tests ====================

/// 测试PR批准命令的参数解析（可选PR ID）
#[rstest]
#[case(None)]
#[case(Some("123"))]
fn test_pr_approve_command_with_various_inputs_parses_correctly(
    #[case] pull_request_id: Option<&str>,
) {
    // Arrange: 准备命令行参数
    let mut args = vec!["test-pr", "approve"];
    if let Some(id) = pull_request_id {
        args.push(id);
    }

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
    match cli.command {
        PRCommands::Approve {
            pull_request_id: id,
        } => {
            assert_eq!(id, pull_request_id.map(|s| s.to_string()));
        }
        _ => panic!("Expected Approve command"),
    }
}

// ==================== Comment Command Tests ====================

/// 测试PR评论命令的参数解析（包含PR ID和消息）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_comment_command_with_pr_id_and_message_parses_correctly() {
    // Arrange: 准备带 PR ID 和消息的 Comment 命令输入
    let args = &["test-pr", "comment", "123", "This is a comment"];

    // Act: 解析命令行参数
    let cli = TestPRCli::try_parse_from(args)
        .expect("CLI args should parse successfully");

    // Assert: 验证参数解析正确
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

/// 测试PR评论命令的参数解析（多词消息）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_comment_command_with_multiple_words_parses_correctly() {
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
    .expect("CLI args should parse successfully");

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

/// 测试PR评论命令的参数解析（只有单个参数，无消息）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_comment_command_without_id() {
    // Arrange: 准备测试场景：只有一个参数时，它会被解析为 PR ID，message 为空
    let cli = TestPRCli::try_parse_from(&["test-pr", "comment", "single-arg"])
        .expect("CLI args should parse successfully");

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

/// 测试PR选择命令的参数解析（包含dry_run选项）
#[rstest]
#[case("feature/source", "main", false)]
#[case("feature/source", "main", true)]
fn test_pr_pick_command(#[case] from_branch: &str, #[case] to_branch: &str, #[case] dry_run: bool) {
    let mut args = vec!["test-pr", "pick", from_branch, to_branch];
    if dry_run {
        args.push("--dry-run");
    }

    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

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

/// 测试PR重写命令的参数解析（包含title、description、dry_run选项）
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

    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");

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

// ==================== Command Enum Tests ====================

/// 测试PR命令枚举的所有变体
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

    let cli = TestPRCli::try_parse_from(&args).expect("CLI args should parse successfully");
    assert!(
        assert_fn(&cli.command),
        "Command should match expected variant"
    );
}

/// 测试PR命令的错误处理（无效子命令）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_commands_error_handling_invalid_subcommand() {
    // Arrange: 准备测试无效子命令的错误处理
    let result = TestPRCli::try_parse_from(&["test-pr", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

/// 测试PR命令的必需参数验证
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_commands_required_parameters() {
    // Arrange: 准备测试必需参数的错误处理

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

// ==================== Boundary Condition Tests ====================

/// 测试PR创建命令的空JIRA ID（应被验证器拒绝）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_create_command_empty_jira_id() {
    // Arrange: 准备测试空字符串 JIRA ID（应该被验证器拒绝）
    // 这是正确的行为：JIRA ID 验证器不允许空字符串
    let result = TestPRCli::try_parse_from(&["test-pr", "create", ""]);

    // Assert: 验证解析失败（空字符串被验证器拒绝）
    match result {
        Ok(_) => panic!("Empty JIRA ID should be rejected by validator"),
        Err(e) => {
            // Assert: 验证错误消息包含验证信息
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

/// 测试PR创建命令的超长标题（边界情况）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_create_command_very_long_title() {
    // Arrange: 准备测试超长标题（边界情况）
    let long_title = "a".repeat(1000);
    let cli = TestPRCli::try_parse_from(&["test-pr", "create", "PROJ-123", "--title", &long_title])
        .expect("CLI args should parse successfully");

    match cli.command {
        PRCommands::Create { title, .. } => {
            assert_eq!(title, Some(long_title));
        }
        _ => panic!("Expected Create command"),
    }
}

/// 测试PR创建命令标题中的特殊字符
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_create_command_special_characters_in_title() {
    // Arrange: 准备测试标题中的特殊字符
    let special_title = "Test PR: Fix bug #123 (urgent!)";
    let cli =
        TestPRCli::try_parse_from(&["test-pr", "create", "PROJ-123", "--title", special_title])
            .expect("CLI args should parse successfully");

    match cli.command {
        PRCommands::Create { title, .. } => {
            assert_eq!(title, Some(special_title.to_string()));
        }
        _ => panic!("Expected Create command"),
    }
}

/// 测试PR评论命令的空消息（边界情况）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_comment_command_empty_message() {
    // Arrange: 准备测试空消息的情况
    let cli = TestPRCli::try_parse_from(&["test-pr", "comment", "123"])
        .expect("CLI args should parse successfully");

    match cli.command {
        PRCommands::Comment { message, .. } => {
            assert!(
                message.is_empty(),
                "Message should be empty when not provided"
            );
        }
        _ => panic!("Expected Comment command"),
    }
}

/// 测试PR列表命令的limit为0（边界值）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_list_command_zero_limit() {
    // Arrange: 准备测试 limit 为 0 的情况（边界值）
    let cli = TestPRCli::try_parse_from(&["test-pr", "list", "--limit", "0"])
        .expect("CLI args should parse successfully");

    match cli.command {
        PRCommands::List { pagination, .. } => {
            assert_eq!(pagination.limit, Some(0));
        }
        _ => panic!("Expected List command"),
    }
}

/// 测试PR列表命令的超大limit值（边界情况）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_list_command_very_large_limit() {
    // Arrange: 准备测试非常大的 limit 值（边界情况）
    let cli = TestPRCli::try_parse_from(&["test-pr", "list", "--limit", "999999"])
        .expect("CLI args should parse successfully");

    match cli.command {
        PRCommands::List { pagination, .. } => {
            assert_eq!(pagination.limit, Some(999999));
        }
        _ => panic!("Expected List command"),
    }
}
