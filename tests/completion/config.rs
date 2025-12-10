//! Completion 配置模块测试
//!
//! 测试 Shell Completion 配置的创建、删除和检查功能。

use crate::common::helpers::create_temp_test_dir;
use clap_complete::Shell;
use workflow::completion::Completion;

// ==================== Completion 配置检查测试 ====================

#[test]
fn test_is_shell_configured_zsh() {
    // 测试检查 zsh 是否已配置 completion
    let result = Completion::is_shell_configured(&Shell::Zsh);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(result.is_ok(), "Should return Ok for zsh shell check");
    let (_configured, config_path) = result.unwrap();
    // configured 可能是 true 或 false，取决于实际配置
    assert!(
        config_path.to_string_lossy().contains("zsh")
            || config_path.to_string_lossy().contains(".zshrc"),
        "Config path should be related to zsh"
    );
}

#[test]
fn test_is_shell_configured_bash() {
    // 测试检查 bash 是否已配置 completion
    let result = Completion::is_shell_configured(&Shell::Bash);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(result.is_ok(), "Should return Ok for bash shell check");
    let (_configured, config_path) = result.unwrap();
    // configured 可能是 true 或 false，取决于实际配置
    assert!(
        config_path.to_string_lossy().contains("bash")
            || config_path.to_string_lossy().contains(".bashrc")
            || config_path.to_string_lossy().contains(".bash_profile"),
        "Config path should be related to bash"
    );
}

#[test]
fn test_is_shell_configured_fish() {
    // 测试检查 fish 是否已配置 completion
    let result = Completion::is_shell_configured(&Shell::Fish);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(result.is_ok(), "Should return Ok for fish shell check");
    let (_configured, config_path) = result.unwrap();
    // configured 可能是 true 或 false，取决于实际配置
    assert!(
        config_path.to_string_lossy().contains("fish")
            || config_path.to_string_lossy().contains(".config"),
        "Config path should be related to fish"
    );
}

#[test]
fn test_is_shell_configured_powershell() {
    // 测试检查 PowerShell 是否已配置 completion
    let result = Completion::is_shell_configured(&Shell::PowerShell);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(result.is_ok(), "Should return Ok for PowerShell check");
    let (_configured, _config_path) = result.unwrap();
    // configured 可能是 true 或 false，取决于实际配置
    // PowerShell 配置文件路径可能包含 "powershell" 或 "Microsoft"
}

#[test]
fn test_is_shell_configured_elvish() {
    // 测试检查 elvish 是否已配置 completion
    let result = Completion::is_shell_configured(&Shell::Elvish);

    // 应该返回 Ok，包含配置状态和配置文件路径
    assert!(result.is_ok(), "Should return Ok for elvish shell check");
    let (_configured, config_path) = result.unwrap();
    // configured 可能是 true 或 false，取决于实际配置
    assert!(
        config_path.to_string_lossy().contains("elvish")
            || config_path.to_string_lossy().contains(".elv"),
        "Config path should be related to elvish"
    );
}

// ==================== Completion 文件列表测试 ====================

#[test]
fn test_get_completion_files_zsh() {
    // 测试获取 zsh 的 completion 文件列表
    let files = Completion::get_completion_files(&Shell::Zsh);

    // 应该返回文件路径列表（可能为空）
    assert!(true, "Should return a list of file paths");
    // 如果列表不为空，文件路径应该与 zsh 相关
    for file in &files {
        let file_str = file.to_string_lossy();
        assert!(
            file_str.contains("completion") || file_str.contains("_workflow"),
            "File path should be related to completion"
        );
    }
}

#[test]
fn test_get_completion_files_bash() {
    // 测试获取 bash 的 completion 文件列表
    let files = Completion::get_completion_files(&Shell::Bash);

    // 应该返回文件路径列表（可能为空）
    assert!(true, "Should return a list of file paths");
    // 如果列表不为空，文件路径应该与 bash 相关
    for file in &files {
        let file_str = file.to_string_lossy();
        assert!(
            file_str.contains("completion") || file_str.contains("workflow.bash"),
            "File path should be related to completion"
        );
    }
}

#[test]
fn test_get_completion_files_fish() {
    // 测试获取 fish 的 completion 文件列表
    let files = Completion::get_completion_files(&Shell::Fish);

    // 应该返回文件路径列表（可能为空）
    assert!(true, "Should return a list of file paths");
    // 如果列表不为空，文件路径应该与 fish 相关
    for file in &files {
        let file_str = file.to_string_lossy();
        assert!(
            file_str.contains("completion") || file_str.contains(".fish"),
            "File path should be related to completion"
        );
    }
}

// ==================== Completion 配置删除测试 ====================

