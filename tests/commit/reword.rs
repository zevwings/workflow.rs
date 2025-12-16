//! Commit Reword 测试
//!
//! 测试 Commit Reword 相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;

use workflow::commit::CommitReword;
use workflow::git::{CommitInfo, GitBranch, GitCommit};

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
    let test_dir = create_temp_test_dir("commit_reword_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 格式化提交信息测试 ====================

#[test]
fn test_format_commit_info() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 格式化提交信息
        let formatted = CommitReword::format_commit_info(&commit_info, &current_branch);

        // 验证包含关键信息
        assert!(formatted.contains("Current Commit Info"));
        assert!(formatted.contains(&commit_info.sha[..8]));
        assert!(formatted.contains(&commit_info.message));
        assert!(formatted.contains(&current_branch));
    });
}

// ==================== 预览信息创建测试 ====================

#[test]
fn test_create_preview() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息
        let preview = CommitReword::create_preview(
            &commit_info,
            "New commit message",
            true, // is_head
            &current_branch,
        )
        .unwrap();

        assert_eq!(preview.original_sha, commit_info.sha);
        assert_eq!(preview.original_message, commit_info.message);
        assert_eq!(preview.new_message, "New commit message");
        assert!(preview.is_head);
    });
}

#[test]
fn test_create_preview_not_head() {
    with_test_repo(|_| {
        // 创建第二个提交
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content");
        Command::new("git")
            .args(&["add", "file1.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Second commit"])
            .output()
            .expect("Failed to commit");

        // 获取第一个提交信息（不是 HEAD）
        let first_commit_sha = Command::new("git")
            .args(&["rev-parse", "HEAD~1"])
            .output()
            .expect("Failed to get commit SHA");
        let first_commit_sha =
            String::from_utf8(first_commit_sha.stdout).unwrap().trim().to_string();

        let commit_info = Command::new("git")
            .args(&[
                "log",
                "-1",
                "--format=%H|%s|%an <%ae>|%ai",
                &first_commit_sha,
            ])
            .output()
            .expect("Failed to get commit info");
        let info_str = String::from_utf8(commit_info.stdout).unwrap();
        let parts: Vec<&str> = info_str.trim().split('|').collect();

        let commit_info = CommitInfo {
            sha: parts[0].to_string(),
            message: parts[1].to_string(),
            author: parts[2].to_string(),
            date: parts[3].to_string(),
        };

        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息（不是 HEAD）
        let preview = CommitReword::create_preview(
            &commit_info,
            "New message",
            false, // is_head
            &current_branch,
        )
        .unwrap();

        assert!(!preview.is_head);
    });
}

// ==================== 格式化预览测试 ====================

#[test]
fn test_format_preview() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息
        let preview =
            CommitReword::create_preview(&commit_info, "New commit message", true, &current_branch)
                .unwrap();

        // 格式化预览
        let formatted = CommitReword::format_preview(&preview);

        // 验证包含关键信息
        assert!(formatted.contains("Commit Reword Preview"));
        assert!(formatted.contains(&commit_info.sha[..8]));
        assert!(formatted.contains("New commit message"));
    });
}

#[test]
fn test_format_preview_with_pushed_warning() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息（模拟已推送的情况）
        let mut preview =
            CommitReword::create_preview(&commit_info, "New message", true, &current_branch)
                .unwrap();
        preview.is_pushed = true;

        // 格式化预览
        let formatted = CommitReword::format_preview(&preview);

        // 验证包含警告信息
        assert!(formatted.contains("Warning"));
        assert!(formatted.contains("force push"));
    });
}
