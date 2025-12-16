//! Rollback 模块测试
//!
//! 测试 Rollback 相关的功能，包括：
//! - 备份功能
//! - 恢复功能
//! - 清理功能

use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;
use crate::common::helpers::{
    cleanup_temp_test_dir, create_temp_test_dir, create_test_file,
};

use workflow::rollback::rollback::{BackupInfo, RollbackResult};

// ==================== 备份信息测试 ====================

// 注意：BackupInfo::new 和相关方法是私有的，不能直接测试
// 这些测试需要通过公共 API 间接测试，或者移除

// ==================== 备份目录创建测试 ====================

#[test]
fn test_create_backup_dir() {
    // 测试创建备份目录（基本概念测试）
    let temp_dir = std::env::temp_dir();
    let backup_dir = temp_dir.join(format!(
        "workflow-backup-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    ));

    fs::create_dir_all(&backup_dir).unwrap();
    assert!(backup_dir.exists());
    assert!(backup_dir.is_dir());

    // 清理
    fs::remove_dir_all(&backup_dir).ok();
}

// ==================== Rollback 结果测试 ====================

#[test]
fn test_rollback_result() {
    // 测试 RollbackResult 结构
    let result = RollbackResult {
        restored_binaries: vec!["workflow".to_string()],
        restored_completions: vec!["workflow.bash".to_string()],
        failed_binaries: vec![],
        failed_completions: vec![],
        shell_reload_success: Some(true),
        shell_config_file: None,
    };

    assert_eq!(result.restored_binaries.len(), 1);
    assert_eq!(result.restored_completions.len(), 1);
    assert!(result.failed_binaries.is_empty());
    assert!(result.failed_completions.is_empty());
    assert_eq!(result.shell_reload_success, Some(true));
}

#[test]
fn test_rollback_result_with_failures() {
    // 测试包含失败项的 RollbackResult
    let result = RollbackResult {
        restored_binaries: vec!["workflow".to_string()],
        restored_completions: vec![],
        failed_binaries: vec![("workflow2".to_string(), "File not found".to_string())],
        failed_completions: vec![("workflow.zsh".to_string(), "Permission denied".to_string())],
        shell_reload_success: Some(false),
        shell_config_file: None,
    };

    assert_eq!(result.restored_binaries.len(), 1);
    assert!(result.restored_completions.is_empty());
    assert_eq!(result.failed_binaries.len(), 1);
    assert_eq!(result.failed_completions.len(), 1);
    assert_eq!(result.shell_reload_success, Some(false));
}

// ==================== 备份结果测试 ====================

#[test]
fn test_backup_result() {
    // 测试 BackupResult 结构
    // 注意：BackupInfo::new 是私有的，这里只测试结构体的基本概念
    let test_dir = create_temp_test_dir("backup_result_test");

    // 创建 BackupInfo 的简化版本用于测试
    let backup_info = BackupInfo {
        backup_dir: test_dir.clone(),
        binary_backups: vec![],
        completion_backups: vec![],
    };

    let result = workflow::rollback::rollback::BackupResult {
        backup_info: backup_info.clone(),
        binary_count: 1,
        completion_count: 2,
    };

    assert_eq!(result.binary_count, 1);
    assert_eq!(result.completion_count, 2);
    assert_eq!(result.backup_info.backup_dir, test_dir);

    cleanup_temp_test_dir(&test_dir);
}
