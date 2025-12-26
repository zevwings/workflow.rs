//! 基础 CLI 测试
//!
//! 使用新的测试工具进行基础 CLI 功能测试。

use crate::common::cli_helpers::{
    contains_error, is_json_format, CliCommandBuilder, TestDataGenerator,
};
use crate::common::environments::CliTestEnv;
use crate::common::fixtures::{cli_env, cli_env_with_git};
use rstest::rstest;

// ==================== Basic Command Tests ====================

/// 测试帮助命令
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_help_command() {
    let binding = CliCommandBuilder::new().arg("--help").assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workflow"));
    assert!(stdout.contains("USAGE") || stdout.contains("Usage"));
}

/// 测试版本命令
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_version_command() {
    let binding = CliCommandBuilder::new().arg("--version").assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workflow"));
}

// ==================== PR 命令测试 ====================

/// 测试PR帮助命令
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_pr_help() {
    let binding = CliCommandBuilder::new().args(["pr", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pr"));
}

/// 测试在非Git仓库中执行PR命令
///
/// ## 测试目的
/// 验证当不在Git仓库中时，PR命令能够正确检测并返回清晰的错误消息。
///
/// ## 为什么被忽略
/// - **可能初始化客户端**: 即使在非Git仓库中，可能仍尝试初始化Jira/GitHub客户端
/// - **长时间阻塞**: 客户端初始化可能导致测试阻塞
/// - **平台限制**: Windows上已通过cfg跳过
/// - **CI时间考虑**: 避免在CI中长时间阻塞
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_pr_without_git_repo -- --ignored
/// ```
/// 注意：此测试在非Windows平台上运行
///
/// ## 测试场景
/// 1. 创建临时目录（非Git仓库）
/// 2. 在该目录中执行`pr create --dry-run`命令
/// 3. 命令应该失败
/// 4. 验证错误消息提示"没有Git仓库"
///
/// ## 预期行为
/// - 命令执行失败（非零退出码）
/// - stderr包含错误消息
/// - 错误消息清晰说明原因（不在Git仓库中）
/// - 不会尝试创建PR或调用远程API
#[rstest]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：可能尝试初始化服务，导致长时间阻塞
#[ignore] // 忽略：可能尝试初始化 Jira/GitHub 客户端，导致长时间阻塞
fn test_pr_without_git_repo_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    // 注意：此测试执行 pr create 命令，即使没有 Git 仓库也可能尝试初始化服务
    // Windows 上已通过 #[cfg] 跳过，因为可能尝试初始化 Jira/GitHub 客户端，导致阻塞
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(cli_env.path())
        .assert_failure();
    let output = binding.get_output();

    // 应该提示没有 Git 仓库
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr));
    Ok(())
}

/// 测试在Git仓库中执行PR命令
///
/// ## 测试目的
/// 验证在有效的Git仓库中，PR命令能够正确执行（dry-run模式）。
///
/// ## 为什么被忽略
/// - **需要网络连接**: 可能需要访问GitHub/GitLab API
/// - **可能调用LLM**: 可能需要LLM服务生成PR描述
/// - **初始化客户端**: 可能初始化Jira/GitHub客户端导致阻塞
/// - **平台限制**: Windows上Git命令和路径处理可能有差异
/// - **集成测试**: 这是一个端到端的集成测试
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_pr_with_git_repo -- --ignored --nocapture
/// ```
/// **警告**: 此测试需要在有效的Git仓库中运行，非Windows平台
///
/// ## 测试场景
/// 1. 创建临时Git仓库
/// 2. 初始化Git仓库（git init）
/// 3. 创建至少一个commit
/// 4. 执行`pr create --dry-run`命令
/// 5. 验证命令执行（可能调用API或LLM）
///
/// ## 预期行为
/// - 命令能够识别Git仓库
/// - dry-run模式不会实际创建PR
/// - 可能显示将要创建的PR信息
/// - 如果需要API/LLM，应该有合适的错误消息
#[rstest]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：Git 命令和路径处理可能有问题
#[ignore] // 忽略：可能涉及网络请求或 LLM 调用，导致长时间阻塞
fn test_pr_with_git_repo_return_ok(cli_env_with_git: CliTestEnv) -> color_eyre::Result<()> {
    // 注意：此测试可能尝试初始化 Jira/GitHub 客户端或调用 LLM，导致阻塞
    // Windows 上已通过 #[cfg] 跳过，因为：
    // - Git 命令路径或行为差异
    // - 路径分隔符处理（虽然 Rust Path 应该处理，但某些情况下可能仍有问题）
    // - 临时目录路径格式差异
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    // cli_env_with_git 已经初始化了 Git 仓库，创建文件并提交
    cli_env_with_git
        .create_file("README.md", "# Test")?
        .create_commit("Initial commit")?;

    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(cli_env_with_git.path())
        .assert();
    let output = binding.get_output();

    // 在有 Git 仓库的情况下，可能成功或失败（取决于配置）
    // 但不应该是因为没有 Git 仓库而失败
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("not a git repository"));
    Ok(())
}

