//! Proxy CLI 命令测试
//!
//! 测试 Proxy CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::ProxySubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-proxy")]
struct TestProxyCli {
    #[command(subcommand)]
    command: ProxySubcommand,
}

// ==================== 命令结构测试 ====================

#[test]
fn test_proxy_command_parsing_all_subcommands() {
    // 测试所有子命令都可以正确解析

    // On
    let cli = TestProxyCli::try_parse_from(&["test-proxy", "on"]).unwrap();
    assert!(matches!(cli.command, ProxySubcommand::On));

    // Off
    let cli = TestProxyCli::try_parse_from(&["test-proxy", "off"]).unwrap();
    assert!(matches!(cli.command, ProxySubcommand::Off));

    // Check
    let cli = TestProxyCli::try_parse_from(&["test-proxy", "check"]).unwrap();
    assert!(matches!(cli.command, ProxySubcommand::Check));
}

#[test]
fn test_proxy_command_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestProxyCli::try_parse_from(&["test-proxy", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_proxy_command_error_handling_missing_subcommand() {
    // 测试缺少子命令的错误处理
    let result = TestProxyCli::try_parse_from(&["test-proxy"]);
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_proxy_all_commands_no_extra_arguments() {
    // 测试所有命令都不接受额外参数
    let commands = ["on", "off", "check"];

    for cmd in commands.iter() {
        let result = TestProxyCli::try_parse_from(&["test-proxy", cmd, "extra-arg"]);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}

#[test]
fn test_proxy_command_case_sensitivity() {
    // 测试命令大小写敏感性（clap 默认区分大小写）
    // 大写命令应该失败
    let result = TestProxyCli::try_parse_from(&["test-proxy", "ON"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );

    let result = TestProxyCli::try_parse_from(&["test-proxy", "OFF"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );

    let result = TestProxyCli::try_parse_from(&["test-proxy", "CHECK"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
}

// 枚举变体完整性已通过 test_proxy_command_parsing_all_subcommands 测试验证

#[test]
fn test_proxy_command_short_names() {
    // 测试命令的简短名称（如果有定义）
    // 这些测试验证命令的基本解析功能
    // 注意：ProxySubcommand 没有定义短名称，所以只测试完整名称

    // On
    let cli = TestProxyCli::try_parse_from(&["test-proxy", "on"]).unwrap();
    assert!(matches!(cli.command, ProxySubcommand::On));

    // Off
    let cli = TestProxyCli::try_parse_from(&["test-proxy", "off"]).unwrap();
    assert!(matches!(cli.command, ProxySubcommand::Off));

    // Check
    let cli = TestProxyCli::try_parse_from(&["test-proxy", "check"]).unwrap();
    assert!(matches!(cli.command, ProxySubcommand::Check));
}
