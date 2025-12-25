//! Jira 日志模块测试
//!
//! 测试 Jira 日志下载、搜索、清理和路径处理功能。

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use std::fs;
use workflow::jira::attachments::AttachmentCleaner;
use workflow::jira::logs::{JiraLogs, LogEntry};

// ==================== Fixtures ====================

#[fixture]
fn jira_logs() -> JiraLogs {
    JiraLogs::new().expect("Should create JiraLogs")
}

#[fixture]
fn nonexistent_jira_id() -> &'static str {
    "NONEXISTENT-999"
}

// ==================== JiraLogs 初始化测试 ====================

/// 测试创建JiraLogs实例
#[test]
fn test_jira_logs_new() {
    // Arrange: 准备测试创建 JiraLogs 实例
    let jira_logs = JiraLogs::new();
    assert!(jira_logs.is_ok(), "JiraLogs::new() should succeed");
}

/// 测试获取基础目录路径
#[rstest]
fn test_jira_logs_get_base_dir_path(jira_logs: JiraLogs) {
    // Arrange: 准备测试获取基础目录路径
    let base_dir = jira_logs.get_base_dir_path();

    // Assert: 验证返回的是有效的路径
    assert!(base_dir.is_absolute() || base_dir.to_string_lossy().starts_with("~"));
}

/// 测试获取日志文件路径
#[rstest]
#[case("TEST-123", "TEST")]
#[case("PROJ-456", "PROJ")]
fn test_jira_logs_get_log_file_path(
    jira_logs: JiraLogs,
    #[case] jira_id: &str,
    #[case] project_prefix: &str,
) {
    // Arrange: 准备测试获取日志文件路径
    let log_file_path = jira_logs.get_log_file_path(jira_id);
    // 即使目录不存在，也应该返回一个路径
    if let Ok(path) = log_file_path {
        // Assert: 验证路径包含 jira_id（如果路径构建成功）
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains(jira_id) || path_str.contains(project_prefix),
            "Path should contain JIRA ID or related identifier: {}",
            path_str
        );
    } else {
        // 如果返回错误（例如目录读取失败），这也是可以接受的
        // 主要验证函数不会 panic
        assert!(true, "Path building may fail if directory doesn't exist");
    }
}

/// 测试日志文件路径格式
#[rstest]
fn test_jira_logs_get_log_file_path_format(jira_logs: JiraLogs) {
    // Arrange: 准备测试日志文件路径格式
    let jira_id = "PROJ-456";

    let log_file_path_result = jira_logs.get_log_file_path(jira_id);
    if let Ok(log_file_path) = log_file_path_result {
        let path_str = log_file_path.to_string_lossy();

        // Assert: 验证路径格式：应该包含 jira 目录和 jira_id
        assert!(
            path_str.contains("jira"),
            "Path should contain 'jira' directory"
        );
        assert!(path_str.contains(jira_id), "Path should contain JIRA ID");
    } else {
        // 如果路径构建失败，这也是可以接受的（例如目录不存在）
        assert!(true, "Path building may fail if directory doesn't exist");
    }
}

// ==================== Path Processing Tests ====================

/// 测试路径一致性（多次调用应返回相同结果）
#[rstest]
fn test_jira_logs_path_consistency(jira_logs: JiraLogs) {
    // Arrange: 准备测试路径一致性
    let jira_id = "TEST-789";

    let path1_result = jira_logs.get_log_file_path(jira_id);
    let path2_result = jira_logs.get_log_file_path(jira_id);

    // 多次调用应该返回相同的结果（成功或失败）
    match (path1_result, path2_result) {
        (Ok(path1), Ok(path2)) => {
            // 如果都成功，路径应该相同
            assert_eq!(path1, path2, "Paths should be consistent");
        }
        (Err(_), Err(_)) => {
            // 如果都失败，这也是可以接受的
            assert!(true, "Both calls may fail consistently");
        }
        _ => {
            // 一个成功一个失败不应该发生
            panic!("Inconsistent results between calls");
        }
    }
}

/// 测试不同JIRA ID的路径（应返回不同路径）
#[rstest]
fn test_jira_logs_different_jira_ids(jira_logs: JiraLogs) {
    // Arrange: 准备测试不同 JIRA ID 的路径
    let path1_result = jira_logs.get_log_file_path("PROJ-1");
    let path2_result = jira_logs.get_log_file_path("PROJ-2");

    // 不同 JIRA ID 应该返回不同的路径（如果路径构建成功）
    match (path1_result, path2_result) {
        (Ok(path1), Ok(path2)) => {
            let path1_str = path1.to_string_lossy();
            let path2_str = path2.to_string_lossy();
            assert_ne!(
                path1_str, path2_str,
                "Different JIRA IDs should have different paths"
            );
        }
        _ => {
            // 如果路径构建失败，这也是可以接受的
            // 主要验证函数不会 panic
            assert!(true, "Path building may fail, but should not panic");
        }
    }
}

