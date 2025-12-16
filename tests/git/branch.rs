//! Git 分支管理测试
//!
//! 测试 Git 分支相关的操作功能，包括：
//! - 获取当前分支名
//! - 检查分支是否存在
//! - 创建或切换分支
//! - 获取默认分支
//! - 合并分支
//! - 推送和删除分支

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use std::path::Path;
use std::process::Command;

use workflow::git::GitBranch;

// ==================== 测试辅助函数 ====================

/// 初始化临时 Git 仓库
fn init_test_repo(dir: &Path) {
    Command::new("git")
        .args(&["init"])
        .current_dir(dir)
        .output()
        .expect("Failed to init git repo");

    // 配置 Git 用户（避免提交时缺少用户信息）
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
    let test_dir = create_temp_test_dir("git_branch_test");
    init_test_repo(&test_dir);
    create_initial_commit(&test_dir);

    // 切换到测试目录
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&test_dir).unwrap();

    test_fn(&test_dir);

    // 恢复原始目录
    std::env::set_current_dir(&original_dir).unwrap();
    cleanup_temp_test_dir(&test_dir);
}

// ==================== 当前分支测试 ====================

#[test]
fn test_current_branch() {
    with_test_repo(|_| {
        // 在初始化的仓库中，默认分支通常是 main 或 master
        let branch = GitBranch::current_branch().unwrap();
        assert!(!branch.is_empty());
        // 可能是 "main" 或 "master" 取决于 Git 版本
        assert!(branch == "main" || branch == "master");
    });
}

// ==================== 分支存在性检查测试 ====================

#[test]
fn test_is_branch_exists_local() {
    with_test_repo(|_| {
        // 创建测试分支
        Command::new("git")
            .args(&["branch", "test-branch"])
            .output()
            .expect("Failed to create branch");

        // 检查本地分支是否存在
        let (exists_local, exists_remote) = GitBranch::is_branch_exists("test-branch").unwrap();
        assert!(exists_local, "Local branch should exist");
        assert!(!exists_remote, "Remote branch should not exist");
    });
}

#[test]
fn test_is_branch_exists_nonexistent() {
    with_test_repo(|_| {
        // 检查不存在的分支
        let (exists_local, exists_remote) =
            GitBranch::is_branch_exists("nonexistent-branch").unwrap();
        assert!(!exists_local, "Local branch should not exist");
        assert!(!exists_remote, "Remote branch should not exist");
    });
}

#[test]
fn test_has_local_branch() {
    with_test_repo(|_| {
        // 创建测试分支
        Command::new("git")
            .args(&["branch", "local-test"])
            .output()
            .expect("Failed to create branch");

        assert!(
            GitBranch::has_local_branch("local-test").unwrap(),
            "Local branch should exist"
        );
        assert!(
            !GitBranch::has_local_branch("nonexistent").unwrap(),
            "Nonexistent branch should not exist"
        );
    });
}

// ==================== 分支创建和切换测试 ====================

#[test]
fn test_checkout_branch_create_new() {
    with_test_repo(|_| {
        // 创建新分支
        GitBranch::checkout_branch("new-feature").unwrap();

        // 验证分支已创建并切换
        let current = GitBranch::current_branch().unwrap();
        assert_eq!(current, "new-feature");
    });
}

#[test]
fn test_checkout_branch_switch_existing() {
    with_test_repo(|_| {
        // 创建分支
        Command::new("git")
            .args(&["branch", "feature-branch"])
            .output()
            .expect("Failed to create branch");

        // 切换到已存在的分支
        GitBranch::checkout_branch("feature-branch").unwrap();

        // 验证已切换
        let current = GitBranch::current_branch().unwrap();
        assert_eq!(current, "feature-branch");
    });
}

#[test]
fn test_checkout_branch_already_current() {
    with_test_repo(|_| {
        let original_branch = GitBranch::current_branch().unwrap();

        // 尝试切换到当前分支（应该跳过）
        GitBranch::checkout_branch(&original_branch).unwrap();

        // 验证仍在同一分支
        let current = GitBranch::current_branch().unwrap();
        assert_eq!(current, original_branch);
    });
}

// ==================== 默认分支测试 ====================

#[test]
fn test_get_default_branch() {
    with_test_repo(|_| {
        // 获取默认分支
        let default = GitBranch::get_default_branch().unwrap();
        assert!(!default.is_empty());
        // 可能是 "main" 或 "master"
        assert!(default == "main" || default == "master");
    });
}

// ==================== 分支列表测试 ====================

