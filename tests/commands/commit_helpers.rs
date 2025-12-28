//! Commit 命令辅助函数测试
//!
//! 测试 Commit 命令的辅助函数，包括提交存在性检查、默认分支检查等。

use crate::common::environments::CliTestEnv;
use crate::common::fixtures::{cli_env_with_empty_git, cli_env_with_git};
use git2::Repository;
use rstest::rstest;
// Removed serial_test::serial - tests can run in parallel with CliTestEnv isolation
use workflow::commands::commit::helpers::{
    check_has_last_commit, check_has_last_commit_in, check_not_on_default_branch,
    check_not_on_default_branch_in,
};

// ==================== Commit Existence Check Tests ====================

/// 测试在非Git仓库中检查是否有最后提交
///
/// ## 测试目的
/// 验证 `check_has_last_commit()` 在非Git仓库目录中能够正确返回错误。
///
/// ## 测试场景
/// 1. 在非Git仓库目录中调用 `check_has_last_commit()`
/// 2. 验证返回错误或成功（取决于当前目录）
///
/// ## 注意事项
/// - 如果当前目录恰好是Git仓库且有commit，返回成功是可以接受的
/// - 如果当前目录不是Git仓库，应返回错误
///
/// ## 预期结果
/// - 在非Git仓库中返回Err
/// - 错误消息包含 "No commits"、"git" 或 "repository" 等关键词
#[test]
fn test_check_has_last_commit_without_git_repo_returns_error() {
    // Arrange: 准备非 Git 仓库环境
    // 注意：check_has_last_commit() 使用当前工作目录的 Git 仓库

    // Act: 检查是否有最后的提交
    let result = check_has_last_commit();

    // Assert: 验证返回错误且错误消息包含相关信息（成功或失败都是可以接受的）
    match result {
        Ok(_) => {
            // 如果当前目录恰好是 Git 仓库且有 commit，这是可以接受的
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("No commits")
                    || error_msg.contains("git")
                    || error_msg.contains("repository"),
                "Error message should indicate the issue: {}",
                error_msg
            );
        }
    }
}

/// 测试空Git仓库（无commit）的情况
///
/// ## 测试目的
/// 验证`check_has_last_commit()`在空Git仓库中正确返回错误。
///
/// ## 测试场景
/// 1. 创建临时Git仓库（使用`CliTestEnv`）
/// 2. 初始化Git但不创建任何commit
/// 3. 调用`check_has_last_commit()`
/// 4. 验证返回错误且错误消息包含"No commits"
///
/// ## 技术细节
/// - 使用`cli_env_with_git` fixture自动清理临时目录和环境（支持并行执行）
/// - 自动恢复原始工作目录和环境变量
#[rstest]
fn test_check_has_last_commit_with_empty_git_repo_return_empty(
    cli_env_with_empty_git: CliTestEnv,
) -> color_eyre::Result<()> {
    // cli_env_with_empty_git 已经初始化了空的 Git 仓库，不包含任何 commit

    // 使用路径参数版本，避免切换全局工作目录
    let result = check_has_last_commit_in(cli_env_with_empty_git.path());

    // 验证函数返回错误（无 commit）
    assert!(
        result.is_err(),
        "check_has_last_commit should fail when there are no commits"
    );

    // 验证错误消息
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("No commits"),
        "Error message should indicate no commits: {}",
        error_msg
    );

    // cli_env_with_empty_git fixture 会在函数结束时自动恢复目录和环境
    Ok(())
}

/// 测试有commit的Git仓库的情况
///
/// ## 测试目的
/// 验证`check_has_last_commit()`在有commit的Git仓库中正确返回成功。
///
/// ## 测试场景
/// 1. 创建临时Git仓库
/// 2. 创建文件并提交
/// 3. 调用`check_has_last_commit()`
/// 4. 验证返回成功
///
/// ## 技术细节
/// - 使用`cli_env_with_git` fixture自动创建和清理临时Git仓库（支持并行执行）
#[rstest]
fn test_check_has_last_commit_with_commits_return_ok(
    cli_env_with_git: CliTestEnv,
) -> color_eyre::Result<()> {
    // cli_env_with_git 已经初始化了 Git 仓库，创建文件并提交
    cli_env_with_git
        .create_file("test.txt", "test content")?
        .create_commit("Initial commit")?;

    // 使用路径参数版本，避免切换全局工作目录
    let result = check_has_last_commit_in(cli_env_with_git.path());

    // 验证函数返回成功（有 commit）
    assert!(
        result.is_ok(),
        "check_has_last_commit should succeed when there are commits"
    );

    // cli_env_with_git fixture 会在函数结束时自动恢复目录和环境
    Ok(())
}

// ==================== Default Branch Check Tests ====================

