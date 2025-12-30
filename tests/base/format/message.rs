//! Base Format MessageFormatter 模块测试
//!
//! 测试 MessageFormatter 模块的公共 API，包括：
//! - 错误消息格式化
//! - 操作消息格式化
//! - 进度信息格式化

use pretty_assertions::assert_eq;
use workflow::base::format::MessageFormatter;

// ==================== Error Message Formatting Tests ====================

/// 测试错误消息格式化功能
///
/// ## 测试目的
/// 验证 `MessageFormatter::error()` 能够正确格式化错误消息。
///
/// ## 测试场景
/// 1. 准备操作名称、目标和错误信息
/// 2. 调用 error() 方法格式化消息
/// 3. 验证格式化结果
///
/// ## 预期结果
/// - 格式化结果符合 "Failed to {operation} {target}: {error}" 格式
#[test]
fn test_error_formatting() {
    // Arrange: 准备输入参数
    let operation = "read";
    let target = "config.toml";
    let error = "Permission denied";

    // Act: 格式化错误消息
    let msg = MessageFormatter::error(operation, target, error);

    // Assert: 验证格式化结果
    assert_eq!(msg, "Failed to read config.toml: Permission denied");
}

// ==================== Operation Message Formatting Tests ====================

/// 测试操作消息格式化功能
///
/// ## 测试目的
/// 验证 `MessageFormatter::operation()` 能够正确格式化操作消息。
///
/// ## 测试场景
/// 1. 准备动作名称和目标
/// 2. 调用 operation() 方法格式化消息
/// 3. 验证格式化结果
///
/// ## 预期结果
/// - 格式化结果符合 "{action} {target}..." 格式
#[test]
fn test_operation_formatting() {
    // Arrange: 准备输入参数
    let action = "Creating";
    let target = "new branch";

    // Act: 格式化操作消息
    let msg = MessageFormatter::operation(action, target);

    // Assert: 验证格式化结果
    assert_eq!(msg, "Creating new branch...");
}

// ==================== Progress Message Formatting Tests ====================

/// 测试进度信息格式化功能
///
/// ## 测试目的
/// 验证 `MessageFormatter::progress()` 能够正确格式化进度信息。
///
/// ## 测试场景
/// 1. 准备当前进度、总进度和项目名称
/// 2. 调用 progress() 方法格式化消息
/// 3. 验证格式化结果
///
/// ## 预期结果
/// - 格式化结果符合 "[current/total] Processing item" 格式
#[test]
fn test_progress_formatting() {
    // Arrange: 准备输入参数
    let current = 3;
    let total = 10;
    let item = "files";

    // Act: 格式化进度信息
    let msg = MessageFormatter::progress(current, total, item);

    // Assert: 验证格式化结果
    assert_eq!(msg, "[3/10] Processing files");
}

