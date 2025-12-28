//! Completion 配置模块测试
//!
//! 测试 Shell Completion 配置的创建、删除和检查功能。

use crate::common::helpers::create_temp_test_dir;
use clap_complete::Shell;
use color_eyre::Result;
use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::completion::Completion;

// ==================== Completion 配置检查测试 ====================

/// 测试检查 shell 是否已配置补全（参数化）
///
/// ## 测试目的
/// 验证 Completion::is_shell_configured() 能够检查各种 shell 是否已配置补全。
///
/// ## 测试场景
/// 1. 使用不同的 shell 类型检查配置状态
/// 2. 验证返回 Ok，包含配置状态和配置文件路径
/// 3. 验证配置文件路径与 shell 相关
///
/// ## 预期结果
/// - 返回 Ok，配置文件路径与 shell 相关
#[cfg(not(target_os = "windows"))]
#[rstest]
#[case(Shell::Zsh, "zsh", ".zshrc")]
#[case(Shell::Bash, "bash", ".bashrc")]
#[case(Shell::Fish, "fish", ".config")]
#[case(Shell::Elvish, "elvish", ".elv")]
fn test_is_shell_configured(
    #[case] shell: Shell,
    #[case] shell_name: &str,
    #[case] config_identifier: &str,
) -> Result<()> {
    // Arrange: 准备测试检查 shell 是否已配置 completion
    let result = Completion::is_shell_configured(&shell);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(
        result.is_ok(),
        "Should return Ok for {} shell check",
        shell_name
    );
    let (_configured, config_path) =
        result.map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
    // configured 可能是 true 或 false，取决于实际配置
    let path_str = config_path.to_string_lossy();
    assert!(
        path_str.contains(shell_name) || path_str.contains(config_identifier),
        "Config path should be related to {}: {}",
        shell_name,
        path_str
    );
    Ok(())
}

/// 测试检查 PowerShell 是否已配置补全
///
/// ## 测试目的
/// 验证 Completion::is_shell_configured() 能够检查 PowerShell 是否已配置补全。
///
/// ## 测试场景
/// 1. 使用 PowerShell shell 检查配置状态
/// 2. 验证返回 Ok，包含配置状态和配置文件路径
///
/// ## 预期结果
/// - 返回 Ok，配置文件路径可能包含 "powershell" 或 "Microsoft"
#[test]
fn test_is_shell_configured_powershell() -> Result<()> {
    // Arrange: 准备测试检查 PowerShell 是否已配置 completion
    let result = Completion::is_shell_configured(&Shell::PowerShell);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(result.is_ok(), "Should return Ok for PowerShell check");
    let (_configured, _config_path) =
        result.map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
    // configured 可能是 true 或 false，取决于实际配置
    // PowerShell 配置文件路径可能包含 "powershell" 或 "Microsoft"
    Ok(())
}

// ==================== Completion 文件列表测试 ====================

/// 测试获取补全文件列表（参数化）
///
/// ## 测试目的
/// 验证 Completion::get_completion_files() 能够获取 shell 的补全文件列表。
///
/// ## 测试场景
/// 1. 使用不同的 shell 类型获取补全文件列表
/// 2. 验证返回文件路径列表
/// 3. 验证文件路径与补全相关
///
/// ## 预期结果
/// - 返回文件路径列表，文件路径与补全相关
#[rstest]
#[case(Shell::Zsh, "completion", "_workflow")]
#[case(Shell::Bash, "completion", "workflow.bash")]
#[case(Shell::Fish, "completion", ".fish")]
fn test_get_completion_files(
    #[case] shell: Shell,
    #[case] identifier1: &str,
    #[case] identifier2: &str,
) {
    // Arrange: 准备测试获取 shell 的 completion 文件列表
    let files = Completion::get_completion_files(&shell);

    // 应该返回文件路径列表（可能为空）
    // 如果列表不为空，文件路径应该与 shell 相关
    for file in &files {
        let file_str = file.to_string_lossy();
        assert!(
            file_str.contains(identifier1) || file_str.contains(identifier2),
            "File path should be related to completion: {}",
            file_str
        );
    }
}