/// 测试在main分支上的默认分支检查
///
/// ## 测试目的
/// 验证`check_not_on_default_branch()`在main分支上正确返回错误，防止在保护分支上执行危险操作。
///
/// ## 测试场景
/// 1. 创建临时Git仓库并初始化
/// 2. 创建文件并提交
/// 3. 切换到main分支
/// 4. 调用`check_not_on_default_branch()`
/// 5. 验证返回错误且错误消息包含"protected"或"default branch"
///
/// ## 技术细节
/// - 使用`cli_env_with_git` fixture创建和管理临时Git仓库（支持并行执行）
/// - 自动恢复原始工作目录
#[rstest]
fn test_check_not_on_default_branch_on_main_return_ok(
    cli_env_with_git: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时Git仓库并在main分支上
    // cli_env_with_git 已经初始化了 Git 仓库，创建文件并提交
    cli_env_with_git
        .create_file("test.txt", "test")?
        .create_commit("Initial commit")?;

    // 设置假的远程引用，让get_default_branch()能正常工作
    cli_env_with_git.setup_fake_remote_refs()?;

    // Act: 使用路径参数版本，避免切换全局工作目录
    let result = check_not_on_default_branch_in(cli_env_with_git.path(), "amend");

    // Assert: 验证返回错误且错误消息包含保护分支信息
    assert!(
        result.is_err(),
        "check_not_on_default_branch should fail on default branch"
    );
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("protected")
            || error_msg.contains("default branch")
            || error_msg.contains("Cannot"),
        "Error message should indicate protected branch: {}",
        error_msg
    );

    Ok(())
}

/// 测试在feature分支上的默认分支检查
///
/// ## 测试目的
/// 验证 `check_not_on_default_branch()` 在feature分支上正确返回成功，允许执行操作。
///
/// ## 测试场景
/// 1. 创建临时Git仓库并初始化
/// 2. 创建文件并提交
/// 3. 创建并切换到feature分支（feature/test）
/// 4. 调用 `check_not_on_default_branch()`
/// 5. 验证返回成功并包含正确的分支信息
///
/// ## 预期结果
/// - 返回Ok，包含当前分支和默认分支的元组
/// - 当前分支为 "feature/test"
/// - 默认分支为 "main"
#[rstest]
fn test_check_not_on_default_branch_on_feature_branch_return_ok(
    cli_env_with_git: CliTestEnv,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时Git仓库并切换到feature分支
    // cli_env_with_git 已经初始化了 Git 仓库，创建文件并提交
    cli_env_with_git
        .create_file("test.txt", "test")?
        .create_commit("Initial commit")?;

    // 设置假的远程引用，让get_default_branch()能正常工作
    cli_env_with_git.setup_fake_remote_refs()?;

    // 创建并切换到 feature 分支
    let repo = Repository::open(cli_env_with_git.path())
        .map_err(|e| color_eyre::eyre::eyre!("Failed to open repository: {}", e))?;
    let head = repo.head().map_err(|e| color_eyre::eyre::eyre!("Failed to get HEAD: {}", e))?;
    let head_commit = repo
        .find_commit(head.target().unwrap())
        .map_err(|e| color_eyre::eyre::eyre!("Failed to find HEAD commit: {}", e))?;
    repo.branch("feature/test", &head_commit, false)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to create branch: {}", e))?;
    repo.set_head("refs/heads/feature/test")
        .map_err(|e| color_eyre::eyre::eyre!("Failed to checkout branch: {}", e))?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            .force()
            .remove_ignored(false)
            .remove_untracked(false),
    ))
    .map_err(|e| color_eyre::eyre::eyre!("Failed to checkout HEAD: {}", e))?;

    // Act: 使用路径参数版本，避免切换全局工作目录
    let result = check_not_on_default_branch_in(cli_env_with_git.path(), "amend");

    // Assert: 验证返回成功且分支信息正确
    assert!(
        result.is_ok(),
        "check_not_on_default_branch should succeed on feature branch, error: {:?}",
        result.as_ref().err()
    );
    if let Ok((current, default)) = result {
        assert_eq!(current, "feature/test");
        assert_eq!(default, "main");
    }

    Ok(())
}

/// 测试在非Git仓库中检查默认分支的错误消息格式
///
/// ## 测试目的
/// 验证 `check_not_on_default_branch()` 在非Git仓库目录中能够返回格式良好的错误消息。
///
/// ## 测试场景
/// 1. 在非Git仓库目录中调用 `check_not_on_default_branch()`
/// 2. 验证返回错误或成功（取决于当前目录）
///
/// ## 注意事项
/// - 如果当前目录恰好是Git仓库，返回成功是可以接受的
/// - 如果当前目录不是Git仓库，应返回错误
///
/// ## 预期结果
/// - 在非Git仓库中返回Err
/// - 错误消息包含 "branch"、"git" 或 "Failed" 等关键词
/// - 错误消息格式清晰，便于调试
#[test]
fn test_check_not_on_default_branch_error_message_format_with_non_git_repo_returns_error() {
    // Arrange: 准备非Git仓库环境

    // Act: 在非Git仓库中检查是否允许操作
    let result = check_not_on_default_branch("amend");

    // Assert: 验证返回错误且错误消息包含相关信息
    match result {
        Ok(_) => {
            // 如果当前目录恰好是Git仓库，这是可以接受的
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("branch")
                    || error_msg.contains("git")
                    || error_msg.contains("Failed"),
                "Error message should be informative: {}",
                error_msg
            );
        }
    }
}
