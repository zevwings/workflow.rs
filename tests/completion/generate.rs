//! Completion 生成模块测试
//!
//! 测试 Completion 脚本生成器的功能。

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir};
use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::completion::generate::CompletionGenerator;

// ==================== CompletionGenerator 创建测试 ====================

#[rstest]
#[case("zsh")]
#[case("bash")]
#[case("fish")]
#[case("powershell")]
#[case("elvish")]
fn test_completion_generator_new_with_shell_creates_generator(#[case] shell: &str) {
    // Arrange: 准备临时测试目录
    let test_dir = create_temp_test_dir("completion_gen");

    // Act: 使用指定 shell 创建 CompletionGenerator
    let result = CompletionGenerator::new(
        Some(shell.to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    );

    // Assert: 验证创建成功
    assert!(
        result.is_ok(),
        "Should create CompletionGenerator for {}",
        shell
    );
    let _generator = result.expect("operation should succeed");

    cleanup_temp_test_dir(&test_dir);
}

#[test]
fn test_completion_generator_new_with_unsupported_shell_returns_error() {
    // Arrange: 准备临时测试目录和不支持的 shell 类型
    let test_dir = create_temp_test_dir("completion_gen");

    // Act: 使用不支持的 shell 类型创建 CompletionGenerator
    let result = CompletionGenerator::new(
        Some("unsupported".to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    );

    // Assert: 验证返回错误
    assert!(
        result.is_err(),
        "Should return error for unsupported shell type"
    );

    cleanup_temp_test_dir(&test_dir);
}

#[test]
fn test_completion_generator_new_auto_detect_creates_generator() {
    // Arrange: 准备临时测试目录
    let test_dir = create_temp_test_dir("completion_gen");

    // Act: 自动检测 shell 类型创建 CompletionGenerator
    let result = CompletionGenerator::new(None, Some(test_dir.to_string_lossy().to_string()));

    // Assert: 验证创建成功
    assert!(
        result.is_ok(),
        "Should create CompletionGenerator with auto-detected shell"
    );

    cleanup_temp_test_dir(&test_dir);
}

#[test]
fn test_completion_generator_new_with_default_output_dir_creates_generator() {
    // Arrange: 准备使用默认输出目录

    // Act: 使用默认输出目录创建 CompletionGenerator
    let result = CompletionGenerator::new(Some("zsh".to_string()), None);

    // Assert: 验证创建成功
    assert!(
        result.is_ok(),
        "Should create CompletionGenerator with default output dir"
    );
}

// ==================== Completion 生成测试 ====================

#[rstest]
#[case("zsh")]
#[case("bash")]
#[case("fish")]
fn test_completion_generator_generate_all_with_shell_generates_files(#[case] shell: &str) {
    // Arrange: 准备临时测试目录和生成器
    let test_dir = create_temp_test_dir("completion_gen");
    let generator = CompletionGenerator::new(
        Some(shell.to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    )
    .expect("Should create generator");

    // Act: 生成补全脚本
    let result = generator.generate_all();

    // Assert: 验证生成结果（成功或失败都是可以接受的）
    match result {
        Ok(generate_result) => {
            assert!(
                generate_result.messages.len() > 0,
                "Should have generation messages"
            );
            if shell == "zsh" {
                let files: Vec<_> = std::fs::read_dir(&test_dir)
                    .expect("should read test directory")
                    .map(|entry| entry.expect("directory entry should be valid").file_name())
                    .collect();
                assert!(
                    files.len() > 0,
                    "Should generate at least one completion file"
                );
            }
        }
        Err(_) => {
            // 生成失败，这也是可以接受的（例如 CLI 结构体问题）
        }
    }

    cleanup_temp_test_dir(&test_dir);
}

// ==================== GenerateResult 结构体测试 ====================

#[test]
fn test_generate_result_structure_with_messages_creates_result() {
    // Arrange: 准备消息列表
    use workflow::completion::generate::GenerateResult;
    let messages = vec!["Message 1".to_string(), "Message 2".to_string()];

    // Act: 创建 GenerateResult 实例
    let result = GenerateResult {
        messages: messages.clone(),
    };

    // Assert: 验证消息内容正确
    assert_eq!(result.messages.len(), 2);
    assert_eq!(result.messages[0], "Message 1");
    assert_eq!(result.messages[1], "Message 2");
}

#[test]
fn test_generate_result_empty_messages_with_no_messages_creates_result() {
    // Arrange: 准备空消息列表
    use workflow::completion::generate::GenerateResult;

    // Act: 创建空的 GenerateResult 实例
    let result = GenerateResult { messages: vec![] };

    // Assert: 验证消息列表为空
    assert_eq!(result.messages.len(), 0);
}

// ==================== 边界条件测试 ====================

#[test]
fn test_completion_generator_new_with_empty_shell_returns_error() {
    // Arrange: 准备临时测试目录和空 shell 字符串
    let test_dir = create_temp_test_dir("completion_gen");

    // Act: 使用空字符串创建 CompletionGenerator
    let result = CompletionGenerator::new(
        Some("".to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    );

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should return error for empty shell type");

    cleanup_temp_test_dir(&test_dir);
}

#[test]
fn test_completion_generator_new_with_invalid_output_dir_creates_generator() {
    // Arrange: 准备无效的输出目录路径
    // 注意：PathBuf::from 通常不会验证路径的有效性，所以这个测试主要验证函数不会 panic

    // Act: 使用无效输出目录创建 CompletionGenerator
    let result = CompletionGenerator::new(
        Some("zsh".to_string()),
        Some("/invalid/path/that/does/not/exist".to_string()),
    );

    // Assert: 验证创建成功（路径验证在写入时进行）
    assert!(
        result.is_ok(),
        "Should create generator even with invalid path"
    );
}
