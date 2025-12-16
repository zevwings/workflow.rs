//! Git Cherry-pick 测试
//!
//! 测试 Git cherry-pick 相关的操作功能，包括：
//! - 应用提交
//! - 继续 Cherry-pick
//! - 中止 Cherry-pick
//! - 状态检查

use std::path::Path;
use std::process::Command;
use crate::common::helpers::{
    cleanup_temp_test_dir, create_temp_test_dir, create_test_file,
};

use workflow::git::{GitBranch, GitCherryPick};

// ==================== 测试辅助函数 ====================

/// 初始化临时 Git 仓库
fn init_test_repo(dir: &Path) {
    Command::new("git")
        .args(&["init"])
        .current_dir(dir)
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(dir)
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .expect("Failed to set git user email");
}

/// 创建初始提交
fn create_initial_commit(dir: &Path) {
    create_test_file(dir, "README.md", "# Test Repository\n");
    Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(dir)
        .output()
        .expect("Failed to add file");

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(dir)
        .output()
        .expect("Failed to create initial commit");
}

/// 切换到指定目录执行测试
fn with_test_repo<F>(test_fn: F)
where
    F: FnOnce(&Path),
{
    let test_dir = create_temp_test_dir("git_cherry_pick_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== Cherry-pick 测试 ====================

#[test]
fn test_cherry_pick() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "feature.txt", "feature content");
        Command::new("git")
            .args(&["add", "feature.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Add feature"])
            .output()
            .expect("Failed to commit");

        // 获取提交 SHA
        let commit_sha = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .expect("Failed to get commit SHA");
        let commit_sha = String::from_utf8(commit_sha.stdout).unwrap().trim().to_string();

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();

        // Cherry-pick 提交
        GitCherryPick::cherry_pick(&commit_sha).unwrap();

        // 验证文件已应用
        assert!(Path::new("feature.txt").exists());
    });
}

#[test]
fn test_cherry_pick_no_commit() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "feature.txt", "feature content");
        Command::new("git")
            .args(&["add", "feature.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Add feature"])
            .output()
            .expect("Failed to commit");

        // 获取提交 SHA
        let commit_sha = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .expect("Failed to get commit SHA");
        let commit_sha = String::from_utf8(commit_sha.stdout).unwrap().trim().to_string();

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();

        // Cherry-pick 但不提交
        GitCherryPick::cherry_pick_no_commit(&commit_sha).unwrap();

        // 验证文件已应用但未提交
        assert!(Path::new("feature.txt").exists());
        // 应该在工作区中（未暂存）
        let status = Command::new("git")
            .args(&["status", "--porcelain"])
            .output()
            .expect("Failed to get status");
        let status_str = String::from_utf8(status.stdout).unwrap();
        assert!(status_str.contains("feature.txt"));
    });
}

// ==================== Cherry-pick 状态检查测试 ====================

#[test]
fn test_is_cherry_pick_in_progress_no() {
    with_test_repo(|_| {
        // 没有进行 cherry-pick 时应该返回 false
        assert!(!GitCherryPick::is_cherry_pick_in_progress());
    });
}

// ==================== Cherry-pick 中止测试 ====================

#[test]
fn test_cherry_pick_abort_no_operation() {
    with_test_repo(|_| {
        // 没有进行 cherry-pick 时中止应该失败
        let result = GitCherryPick::cherry_pick_abort();
        assert!(result.is_err(), "Should fail when no cherry-pick in progress");
    });
}