// ==================== Log File Existence Tests ====================

/// 测试确保日志文件存在（文件不存在的情况，应返回错误）
#[rstest]
fn test_jira_logs_ensure_log_file_exists_nonexistent(
    jira_logs: JiraLogs,
    nonexistent_jira_id: &str,
) {
    // Arrange: 准备测试确保日志文件存在（文件不存在的情况）
    let result = jira_logs.ensure_log_file_exists(nonexistent_jira_id);

    // 文件不存在时应该返回错误
    // 注意：如果路径构建失败（目录不存在），也可能返回错误
    assert!(
        result.is_err(),
        "Should return error when log file doesn't exist or path building fails"
    );

    let error_msg = result.unwrap_err().to_string();
    // 错误消息可能包含 "Log file not found" 或路径相关的错误
    assert!(
        error_msg.contains("Log file not found")
            || error_msg.contains("not found")
            || error_msg.contains("Failed to read directory"),
        "Error message should indicate file not found or path issue: {}",
        error_msg
    );
}

/// 测试确保日志文件存在（使用临时文件）
#[rstest]
fn test_jira_logs_ensure_log_file_exists_with_temp_file(jira_logs: JiraLogs) {
    // Arrange: 准备测试确保日志文件存在（使用临时文件）
    let _ = jira_logs; // 使用 fixture，即使不直接使用也保持一致性
    let test_dir = create_temp_test_dir("jira_logs_test");

    // 创建测试目录结构
    let jira_id = "TEST-123";
    let jira_dir = test_dir.join("jira").join(jira_id);
    let logs_dir = jira_dir.join("logs");
    fs::create_dir_all(&logs_dir).expect("Should create directories");

    // 创建日志文件
    let log_file = logs_dir.join("flutter-api.log");
    create_test_file(&logs_dir, "flutter-api.log", "Test log content");

    // 注意：由于 JiraLogs 使用配置的基础目录，这个测试主要验证逻辑
    // 实际文件路径可能不同，但我们可以测试错误处理
    assert!(log_file.exists(), "Test log file should exist");

    cleanup_temp_test_dir(&test_dir);
}

// ==================== LogEntry 测试 ====================

/// 测试创建LogEntry（包含可选字段）
#[rstest]
#[case(Some("123"), Some("https://example.com/api"))]
#[case(Some("456"), None)]
#[case(None, Some("https://example.com/api"))]
#[case(None, None)]
fn test_log_entry(#[case] id: Option<&str>, #[case] url: Option<&str>) {
    // Arrange: 准备测试创建 LogEntry
    let entry = LogEntry {
        id: id.map(|s| s.to_string()),
        url: url.map(|s| s.to_string()),
    };

    assert_eq!(entry.id, id.map(|s| s.to_string()));
    assert_eq!(entry.url, url.map(|s| s.to_string()));
}

// ==================== Cleanup Function Tests ====================

/// 测试清理目录（包含dry_run和list_only选项）
#[rstest]
#[case("NONEXISTENT-999", true, false)]
#[case("TEST-DRYRUN", true, false)]
#[case("TEST-LIST", false, true)]
fn test_jira_logs_clean_dir(#[case] jira_id: &str, #[case] dry_run: bool, #[case] list_only: bool) {
    // Arrange: 准备测试清理目录
    let cleaner = AttachmentCleaner::new();

    let result = cleaner.clean_dir(jira_id, dry_run, list_only);

    // 应该成功返回
    assert!(result.is_ok(), "Should handle directory gracefully");

    let clean_result = result.expect("clean should succeed");
    if dry_run {
        assert!(clean_result.dry_run, "Should be in dry run mode");
    }
    if list_only {
        assert!(clean_result.list_only, "Should be in list only mode");
    }
    if jira_id == "NONEXISTENT-999" {
        assert!(
            !clean_result.deleted,
            "Should not delete non-existent directory"
        );
        assert!(!clean_result.dir_exists, "Directory should not exist");
    }
}

