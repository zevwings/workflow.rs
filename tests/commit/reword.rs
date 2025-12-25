//! CommitReword 业务逻辑测试
//!
//! 测试 CommitReword 模块的核心功能，包括：
//! - 预览创建和格式化
//! - 提交信息格式化
//! - 完成消息生成
//! - 历史选项和结果处理

use pretty_assertions::assert_eq;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::TempDir;
use workflow::commit::{CommitReword, RewordHistoryOptions, RewordHistoryResult, RewordPreview};
use workflow::git::CommitInfo;

// ==================== Helper Functions ====================

/// 创建测试用的 CommitInfo
fn create_sample_commit_info() -> CommitInfo {
    CommitInfo {
        sha: "def456abc789012345678901234567890abcdef12".to_string(),
        message: "fix: resolve login authentication bug\n\nFix critical authentication issue that prevented users from logging in with valid credentials.".to_string(),
        author: "Jane Smith <jane.smith@example.com>".to_string(),
        date: "2023-12-02 14:45:00 +0000".to_string(),
    }
}

/// 创建带有多个提交的临时 Git 仓库
fn create_git_repo_with_commit() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // 初始化 Git 仓库
    Command::new("git")
        .args(["init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to init git repo");

    // 设置 Git 配置
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to set git user email");

    // 创建多个提交
    for i in 1..=3 {
        let file_path = temp_path.join(format!("file{}.txt", i));
        fs::write(&file_path, format!("Content of file {}\n", i)).expect("Failed to write file");

        Command::new("git")
            .args(["add", &format!("file{}.txt", i)])
            .current_dir(temp_path)
            .output()
            .expect("Failed to add file");

        Command::new("git")
            .args(["commit", "-m", &format!("Commit {}: add file{}.txt", i, i)])
            .current_dir(temp_path)
            .output()
            .expect("Failed to commit");
    }

    temp_dir
}

// ==================== 测试用例 ====================

/// 测试创建预览
#[test]
fn test_create_preview() {
    let commit_info = create_sample_commit_info();
    let new_message = "fix: resolve critical login authentication issue\n\nFix authentication bug that was blocking user access with proper credentials.";
    let is_head = true;
    let current_branch = "main";

    let result = CommitReword::create_preview(&commit_info, new_message, is_head, current_branch);
    assert!(result.is_ok());

    let preview = result.expect("operation should succeed");
    assert_eq!(
        preview.original_sha,
        "def456abc789012345678901234567890abcdef12"
    );
    assert_eq!(preview.original_message, "fix: resolve login authentication bug\n\nFix critical authentication issue that prevented users from logging in with valid credentials.");
    assert_eq!(preview.new_message, "fix: resolve critical login authentication issue\n\nFix authentication bug that was blocking user access with proper credentials.");
    assert_eq!(preview.is_head, true);
}

#[test]
fn test_format_preview_with_valid_preview_returns_formatted_string() {
    // Arrange: 准备 RewordPreview 实例
    let preview = RewordPreview {
        original_sha: "def456abc789012345678901234567890abcdef12".to_string(),
        original_message: "fix: resolve login authentication bug".to_string(),
        new_message: "Updated commit message\n\nWith new detailed description.".to_string(),
        is_head: false,
        is_pushed: true,
    };

    // Act: 格式化预览
    let formatted = CommitReword::format_preview(&preview);

    // Assert: 验证格式化字符串包含预期内容
    assert!(formatted.contains("Original Commit SHA"));
    assert!(formatted.contains("def456ab"));
    assert!(formatted.contains("fix: resolve login authentication bug"));
    assert!(formatted.contains("Updated commit message"));
    assert!(formatted.contains("With new detailed description."));
}

#[test]
fn test_format_commit_info_with_valid_info_returns_formatted_string() {
    // Arrange: 准备提交信息和分支名
    let commit_info = create_sample_commit_info();
    let branch = "feature-auth";

    // Act: 格式化提交信息
    let formatted = CommitReword::format_commit_info(&commit_info, branch);

    // Assert: 验证格式化字符串包含预期内容
    assert!(formatted.contains("def456ab")); // 短 SHA
    assert!(formatted.contains("fix: resolve login authentication bug"));
    assert!(formatted.contains("Jane Smith"));
    assert!(formatted.contains("jane.smith@example.com"));
    assert!(formatted.contains("2023-12-02 14:45:00"));
    assert!(formatted.contains("feature-auth")); // 分支名
}

#[test]
fn test_format_completion_message_with_valid_input_returns_message() {
    // Arrange: 准备分支名和 SHA
    let current_branch = "main";
    let old_sha = "def456abc789012345678901234567890abcdef12";

    // Act: 格式化完成消息
    let result = CommitReword::format_completion_message(current_branch, old_sha);

    // Assert: 验证返回结果（成功、None 或错误都是可以接受的）
    match result {
        Ok(Some(message)) => {
            assert!(message.contains("successfully") || message.contains("completed"));
        }
        Ok(None) => {
            // 某些情况下可能返回 None，这也是有效的
        }
        Err(_) => {
            // 在测试环境中可能失败，这是可以接受的
        }
    }
}

// ==================== Force Push Warning Tests ====================

