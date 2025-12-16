//! Git 仓库检测测试
//!
//! 测试 Git 仓库相关的检测功能，包括：
//! - 检测当前目录是否为 Git 仓库
//! - 检测远程仓库类型（GitHub、Codeup 等）
//! - 获取远程仓库 URL

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;

use workflow::git::{GitRepo, RepoType};

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

/// 添加远程仓库
fn add_remote(dir: &Path, name: &str, url: &str) {
    Command::new("git")
        .args(&["remote", "add", name, url])
        .current_dir(dir)
        .output()
        .expect("Failed to add remote");
}

/// 切换到指定目录执行测试
fn with_test_repo<F>(test_fn: F)
where
    F: FnOnce(&Path),
{
    let test_dir = create_temp_test_dir("git_repo_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== Git 仓库检测测试 ====================

#[test]
fn test_is_git_repo() {
    with_test_repo(|_| {
        // 在 Git 仓库中应该返回 true
        assert!(GitRepo::is_git_repo());
    });
}

#[test]
fn test_is_git_repo_outside() {
    // 在非 Git 仓库目录中应该返回 false
    let test_dir = create_temp_test_dir("not_git_repo");
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    assert!(!GitRepo::is_git_repo());

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 远程仓库类型检测测试 ====================

#[test]
fn test_detect_repo_type_github_https() {
    with_test_repo(|dir| {
        // 添加 GitHub HTTPS URL
        add_remote(dir, "origin", "https://github.com/user/repo.git");

        // 检测仓库类型
        let repo_type = GitRepo::detect_repo_type().unwrap();
        assert_eq!(repo_type, RepoType::GitHub);
    });
}

#[test]
fn test_detect_repo_type_github_ssh() {
    with_test_repo(|dir| {
        // 添加 GitHub SSH URL
        add_remote(dir, "origin", "git@github.com:user/repo.git");

        // 检测仓库类型
        let repo_type = GitRepo::detect_repo_type().unwrap();
        assert_eq!(repo_type, RepoType::GitHub);
    });
}

#[test]
fn test_detect_repo_type_github_ssh_alias() {
    with_test_repo(|dir| {
        // 添加 GitHub SSH Host 别名 URL
        add_remote(dir, "origin", "git@github-brainim:user/repo.git");

        // 检测仓库类型
        let repo_type = GitRepo::detect_repo_type().unwrap();
        assert_eq!(repo_type, RepoType::GitHub);
    });
}

#[test]
fn test_detect_repo_type_codeup() {
    with_test_repo(|dir| {
        // 添加 Codeup URL
        add_remote(dir, "origin", "https://codeup.aliyun.com/user/repo.git");

        // 检测仓库类型
        let repo_type = GitRepo::detect_repo_type().unwrap();
        assert_eq!(repo_type, RepoType::Codeup);
    });
}

#[test]
fn test_detect_repo_type_unknown() {
    with_test_repo(|dir| {
        // 添加未知类型的 URL
        add_remote(dir, "origin", "https://example.com/user/repo.git");

        // 检测仓库类型
        let repo_type = GitRepo::detect_repo_type().unwrap();
        assert_eq!(repo_type, RepoType::Unknown);
    });
}

#[test]
fn test_detect_repo_type_no_remote() {
    with_test_repo(|_| {
        // 没有远程仓库时应该返回错误
        let result = GitRepo::detect_repo_type();
        assert!(result.is_err(), "Should fail when no remote is configured");
    });
}

// ==================== 获取远程 URL 测试 ====================

#[test]
fn test_get_remote_url() {
    with_test_repo(|dir| {
        // 添加远程仓库
        let test_url = "https://github.com/user/repo.git";
        add_remote(dir, "origin", test_url);

        // 获取远程 URL
        let url = GitRepo::get_remote_url().unwrap();
        assert_eq!(url.trim(), test_url);
    });
}

#[test]
fn test_get_remote_url_no_remote() {
    with_test_repo(|_| {
        // 没有远程仓库时应该返回错误
        let result = GitRepo::get_remote_url();
        assert!(result.is_err(), "Should fail when no remote is configured");
    });
}

// ==================== 提取仓库名测试 ====================

#[test]
fn test_extract_repo_name_from_url_github_ssh() {
    // 测试从 GitHub SSH URL 提取仓库名
    let url = "git@github.com:user/repo.git";
    let repo_name = GitRepo::extract_repo_name_from_url(url).unwrap();
    assert_eq!(repo_name, "user/repo");
}

#[test]
fn test_extract_repo_name_from_url_github_https() {
    // 测试从 GitHub HTTPS URL 提取仓库名
    let url = "https://github.com/user/repo.git";
    let repo_name = GitRepo::extract_repo_name_from_url(url).unwrap();
    assert_eq!(repo_name, "user/repo");
}

#[test]
fn test_extract_repo_name_from_url_codeup_ssh() {
    // 测试从 Codeup SSH URL 提取仓库名
    let url = "git@codeup.aliyun.com:user/repo.git";
    let repo_name = GitRepo::extract_repo_name_from_url(url).unwrap();
    assert_eq!(repo_name, "user/repo");
}

#[test]
fn test_extract_repo_name_from_url_codeup_https() {
    // 测试从 Codeup HTTPS URL 提取仓库名
    let url = "https://codeup.aliyun.com/user/repo.git";
    let repo_name = GitRepo::extract_repo_name_from_url(url).unwrap();
    assert_eq!(repo_name, "user/repo");
}

#[test]
fn test_extract_repo_name() {
    with_test_repo(|dir| {
        // 添加远程仓库
        add_remote(dir, "origin", "https://github.com/testuser/testrepo.git");

        // 提取仓库名
        let repo_name = GitRepo::extract_repo_name().unwrap();
        assert_eq!(repo_name, "testuser/testrepo");
    });
}

// ==================== 错误处理测试 ====================

#[test]
fn test_get_remote_url_wrong_remote() {
    with_test_repo(|_| {
        // 尝试获取不存在的远程仓库 URL 应该失败
        let result = Command::new("git").args(&["remote", "get-url", "nonexistent"]).output();
        assert!(result.is_err() || !result.unwrap().status.success());
    });
}
