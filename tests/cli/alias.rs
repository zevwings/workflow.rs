//! Alias CLI 命令测试
//!
//! 测试 Alias CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use pretty_assertions::assert_eq;
use workflow::cli::AliasSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-alias")]
struct TestAliasCli {
    #[command(subcommand)]
    command: AliasSubcommand,
}

// ==================== List 命令测试 ====================

#[test]
fn test_alias_list_command() {
    // 测试 List 命令（无参数）
    let cli = TestAliasCli::try_parse_from(&["test-alias", "list"]).unwrap();

    match cli.command {
        AliasSubcommand::List => {
            // List 命令没有参数
            assert!(true, "List command should have no parameters");
        }
        _ => panic!("Expected List command"),
    }
}

// ==================== Add 命令测试 ====================

#[test]
fn test_alias_add_command_direct() {
    // 测试 Add 命令（直接模式，提供 name 和 command）
    let cli = TestAliasCli::try_parse_from(&[
        "test-alias",
        "add",
        "ci",
        "pr create",
    ])
    .unwrap();

    match cli.command {
        AliasSubcommand::Add { name, command } => {
            assert_eq!(name, Some("ci".to_string()));
            assert_eq!(command, Some("pr create".to_string()));
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_alias_add_command_name_only() {
    // 测试 Add 命令（只提供 name）
    let cli = TestAliasCli::try_parse_from(&["test-alias", "add", "ci"]).unwrap();

    match cli.command {
        AliasSubcommand::Add { name, command } => {
            assert_eq!(name, Some("ci".to_string()));
            assert_eq!(command, None);
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_alias_add_command_interactive() {
    // 测试 Add 命令（交互式模式，无参数）
    let cli = TestAliasCli::try_parse_from(&["test-alias", "add"]).unwrap();

    match cli.command {
        AliasSubcommand::Add { name, command } => {
            assert_eq!(name, None);
            assert_eq!(command, None);
        }
        _ => panic!("Expected Add command"),
    }
}

// ==================== Remove 命令测试 ====================

#[test]
fn test_alias_remove_command_direct() {
    // 测试 Remove 命令（直接模式，提供 name）
    let cli = TestAliasCli::try_parse_from(&["test-alias", "remove", "ci"]).unwrap();

    match cli.command {
        AliasSubcommand::Remove { name } => {
            assert_eq!(name, Some("ci".to_string()));
        }
        _ => panic!("Expected Remove command"),
    }
}

#[test]
fn test_alias_remove_command_interactive() {
    // 测试 Remove 命令（交互式模式，无参数）
    let cli = TestAliasCli::try_parse_from(&["test-alias", "remove"]).unwrap();

    match cli.command {
        AliasSubcommand::Remove { name } => {
            assert_eq!(name, None);
        }
        _ => panic!("Expected Remove command"),
    }
}

// ==================== 命令枚举测试 ====================

#[test]
fn test_alias_commands_enum_all_variants() {
    // 测试所有命令变体
    let commands = vec![
        ("list", |cmd: &AliasSubcommand| matches!(cmd, AliasSubcommand::List)),
        (
            "add",
            |cmd: &AliasSubcommand| matches!(cmd, AliasSubcommand::Add { .. }),
        ),
        (
            "remove",
            |cmd: &AliasSubcommand| matches!(cmd, AliasSubcommand::Remove { .. }),
        ),
    ];

    for (subcommand, assert_fn) in commands {
        let mut args = vec!["test-alias", subcommand];
        // 为需要参数的命令添加最小参数
        match subcommand {
            "add" => {
                args.push("test");
                args.push("command");
            }
            "remove" => args.push("test"),
            _ => {}
        }

        let cli = TestAliasCli::try_parse_from(&args).unwrap();
        assert!(
            assert_fn(&cli.command),
            "Command '{}' should match expected variant",
            subcommand
        );
    }
}

#[test]
fn test_alias_commands_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestAliasCli::try_parse_from(&["test-alias", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}
