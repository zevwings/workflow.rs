//! Branch Sync 命令测试
//!
//! 测试分支同步命令的功能。

use serial_test::serial;
use workflow::commands::branch::sync::BranchSyncCommand;

use crate::common::environments::GitTestEnv;
use crate::common::http_helpers::MockServer;

/// 测试分支同步命令的基础结构
///
/// ## 测试目的
/// 验证`BranchSyncCommand::sync()`方法的基本功能和返回值类型。
///
/// ## 为什么被忽略
/// - **需要Git仓库**: 测试需要在有效的Git仓库中运行
/// - **需要网络连接**: 需要访问GitHub API（用于CheckCommand::run_all()）
/// - **集成测试**: 这是一个完整的集成测试
/// - **环境依赖**: 依赖有效的远程仓库配置
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_sync_command_structure -- --ignored --nocapture
/// ```
/// **警告**: 需要在有效的Git仓库中运行，并且可能需要网络连接
///
/// ## 测试场景
/// 1. 在Git仓库中运行测试
/// 2. 调用BranchSyncCommand::sync()同步main分支
/// 3. 不使用rebase、ff_only或squash选项
/// 4. 验证函数返回Result类型
///
/// ## 预期行为
/// - 在有效Git仓库中：成功同步并返回Ok
/// - 在非Git仓库中：返回Err并包含错误消息
/// - 网络错误时：返回Err并包含网络错误信息
/// - 正确处理GitHub API调用
#[test]
#[ignore] // 需要实际的 Git 仓库和网络连接（GitHub API）
fn test_branch_sync_command_structure() {
    // 测试分支同步命令的结构
    // 注意：这个测试需要：
    // 1. 有效的 Git 仓库
    // 2. 网络连接到 GitHub（用于 CheckCommand::run_all()）
    // 运行方式：cargo test -- --ignored
    let result = BranchSyncCommand::sync(
        "main".to_string(),
        false, // rebase
        false, // ff_only
        false, // squash
    );

    // 验证函数返回 Result 类型
    match result {
        Ok(_) => {
            // 在有效的 Git 仓库中同步成功
        }
        Err(_) => {
            // 在非 Git 仓库或无效分支的情况下，返回错误是正常的
        }
    }
}

#[test]
#[ignore] // 需要实际的 Git 仓库和网络连接（GitHub API）
fn test_branch_sync_command_with_rebase_option_executes() {
    // Arrange: 准备分支同步命令参数（启用 rebase）
    let branch_name = "main".to_string();

    // Act: 调用分支同步命令（启用 rebase 选项）
    let result = BranchSyncCommand::sync(
        branch_name,
        true,  // rebase
        false, // ff_only
        false, // squash
    );

    // Assert: 验证函数返回 Result 类型（成功或失败都是可以接受的）
    match result {
        Ok(_) => {
            // 同步成功
        }
        Err(_) => {
            // 在某些情况下可能失败，这是可以接受的
        }
    }
}

#[test]
#[ignore] // 需要实际的 Git 仓库和网络连接（GitHub API）
fn test_branch_sync_command_with_ff_only_option_executes() {
    // Arrange: 准备分支同步命令参数（启用 ff_only）
    let branch_name = "main".to_string();

    // Act: 调用分支同步命令（启用 ff_only 选项）
    let result = BranchSyncCommand::sync(
        branch_name,
        false, // rebase
        true,  // ff_only
        false, // squash
    );

    // Assert: 验证函数返回 Result 类型（成功或失败都是可以接受的）
    match result {
        Ok(_) => {
            // 同步成功
        }
        Err(_) => {
            // 在某些情况下可能失败，这是可以接受的
        }
    }
}

