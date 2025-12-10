//! Completion 辅助函数测试
//!
//! 测试 completion::helpers 模块中的工具函数，包括：
//! - 补全脚本文件名生成
//! - Shell 类型文件列表获取
//! - 所有 shell 类型文件列表获取

use workflow::completion::{
    get_all_completion_files, get_completion_filename, get_completion_files_for_shell,
};

// ==================== get_completion_filename 测试 ====================

/// 测试 zsh shell 的文件名生成
#[test]
fn test_get_completion_filename_zsh() {
    let result = get_completion_filename("zsh", "workflow").unwrap();
    assert_eq!(result, "_workflow");
}

/// 测试 bash shell 的文件名生成
#[test]
fn test_get_completion_filename_bash() {
    let result = get_completion_filename("bash", "workflow").unwrap();
    assert_eq!(result, "workflow.bash");
}

/// 测试 fish shell 的文件名生成
#[test]
fn test_get_completion_filename_fish() {
    let result = get_completion_filename("fish", "workflow").unwrap();
    assert_eq!(result, "workflow.fish");
}

/// 测试 powershell shell 的文件名生成
#[test]
fn test_get_completion_filename_powershell() {
    let result = get_completion_filename("powershell", "workflow").unwrap();
    assert_eq!(result, "_workflow.ps1");
}

/// 测试 elvish shell 的文件名生成
#[test]
fn test_get_completion_filename_elvish() {
    let result = get_completion_filename("elvish", "workflow").unwrap();
    assert_eq!(result, "workflow.elv");
}

/// 测试不支持的 shell 类型
#[test]
fn test_get_completion_filename_unsupported_shell() {
    let result = get_completion_filename("csh", "workflow");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported shell type"));
    assert!(error_msg.contains("csh"));
}

/// 测试不同命令名的文件名生成
#[test]
fn test_get_completion_filename_different_command() {
    let result = get_completion_filename("zsh", "mycmd").unwrap();
    assert_eq!(result, "_mycmd");
}

/// 测试空命令名
#[test]
fn test_get_completion_filename_empty_command() {
    let result = get_completion_filename("zsh", "").unwrap();
    assert_eq!(result, "_");
}

/// 测试所有支持的 shell 类型文件名格式
#[test]
fn test_get_completion_filename_all_shells() {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let expected = [
        "_workflow",
        "workflow.bash",
        "workflow.fish",
        "_workflow.ps1",
        "workflow.elv",
    ];

    for (shell, expected_filename) in shells.iter().zip(expected.iter()) {
        let result = get_completion_filename(shell, "workflow").unwrap();
        assert_eq!(
            result, *expected_filename,
            "Wrong filename for {}: expected {}, got {}",
            shell, expected_filename, result
        );
    }
}

// ==================== get_completion_files_for_shell 测试 ====================

/// 测试获取单个命令的补全文件列表
#[test]
fn test_get_completion_files_for_shell_single_command() {
    let result = get_completion_files_for_shell("zsh", &["workflow"]).unwrap();
    assert_eq!(result, vec!["_workflow"]);
}

/// 测试获取多个命令的补全文件列表
#[test]
fn test_get_completion_files_for_shell_multiple_commands() {
    let result = get_completion_files_for_shell("zsh", &["workflow", "mycmd"]).unwrap();
    assert_eq!(result, vec!["_workflow", "_mycmd"]);
}

/// 测试 bash shell 的多个命令文件列表
#[test]
fn test_get_completion_files_for_shell_bash_multiple() {
    let result = get_completion_files_for_shell("bash", &["workflow", "tool"]).unwrap();
    assert_eq!(result, vec!["workflow.bash", "tool.bash"]);
}

/// 测试空命令列表
#[test]
fn test_get_completion_files_for_shell_empty_commands() {
    let result = get_completion_files_for_shell("zsh", &[]).unwrap();
    assert!(result.is_empty());
}

/// 测试不支持的 shell 类型
#[test]
fn test_get_completion_files_for_shell_unsupported() {
    let result = get_completion_files_for_shell("csh", &["workflow"]);
    assert!(result.is_err());
}

