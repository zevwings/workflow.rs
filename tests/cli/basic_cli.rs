//! 基础 CLI 测试
//!
//! 使用新的测试工具进行基础 CLI 功能测试。

use crate::common::cli_helpers::{
    contains_error, is_json_format, CliCommandBuilder, CliTestEnv, TestDataGenerator,
};

// ==================== 基本命令测试 ====================

#[test]
fn test_help_command() {
    let binding = CliCommandBuilder::new().arg("--help").assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workflow"));
    assert!(stdout.contains("USAGE") || stdout.contains("Usage"));
}

#[test]
fn test_version_command() {
    let binding = CliCommandBuilder::new().arg("--version").assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workflow"));
}

// ==================== PR 命令测试 ====================

#[test]
fn test_pr_help() {
    let binding = CliCommandBuilder::new().args(["pr", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pr"));
}

#[test]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：可能尝试初始化服务，导致长时间阻塞
#[ignore] // 忽略：可能尝试初始化 Jira/GitHub 客户端，导致长时间阻塞
fn test_pr_without_git_repo() {
    // 注意：此测试执行 pr create 命令，即使没有 Git 仓库也可能尝试初始化服务
    // Windows 上已通过 #[cfg] 跳过，因为可能尝试初始化 Jira/GitHub 客户端，导致阻塞
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    let env = CliTestEnv::new();

    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(env.path())
        .assert_failure();
    let output = binding.get_output();

    // 应该提示没有 Git 仓库
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr));
}

#[test]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：Git 命令和路径处理可能有问题
#[ignore] // 忽略：可能涉及网络请求或 LLM 调用，导致长时间阻塞
fn test_pr_with_git_repo() {
    // 注意：此测试可能尝试初始化 Jira/GitHub 客户端或调用 LLM，导致阻塞
    // Windows 上已通过 #[cfg] 跳过，因为：
    // - Git 命令路径或行为差异
    // - 路径分隔符处理（虽然 Rust Path 应该处理，但某些情况下可能仍有问题）
    // - 临时目录路径格式差异
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("README.md", "# Test")
        .create_commit("Initial commit");

    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 在有 Git 仓库的情况下，可能成功或失败（取决于配置）
    // 但不应该是因为没有 Git 仓库而失败
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("not a git repository"));
}

// ==================== Branch 命令测试 ====================

#[test]
fn test_branch_help() {
    let binding = CliCommandBuilder::new().args(["branch", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("branch"));
}

#[test]
fn test_branch_without_git() {
    let env = CliTestEnv::new();

    let binding = CliCommandBuilder::new()
        .args(["branch", "create", "test-branch"])
        .current_dir(env.path())
        .assert_failure();
    let output = binding.get_output();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr));
}

// ==================== Config 命令测试 ====================

#[test]
fn test_config_help() {
    let binding = CliCommandBuilder::new().args(["config", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config"));
}

#[test]
fn test_config_show() {
    let env = CliTestEnv::new();
    env.create_config(&TestDataGenerator::config_content());

    let binding = CliCommandBuilder::new()
        .args(["config", "show"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 配置显示可能成功或失败，但应该有输出
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.is_empty() || !stderr.is_empty());
}

// ==================== Jira 命令测试 ====================

#[test]
fn test_jira_help() {
    let binding = CliCommandBuilder::new().args(["jira", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jira"));
}

#[test]
fn test_jira_info_without_config() {
    let env = CliTestEnv::new();

    let binding = CliCommandBuilder::new()
        .args(["jira", "info", "TEST-123"])
        .current_dir(env.path())
        .assert_failure();
    let output = binding.get_output();

    // 没有配置时应该失败
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr));
}

// ==================== 输出格式测试 ====================

#[test]
fn test_json_output_format() {
    let binding = CliCommandBuilder::new().args(["--help", "--format", "json"]).assert();
    let output = binding.get_output();

    // 检查是否支持 JSON 格式（可能不支持，这是正常的）
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 如果支持 JSON 格式，输出应该是 JSON
    if is_json_format(&stdout) {
        // JSON 格式输出
        assert!(stdout.starts_with('{'));
    } else {
        // 不支持 JSON 格式，应该有正常的帮助输出
        assert!(stdout.contains("workflow") || stderr.contains("format"));
    }
}

// ==================== 参数验证测试 ====================

#[test]
fn test_invalid_command() {
    let binding = CliCommandBuilder::new().arg("invalid-command").assert_failure();
    let output = binding.get_output();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr) || stderr.contains("invalid") || stderr.contains("unknown"));
}

#[test]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：错误消息格式可能不同
#[ignore] // 忽略：可能尝试初始化 Jira 客户端，导致长时间阻塞
fn test_missing_required_argument() {
    // 注意：此测试可能尝试初始化 Jira 客户端，即使缺少参数也可能导致阻塞
    // Windows 上已通过 #[cfg] 跳过，因为错误消息格式可能不同
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    let binding = CliCommandBuilder::new()
        .args(["jira", "info"]) // 缺少 issue ID
        .assert_failure();
    let output = binding.get_output();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr) || stderr.contains("required") || stderr.contains("missing"));
}

// ==================== 环境变量测试 ====================

#[test]
fn test_environment_variables() {
    let env = CliTestEnv::new();

    let binding = CliCommandBuilder::new()
        .args(["config", "show"])
        .env("WORKFLOW_CONFIG_DIR", env.path())
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 环境变量设置后，命令应该能运行（可能成功或失败，但不应该崩溃）
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.is_empty() || !stderr.is_empty());
}

// ==================== 性能测试 ====================

#[test]
fn test_help_command_performance() {
    use std::time::Instant;

    let start = Instant::now();

    let _output = CliCommandBuilder::new().arg("--help").assert_success().get_output();

    let duration = start.elapsed();

    // 帮助命令应该很快（< 5秒）
    assert!(
        duration.as_secs() < 5,
        "Help command too slow: {:?}",
        duration
    );
}

// ==================== 集成测试 ====================

#[test]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：多个命令执行和路径处理可能有问题
#[ignore] // 忽略：执行多个命令，可能涉及网络请求或 LLM 调用，导致长时间阻塞
fn test_complete_workflow_dry_run() {
    // 注意：此测试执行多个命令，其中一些可能尝试初始化服务或调用 LLM，导致阻塞
    // Windows 上已通过 #[cfg] 跳过，因为：
    // - 多个 Git 命令执行可能更慢
    // - 路径处理在多个命令间可能不一致
    // - 配置文件路径格式差异
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("src/main.rs", "fn main() {}")
        .create_commit("Initial commit")
        .create_config(&TestDataGenerator::config_content());

    // 尝试完整的工作流（dry-run 模式）
    let commands = vec![
        vec!["branch", "create", "feature/test", "--dry-run"],
        vec!["pr", "create", "--dry-run"],
        vec!["config", "show"],
    ];

    for cmd_args in commands {
        let binding = CliCommandBuilder::new().args(&cmd_args).current_dir(env.path()).assert();
        let output = binding.get_output();

        // 每个命令都应该有输出（成功或失败都可以）
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stdout.is_empty() || !stderr.is_empty(),
            "Command {:?} produced no output",
            cmd_args
        );
    }
}
