//! Completion 辅助函数测试
//!
//! 测试 completion::helpers 模块中的工具函数，包括：
//! - 补全脚本文件名生成
//! - Shell 类型文件列表获取
//! - 所有 shell 类型文件列表获取

use color_eyre::Result;
use workflow::completion::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell,
};

// ==================== Completion Filename Generation Tests ====================

/// 测试生成 zsh shell 补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 能够为 zsh shell 生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用 zsh shell 和命令名生成文件名
/// 2. 验证文件名格式正确（zsh 使用下划线前缀）
///
/// ## 预期结果
/// - 返回 "_workflow"
#[test]
fn test_get_completion_filename_with_zsh_shell_returns_filename() -> Result<()> {
    // Arrange: 准备 zsh shell 和命令名
    let shell = "zsh";
    let command = "workflow";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "_workflow");
    Ok(())
}

/// 测试生成 bash shell 补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 能够为 bash shell 生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用 bash shell 和命令名生成文件名
/// 2. 验证文件名格式正确（bash 使用 .bash 扩展名）
///
/// ## 预期结果
/// - 返回 "workflow.bash"
#[test]
fn test_get_completion_filename_with_bash_shell_returns_filename() -> Result<()> {
    // Arrange: 准备 bash shell 和命令名
    let shell = "bash";
    let command = "workflow";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "workflow.bash");
    Ok(())
}

/// 测试生成 fish shell 补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 能够为 fish shell 生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用 fish shell 和命令名生成文件名
/// 2. 验证文件名格式正确（fish 使用 .fish 扩展名）
///
/// ## 预期结果
/// - 返回 "workflow.fish"
#[test]
fn test_get_completion_filename_with_fish_shell_returns_filename() -> Result<()> {
    // Arrange: 准备 fish shell 和命令名
    let shell = "fish";
    let command = "workflow";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "workflow.fish");
    Ok(())
}

/// 测试生成 PowerShell shell 补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 能够为 PowerShell shell 生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用 PowerShell shell 和命令名生成文件名
/// 2. 验证文件名格式正确（PowerShell 使用下划线前缀和 .ps1 扩展名）
///
/// ## 预期结果
/// - 返回 "_workflow.ps1"
#[test]
fn test_get_completion_filename_with_powershell_shell_returns_filename() -> Result<()> {
    // Arrange: 准备 powershell shell 和命令名
    let shell = "powershell";
    let command = "workflow";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "_workflow.ps1");
    Ok(())
}

/// 测试生成 elvish shell 补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 能够为 elvish shell 生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用 elvish shell 和命令名生成文件名
/// 2. 验证文件名格式正确（elvish 使用 .elv 扩展名）
///
/// ## 预期结果
/// - 返回 "workflow.elv"
#[test]
fn test_get_completion_filename_with_elvish_shell_returns_filename() -> Result<()> {
    // Arrange: 准备 elvish shell 和命令名
    let shell = "elvish";
    let command = "workflow";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "workflow.elv");
    Ok(())
}

/// 测试不支持的 shell 类型处理
///
/// ## 测试目的
/// 验证 get_completion_filename() 对不支持的 shell 类型返回错误。
///
/// ## 测试场景
/// 1. 使用不支持的 shell 类型（如 csh）生成文件名
/// 2. 验证返回错误且错误消息包含提示信息
///
/// ## 预期结果
/// - 返回错误，错误消息包含 "Unsupported shell type" 和 shell 名称
#[test]
fn test_get_completion_filename_with_unsupported_shell_returns_error() -> Result<()> {
    // Arrange: 准备不支持的 shell 类型
    let shell = "csh";
    let command = "workflow";

    // Act: 尝试生成文件名
    let result = get_completion_filename(shell, command);

    // Assert: 验证返回错误且错误消息包含提示
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported shell type"));
    assert!(error_msg.contains("csh"));
    Ok(())
}

/// 测试使用不同命令名生成补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 能够为不同的命令名生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用不同的命令名生成文件名
/// 2. 验证文件名包含命令名
///
/// ## 预期结果
/// - 文件名包含命令名
#[test]
fn test_get_completion_filename_with_different_command_returns_filename() -> Result<()> {
    // Arrange: 准备不同的命令名
    let shell = "zsh";
    let command = "mycmd";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "_mycmd");
    Ok(())
}

/// 测试使用空命令名生成补全文件名
///
/// ## 测试目的
/// 验证 get_completion_filename() 对空命令名的处理。
///
/// ## 测试场景
/// 1. 使用空命令名生成文件名
/// 2. 验证文件名格式正确（只包含 shell 特定的前缀/后缀）
///
/// ## 预期结果
/// - 返回只包含 shell 特定格式的文件名
#[test]
fn test_get_completion_filename_with_empty_command_returns_filename() -> Result<()> {
    // Arrange: 准备空命令名
    let shell = "zsh";
    let command = "";

    // Act: 生成文件名
    let result = get_completion_filename(shell, command)?;

    // Assert: 验证文件名格式正确
    assert_eq!(result, "_");
    Ok(())
}