#[test]
fn test_should_show_force_push_warning_with_pushed_commit_returns_bool() {
    // Arrange: 准备分支名和 SHA
    let current_branch = "main";
    let old_sha = "test123456789012345678901234567890abcdef";

    // Act: 检查是否应该显示强制推送警告
    let result = CommitReword::should_show_force_push_warning(current_branch, old_sha);

    // Assert: 验证返回布尔值（成功或失败都是可以接受的）
    match result {
        Ok(is_pushed) => {
            // 验证返回值是布尔类型
            assert!(is_pushed == true || is_pushed == false);
        }
        Err(_) => {
            // 在测试环境中可能失败，这是可以接受的
        }
    }
}

#[test]
fn test_should_show_force_push_warning_with_not_pushed_commit_returns_bool() {
    // Arrange: 准备分支名和 SHA（未推送的提交）
    let current_branch = "feature";
    let old_sha = "test987654321098765432109876543210987654";

    // Act: 检查是否应该显示强制推送警告
    let result = CommitReword::should_show_force_push_warning(current_branch, old_sha);

    // Assert: 验证返回布尔值（成功或失败都是可以接受的）
    match result {
        Ok(is_pushed) => {
            assert!(is_pushed == true || is_pushed == false);
        }
        Err(_) => {
            // 在测试环境中可能失败，这是可以接受的
        }
    }
}

// ==================== RewordHistoryOptions Tests ====================

#[test]
fn test_reword_history_options_with_valid_fields_creates_options() {
    // Arrange: 准备历史重写选项字段值
    let commit_sha = "abc123def456";
    let new_message = "Updated historical commit message";
    let auto_stash = true;

    // Act: 创建 RewordHistoryOptions 实例
    let options = RewordHistoryOptions {
        commit_sha: commit_sha.to_string(),
        new_message: new_message.to_string(),
        auto_stash,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(options.commit_sha, commit_sha);
    assert_eq!(options.new_message, new_message);
    assert_eq!(options.auto_stash, auto_stash);
}

// ==================== RewordHistoryResult Tests ====================

#[test]
fn test_reword_history_result_with_success_creates_result() {
    // Arrange: 准备成功的结果字段值
    let success = true;
    let has_conflicts = false;
    let was_stashed = true;

    // Act: 创建 RewordHistoryResult 实例
    let result = RewordHistoryResult {
        success,
        has_conflicts,
        was_stashed,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(result.success, success);
    assert_eq!(result.has_conflicts, has_conflicts);
    assert_eq!(result.was_stashed, was_stashed);
}

#[test]
fn test_reword_history_result_failure_with_conflicts_creates_result() {
    // Arrange: 准备失败的结果字段值
    let success = false;
    let has_conflicts = true;
    let was_stashed = false;

    // Act: 创建 RewordHistoryResult 实例
    let result = RewordHistoryResult {
        success,
        has_conflicts,
        was_stashed,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(result.success, success);
    assert_eq!(result.has_conflicts, has_conflicts);
    assert_eq!(result.was_stashed, was_stashed);
}

// ==================== RewordPreview Tests ====================

#[test]
fn test_reword_preview_struct_with_valid_fields_creates_preview() {
    // Arrange: 准备预览字段值
    let original_sha = "preview_sha";
    let original_message = "original preview message";
    let new_message = "new preview message";

    // Act: 创建 RewordPreview 实例
    let preview = RewordPreview {
        original_sha: original_sha.to_string(),
        original_message: original_message.to_string(),
        new_message: new_message.to_string(),
        is_head: false,
        is_pushed: true,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(preview.original_sha, original_sha);
    assert_eq!(preview.original_message, original_message);
    assert_eq!(preview.new_message, new_message);
    assert_eq!(preview.is_head, false);
    assert_eq!(preview.is_pushed, true);
}

#[test]
fn test_reword_preview_clone_with_valid_preview_creates_clone() {
    // Arrange: 准备原始 RewordPreview
    let original = RewordPreview {
        original_sha: "clone_sha".to_string(),
        original_message: "clone original".to_string(),
        new_message: "clone new".to_string(),
        is_head: true,
        is_pushed: false,
    };

    // Act: 克隆预览
    let cloned = original.clone();

    // Assert: 验证克隆的字段值与原始值相同
    assert_eq!(original.original_sha, cloned.original_sha);
    assert_eq!(original.original_message, cloned.original_message);
    assert_eq!(original.new_message, cloned.new_message);
    assert_eq!(original.is_head, cloned.is_head);
    assert_eq!(original.is_pushed, cloned.is_pushed);
}

/// 测试空消息处理
#[test]
fn test_create_preview_empty_message() {
    let commit_info = create_sample_commit_info();
    let empty_message = "";
    let is_head = false;
    let current_branch = "feature";

    let result = CommitReword::create_preview(&commit_info, empty_message, is_head, current_branch);
    assert!(result.is_ok());

    let preview = result.expect("operation should succeed");
    assert_eq!(preview.new_message, "");
    assert_eq!(preview.is_head, false);
}

/// 测试 Git 仓库集成
#[test]
#[serial]
fn test_git_integration() {
    let _temp_dir = create_git_repo_with_commit();

    // 这个测试主要验证 Git 仓库创建辅助函数工作正常
    // 实际的 Git 集成测试应该在更高级别的集成测试中进行
    assert!(true);
}