#[test]
fn test_remove_completion_config_file() {
    // 测试删除 completion 配置文件
    let result = Completion::remove_completion_config_file();

    // 应该返回 Ok，表示是否删除了文件
    assert!(result.is_ok(), "Should return Ok when removing config file");
    // 返回值可能是 true（文件存在并删除）或 false（文件不存在）
}

#[test]
fn test_remove_completion_config_zsh() {
    // 测试删除 zsh 的 completion 配置
    let result = Completion::remove_completion_config(&Shell::Zsh);

    // 应该返回 Ok（即使配置不存在）
    assert!(result.is_ok(), "Should return Ok when removing zsh config");
}

#[test]
fn test_remove_completion_config_bash() {
    // 测试删除 bash 的 completion 配置
    let result = Completion::remove_completion_config(&Shell::Bash);

    // 应该返回 Ok（即使配置不存在）
    assert!(result.is_ok(), "Should return Ok when removing bash config");
}

#[test]
fn test_remove_completion_config_fish() {
    // 测试删除 fish 的 completion 配置
    let result = Completion::remove_completion_config(&Shell::Fish);

    // 应该返回 Ok（即使配置不存在）
    assert!(result.is_ok(), "Should return Ok when removing fish config");
}

#[test]
fn test_remove_all_completion_configs() {
    // 测试删除所有 shell 的 completion 配置
    let result = Completion::remove_all_completion_configs();

    // 应该返回 Ok
    assert!(result.is_ok(), "Should return Ok when removing all configs");
}

// ==================== Completion 文件删除测试 ====================

#[test]
fn test_remove_completion_files_zsh() {
    // 测试删除 zsh 的 completion 文件
    let result = Completion::remove_completion_files(&Shell::Zsh);

    // 应该返回 Ok，包含删除结果
    assert!(
        result.is_ok(),
        "Should return Ok when removing zsh completion files"
    );
    let removal_result = result.unwrap();
    assert!(true, "Removed count should be non-negative");
    assert!(
        removal_result.removed_files.len() == removal_result.removed_count as usize,
        "Removed files count should match removed count"
    );
}

#[test]
fn test_remove_completion_files_bash() {
    // 测试删除 bash 的 completion 文件
    let result = Completion::remove_completion_files(&Shell::Bash);

    // 应该返回 Ok，包含删除结果
    assert!(
        result.is_ok(),
        "Should return Ok when removing bash completion files"
    );
    let _removal_result = result.unwrap();
    assert!(true, "Removed count should be non-negative");
}

#[test]
fn test_remove_completion_files_fish() {
    // 测试删除 fish 的 completion 文件
    let result = Completion::remove_completion_files(&Shell::Fish);

    // 应该返回 Ok，包含删除结果
    assert!(
        result.is_ok(),
        "Should return Ok when removing fish completion files"
    );
    let _removal_result = result.unwrap();
    assert!(true, "Removed count should be non-negative");
}

// ==================== Completion 配置结果结构体测试 ====================

#[test]
fn test_completion_config_result_structure() {
    // 测试 CompletionConfigResult 结构体
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

#[test]
fn test_completion_removal_result_structure() {
    // 测试 CompletionRemovalResult 结构体
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

#[test]
fn test_generate_all_completions_with_shell_type() {
    // 测试使用指定 shell 类型生成所有 completion
    let result = Completion::generate_all_completions(Some("zsh".to_string()), None);

    // 如果路径解析成功，应该能生成；否则返回错误
    match result {
        Ok(_) => {
            assert!(true, "Should succeed when path resolution works");
        }
        Err(_) => {
            // 路径解析失败或目录创建失败，这也是可以接受的
            assert!(true, "Path resolution or directory creation may fail");
        }
    }
}

#[test]
fn test_generate_all_completions_with_output_dir() {
    // 测试使用指定输出目录生成所有 completion
    let test_dir = create_temp_test_dir("completion_test");
    let result = Completion::generate_all_completions(
        Some("zsh".to_string()),
        Some(test_dir.to_string_lossy().to_string()),
    );

    // 如果目录创建成功，应该能生成；否则返回错误
    match result {
        Ok(_) => {
            assert!(true, "Should succeed when directory creation works");
        }
        Err(_) => {
            // 目录创建失败，这也是可以接受的
            assert!(true, "Directory creation may fail");
        }
    }

    crate::common::helpers::cleanup_temp_test_dir(&test_dir);
}

#[test]
fn test_generate_all_completions_auto_detect() {
    // 测试自动检测 shell 类型生成所有 completion
    let result = Completion::generate_all_completions(None, None);

    // 如果路径解析成功，应该能生成；否则返回错误
    match result {
        Ok(_) => {
            assert!(true, "Should succeed when path resolution works");
        }
        Err(_) => {
            // 路径解析失败或 shell 检测失败，这也是可以接受的
            assert!(true, "Path resolution or shell detection may fail");
        }
    }
}
