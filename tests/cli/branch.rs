//! Branch CLI 命令测试
//!
//! 测试 Branch CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::cli::BranchSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-branch")]
struct TestBranchCli {
    #[command(subcommand)]
    command: BranchSubcommand,
}

// ==================== Create Command Tests ====================

/// 测试分支创建命令解析所有选项
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令能够正确解析所有选项（JIRA ID、--from-default、--dry-run）。
///
/// ## 测试场景
/// 1. 准备包含所有参数的命令行输入
/// 2. 解析命令行参数
/// 3. 验证所有参数解析正确
///
/// ## 预期结果
/// - 解析成功
/// - JIRA ID、from_default、dry_run 都正确设置
#[test]
fn test_branch_create_command_with_all_options_parses_correctly() -> Result<()> {
    // Arrange: 准备包含所有参数的 Create 命令输入
    let args = &[
        "test-branch",
        "create",
        "PROJ-123",
        "--from-default",
        "--dry-run",
    ];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证所有参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}

/// 测试分支创建命令解析最小参数
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令在使用最小参数（只有命令名）时能够正确解析，所有可选参数使用默认值。
///
/// ## 测试场景
/// 1. 准备只包含命令名的命令行输入
/// 2. 解析命令行参数
/// 3. 验证所有可选参数使用默认值
///
/// ## 预期结果
/// - 解析成功
/// - JIRA ID为None，from_default为false，dry_run为false
#[test]
fn test_branch_create_command_with_minimal_args_parses_correctly() -> Result<()> {
    // Arrange: 准备最小参数的 Create 命令输入
    let args = &["test-branch", "create"];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证最小参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}

/// 测试分支创建命令解析仅JIRA ticket参数
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令能够正确解析只包含JIRA ticket ID的参数。
///
/// ## 测试场景
/// 1. 准备只包含JIRA ticket ID的命令行输入
/// 2. 解析命令行参数
/// 3. 验证JIRA ticket ID解析正确，其他参数使用默认值
///
/// ## 预期结果
/// - 解析成功
/// - JIRA ID正确设置，from_default和dry_run为默认值
#[test]
fn test_branch_create_command_with_jira_ticket_only_parses_correctly() -> Result<()> {
    // Arrange: 准备只带 JIRA ticket 的 Create 命令输入
    let args = &["test-branch", "create", "PROJ-456"];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证 JIRA ticket 解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}

/// 测试分支创建命令解析--from-default标志
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令能够正确解析 `--from-default` 标志。
///
/// ## 测试场景
/// 1. 准备包含 `--from-default` 标志的命令行输入
/// 2. 解析命令行参数
/// 3. 验证 `from_default` 标志正确设置
///
/// ## 预期结果
/// - 解析成功
/// - from_default为true，其他参数使用默认值
#[test]
fn test_branch_create_command_with_from_default_flag_parses_correctly() -> Result<()> {
    // Arrange: 准备带 --from-default 参数的 Create 命令输入
    let args = &["test-branch", "create", "--from-default"];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证 --from-default 参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}

/// 测试分支创建命令解析--dry-run标志
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令能够正确解析 `--dry-run` 标志。
///
/// ## 测试场景
/// 1. 准备包含 `--dry-run` 标志的命令行输入
/// 2. 解析命令行参数
/// 3. 验证 `dry_run` 标志正确设置
///
/// ## 预期结果
/// - 解析成功
/// - dry_run为true，其他参数使用默认值
#[test]
fn test_branch_create_command_with_dry_run_flag_parses_correctly() -> Result<()> {
    // Arrange: 准备带 --dry-run 参数的 Create 命令输入
    let args = &["test-branch", "create", "--dry-run"];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证 --dry-run 参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}

// ==================== Boundary Condition Tests ====================

/// 测试分支创建命令使用空JIRA ID返回错误
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令在使用空字符串作为JIRA ID时能够正确返回错误（被验证器拒绝）。
///
/// ## 测试场景
/// 1. 准备包含空字符串JIRA ID的命令行输入
/// 2. 尝试解析命令行参数
/// 3. 验证解析失败并返回适当的错误消息
///
/// ## 预期结果
/// - 解析失败，返回错误
/// - 错误消息包含 "JIRA ID"、"empty"、"Invalid" 或 "validation" 等关键词
#[test]
fn test_branch_create_command_with_empty_jira_id_returns_error() -> Result<()> {
    // Arrange: 准备空字符串 JIRA ID 的输入
    // 注意：这是正确的行为，JIRA ID 验证器不允许空字符串
    let args = &["test-branch", "create", ""];

    // Act: 尝试解析空字符串 JIRA ID
    let result = TestBranchCli::try_parse_from(args);

    // Assert: 验证解析失败（空字符串被验证器拒绝）
    match result {
        Ok(_) => return Err(color_eyre::eyre::eyre!("Empty JIRA ID should be rejected by validator")),
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

    Ok(())
}

/// 测试分支创建命令解析超长JIRA ID（边界情况）
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令能够正确处理超长的JIRA ID字符串（边界情况测试）。
///
/// ## 测试场景
/// 1. 准备包含超长JIRA ID（100+字符）的命令行输入
/// 2. 解析命令行参数
/// 3. 验证超长JIRA ID解析正确
///
/// ## 预期结果
/// - 解析成功
/// - JIRA ID正确存储（即使很长）
#[test]
fn test_branch_create_command_with_very_long_jira_id_parses_correctly() -> Result<()> {
    // Arrange: 准备超长 JIRA ID（边界情况）
    let long_jira_id = "PROJ-".to_string() + &"1".repeat(100);
    let args = &["test-branch", "create", &long_jira_id];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证超长 JIRA ID 解析正确
    match cli.command {
        BranchSubcommand::Create { jira_id, .. } => {
            assert_eq!(jira_id.into_option(), Some(long_jira_id));
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}

/// 测试分支创建命令解析包含特殊字符的JIRA ID（边界情况）
///
/// ## 测试目的
/// 验证 `BranchSubcommand::Create` 命令能够正确解析包含特殊字符的JIRA ID字符串（CLI解析层应该接受任何字符串，格式验证在业务逻辑层进行）。
///
/// ## 测试场景
/// 1. 准备包含特殊字符的JIRA ID的命令行输入
/// 2. 解析命令行参数
/// 3. 验证特殊字符JIRA ID解析正确
///
/// ## 注意事项
/// - CLI解析层应该接受任何字符串
/// - 实际业务逻辑可能会验证JIRA ID格式
///
/// ## 预期结果
/// - 解析成功
/// - JIRA ID正确存储（包含特殊字符）
#[test]
fn test_branch_create_command_with_special_characters_in_jira_id_parses_correctly() -> Result<()> {
    // Arrange: 准备包含特殊字符的 JIRA ID（边界情况）
    // 注意：实际业务逻辑可能会验证 JIRA ID 格式，但 CLI 解析应该接受任何字符串
    let special_jira_id = "PROJ-123_test@example.com";
    let args = &["test-branch", "create", special_jira_id];

    // Act: 解析命令行参数
    let cli = TestBranchCli::try_parse_from(args)?;

    // Assert: 验证特殊字符 JIRA ID 解析正确
    match cli.command {
        BranchSubcommand::Create { jira_id, .. } => {
            assert_eq!(jira_id.into_option(), Some(special_jira_id.to_string()));
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Create command")),
    }

    Ok(())
}