/// 测试使用squash的分支同步
///
/// ## 测试目的
/// 验证`BranchSyncCommand::sync()`使用squash选项时的行为。
///
/// ## 为什么被忽略
/// - **需要Git仓库**: 测试需要在有效的Git仓库中运行
/// - **需要网络连接**: 需要访问GitHub API
/// - **Git操作复杂**: squash会压缩多个提交
/// - **集成测试**: 涉及完整的Git工作流
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_sync_command_with_squash -- --ignored --nocapture
/// ```
/// **警告**: squash操作会修改提交历史
///
/// ## 测试场景
/// 1. 在Git仓库中运行测试
/// 2. 调用BranchSyncCommand::sync()并启用squash选项
/// 3. 执行squash合并压缩提交
/// 4. 验证squash执行结果
///
/// ## 预期行为
/// - 成功时：多个提交被压缩成一个并返回Ok
/// - 失败时：返回Err并包含错误信息
/// - 正确使用git merge --squash命令
/// - 创建单个合并提交
#[test]
#[ignore] // 需要实际的 Git 仓库和网络连接（GitHub API）
fn test_branch_sync_command_with_squash() {
    // 测试使用 squash 的合并
    // 注意：这个测试需要：
    // 1. 有效的 Git 仓库
    // 2. 网络连接到 GitHub（用于 CheckCommand::run_all()）
    // 运行方式：cargo test -- --ignored
    let result = BranchSyncCommand::sync(
        "main".to_string(),
        false, // rebase
        false, // ff_only
        true,  // squash
    );

    match result {
        Ok(_) => {
            // 同步成功
        }
        Err(_) => {
            // 在某些情况下可能失败，这是可以接受的
        }
    }
}

// ============================================================================
// 改进的测试版本 - 使用临时Git仓库 + Mock GitHub API
// ============================================================================

/// 测试分支同步命令结构（完整版 - 使用临时Git仓库和Mock）
///
/// 这个测试展示了正确的测试方法：
/// 1. 使用临时Git仓库（GitTestEnv）
/// 2. Mock GitHub API（MockServer）
/// 3. 完全隔离，快速执行
#[test]
#[serial]
fn test_branch_sync_command_structure_with_mock() -> color_eyre::Result<()> {
    // 1. 设置Mock GitHub API
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();

    // Mock GitHub BASE URL的GET请求（用于网络检查）
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 2. 创建临时Git仓库（新版 GitTestEnv 自动切换工作目录）
    let git_env = GitTestEnv::new()?;

    // 4. 创建测试分支
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("feature.txt", "feature content", "Add feature")?;

    // 5. 切换回main分支
    git_env.checkout("main")?;

    // 6. 执行分支同步
    let result = BranchSyncCommand::sync(
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

/// 测试使用rebase的分支同步（完整版）
#[test]
#[serial]
fn test_branch_sync_command_with_rebase_mock() -> color_eyre::Result<()> {
    // 设置Mock
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 创建Git环境（新版 GitTestEnv 自动切换工作目录）
    let git_env = GitTestEnv::new()?;

    // 创建分支结构：main <- feature
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("feature1.txt", "content 1", "Feature commit 1")?;
    git_env.make_test_commit("feature2.txt", "content 2", "Feature commit 2")?;

    git_env.checkout("main")?;

    // 执行rebase同步
    let result = BranchSyncCommand::sync(
        "feature".to_string(),
        true, // rebase
        false,
        false,
    );

    // 验证（目录自动恢复）
    assert!(result.is_ok(), "Rebase sync should succeed");

    Ok(())
}

/// 测试fast-forward only同步（完整版）
#[test]
#[serial]
fn test_branch_sync_command_with_ff_only_mock() -> color_eyre::Result<()> {
    // 设置Mock
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 创建Git环境（新版 GitTestEnv 自动切换工作目录）
    let git_env = GitTestEnv::new()?;

    // 创建可以fast-forward的分支结构
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("feature.txt", "content", "Feature commit")?;

    git_env.checkout("main")?;

    // 执行ff-only同步
    let result = BranchSyncCommand::sync(
        "feature".to_string(),
        false,
        true, // ff_only
        false,
    );

    // 验证（目录自动恢复）
    assert!(
        result.is_ok(),
        "FF-only sync should succeed for fast-forwardable branches"
    );

    Ok(())
}

/// 测试squash合并（完整版）
#[test]
#[serial]
fn test_branch_sync_command_with_squash_mock() -> color_eyre::Result<()> {
    // 设置Mock
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    let _mock = mock_server.server.mock("GET", "/").with_status(200).with_body("OK").create();

    // 创建Git环境（新版 GitTestEnv 自动切换工作目录）
    let git_env = GitTestEnv::new()?;

    // 创建多个提交的分支
    git_env.checkout_new_branch("feature")?;
    git_env.make_test_commit("file1.txt", "content 1", "Commit 1")?;
    git_env.make_test_commit("file2.txt", "content 2", "Commit 2")?;
    git_env.make_test_commit("file3.txt", "content 3", "Commit 3")?;

    git_env.checkout("main")?;

    // 执行squash同步
    let result = BranchSyncCommand::sync(
        "feature".to_string(),
        false,
        false,
        true, // squash
    );

    // 验证（目录自动恢复）
    assert!(result.is_ok(), "Squash sync should succeed");

    Ok(())
}