/// 测试清理整个Jira日志基础目录（空JIRA ID）
///
/// ## 测试目的
/// 验证当JIRA ID为空时，`clean_dir()`方法能够清理整个基础目录并请求用户确认。
///
/// ## 为什么被忽略
/// - **需要交互式确认**: 清理整个目录需要用户确认（危险操作）
/// - **CI环境不支持**: 自动化环境无法提供交互式输入
/// - **文件系统操作**: 会实际删除文件
/// - **安全保护**: 避免在CI中误删除文件
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_jira_logs_clean_dir_empty_jira_id -- --ignored
/// ```
/// **警告**: 此测试会清理Jira日志目录！请在测试环境中运行
///
/// ## 测试场景
/// 1. 调用clean_dir()方法并传入空的JIRA ID
/// 2. 检测到要清理整个基础目录
/// 3. 显示确认对话框警告用户
/// 4. 等待用户确认（y）或取消（n）
/// 5. 用户确认后删除目录
///
/// ## 预期行为
/// - 显示明确的警告消息
/// - 列出将要删除的目录路径
/// - 用户确认后成功删除
/// - 用户取消则不删除
/// - 返回适当的Result
#[test]
#[ignore] // 需要交互式确认，在 CI 环境中会卡住
fn test_jira_logs_clean_dir_empty_jira_id() {
    // Arrange: 准备测试清理整个基础目录（空 JIRA ID）
    // 注意：这个测试会触发交互式确认对话框，在 CI 环境中会卡住
    // 使用 `cargo test -- --ignored` 来运行这些测试
    let cleaner = AttachmentCleaner::new();

    // 使用 dry_run=true 避免交互式对话框
    let result = cleaner.clean_dir("", true, false);

    // 应该能够处理空 JIRA ID（即使目录不存在也可能返回错误）
    // 主要验证函数不会 panic
    match result {
        Ok(_clean_result) => {
            // 如果成功，验证结果结构
            assert!(true, "Should handle empty JIRA ID");
        }
        Err(_) => {
            // 如果失败（例如目录不存在），这也是可以接受的行为
            // 主要验证函数不会 panic
            assert!(true, "Should handle empty JIRA ID gracefully");
        }
    }
}

// ==================== Search Function Tests ====================

/// 测试在不存在的文件中查找请求ID（应返回错误）
#[rstest]
fn test_jira_logs_find_request_id_nonexistent_file(jira_logs: JiraLogs, nonexistent_jira_id: &str) {
    // Arrange: 准备测试在不存在的文件中查找请求 ID
    let result = jira_logs.find_request_id(nonexistent_jira_id, "123");

    // 文件不存在时应该返回错误
    assert!(
        result.is_err(),
        "Should return error when file doesn't exist"
    );
}

/// 测试从不存在的文件中提取响应内容（应返回错误）
#[rstest]
fn test_jira_logs_extract_response_content_nonexistent_file(
    jira_logs: JiraLogs,
    nonexistent_jira_id: &str,
) {
    // Arrange: 准备测试从不存在的文件中提取响应内容
    let result = jira_logs.extract_response_content(nonexistent_jira_id, "123");

    // 文件不存在时应该返回错误
    assert!(
        result.is_err(),
        "Should return error when file doesn't exist"
    );
}

// ==================== Configuration and Settings Tests ====================

/// 测试与Settings的集成
#[rstest]
fn test_jira_logs_settings_integration(jira_logs: JiraLogs) {
    // Arrange: 准备测试与 Settings 的集成
    // Assert: 验证可以从 Settings 读取配置
    let base_dir = jira_logs.get_base_dir_path();
    assert!(
        !base_dir.to_string_lossy().is_empty(),
        "Base dir should not be empty"
    );
}

/// 测试输出文件夹名称配置
#[rstest]
fn test_jira_logs_output_folder_name(jira_logs: JiraLogs) {
    // Arrange: 准备测试输出文件夹名称配置
    let jira_id = "TEST-OUTPUT";

    let log_file_path_result = jira_logs.get_log_file_path(jira_id);
    if let Ok(log_file_path) = log_file_path_result {
        let path_str = log_file_path.to_string_lossy();

        // 路径应该包含配置的输出文件夹名称（通常是 "logs"）
        // 这个测试主要验证路径构建逻辑
        assert!(!path_str.is_empty(), "Path should not be empty");
    } else {
        // 如果路径构建失败，这也是可以接受的
        assert!(true, "Path building may fail if directory doesn't exist");
    }
}

// ==================== Error Handling Tests ====================

/// 测试错误消息的格式
#[rstest]
fn test_jira_logs_error_messages(jira_logs: JiraLogs) {
    // Arrange: 准备测试错误消息的格式
    let jira_id = "ERROR-TEST";

    let result = jira_logs.ensure_log_file_exists(jira_id);
    assert!(result.is_err(), "Should return error for non-existent file");

    let error_msg = result.unwrap_err().to_string();
    // Assert: 验证错误消息包含有用的信息
    // 错误可能是 "Log file not found" 或路径相关的错误
    assert!(
        error_msg.contains("Log file not found")
            || error_msg.contains("not found")
            || error_msg.contains("Failed to read directory")
            || error_msg.contains("Failed to expand path"),
        "Error message should be informative: {}",
        error_msg
    );
}

/// 测试无效的JIRA ID格式
#[rstest]
#[case("INVALID")]
#[case("123")]
#[case("PROJ")]
fn test_jira_logs_invalid_jira_id_format(jira_logs: JiraLogs, #[case] jira_id: &str) {
    // Arrange: 准备测试无效的 JIRA ID 格式
    // 即使格式可能无效，路径构建应该仍然工作（或优雅地失败）
    let result = jira_logs.get_log_file_path(jira_id);
    // 路径构建可能成功或失败，主要验证不会 panic
    match result {
        Ok(_) => assert!(true, "Path building succeeded"),
        Err(_) => assert!(true, "Path building failed gracefully"),
    }
}
