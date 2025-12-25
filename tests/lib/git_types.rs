//! Git Types 模块测试
//!
//! 测试 Git 类型定义，包括 RepoType 枚举。

use pretty_assertions::assert_eq;
use workflow::git::RepoType;

// ==================== RepoType Variant Tests ====================

#[test]
fn test_repo_type_variants_with_all_types_creates_variants() {
    // Arrange: 准备所有 RepoType 变体
    let github = RepoType::GitHub;
    let codeup = RepoType::Codeup;
    let unknown = RepoType::Unknown;

    // Act & Assert: 验证所有变体都可以创建且格式化正确
    assert_eq!(format!("{:?}", github), "GitHub");
    assert_eq!(format!("{:?}", codeup), "Codeup");
    assert_eq!(format!("{:?}", unknown), "Unknown");
}

// ==================== RepoType Equality Tests ====================

#[test]
fn test_repo_type_equality_with_same_types_returns_equal() {
    // Arrange: 准备相同类型的 RepoType 实例

    // Act & Assert: 验证相同类型相等
    assert_eq!(RepoType::GitHub, RepoType::GitHub);
    assert_eq!(RepoType::Codeup, RepoType::Codeup);
    assert_eq!(RepoType::Unknown, RepoType::Unknown);
}

#[test]
fn test_repo_type_equality_with_different_types_returns_not_equal() {
    // Arrange: 准备不同类型的 RepoType 实例

    // Act & Assert: 验证不同类型不相等
    assert_ne!(RepoType::GitHub, RepoType::Codeup);
    assert_ne!(RepoType::GitHub, RepoType::Unknown);
    assert_ne!(RepoType::Codeup, RepoType::Unknown);
}

// ==================== RepoType Clone and Copy Tests ====================

#[test]
fn test_repo_type_clone_with_valid_type_creates_clone() {
    // Arrange: 准备原始 RepoType
    let original = RepoType::GitHub;

    // Act: 克隆 RepoType
    let cloned = original.clone();

    // Assert: 验证克隆的实例与原始实例相等
    assert_eq!(original, cloned);
}

#[test]
fn test_repo_type_copy_with_valid_type_creates_copy() {
    // Arrange: 准备原始 RepoType
    let original = RepoType::Codeup;

    // Act: 复制 RepoType（Copy trait）
    let copied = original;

    // Assert: 验证复制的实例与原始实例相等
    assert_eq!(original, copied);
}

// ==================== RepoType Debug Tests ====================

#[test]
fn test_repo_type_debug_with_all_types_returns_debug_string() {
    // Arrange: 准备所有 RepoType 变体
    let repo_types = vec![RepoType::GitHub, RepoType::Codeup, RepoType::Unknown];

    // Act & Assert: 验证所有类型的调试输出不为空
    for repo_type in repo_types {
        let debug_str = format!("{:?}", repo_type);
        assert!(!debug_str.is_empty());
    }
}

