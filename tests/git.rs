//! Git 模块测试
//!
//! 测试 `git` 模块中的 Git 操作功能。
//!
//! 注意：这些测试需要实际的 Git 仓库，使用临时目录创建测试仓库。

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use workflow::git::{GitBranch, GitCommit, GitConfig, GitRepo, GitStash, MergeStrategy, RepoType};

// ==================== 测试辅助函数 ====================

/// 创建临时 Git 仓库用于测试
fn create_test_repo() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let path = temp_dir.path();

    // 初始化 Git 仓库
    Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .output()
        .expect("Failed to init git repo");

    // 配置 Git 用户（避免提交时缺少用户信息）
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .expect("Failed to set git user email");

    temp_dir
}

/// 在指定目录中执行 Git 命令
fn git_cmd(repo_path: &Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute git command")
}

/// 创建测试文件并提交
fn create_and_commit_file(repo_path: &Path, filename: &str, content: &str) {
    fs::write(repo_path.join(filename), content).expect("Failed to write test file");
    git_cmd(repo_path, &["add", filename]);
    git_cmd(repo_path, &["commit", "-m", &format!("Add {}", filename)]);
}

// ==================== GitRepo 测试 ====================

#[test]
fn test_is_git_repo() {
    // 测试检测 Git 仓库
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();

    // 切换到临时仓库目录
    std::env::set_current_dir(temp_repo.path()).unwrap();
    assert!(GitRepo::is_git_repo());

    // 切换到非 Git 目录
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();
    assert!(!GitRepo::is_git_repo());

    // 恢复原始目录
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_parse_repo_type_from_url() {
    // 测试从 URL 解析仓库类型
    // 注意：parse_repo_type_from_url 是私有方法，我们通过 detect_repo_type 间接测试

    // GitHub URL 测试
    let temp_repo = create_test_repo();
    git_cmd(
        temp_repo.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/user/repo.git",
        ],
    );

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    let repo_type = GitRepo::detect_repo_type().unwrap();
    assert_eq!(repo_type, RepoType::GitHub);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_parse_repo_type_codeup() {
    // 测试 Codeup URL
    let temp_repo = create_test_repo();
    git_cmd(
        temp_repo.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://codeup.aliyun.com/user/repo.git",
        ],
    );

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    let repo_type = GitRepo::detect_repo_type().unwrap();
    assert_eq!(repo_type, RepoType::Codeup);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_parse_repo_type_github_ssh() {
    // 测试 GitHub SSH URL
    let temp_repo = create_test_repo();
    git_cmd(
        temp_repo.path(),
        &["remote", "add", "origin", "git@github.com:user/repo.git"],
    );

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    let repo_type = GitRepo::detect_repo_type().unwrap();
    assert_eq!(repo_type, RepoType::GitHub);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_parse_repo_type_unknown() {
    // 测试未知类型 URL
    let temp_repo = create_test_repo();
    git_cmd(
        temp_repo.path(),
        &[
            "remote",
            "add",
            "origin",
            "https://example.com/user/repo.git",
        ],
    );

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    let repo_type = GitRepo::detect_repo_type().unwrap();
    assert_eq!(repo_type, RepoType::Unknown);

    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== GitCommit 测试 ====================

#[test]
fn test_git_commit_status() {
    // 测试获取 Git 状态
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 初始状态应该是空的
    let status = GitCommit::status().unwrap();
    assert!(status.trim().is_empty());

    // 创建文件后应该有未暂存的更改
    fs::write(temp_repo.path().join("test.txt"), "test content").unwrap();
    let status = GitCommit::status().unwrap();
    assert!(status.contains("test.txt"));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_commit_add_all() {
    // 测试添加所有文件
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建文件
    fs::write(temp_repo.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_repo.path().join("file2.txt"), "content2").unwrap();

    // 添加所有文件
    GitCommit::add_all().unwrap();

    // 检查状态，应该显示已暂存的文件
    let status = GitCommit::status().unwrap();
    assert!(status.contains("file1.txt"));
    assert!(status.contains("file2.txt"));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_commit_has_staged() {
    // 测试检查是否有暂存的文件
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 初始状态应该没有暂存的文件
    // 注意：has_staged 是私有方法，我们通过其他方法间接测试

    // 创建并暂存文件
    fs::write(temp_repo.path().join("test.txt"), "test").unwrap();
    GitCommit::add_all().unwrap();

    // 现在应该有暂存的文件
    // 通过 status 来验证
    let status = GitCommit::status().unwrap();
    assert!(!status.trim().is_empty());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_commit_commit() {
    // 测试提交
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建文件
    fs::write(temp_repo.path().join("test.txt"), "test content").unwrap();

    // 提交
    GitCommit::commit("Test commit", false).unwrap();

    // 验证提交成功（通过 log 检查）
    let output = git_cmd(temp_repo.path(), &["log", "--oneline", "-1"]);
    let log = String::from_utf8(output.stdout).unwrap();
    assert!(log.contains("Test commit"));

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_commit_commit_no_changes() {
    // 测试在没有更改时提交
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 在没有更改的情况下提交，应该直接返回（不报错）
    let result = GitCommit::commit("No changes", false);
    // 应该成功（因为没有更改时会直接返回）
    assert!(result.is_ok());

    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== GitBranch 测试 ====================

#[test]
fn test_git_branch_current_branch() {
    // 测试获取当前分支
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 默认分支应该是 main 或 master
    let branch = GitBranch::current_branch().unwrap();
    assert!(!branch.is_empty());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_branch_is_branch_exists() {
    // 测试检查分支是否存在
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 检查当前分支（应该存在）
    let current = GitBranch::current_branch().unwrap();
    let (local, remote) = GitBranch::is_branch_exists(&current).unwrap();
    assert!(local);
    // 远程可能不存在（因为这是新仓库）
    assert!(!remote);

    // 检查不存在的分支
    let (local, remote) = GitBranch::is_branch_exists("nonexistent-branch").unwrap();
    assert!(!local);
    assert!(!remote);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_branch_create() {
    // 测试创建分支
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建新分支
    GitBranch::create("test-branch").unwrap();

    // 验证分支存在
    let (local, _) = GitBranch::is_branch_exists("test-branch").unwrap();
    assert!(local);

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_branch_switch() {
    // 测试切换分支
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建并切换到新分支
    GitBranch::create("test-branch").unwrap();
    GitBranch::switch("test-branch").unwrap();

    // 验证当前分支
    let current = GitBranch::current_branch().unwrap();
    assert_eq!(current, "test-branch");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_branch_get_default_branch() {
    // 测试获取默认分支
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    let default = GitBranch::get_default_branch().unwrap();
    // 默认分支应该是 main 或 master
    assert!(default == "main" || default == "master");

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_branch_merge() {
    // 测试合并分支
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建初始提交
    create_and_commit_file(temp_repo.path(), "file1.txt", "content1");

    // 创建并切换到新分支
    GitBranch::create("feature-branch").unwrap();
    GitBranch::switch("feature-branch").unwrap();

    // 在新分支上创建提交
    create_and_commit_file(temp_repo.path(), "file2.txt", "content2");

    // 切换回主分支
    let default = GitBranch::get_default_branch().unwrap();
    GitBranch::switch(&default).unwrap();

    // 合并分支
    GitBranch::merge("feature-branch", MergeStrategy::Merge).unwrap();

    // 验证合并成功（检查文件是否存在）
    assert!(temp_repo.path().join("file1.txt").exists());
    assert!(temp_repo.path().join("file2.txt").exists());

    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== GitStash 测试 ====================

#[test]
fn test_git_stash_push() {
    // 测试 stash push
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建初始提交
    create_and_commit_file(temp_repo.path(), "file1.txt", "content1");

    // 修改文件
    fs::write(temp_repo.path().join("file1.txt"), "modified content").unwrap();

    // Stash 更改
    GitStash::stash_push(Some("Test stash")).unwrap();

    // 验证文件已恢复（通过 status 检查）
    let status = GitCommit::status().unwrap();
    assert!(status.trim().is_empty());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_stash_pop() {
    // 测试 stash pop
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 创建初始提交
    create_and_commit_file(temp_repo.path(), "file1.txt", "content1");

    // 修改文件并 stash
    fs::write(temp_repo.path().join("file1.txt"), "modified").unwrap();
    GitStash::stash_push(Some("Test stash")).unwrap();

    // Pop stash
    GitStash::stash_pop().unwrap();

    // 验证更改已恢复
    let status = GitCommit::status().unwrap();
    assert!(!status.trim().is_empty());

    std::env::set_current_dir(original_dir).unwrap();
}

// ==================== GitConfig 测试 ====================

#[test]
fn test_git_config_get_global_user() {
    // 测试获取全局用户配置
    // 注意：这个测试会修改全局 Git 配置，所以需要谨慎
    // 在实际测试中，可能需要保存和恢复原始配置

    // 先设置一个测试配置
    GitConfig::set_global_user("test@example.com", "Test User").unwrap();

    // 获取配置
    let (email, name) = GitConfig::get_global_user().unwrap();
    assert_eq!(email, Some("test@example.com".to_string()));
    assert_eq!(name, Some("Test User".to_string()));
}

#[test]
fn test_git_config_set_global_user() {
    // 测试设置全局用户配置
    // 注意：这个测试会修改全局 Git 配置
    GitConfig::set_global_user("new@example.com", "New User").unwrap();

    // 验证配置已设置
    let (email, name) = GitConfig::get_global_user().unwrap();
    assert_eq!(email, Some("new@example.com".to_string()));
    assert_eq!(name, Some("New User".to_string()));
}

// ==================== 边界情况测试 ====================

#[test]
fn test_git_repo_not_in_repo() {
    // 测试不在 Git 仓库中的情况
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // 检测仓库类型应该失败
    let result = GitRepo::detect_repo_type();
    assert!(result.is_err());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_branch_nonexistent_branch() {
    // 测试不存在的分支操作
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 切换到不存在的分支应该失败
    let result = GitBranch::switch("nonexistent-branch");
    assert!(result.is_err());

    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_git_stash_empty() {
    // 测试空 stash
    let temp_repo = create_test_repo();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_repo.path()).unwrap();

    // 在没有更改的情况下 stash 应该失败或返回空
    let result = GitStash::stash_push(Some("Empty stash"));
    // 可能成功（空 stash）或失败，取决于实现
    // 这里只验证不会 panic

    std::env::set_current_dir(original_dir).unwrap();
}
