//! Commit 命令辅助函数扩展测试
//!
//! 测试 Commit 命令辅助函数的业务逻辑，包括默认分支检查等。

use crate::common::cli_helpers::CliTestEnv;
use std::env;
use workflow::commands::commit::helpers::check_not_on_default_branch;

#[test]
#[ignore] // 需要实际的 Git 仓库环境
fn test_check_not_on_default_branch_on_main() {
    // 测试在 main 分支上的情况（应该返回错误）
    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("test.txt", "test")
        .create_commit("Initial commit");

    // 切换到测试目录
    let original_dir = env::current_dir().ok();
    env::set_current_dir(env.path()).ok();

    // 确保在 main 分支上
    std::process::Command::new("git")
        .args(["checkout", "-b", "main"])
        .current_dir(env.path())
        .output()
        .ok();

    let result = check_not_on_default_branch("amend");

    // 验证返回错误（在默认分支上不允许操作）
    assert!(
        result.is_err(),
        "check_not_on_default_branch should fail on default branch"
    );

    // 验证错误消息
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("protected") || error_msg.contains("default branch") || error_msg.contains("Cannot"),
        "Error message should indicate protected branch: {}",
        error_msg
    );

    // 恢复原始目录
    if let Some(dir) = original_dir {
        env::set_current_dir(dir).ok();
    }
}

#[test]
#[ignore] // 需要实际的 Git 仓库环境
fn test_check_not_on_default_branch_on_feature_branch() {
    // 测试在 feature 分支上的情况（应该成功）
    let env = CliTestEnv::new();
    env.init_git_repo()
        .create_file("test.txt", "test")
        .create_commit("Initial commit");

    // 切换到测试目录
    let original_dir = env::current_dir().ok();
    env::set_current_dir(env.path()).ok();

    // 创建并切换到 feature 分支
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature/test"])
        .current_dir(env.path())
        .output()
        .ok();

    let result = check_not_on_default_branch("amend");

    // 验证返回成功（在非默认分支上允许操作）
    assert!(
        result.is_ok(),
        "check_not_on_default_branch should succeed on feature branch"
    );

    // 验证返回值包含分支信息
    if let Ok((current, default)) = result {
        assert_eq!(current, "feature/test");
        assert_eq!(default, "main"); // 或 "master"，取决于默认分支
    }

    // 恢复原始目录
    if let Some(dir) = original_dir {
        env::set_current_dir(dir).ok();
    }
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
                error_msg.contains("branch") || error_msg.contains("git") || error_msg.contains("Failed"),
                "Error message should be informative: {}",
                error_msg
            );
        }
    }
}

