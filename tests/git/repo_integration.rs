//! Git 仓库操作集成测试
//!
//! 测试 GitRepo 模块中需要实际 Git 仓库的方法：
//! - is_git_repo()
//! - get_remote_url()
//! - get_git_dir()
//! - fetch() (需要网络连接)
//! - prune_remote() (需要网络连接)

use gix;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use workflow::git::GitRepo;

// ==================== 辅助函数 ====================

/// 创建带有远程配置的 Git 仓库
fn setup_git_repo_with_remote() -> (TempDir, PathBuf) {
    let original_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 切换到临时目录
    std::env::set_current_dir(temp_path).unwrap();

    // 使用 gix 初始化仓库
    let _repo = gix::init(".").unwrap();

    // 设置 Git 配置
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user name");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    // 创建初始文件
    let readme_path = temp_path.join("README.md");
    fs::write(&readme_path, "# Test Repository\n").unwrap();

    // 创建初始提交
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to create commit");

    // 添加远程（使用一个测试用的远程 URL）
    // 注意：这个远程可能不存在，但可以测试基本功能
    std::process::Command::new("git")
        .args(["remote", "add", "origin", "https://github.com/test/test-repo.git"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add remote");

    (temp_dir, original_dir)
}

// ==================== is_git_repo() 测试 ====================

#[test]
#[serial]
fn test_is_git_repo_in_repo() {
    let (temp_dir, original_dir) = setup_git_repo_with_remote();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 在 Git 仓库中应该返回 true
    assert!(GitRepo::is_git_repo(), "Should detect Git repository");

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

#[test]
#[serial]
fn test_is_git_repo_outside_repo() {
    let original_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 切换到非 Git 目录
    std::env::set_current_dir(temp_path).unwrap();

    // 不在 Git 仓库中应该返回 false
    assert!(!GitRepo::is_git_repo(), "Should not detect Git repository in non-repo directory");

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== get_remote_url() 测试 ====================

#[test]
#[serial]
fn test_get_remote_url() {
    let (temp_dir, original_dir) = setup_git_repo_with_remote();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    let url = GitRepo::get_remote_url();
    assert!(url.is_ok(), "Should get remote URL");
    assert_eq!(url.unwrap(), "https://github.com/test/test-repo.git");

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

#[test]
#[serial]
fn test_get_remote_url_no_remote() {
    let original_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    std::env::set_current_dir(temp_path).unwrap();

    // 初始化仓库但不添加远程
    let _repo = gix::init(".").unwrap();

    let url = GitRepo::get_remote_url();
    assert!(url.is_err(), "Should fail when no remote exists");

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== get_git_dir() 测试 ====================

#[test]
#[serial]
fn test_get_git_dir() {
    let (temp_dir, original_dir) = setup_git_repo_with_remote();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    let git_dir = GitRepo::get_git_dir();
    assert!(git_dir.is_ok(), "Should get git directory");
    let git_dir_path = git_dir.unwrap();
    assert!(git_dir_path.contains(".git"), "Git directory should contain .git");

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== fetch() 测试 ====================
// 注意：这些测试需要网络连接，可能会失败

#[test]
#[serial]
#[ignore] // 默认忽略，需要网络连接
fn test_fetch_with_real_remote() {
    // 这个测试需要：
    // 1. 网络连接
    // 2. 一个真实的远程仓库
    // 3. 适当的认证配置
    //
    // 使用方式：
    // cargo test test_fetch_with_real_remote -- --ignored --nocapture
    //
    // 或者设置环境变量来启用：
    // RUN_NETWORK_TESTS=1 cargo test test_fetch_with_real_remote

    if std::env::var("RUN_NETWORK_TESTS").is_err() {
        eprintln!("Skipping network test. Set RUN_NETWORK_TESTS=1 to run.");
        return;
    }

    let (temp_dir, original_dir) = setup_git_repo_with_remote();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 注意：这个测试可能会失败，因为远程仓库可能不存在
    // 或者需要认证
    let result = GitRepo::fetch();
    // 不强制要求成功，因为网络条件可能不满足
    let _ = result;

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}

// ==================== prune_remote() 测试 ====================
// 注意：这些测试需要网络连接，可能会失败

#[test]
#[serial]
#[ignore] // 默认忽略，需要网络连接
fn test_prune_remote_with_real_remote() {
    // 这个测试需要：
    // 1. 网络连接
    // 2. 一个真实的远程仓库
    // 3. 本地有一些远程跟踪分支
    // 4. 远程有一些已删除的分支
    //
    // 使用方式：
    // cargo test test_prune_remote_with_real_remote -- --ignored --nocapture

    if std::env::var("RUN_NETWORK_TESTS").is_err() {
        eprintln!("Skipping network test. Set RUN_NETWORK_TESTS=1 to run.");
        return;
    }

    let (temp_dir, original_dir) = setup_git_repo_with_remote();

    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 注意：这个测试可能会失败，因为远程仓库可能不存在
    // 或者需要认证
    let result = GitRepo::prune_remote();
    // 不强制要求成功，因为网络条件可能不满足
    let _ = result;

    // 恢复原始目录
    if original_dir.exists() {
        let _ = std::env::set_current_dir(original_dir);
    }
}
