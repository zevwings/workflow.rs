//! 标准测试 Fixture
//!
//! 提供可复用的测试 Fixture，使用 `rstest::fixture` 创建，减少测试代码重复。
//!
//! # 使用示例
//!
//! ```rust
//! use rstest::rstest;
//! use crate::common::fixtures::{git_repo_with_commit, cli_env_with_git, mock_server};
//!
//! #[rstest]
//! fn test_with_git_repo_return_ok(git_repo_with_commit: GitTestEnv) -> Result<()> {
//!     // 使用 fixture
//!     Ok(())
//! }
//! ```

use color_eyre::Result;
use rstest::fixture;

use crate::common::environments::{CliTestEnv, GitTestEnv};
use crate::common::isolation::TestIsolation;
use crate::common::mock::server::MockServer;

/// Git 仓库 Fixture（带初始提交）
///
/// 创建一个已初始化的 Git 测试环境，包含：
/// - 临时目录
/// - Git 仓库（默认分支为 main）
/// - 测试用户配置
/// - 初始提交
///
/// # 使用场景
///
/// 适用于需要 Git 仓库的测试，如：
/// - Git 操作测试
/// - 分支操作测试
/// - 提交操作测试
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::git_repo_with_commit;
///
/// #[rstest]
/// fn test_git_operations_return_ok(git_repo_with_commit: GitTestEnv) -> Result<()> {
///     // git_repo_with_commit 已经初始化了 Git 仓库
///     // 可以直接进行 Git 操作
///     Ok(())
/// }
/// ```
#[fixture]
pub fn git_repo_with_commit() -> GitTestEnv {
    GitTestEnv::new().unwrap_or_else(|e| panic!("Failed to create git test env: {:?}", e))
}

/// CLI 测试环境 Fixture（带 Git 仓库）
///
/// 创建一个 CLI 测试环境，并初始化 Git 仓库，包含：
/// - 临时目录
/// - Git 仓库（默认分支为 main）
/// - 测试用户配置
/// - 初始提交
///
/// # 使用场景
///
/// 适用于需要 CLI 命令测试且需要 Git 仓库的测试，如：
/// - PR 命令测试
/// - Commit 命令测试
/// - Branch 命令测试
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::cli_env_with_git;
///
/// #[rstest]
/// fn test_cli_command_return_ok(cli_env_with_git: CliTestEnv) -> Result<()> {
///     // cli_env_with_git 已经初始化了 Git 仓库
///     // 可以直接进行 CLI 命令测试
///     Ok(())
/// }
/// ```
#[fixture]
pub fn cli_env_with_git() -> CliTestEnv {
    let env =
        CliTestEnv::new().unwrap_or_else(|e| panic!("Failed to create CLI test env: {:?}", e));
    env.init_git_repo()
        .unwrap_or_else(|e| panic!("Failed to init git repo: {:?}", e));
    env
}

/// CLI 测试环境 Fixture（不带 Git 仓库）
///
/// 创建一个基础的 CLI 测试环境，不包含 Git 仓库。
///
/// # 使用场景
///
/// 适用于不需要 Git 仓库的 CLI 命令测试，如：
/// - Config 命令测试
/// - Help 命令测试
/// - Version 命令测试
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::cli_env;
///
/// #[rstest]
/// fn test_cli_command_return_ok(cli_env: CliTestEnv) -> Result<()> {
///     // cli_env 提供临时目录和环境隔离
///     Ok(())
/// }
/// ```
#[fixture]
pub fn cli_env() -> CliTestEnv {
    CliTestEnv::new().unwrap_or_else(|e| panic!("Failed to create CLI test env: {:?}", e))
}

/// CLI 测试环境 Fixture（带 Git 仓库和初始提交）
///
/// 创建一个 CLI 测试环境，初始化 Git 仓库，并创建初始提交，包含：
/// - 临时目录
/// - Git 仓库（默认分支为 main）
/// - 测试用户配置
/// - 测试文件（test.txt）
/// - 初始提交
///
/// # 使用场景
///
/// 适用于需要 Git 仓库且有提交历史的 CLI 命令测试，如：
/// - Commit 命令测试（需要已有提交）
/// - Branch 命令测试（需要提交历史）
/// - PR 命令测试（需要提交历史）
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::cli_env_with_git_and_commit;
///
/// #[rstest]
/// fn test_cli_command_return_ok(cli_env_with_git_and_commit: CliTestEnv) -> Result<()> {
///     // cli_env_with_git_and_commit 已经初始化了 Git 仓库并创建了初始提交
///     // 可以直接进行需要提交历史的 CLI 命令测试
///     Ok(())
/// }
/// ```
#[fixture]
pub fn cli_env_with_git_and_commit() -> CliTestEnv {
    let env =
        CliTestEnv::new().unwrap_or_else(|e| panic!("Failed to create CLI test env: {:?}", e));
    env.init_git_repo()
        .unwrap_or_else(|e| panic!("Failed to init git repo: {:?}", e))
        .create_file("test.txt", "test content")
        .unwrap_or_else(|e| panic!("Failed to create file: {:?}", e))
        .create_commit("Initial commit")
        .unwrap_or_else(|e| panic!("Failed to create commit: {:?}", e));
    env
}