/// 测试所有支持的 shell 类型的文件名生成
///
/// ## 测试目的
/// 验证 get_completion_filename() 为所有支持的 shell 类型生成正确的文件名。
///
/// ## 测试场景
/// 1. 使用所有支持的 shell 类型生成文件名
/// 2. 验证每个 shell 的文件名格式正确
///
/// ## 预期结果
/// - 所有 shell 类型都返回正确的文件名格式
#[test]
fn test_get_completion_filename_with_all_shells_returns_correct_filenames() -> Result<()> {
    // Arrange: 准备所有支持的 shell 类型和预期文件名
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let expected = [
        "_workflow",
        "workflow.bash",
        "workflow.fish",
        "_workflow.ps1",
        "workflow.elv",
    ];

    // Act & Assert: 验证每个 shell 的文件名格式正确
    for (shell, expected_filename) in shells.iter().zip(expected.iter()) {
        let result = get_completion_filename(shell, "workflow")?;
        assert_eq!(
            result, *expected_filename,
            "Wrong filename for {}: expected {}, got {}",
            shell, expected_filename, result
        );
    }
    Ok(())
}

// ==================== Completion Files for Shell Tests ====================

/// 测试获取单个命令的补全文件列表
///
/// ## 测试目的
/// 验证 get_completion_files_for_shell() 能够为单个命令返回补全文件列表。
///
/// ## 测试场景
/// 1. 使用单个命令获取补全文件列表
/// 2. 验证返回的文件列表正确
///
/// ## 预期结果
/// - 返回包含单个文件的列表
#[test]
fn test_get_completion_files_for_shell_with_single_command_returns_files() -> Result<()> {
    // Arrange: 准备 shell 类型和单个命令
    let shell = "zsh";
    let commands = &["workflow"];

    // Act: 获取补全文件列表
    let result = get_completion_files_for_shell(shell, commands)?;

    // Assert: 验证返回的文件列表正确
    assert_eq!(result, vec!["_workflow"]);
    Ok(())
}

/// 测试获取多个命令的补全文件列表
///
/// ## 测试目的
/// 验证 get_completion_files_for_shell() 能够为多个命令返回补全文件列表。
///
/// ## 测试场景
/// 1. 使用多个命令获取补全文件列表
/// 2. 验证返回的文件列表包含所有命令的文件
///
/// ## 预期结果
/// - 返回包含所有命令文件的列表
#[test]
fn test_get_completion_files_for_shell_with_multiple_commands_returns_files() -> Result<()> {
    // Arrange: 准备 shell 类型和多个命令
    let shell = "zsh";
    let commands = &["workflow", "mycmd"];

    // Act: 获取补全文件列表
    let result = get_completion_files_for_shell(shell, commands)?;

    // Assert: 验证返回的文件列表正确
    assert_eq!(result, vec!["_workflow", "_mycmd"]);
    Ok(())
}

/// 测试获取 bash shell 多个命令的补全文件列表
///
/// ## 测试目的
/// 验证 get_completion_files_for_shell() 能够为 bash shell 的多个命令返回补全文件列表。
///
/// ## 测试场景
/// 1. 使用 bash shell 和多个命令获取补全文件列表
/// 2. 验证返回的文件列表正确
///
/// ## 预期结果
/// - 返回包含所有命令的 bash 补全文件列表
#[test]
fn test_get_completion_files_for_shell_with_bash_multiple_commands_returns_files() -> Result<()> {
    // Arrange: 准备 bash shell 和多个命令
    let shell = "bash";
    let commands = &["workflow", "tool"];

    // Act: 获取补全文件列表
    let result = get_completion_files_for_shell(shell, commands)?;

    // Assert: 验证返回的文件列表正确
    assert_eq!(result, vec!["workflow.bash", "tool.bash"]);
    Ok(())
}

/// 测试获取空命令列表的补全文件列表
///
/// ## 测试目的
/// 验证 get_completion_files_for_shell() 对空命令列表返回空列表。
///
/// ## 测试场景
/// 1. 使用空命令列表获取补全文件列表
/// 2. 验证返回空列表
///
/// ## 预期结果
/// - 返回空列表
#[test]
fn test_get_completion_files_for_shell_with_empty_commands_returns_empty() -> Result<()> {
    // Arrange: 准备 shell 类型和空命令列表
    let shell = "zsh";
    let commands = &[];

    // Act: 获取补全文件列表
    let result = get_completion_files_for_shell(shell, commands)?;

    // Assert: 验证返回空列表
    assert!(result.is_empty());
    Ok(())
}

/// 测试获取不支持的 shell 类型的补全文件列表
///
/// ## 测试目的
/// 验证 get_completion_files_for_shell() 对不支持的 shell 类型返回错误。
///
/// ## 测试场景
/// 1. 使用不支持的 shell 类型获取补全文件列表
/// 2. 验证返回错误
///
/// ## 预期结果
/// - 返回错误
#[test]
fn test_get_completion_files_for_shell_with_unsupported_shell_returns_error() -> Result<()> {
    // Arrange: 准备不支持的 shell 类型
    let shell = "csh";
    let commands = &["workflow"];

    // Act: 尝试获取补全文件列表
    let result = get_completion_files_for_shell(shell, commands);

    // Assert: 验证返回错误
    assert!(result.is_err());
    Ok(())
}

