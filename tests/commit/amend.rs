//! Commit Amend 测试
//!
//! 测试 Commit Amend 相关的业务逻辑，包括：
//! - 预览信息生成
//! - 格式化显示
//! - 完成提示生成

use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;
use crate::common::helpers::{
    cleanup_temp_test_dir, create_temp_test_dir, create_test_file,
};

use workflow::commit::CommitAmend;
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
    let test_dir = create_temp_test_dir("commit_amend_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 预览信息创建测试 ====================

#[test]
fn test_create_preview_with_new_message() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息（带新消息）
        let preview = CommitAmend::create_preview(
            &commit_info,
            &Some("New commit message".to_string()),
            &["file1.txt".to_string()],
            "amend",
            &current_branch,
        )
        .unwrap();

        assert_eq!(preview.original_sha, commit_info.sha);
        assert_eq!(preview.original_message, commit_info.message);
        assert_eq!(preview.new_message, Some("New commit message".to_string()));
        assert_eq!(preview.files_to_add, vec!["file1.txt".to_string()]);
        assert_eq!(preview.operation_type, "amend");
    });
}

#[test]
fn test_create_preview_without_new_message() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息（不带新消息）
        let preview = CommitAmend::create_preview(
            &commit_info,
            &None,
            &[],
            "amend",
            &current_branch,
        )
        .unwrap();

        assert_eq!(preview.original_sha, commit_info.sha);
        assert_eq!(preview.new_message, None);
        assert!(preview.files_to_add.is_empty());
    });
}

// ==================== 格式化预览测试 ====================

#[test]
fn test_format_preview_with_new_message() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息
        let preview = CommitAmend::create_preview(
            &commit_info,
            &Some("New message".to_string()),
            &["file1.txt".to_string(), "file2.txt".to_string()],
            "amend",
            &current_branch,
        )
        .unwrap();

        // 格式化预览
        let formatted = CommitAmend::format_preview(&preview);

        // 验证包含关键信息
        assert!(formatted.contains("Commit Amend Preview"));
        assert!(formatted.contains(&commit_info.sha[..8]));
        assert!(formatted.contains("New message"));
        assert!(formatted.contains("file1.txt"));
        assert!(formatted.contains("file2.txt"));
    });
}

#[test]
fn test_format_preview_without_new_message() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息（不带新消息）
        let preview = CommitAmend::create_preview(
            &commit_info,
            &None,
            &[],
            "amend",
            &current_branch,
        )
        .unwrap();

        // 格式化预览
        let formatted = CommitAmend::format_preview(&preview);

        // 验证包含关键信息
        assert!(formatted.contains("Commit Amend Preview"));
        assert!(formatted.contains("(unchanged)"));
        assert!(formatted.contains("Files to add:         None"));
    });
}

#[test]
fn test_format_preview_with_pushed_warning() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 创建预览信息（模拟已推送的情况）
        let mut preview = CommitAmend::create_preview(
            &commit_info,
            &Some("New message".to_string()),
            &[],
            "amend",
            &current_branch,
        )
        .unwrap();
        preview.is_pushed = true;

        // 格式化预览
        let formatted = CommitAmend::format_preview(&preview);

        // 验证包含警告信息
        assert!(formatted.contains("Warning"));
        assert!(formatted.contains("force push"));
    });
}

// ==================== 格式化提交信息测试 ====================

#[test]
fn test_format_commit_info_detailed() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 格式化详细提交信息
        let formatted = CommitAmend::format_commit_info_detailed(
            &commit_info,
            &current_branch,
            None,
        );

        // 验证包含关键信息
        assert!(formatted.contains("当前 Commit 信息"));
        assert!(formatted.contains(&commit_info.sha[..8]));
        assert!(formatted.contains(&commit_info.message));
        assert!(formatted.contains(&current_branch));
    });
}

#[test]
fn test_format_commit_info_detailed_with_status() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 获取工作区状态
        let status = GitCommit::get_worktree_status().unwrap();

        // 格式化详细提交信息（带状态）
        let formatted = CommitAmend::format_commit_info_detailed(
            &commit_info,
            &current_branch,
            Some(&status),
        );

        // 验证包含状态信息
        assert!(formatted.contains("当前 Commit 信息"));
    });
}

// ==================== Force Push 警告测试 ====================

#[test]
fn test_should_show_force_push_warning_not_pushed() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 检查是否需要显示 force push 警告（未推送）
        let should_show = CommitAmend::should_show_force_push_warning(
            &current_branch,
            &commit_info.sha,
        )
        .unwrap();

        // 未推送时应该返回 false
        assert!(!should_show);
    });
}

// ==================== 完成消息测试 ====================

#[test]
fn test_format_completion_message_not_pushed() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let commit_info = GitCommit::get_last_commit_info().unwrap();
        let current_branch = GitBranch::current_branch().unwrap();

        // 格式化完成消息（未推送）
        let message = CommitAmend::format_completion_message(
            &current_branch,
            &commit_info.sha,
        )
        .unwrap();

        // 未推送时应该返回 None
        assert!(message.is_none());
    });
}
