//! Branch 命令辅助函数测试
//!
//! 测试分支命令的辅助函数，包括分支排序、优先级计算等。

use pretty_assertions::assert_eq;
use workflow::commands::branch::helpers::sort_branches_with_priority;

// 注意: get_branch_priority 是私有函数，我们需要通过公共 API 测试其效果

#[test]
fn test_sort_branches_with_priority_main_first() {
    let branches = vec![
        "feature/test".to_string(),
        "main".to_string(),
        "develop".to_string(),
        "hotfix/fix".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches).unwrap();

    // main 应该在第一位
    assert_eq!(sorted[0], "main");
}

#[test]
fn test_sort_branches_with_priority_master_after_main() {
    let branches = vec![
        "master".to_string(),
        "main".to_string(),
        "feature/test".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches).unwrap();

    // main 应该在 master 之前
    let main_pos = sorted.iter().position(|b| b == "main").unwrap();
    let master_pos = sorted.iter().position(|b| b == "master").unwrap();
    assert!(main_pos < master_pos);
}

#[test]
fn test_sort_branches_with_priority_develop_second() {
    let branches = vec![
        "feature/test".to_string(),
        "develop".to_string(),
        "hotfix/fix".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches).unwrap();

    // develop 应该在 feature 和 hotfix 之前
    let develop_pos = sorted.iter().position(|b| b == "develop").unwrap();
    let feature_pos = sorted.iter().position(|b| b == "feature/test").unwrap();
    assert!(develop_pos < feature_pos);
}

#[test]
fn test_sort_branches_with_priority_alphabetical_others() {
    let branches = vec![
        "zebra".to_string(),
        "alpha".to_string(),
        "beta".to_string(),
    ];

    let sorted = sort_branches_with_priority(branches).unwrap();

    // 其他分支应该按字母顺序排序
    assert_eq!(sorted[0], "alpha");
    assert_eq!(sorted[1], "beta");
    assert_eq!(sorted[2], "zebra");
}

#[test]
fn test_sort_branches_empty() {
    let branches = vec![];
    let sorted = sort_branches_with_priority(branches).unwrap();
    assert!(sorted.is_empty());
}

#[test]
fn test_sort_branches_single() {
    let branches = vec!["feature/test".to_string()];
    let sorted = sort_branches_with_priority(branches).unwrap();
    assert_eq!(sorted.len(), 1);
    assert_eq!(sorted[0], "feature/test");
}