#[test]
fn test_get_all_branches() {
    with_test_repo(|_| {
        // 创建几个分支
        Command::new("git")
            .args(&["branch", "branch1"])
            .output()
            .expect("Failed to create branch");
        Command::new("git")
            .args(&["branch", "branch2"])
            .output()
            .expect("Failed to create branch");

        // 获取所有分支
        let branches = GitBranch::get_all_branches(false).unwrap();
        assert!(branches.len() >= 3); // 至少包含 main/master, branch1, branch2
        assert!(branches.contains(&"branch1".to_string()));
        assert!(branches.contains(&"branch2".to_string()));
    });
}

#[test]
fn test_get_local_branches() {
    with_test_repo(|_| {
        // 创建几个分支
        Command::new("git")
            .args(&["branch", "local1"])
            .output()
            .expect("Failed to create branch");
        Command::new("git")
            .args(&["branch", "local2"])
            .output()
            .expect("Failed to create branch");

        // 获取本地分支
        let branches = GitBranch::get_local_branches().unwrap();
        assert!(branches.len() >= 3); // 至少包含 main/master, local1, local2
        assert!(branches.contains(&"local1".to_string()));
        assert!(branches.contains(&"local2".to_string()));
    });
}

#[test]
fn test_extract_base_branch_names() {
    // 测试提取基础分支名（去掉前缀）
    let branches = vec![
        "zw/feature-branch".to_string(),
        "master".to_string(),
        "ticket--bug-fix".to_string(),
    ];

    let base_names = GitBranch::extract_base_branch_names(branches);
    assert!(base_names.contains(&"feature-branch".to_string()));
    assert!(base_names.contains(&"master".to_string()));
    assert!(base_names.contains(&"bug-fix".to_string()));
}

// ==================== 分支合并测试 ====================

#[test]
fn test_merge_branch_fast_forward() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(
            &std::env::current_dir().unwrap(),
            "feature.txt",
            "feature content",
        );
        Command::new("git")
            .args(&["add", "feature.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Add feature"])
            .output()
            .expect("Failed to commit");

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();

        // 合并 feature 分支（应该可以 fast-forward）
        GitBranch::merge_branch("feature", workflow::git::MergeStrategy::FastForwardOnly).unwrap();

        // 验证合并成功（文件应该存在）
        assert!(Path::new("feature.txt").exists());
    });
}

#[test]
fn test_merge_branch_merge_strategy() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("feature").unwrap();

        // 在新分支上创建提交
        create_test_file(
            &std::env::current_dir().unwrap(),
            "feature.txt",
            "feature content",
        );
        Command::new("git")
            .args(&["add", "feature.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Add feature"])
            .output()
            .expect("Failed to commit");

        // 切换回主分支并创建另一个提交
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();
        create_test_file(
            &std::env::current_dir().unwrap(),
            "main.txt",
            "main content",
        );
        Command::new("git")
            .args(&["add", "main.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Add main"])
            .output()
            .expect("Failed to commit");

        // 合并 feature 分支（使用普通合并策略）
        GitBranch::merge_branch("feature", workflow::git::MergeStrategy::Merge).unwrap();

        // 验证合并成功
        assert!(Path::new("feature.txt").exists());
        assert!(Path::new("main.txt").exists());
    });
}

// ==================== 分支删除测试 ====================

#[test]
fn test_delete_branch() {
    with_test_repo(|_| {
        // 创建分支
        Command::new("git")
            .args(&["branch", "to-delete"])
            .output()
            .expect("Failed to create branch");

        // 验证分支存在
        assert!(GitBranch::has_local_branch("to-delete").unwrap());

        // 删除分支
        GitBranch::delete("to-delete", false).unwrap();

        // 验证分支已删除
        assert!(!GitBranch::has_local_branch("to-delete").unwrap());
    });
}

#[test]
fn test_delete_branch_force() {
    with_test_repo(|_| {
        // 创建分支并创建提交
        GitBranch::checkout_branch("unmerged-branch").unwrap();
        create_test_file(&std::env::current_dir().unwrap(), "unmerged.txt", "content");
        Command::new("git")
            .args(&["add", "unmerged.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Unmerged commit"])
            .output()
            .expect("Failed to commit");

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();

        // 强制删除未合并的分支
        GitBranch::delete("unmerged-branch", true).unwrap();

        // 验证分支已删除
        assert!(!GitBranch::has_local_branch("unmerged-branch").unwrap());
    });
}

// ==================== 分支重命名测试 ====================

#[test]
fn test_rename_branch() {
    with_test_repo(|_| {
        // 创建分支
        Command::new("git")
            .args(&["branch", "old-name"])
            .output()
            .expect("Failed to create branch");

        // 重命名分支
        GitBranch::rename(Some("old-name"), "new-name").unwrap();

        // 验证新名称存在，旧名称不存在
        assert!(GitBranch::has_local_branch("new-name").unwrap());
        assert!(!GitBranch::has_local_branch("old-name").unwrap());
    });
}

