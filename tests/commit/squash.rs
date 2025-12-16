//! Commit Squash 测试
//!
//! 测试 Commit Squash 相关的业务逻辑，包括：
//! - 获取分支提交
//! - 预览信息生成
//! - 格式化显示

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;

use workflow::commit::CommitSquash;
use workflow::git::GitBranch;

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
    let test_dir = create_temp_test_dir("commit_squash_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 获取分支提交测试 ====================

#[test]
fn test_get_branch_commits() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建多个提交
        for i in 1..=3 {
            create_test_file(
                &std::env::current_dir().unwrap(),
                &format!("file{}.txt", i),
                &format!("content {}", i),
            );
            Command::new("git")
                .args(&["add", &format!("file{}.txt", i)])
                .output()
                .expect("Failed to add file");
            Command::new("git")
                .args(&["commit", "-m", &format!("Commit {}", i)])
                .output()
                .expect("Failed to commit");
        }

        // 获取分支提交
        let commits = CommitSquash::get_branch_commits("feature").unwrap();

        // 应该包含 3 个提交
        assert_eq!(commits.len(), 3);
    });
}

#[test]
fn test_get_branch_commits_empty() {
    with_test_repo(|_| {
        // 创建新分支但不创建提交
        GitBranch::checkout_branch("empty-branch").unwrap();

        // 获取分支提交（应该为空）
        let commits = CommitSquash::get_branch_commits("empty-branch").unwrap();
        assert!(commits.is_empty());
    });
}

// ==================== 预览信息创建测试 ====================

#[test]
fn test_create_preview() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        Command::new("git")
            .args(&["add", "file1.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "First commit"])
            .output()
            .expect("Failed to commit");

        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");
        Command::new("git")
            .args(&["add", "file2.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Second commit"])
            .output()
            .expect("Failed to commit");

        // 获取分支提交
        let commits = CommitSquash::get_branch_commits("feature").unwrap();

        // 创建预览信息
        let preview =
            CommitSquash::create_preview(&commits, "Squashed commit message", "feature").unwrap();

        assert_eq!(preview.commits.len(), 2);
        assert_eq!(preview.new_message, "Squashed commit message");
        assert!(!preview.base_sha.is_empty());
    });
}

// ==================== 格式化预览测试 ====================

#[test]
fn test_format_preview() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        Command::new("git")
            .args(&["add", "file1.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "First commit"])
            .output()
            .expect("Failed to commit");

        // 获取分支提交
        let commits = CommitSquash::get_branch_commits("feature").unwrap();

        // 创建预览信息
        let preview =
            CommitSquash::create_preview(&commits, "Squashed message", "feature").unwrap();

        // 格式化预览
        let formatted = CommitSquash::format_preview(&preview);

        // 验证包含关键信息
        assert!(formatted.contains("Commit Squash Preview"));
        assert!(formatted.contains("Squashed message"));
        assert!(formatted.contains("First commit"));
    });
}

#[test]
fn test_format_preview_with_pushed_warning() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        Command::new("git")
            .args(&["add", "file1.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "First commit"])
            .output()
            .expect("Failed to commit");

        // 获取分支提交
        let commits = CommitSquash::get_branch_commits("feature").unwrap();

        // 创建预览信息（模拟已推送的情况）
        let mut preview =
            CommitSquash::create_preview(&commits, "Squashed message", "feature").unwrap();
        preview.is_pushed = true;

        // 格式化预览
        let formatted = CommitSquash::format_preview(&preview);

        // 验证包含警告信息
        assert!(formatted.contains("Warning"));
        assert!(formatted.contains("force push"));
    });
}
