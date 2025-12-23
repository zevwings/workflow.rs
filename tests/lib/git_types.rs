//! Git Types 模块测试
//!
//! 测试 Git 类型定义，包括 RepoType 枚举。

use pretty_assertions::assert_eq;
use workflow::git::RepoType;

#[test]
fn test_repo_type_variants() {
    let github = RepoType::GitHub;
    let codeup = RepoType::Codeup;
    let unknown = RepoType::Unknown;

    // 验证所有变体都可以创建
    assert_eq!(format!("{:?}", github), "GitHub");
    assert_eq!(format!("{:?}", codeup), "Codeup");
    assert_eq!(format!("{:?}", unknown), "Unknown");
}

#[test]
fn test_repo_type_equality() {
    assert_eq!(RepoType::GitHub, RepoType::GitHub);
    assert_eq!(RepoType::Codeup, RepoType::Codeup);
    assert_eq!(RepoType::Unknown, RepoType::Unknown);

    assert_ne!(RepoType::GitHub, RepoType::Codeup);
    assert_ne!(RepoType::GitHub, RepoType::Unknown);
    assert_ne!(RepoType::Codeup, RepoType::Unknown);
}

#[test]
fn test_repo_type_clone() {
    let original = RepoType::GitHub;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_repo_type_copy() {
    let original = RepoType::Codeup;
    let copied = original;
    assert_eq!(original, copied);
}

#[test]
fn test_repo_type_debug() {
    let repo_types = vec![RepoType::GitHub, RepoType::Codeup, RepoType::Unknown];

    for repo_type in repo_types {
        let debug_str = format!("{:?}", repo_type);
        assert!(!debug_str.is_empty());
    }
}

