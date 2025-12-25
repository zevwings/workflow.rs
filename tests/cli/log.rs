//! Log CLI 命令测试
//!
//! 测试 Log CLI 命令的参数解析、命令执行流程和错误处理。
//!
//! 注意：Log 命令现在是 Jira 命令的子命令，路径为 `workflow jira log`.

use clap::Parser;
use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::cli::{JiraSubcommand, LogSubcommand};

// 创建一个测试用的 CLI 结构来测试参数解析（通过 Jira 命令）
#[derive(Parser)]
#[command(name = "test-jira")]
struct TestJiraCli {
    #[command(subcommand)]
    command: JiraSubcommand,
}

// 创建一个测试用的 CLI 结构来测试 LogSubcommand 的参数解析
#[derive(Parser)]
#[command(name = "test-log")]
struct TestLogCli {
    #[command(subcommand)]
    command: LogSubcommand,
}

// ==================== 命令结构测试 ====================

#[test]
fn test_log_download_command_structure() -> Result<()> {
    // 测试 Download 命令结构
    // 验证命令可以解析
    let cli = TestLogCli::try_parse_from(&["test-log", "download", "PROJ-123"])?;

    match cli.command {
        LogSubcommand::Download { jira_id } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-123".to_string()));
        }
        _ => panic!("Expected Download command"),
    }
Ok(())
}

#[test]
fn test_log_download_command_without_id() -> Result<()> {
    // 测试 Download 命令不带 ID（应该为 None）
    let cli = TestLogCli::try_parse_from(&["test-log", "download"])?;

    match cli.command {
        LogSubcommand::Download { jira_id } => {
            assert_eq!(jira_id.jira_id, None);
        }
        _ => panic!("Expected Download command"),
    }
Ok(())
}

#[test]
fn test_log_find_command_structure() -> Result<()> {
    // 测试 Find 命令结构（带 JIRA ID 和 Request ID）
    let cli = TestLogCli::try_parse_from(&["test-log", "find", "PROJ-456", "req-12345"])?;

    match cli.command {
        LogSubcommand::Find {
            jira_id,
            request_id,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-456".to_string()));
            assert_eq!(request_id, Some("req-12345".to_string()));
        }
        _ => panic!("Expected Find command"),
    }
Ok(())
}

#[test]
fn test_log_find_command_with_jira_id_only() -> Result<()> {
    // 测试 Find 命令只带 JIRA ID
    let cli = TestLogCli::try_parse_from(&["test-log", "find", "PROJ-456"])?;

    match cli.command {
        LogSubcommand::Find {
            jira_id,
            request_id,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-456".to_string()));
            assert_eq!(request_id, None);
        }
        _ => panic!("Expected Find command"),
    }
Ok(())
}

#[test]
fn test_log_find_command_without_parameters() -> Result<()> {
    // 测试 Find 命令不带任何参数
    let cli = TestLogCli::try_parse_from(&["test-log", "find"])?;

    match cli.command {
        LogSubcommand::Find {
            jira_id,
            request_id,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert_eq!(request_id, None);
        }
        _ => panic!("Expected Find command"),
    }
Ok(())
}

#[test]
fn test_log_search_command_structure() -> Result<()> {
    // 测试 Search 命令结构（带 JIRA ID 和搜索关键词）
    let cli = TestLogCli::try_parse_from(&["test-log", "search", "PROJ-789", "error"])?;

    match cli.command {
        LogSubcommand::Search {
            jira_id,
            search_term,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-789".to_string()));
            assert_eq!(search_term, Some("error".to_string()));
        }
        _ => panic!("Expected Search command"),
    }
Ok(())
}

#[test]
fn test_log_search_command_with_jira_id_only() -> Result<()> {
    // 测试 Search 命令只带 JIRA ID
    let cli = TestLogCli::try_parse_from(&["test-log", "search", "PROJ-789"])?;

    match cli.command {
        LogSubcommand::Search {
            jira_id,
            search_term,
        } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-789".to_string()));
            assert_eq!(search_term, None);
        }
        _ => panic!("Expected Search command"),
    }
Ok(())
}

#[test]
fn test_log_search_command_without_parameters() -> Result<()> {
    // 测试 Search 命令不带任何参数
    let cli = TestLogCli::try_parse_from(&["test-log", "search"])?;

    match cli.command {
        LogSubcommand::Search {
            jira_id,
            search_term,
        } => {
            assert_eq!(jira_id.jira_id, None);
            assert_eq!(search_term, None);
        }
        _ => panic!("Expected Search command"),
    }
Ok(())
}

#[test]
fn test_log_command_parsing_all_subcommands() -> Result<()> {
    // 测试所有子命令都可以正确解析

    // Download
    let cli = TestLogCli::try_parse_from(&["test-log", "download"])?;
    assert!(matches!(cli.command, LogSubcommand::Download { .. }));

    // Find
    let cli = TestLogCli::try_parse_from(&["test-log", "find"])?;
    assert!(matches!(cli.command, LogSubcommand::Find { .. }));

    // Search
    let cli = TestLogCli::try_parse_from(&["test-log", "search"])?;
    assert!(matches!(cli.command, LogSubcommand::Search { .. }));
Ok(())
}

#[test]
fn test_log_command_error_handling_invalid_subcommand() -> Result<()> {
    // 测试无效子命令的错误处理
    let result = TestLogCli::try_parse_from(&["test-log", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
Ok(())
}

#[test]
fn test_log_jira_id_parameter_optional() -> Result<()> {
    // 测试 JIRA ID 参数在所有命令中都是可选的
    // Download
    let cli = TestLogCli::try_parse_from(&["test-log", "download"])?;
    match cli.command {
        LogSubcommand::Download { jira_id } => assert_eq!(jira_id.jira_id, None),
        _ => panic!(),
    }

    // Find
    let cli = TestLogCli::try_parse_from(&["test-log", "find"])?;
    match cli.command {
        LogSubcommand::Find { jira_id, .. } => assert_eq!(jira_id.jira_id, None),
        _ => panic!(),
    }

    // Search
    let cli = TestLogCli::try_parse_from(&["test-log", "search"])?;
    match cli.command {
        LogSubcommand::Search { jira_id, .. } => assert_eq!(jira_id.jira_id, None),
        _ => panic!(),
    }
Ok(())
}

#[test]
fn test_log_find_with_request_id_only() -> Result<()> {
    // 测试 Find 命令只带 Request ID（不带 JIRA ID）
    // 注意：clap 按位置解析，所以第一个参数会被解析为 jira_id
    let cli = TestLogCli::try_parse_from(&["test-log", "find", "req-12345"])?;

    match cli.command {
        LogSubcommand::Find {
            jira_id,
            request_id,
        } => {
            // 第一个参数会被解析为 jira_id
            assert_eq!(jira_id.jira_id, Some("req-12345".to_string()));
            assert_eq!(request_id, None);
        }
        _ => panic!("Expected Find command"),
    }
Ok(())
}

#[test]
fn test_log_search_with_search_term_only() -> Result<()> {
    // 测试 Search 命令只带搜索关键词（不带 JIRA ID）
    // 注意：clap 按位置解析，所以第一个参数会被解析为 jira_id
    let cli = TestLogCli::try_parse_from(&["test-log", "search", "error"])?;

    match cli.command {
        LogSubcommand::Search {
            jira_id,
            search_term,
        } => {
            // 第一个参数会被解析为 jira_id
            assert_eq!(jira_id.jira_id, Some("error".to_string()));
            assert_eq!(search_term, None);
        }
        _ => panic!("Expected Search command"),
    }
Ok(())
}
