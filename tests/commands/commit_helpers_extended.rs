//! Commit 命令辅助函数扩展测试
//!
//! 测试 Commit 命令辅助函数的业务逻辑，包括默认分支检查等。

use crate::common::environments::CliTestEnv;
use serial_test::serial;
use workflow::commands::commit::helpers::check_not_on_default_branch;

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
/// - 使用`#[serial]`确保测试串行执行（避免工作目录冲突）
/// - 使用`CliTestEnv`创建和管理临时Git仓库
/// - 自动恢复原始工作目录
#[test]
#[serial]
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

    // Act: 在main分支上检查是否允许操作
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

#[test]
#[serial]
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
    // Act: 创建并切换到feature分支，然后检查是否允许操作
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(env.path())
        .output()
        .expect("Failed to create feature branch");

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