// ==================== Branch 命令测试 ====================

/// 测试Branch帮助命令
///
/// ## 测试目的
/// 验证 Branch 命令的帮助信息能够正确显示。
///
/// ## 测试场景
/// 1. 执行 `workflow branch --help` 命令
/// 2. 验证帮助信息输出
///
/// ## 预期结果
/// - 帮助信息正确显示
#[test]
fn test_branch_help() {
    let binding = CliCommandBuilder::new().args(["branch", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("branch"));
}

/// 测试在没有Git仓库时执行Branch命令（应返回错误）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_branch_without_git_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    let binding = CliCommandBuilder::new()
        .args(["branch", "create", "test-branch"])
        .current_dir(cli_env.path())
        .assert_failure();
    let output = binding.get_output();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr));
    Ok(())
}

// ==================== Config 命令测试 ====================

/// 测试Config帮助命令
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_config_help() {
    let binding = CliCommandBuilder::new().args(["config", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config"));
}

/// 测试Config显示命令
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_config_show_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    cli_env.create_config(&TestDataGenerator::config_content())?;

    let binding = CliCommandBuilder::new()
        .args(["config", "show"])
        .current_dir(cli_env.path())
        .assert();
    let output = binding.get_output();

    // 配置显示可能成功或失败，但应该有输出
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.is_empty() || !stderr.is_empty());
    Ok(())
}

// ==================== Jira 命令测试 ====================

/// 测试Jira帮助命令
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_jira_help() {
    let binding = CliCommandBuilder::new().args(["jira", "--help"]).assert_success();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jira"));
}

/// 测试在没有配置时执行Jira info命令（应返回错误）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[rstest]
fn test_jira_info_without_config_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    let binding = CliCommandBuilder::new()
        .args(["jira", "info", "TEST-123"])
        .current_dir(cli_env.path())
        .assert_failure();
    let output = binding.get_output();

    // 没有配置时应该失败
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr));
    Ok(())
}

// ==================== Output Format Tests ====================

/// 测试JSON输出格式
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
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

// ==================== Parameter Validation Tests ====================

/// 测试无效命令（应返回错误）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
#[test]
fn test_invalid_command() {
    let binding = CliCommandBuilder::new().arg("invalid-command").assert_failure();
    let output = binding.get_output();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(contains_error(&stderr) || stderr.contains("invalid") || stderr.contains("unknown"));
}

/// 测试缺少必需参数时的错误处理
///
/// ## 测试目的
/// 验证CLI在缺少必需参数时，能够正确进行参数验证并返回清晰的错误消息。
///
/// ## 为什么被忽略
/// - **可能初始化客户端**: 即使缺少参数，可能仍尝试初始化Jira客户端导致阻塞
/// - **参数验证时机**: 参数验证可能在客户端初始化之后进行
/// - **平台限制**: Windows上错误消息格式可能不同
/// - **CI时间考虑**: 避免因客户端初始化而阻塞
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_missing_required_argument -- --ignored
/// ```
/// 注意：此测试在非Windows平台上运行
///
/// ## 测试场景
/// 1. 构造缺少必需参数的jira命令
/// 2. 执行命令（例如：`jira info`缺少issue ID）
/// 3. 触发参数验证逻辑
/// 4. 验证返回错误消息
///
/// ## 预期行为
/// - 命令执行失败（非零退出码）
/// - 错误消息明确指出缺少哪个参数
/// - 错误消息提示正确的使用方法
/// - 理想情况下，不应初始化不必要的客户端
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

