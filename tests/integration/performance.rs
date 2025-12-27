//! 性能集成测试
//!
//! 测试关键路径的性能，确保不会出现性能回归。
//!
//! 注意：这些测试默认被忽略，只在性能测试时运行。
//! 使用 `cargo test -- --ignored` 来运行这些测试。

use std::time::Duration;
#[cfg(feature = "performance-tests")]
use {
    crate::common::cli_helpers::CliCommandBuilder, crate::common::environments::CliTestEnv,
    crate::common::performance::measure_test_time_with_threshold, color_eyre::Result,
};

/// 性能测试配置
///
/// 定义各个操作的超时时间阈值。
#[allow(dead_code)]
pub struct PerformanceConfig {
    /// PR创建超时时间
    pub pr_creation_timeout: Duration,
    /// 分支创建超时时间
    pub branch_creation_timeout: Duration,
    /// 提交创建超时时间
    pub commit_timeout: Duration,
    /// 配置加载超时时间
    pub config_load_timeout: Duration,
    /// Git操作超时时间
    pub git_operation_timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        // 从环境变量读取超时时间，如果没有设置则使用默认值
        let pr_timeout = std::env::var("PERF_PR_CREATION_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let branch_timeout = std::env::var("PERF_BRANCH_CREATION_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);

        let commit_timeout = std::env::var("PERF_COMMIT_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let config_timeout = std::env::var("PERF_CONFIG_LOAD_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let git_timeout = std::env::var("PERF_GIT_OPERATION_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2);

        Self {
            pr_creation_timeout: Duration::from_secs(pr_timeout),
            branch_creation_timeout: Duration::from_secs(branch_timeout),
            commit_timeout: Duration::from_secs(commit_timeout),
            config_load_timeout: Duration::from_secs(config_timeout),
            git_operation_timeout: Duration::from_secs(git_timeout),
        }
    }
}

// ==================== PR 性能测试 ====================

/// 测试PR创建性能
///
/// ## 测试目的
/// 验证PR创建流程的性能，确保不会出现性能回归。
///
/// ## 为什么被忽略
/// - 性能测试需要更多时间
/// - 可能涉及网络请求
/// - 只在需要性能验证时运行
///
/// ## 如何运行
/// ```bash
/// cargo test test_pr_creation_performance -- --ignored
/// ```
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_pr_creation_performance() -> Result<()> {
    let config = PerformanceConfig::default();

    measure_test_time_with_threshold(
        "test_pr_creation_performance",
        config.pr_creation_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?
                .create_file("README.md", "# Test")?
                .create_commit("Initial commit")?
                .create_branch("feature/test")?
                .checkout("feature/test")?
                .create_file("test.txt", "content")?
                .create_commit("feat: add test")?;

            // 执行PR创建（dry-run模式）
            let _output = CliCommandBuilder::new()
                .args(["pr", "create", "--dry-run"])
                .current_dir(env.path())
                .assert()
                .get_output();

            Ok(())
        },
    )
}

// ==================== 分支操作性能测试 ====================

/// 测试分支创建性能
///
/// ## 测试目的
/// 验证分支创建操作的性能。
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_branch_creation_performance() -> Result<()> {
    let config = PerformanceConfig::default();

    measure_test_time_with_threshold(
        "test_branch_creation_performance",
        config.branch_creation_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?
                .create_file("README.md", "# Test")?
                .create_commit("Initial commit")?
                .create_branch("feature/test")?;

            Ok(())
        },
    )
}

/// 测试分支切换性能
///
/// ## 测试目的
/// 验证分支切换操作的性能。
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_branch_checkout_performance() -> Result<()> {
    let config = PerformanceConfig::default();

    measure_test_time_with_threshold(
        "test_branch_checkout_performance",
        config.git_operation_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?
                .create_file("README.md", "# Test")?
                .create_commit("Initial commit")?
                .create_branch("feature/test")?
                .checkout("feature/test")?;

            Ok(())
        },
    )
}

// ==================== 提交操作性能测试 ====================

/// 测试提交创建性能
///
/// ## 测试目的
/// 验证提交创建操作的性能。
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_commit_creation_performance() -> Result<()> {
    let config = PerformanceConfig::default();

    measure_test_time_with_threshold(
        "test_commit_creation_performance",
        config.commit_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?
                .create_file("test.txt", "content")?
                .create_commit("feat: add test")?;

            Ok(())
        },
    )
}

/// 测试多个提交的性能
///
/// ## 测试目的
/// 验证创建多个提交时的性能。
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_multiple_commits_performance() -> Result<()> {
    // 10个提交应该在合理时间内完成（每个提交约1秒）
    let expected_timeout = Duration::from_secs(15);

    measure_test_time_with_threshold(
        "test_multiple_commits_performance",
        expected_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?;

            // 创建10个提交
            for i in 1..=10 {
                env.create_file(&format!("file{}.txt", i), &format!("content {}", i))?
                    .create_commit(&format!("feat: add file {}", i))?;
            }

            Ok(())
        },
    )
}

// ==================== 配置操作性能测试 ====================

/// 测试配置加载性能
///
/// ## 测试目的
/// 验证配置文件加载的性能。
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_config_load_performance() -> Result<()> {
    let config = PerformanceConfig::default(); // 用于获取超时配置

    measure_test_time_with_threshold(
        "test_config_load_performance",
        config.config_load_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.create_config(
                r#"
[jira]
url = "https://test.atlassian.net"
username = "test@example.com"
"#,
            )?;

            // 验证配置文件存在
            let env_path = env.path();
            let config_path = env_path.join(".workflow").join("workflow.toml");
            assert!(config_path.exists());

            Ok(())
        },
    )
}

// ==================== Git 操作性能测试 ====================

/// 测试Git仓库初始化性能
///
/// ## 测试目的
/// 验证Git仓库初始化的性能。
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_git_init_performance() -> Result<()> {
    let config = PerformanceConfig::default(); // 用于获取超时配置

    measure_test_time_with_threshold(
        "test_git_init_performance",
        config.git_operation_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?;

            Ok(())
        },
    )
}

/// 测试完整工作流性能
///
/// ## 测试目的
/// 验证完整工作流（初始化、分支、提交、PR）的性能。
///
/// ## 测试场景
/// 1. 初始化Git仓库
/// 2. 创建分支
/// 3. 创建提交
/// 4. 创建PR（dry-run）
#[test]
#[ignore] // 只在性能测试时运行
#[cfg(feature = "performance-tests")]
fn test_complete_workflow_performance() -> Result<()> {
    // 完整工作流应该在合理时间内完成
    let expected_timeout = Duration::from_secs(10);

    measure_test_time_with_threshold(
        "test_complete_workflow_performance",
        expected_timeout,
        || {
            let env = CliTestEnv::new()?;
            env.init_git_repo()?
                .create_file("README.md", "# Test")?
                .create_commit("Initial commit")?
                .create_branch("feature/test")?
                .checkout("feature/test")?
                .create_file("test.txt", "content")?
                .create_commit("feat: add test")?;

            // PR创建（dry-run）
            let binding = CliCommandBuilder::new()
                .args(["pr", "create", "--dry-run"])
                .current_dir(env.path())
                .assert();
            let _output = binding.get_output();

            Ok(())
        },
    )
}
