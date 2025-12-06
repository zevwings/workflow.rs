//! Branch 命令测试
//!
//! 测试 `commands::branch` 模块中的分支管理命令。

use workflow::commands::branch::{
    add_ignore_branch, get_ignore_branches, remove_ignore_branch, BranchConfig,
};

// ==================== BranchConfig 测试 ====================

#[test]
fn test_branch_config_new() {
    // 测试创建新的分支配置
    let config = BranchConfig::default();
    assert!(config.repositories.is_empty());
}

#[test]
fn test_branch_config_with_ignores() {
    // 测试包含忽略分支的配置
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/old".to_string()).unwrap();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "bugfix/test".to_string()).unwrap();

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert_eq!(ignores.len(), 2);
    assert!(ignores.contains(&"feature/old".to_string()));
    assert!(ignores.contains(&"bugfix/test".to_string()));
}

// ==================== 忽略分支操作测试 ====================

#[test]
fn test_add_ignore_branch() {
    // 测试添加忽略分支
    let mut config = BranchConfig::default();
    let added = add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test".to_string()).unwrap();

    assert!(added); // 新添加
    let ignores = get_ignore_branches(&config, "owner/repo");
    assert!(ignores.contains(&"feature/test".to_string()));
}

#[test]
fn test_add_ignore_branch_duplicate() {
    // 测试添加重复的忽略分支
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test".to_string()).unwrap();
    let added = add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test".to_string()).unwrap();

    // 应该返回 false（已存在）
    assert!(!added);

    // 应该只添加一次
    let ignores = get_ignore_branches(&config, "owner/repo");
    let count = ignores.iter().filter(|&b| b == "feature/test").count();
    assert_eq!(count, 1);
}

#[test]
fn test_remove_ignore_branch() {
    // 测试移除忽略分支
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test".to_string()).unwrap();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "bugfix/test".to_string()).unwrap();

    remove_ignore_branch(&mut config, "owner/repo", "feature/test").unwrap();

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert!(!ignores.contains(&"feature/test".to_string()));
    assert!(ignores.contains(&"bugfix/test".to_string()));
}

#[test]
fn test_remove_ignore_branch_nonexistent() {
    // 测试移除不存在的忽略分支
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test".to_string()).unwrap();

    // 移除不存在的分支应该不会报错
    remove_ignore_branch(&mut config, "owner/repo", "nonexistent").unwrap();

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert_eq!(ignores.len(), 1);
    assert!(ignores.contains(&"feature/test".to_string()));
}

#[test]
fn test_get_ignore_branches() {
    // 测试获取忽略分支列表
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test".to_string()).unwrap();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "bugfix/test".to_string()).unwrap();

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert_eq!(ignores.len(), 2);
    assert!(ignores.contains(&"feature/test".to_string()));
    assert!(ignores.contains(&"bugfix/test".to_string()));
}

#[test]
fn test_get_ignore_branches_empty() {
    // 测试空配置
    let config = BranchConfig::default();
    let ignores = get_ignore_branches(&config, "owner/repo");
    assert!(ignores.is_empty());
}

#[test]
fn test_get_ignore_branches_different_repos() {
    // 测试不同仓库的忽略分支
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo1".to_string(), "feature/test".to_string()).unwrap();
    add_ignore_branch(&mut config, "owner/repo2".to_string(), "bugfix/test".to_string()).unwrap();

    let ignores1 = get_ignore_branches(&config, "owner/repo1");
    let ignores2 = get_ignore_branches(&config, "owner/repo2");

    assert_eq!(ignores1.len(), 1);
    assert_eq!(ignores2.len(), 1);
    assert!(ignores1.contains(&"feature/test".to_string()));
    assert!(ignores2.contains(&"bugfix/test".to_string()));
}

// ==================== 边界情况测试 ====================

#[test]
fn test_ignore_branch_empty_string() {
    // 测试空字符串分支名
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "".to_string()).unwrap();

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert!(ignores.contains(&"".to_string()));
}

#[test]
fn test_ignore_branch_special_characters() {
    // 测试特殊字符分支名
    let mut config = BranchConfig::default();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "feature/test-branch".to_string()).unwrap();
    add_ignore_branch(&mut config, "owner/repo".to_string(), "bugfix/test_branch".to_string()).unwrap();

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert!(ignores.contains(&"feature/test-branch".to_string()));
    assert!(ignores.contains(&"bugfix/test_branch".to_string()));
}

#[test]
fn test_ignore_branch_large_list() {
    // 测试大量忽略分支
    let mut config = BranchConfig::default();
    for i in 0..100 {
        add_ignore_branch(&mut config, "owner/repo".to_string(), format!("feature/branch_{}", i)).unwrap();
    }

    let ignores = get_ignore_branches(&config, "owner/repo");
    assert_eq!(ignores.len(), 100);
}

// ==================== 集成测试场景 ====================

#[test]
fn test_branch_ignore_workflow() {
    // 测试完整的忽略分支工作流
    let mut config = BranchConfig::default();
    let repo = "owner/repo".to_string();

    // 1. 添加多个忽略分支
    add_ignore_branch(&mut config, repo.clone(), "feature/old".to_string()).unwrap();
    add_ignore_branch(&mut config, repo.clone(), "bugfix/test".to_string()).unwrap();
    add_ignore_branch(&mut config, repo.clone(), "hotfix/urgent".to_string()).unwrap();

    let ignores = get_ignore_branches(&config, &repo);
    assert_eq!(ignores.len(), 3);

    // 2. 获取忽略分支列表
    let ignores = get_ignore_branches(&config, &repo);
    assert_eq!(ignores.len(), 3);

    // 3. 移除一个忽略分支
    remove_ignore_branch(&mut config, &repo, "bugfix/test").unwrap();
    let ignores = get_ignore_branches(&config, &repo);
    assert_eq!(ignores.len(), 2);
    assert!(!ignores.contains(&"bugfix/test".to_string()));

    // 4. 验证剩余的忽略分支
    let remaining = get_ignore_branches(&config, &repo);
    assert_eq!(remaining.len(), 2);
    assert!(remaining.contains(&"feature/old".to_string()));
    assert!(remaining.contains(&"hotfix/urgent".to_string()));
}
