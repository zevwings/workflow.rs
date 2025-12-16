//! Git Tag 管理测试
//!
//! 测试 Git tag 相关的操作功能，包括：
//! - 列出所有 tag
//! - 删除本地和远程 tag
//! - 检查 tag 是否存在
//! - 获取 tag 信息

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;

use workflow::git::GitTag;

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
    let test_dir = create_temp_test_dir("git_tag_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 列出 Tag 测试 ====================

#[test]
fn test_list_local_tags_empty() {
    with_test_repo(|_| {
        // 初始状态应该没有 tag
        let tags = GitTag::list_local_tags().unwrap();
        assert!(tags.is_empty());
    });
}

#[test]
fn test_list_local_tags() {
    with_test_repo(|_| {
        // 创建几个 tag
        Command::new("git")
            .args(&["tag", "v1.0.0"])
            .output()
            .expect("Failed to create tag");

        Command::new("git")
            .args(&["tag", "v1.1.0"])
            .output()
            .expect("Failed to create tag");

        // 列出本地 tag
        let tags = GitTag::list_local_tags().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"v1.0.0".to_string()));
        assert!(tags.contains(&"v1.1.0".to_string()));
    });
}

#[test]
fn test_list_local_tags_sorted() {
    with_test_repo(|_| {
        // 创建 tag（无序）
        Command::new("git")
            .args(&["tag", "v1.2.0"])
            .output()
            .expect("Failed to create tag");
        Command::new("git")
            .args(&["tag", "v1.0.0"])
            .output()
            .expect("Failed to create tag");
        Command::new("git")
            .args(&["tag", "v1.1.0"])
            .output()
            .expect("Failed to create tag");

        // 列出 tag（应该已排序）
        let tags = GitTag::list_local_tags().unwrap();
        assert_eq!(tags.len(), 3);
        // 验证已排序
        assert_eq!(tags[0], "v1.0.0");
        assert_eq!(tags[1], "v1.1.0");
        assert_eq!(tags[2], "v1.2.0");
    });
}

// ==================== Tag 存在性检查测试 ====================

#[test]
fn test_is_tag_exists_local() {
    with_test_repo(|_| {
        // 创建 tag
        Command::new("git")
            .args(&["tag", "test-tag"])
            .output()
            .expect("Failed to create tag");

        // 检查 tag 是否存在
        let (exists_local, exists_remote) = GitTag::is_tag_exists("test-tag").unwrap();
        assert!(exists_local, "Local tag should exist");
        assert!(!exists_remote, "Remote tag should not exist");
    });
}

#[test]
fn test_is_tag_exists_nonexistent() {
    with_test_repo(|_| {
        // 检查不存在的 tag
        let (exists_local, exists_remote) = GitTag::is_tag_exists("nonexistent-tag").unwrap();
        assert!(!exists_local, "Local tag should not exist");
        assert!(!exists_remote, "Remote tag should not exist");
    });
}

// ==================== 获取 Tag 信息测试 ====================

#[test]
fn test_get_tag_info() {
    with_test_repo(|_| {
        // 创建 tag
        Command::new("git")
            .args(&["tag", "info-tag"])
            .output()
            .expect("Failed to create tag");

        // 获取 tag 信息
        let info = GitTag::get_tag_info("info-tag").unwrap();
        assert_eq!(info.name, "info-tag");
        assert!(!info.commit_hash.is_empty());
        assert!(info.exists_local);
        assert!(!info.exists_remote);
    });
}

#[test]
fn test_get_tag_info_nonexistent() {
    with_test_repo(|_| {
        // 尝试获取不存在的 tag 信息应该失败
        let result = GitTag::get_tag_info("nonexistent-tag");
        assert!(result.is_err(), "Should fail for nonexistent tag");
    });
}

// ==================== 删除 Tag 测试 ====================

#[test]
fn test_delete_local_tag() {
    with_test_repo(|_| {
        // 创建 tag
        Command::new("git")
            .args(&["tag", "to-delete"])
            .output()
            .expect("Failed to create tag");

        // 验证 tag 存在
        let (exists_local, _) = GitTag::is_tag_exists("to-delete").unwrap();
        assert!(exists_local);

        // 删除本地 tag
        GitTag::delete_local("to-delete").unwrap();

        // 验证 tag 已删除
        let (exists_local, _) = GitTag::is_tag_exists("to-delete").unwrap();
        assert!(!exists_local);
    });
}

#[test]
fn test_delete_local_tag_nonexistent() {
    with_test_repo(|_| {
        // 尝试删除不存在的 tag 应该失败
        let result = GitTag::delete_local("nonexistent-tag");
        assert!(result.is_err(), "Should fail for nonexistent tag");
    });
}

// ==================== 列出所有 Tag 测试 ====================

#[test]
fn test_list_all_tags() {
    with_test_repo(|_| {
        // 创建几个 tag
        Command::new("git")
            .args(&["tag", "tag1"])
            .output()
            .expect("Failed to create tag");
        Command::new("git")
            .args(&["tag", "tag2"])
            .output()
            .expect("Failed to create tag");

        // 列出所有 tag
        let tags = GitTag::list_all_tags().unwrap();
        assert!(tags.len() >= 2);
        assert!(tags.iter().any(|t| t.name == "tag1"));
        assert!(tags.iter().any(|t| t.name == "tag2"));
    });
}

#[test]
fn test_list_all_tags_empty() {
    with_test_repo(|_| {
        // 初始状态应该没有 tag
        let tags = GitTag::list_all_tags().unwrap();
        assert!(tags.is_empty());
    });
}

// ==================== Tag 信息验证测试 ====================

#[test]
fn test_tag_info_commit_hash() {
    with_test_repo(|_| {
        // 创建 tag
        Command::new("git")
            .args(&["tag", "hash-test"])
            .output()
            .expect("Failed to create tag");

        // 获取 tag 信息
        let info = GitTag::get_tag_info("hash-test").unwrap();
        // commit hash 应该是 40 个字符（SHA-1）
        assert_eq!(info.commit_hash.len(), 40);
    });
}
