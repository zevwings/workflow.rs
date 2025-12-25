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

// ==================== Command Parsing Tests ====================

#[test]
fn test_proxy_command_with_all_subcommands_parses_successfully() {
    // Arrange: 准备所有子命令的输入
    let on_args = &["test-proxy", "on"];
    let off_args = &["test-proxy", "off"];
    let check_args = &["test-proxy", "check"];

    // Act: 解析所有子命令
    let on_cli = TestProxyCli::try_parse_from(on_args)
        .expect("CLI args should parse successfully");
    let off_cli = TestProxyCli::try_parse_from(off_args)
        .expect("CLI args should parse successfully");
    let check_cli = TestProxyCli::try_parse_from(check_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(on_cli.command, ProxySubcommand::On));
    assert!(matches!(off_cli.command, ProxySubcommand::Off));
    assert!(matches!(check_cli.command, ProxySubcommand::Check));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_proxy_command_with_invalid_subcommand_returns_error() {
    // Arrange: 准备无效子命令的输入
    let args = &["test-proxy", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestProxyCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_proxy_command_with_missing_subcommand_returns_error() {
    // Arrange: 准备缺少子命令的输入
    let args = &["test-proxy"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestProxyCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_proxy_all_commands_with_extra_arguments_return_error() {
    // Arrange: 准备所有命令和额外参数
    let commands = ["on", "off", "check"];

    // Act & Assert: 验证所有命令都不接受额外参数
    for cmd in commands.iter() {
        let args = &["test-proxy", cmd, "extra-arg"];
        let result = TestProxyCli::try_parse_from(args);
        assert!(
            result.is_err(),
            "{} command should not accept extra arguments",
            cmd
        );
    }
}

#[test]
fn test_proxy_command_with_uppercase_subcommand_returns_error() {
    // Arrange: 准备大写子命令的输入（clap 默认区分大小写）
    let on_args = &["test-proxy", "ON"];
    let off_args = &["test-proxy", "OFF"];
    let check_args = &["test-proxy", "CHECK"];

    // Act: 尝试解析大写子命令
    let on_result = TestProxyCli::try_parse_from(on_args);
    let off_result = TestProxyCli::try_parse_from(off_args);
    let check_result = TestProxyCli::try_parse_from(check_args);

    // Assert: 验证大写命令返回错误
    assert!(
        on_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
    assert!(
        off_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
    assert!(
        check_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
}

// ==================== Command Name Tests ====================

#[test]
fn test_proxy_command_with_full_names_parses_successfully() {
    // Arrange: 准备完整命令名称的输入
    // 注意：ProxySubcommand 没有定义短名称，所以只测试完整名称
    let on_args = &["test-proxy", "on"];
    let off_args = &["test-proxy", "off"];
    let check_args = &["test-proxy", "check"];

    // Act: 解析所有命令
    let on_cli = TestProxyCli::try_parse_from(on_args)
        .expect("CLI args should parse successfully");
    let off_cli = TestProxyCli::try_parse_from(off_args)
        .expect("CLI args should parse successfully");
    let check_cli = TestProxyCli::try_parse_from(check_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证命令可以正确解析
    assert!(matches!(on_cli.command, ProxySubcommand::On));
    assert!(matches!(off_cli.command, ProxySubcommand::Off));
    assert!(matches!(check_cli.command, ProxySubcommand::Check));
}
