//! Git 测试辅助函数
//!
//! 提供创建测试用 Git 仓库的共享函数和 fixtures。

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// 创建带有初始提交的 Git 仓库
///
/// 返回 `(TempDir, 原始目录路径)`，调用者需要负责恢复原始目录。
/// 如果 Git 命令不可用，返回 `None`。
#[allow(dead_code)]
pub fn setup_git_repo_with_gix() -> Option<(TempDir, PathBuf)> {
    // 保存原始目录在创建临时目录之前
    let original_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 切换到临时目录，确保 gix::init 有正确的工作目录上下文
    std::env::set_current_dir(temp_path).unwrap();

    // 使用 gix 初始化仓库（在临时目录中）
    let _repo = gix::init(".").unwrap();

    // 设置基本的 Git 配置
    let git_config_result = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output();

    if git_config_result.is_err() {
        // 如果 git 命令不可用，跳过这个测试
        eprintln!("Git command not available, skipping test");
        return None;
    }

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    // 创建初始文件
    let readme_path = temp_path.join("README.md");
    std::fs::write(&readme_path, "# Test Repository\n").unwrap();

    // 使用命令行 git 来创建初始提交（在临时目录中执行）
    let add_output = std::process::Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add file");

    if !add_output.status.success() {
        panic!(
            "Git add failed: {}",
            String::from_utf8_lossy(&add_output.stderr)
        );
    }

    let commit_output = std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to create commit");

    if !commit_output.status.success() {
        panic!(
            "Git commit failed: {}",
            String::from_utf8_lossy(&commit_output.stderr)
        );
    }

    Some((temp_dir, original_dir))
}

/// 创建带有初始提交的 Git 仓库（使用命令行 git）
///
/// 返回 `(TempDir, 原始目录路径)`，调用者需要负责恢复原始目录。
/// 如果 Git 命令不可用，会 panic。
#[allow(dead_code)]
pub fn setup_git_repo() -> (TempDir, PathBuf) {
    // 保存原始目录在创建临时目录之前
    let original_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));

    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 切换到临时目录，确保 gix::init 有正确的工作目录上下文
    std::env::set_current_dir(temp_path).unwrap();

    // 使用 gix 初始化仓库（在临时目录中）
    let _repo = gix::init(".").unwrap();

    // 设置基本的 Git 配置
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
    std::fs::write(&readme_path, "# Test Repository\n").unwrap();

    // 使用命令行 git 来创建初始提交（在临时目录中执行）
    let add_output = std::process::Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add file");

    if !add_output.status.success() {
        panic!(
            "Git add failed: {}",
            String::from_utf8_lossy(&add_output.stderr)
        );
    }

    let commit_output = std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to create commit");

    if !commit_output.status.success() {
        panic!(
            "Git commit failed: {}",
            String::from_utf8_lossy(&commit_output.stderr)
        );
    }

    (temp_dir, original_dir)
}

/// 创建带有初始提交的 Git 仓库（使用命令行 git，不切换目录）
///
/// 返回 `TempDir`，不切换当前工作目录。
/// 适用于不需要切换目录的测试场景。
#[allow(dead_code)]
pub fn create_git_repo_with_commit() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // 在临时目录中执行 Git 操作，而不是切换当前工作目录
    let init_output = std::process::Command::new("git")
        .args(["init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to init git repo");

    if !init_output.status.success() {
        panic!(
            "Git init failed: {}",
            String::from_utf8_lossy(&init_output.stderr)
        );
    }

    // 配置 Git 用户
    let name_output = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user name");

    if !name_output.status.success() {
        panic!(
            "Git config user.name failed: {}",
            String::from_utf8_lossy(&name_output.stderr)
        );
    }

    let email_output = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    if !email_output.status.success() {
        panic!(
            "Git config user.email failed: {}",
            String::from_utf8_lossy(&email_output.stderr)
        );
    }

    // 创建初始提交
    let readme_path = temp_path.join("README.md");
    fs::write(&readme_path, "# Test Repository").expect("Failed to write file");

    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to add file");

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to commit");

    temp_dir
}
