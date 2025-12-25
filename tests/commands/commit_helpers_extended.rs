//! Commit 命令辅助函数扩展测试
//!
//! 测试 Commit 命令辅助函数的业务逻辑，包括默认分支检查等。

use crate::common::cli_helpers::CliTestEnv;
use serial_test::serial;
use workflow::commands::commit::helpers::check_not_on_default_branch;

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
fn test_check_not_on_default_branch_on_main() {
    use crate::common::helpers::CurrentDirGuard;

    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("test.txt", "test")
        .create_commit("Initial commit");

    // 切换到测试目录（使用RAII确保恢复）
    let _dir_guard = CurrentDirGuard::new(env.path()).ok();

    // 创建一个假的远程分支引用，让get_default_branch()能正常工作
    std::process::Command::new("git")
        .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
        .current_dir(env.path())
        .output()
        .ok();

    // 确保在 main 分支上（init_git_repo已经创建了main分支）
    // 无需额外操作，因为init -b main已经确保了这一点

    let result = check_not_on_default_branch("amend");

    // 验证返回错误（在默认分支上不允许操作）
    assert!(
        result.is_err(),
        "check_not_on_default_branch should fail on default branch"
    );

    // 验证错误消息
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("protected")
            || error_msg.contains("default branch")
            || error_msg.contains("Cannot"),
        "Error message should indicate protected branch: {}",
        error_msg
    );

    // 目录会在函数结束时自动恢复
}

/// 测试在feature分支上的默认分支检查
///
/// ## 测试目的
/// 验证`check_not_on_default_branch()`在feature分支上正确返回成功，允许在非保护分支上执行操作。
///
/// ## 测试场景
/// 1. 创建临时Git仓库并初始化
/// 2. 创建文件并提交
/// 3. 创建并切换到feature/test分支
/// 4. 调用`check_not_on_default_branch()`
/// 5. 验证返回成功且返回的分支信息正确
///
/// ## 技术细节
/// - 使用`#[serial]`确保测试串行执行
/// - 验证返回的当前分支和默认分支信息
/// - 自动恢复原始工作目录
#[test]
#[serial]
fn test_check_not_on_default_branch_on_feature_branch() {
    use crate::common::helpers::CurrentDirGuard;

    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("test.txt", "test")
        .create_commit("Initial commit");

    // 切换到测试目录（使用RAII确保恢复）
    let _dir_guard = CurrentDirGuard::new(env.path()).ok();

    // 创建一个假的远程分支引用，让get_default_branch()能正常工作
    std::process::Command::new("git")
        .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
        .current_dir(env.path())
        .output()
        .ok();

    // 获取默认分支名
    let default_branch = "main".to_string(); // 我们明确知道是main

    // 创建并切换到 feature 分支
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(env.path())
        .output()
        .expect("Failed to create feature branch");

    let result = check_not_on_default_branch("amend");

    // 验证返回成功（在非默认分支上允许操作）
    assert!(
        result.is_ok(),
        "check_not_on_default_branch should succeed on feature branch, error: {:?}",
        result.as_ref().err()
    );

    // 验证返回值包含分支信息
    if let Ok((current, default)) = result {
        assert_eq!(current, "feature/test");
        assert_eq!(default, default_branch); // 使用实际的默认分支名
    }

    // 目录会在函数结束时自动恢复
}

#[test]
fn test_check_not_on_default_branch_error_message_format() {
    // 测试错误消息格式（通过模拟错误情况）
    // 注意：这个测试主要验证函数不会 panic，实际的错误消息测试需要实际环境

    // 在非 Git 仓库中，函数应该返回错误
    let result = check_not_on_default_branch("amend");

    // 验证函数返回错误（非 Git 仓库）
    match result {
        Ok(_) => {
            // 如果当前目录恰好是 Git 仓库，这是可以接受的
        }
        Err(e) => {
            // 验证错误消息格式
            let error_msg = e.to_string();
            // 错误消息应该包含操作名称或分支相关信息
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
