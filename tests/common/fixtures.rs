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
//! fn test_with_git_repo_return_result(git_repo_with_commit: GitTestEnv) -> Result<()> {
//!     // 使用 fixture
//!     Ok(())
//! }
//! ```

use color_eyre::Result;
use rstest::fixture;

use crate::common::environments::{CliTestEnv, GitTestEnv};
use crate::common::http_helpers::MockServer;

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
/// fn test_git_operations_return_result(git_repo_with_commit: GitTestEnv) -> Result<()> {
///     // git_repo_with_commit 已经初始化了 Git 仓库
///     // 可以直接进行 Git 操作
///     Ok(())
/// }
/// ```
#[fixture]
pub fn git_repo_with_commit() -> GitTestEnv {
    GitTestEnv::new().expect("Failed to create git test env")
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
/// fn test_cli_command_return_result(cli_env_with_git: CliTestEnv) -> Result<()> {
///     // cli_env_with_git 已经初始化了 Git 仓库
///     // 可以直接进行 CLI 命令测试
///     Ok(())
/// }
/// ```
#[fixture]
pub fn cli_env_with_git() -> CliTestEnv {
    let env = CliTestEnv::new().expect("Failed to create CLI test env");
    env.init_git_repo().expect("Failed to init git repo");
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
/// fn test_cli_command_return_result(cli_env: CliTestEnv) -> Result<()> {
///     // cli_env 提供临时目录和环境隔离
///     Ok(())
/// }
/// ```
#[fixture]
pub fn cli_env() -> CliTestEnv {
    CliTestEnv::new().expect("Failed to create CLI test env")
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
/// fn test_http_request_return_result(mut mock_server: MockServer) -> Result<()> {
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
/// fn test_github_api_return_result(mut mock_server_github: MockServer) -> Result<()> {
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
/// fn test_jira_api_return_result(mut mock_server_jira: MockServer) -> Result<()> {
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
    fn test_git_repo_with_commit_fixture_return_result() -> Result<()> {
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
    fn test_cli_env_with_git_fixture_return_result() -> Result<()> {
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
}

