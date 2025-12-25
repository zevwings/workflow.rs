//! Branch 命令辅助函数测试
//!
//! 测试分支命令的辅助函数，包括分支排序、优先级计算等。
//!
//! ## 测试策略
//!
//! - 测试函数返回 `Result<()>`，使用 `?` 运算符处理错误
//! - 测试分支优先级排序逻辑
//! - 测试各种边界情况

use color_eyre::Result;
use pretty_assertions::assert_eq;
use workflow::commands::branch::helpers::sort_branches_with_priority;

// 注意: get_branch_priority 是私有函数，我们需要通过公共 API 测试其效果

#[test]
fn test_sort_branches_with_priority_main_first() -> Result<()> {
    let branches = vec![
        "feature/test".to_string(),
        "main".to_string(),
        "develop".to_string(),
        "hotfix/fix".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // main 应该在第一位
    assert_eq!(sorted[0], "main");
    Ok(())
}

#[test]
fn test_sort_branches_with_priority_master_after_main() -> Result<()> {
    let branches = vec![
        "master".to_string(),
        "main".to_string(),
        "feature/test".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // main 应该在 master 之前
    let main_pos = sorted.iter().position(|b| b == "main").expect("main not found");
    let master_pos = sorted.iter().position(|b| b == "master").expect("master not found");
    assert!(main_pos < master_pos);
    Ok(())
}

#[test]
fn test_sort_branches_with_priority_develop_second() -> Result<()> {
    let branches = vec![
        "feature/test".to_string(),
        "develop".to_string(),
        "hotfix/fix".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // develop 应该在 feature 和 hotfix 之前
    let develop_pos = sorted.iter().position(|b| b == "develop").expect("develop not found");
    let feature_pos = sorted.iter().position(|b| b == "feature/test").expect("feature not found");
    assert!(develop_pos < feature_pos);
    Ok(())
}

#[test]
fn test_sort_branches_with_priority_alphabetical_others() -> Result<()> {
    let branches = vec!["zebra".to_string(), "alpha".to_string(), "beta".to_string()];

    let sorted = sort_branches_with_priority(branches)?;

    // 其他分支应该按字母顺序排序
    assert_eq!(sorted[0], "alpha");
    assert_eq!(sorted[1], "beta");
    assert_eq!(sorted[2], "zebra");
    Ok(())
}

#[test]
fn test_sort_branches_empty() -> Result<()> {
    let branches = vec![];
    let sorted = sort_branches_with_priority(branches)?;
    assert!(sorted.is_empty());
    Ok(())
}

#[test]
fn test_sort_branches_single() -> Result<()> {
    let branches = vec!["feature/test".to_string()];
    let sorted = sort_branches_with_priority(branches)?;
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0], "feature/test");
    Ok(())
}

// ==================== 更多业务逻辑测试 ====================

#[test]
fn test_sort_branches_with_prefix_priority() -> Result<()> {
    // Arrange: 准备测试带前缀的分支优先级
    // 注意：这个测试依赖于当前分支或配置的前缀
    // 如果当前分支有前缀（如 "zw/feature-branch"），则 "zw/" 开头的分支应该有更高优先级
    let branches = vec![
        "other/feature".to_string(),
        "zw/feature-branch".to_string(),
        "main".to_string(),
        "zw/bugfix".to_string(),
        "develop".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // main 应该在第一位
    assert_eq!(sorted[0], "main");

    // develop 应该在 main 之后
    let develop_pos = sorted.iter().position(|b| b == "develop").expect("develop not found");
    assert!(develop_pos > 0, "develop should come after main");

    // 其他分支应该按字母顺序或前缀优先级排序
    // 注意：前缀优先级取决于当前分支或配置，这里主要验证排序不会崩溃
    assert_eq!(sorted.len(), 5);
    Ok(())
}

#[test]
fn test_sort_branches_all_priority_levels() -> Result<()> {
    // Arrange: 准备测试所有优先级级别的分支
    let branches = vec![
        "zebra".to_string(),        // Priority 4
        "main".to_string(),         // Priority 1
        "master".to_string(),       // Priority 1 (after main)
        "develop".to_string(),      // Priority 2
        "alpha".to_string(),        // Priority 4
        "feature/test".to_string(), // Priority 4
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // Assert: 验证排序顺序
    assert_eq!(sorted[0], "main", "main should be first");
    assert_eq!(sorted[1], "master", "master should be second");

    let develop_pos = sorted.iter().position(|b| b == "develop").expect("develop not found");
    assert!(develop_pos == 2, "develop should be third");

    // 其他分支应该按字母顺序
    let alpha_pos = sorted.iter().position(|b| b == "alpha").expect("alpha not found");
    let feature_pos = sorted.iter().position(|b| b == "feature/test").expect("feature not found");
    let zebra_pos = sorted.iter().position(|b| b == "zebra").expect("zebra not found");

    assert!(
        alpha_pos < feature_pos,
        "alpha should come before feature/test"
    );
    assert!(
        feature_pos < zebra_pos,
        "feature/test should come before zebra"
    );
    Ok(())
}

#[test]
fn test_sort_branches_duplicate_names() -> Result<()> {
    // Arrange: 准备测试重复分支名（边界情况）
    let branches = vec![
        "main".to_string(),
        "main".to_string(), // 重复
        "develop".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // 应该保留重复项（排序算法不会去重）
    assert!(sorted.len() >= 2, "Should preserve duplicate entries");
    assert_eq!(sorted[0], "main");
    Ok(())
}

#[test]
fn test_sort_branches_special_characters() -> Result<()> {
    // Arrange: 准备测试特殊字符分支名（边界情况）
    let branches = vec![
        "feature/test-branch".to_string(),
        "feature/test_branch".to_string(),
        "feature/test.branch".to_string(),
        "main".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches)?;

    // main 应该在第一位
    assert_eq!(sorted[0], "main");

    // 其他分支应该按字母顺序排序
    assert_eq!(sorted.len(), 4);
    Ok(())
}
