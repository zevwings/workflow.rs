//! Branch 同步测试
//!
//! 测试 Branch 同步相关的功能，包括：
//! - 同步策略
//! - 同步执行
//! - 回调处理

use pretty_assertions::assert_eq;
use workflow::branch::sync::{BranchSyncOptions, BranchSyncResult, SourceBranchInfo, SyncStrategy};

// ==================== 同步选项测试 ====================

#[test]
fn test_branch_sync_options() {
    // 测试 BranchSyncOptions 结构
    let options = BranchSyncOptions {
        source_branch: "feature".to_string(),
        rebase: false,
        ff_only: false,
        squash: false,
    };

    assert_eq!(options.source_branch, "feature");
    assert!(!options.rebase);
    assert!(!options.ff_only);
    assert!(!options.squash);
}

#[test]
fn test_branch_sync_options_rebase() {
    // 测试 rebase 选项
    let options = BranchSyncOptions {
        source_branch: "develop".to_string(),
        rebase: true,
        ff_only: false,
        squash: false,
    };

    assert!(options.rebase);
}

#[test]
fn test_branch_sync_options_squash() {
    // 测试 squash 选项
    let options = BranchSyncOptions {
        source_branch: "feature".to_string(),
        rebase: false,
        ff_only: false,
        squash: true,
    };

    assert!(options.squash);
}

// ==================== 源分支信息测试 ====================

#[test]
fn test_source_branch_info() {
    // 测试 SourceBranchInfo 结构
    let info = SourceBranchInfo {
        is_remote: false,
        merge_ref: "feature".to_string(),
    };

    assert!(!info.is_remote);
    assert_eq!(info.merge_ref, "feature");
}

#[test]
fn test_source_branch_info_remote() {
    // 测试远程分支信息
    let info = SourceBranchInfo {
        is_remote: true,
        merge_ref: "origin/feature".to_string(),
    };

    assert!(info.is_remote);
    assert_eq!(info.merge_ref, "origin/feature");
}

// ==================== 同步策略测试 ====================

#[test]
fn test_sync_strategy_merge() {
    // 测试 Merge 同步策略
    let strategy = SyncStrategy::Merge(workflow::git::MergeStrategy::Merge);

    // 验证策略类型
    match strategy {
        SyncStrategy::Merge(_) => assert!(true),
        _ => assert!(false, "Expected Merge strategy"),
    }
}

#[test]
fn test_sync_strategy_rebase() {
    // 测试 Rebase 同步策略
    let strategy = SyncStrategy::Rebase;

    match strategy {
        SyncStrategy::Rebase => assert!(true),
        _ => assert!(false, "Expected Rebase strategy"),
    }
}

// ==================== 同步结果测试 ====================

#[test]
fn test_branch_sync_result() {
    // 测试 BranchSyncResult 结构
    let source_info = SourceBranchInfo {
        is_remote: false,
        merge_ref: "feature".to_string(),
    };

    let result = BranchSyncResult {
        current_branch: "main".to_string(),
        source_branch_info: source_info.clone(),
        strategy: SyncStrategy::Merge(workflow::git::MergeStrategy::Merge),
        has_stashed: false,
        success: true,
    };

    assert_eq!(result.current_branch, "main");
    assert_eq!(result.source_branch_info.merge_ref, "feature");
    assert!(!result.has_stashed);
    assert!(result.success);
}

#[test]
fn test_branch_sync_result_with_stash() {
    // 测试包含 stash 的同步结果
    let source_info = SourceBranchInfo {
        is_remote: true,
        merge_ref: "origin/develop".to_string(),
    };

    let result = BranchSyncResult {
        current_branch: "feature".to_string(),
        source_branch_info: source_info,
        strategy: SyncStrategy::Rebase,
        has_stashed: true,
        success: true,
    };

    assert!(result.has_stashed);
    assert_eq!(result.current_branch, "feature");
}
