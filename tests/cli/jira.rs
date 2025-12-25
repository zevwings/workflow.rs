//! Jira CLI 命令测试
//!
//! 测试 Jira CLI 命令的参数解析、命令执行流程和错误处理。
//!
//! ## 测试策略
//!
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 使用 `rstest` 进行参数化测试
//! - 测试所有 Jira 子命令的参数解析

use clap::Parser;
use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use workflow::cli::JiraSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-jira")]
struct TestJiraCli {
    #[command(subcommand)]
    command: JiraSubcommand,
}

// ==================== Fixtures ====================

#[fixture]
fn test_jira_id() -> &'static str {
    "PROJ-123"
}

#[fixture]
fn test_jira_id_alt() -> &'static str {
    "PROJ-456"
}

// ==================== 命令结构测试 ====================

/// 测试Jira子命令的参数解析（带Jira ID）
///
/// ## 测试目的
/// 验证所有Jira子命令能够正确解析和存储Jira ticket ID参数。
///
/// ## 测试场景（参数化测试）
/// 使用`rstest`对以下子命令进行参数化测试：
/// - `info`: 查询issue信息
/// - `related`: 查询关联issue
/// - `changelog`: 查询变更历史
/// - `comment`: 添加评论
/// - `comments`: 查询评论列表
/// - `attachments`: 查询附件列表
///
/// ## 测试方法
/// 1. 使用`clap::Parser`解析命令行参数
/// 2. 使用模式匹配验证解析结果
/// 3. 确认Jira ID被正确存储在对应的字段中
///
/// ## 技术细节
/// - 使用`#[rstest]`和`#[case]`进行参数化测试
/// - 每个case运行一次独立的测试
/// - 使用模式匹配处理不同的子命令枚举变体
///
/// ## 预期结果
/// 每个子命令的`jira_id`字段应包含传入的Jira ID
#[rstest]
#[case("info", "PROJ-123")]
#[case("related", "PROJ-123")]
#[case("changelog", "PROJ-123")]
#[case("comment", "PROJ-123")]
#[case("comments", "PROJ-123")]
#[case("attachments", "PROJ-456")]
fn test_jira_command_with_id(#[case] subcommand: &str, #[case] jira_id: &str) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", subcommand, jira_id])?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert_eq!(args.jira_id.jira_id, Some(jira_id.to_string()));
        }
        JiraSubcommand::Related { args, .. } => {
            assert_eq!(args.jira_id.jira_id, Some(jira_id.to_string()));
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert_eq!(args.jira_id.jira_id, Some(jira_id.to_string()));
        }
        JiraSubcommand::Comment { jira_id: id } => {
            assert_eq!(id.jira_id, Some(jira_id.to_string()));
        }
        JiraSubcommand::Comments { jira_id: id, .. } => {
            assert_eq!(id.jira_id, Some(jira_id.to_string()));
        }
        JiraSubcommand::Attachments { jira_id: id } => {
            assert_eq!(id.jira_id, Some(jira_id.to_string()));
        }
        _ => panic!("Unexpected command variant"),
    }
    Ok(())
}

#[rstest]
#[case("info")]
#[case("related")]
#[case("changelog")]
#[case("comment")]
#[case("comments")]
#[case("attachments")]
#[case("clean")]
fn test_jira_command_without_id(#[case] subcommand: &str) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", subcommand])?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert_eq!(args.jira_id.jira_id, None);
        }
        JiraSubcommand::Related { args, .. } => {
            assert_eq!(args.jira_id.jira_id, None);
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert_eq!(args.jira_id.jira_id, None);
        }
        JiraSubcommand::Comment { jira_id } => {
            assert_eq!(jira_id.jira_id, None);
        }
        JiraSubcommand::Comments { jira_id, .. } => {
            assert_eq!(jira_id.jira_id, None);
        }
        JiraSubcommand::Attachments { jira_id } => {
            assert_eq!(jira_id.jira_id, None);
        }
        JiraSubcommand::Clean { jira_id, .. } => {
            assert_eq!(jira_id.jira_id, None);
        }
        _ => panic!("Unexpected command variant"),
    }
Ok(())
}

// ==================== 输出格式测试 ====================

#[rstest]
#[case("info", "PROJ-123")]
#[case("related", "PROJ-123")]
#[case("changelog", "PROJ-123")]
#[case("comments", "PROJ-123")]
fn test_jira_command_output_format_table(#[case] subcommand: &str, #[case] jira_id: &str) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", subcommand, jira_id, "--table"])?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert!(args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Related { args, .. } => {
            assert!(args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert!(args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Comments { output_format, .. } => {
            assert!(output_format.table);
            assert!(!output_format.json);
            assert!(!output_format.yaml);
            assert!(!output_format.markdown);
        }
        _ => panic!("Unexpected command variant"),
    }
Ok(())
}

#[rstest]
#[case("info", "PROJ-123")]
#[case("related", "PROJ-123")]
#[case("changelog", "PROJ-123")]
#[case("comments", "PROJ-123")]
fn test_jira_command_output_format_json(#[case] subcommand: &str, #[case] jira_id: &str) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", subcommand, jira_id, "--json"])?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Related { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Comments { output_format, .. } => {
            assert!(!output_format.table);
            assert!(output_format.json);
            assert!(!output_format.yaml);
            assert!(!output_format.markdown);
        }
        _ => panic!("Unexpected command variant"),
    }
