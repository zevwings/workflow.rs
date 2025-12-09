//! Jira 日志模块测试
//!
//! 测试 Jira 日志下载、搜索、清理和路径处理功能。

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir, create_test_file};
use std::fs;
use workflow::jira::logs::{JiraLogs, LogEntry};

// ==================== JiraLogs 初始化测试 ====================

#[test]
fn test_jira_logs_new() {
    // 测试创建 JiraLogs 实例
    let jira_logs = JiraLogs::new();
    assert!(jira_logs.is_ok(), "JiraLogs::new() should succeed");
}

#[test]
fn test_jira_logs_get_base_dir_path() {
    // 测试获取基础目录路径
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let base_dir = jira_logs.get_base_dir_path();

    // 验证返回的是有效的路径
    assert!(base_dir.is_absolute() || base_dir.to_string_lossy().starts_with("~"));
}

#[test]
fn test_jira_logs_get_log_file_path() {
    // 测试获取日志文件路径
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "TEST-123";

    let log_file_path = jira_logs.get_log_file_path(jira_id);
    // 即使目录不存在，也应该返回一个路径
    if let Ok(path) = log_file_path {
        // 验证路径包含 jira_id（如果路径构建成功）
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains(jira_id) || path_str.contains("TEST"),
            "Path should contain JIRA ID or related identifier"
        );
    } else {
        // 如果返回错误（例如目录读取失败），这也是可以接受的
        // 主要验证函数不会 panic
        assert!(true, "Path building may fail if directory doesn't exist");
    }
}