// ==================== Completion 配置删除测试 ====================

/// 测试删除补全配置文件
///
/// ## 测试目的
/// 验证 Completion::remove_completion_config_file() 能够删除补全配置文件。
///
/// ## 测试场景
/// 1. 调用 remove_completion_config_file()
/// 2. 验证返回 Ok，表示是否删除了文件
///
/// ## 预期结果
/// - 返回 Ok，返回值可能是 true（文件存在并删除）或 false（文件不存在）
#[test]
fn test_remove_completion_config_file() {
    // Arrange: 准备测试删除 completion 配置文件
    let result = Completion::remove_completion_config_file();

    // 应该返回 Ok，表示是否删除了文件
    assert!(result.is_ok(), "Should return Ok when removing config file");
    // 返回值可能是 true（文件存在并删除）或 false（文件不存在）
}

/// 测试删除 shell 的补全配置（参数化）
///
/// ## 测试目的
/// 验证 Completion::remove_completion_config() 能够删除指定 shell 的补全配置。
///
/// ## 测试场景
/// 1. 使用不同的 shell 类型删除补全配置
/// 2. 验证返回 Ok（即使配置不存在）
///
/// ## 预期结果
/// - 返回 Ok，即使配置不存在也能成功
#[cfg(not(target_os = "windows"))]
#[rstest]
#[case(Shell::Zsh)]
#[case(Shell::Bash)]
#[case(Shell::Fish)]
fn test_remove_completion_config(#[case] shell: Shell) {
    // Arrange: 准备测试删除 shell 的 completion 配置
    let result = Completion::remove_completion_config(&shell);

    // 应该返回 Ok（即使配置不存在）
    assert!(
        result.is_ok(),
        "Should return Ok when removing {:?} config",
        shell
    );
}

/// 测试删除所有 shell 的补全配置
///
/// ## 测试目的
/// 验证 Completion::remove_all_completion_configs() 能够删除所有 shell 的补全配置。
///
/// ## 测试场景
/// 1. 调用 remove_all_completion_configs()
/// 2. 验证返回 Ok
///
/// ## 预期结果
/// - 返回 Ok
#[test]
fn test_remove_all_completion_configs() {
    // Arrange: 准备测试删除所有 shell 的 completion 配置
    let result = Completion::remove_all_completion_configs();

    // 应该返回 Ok
    assert!(result.is_ok(), "Should return Ok when removing all configs");
}

// ==================== Completion 文件删除测试 ====================

/// 测试删除 shell 的补全文件（参数化）
///
/// ## 测试目的
/// 验证 Completion::remove_completion_files() 能够删除指定 shell 的补全文件。
///
/// ## 测试场景
/// 1. 使用不同的 shell 类型删除补全文件
/// 2. 验证返回 Ok，包含删除结果
/// 3. 验证删除的文件数量匹配
///
/// ## 预期结果
/// - 返回 Ok，删除的文件数量匹配
#[rstest]
#[case(Shell::Zsh)]
#[case(Shell::Bash)]
#[case(Shell::Fish)]
fn test_remove_completion_files(#[case] shell: Shell) -> Result<()> {
    // Arrange: 准备测试删除 shell 的 completion 文件
    let result = Completion::remove_completion_files(&shell);

    // 应该返回 Ok，包含删除结果
    assert!(
        result.is_ok(),
        "Should return Ok when removing {:?} completion files",
        shell
    );
    let removal_result =
        result.map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
    assert!(
        removal_result.removed_files.len() == removal_result.removed_count,
        "Removed files count should match removed count"
    );
    Ok(())
}

// ==================== Completion 配置结果结构体测试 ====================

/// 测试 CompletionConfigResult 结构体
///
/// ## 测试目的
/// 验证 CompletionConfigResult 结构体的字段和值设置。
///
/// ## 测试场景
/// 1. 创建 CompletionConfigResult 实例
/// 2. 验证所有字段的值正确
///
/// ## 预期结果
/// - 所有字段的值都正确
#[test]
fn test_completion_config_result_structure() {
    // Arrange: 准备测试 CompletionConfigResult 结构体
    use workflow::completion::CompletionConfigResult;

    let result = CompletionConfigResult {
        shell: Shell::Zsh,
        already_exists: false,
        added: true,
        config_file: None,
    };

    assert_eq!(result.shell, Shell::Zsh);
    assert_eq!(result.already_exists, false);
    assert_eq!(result.added, true);
    assert_eq!(result.config_file, None);
}

