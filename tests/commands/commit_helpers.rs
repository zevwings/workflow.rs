//! Commit 命令辅助函数测试
//!
//! 测试 Commit 命令的辅助函数，包括分支检查、force push 处理等。

use crate::common::cli_helpers::CliTestEnv;
use serial_test::serial;
use workflow::commands::commit::helpers::check_has_last_commit;

#[test]
fn test_check_has_last_commit_without_git_repo() {
    // 测试非 Git 仓库的情况
    // 注意：check_has_last_commit() 使用当前工作目录的 Git 仓库
    // 在测试环境中，如果没有 Git 仓库，应该返回错误
    let result = check_has_last_commit();

    // 验证函数返回错误（非 Git 仓库或无 commit）
    // 这个测试可能在不同环境下表现不同，主要验证函数不会 panic
    match result {
        Ok(_) => {
            // 如果当前目录恰好是 Git 仓库且有 commit，这是可以接受的
        }
        Err(e) => {
            // 验证错误消息包含相关信息
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
/// - 使用`#[serial]`确保测试串行执行（避免目录切换冲突）
/// - 使用`TempDir`自动清理临时目录
/// - 自动恢复原始工作目录
#[test]
#[serial]
fn test_check_has_last_commit_with_empty_git_repo() {
    use crate::common::helpers::CurrentDirGuard;

    let env = CliTestEnv::new();
    env.init_git_repo();
    // 不创建任何 commit

    // 切换到测试目录（使用RAII确保恢复）
    let _dir_guard = CurrentDirGuard::new(env.path()).ok();

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

    // 目录会在函数结束时自动恢复
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
/// - 使用`#[serial]`确保测试串行执行
/// - 自动创建和清理临时Git仓库
#[test]
#[serial]
fn test_check_has_last_commit_with_commits() {
    use crate::common::helpers::CurrentDirGuard;

    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("test.txt", "test content")
        .create_commit("Initial commit");

    // 切换到测试目录（使用RAII确保恢复）
    let _dir_guard = CurrentDirGuard::new(env.path()).ok();

    let result = check_has_last_commit();

    // 验证函数返回成功（有 commit）
    assert!(
        result.is_ok(),
        "check_has_last_commit should succeed when there are commits"
    );

    // 目录会在函数结束时自动恢复
}
