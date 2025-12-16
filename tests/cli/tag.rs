//! Tag CLI 命令测试
//!
//! 测试 Tag CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use pretty_assertions::assert_eq;
use workflow::cli::TagSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-tag")]
struct TestTagCli {
    #[command(subcommand)]
    command: TagSubcommand,
}

// ==================== Delete 命令测试 ====================

#[test]
fn test_tag_delete_command_direct() {
    // 测试 Delete 命令（直接模式，提供 tag_name）
    let cli = TestTagCli::try_parse_from(&["test-tag", "delete", "v1.0.0"]).unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            local,
            remote,
            pattern,
            dry_run,
            force,
        } => {
            assert_eq!(tag_name, Some("v1.0.0".to_string()));
            assert!(!local);
            assert!(!remote);
            assert_eq!(pattern, None);
            assert!(!dry_run.is_dry_run());
            assert!(!force);
        }
    }
}

#[test]
fn test_tag_delete_command_local_only() {
    // 测试 Delete 命令（只删除本地 tag）
    let cli = TestTagCli::try_parse_from(&["test-tag", "delete", "v1.0.0", "--local"]).unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            local,
            remote,
            ..
        } => {
            assert_eq!(tag_name, Some("v1.0.0".to_string()));
            assert!(local);
            assert!(!remote);
        }
    }
}

#[test]
fn test_tag_delete_command_remote_only() {
    // 测试 Delete 命令（只删除远程 tag）
    let cli = TestTagCli::try_parse_from(&["test-tag", "delete", "v1.0.0", "--remote"]).unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            local,
            remote,
            ..
        } => {
            assert_eq!(tag_name, Some("v1.0.0".to_string()));
            assert!(!local);
            assert!(remote);
        }
    }
}

#[test]
fn test_tag_delete_command_with_pattern() {
    // 测试 Delete 命令（使用模式匹配）
    let cli = TestTagCli::try_parse_from(&[
        "test-tag",
        "delete",
        "--pattern",
        "v1.*",
    ])
    .unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            pattern,
            ..
        } => {
            assert_eq!(tag_name, None);
            assert_eq!(pattern, Some("v1.*".to_string()));
        }
    }
}

#[test]
fn test_tag_delete_command_with_dry_run() {
    // 测试 Delete 命令（dry-run 模式）
    let cli = TestTagCli::try_parse_from(&[
        "test-tag",
        "delete",
        "v1.0.0",
        "--dry-run",
    ])
    .unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            dry_run,
            ..
        } => {
            assert_eq!(tag_name, Some("v1.0.0".to_string()));
            assert!(dry_run.is_dry_run());
        }
    }
}

#[test]
fn test_tag_delete_command_with_force() {
    // 测试 Delete 命令（force 模式）
    let cli = TestTagCli::try_parse_from(&["test-tag", "delete", "v1.0.0", "--force"]).unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            force,
            ..
        } => {
            assert_eq!(tag_name, Some("v1.0.0".to_string()));
            assert!(force);
        }
    }
}

#[test]
fn test_tag_delete_command_interactive() {
    // 测试 Delete 命令（交互式模式，无参数）
    let cli = TestTagCli::try_parse_from(&["test-tag", "delete"]).unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            pattern,
            ..
        } => {
            assert_eq!(tag_name, None);
            assert_eq!(pattern, None);
        }
    }
}

#[test]
fn test_tag_delete_command_all_options() {
    // 测试 Delete 命令（所有选项）
    let cli = TestTagCli::try_parse_from(&[
        "test-tag",
        "delete",
        "v1.0.0",
        "--local",
        "--dry-run",
        "--force",
    ])
    .unwrap();

    match cli.command {
        TagSubcommand::Delete {
            tag_name,
            local,
            remote,
            dry_run,
            force,
            ..
        } => {
            assert_eq!(tag_name, Some("v1.0.0".to_string()));
            assert!(local);
            assert!(!remote);
            assert!(dry_run.is_dry_run());
            assert!(force);
        }
    }
}

// ==================== 命令枚举测试 ====================

#[test]
fn test_tag_commands_enum_all_variants() {
    // 测试所有命令变体
    let cli = TestTagCli::try_parse_from(&["test-tag", "delete", "v1.0.0"]).unwrap();

    match cli.command {
        TagSubcommand::Delete { .. } => {
            assert!(true, "Delete command should be parsed correctly");
        }
    }
}

#[test]
fn test_tag_commands_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestTagCli::try_parse_from(&["test-tag", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}
