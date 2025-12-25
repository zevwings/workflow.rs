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

// ==================== 命令结构测试 ====================

#[test]
fn test_llm_command_parsing_all_subcommands() {
    // 测试所有子命令都可以正确解析

    // Show
    let cli = TestLlmCli::try_parse_from(&["test-llm", "show"]).expect("CLI args should parse successfully");
    assert!(matches!(cli.command, LLMSubcommand::Show));

    // Setup
    let cli = TestLlmCli::try_parse_from(&["test-llm", "setup"]).expect("CLI args should parse successfully");
    assert!(matches!(cli.command, LLMSubcommand::Setup));
}

#[test]
fn test_llm_command_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestLlmCli::try_parse_from(&["test-llm", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_llm_command_error_handling_missing_subcommand() {
    // 测试缺少子命令的错误处理
    let result = TestLlmCli::try_parse_from(&["test-llm"]);
    assert!(result.is_err(), "Should fail when subcommand is missing");
}

#[test]
fn test_llm_show_command_no_arguments() {
    // 测试 Show 命令不接受额外参数
    let result = TestLlmCli::try_parse_from(&["test-llm", "show", "extra-arg"]);
    assert!(
        result.is_err(),
        "Show command should not accept extra arguments"
    );
}

#[test]
fn test_llm_setup_command_no_arguments() {
    // 测试 Setup 命令不接受额外参数
    let result = TestLlmCli::try_parse_from(&["test-llm", "setup", "extra-arg"]);
    assert!(
        result.is_err(),
        "Setup command should not accept extra arguments"
    );
}

#[test]
fn test_llm_command_case_sensitivity() {
    // 测试命令大小写敏感性（clap 默认区分大小写）
    // 大写命令应该失败
    let result = TestLlmCli::try_parse_from(&["test-llm", "SHOW"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );

    let result = TestLlmCli::try_parse_from(&["test-llm", "SETUP"]);
    assert!(
        result.is_err(),
        "Uppercase commands should fail (clap is case-sensitive by default)"
    );
}

// 枚举变体完整性已通过 test_llm_command_parsing_all_subcommands 测试验证