/// Mock 服务器 Fixture
///
/// 创建一个 Mock HTTP 服务器，用于模拟外部 API 调用。
///
/// # 使用场景
///
/// 适用于需要 Mock HTTP 请求的测试，如：
/// - GitHub API 测试
/// - Jira API 测试
/// - HTTP 客户端测试
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::mock_server;
///
/// #[rstest]
/// fn test_http_request_return_ok(mut mock_server: MockServer) -> Result<()> {
///     let url = format!("{}/test", mock_server.base_url);
///     // 配置 Mock 端点
///     // ...
///     Ok(())
/// }
/// ```
#[fixture]
pub fn mock_server() -> MockServer {
    MockServer::new()
}

/// Mock 服务器 Fixture（带 GitHub API 配置）
///
/// 创建一个 Mock HTTP 服务器，并自动配置 GitHub API 环境变量。
///
/// # 使用场景
///
/// 适用于需要 GitHub API Mock 的测试。
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::mock_server_github;
///
/// #[rstest]
/// fn test_github_api_return_ok(mut mock_server_github: MockServer) -> Result<()> {
///     // GITHUB_API_URL 已自动设置
///     // 可以直接配置 GitHub API Mock 端点
///     Ok(())
/// }
/// ```
#[fixture]
pub fn mock_server_github() -> MockServer {
    let server = MockServer::new();
    server.setup_github_base_url();
    server
}

/// Mock 服务器 Fixture（带 Jira API 配置）
///
/// 创建一个 Mock HTTP 服务器，并自动配置 Jira API 环境变量。
///
/// # 使用场景
///
/// 适用于需要 Jira API Mock 的测试。
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::mock_server_jira;
///
/// #[rstest]
/// fn test_jira_api_return_ok(mut mock_server_jira: MockServer) -> Result<()> {
///     // JIRA_API_URL 已自动设置
///     // 可以直接配置 Jira API Mock 端点
///     Ok(())
/// }
/// ```
#[fixture]
pub fn mock_server_jira() -> MockServer {
    let server = MockServer::new();
    server.setup_jira_base_url();
    server
}

// ==================== 简单测试数据 Fixture ====================

/// 测试用的 PR ID
///
/// 用于测试中需要 PR ID 的场景。
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::test_pr_id;
///
/// #[rstest]
/// fn test_pr_command(test_pr_id: &str) {
///     assert_eq!(test_pr_id, "123");
/// }
/// ```
#[fixture]
pub fn test_pr_id() -> &'static str {
    "123"
}

/// 测试用的分支名
///
/// 用于测试中需要分支名的场景。
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::test_branch;
///
/// #[rstest]
/// fn test_branch_command(test_branch: &str) {
///     assert_eq!(test_branch, "feature/my-branch");
/// }
/// ```
#[fixture]
pub fn test_branch() -> &'static str {
    "feature/my-branch"
}

/// 测试用的 Jira ID
///
/// 用于测试中需要 Jira ticket ID 的场景。
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::test_jira_id;
///
/// #[rstest]
/// fn test_jira_command(test_jira_id: &str) {
///     assert_eq!(test_jira_id, "PROJ-123");
/// }
/// ```
#[fixture]
pub fn test_jira_id() -> &'static str {
    "PROJ-123"
}

/// 测试用的备用 Jira ID
///
/// 用于测试中需要多个不同 Jira ticket ID 的场景。
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::{test_jira_id, test_jira_id_alt};
///
/// #[rstest]
/// fn test_jira_commands(test_jira_id: &str, test_jira_id_alt: &str) {
///     assert_eq!(test_jira_id, "PROJ-123");
///     assert_eq!(test_jira_id_alt, "PROJ-456");
/// }
/// ```
#[fixture]
pub fn test_jira_id_alt() -> &'static str {
    "PROJ-456"
}

