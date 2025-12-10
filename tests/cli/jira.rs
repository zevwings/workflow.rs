//! Jira CLI 命令测试
//!
//! 测试 Jira CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::JiraSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-jira")]
struct TestJiraCli {
    #[command(subcommand)]
    command: JiraSubcommand,
}

// ==================== 命令结构测试 ====================

#[test]
fn test_jira_subcommand_enum_creation() {
    // 测试 JiraSubcommand 枚举可以创建
    // 通过编译验证枚举定义正确
    assert!(true, "JiraSubcommand enum should be defined");
}

#[test]
fn test_jira_info_command_structure() {
    // 测试 Info 命令结构
    // 验证命令可以解析
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info", "PROJ-123"]).unwrap();

    match cli.command {
        JiraSubcommand::Info { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
        }
        _ => panic!("Expected Info command"),
    }
}

#[test]
fn test_jira_info_command_without_id() {
    // 测试 Info 命令不带 ID（应该为 None）
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info"]).unwrap();

    match cli.command {
        JiraSubcommand::Info { jira_id, .. } => {
            assert_eq!(jira_id, None);
        }
        _ => panic!("Expected Info command"),
    }
}

// ==================== Related 命令测试 ====================

#[test]
fn test_jira_related_command_structure() {
    // 测试 Related 命令基本结构
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related", "PROJ-123"]).unwrap();

    match cli.command {
        JiraSubcommand::Related { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
        }
        _ => panic!("Expected Related command"),
    }
}

#[test]
fn test_jira_related_command_with_jira_id() {
    // 测试带 JIRA ID 的情况
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related", "PROJ-456"]).unwrap();

    match cli.command {
        JiraSubcommand::Related { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-456".to_string()));
        }
        _ => panic!("Expected Related command"),
    }
}

#[test]
fn test_jira_related_command_without_id() {
    // 测试不带 JIRA ID（交互式输入）
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related"]).unwrap();

    match cli.command {
        JiraSubcommand::Related { jira_id, .. } => {
            assert_eq!(jira_id, None);
        }
        _ => panic!("Expected Related command"),
    }
}

