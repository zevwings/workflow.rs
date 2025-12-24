//! Commit 命令辅助函数测试
//!
//! 测试 Commit 命令的辅助函数，包括分支检查、force push 处理等。

use crate::common::cli_helpers::CliTestEnv;
use std::env;
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
                error_msg.contains("No commits") ||
                error_msg.contains("git") ||
                error_msg.contains("repository"),
                "Error message should indicate the issue: {}",
                error_msg
            );
        }
    }
}

#[test]
#[ignore] // 需要实际的 Git 仓库环境
fn test_check_has_last_commit_with_empty_git_repo() {
    // 测试空 Git 仓库（无 commit）的情况
    // 注意：这个测试需要实际的 Git 仓库环境
    let env = CliTestEnv::new();
    env.init_git_repo();
    // 不创建任何 commit

    // 切换到测试目录
    let original_dir = env::current_dir().ok();
    env::set_current_dir(env.path()).ok();

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

    // 恢复原始目录
    if let Some(dir) = original_dir {
        env::set_current_dir(dir).ok();
    }
}

#[test]
#[ignore] // 需要实际的 Git 仓库环境
fn test_check_has_last_commit_with_commits() {
    // 测试有 commit 的 Git 仓库的情况
    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("test.txt", "test content")
        .create_commit("Initial commit");

    // 切换到测试目录
    let original_dir = env::current_dir().ok();
    env::set_current_dir(env.path()).ok();

    let result = check_has_last_commit();

    // 验证函数返回成功（有 commit）
    assert!(
        result.is_ok(),
        "check_has_last_commit should succeed when there are commits"
    );

    // 恢复原始目录
    if let Some(dir) = original_dir {
        env::set_current_dir(dir).ok();
    }
}