// ==================== 参数化 Fixture ====================

/// Git 仓库 Fixture（带特定分支）
///
/// 创建一个已初始化的 Git 测试环境，并切换到指定的分支，包含：
/// - 临时目录
/// - Git 仓库（默认分支为 main）
/// - 测试用户配置
/// - 初始提交
/// - 切换到指定分支
///
/// # 参数
///
/// * `branch_name` - 要创建并切换到的分支名（默认: "feature/test"）
///
/// # 使用场景
///
/// 适用于需要特定分支的 Git 操作测试，如：
/// - 分支操作测试
/// - 分支同步测试
/// - 分支合并测试
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::git_repo_with_branch;
///
/// #[rstest]
/// fn test_branch_operations(git_repo_with_branch: GitTestEnv) -> Result<()> {
///     // git_repo_with_branch 已经创建并切换到指定分支
///     // 可以直接进行分支操作测试
///     Ok(())
/// }
///
/// // 使用自定义分支名
/// #[rstest]
/// fn test_custom_branch(#[from(git_repo_with_branch("feature/custom"))] env: GitTestEnv) -> Result<()> {
///     // env 已经切换到 feature/custom 分支
///     Ok(())
/// }
/// ```
#[fixture]
pub fn git_repo_with_branch(#[default("feature/test")] branch_name: &str) -> GitTestEnv {
    let env =
        GitTestEnv::new().unwrap_or_else(|e| panic!("Failed to create git test env: {:?}", e));
    env.checkout_new_branch(branch_name)
        .unwrap_or_else(|e| panic!("Failed to create branch '{}': {:?}", branch_name, e));
    env
}

/// CLI 测试环境 Fixture（带配置文件）
///
/// 创建一个 CLI 测试环境，并创建配置文件，包含：
/// - 临时目录
/// - 环境变量隔离
/// - 配置文件（.workflow/workflow.toml）
///
/// # 参数
///
/// * `config_content` - 配置文件内容（默认: 基础 Jira 配置）
///
/// # 使用场景
///
/// 适用于需要配置文件的 CLI 命令测试，如：
/// - Config 命令测试
/// - 配置读取测试
/// - 配置验证测试
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::cli_env_with_config;
///
/// #[rstest]
/// fn test_config_command(cli_env_with_config: CliTestEnv) -> Result<()> {
///     // cli_env_with_config 已经创建了配置文件
///     // 可以直接进行配置相关的 CLI 命令测试
///     Ok(())
/// }
///
/// // 使用自定义配置内容
/// #[rstest]
/// fn test_custom_config(#[from(cli_env_with_config("[jira]\nurl = \"custom\"\n"))] env: CliTestEnv) -> Result<()> {
///     // env 已经创建了自定义配置文件
///     Ok(())
/// }
/// ```
#[fixture]
pub fn cli_env_with_config(
    #[default("[jira]\nurl = \"test\"")] config_content: &str,
) -> CliTestEnv {
    let env =
        CliTestEnv::new().unwrap_or_else(|e| panic!("Failed to create CLI test env: {:?}", e));
    env.create_config(config_content)
        .unwrap_or_else(|e| panic!("Failed to create config: {:?}", e));
    env
}