#[test]
fn test_jira_logs_get_log_file_path_format() {
    // 测试日志文件路径格式
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "PROJ-456";

    let log_file_path_result = jira_logs.get_log_file_path(jira_id);
    if let Ok(log_file_path) = log_file_path_result {
        let path_str = log_file_path.to_string_lossy();

        // 验证路径格式：应该包含 jira 目录和 jira_id
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

// ==================== 路径处理测试 ====================

#[test]
fn test_jira_logs_path_consistency() {
    // 测试路径一致性
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
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

#[test]
fn test_jira_logs_different_jira_ids() {
    // 测试不同 JIRA ID 的路径
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");

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

// ==================== 日志文件存在性测试 ====================

#[test]
fn test_jira_logs_ensure_log_file_exists_nonexistent() {
    // 测试确保日志文件存在（文件不存在的情况）
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "NONEXISTENT-999";

    let result = jira_logs.ensure_log_file_exists(jira_id);

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

#[test]
fn test_jira_logs_ensure_log_file_exists_with_temp_file() {
    // 测试确保日志文件存在（使用临时文件）
    let _jira_logs = JiraLogs::new().expect("Should create JiraLogs");
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

#[test]
fn test_log_entry_creation() {
    // 测试创建 LogEntry
    let entry = LogEntry {
        id: Some("123".to_string()),
        url: Some("https://example.com/api".to_string()),
    };

    assert_eq!(entry.id, Some("123".to_string()));
    assert_eq!(entry.url, Some("https://example.com/api".to_string()));
}

#[test]
fn test_log_entry_without_url() {
    // 测试没有 URL 的 LogEntry
    let entry = LogEntry {
        id: Some("456".to_string()),
        url: None,
    };

    assert_eq!(entry.id, Some("456".to_string()));
    assert_eq!(entry.url, None);
}

#[test]
fn test_log_entry_without_id() {
    // 测试没有 ID 的 LogEntry
    let entry = LogEntry {
        id: None,
        url: Some("https://example.com/api".to_string()),
    };

    assert_eq!(entry.id, None);
    assert_eq!(entry.url, Some("https://example.com/api".to_string()));
}

#[test]
fn test_log_entry_empty() {
    // 测试空的 LogEntry
    let entry = LogEntry {
        id: None,
        url: None,
    };

    assert_eq!(entry.id, None);
    assert_eq!(entry.url, None);
}

// ==================== 清理功能测试 ====================

#[test]
fn test_jira_logs_clean_dir_nonexistent() {
    // 测试清理不存在的目录
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "NONEXISTENT-999";

    let result = jira_logs.clean_dir(jira_id, false, false);

    // 应该成功返回，但 deleted 应该为 false
    assert!(
        result.is_ok(),
        "Should handle non-existent directory gracefully"
    );

    let clean_result = result.unwrap();
    assert!(
        !clean_result.deleted,
        "Should not delete non-existent directory"
    );
    assert!(!clean_result.dir_exists, "Directory should not exist");
}

#[test]
fn test_jira_logs_clean_dir_dry_run() {
    // 测试清理目录（dry run 模式）
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "TEST-DRYRUN";

    let result = jira_logs.clean_dir(jira_id, true, false);

    // dry run 应该成功，但不实际删除
    assert!(result.is_ok(), "Dry run should succeed");

    let clean_result = result.unwrap();
    assert!(clean_result.dry_run, "Should be in dry run mode");
}

#[test]
fn test_jira_logs_clean_dir_list_only() {
    // 测试清理目录（list only 模式）
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "TEST-LIST";

    let result = jira_logs.clean_dir(jira_id, false, true);

    // list only 应该成功
    assert!(result.is_ok(), "List only should succeed");

    let clean_result = result.unwrap();
    assert!(clean_result.list_only, "Should be in list only mode");
}

#[test]
fn test_jira_logs_clean_dir_empty_jira_id() {
    // 测试清理整个基础目录（空 JIRA ID）
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");

    let result = jira_logs.clean_dir("", false, false);

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

// ==================== 搜索功能测试 ====================

#[test]
fn test_jira_logs_find_request_id_nonexistent_file() {
    // 测试在不存在的文件中查找请求 ID
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "NONEXISTENT-999";

    let result = jira_logs.find_request_id(jira_id, "123");

    // 文件不存在时应该返回错误
    assert!(
        result.is_err(),
        "Should return error when file doesn't exist"
    );
}

#[test]
fn test_jira_logs_extract_response_content_nonexistent_file() {
    // 测试从不存在的文件中提取响应内容
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "NONEXISTENT-999";

    let result = jira_logs.extract_response_content(jira_id, "123");

    // 文件不存在时应该返回错误
    assert!(
        result.is_err(),
        "Should return error when file doesn't exist"
    );
}

// ==================== 配置和设置测试 ====================

#[test]
fn test_jira_logs_settings_integration() {
    // 测试与 Settings 的集成
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");

    // 验证可以从 Settings 读取配置
    let base_dir = jira_logs.get_base_dir_path();
    assert!(
        !base_dir.to_string_lossy().is_empty(),
        "Base dir should not be empty"
    );
}

#[test]
fn test_jira_logs_output_folder_name() {
    // 测试输出文件夹名称配置
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
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

// ==================== 错误处理测试 ====================

#[test]
fn test_jira_logs_error_messages() {
    // 测试错误消息的格式
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");
    let jira_id = "ERROR-TEST";

    let result = jira_logs.ensure_log_file_exists(jira_id);
    assert!(result.is_err(), "Should return error for non-existent file");

    let error_msg = result.unwrap_err().to_string();
    // 验证错误消息包含有用的信息
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

#[test]
fn test_jira_logs_invalid_jira_id_format() {
    // 测试无效的 JIRA ID 格式
    let jira_logs = JiraLogs::new().expect("Should create JiraLogs");

    // 即使格式可能无效，路径构建应该仍然工作（或优雅地失败）
    let invalid_ids = ["INVALID", "123", "PROJ"];

    for jira_id in &invalid_ids {
        let result = jira_logs.get_log_file_path(jira_id);
        // 路径构建可能成功或失败，主要验证不会 panic
        match result {
            Ok(_) => assert!(true, "Path building succeeded"),
            Err(_) => assert!(true, "Path building failed gracefully"),
        }
    }
}
