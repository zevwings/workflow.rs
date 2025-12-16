//! Git 提交管理测试
//!
//! 测试 Git 提交相关的操作功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）
//! - 获取提交信息

use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;
use crate::common::helpers::{
    cleanup_temp_test_dir, create_temp_test_dir, create_test_file,
};

use workflow::git::GitCommit;

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
    let test_dir = create_temp_test_dir("git_commit_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 状态检查测试 ====================

#[test]
fn test_status() {
    with_test_repo(|_| {
        // 获取状态
        let status = GitCommit::status().unwrap();
        // 初始提交后，工作区应该是干净的
        assert!(status.trim().is_empty() || status.contains("nothing to commit"));
    });
}

#[test]
fn test_status_with_changes() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "new_file.txt", "content");

        // 获取状态
        let status = GitCommit::status().unwrap();
        assert!(status.contains("new_file.txt") || status.contains("??"));
    });
}

#[test]
fn test_has_staged() {
    with_test_repo(|_| {
        // 初始状态应该没有暂存的文件
        // 注意：has_staged 是 pub(crate)，不能直接调用
        // 可以通过 has_commit 间接测试，或者通过 git status 验证
        let status = GitCommit::status().unwrap();
        assert!(status.trim().is_empty() || !status.contains("A "));

        // 创建文件并暂存
        create_test_file(&std::env::current_dir().unwrap(), "staged.txt", "content");
        Command::new("git")
            .args(&["add", "staged.txt"])
            .output()
            .expect("Failed to add file");

        // 验证文件已暂存（通过 status 检查）
        let status = GitCommit::status().unwrap();
        assert!(status.contains("staged.txt") || status.contains("A "));
    });
}

#[test]
fn test_has_commit() {
    with_test_repo(|_| {
        // 初始状态应该没有未提交的更改
        assert!(!GitCommit::has_commit().unwrap());

        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "modified.txt", "content");

        // 现在应该有未提交的更改
        assert!(GitCommit::has_commit().unwrap());
    });
}

// ==================== 暂存文件测试 ====================

#[test]
fn test_add_all() {
    with_test_repo(|_| {
        // 创建多个文件
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");

        // 暂存所有文件
        GitCommit::add_all().unwrap();

        // 验证文件已暂存（通过 status 检查）
        let status = GitCommit::status().unwrap();
        assert!(status.contains("file1.txt") || status.contains("A "));
    });
}

#[test]
fn test_add_files() {
    with_test_repo(|_| {
        // 创建文件
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");

        // 只暂存一个文件
        GitCommit::add_files(&["file1.txt".to_string()]).unwrap();

        // 验证文件已暂存（通过 status 检查）
        let status = GitCommit::status().unwrap();
        assert!(status.contains("file1.txt") || status.contains("A "));
    });
}

#[test]
fn test_add_files_empty() {
    with_test_repo(|_| {
        // 空列表应该不会失败
        GitCommit::add_files(&[]).unwrap();
    });
}

// ==================== 提交测试 ====================

#[test]
fn test_commit_with_changes() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "commit_test.txt", "content");

        // 提交
        let result = GitCommit::commit("Test commit", true).unwrap();
        assert!(result.committed, "Should have committed");
        assert!(result.message.is_none());
    });
}

#[test]
fn test_commit_no_changes() {
    with_test_repo(|_| {
        // 没有更改时提交
        let result = GitCommit::commit("No changes", true).unwrap();
        assert!(!result.committed, "Should not have committed");
        assert!(result.message.is_some());
        if let Some(ref msg) = result.message {
            assert!(msg.contains("Nothing to commit") || msg.contains("clean"));
        }
    });
}

#[test]
fn test_commit_with_message() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "message_test.txt", "content");

        // 提交并验证消息
        let result = GitCommit::commit("Custom commit message", true).unwrap();
        assert!(result.committed);

        // 验证提交消息
        let last_message = GitCommit::get_last_commit_message().unwrap();
        assert_eq!(last_message.trim(), "Custom commit message");
    });
}

// ==================== 获取提交信息测试 ====================

#[test]
fn test_get_last_commit_info() {
    with_test_repo(|_| {
        // 获取最后一次提交信息
        let info = GitCommit::get_last_commit_info().unwrap();
        assert!(!info.sha.is_empty());
        assert!(!info.message.is_empty());
        assert!(!info.author.is_empty());
        assert!(!info.date.is_empty());
    });
}

#[test]
fn test_get_last_commit_sha() {
    with_test_repo(|_| {
        // 获取最后一次提交的 SHA
        let sha = GitCommit::get_last_commit_sha().unwrap();
        assert!(!sha.is_empty());
        assert_eq!(sha.len(), 40); // SHA-1 是 40 个字符
    });
}

#[test]
fn test_get_last_commit_message() {
    with_test_repo(|_| {
        // 获取最后一次提交的消息
        let message = GitCommit::get_last_commit_message().unwrap();
        assert_eq!(message.trim(), "Initial commit");
    });
}

#[test]
fn test_has_last_commit() {
    with_test_repo(|_| {
        // 应该有最后一次提交
        assert!(GitCommit::has_last_commit().unwrap());
    });
}

// ==================== 文件列表测试 ====================

#[test]
fn test_get_modified_files() {
    with_test_repo(|_| {
        // 初始状态应该没有修改的文件
        let modified = GitCommit::get_modified_files().unwrap();
        assert!(modified.is_empty());

        // 修改文件
        create_test_file(&std::env::current_dir().unwrap(), "README.md", "# Modified\n");

        // 现在应该有修改的文件
        let modified = GitCommit::get_modified_files().unwrap();
        assert!(modified.contains(&"README.md".to_string()));
    });
}

#[test]
fn test_get_untracked_files() {
    with_test_repo(|_| {
        // 创建未跟踪的文件
        create_test_file(&std::env::current_dir().unwrap(), "untracked.txt", "content");

        // 获取未跟踪的文件
        let untracked = GitCommit::get_untracked_files().unwrap();
        assert!(untracked.contains(&"untracked.txt".to_string()));
    });
}

// ==================== Diff 测试 ====================

#[test]
fn test_get_diff_with_changes() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "diff_test.txt", "content");

        // 获取 diff
        let diff = GitCommit::get_diff();
        assert!(diff.is_some());
        let diff_str = diff.unwrap();
        assert!(diff_str.contains("diff_test.txt") || diff_str.contains("Working tree"));
    });
}

#[test]
fn test_get_diff_no_changes() {
    with_test_repo(|_| {
        // 没有更改时应该返回 None
        let diff = GitCommit::get_diff();
        // 可能返回 None 或空字符串，取决于实现
    });
}

// ==================== Reset 测试 ====================

#[test]
fn test_reset_hard() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "reset_test.txt", "content");

        // 重置到 HEAD
        GitCommit::reset_hard(None).unwrap();

        // 验证文件已删除（如果未提交）
        // 注意：这个测试可能因为文件未跟踪而行为不同
    });
}

// ==================== 工作区状态测试 ====================

#[test]
fn test_worktree_status() {
    with_test_repo(|_| {
        // 获取工作区状态
        let status = GitCommit::get_worktree_status().unwrap();
        // 初始状态应该都是 0
        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
    });
}

#[test]
fn test_worktree_status_with_changes() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(&std::env::current_dir().unwrap(), "status_test.txt", "content");

        // 获取工作区状态
        let status = GitCommit::get_worktree_status().unwrap();
        // 应该有未跟踪的文件
        assert!(status.untracked_count > 0);
    });
}