/// 基础测试隔离环境 Fixture
///
/// 创建一个基础的测试隔离环境，包含：
/// - 临时目录（使用绝对路径）
/// - 环境变量隔离
///
/// # 使用场景
///
/// 适用于需要基础隔离环境但不需要Git或CLI功能的测试，如：
/// - 配置文件管理测试（需要直接访问 `work_dir()`）
/// - 文件系统操作测试
/// - 需要基础隔离但不需Git/CLI的场景
///
/// # 示例
///
/// ```rust
/// use rstest::rstest;
/// use crate::common::fixtures::test_isolation;
///
/// #[rstest]
/// fn test_config_manager(test_isolation: TestIsolation) -> Result<()> {
///     let config_path = test_isolation.work_dir().join("config.toml");
///     // 使用基础隔离环境进行测试
///     Ok(())
/// }
/// ```
#[fixture]
pub fn test_isolation() -> TestIsolation {
    TestIsolation::new().unwrap_or_else(|e| panic!("Failed to create test isolation: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 git_repo_with_commit fixture
    ///
    /// ## 测试目的
    /// 验证 `git_repo_with_commit` fixture 能够成功创建 Git 测试环境。
    ///
    /// ## 预期结果
    /// - Git 测试环境创建成功
    /// - Git 仓库已初始化
    #[test]
    fn test_git_repo_with_commit_fixture_return_ok() -> Result<()> {
        let env = git_repo_with_commit();
        assert!(env.path().join(".git").exists());
        Ok(())
    }

    /// 测试 cli_env_with_git fixture
    ///
    /// ## 测试目的
    /// 验证 `cli_env_with_git` fixture 能够成功创建 CLI 测试环境并初始化 Git 仓库。
    ///
    /// ## 预期结果
    /// - CLI 测试环境创建成功
    /// - Git 仓库已初始化
    #[test]
    fn test_cli_env_with_git_fixture_return_ok() -> Result<()> {
        let env = cli_env_with_git();
        assert!(env.path().join(".git").exists());
        Ok(())
    }

    /// 测试 mock_server fixture
    ///
    /// ## 测试目的
    /// 验证 `mock_server` fixture 能够成功创建 Mock 服务器。
    ///
    /// ## 预期结果
    /// - Mock 服务器创建成功
    /// - base_url 不为空
    #[test]
    fn test_mock_server_fixture() {
        let server = mock_server();
        assert!(!server.base_url().is_empty());
    }

    /// 测试 cli_env_with_git_and_commit fixture
    ///
    /// ## 测试目的
    /// 验证 `cli_env_with_git_and_commit` fixture 能够成功创建 CLI 测试环境、初始化 Git 仓库并创建初始提交。
    ///
    /// ## 预期结果
    /// - CLI 测试环境创建成功
    /// - Git 仓库已初始化
    /// - 初始提交已创建
    #[test]
    fn test_cli_env_with_git_and_commit_fixture_return_ok() -> Result<()> {
        let env = cli_env_with_git_and_commit();
        assert!(env.path().join(".git").exists());
        // 验证文件存在
        assert!(env.path().join("test.txt").exists());
        Ok(())
    }

    /// 测试简单数据 fixture
    ///
    /// ## 测试目的
    /// 验证简单数据 fixture（test_pr_id, test_branch, test_jira_id, test_jira_id_alt）返回正确的值。
    ///
    /// ## 预期结果
    /// - 所有 fixture 返回预期的值
    #[test]
    fn test_simple_data_fixtures() {
        assert_eq!(test_pr_id(), "123");
        assert_eq!(test_branch(), "feature/my-branch");
        assert_eq!(test_jira_id(), "PROJ-123");
        assert_eq!(test_jira_id_alt(), "PROJ-456");
    }

    /// 测试 git_repo_with_branch fixture
    ///
    /// ## 测试目的
    /// 验证 `git_repo_with_branch` fixture 能够成功创建 Git 测试环境并切换到指定分支。
    ///
    /// ## 预期结果
    /// - Git 测试环境创建成功
    /// - Git 仓库已初始化
    /// - 已切换到指定分支
    #[test]
    fn test_git_repo_with_branch_fixture_return_ok() -> Result<()> {
        let env = git_repo_with_branch("feature/test-branch");
        assert!(env.path().join(".git").exists());
        let current_branch = env.current_branch()?;
        assert_eq!(current_branch, "feature/test-branch");
        Ok(())
    }

    /// 测试 cli_env_with_config fixture
    ///
    /// ## 测试目的
    /// 验证 `cli_env_with_config` fixture 能够成功创建 CLI 测试环境并创建配置文件。
    ///
    /// ## 预期结果
    /// - CLI 测试环境创建成功
    /// - 配置文件已创建
    /// - 配置文件内容正确
    #[test]
    fn test_cli_env_with_config_fixture_return_ok() -> Result<()> {
        let env = cli_env_with_config("[jira]\nurl = \"test\"");
        let config_path = env.path().join(".workflow").join("workflow.toml");
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path)?;
        assert!(content.contains("jira"));
        assert!(content.contains("url = \"test\""));
        Ok(())
    }

    /// 测试 test_isolation fixture
    ///
    /// ## 测试目的
    /// 验证 `test_isolation` fixture 能够成功创建基础测试隔离环境。
    ///
    /// ## 预期结果
    /// - 测试隔离环境创建成功
    /// - 工作目录存在且为目录
    /// - 工作目录路径为绝对路径
    #[test]
    fn test_test_isolation_fixture_return_ok() -> Result<()> {
        let isolation = test_isolation();
        let work_dir = isolation.work_dir();

        // 验证工作目录存在
        assert!(work_dir.exists());
        assert!(work_dir.is_dir());

        // 验证返回的是绝对路径
        assert!(work_dir.is_absolute());

        Ok(())
    }
}