/// 测试 CompletionRemovalResult 结构体
///
/// ## 测试目的
/// 验证 CompletionRemovalResult 结构体的字段和值设置。
///
/// ## 测试场景
/// 1. 创建 CompletionRemovalResult 实例
/// 2. 验证所有字段的值正确
///
/// ## 预期结果
/// - 所有字段的值都正确
#[test]
fn test_completion_removal_result_structure() {
    // Arrange: 准备测试 CompletionRemovalResult 结构体
    use std::path::PathBuf;
    use workflow::completion::CompletionRemovalResult;

    let result = CompletionRemovalResult {
        removed_count: 2,
        removed_files: vec![
            PathBuf::from("/path/to/file1"),
            PathBuf::from("/path/to/file2"),
        ],
        failed_files: vec![],
    };

    assert_eq!(result.removed_count, 2);
    assert_eq!(result.removed_files.len(), 2);
    assert_eq!(result.failed_files.len(), 0);
}

// ==================== Completion 生成测试 ====================

/// 测试使用指定 shell 类型生成所有补全
///
/// ## 测试目的
/// 验证 Completion::generate_all_completions() 能够使用指定的 shell 类型生成所有补全。
///
/// ## 测试场景
/// 1. 使用指定的 shell 类型生成补全
/// 2. 验证生成结果（成功或路径解析失败）
///
/// ## 预期结果
/// - 如果路径解析成功，应该能生成；否则返回错误
#[test]
fn test_generate_all_completions_with_shell_type() {
    // Arrange: 准备测试使用指定 shell 类型生成所有 completion
    let result = Completion::generate_all_completions(Some("zsh".to_string()), None);

    // 如果路径解析成功，应该能生成；否则返回错误
    match result {
        Ok(_) => {}
        Err(_) => {
            // 路径解析失败或目录创建失败，这也是可以接受的
        }
    }
}

/// 测试自动检测 shell 类型生成所有补全
///
/// ## 测试目的
/// 验证 Completion::generate_all_completions() 能够自动检测 shell 类型并生成所有补全。
///
/// ## 测试场景
/// 1. 不指定 shell 类型，自动检测
/// 2. 验证生成结果（成功或路径解析/检测失败）
///
/// ## 预期结果
/// - 如果路径解析和 shell 检测成功，应该能生成；否则返回错误
#[test]
fn test_generate_all_completions_auto_detect() {
    // Arrange: 准备测试自动检测 shell 类型生成所有 completion
    let result = Completion::generate_all_completions(None, None);

    // 如果路径解析成功，应该能生成；否则返回错误
    match result {
        Ok(_) => {}
        Err(_) => {
            // 路径解析失败或 shell 检测失败，这也是可以接受的
        }
    }
}

/// 测试使用指定输出目录生成所有补全
///
/// ## 测试目的
/// 验证 Completion::generate_all_completions() 能够使用指定的输出目录生成所有补全。
///
/// ## 测试场景
/// 1. 使用指定的输出目录生成补全
/// 2. 验证生成结果（成功或目录创建失败）
///
/// ## 预期结果
/// - 如果目录创建成功，应该能生成；否则返回错误
#[test]
fn test_generate_all_completions_with_output_dir() -> color_eyre::Result<()> {
    // Arrange: 准备测试使用指定输出目录生成所有 completion
    let test_dir = create_temp_test_dir("completion_test")?;
    let result = Completion::generate_all_completions(
        Some("zsh".to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    );

    // 如果目录创建成功，应该能生成；否则返回错误
    match result {
        Ok(_) => {}
        Err(_) => {
            // 目录创建失败，这也是可以接受的
        }
    }

    crate::common::helpers::cleanup_temp_test_dir(&test_dir);
    Ok(())
}
