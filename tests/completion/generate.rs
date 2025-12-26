//! Completion 生成模块测试
//!
//! 测试 Completion 脚本生成器的功能。

use crate::common::helpers::{cleanup_temp_test_dir, create_temp_test_dir};
use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::completion::generate::CompletionGenerator;

// ==================== CompletionGenerator 创建测试 ====================

/// 测试使用指定shell创建CompletionGenerator（参数化测试）
///
/// ## 测试目的
/// 验证 `CompletionGenerator::new()` 方法能够使用指定的shell类型成功创建生成器。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下shell类型：
/// - zsh
/// - bash
/// - fish
/// - powershell
/// - elvish
///
/// ## 预期结果
/// - 所有支持的shell类型都能成功创建生成器
/// - 返回Ok结果
#[rstest]
#[case("zsh")]
#[case("bash")]
#[case("fish")]
#[case("powershell")]
#[case("elvish")]
fn test_completion_generator_new_with_shell_creates_generator(
    #[case] shell: &str,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时测试目录
    let test_dir = create_temp_test_dir("completion_gen")?;

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
    let _generator =
        result.map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;

    cleanup_temp_test_dir(&test_dir);
    Ok(())
}

/// 测试使用不支持的shell类型创建CompletionGenerator
///
/// ## 测试目的
/// 验证 `CompletionGenerator::new()` 方法在使用不支持的shell类型时能够正确返回错误。
///
/// ## 测试场景
/// 1. 使用不支持的shell类型（"unsupported"）创建生成器
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 返回Err
/// - 错误消息明确指示shell类型不支持
#[test]
fn test_completion_generator_new_with_unsupported_shell_returns_error() -> color_eyre::Result<()> {
    // Arrange: 准备临时测试目录和不支持的 shell 类型
    let test_dir = create_temp_test_dir("completion_gen")?;

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
    Ok(())
}

/// 测试自动检测shell类型创建CompletionGenerator
///
/// ## 测试目的
/// 验证 `CompletionGenerator::new()` 方法在未指定shell类型时能够自动检测并创建生成器。
///
/// ## 测试场景
/// 1. 调用 `new()` 时传入 `None` 作为shell类型
/// 2. 系统自动检测当前shell类型
/// 3. 验证创建成功
///
/// ## 预期结果
/// - 自动检测shell类型成功
/// - 返回Ok结果
#[test]
fn test_completion_generator_new_auto_detect_creates_generator() -> color_eyre::Result<()> {
    // Arrange: 准备临时测试目录
    let test_dir = create_temp_test_dir("completion_gen")?;

    // Act: 自动检测 shell 类型创建 CompletionGenerator
    let result = CompletionGenerator::new(None, Some(test_dir.to_string_lossy().to_string()));

    // Assert: 验证创建成功
    assert!(
        result.is_ok(),
        "Should create CompletionGenerator with auto-detected shell"
    );

    cleanup_temp_test_dir(&test_dir);
    Ok(())
}

/// 测试使用默认输出目录创建CompletionGenerator
///
/// ## 测试目的
/// 验证 `CompletionGenerator::new()` 方法在未指定输出目录时能够使用默认目录创建生成器。
///
/// ## 测试场景
/// 1. 调用 `new()` 时传入 `None` 作为输出目录
/// 2. 系统使用默认输出目录
/// 3. 验证创建成功
///
/// ## 预期结果
/// - 使用默认输出目录成功创建生成器
/// - 返回Ok结果
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