Ok(())
}

#[rstest]
#[case("info", "PROJ-123")]
#[case("related", "PROJ-123")]
#[case("changelog", "PROJ-123")]
#[case("comments", "PROJ-123")]
fn test_jira_command_output_format_yaml(#[case] subcommand: &str, #[case] jira_id: &str) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", subcommand, jira_id, "--yaml"])?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Related { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(args.query_display.output_format.yaml);
            assert!(!args.query_display.output_format.markdown);
        }
        JiraSubcommand::Comments { output_format, .. } => {
            assert!(!output_format.table);
            assert!(!output_format.json);
            assert!(output_format.yaml);
            assert!(!output_format.markdown);
        }
        _ => panic!("Unexpected command variant"),
    }
Ok(())
}

#[rstest]
#[case("info", "PROJ-123")]
#[case("related", "PROJ-123")]
#[case("changelog", "PROJ-123")]
#[case("comments", "PROJ-123")]
fn test_jira_command_output_format_markdown(#[case] subcommand: &str, #[case] jira_id: &str) -> Result<()> {
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", subcommand, jira_id, "--markdown"])?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(args.query_display.output_format.markdown);
        }
        JiraSubcommand::Related { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(args.query_display.output_format.markdown);
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert!(!args.query_display.output_format.table);
            assert!(!args.query_display.output_format.json);
            assert!(!args.query_display.output_format.yaml);
            assert!(args.query_display.output_format.markdown);
        }
        JiraSubcommand::Comments { output_format, .. } => {
            assert!(!output_format.table);
            assert!(!output_format.json);
            assert!(!output_format.yaml);
            assert!(output_format.markdown);
        }
        _ => panic!("Unexpected command variant"),
    }
Ok(())
}

#[rstest]
#[case("info", "PROJ-123")]
#[case("related", "PROJ-123")]
#[case("changelog", "PROJ-123")]
fn test_jira_command_output_format_all_flags(#[case] subcommand: &str, #[case] jira_id: &str) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        subcommand,
        jira_id,
        "--table",
        "--json",
        "--yaml",
        "--markdown",
    ])
    ?;

    match &cli.command {
        JiraSubcommand::Info { args, .. } => {
            assert!(args.query_display.output_format.table);
            assert!(args.query_display.output_format.json);
            assert!(args.query_display.output_format.yaml);
            assert!(args.query_display.output_format.markdown);
        }
        JiraSubcommand::Related { args, .. } => {
            assert!(args.query_display.output_format.table);
            assert!(args.query_display.output_format.json);
            assert!(args.query_display.output_format.yaml);
            assert!(args.query_display.output_format.markdown);
        }
        JiraSubcommand::Changelog { args, .. } => {
            assert!(args.query_display.output_format.table);
            assert!(args.query_display.output_format.json);
            assert!(args.query_display.output_format.yaml);
            assert!(args.query_display.output_format.markdown);
        }
        _ => panic!("Unexpected command variant"),
    }
Ok(())
}

// ==================== Clean 命令测试 ====================

#[test]
fn test_jira_clean_command_structure() -> Result<()> {
    // 测试 Clean 命令结构（带所有参数）
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "clean",
        "PROJ-789",
        "--all",
        "--dry-run",
        "--list",
    ])
    ?;

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-789".to_string()));
            assert!(all);
            assert!(dry_run.dry_run);
            assert!(list);
        }
        _ => panic!("Expected Clean command"),
    }
Ok(())
}

#[rstest]
#[case("PROJ-123", false, false, false, false)]
#[case("PROJ-123", true, false, false, false)]
#[case("PROJ-123", false, true, false, false)]
#[case("PROJ-123", false, false, true, false)]
#[case("PROJ-123", true, true, true, false)]
#[case("", false, false, false, true)]
#[case("", true, false, false, true)]
fn test_jira_clean_command_flags(
    #[case] jira_id: &str,
    #[case] all: bool,
    #[case] dry_run: bool,
    #[case] list: bool,
    #[case] no_jira_id: bool,
) -> Result<()> {
    let mut args = vec!["test-jira", "clean"];
    if !no_jira_id && !jira_id.is_empty() {
        args.push(jira_id);
    }
    if all {
        args.push("--all");
    }
    if dry_run {
        args.push("--dry-run");
    }
    if list {
        args.push("--list");
    }

    let cli = TestJiraCli::try_parse_from(&args)?;

    match cli.command {
        JiraSubcommand::Clean {
            jira_id: id,
            all: a,
            dry_run: dr,
            list: l,
        } => {
            if no_jira_id {
                assert_eq!(id.jira_id, None);
            } else {
                assert_eq!(id.jira_id, Some(jira_id.to_string()));
            }
            assert_eq!(a, all);
            assert_eq!(dr.dry_run, dry_run);
            assert_eq!(l, list);
        }
        _ => panic!("Expected Clean command"),
    }
    Ok(())
}

