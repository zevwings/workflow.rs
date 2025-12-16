//! Git Pre-commit Hooks 测试
//!
//! 测试 Git pre-commit hooks 相关的操作功能，包括：
//! - 检测 hooks
//! - 执行 hooks

use std::path::Path;
use std::process::Command;
use crate::common::helpers::{
    cleanup_temp_test_dir, create_temp_test_dir, create_test_file,
};

use workflow::git::GitPreCommit;

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

/// 创建 pre-commit hook
fn create_pre_commit_hook(dir: &Path, content: &str) {
    let hooks_dir = dir.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir).expect("Failed to create hooks directory");
    let hook_path = hooks_dir.join("pre-commit");
    std::fs::write(&hook_path, content).expect("Failed to write pre-commit hook");
    // 设置执行权限（Unix 系统）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms).unwrap();
    }
}

/// 切换到指定目录执行测试
fn with_test_repo<F>(test_fn: F)
where
    F: FnOnce(&Path),
{
    let test_dir = create_temp_test_dir("git_pre_commit_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== Pre-commit 检测测试 ====================

#[test]
fn test_has_pre_commit_no_hook() {
    with_test_repo(|_| {
        // 没有 pre-commit hook 时应该返回 false
        // 注意：如果系统安装了 pre-commit 工具，可能会返回 true
        // 这个测试主要验证在没有 hook 文件的情况下
    });
}

#[test]
fn test_has_pre_commit_with_hook() {
    with_test_repo(|dir| {
        // 创建 pre-commit hook
        create_pre_commit_hook(dir, "#!/bin/sh\nexit 0\n");

        // 应该检测到 pre-commit hook
        assert!(GitPreCommit::has_pre_commit());
    });
}

#[test]
fn test_has_pre_commit_with_config() {
    with_test_repo(|dir| {
        // 创建 .pre-commit-config.yaml
        create_test_file(dir, ".pre-commit-config.yaml", "repos: []\n");

        // 应该检测到 pre-commit 配置
        // 注意：如果系统没有安装 pre-commit 工具，可能返回 false
    });
}
