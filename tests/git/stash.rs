//! Git Stash 管理测试
//!
//! 测试 Git stash 相关的操作功能，包括：
//! - 保存未提交的修改到 stash
//! - 恢复 stash 中的修改
//! - 列出 stash 条目
//! - 删除 stash

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;

use workflow::git::GitStash;

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
    let test_dir = create_temp_test_dir("git_stash_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== Stash Push 测试 ====================

#[test]
fn test_stash_push() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(
            &std::env::current_dir().unwrap(),
            "stash_test.txt",
            "content",
        );

        // 保存到 stash
        GitStash::stash_push(None).unwrap();

        // 验证工作区已清理
        let status = Command::new("git")
            .args(&["status", "--porcelain"])
            .output()
            .expect("Failed to run git status");
        let status_str = String::from_utf8(status.stdout).unwrap();
        assert!(!status_str.contains("stash_test.txt"));
    });
}

#[test]
fn test_stash_push_with_message() {
    with_test_repo(|_| {
        // 创建修改
        create_test_file(
            &std::env::current_dir().unwrap(),
            "stash_msg.txt",
            "content",
        );

        // 保存到 stash 并带消息
        GitStash::stash_push(Some("Test stash message")).unwrap();

        // 验证 stash 列表包含消息
        let list = GitStash::stash_list().unwrap();
        assert!(!list.is_empty());
        assert!(list[0].message.contains("Test stash message"));
    });
}

// ==================== Stash List 测试 ====================

#[test]
fn test_stash_list_empty() {
    with_test_repo(|_| {
        // 初始状态应该没有 stash
        let list = GitStash::stash_list().unwrap();
        assert!(list.is_empty());
    });
}

#[test]
fn test_stash_list_with_entries() {
    with_test_repo(|_| {
        // 创建多个 stash
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        GitStash::stash_push(Some("Stash 1")).unwrap();

        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");
        GitStash::stash_push(Some("Stash 2")).unwrap();

        // 列出 stash
        let list = GitStash::stash_list().unwrap();
        assert_eq!(list.len(), 2);
        // 最新的应该在第一个（stash@{0}）
        assert_eq!(list[0].index, 0);
        assert_eq!(list[1].index, 1);
    });
}

// ==================== Stash Apply 测试 ====================

#[test]
fn test_stash_apply() {
    with_test_repo(|_| {
        // 创建修改并保存到 stash
        create_test_file(
            &std::env::current_dir().unwrap(),
            "apply_test.txt",
            "original content",
        );
        GitStash::stash_push(Some("Apply test")).unwrap();

        // 应用 stash
        let result = GitStash::stash_apply(None).unwrap();
        assert!(result.applied, "Stash should be applied");
        assert!(!result.has_conflicts, "Should not have conflicts");

        // 验证文件已恢复
        let content = std::fs::read_to_string("apply_test.txt").unwrap();
        assert_eq!(content, "original content");
    });
}

#[test]
fn test_stash_apply_specific() {
    with_test_repo(|_| {
        // 创建多个 stash
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        GitStash::stash_push(Some("First")).unwrap();

        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");
        GitStash::stash_push(Some("Second")).unwrap();

        // 应用特定的 stash（第二个，即 stash@{1}）
        let result = GitStash::stash_apply(Some("stash@{1}")).unwrap();
        assert!(result.applied);
    });
}

// ==================== Stash Pop 测试 ====================

#[test]
fn test_stash_pop() {
    with_test_repo(|_| {
        // 创建修改并保存到 stash
        create_test_file(&std::env::current_dir().unwrap(), "pop_test.txt", "content");
        GitStash::stash_push(Some("Pop test")).unwrap();

        // Pop stash
        let result = GitStash::stash_pop(None).unwrap();
        assert!(result.restored, "Stash should be restored");

        // 验证 stash 已删除
        let list = GitStash::stash_list().unwrap();
        assert!(list.is_empty(), "Stash should be deleted after pop");
    });
}

#[test]
fn test_stash_pop_specific() {
    with_test_repo(|_| {
        // 创建多个 stash
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        GitStash::stash_push(Some("First")).unwrap();

        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");
        GitStash::stash_push(Some("Second")).unwrap();

        // Pop 特定的 stash
        let result = GitStash::stash_pop(Some("stash@{1}")).unwrap();
        assert!(result.restored);

        // 验证只剩下一个 stash
        let list = GitStash::stash_list().unwrap();
        assert_eq!(list.len(), 1);
    });
}

// ==================== Stash Drop 测试 ====================

#[test]
fn test_stash_drop() {
    with_test_repo(|_| {
        // 创建 stash
        create_test_file(
            &std::env::current_dir().unwrap(),
            "drop_test.txt",
            "content",
        );
        GitStash::stash_push(Some("Drop test")).unwrap();

        // 删除 stash
        GitStash::stash_drop(Some("stash@{0}")).unwrap();

        // 验证 stash 已删除
        let list = GitStash::stash_list().unwrap();
        assert!(list.is_empty());
    });
}

#[test]
fn test_stash_drop_latest() {
    with_test_repo(|_| {
        // 创建多个 stash
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        GitStash::stash_push(Some("First")).unwrap();

        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");
        GitStash::stash_push(Some("Second")).unwrap();

        // 删除最新的 stash（None 表示最新的）
        GitStash::stash_drop(None).unwrap();

        // 验证只剩下一个 stash
        let list = GitStash::stash_list().unwrap();
        assert_eq!(list.len(), 1);
    });
}

// ==================== 冲突检测测试 ====================

#[test]
fn test_has_unmerged_no_conflicts() {
    with_test_repo(|_| {
        // 没有冲突时
        let has_conflicts = GitStash::has_unmerged().unwrap();
        assert!(!has_conflicts, "Should not have conflicts");
    });
}

// ==================== 错误处理测试 ====================

#[test]
fn test_stash_pop_empty() {
    with_test_repo(|_| {
        // 尝试从空的 stash 中 pop 应该失败
        let result = GitStash::stash_pop(None);
        assert!(result.is_err(), "Should fail when stash is empty");
    });
}

#[test]
fn test_stash_drop_nonexistent() {
    with_test_repo(|_| {
        // 尝试删除不存在的 stash 应该失败
        let result = GitStash::stash_drop(Some("stash@{999}"));
        assert!(result.is_err(), "Should fail for nonexistent stash");
    });
}
