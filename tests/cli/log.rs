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

// ==================== Download Command Tests ====================

/// 测试 Log Download 命令解析（带 JIRA ID）
///
/// ## 测试目的
/// 验证 LogSubcommand::Download 能够正确解析带 JIRA ID 的命令行参数。
///
/// ## 测试场景
/// 1. 准备带 JIRA ID 的 Download 命令输入
/// 2. 解析命令行参数
/// 3. 验证 Download 命令结构解析正确
///
/// ## 预期结果
/// - Download 命令正确解析，JIRA ID 被正确设置
#[test]
fn test_log_download_command_with_jira_id_parses_correctly() -> Result<()> {
    // Arrange: 准备带 JIRA ID 的 Download 命令输入
    let args = &["test-log", "download", "PROJ-123"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证 Download 命令结构解析正确
    match cli.command {
        LogSubcommand::Download { jira_id } => {
            assert_eq!(jira_id.jira_id, Some("PROJ-123".to_string()));
        }
        _ => panic!("Expected Download command"),
    }
    Ok(())
}

/// 测试 Log Download 命令解析（不带 ID）
///
/// ## 测试目的
/// 验证 LogSubcommand::Download 能够正确解析不带 JIRA ID 的命令行参数。
///
/// ## 测试场景
/// 1. 准备不带 ID 的 Download 命令输入
/// 2. 解析命令行参数
/// 3. 验证 JIRA ID 为 None
///
/// ## 预期结果
/// - Download 命令正确解析，JIRA ID 为 None
#[test]
fn test_log_download_command_without_id_parses_correctly() -> Result<()> {
    // Arrange: 准备不带 ID 的 Download 命令输入
    let args = &["test-log", "download"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证 JIRA ID 为 None
    match cli.command {
        LogSubcommand::Download { jira_id } => {
            assert_eq!(jira_id.jira_id, None);
        }
        _ => panic!("Expected Download command"),
    }
    Ok(())
}

// ==================== Find Command Tests ====================

/// 测试 Log Find 命令解析（带 JIRA ID 和 Request ID）
///
/// ## 测试目的
/// 验证 LogSubcommand::Find 能够正确解析带 JIRA ID 和 Request ID 的命令行参数。
///
/// ## 测试场景
/// 1. 准备带 JIRA ID 和 Request ID 的 Find 命令输入
/// 2. 解析命令行参数
/// 3. 验证 Find 命令结构解析正确
///
/// ## 预期结果
/// - Find 命令正确解析，JIRA ID 和 Request ID 都被正确设置
#[test]
fn test_log_find_command_with_jira_id_and_request_id_parses_correctly() -> Result<()> {
    // Arrange: 准备带 JIRA ID 和 Request ID 的 Find 命令输入
    let args = &["test-log", "find", "PROJ-456", "req-12345"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证 Find 命令结构解析正确
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

/// 测试 Log Find 命令解析（只带 JIRA ID）
///
/// ## 测试目的
/// 验证 LogSubcommand::Find 能够正确解析只带 JIRA ID 的命令行参数。
///
/// ## 测试场景
/// 1. 准备只带 JIRA ID 的 Find 命令输入
/// 2. 解析命令行参数
/// 3. 验证 JIRA ID 解析正确，Request ID 为 None
///
/// ## 预期结果
/// - Find 命令正确解析，JIRA ID 被设置，Request ID 为 None
#[test]
fn test_log_find_command_with_jira_id_only_parses_correctly() -> Result<()> {
    // Arrange: 准备只带 JIRA ID 的 Find 命令输入
    let args = &["test-log", "find", "PROJ-456"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证 JIRA ID 解析正确，Request ID 为 None
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

/// 测试 Log Find 命令解析（不带参数）
///
/// ## 测试目的
/// 验证 LogSubcommand::Find 能够正确解析不带任何参数的命令行参数。
///
/// ## 测试场景
/// 1. 准备不带任何参数的 Find 命令输入
/// 2. 解析命令行参数
/// 3. 验证所有参数为 None
///
/// ## 预期结果
/// - Find 命令正确解析，所有参数都为 None
#[test]
fn test_log_find_command_without_parameters_parses_correctly() -> Result<()> {
    // Arrange: 准备不带任何参数的 Find 命令输入
    let args = &["test-log", "find"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证所有参数为 None
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

// ==================== Search Command Tests ====================

/// 测试 Log Search 命令解析（带 JIRA ID 和搜索关键词）
///
/// ## 测试目的
/// 验证 LogSubcommand::Search 能够正确解析带 JIRA ID 和搜索关键词的命令行参数。
///
/// ## 测试场景
/// 1. 准备带 JIRA ID 和搜索关键词的 Search 命令输入
/// 2. 解析命令行参数
/// 3. 验证 Search 命令结构解析正确
///
/// ## 预期结果
/// - Search 命令正确解析，JIRA ID 和搜索关键词都被正确设置
#[test]
fn test_log_search_command_with_jira_id_and_search_term_parses_correctly() -> Result<()> {
    // Arrange: 准备带 JIRA ID 和搜索关键词的 Search 命令输入
    let args = &["test-log", "search", "PROJ-789", "error"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证 Search 命令结构解析正确
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

/// 测试 Log Search 命令解析（只带 JIRA ID）
///
/// ## 测试目的
/// 验证 LogSubcommand::Search 能够正确解析只带 JIRA ID 的命令行参数。
///
/// ## 测试场景
/// 1. 准备只带 JIRA ID 的 Search 命令输入
/// 2. 解析命令行参数
/// 3. 验证 JIRA ID 解析正确，搜索关键词为 None
///
/// ## 预期结果
/// - Search 命令正确解析，JIRA ID 被设置，搜索关键词为 None
#[test]
fn test_log_search_command_with_jira_id_only_parses_correctly() -> Result<()> {
    // Arrange: 准备只带 JIRA ID 的 Search 命令输入
    let args = &["test-log", "search", "PROJ-789"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证 JIRA ID 解析正确，搜索关键词为 None
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

/// 测试 Log Search 命令解析（不带参数）
///
/// ## 测试目的
/// 验证 LogSubcommand::Search 能够正确解析不带任何参数的命令行参数。
///
/// ## 测试场景
/// 1. 准备不带任何参数的 Search 命令输入
/// 2. 解析命令行参数
/// 3. 验证所有参数为 None
///
/// ## 预期结果
/// - Search 命令正确解析，所有参数都为 None
#[test]
fn test_log_search_command_without_parameters_parses_correctly() -> Result<()> {
    // Arrange: 准备不带任何参数的 Search 命令输入
    let args = &["test-log", "search"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证所有参数为 None
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

// ==================== Common Command Tests ====================

/// 测试 Log 命令的所有子命令解析
///
/// ## 测试目的
/// 验证 LogSubcommand 的所有子命令（download、find、search）都能够正确解析。
///
/// ## 测试场景
/// 1. 准备所有子命令的输入
/// 2. 解析所有子命令
/// 3. 验证所有子命令都可以正确解析
///
/// ## 预期结果
/// - 所有子命令都可以正确解析
#[test]
fn test_log_command_with_all_subcommands_parses_successfully() -> Result<()> {
    // Arrange: 准备所有子命令的输入
    let download_args = &["test-log", "download"];
    let find_args = &["test-log", "find"];
    let search_args = &["test-log", "search"];

    // Act: 解析所有子命令
    let download_cli = TestLogCli::try_parse_from(download_args)?;
    let find_cli = TestLogCli::try_parse_from(find_args)?;
    let search_cli = TestLogCli::try_parse_from(search_args)?;

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(download_cli.command, LogSubcommand::Download { .. }));
    assert!(matches!(find_cli.command, LogSubcommand::Find { .. }));
    assert!(matches!(search_cli.command, LogSubcommand::Search { .. }));
    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试 Log 命令无效子命令错误处理
///
/// ## 测试目的
/// 验证 LogSubcommand 对无效子命令返回错误。
///
/// ## 测试场景
/// 1. 准备无效子命令的输入
/// 2. 尝试解析无效子命令
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 无效子命令返回解析错误
#[test]
fn test_log_command_with_invalid_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备无效子命令的输入
    let args = &["test-log", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestLogCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
    Ok(())
}

// ==================== Parameter Optionality Tests ====================

/// 测试 Log 命令 JIRA ID 参数可选性
///
/// ## 测试目的
/// 验证 LogSubcommand 的所有命令中 JIRA ID 参数都是可选的。
///
/// ## 测试场景
/// 1. 准备不带 JIRA ID 的所有命令输入
/// 2. 解析所有命令
/// 3. 验证 JIRA ID 参数在所有命令中都是可选的
///
/// ## 预期结果
/// - 所有命令中 JIRA ID 参数都是可选的（为 None）
#[test]
fn test_log_jira_id_parameter_is_optional_in_all_commands() -> Result<()> {
    // Arrange: 准备不带 JIRA ID 的所有命令输入
    let download_args = &["test-log", "download"];
    let find_args = &["test-log", "find"];
    let search_args = &["test-log", "search"];

    // Act: 解析所有命令
    let download_cli = TestLogCli::try_parse_from(download_args)?;
    let find_cli = TestLogCli::try_parse_from(find_args)?;
    let search_cli = TestLogCli::try_parse_from(search_args)?;

    // Assert: 验证 JIRA ID 参数在所有命令中都是可选的
    match download_cli.command {
        LogSubcommand::Download { jira_id } => assert_eq!(jira_id.jira_id, None),
        _ => panic!(),
    }

    match find_cli.command {
        LogSubcommand::Find { jira_id, .. } => assert_eq!(jira_id.jira_id, None),
        _ => panic!(),
    }

    match search_cli.command {
        LogSubcommand::Search { jira_id, .. } => assert_eq!(jira_id.jira_id, None),
        _ => panic!(),
    }
    Ok(())
}

/// 测试 Log Find 命令解析（只带 Request ID）
///
/// ## 测试目的
/// 验证 LogSubcommand::Find 在只提供 Request ID 时的解析行为（clap 按位置解析）。
///
/// ## 测试场景
/// 1. 准备只带 Request ID 的 Find 命令输入
/// 2. 解析命令行参数
/// 3. 验证第一个参数被解析为 jira_id（clap 按位置解析）
///
/// ## 预期结果
/// - 第一个参数被解析为 jira_id，request_id 为 None
#[test]
fn test_log_find_command_with_request_id_only_parses_correctly() -> Result<()> {
    // Arrange: 准备只带 Request ID 的 Find 命令输入
    // 注意：clap 按位置解析，所以第一个参数会被解析为 jira_id
    let args = &["test-log", "find", "req-12345"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证第一个参数被解析为 jira_id
    match cli.command {
        LogSubcommand::Find {
            jira_id,
            request_id,
        } => {
            assert_eq!(jira_id.jira_id, Some("req-12345".to_string()));
            assert_eq!(request_id, None);
        }
        _ => panic!("Expected Find command"),
    }
    Ok(())
}

/// 测试 Log Search 命令解析（只带搜索关键词）
///
/// ## 测试目的
/// 验证 LogSubcommand::Search 在只提供搜索关键词时的解析行为（clap 按位置解析）。
///
/// ## 测试场景
/// 1. 准备只带搜索关键词的 Search 命令输入
/// 2. 解析命令行参数
/// 3. 验证第一个参数被解析为 jira_id（clap 按位置解析）
///
/// ## 预期结果
/// - 第一个参数被解析为 jira_id，search_term 为 None
#[test]
fn test_log_search_command_with_search_term_only_parses_correctly() -> Result<()> {
    // Arrange: 准备只带搜索关键词的 Search 命令输入
    // 注意：clap 按位置解析，所以第一个参数会被解析为 jira_id
    let args = &["test-log", "search", "error"];

    // Act: 解析命令行参数
    let cli = TestLogCli::try_parse_from(args)?;

    // Assert: 验证第一个参数被解析为 jira_id
    match cli.command {
        LogSubcommand::Search {
            jira_id,
            search_term,
        } => {
            assert_eq!(jira_id.jira_id, Some("error".to_string()));
            assert_eq!(search_term, None);
        }
        _ => panic!("Expected Search command"),
    }
    Ok(())
}
