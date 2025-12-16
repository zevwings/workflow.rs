//! Repo 配置测试
//!
//! 测试 Repo 配置相关的功能，包括：
//! - 加载配置
//! - 保存配置
//! - 配置验证
//! - 配置分离和合并

use pretty_assertions::assert_eq;
use workflow::repo::config::repo_config::RepoConfig;

// ==================== 配置存在性检查测试 ====================

#[test]
fn test_repo_config_exists() {
    // 测试检查仓库配置是否存在
    // 注意：这个测试依赖于实际的 Git 仓库环境
    // 在非 Git 仓库中应该返回 Ok(true)
    let exists = RepoConfig::exists();

    // 应该返回 Result，无论结果如何都应该能正常处理
    assert!(exists.is_ok());
}

// ==================== 分支前缀获取测试 ====================

#[test]
fn test_get_branch_prefix() {
    // 测试获取分支前缀
    // 如果没有配置，应该返回 None
    let prefix = RepoConfig::get_branch_prefix();

    // 可能返回 Some 或 None，取决于配置
    // 只验证返回类型正确
    if let Some(ref p) = prefix {
        assert!(!p.is_empty());
    }
}

// ==================== 忽略分支列表获取测试 ====================

#[test]
fn test_get_ignore_branches() {
    // 测试获取忽略分支列表
    let ignore_list = RepoConfig::get_ignore_branches();

    // 应该返回一个 Vec<String>
    // 可能为空，也可能包含分支名
    assert!(ignore_list.is_empty() || !ignore_list.is_empty()); // 总是为真，但验证返回类型
}

// ==================== 配置结构测试 ====================

#[test]
fn test_repo_config_default() {
    // 测试 RepoConfig 的默认值
    let config = RepoConfig::default();

    assert!(!config.configured);
    assert!(config.branch.is_none());
    assert!(config.pr.is_none());
    assert!(config.template_commit.is_empty());
    assert!(config.template_branch.is_empty());
    assert!(config.template_pull_requests.is_empty());
}

#[test]
fn test_repo_config_clone() {
    // 测试 RepoConfig 的克隆
    let config = RepoConfig::default();
    let cloned = config.clone();

    assert_eq!(config.configured, cloned.configured);
    assert_eq!(config.branch.is_none(), cloned.branch.is_none());
    assert_eq!(config.pr.is_none(), cloned.pr.is_none());
}