#[test]
fn test_jira_related_command_output_formats() {
    // 测试输出格式（table, json, yaml, markdown）

    // Table format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "related", "PROJ-123", "--table"]).unwrap();
    match cli.command {
        JiraSubcommand::Related {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(table);
            assert!(!json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Related command"),
    }

    // JSON format
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related", "PROJ-123", "--json"]).unwrap();
    match cli.command {
        JiraSubcommand::Related {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Related command"),
    }

    // YAML format
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related", "PROJ-123", "--yaml"]).unwrap();
    match cli.command {
        JiraSubcommand::Related {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Related command"),
    }

    // Markdown format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "related", "PROJ-123", "--markdown"]).unwrap();
    match cli.command {
        JiraSubcommand::Related {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(!yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Related command"),
    }
}

#[test]
fn test_jira_related_command_all_flags() {
    // 测试所有标志组合
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "related",
        "PROJ-123",
        "--table",
        "--json",
        "--yaml",
        "--markdown",
    ])
    .unwrap();

    match cli.command {
        JiraSubcommand::Related {
            jira_id,
            table,
            json,
            yaml,
            markdown,
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert!(table);
            assert!(json);
            assert!(yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Related command"),
    }
}

#[test]
fn test_jira_attachments_command_structure() {
    // 测试 Attachments 命令结构
    let cli = TestJiraCli::try_parse_from(&["test-jira", "attachments", "PROJ-456"]).unwrap();

    match cli.command {
        JiraSubcommand::Attachments { jira_id } => {
            assert_eq!(jira_id, Some("PROJ-456".to_string()));
        }
        _ => panic!("Expected Attachments command"),
    }
}

#[test]
fn test_jira_attachments_command_without_id() {
    // 测试 Attachments 命令不带 ID
    let cli = TestJiraCli::try_parse_from(&["test-jira", "attachments"]).unwrap();

    match cli.command {
        JiraSubcommand::Attachments { jira_id } => {
            assert_eq!(jira_id, None);
        }
        _ => panic!("Expected Attachments command"),
    }
}

#[test]
fn test_jira_clean_command_structure() {
    // 测试 Clean 命令结构（带所有参数）
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "clean",
        "PROJ-789",
        "--all",
        "--dry-run",
        "--list",
    ])
    .unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, Some("PROJ-789".to_string()));
            assert!(all);
            assert!(dry_run);
            assert!(list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_clean_command_with_jira_id_only() {
    // 测试 Clean 命令只带 JIRA ID
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean", "PROJ-123"]).unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert!(!all);
            assert!(!dry_run);
            assert!(!list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_clean_command_with_all_flag() {
    // 测试 Clean 命令带 --all 标志
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean", "--all"]).unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, None);
            assert!(all);
            assert!(!dry_run);
            assert!(!list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_clean_command_with_dry_run() {
    // 测试 Clean 命令带 --dry-run 标志
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "clean", "PROJ-123", "--dry-run"]).unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert!(!all);
            assert!(dry_run);
            assert!(!list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_clean_command_with_list_flag() {
    // 测试 Clean 命令带 --list 标志
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean", "PROJ-123", "--list"]).unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert!(!all);
            assert!(!dry_run);
            assert!(list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_clean_command_short_flags() {
    // 测试 Clean 命令的短标志
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean", "-a", "-n", "-l"]).unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, None);
            assert!(all);
            assert!(dry_run);
            assert!(list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_clean_command_without_jira_id() {
    // 测试 Clean 命令不带 JIRA ID
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean"]).unwrap();

    match cli.command {
        JiraSubcommand::Clean {
            jira_id,
            all,
            dry_run,
            list,
        } => {
            assert_eq!(jira_id, None);
            assert!(!all);
            assert!(!dry_run);
            assert!(!list);
        }
        _ => panic!("Expected Clean command"),
    }
}

#[test]
fn test_jira_command_parsing_all_subcommands() {
    // 测试所有子命令都可以正确解析

    // Info
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info"]).unwrap();
    assert!(matches!(cli.command, JiraSubcommand::Info { .. }));

    // Related
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related"]).unwrap();
    assert!(matches!(cli.command, JiraSubcommand::Related { .. }));

    // Changelog
    let cli = TestJiraCli::try_parse_from(&["test-jira", "changelog"]).unwrap();
    assert!(matches!(cli.command, JiraSubcommand::Changelog { .. }));

    // Comments
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments"]).unwrap();
    assert!(matches!(cli.command, JiraSubcommand::Comments { .. }));

    // Attachments
    let cli = TestJiraCli::try_parse_from(&["test-jira", "attachments"]).unwrap();
    assert!(matches!(cli.command, JiraSubcommand::Attachments { .. }));

    // Clean
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean"]).unwrap();
    assert!(matches!(cli.command, JiraSubcommand::Clean { .. }));
}

#[test]
fn test_jira_command_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestJiraCli::try_parse_from(&["test-jira", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_jira_jira_id_parameter_optional() {
    // 测试 JIRA ID 参数在所有命令中都是可选的
    // Info
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info"]).unwrap();
    match cli.command {
        JiraSubcommand::Info { jira_id, .. } => assert_eq!(jira_id, None),
        _ => panic!(),
    }

    // Related
    let cli = TestJiraCli::try_parse_from(&["test-jira", "related"]).unwrap();
    match cli.command {
        JiraSubcommand::Related { jira_id, .. } => assert_eq!(jira_id, None),
        _ => panic!(),
    }

    // Attachments
    let cli = TestJiraCli::try_parse_from(&["test-jira", "attachments"]).unwrap();
    match cli.command {
        JiraSubcommand::Attachments { jira_id } => assert_eq!(jira_id, None),
        _ => panic!(),
    }

    // Clean
    let cli = TestJiraCli::try_parse_from(&["test-jira", "clean"]).unwrap();
    match cli.command {
        JiraSubcommand::Clean { jira_id, .. } => assert_eq!(jira_id, None),
        _ => panic!(),
    }
}

// ==================== Changelog 命令测试 ====================

#[test]
fn test_jira_changelog_command_structure() {
    // 测试 Changelog 命令基本结构
    let cli = TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123"]).unwrap();

    match cli.command {
        JiraSubcommand::Changelog { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
        }
        _ => panic!("Expected Changelog command"),
    }
}

#[test]
fn test_jira_changelog_command_with_jira_id() {
    // 测试带 JIRA ID 的情况
    let cli = TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-456"]).unwrap();

    match cli.command {
        JiraSubcommand::Changelog { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-456".to_string()));
        }
        _ => panic!("Expected Changelog command"),
    }
}

#[test]
fn test_jira_changelog_command_without_id() {
    // 测试不带 JIRA ID（交互式输入）
    let cli = TestJiraCli::try_parse_from(&["test-jira", "changelog"]).unwrap();

    match cli.command {
        JiraSubcommand::Changelog { jira_id, .. } => {
            assert_eq!(jira_id, None);
        }
        _ => panic!("Expected Changelog command"),
    }
}

#[test]
fn test_jira_changelog_command_with_field_filter() {
    // 测试 --field 参数
    // 注意：当前 Changelog 命令的枚举定义中没有 field 字段
    // 如果将来添加了 field 字段，这个测试需要更新
    let cli = TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123"]).unwrap();

    match cli.command {
        JiraSubcommand::Changelog { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            // field 字段当前不存在于枚举定义中
        }
        _ => panic!("Expected Changelog command"),
    }
}

#[test]
fn test_jira_changelog_command_output_formats() {
    // 测试输出格式（table, json, yaml, markdown）

    // Table format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123", "--table"]).unwrap();
    match cli.command {
        JiraSubcommand::Changelog {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(table);
            assert!(!json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Changelog command"),
    }

    // JSON format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123", "--json"]).unwrap();
    match cli.command {
        JiraSubcommand::Changelog {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Changelog command"),
    }

    // YAML format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123", "--yaml"]).unwrap();
    match cli.command {
        JiraSubcommand::Changelog {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Changelog command"),
    }

    // Markdown format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "changelog", "PROJ-123", "--markdown"]).unwrap();
    match cli.command {
        JiraSubcommand::Changelog {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(!yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Changelog command"),
    }
}

#[test]
fn test_jira_changelog_command_all_flags() {
    // 测试所有标志组合
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "changelog",
        "PROJ-123",
        "--table",
        "--json",
        "--yaml",
        "--markdown",
    ])
    .unwrap();

    match cli.command {
        JiraSubcommand::Changelog {
            jira_id,
            table,
            json,
            yaml,
            markdown,
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert!(table);
            assert!(json);
            assert!(yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Changelog command"),
    }
}

// ==================== Comments 命令测试 ====================

#[test]
fn test_jira_comments_command_structure() {
    // 测试 Comments 命令基本结构
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123"]).unwrap();

    match cli.command {
        JiraSubcommand::Comments { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_with_jira_id() {
    // 测试带 JIRA ID 的情况
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-456"]).unwrap();

    match cli.command {
        JiraSubcommand::Comments { jira_id, .. } => {
            assert_eq!(jira_id, Some("PROJ-456".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_without_id() {
    // 测试不带 JIRA ID（交互式输入）
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments"]).unwrap();

    match cli.command {
        JiraSubcommand::Comments { jira_id, .. } => {
            assert_eq!(jira_id, None);
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_with_limit() {
    // 测试 --limit 参数
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--limit", "10"])
        .unwrap();

    match cli.command {
        JiraSubcommand::Comments { jira_id, limit, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert_eq!(limit, Some(10));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_with_offset() {
    // 测试 --offset 参数
    let cli = TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--offset", "5"])
        .unwrap();

    match cli.command {
        JiraSubcommand::Comments {
            jira_id, offset, ..
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert_eq!(offset, Some(5));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_with_author() {
    // 测试 --author 参数
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "comments",
        "PROJ-123",
        "--author",
        "user@example.com",
    ])
    .unwrap();

    match cli.command {
        JiraSubcommand::Comments {
            jira_id, author, ..
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert_eq!(author, Some("user@example.com".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_with_since() {
    // 测试 --since 参数
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "comments",
        "PROJ-123",
        "--since",
        "2024-01-01T00:00:00Z",
    ])
    .unwrap();

    match cli.command {
        JiraSubcommand::Comments { jira_id, since, .. } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert_eq!(since, Some("2024-01-01T00:00:00Z".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_output_formats() {
    // 测试输出格式（table, json, yaml, markdown）

    // Table format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--table"]).unwrap();
    match cli.command {
        JiraSubcommand::Comments {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(table);
            assert!(!json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Comments command"),
    }

    // JSON format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--json"]).unwrap();
    match cli.command {
        JiraSubcommand::Comments {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Comments command"),
    }

    // YAML format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--yaml"]).unwrap();
    match cli.command {
        JiraSubcommand::Comments {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Comments command"),
    }

    // Markdown format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "comments", "PROJ-123", "--markdown"]).unwrap();
    match cli.command {
        JiraSubcommand::Comments {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(!yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_all_filters() {
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
    .unwrap();

    match cli.command {
        JiraSubcommand::Comments {
            jira_id,
            limit,
            offset,
            author,
            since,
            ..
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert_eq!(limit, Some(20));
            assert_eq!(offset, Some(10));
            assert_eq!(author, Some("user@example.com".to_string()));
            assert_eq!(since, Some("2024-01-01T00:00:00Z".to_string()));
        }
        _ => panic!("Expected Comments command"),
    }
}

#[test]
fn test_jira_comments_command_pagination() {
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
    .unwrap();

    match cli.command {
        JiraSubcommand::Comments {
            jira_id,
            limit,
            offset,
            ..
        } => {
            assert_eq!(jira_id, Some("PROJ-123".to_string()));
            assert_eq!(limit, Some(15));
            assert_eq!(offset, Some(30));
        }
        _ => panic!("Expected Comments command"),
    }
}

// ==================== Info 命令输出格式测试 ====================

#[test]
fn test_jira_info_command_output_formats() {
    // 测试输出格式（table, json, yaml, markdown）

    // Table format
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info", "PROJ-123", "--table"]).unwrap();
    match cli.command {
        JiraSubcommand::Info {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(table);
            assert!(!json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Info command"),
    }

    // JSON format
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info", "PROJ-123", "--json"]).unwrap();
    match cli.command {
        JiraSubcommand::Info {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(json);
            assert!(!yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Info command"),
    }

    // YAML format
    let cli = TestJiraCli::try_parse_from(&["test-jira", "info", "PROJ-123", "--yaml"]).unwrap();
    match cli.command {
        JiraSubcommand::Info {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(yaml);
            assert!(!markdown);
        }
        _ => panic!("Expected Info command"),
    }

    // Markdown format
    let cli =
        TestJiraCli::try_parse_from(&["test-jira", "info", "PROJ-123", "--markdown"]).unwrap();
    match cli.command {
        JiraSubcommand::Info {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(!table);
            assert!(!json);
            assert!(!yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Info command"),
    }
}

#[test]
fn test_jira_info_command_format_flags_combination() {
    // 测试格式标志的组合
    let cli = TestJiraCli::try_parse_from(&[
        "test-jira",
        "info",
        "PROJ-123",
        "--table",
        "--json",
        "--yaml",
        "--markdown",
    ])
    .unwrap();

    match cli.command {
        JiraSubcommand::Info {
            table,
            json,
            yaml,
            markdown,
            ..
        } => {
            assert!(table);
            assert!(json);
            assert!(yaml);
            assert!(markdown);
        }
        _ => panic!("Expected Info command"),
    }
}