#[test]
fn test_rename_current_branch() {
    with_test_repo(|_| {
        let _original_branch = GitBranch::current_branch().unwrap();

        // 创建新分支并切换
        GitBranch::checkout_branch("to-rename").unwrap();

        // 重命名当前分支
        GitBranch::rename(None, "renamed-branch").unwrap();

        // 验证重命名成功
        let current = GitBranch::current_branch().unwrap();
        assert_eq!(current, "renamed-branch");
    });
}

// ==================== 分支比较测试 ====================

#[test]
fn test_is_branch_ahead() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("ahead-branch").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "ahead.txt", "content");
        Command::new("git")
            .args(&["add", "ahead.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Ahead commit"])
            .output()
            .expect("Failed to commit");

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();

        // 检查新分支是否领先
        let is_ahead = GitBranch::is_branch_ahead("ahead-branch", &default_branch).unwrap();
        assert!(is_ahead, "Branch should be ahead of base branch");
    });
}

#[test]
fn test_get_commits_between() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("commits-branch").unwrap();

        // 创建多个提交
        for i in 1..=3 {
            create_test_file(
                &std::env::current_dir().unwrap(),
                &format!("file{}.txt", i),
                &format!("content {}", i),
            );
            Command::new("git")
                .args(&["add", &format!("file{}.txt", i)])
                .output()
                .expect("Failed to add file");
            Command::new("git")
                .args(&["commit", "-m", &format!("Commit {}", i)])
                .output()
                .expect("Failed to commit");
        }

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();

        // 获取两个分支之间的提交
        let commits = GitBranch::get_commits_between(&default_branch, "commits-branch").unwrap();
        assert_eq!(commits.len(), 3, "Should have 3 commits");
    });
}

#[test]
fn test_merge_base() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("branch1").unwrap();
        create_test_file(&std::env::current_dir().unwrap(), "file1.txt", "content1");
        Command::new("git")
            .args(&["add", "file1.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Commit on branch1"])
            .output()
            .expect("Failed to commit");

        // 切换回主分支并创建另一个分支
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();
        GitBranch::checkout_branch("branch2").unwrap();
        create_test_file(&std::env::current_dir().unwrap(), "file2.txt", "content2");
        Command::new("git")
            .args(&["add", "file2.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Commit on branch2"])
            .output()
            .expect("Failed to commit");

        // 获取两个分支的共同祖先
        let merge_base = GitBranch::merge_base("branch1", "branch2").unwrap();
        assert!(!merge_base.is_empty());
    });
}

#[test]
fn test_is_branch_based_on() {
    with_test_repo(|_| {
        let default_branch = GitBranch::get_default_branch().unwrap();

        // 创建基于默认分支的新分支
        GitBranch::checkout_branch("based-branch").unwrap();

        // 检查新分支是否基于默认分支
        let is_based = GitBranch::is_branch_based_on("based-branch", &default_branch).unwrap();
        assert!(is_based, "Branch should be based on default branch");
    });
}

#[test]
fn test_is_branch_merged() {
    with_test_repo(|_| {
        // 创建并切换到新分支
        GitBranch::checkout_branch("merge-test").unwrap();

        // 在新分支上创建提交
        create_test_file(&std::env::current_dir().unwrap(), "merge.txt", "content");
        Command::new("git")
            .args(&["add", "merge.txt"])
            .output()
            .expect("Failed to add file");
        Command::new("git")
            .args(&["commit", "-m", "Merge test"])
            .output()
            .expect("Failed to commit");

        // 切换回主分支
        let default_branch = GitBranch::get_default_branch().unwrap();
        GitBranch::checkout_branch(&default_branch).unwrap();

        // 合并分支
        GitBranch::merge_branch("merge-test", workflow::git::MergeStrategy::Merge).unwrap();

        // 检查分支是否已合并
        let is_merged = GitBranch::is_branch_merged("merge-test", &default_branch).unwrap();
        assert!(is_merged, "Branch should be merged");
    });
}

// ==================== 冲突检测测试 ====================

#[test]
fn test_has_merge_conflicts_no_conflicts() {
    with_test_repo(|_| {
        // 在没有合并冲突的情况下
        let has_conflicts = GitBranch::has_merge_conflicts().unwrap();
        assert!(!has_conflicts, "Should not have merge conflicts");
    });
}

// ==================== 错误处理测试 ====================

#[test]
fn test_delete_nonexistent_branch_error() {
    with_test_repo(|_| {
        // 尝试删除不存在的分支应该失败
        let result = GitBranch::delete("nonexistent", false);
        assert!(result.is_err(), "Should fail to delete nonexistent branch");
    });
}

#[test]
fn test_merge_base_nonexistent_branch_error() {
    with_test_repo(|_| {
        // 尝试获取不存在分支的 merge base 应该失败
        let result = GitBranch::merge_base("nonexistent1", "nonexistent2");
        assert!(result.is_err(), "Should fail for nonexistent branches");
    });
}
