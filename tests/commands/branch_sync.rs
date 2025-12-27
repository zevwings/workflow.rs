//! Branch Sync 命令测试
//!
//! 测试分支同步命令的功能。

// Removed serial_test::serial - tests can run in parallel with GitTestEnv + MockServer isolation
use workflow::commands::branch::sync::BranchSyncCommand;

use crate::common::environments::GitTestEnv;
use crate::common::fixtures::git_repo_with_commit;
use crate::common::mock::server::MockServer;
use rstest::rstest;

// ============================================================================
// 测试版本 - 使用临时Git仓库 + Mock GitHub API
// ============================================================================

/// 测试分支同步命令结构（使用Mock）
///
/// ## 测试目的
/// 验证 `BranchSyncCommand::sync()` 方法在使用 Mock 服务器时的基本功能和结构。
///
/// ## 测试场景
/// 1. 创建 Git 测试环境
/// 2. 设置 Mock GitHub API 服务器
/// 3. 调用分支同步命令
/// 4. 验证命令结构正确
///
/// ## 技术细节
/// - 使用 GitTestEnv 创建隔离的 Git 环境
/// - 使用 MockServer 模拟 GitHub API
/// - 完全隔离，快速执行
///
/// ## 预期结果
/// - 命令结构正确
/// - Mock 服务器正常工作
#[rstest]
fn test_branch_sync_command_structure(git_repo_with_commit: GitTestEnv) -> color_eyre::Result<()> {
    // 1. 设置Mock GitHub API
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // Mock GitHub BASE URL的GET请求（用于网络检查）
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 2. 创建临时Git仓库
    let git_env = &git_repo_with_commit;

    // 4. 创建测试分支
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("feature.txt", "feature content", "Add feature")?;

    // 5. 切换回main分支
    git_env.checkout("main")?;

    // 6. 执行分支同步
    let result = BranchSyncCommand::sync_in(
        git_env.path(),
        "feature".to_string(),
        false, // rebase
        false, // ff_only
        false, // squash
    );

    // 7. 验证结果（GitTestEnv 会在函数结束时自动恢复目录和环境）
    assert!(
        result.is_ok(),
        "Branch sync should succeed with mock GitHub API"
    );

    Ok(())
}

/// 测试使用rebase的分支同步
///
/// ## 测试目的
/// 验证 `BranchSyncCommand::sync()` 方法在使用 `rebase` 选项时能够正确执行。
///
/// ## 测试场景
/// 1. 创建 Git 测试环境
/// 2. 设置 Mock GitHub API 服务器
/// 3. 创建分支结构
/// 4. 执行rebase同步
/// 5. 验证结果
///
/// ## 预期结果
/// - Rebase同步成功
#[rstest]
fn test_branch_sync_command_with_rebase(
    git_repo_with_commit: GitTestEnv,
) -> color_eyre::Result<()> {
    // 设置Mock
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 创建Git环境
    let git_env = &git_repo_with_commit;

    // 创建分支结构：main <- feature
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("feature1.txt", "content 1", "Feature commit 1")?;
    git_env.make_test_commit("feature2.txt", "content 2", "Feature commit 2")?;

    git_env.checkout("main")?;

    // 执行rebase同步
    let result = BranchSyncCommand::sync_in(
        git_env.path(),
        "feature".to_string(),
        true, // rebase
        false,
        false,
    );

    // Assert: 验证（目录自动恢复）
    assert!(result.is_ok(), "Rebase sync should succeed");

    Ok(())
}

/// 测试fast-forward only同步
///
/// ## 测试目的
/// 验证 `BranchSyncCommand::sync()` 方法在使用 `ff_only`（fast-forward only）选项时能够正确执行。
///
/// ## 测试场景
/// 1. 创建 Git 测试环境
/// 2. 设置 Mock GitHub API 服务器
/// 3. 创建可以fast-forward的分支结构
/// 4. 执行ff-only同步
/// 5. 验证结果
///
/// ## 预期结果
/// - FF-only同步成功
#[rstest]
fn test_branch_sync_command_with_ff_only(
    git_repo_with_commit: GitTestEnv,
) -> color_eyre::Result<()> {
    // 设置Mock
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 创建Git环境
    let git_env = &git_repo_with_commit;

    // 创建可以fast-forward的分支结构
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("feature.txt", "content", "Feature commit")?;

    git_env.checkout("main")?;

    // 执行ff-only同步
    let result = BranchSyncCommand::sync_in(
        git_env.path(),
        "feature".to_string(),
        false,
        true, // ff_only
        false,
    );

    // Assert: 验证（目录自动恢复）
    assert!(
        result.is_ok(),
        "FF-only sync should succeed for fast-forwardable branches"
    );

    Ok(())
}

/// 测试squash合并
///
/// ## 测试目的
/// 验证`BranchSyncCommand::sync()`使用squash选项时的行为。
///
/// ## 测试场景
/// 1. 创建 Git 测试环境
/// 2. 设置 Mock GitHub API 服务器
/// 3. 创建多个提交的分支
/// 4. 执行squash同步
/// 5. 验证结果
///
/// ## 预期结果
/// - Squash同步成功
#[rstest]
fn test_branch_sync_command_with_squash(
    git_repo_with_commit: GitTestEnv,
) -> color_eyre::Result<()> {
    // 设置Mock
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 创建Git环境
    let git_env = &git_repo_with_commit;

    // 创建多个提交的分支
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("file1.txt", "content 1", "Commit 1")?;
    git_env.make_test_commit("file2.txt", "content 2", "Commit 2")?;
    git_env.make_test_commit("file3.txt", "content 3", "Commit 3")?;

    git_env.checkout("main")?;

    // 执行squash同步
    let result = BranchSyncCommand::sync_in(
        git_env.path(),
        "feature".to_string(),
        false,
        false,
        true, // squash
    );

    // Assert: 验证（目录自动恢复）
    assert!(result.is_ok(), "Squash sync should succeed");

    Ok(())
}