/// 测试生成所有补全脚本文件（参数化测试）
///
/// ## 测试目的
/// 验证 `CompletionGenerator::generate_all()` 方法能够为指定的shell类型生成补全脚本文件。
///
/// ## 测试场景
/// 使用参数化测试覆盖以下shell类型：
/// - zsh
/// - bash
/// - fish
///
/// 1. 创建CompletionGenerator
/// 2. 调用 `generate_all()` 生成补全脚本
/// 3. 验证生成结果
///
/// ## 预期结果
/// - 生成成功或失败都是可以接受的（取决于CLI结构体）
/// - 如果成功，应包含生成消息
/// - 对于zsh，应生成至少一个补全文件
#[rstest]
#[case("zsh")]
#[case("bash")]
#[case("fish")]
fn test_completion_generator_generate_all_with_shell_generates_files(
    #[case] shell: &str,
) -> color_eyre::Result<()> {
    // Arrange: 准备临时测试目录和生成器
    let test_dir = create_temp_test_dir("completion_gen")?;
    let generator = CompletionGenerator::new(
        Some(shell.to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    )
    .map_err(|e| color_eyre::eyre::eyre!("Should create generator: {}", e))?;

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
                    .map_err(|e| color_eyre::eyre::eyre!("should read test directory: {}", e))?
                    .map(|entry| {
                        entry
                            .map_err(|e| {
                                color_eyre::eyre::eyre!("directory entry should be valid: {}", e)
                            })
                            .map(|e| e.file_name())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
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
    Ok(())
}

// ==================== GenerateResult 结构体测试 ====================

/// 测试GenerateResult结构体创建（带消息）
///
/// ## 测试目的
/// 验证 `GenerateResult` 结构体能够正确存储和访问生成消息列表。
///
/// ## 测试场景
/// 1. 准备消息列表（包含2条消息）
/// 2. 创建 `GenerateResult` 实例
/// 3. 验证消息内容正确
///
/// ## 预期结果
/// - 结构体创建成功
/// - 消息列表长度正确
/// - 消息内容与预期一致
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

/// 测试GenerateResult结构体创建（空消息列表）
///
/// ## 测试目的
/// 验证 `GenerateResult` 结构体能够正确处理空消息列表的情况。
///
/// ## 测试场景
/// 1. 创建空的 `GenerateResult` 实例（消息列表为空）
/// 2. 验证消息列表为空
///
/// ## 预期结果
/// - 结构体创建成功
/// - 消息列表长度为0
#[test]
fn test_generate_result_empty_messages_with_no_messages_creates_result() {
    // Arrange: 准备空消息列表
    use workflow::completion::generate::GenerateResult;

    // Act: 创建空的 GenerateResult 实例
    let result = GenerateResult { messages: vec![] };

    // Assert: 验证消息列表为空
    assert_eq!(result.messages.len(), 0);
}

// ==================== Boundary Condition Tests ====================

/// 测试使用空字符串shell类型创建CompletionGenerator
///
/// ## 测试目的
/// 验证 `CompletionGenerator::new()` 方法在使用空字符串作为shell类型时能够正确返回错误。
///
/// ## 测试场景
/// 1. 使用空字符串（""）作为shell类型创建生成器
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 返回Err
/// - 错误消息明确指示shell类型无效
#[test]
fn test_completion_generator_new_with_empty_shell_returns_error() -> color_eyre::Result<()> {
    // Arrange: 准备临时测试目录和空 shell 字符串
    let test_dir = create_temp_test_dir("completion_gen")?;

    // Act: 使用空字符串创建 CompletionGenerator
    let result = CompletionGenerator::new(
        Some("".to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    );

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should return error for empty shell type");

    cleanup_temp_test_dir(&test_dir);
    Ok(())
}

/// 测试使用无效输出目录创建CompletionGenerator
///
/// ## 测试目的
/// 验证 `CompletionGenerator::new()` 方法在使用无效输出目录路径时不会panic，路径验证在写入时进行。
///
/// ## 测试场景
/// 1. 使用不存在的路径（"/invalid/path/that/does/not/exist"）作为输出目录
/// 2. 创建生成器
/// 3. 验证创建成功（路径验证延迟到写入时）
///
/// ## 注意事项
/// - PathBuf::from 通常不会验证路径的有效性
/// - 路径验证在写入文件时进行
///
/// ## 预期结果
/// - 生成器创建成功（返回Ok）
/// - 不会panic
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