/// 测试所有支持的 shell 类型的文件列表生成
#[test]
fn test_get_completion_files_for_shell_all_shells() {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let commands = &["workflow"];

    for shell in &shells {
        let result = get_completion_files_for_shell(shell, commands).unwrap();
        assert_eq!(result.len(), 1, "Should have one file for {}", shell);
        assert!(
            !result[0].is_empty(),
            "Filename should not be empty for {}",
            shell
        );
    }
}

// ==================== get_all_completion_files 测试 ====================

/// 测试获取所有 shell 类型的补全文件列表
#[test]
fn test_get_all_completion_files_single_command() {
    let result = get_all_completion_files(&["workflow"]);

    // 应该包含所有 5 种 shell 类型的文件
    assert_eq!(result.len(), 5);

    // 验证包含所有预期的文件名
    assert!(result.contains(&"_workflow".to_string()));
    assert!(result.contains(&"workflow.bash".to_string()));
    assert!(result.contains(&"workflow.fish".to_string()));
    assert!(result.contains(&"_workflow.ps1".to_string()));
    assert!(result.contains(&"workflow.elv".to_string()));
}

/// 测试获取多个命令的所有补全文件列表
#[test]
fn test_get_all_completion_files_multiple_commands() {
    let result = get_all_completion_files(&["workflow", "mycmd"]);

    // 应该包含 5 种 shell 类型 × 2 个命令 = 10 个文件
    assert_eq!(result.len(), 10);

    // 验证包含所有预期的文件名
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
}

/// 测试空命令列表
#[test]
fn test_get_all_completion_files_empty_commands() {
    let result = get_all_completion_files(&[]);
    assert!(result.is_empty());
}

/// 测试所有文件名的唯一性
#[test]
fn test_get_all_completion_files_uniqueness() {
    let result = get_all_completion_files(&["workflow"]);
    let unique_count = result.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(unique_count, result.len(), "All filenames should be unique");
}

/// 测试文件列表的排序（虽然不是必需的，但验证结果的一致性）
#[test]
fn test_get_all_completion_files_consistency() {
    let result1 = get_all_completion_files(&["workflow"]);
    let result2 = get_all_completion_files(&["workflow"]);

    // 结果应该一致（虽然顺序可能不同，但内容应该相同）
    let set1: std::collections::HashSet<_> = result1.iter().collect();
    let set2: std::collections::HashSet<_> = result2.iter().collect();
    assert_eq!(set1, set2, "Results should be consistent across calls");
}

// ==================== 集成测试 ====================

/// 测试函数之间的集成：从单个文件名到所有文件列表
#[test]
fn test_integration_filename_to_all_files() {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let command = "workflow";

    // 使用 get_completion_filename 生成每个 shell 的文件名
    let mut expected_files = Vec::new();
    for shell in &shells {
        let filename = get_completion_filename(shell, command).unwrap();
        expected_files.push(filename);
    }

    // 使用 get_all_completion_files 获取所有文件
    let all_files = get_all_completion_files(&[command]);

    // 验证所有预期的文件都在结果中
    for expected_file in &expected_files {
        assert!(
            all_files.contains(expected_file),
            "Expected file {} not found in all_files",
            expected_file
        );
    }

    // 验证数量一致
    assert_eq!(expected_files.len(), all_files.len());
}

/// 测试函数之间的集成：从 shell 文件列表到所有文件列表
#[test]
fn test_integration_shell_files_to_all_files() {
    let shells = ["zsh", "bash", "fish", "powershell", "elvish"];
    let commands = &["workflow"];

    // 使用 get_completion_files_for_shell 获取每个 shell 的文件列表
    let mut expected_files = Vec::new();
    for shell in &shells {
        let files = get_completion_files_for_shell(shell, commands).unwrap();
        expected_files.extend(files);
    }

    // 使用 get_all_completion_files 获取所有文件
    let all_files = get_all_completion_files(commands);

    // 验证所有预期的文件都在结果中
    for expected_file in &expected_files {
        assert!(
            all_files.contains(expected_file),
            "Expected file {} not found in all_files",
            expected_file
        );
    }

    // 验证数量一致
    assert_eq!(expected_files.len(), all_files.len());
}