#[test]
fn test_jira_clean_command_short_flags() -> Result<()> {
    // 测试 Clean 命令的短标志
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean", "-a", "-n", "-l"])?;

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert!(all);
            assert!(dry_run.dry_run);
            assert!(list);
        }
        _ => panic!("Expected Clean command"),
    }
Ok(())
}

// ==================== Comments 命令参数测试 ====================

#[test]
fn test_jira_comments_command_with_limit() -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--limit", "10"])
        ?;
    match cli.command {
        JiraSubcommand::Comments { pagination, .. } => assert_eq!(pagination.limit, Some(10)),
        _ => panic!("Expected Comments command"),
    }
Ok(())
}

#[test]
fn test_jira_comments_command_with_offset() -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--offset", "5"])
        ?;
    match cli.command {
        JiraSubcommand::Comments { pagination, .. } => assert_eq!(pagination.offset, Some(5)),
        _ => panic!("Expected Comments command"),
    }
Ok(())
}

#[test]
fn test_jira_comments_command_with_author() -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "comments",
        "PROJ-123",
        "--author",
        "user@example.com",
    ])
    ?;
    match cli.command {
        JiraSubcommand::Comments { author, .. } => {
            assert_eq!(author, Some("user@example.com".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
Ok(())
}

#[test]
fn test_jira_comments_command_with_since() -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "comments",
        "PROJ-123",
        "--since",
        "2024-01-01T00:00:00Z",
    ])
    ?;
    match cli.command {
        JiraSubcommand::Comments { since, .. } => {
            assert_eq!(since, Some("2024-01-01T00:00:00Z".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
Ok(())
}

#[test]
fn test_jira_comments_command_all_filters() -> Result<()> {
    // 测试所有过滤参数组合
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "comments",
        "PROJ-123",
        "--limit",
        "20",
        "--offset",
        "10",
        "--author",
        "user@example.com",
        "--since",
        "2024-01-01T00:00:00Z",
    ])
    ?;

    match cli.command {
        JiraSubcommand::Comments {
            jira_id,
            pagination,
            author,
            since,
            ..
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-123".to_string()));
            assert_eq!(pagination.limit, Some(20));
            assert_eq!(pagination.offset, Some(10));
            assert_eq!(author, Some("user@example.com".to_string()));
            assert_eq!(since, Some("2024-01-01T00:00:00Z".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
Ok(())
}

#[test]
fn test_jira_comments_command_pagination() -> Result<()> {
    // 测试分页参数组合（limit + offset）
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "comments",
        "PROJ-123",
        "--limit",
        "15",
        "--offset",
        "30",
    ])
    ?;

    match cli.command {
        JiraSubcommand::Comments {
            jira_id,
            pagination,
            ..
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-123".to_string()));
            assert_eq!(pagination.limit, Some(15));
            assert_eq!(pagination.offset, Some(30));
        }
        _ => panic!("Expected Comments command"),
    }
Ok(())
}

// ==================== 命令枚举测试 ====================

#[rstest]
#[case("info", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Info { .. }))]
#[case("related", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Related { .. }))]
#[case("changelog", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Changelog { .. }))]
#[case("comment", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Comment { .. }))]
#[case("comments", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Comments { .. }))]
#[case("attachments", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Attachments { .. }))]
#[case("clean", |cmd: &JiraSubcommand| matches!(cmd, JiraSubcommand::Clean { .. }))]
fn test_jira_command_parsing_all_subcommands(
    #[case] subcommand: &str,
    #[case] assert_fn: fn(&JiraSubcommand) -> bool,
) -> Result<()> {
    let cli = TestJiraCli::try_parse_from(&["test-jira", subcommand])?;
    assert!(
        assert_fn(&cli.command),
        "Command should match expected variant"
    );
    Ok(())
}

#[test]
fn test_jira_command_error_handling_invalid_subcommand() -> Result<()> {
    // 测试无效子命令的错误处理
    let result = TestJiraCli::try_parse_from(&["test-jira", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
Ok(())
}

// ==================== Changelog 命令测试 ====================

#[test]
fn test_jira_changelog_command_with_field_filter() -> Result<()> {
    // 测试 --field 参数
    // 注意：当前 Changelog 命令的枚举定义中没有 field 字段
    // 如果将来添加了 field 字段，这个测试需要更新
    let cli = TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123"])?;

    match cli.command {
        JiraSubcommand::Changelog { args, .. } => {
            assert_eq!(args.jira_id.jira_id, Some("PROJ-123".to_string()));
            // field 字段当前不存在于枚举定义中
        }
        _ => panic!("Expected Changelog command"),
    }
Ok(())
}
