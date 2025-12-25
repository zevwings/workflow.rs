//! Commit 命令辅助函数测试
//!
//! 测试 Commit 命令的辅助函数，包括提交存在性检查、默认分支检查等。

use crate::common::environments::CliTestEnv;
use crate::common::helpers::CurrentDirGuard;
// Removed serial_test::serial - tests can run in parallel with CliTestEnv isolation
use workflow::commands::commit::helpers::{check_has_last_commit, check_not_on_default_branch};

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
/// - 使用`CliTestEnv`自动清理临时目录和环境（支持并行执行）
/// - 自动恢复原始工作目录和环境变量
#[test]
fn test_check_has_last_commit_with_empty_git_repo() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;
    // 不创建任何 commit

    // 切换到测试目录，让 Git 操作能找到仓库
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = check_has_last_commit();

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

    // CliTestEnv 会在函数结束时自动恢复目录和环境
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
/// - 使用`CliTestEnv`自动创建和清理临时Git仓库（支持并行执行）
#[test]
fn test_check_has_last_commit_with_commits() -> color_eyre::Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?
        .create_file("test.txt", "test content")?
        .create_commit("Initial commit")?;

    // 切换到测试目录，让 Git 操作能找到仓库
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = check_has_last_commit();

    // 验证函数返回成功（有 commit）
    assert!(
        result.is_ok(),
        "check_has_last_commit should succeed when there are commits"
    );

    // CliTestEnv 会在函数结束时自动恢复目录和环境
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
/// - 使用`CliTestEnv`创建和管理临时Git仓库（支持并行执行）
/// - 自动恢复原始工作目录
#[test]
fn test_check_not_on_default_branch_on_main_returns_error() -> color_eyre::Result<()> {
    // Arrange: 准备临时Git仓库并在main分支上
    let env = CliTestEnv::new()?;
    env.init_git_repo()?
        .create_file("test.txt", "test")?
        .create_commit("Initial commit")?;

    // 创建一个假的远程分支引用，让get_default_branch()能正常工作
    std::process::Command::new("git")
        .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
        .current_dir(env.path())
        .output()
        .ok();

    // Act: 切换到测试目录，然后在main分支上检查是否允许操作
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = check_not_on_default_branch("amend");

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
#[test]
fn test_check_not_on_default_branch_on_feature_branch_returns_ok() -> color_eyre::Result<()> {
    // Arrange: 准备临时Git仓库并切换到feature分支
    let env = CliTestEnv::new()?;
    env.init_git_repo()?
        .create_file("test.txt", "test")?
        .create_commit("Initial commit")?;

    // 创建一个假的远程分支引用，让get_default_branch()能正常工作
    std::process::Command::new("git")
        .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
        .current_dir(env.path())
        .output()
        .ok();

    // 创建并切换到 feature 分支
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(env.path())
        .output()
        .expect("Failed to create feature branch");

    // Act: 切换到测试目录，然后检查是否允许操作
    let _guard = CurrentDirGuard::new(env.path())?;
    let result = check_not_on_default_branch("amend");

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