/// 测试所有支持的 shell 类型的文件列表生成
#[test]
fn test_get_completion_files_for_shell_all_shells() -> Result<()> {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let commands = &["workflow"];

    for shell in &shells {
        let result = get_completion_files_for_shell(shell, commands)?;
        assert_eq!(result.len(), 1, "Should have one file for {}", shell);
        assert!(
            !result[0].is_empty(),
            "Filename should not be empty for {}",
            shell
        );
    }
    Ok(())
}

// ==================== get_all_completion_files 测试 ====================

/// 测试获取所有 shell 类型的补全文件列表
#[test]
fn test_get_all_completion_files_single_command() -> Result<()> {
    let result = get_all_completion_files(&["workflow"]);

    // 应该包含所有 5 种 shell 类型的文件
    assert_eq!(result.len(), 5);

    // Assert: 验证包含所有预期的文件名
    assert!(result.contains(&"_workflow".to_string()));
    assert!(result.contains(&"workflow.bash".to_string()));
    assert!(result.contains(&"workflow.fish".to_string()));
    assert!(result.contains(&"_workflow.ps1".to_string()));
    assert!(result.contains(&"workflow.elv".to_string()));
    Ok(())
}

/// 测试获取多个命令的所有补全文件列表
#[test]
fn test_get_all_completion_files_multiple_commands() -> Result<()> {
    let result = get_all_completion_files(&["workflow", "mycmd"]);

    // 应该包含 5 种 shell 类型 × 2 个命令 = 10 个文件
    assert_eq!(result.len(), 10);

    // Assert: 验证包含所有预期的文件名
    assert!(result.contains(&"_workflow".to_string()));
    assert!(result.contains(&"_mycmd".to_string()));
    assert!(result.contains(&"workflow.bash".to_string()));
    assert!(result.contains(&"mycmd.bash".to_string()));
    assert!(result.contains(&"workflow.fish".to_string()));
    assert!(result.contains(&"mycmd.fish".to_string()));
    assert!(result.contains(&"_workflow.ps1".to_string()));
    assert!(result.contains(&"_mycmd.ps1".to_string()));
    assert!(result.contains(&"workflow.elv".to_string()));
    assert!(result.contains(&"mycmd.elv".to_string()));
    Ok(())
}

/// 测试空命令列表
#[test]
fn test_get_all_completion_files_empty_commands() -> Result<()> {
    let result = get_all_completion_files(&[]);
    assert!(result.is_empty());
    Ok(())
}

/// 测试所有文件名的唯一性
#[test]
fn test_get_all_completion_files_uniqueness() -> Result<()> {
    let result = get_all_completion_files(&["workflow"]);
    let unique_count = result.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, result.len(), "All filenames should be unique");
    Ok(())
}

/// 测试文件列表的排序（虽然不是必需的，但验证结果的一致性）
#[test]
fn test_get_all_completion_files_consistency() -> Result<()> {
    let result1 = get_all_completion_files(&["workflow"]);
    let result2 = get_all_completion_files(&["workflow"]);

    // 结果应该一致（虽然顺序可能不同，但内容应该相同）
    let set1: std::collections::HashSet<_> = result1.iter().collect();
    let set2: std::collections::HashSet<_> = result2.iter().collect();
    assert_eq!(set1, set2, "Results should be consistent across calls");
    Ok(())
}

// ==================== 集成测试 ====================

/// 测试函数之间的集成：从单个文件名到所有文件列表
#[test]
fn test_integration_filename_to_all_files() -> Result<()> {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let command = "workflow";

    // 使用 get_completion_filename 生成每个 shell 的文件名
    let mut expected_files = Vec::new();
    for shell in &shells {
        let filename = get_completion_filename(shell, command)?;
        expected_files.push(filename);
    }

    // 使用 get_all_completion_files 获取所有文件
    let all_files = get_all_completion_files(&[command]);

    // Assert: 验证所有预期的文件都在结果中
    for expected_file in &expected_files {
        assert!(
            all_files.contains(expected_file),
            "Expected file {} not found in all_files",
            expected_file
        );
    }

    // Assert: 验证数量一致
    assert_eq!(expected_files.len(), all_files.len());
    Ok(())
}

/// 测试函数之间的集成：从 shell 文件列表到所有文件列表
#[test]
fn test_integration_shell_files_to_all_files() -> Result<()> {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let commands = &["workflow"];

    // 使用 get_completion_files_for_shell 获取每个 shell 的文件列表
    let mut expected_files = Vec::new();
    for shell in &shells {
        let files = get_completion_files_for_shell(shell, commands)?;
        expected_files.extend(files);
    }

    // 使用 get_all_completion_files 获取所有文件
    let all_files = get_all_completion_files(commands);

    // Assert: 验证所有预期的文件都在结果中
    for expected_file in &expected_files {
        assert!(
            all_files.contains(expected_file),
            "Expected file {} not found in all_files",
            expected_file
        );
    }

    // Assert: 验证数量一致
    assert_eq!(expected_files.len(), all_files.len());
    Ok(())
}
