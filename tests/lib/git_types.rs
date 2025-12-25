//! Git Types 模块测试
//!
//! 测试 Git 类型定义，包括 RepoType 枚举。

use pretty_assertions::assert_eq;
use workflow::git::RepoType;

// ==================== RepoType Variant Tests ====================

/// 测试RepoType所有变体创建和格式化
///
/// ## 测试目的
/// 验证 `RepoType` 枚举的所有变体（GitHub, Codeup, Unknown）都能够正确创建和格式化。
///
/// ## 测试场景
/// 1. 创建所有RepoType变体
/// 2. 验证Debug格式化输出正确
///
/// ## 预期结果
/// - 所有变体都能正确创建
/// - Debug格式化输出为预期的字符串（"GitHub", "Codeup", "Unknown"）
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

/// 测试RepoType相等性（相同类型）
///
/// ## 测试目的
/// 验证 `RepoType` 枚举的相等性比较，相同类型的实例应该相等。
///
/// ## 测试场景
/// 1. 创建相同类型的RepoType实例
/// 2. 验证相等性比较
///
/// ## 预期结果
/// - 相同类型的实例相等
#[test]
fn test_repo_type_equality_with_same_types_returns_equal() {
    // Arrange: 准备相同类型的 RepoType 实例

    // Act & Assert: 验证相同类型相等
    assert_eq!(RepoType::GitHub, RepoType::GitHub);
    assert_eq!(RepoType::Codeup, RepoType::Codeup);
    assert_eq!(RepoType::Unknown, RepoType::Unknown);
}

/// 测试RepoType相等性（不同类型）
///
/// ## 测试目的
/// 验证 `RepoType` 枚举的相等性比较，不同类型的实例应该不相等。
///
/// ## 测试场景
/// 1. 创建不同类型的RepoType实例
/// 2. 验证不相等性比较
///
/// ## 预期结果
/// - 不同类型的实例不相等
#[test]
fn test_repo_type_equality_with_different_types_returns_not_equal() {
    // Arrange: 准备不同类型的 RepoType 实例

    // Act & Assert: 验证不同类型不相等
    assert_ne!(RepoType::GitHub, RepoType::Codeup);
    assert_ne!(RepoType::GitHub, RepoType::Unknown);
    assert_ne!(RepoType::Codeup, RepoType::Unknown);
}

// ==================== RepoType Clone and Copy Tests ====================

/// 测试RepoType克隆功能
///
/// ## 测试目的
/// 验证 `RepoType` 枚举实现了 `Clone` trait，能够正确克隆实例。
///
/// ## 测试场景
/// 1. 创建原始RepoType实例
/// 2. 克隆实例
/// 3. 验证克隆的实例与原始实例相等
///
/// ## 预期结果
/// - 克隆成功
/// - 克隆的实例与原始实例相等
#[test]
fn test_repo_type_clone_with_valid_type_creates_clone() {
    // Arrange: 准备原始 RepoType
    let original = RepoType::GitHub;

    // Act: 克隆 RepoType
    let cloned = original.clone();

    // Assert: 验证克隆的实例与原始实例相等
    assert_eq!(original, cloned);
}

/// 测试RepoType复制功能
///
/// ## 测试目的
/// 验证 `RepoType` 枚举实现了 `Copy` trait，能够正确复制实例。
///
/// ## 测试场景
/// 1. 创建原始RepoType实例
/// 2. 复制实例（通过赋值）
/// 3. 验证复制的实例与原始实例相等
///
/// ## 预期结果
/// - 复制成功
/// - 复制的实例与原始实例相等
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

/// 测试RepoType Debug格式化
///
/// ## 测试目的
/// 验证 `RepoType` 枚举实现了 `Debug` trait，能够正确格式化输出。
///
/// ## 测试场景
/// 1. 创建所有RepoType变体
/// 2. 使用Debug格式化输出
/// 3. 验证输出不为空
///
/// ## 预期结果
/// - 所有类型的Debug输出不为空
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

