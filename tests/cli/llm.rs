//! LLM CLI 命令测试
//!
//! 测试 LLM CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::LLMSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-llm")]
struct TestLlmCli {
    #[command(subcommand)]
    command: LLMSubcommand,
}

// ==================== Command Parsing Tests ====================

#[test]
fn test_llm_command_with_all_subcommands_parses_successfully() {
    // Arrange: 准备所有子命令的输入
    let show_args = &["test-llm", "show"];
    let setup_args = &["test-llm", "setup"];

    // Act: 解析命令行参数
    let show_cli = TestLlmCli::try_parse_from(show_args)
        .expect("CLI args should parse successfully");
    let setup_cli = TestLlmCli::try_parse_from(setup_args)
        .expect("CLI args should parse successfully");

    // Assert: 验证所有子命令都可以正确解析
    assert!(matches!(show_cli.command, LLMSubcommand::Show));
    assert!(matches!(setup_cli.command, LLMSubcommand::Setup));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_llm_command_with_invalid_subcommand_returns_error() {
    // Arrange: 准备无效子命令的输入
    let invalid_args = &["test-llm", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestLlmCli::try_parse_from(invalid_args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_llm_command_with_missing_subcommand_returns_error() {
    // Arrange: 准备缺少子命令的输入
    let missing_args = &["test-llm"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestLlmCli::try_parse_from(missing_args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_llm_show_command_with_extra_arguments_returns_error() {
    // Arrange: 准备包含额外参数的 Show 命令输入
    let extra_args = &["test-llm", "show", "extra-arg"];

    // Act: 尝试解析包含额外参数的命令
    let result = TestLlmCli::try_parse_from(extra_args);

    // Assert: 验证返回错误
    assert!(
        result.is_err(),
        "Show command should not accept extra arguments"
    );
}

#[test]
fn test_llm_setup_command_with_extra_arguments_returns_error() {
    // Arrange: 准备包含额外参数的 Setup 命令输入
    let extra_args = &["test-llm", "setup", "extra-arg"];

    // Act: 尝试解析包含额外参数的命令
    let result = TestLlmCli::try_parse_from(extra_args);

    // Assert: 验证返回错误
    assert!(
        result.is_err(),
        "Setup command should not accept extra arguments"
    );
}

#[test]
fn test_llm_command_with_uppercase_subcommand_returns_error() {
    // Arrange: 准备大写子命令的输入（clap 默认区分大小写）
    let uppercase_show_args = &["test-llm", "SHOW"];
    let uppercase_setup_args = &["test-llm", "SETUP"];

    // Act: 尝试解析大写子命令
    let show_result = TestLlmCli::try_parse_from(uppercase_show_args);
    let setup_result = TestLlmCli::try_parse_from(uppercase_setup_args);

    // Assert: 验证大写命令返回错误
    assert!(
        show_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
    assert!(
        setup_result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
}

// 枚举变体完整性已通过 test_llm_command_parsing_all_subcommands 测试验证