// ==================== Environment Variable Tests ====================

/// 测试环境变量设置
#[rstest]
fn test_environment_variables_return_ok(cli_env: CliTestEnv) -> color_eyre::Result<()> {
    let binding = CliCommandBuilder::new()
        .args(["config", "show"])
        .env("WORKFLOW_CONFIG_DIR", cli_env.path())
        .current_dir(cli_env.path())
        .assert();
    let output = binding.get_output();

    // 环境变量设置后，命令应该能运行（可能成功或失败，但不应该崩溃）
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.is_empty() || !stderr.is_empty());
    Ok(())
}

// ==================== Performance Tests ====================

/// 测试帮助命令的性能（应快速执行）
///
/// ## 测试目的
/// 验证测试函数能够正确执行预期功能。
///
/// ## 测试场景
/// 1. 准备测试数据
/// 2. 执行被测试的操作
/// 3. 验证结果
///
/// ## 预期结果
/// - 测试通过，无错误
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

// ==================== Integration Tests ====================

/// 测试完整工作流程的dry-run模式
///
/// ## 测试目的
/// 验证完整的开发工作流程在dry-run模式下能够正确执行（不实际修改数据）。
///
/// ## 为什么被忽略
/// - **执行多个命令**: 涉及多个子命令的连续执行
/// - **可能涉及网络**: 某些命令可能需要网络请求
/// - **可能调用LLM**: 某些命令可能需要LLM服务
/// - **初始化多个服务**: 可能初始化Jira/GitHub客户端
/// - **平台限制**: Windows上多命令执行和路径处理可能有差异
/// - **集成测试**: 这是一个完整的端到端集成测试
/// - **测试时间长**: 多个命令的执行需要更多时间
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_complete_workflow_dry_run -- --ignored --nocapture
/// ```
/// 注意：此测试在非Windows平台上运行，可能需要网络连接
///
/// ## 测试场景
/// 1. 创建完整的测试环境（Git仓库+配置）
/// 2. 初始化Git仓库并创建初始commit
/// 3. 创建工作流配置文件
/// 4. 执行一系列命令（如：branch create, pr create等）
/// 5. 所有命令使用--dry-run模式
/// 6. 验证命令输出和状态
///
/// ## 预期行为
/// - 所有命令在dry-run模式下执行
/// - 不实际创建分支、PR或Issue
/// - 不修改Git仓库状态
/// - 显示将要执行的操作计划
/// - 命令之间的依赖关系正确处理
/// - 配置正确加载和使用
#[rstest]
#[cfg(not(target_os = "windows"))] // Windows 上跳过：多个命令执行和路径处理可能有问题
#[ignore] // 忽略：执行多个命令，可能涉及网络请求或 LLM 调用，导致长时间阻塞
fn test_complete_workflow_dry_run_return_ok(
    cli_env_with_git: CliTestEnv,
) -> color_eyre::Result<()> {
    // 注意：此测试执行多个命令，其中一些可能尝试初始化服务或调用 LLM，导致阻塞
    // Windows 上已通过 #[cfg] 跳过，因为：
    // - 多个 Git 命令执行可能更慢
    // - 路径处理在多个命令间可能不一致
    // - 配置文件路径格式差异
    // 如果需要运行此测试，请使用: cargo test -- --ignored
    // cli_env_with_git 已经初始化了 Git 仓库，创建文件并提交
    cli_env_with_git
        .create_file("src/main.rs", "fn main() {}")?
        .create_commit("Initial commit")?
        .create_config(&TestDataGenerator::config_content())?;

    // 尝试完整的工作流（dry-run 模式）
    let commands = vec![
        vec!["branch", "create", "feature/test", "--dry-run"],
        vec!["pr", "create", "--dry-run"],
        vec!["config", "show"],
    ];

    for cmd_args in commands {
        let binding = CliCommandBuilder::new()
            .args(&cmd_args)
            .current_dir(cli_env_with_git.path())
            .assert();
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
    Ok(())
}
