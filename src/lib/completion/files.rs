//! Completion 文件工具函数
//!
//! 提供 completion 文件命名和列表相关的工具函数。

use anyhow::Result;

/// 根据 shell 类型和命令名生成补全脚本文件名
///
/// # 参数
///
/// * `shell_type` - Shell 类型（"zsh", "bash", "fish", "powershell", "elvish"）
/// * `command` - 命令名（"workflow", "pr", "qk"）
///
/// # 返回
///
/// 返回补全脚本文件名。
///
/// # 示例
///
/// ```
/// use workflow::completion::get_completion_filename;
///
/// assert_eq!(get_completion_filename("zsh", "workflow").unwrap(), "_workflow");
/// assert_eq!(get_completion_filename("bash", "workflow").unwrap(), "workflow.bash");
/// assert_eq!(get_completion_filename("fish", "pr").unwrap(), "pr.fish");
/// ```
pub fn get_completion_filename(shell_type: &str, command: &str) -> Result<String> {
    match shell_type {
        "zsh" => Ok(format!("_{}", command)),
        "bash" => Ok(format!("{}.bash", command)),
        "fish" => Ok(format!("{}.fish", command)),
        "powershell" => Ok(format!("_{}.ps1", command)),
        "elvish" => Ok(format!("{}.elv", command)),
        _ => anyhow::bail!("不支持的 shell 类型: {}", shell_type),
    }
}

/// 获取指定 shell 类型的所有补全脚本文件名
///
/// # 参数
///
/// * `shell_type` - Shell 类型
/// * `commands` - 命令名列表（通常为 ["workflow", "pr", "qk"]）
///
/// # 返回
///
/// 返回该 shell 类型的所有补全脚本文件名列表。
///
/// # 示例
///
/// ```
/// use workflow::completion::get_completion_files_for_shell;
///
/// let files = get_completion_files_for_shell("zsh", &["workflow", "pr", "qk"]).unwrap();
/// assert_eq!(files, vec!["_workflow", "_pr", "_qk"]);
/// ```
pub fn get_completion_files_for_shell(shell_type: &str, commands: &[&str]) -> Result<Vec<String>> {
    commands
        .iter()
        .map(|cmd| get_completion_filename(shell_type, cmd))
        .collect()
}

/// 获取所有 shell 类型的所有补全脚本文件名
///
/// 用于备份、删除等需要处理所有 shell 类型文件的场景。
///
/// # 参数
///
/// * `commands` - 命令名列表（通常为 ["workflow", "pr", "qk"]）
///
/// # 返回
///
/// 返回所有 shell 类型的所有补全脚本文件名列表。
///
/// # 示例
///
/// ```
/// use workflow::completion::get_all_completion_files;
///
/// let all_files = get_all_completion_files(&["workflow", "pr", "qk"]);
/// // 包含 zsh, bash, fish, powershell, elvish 的所有文件
/// assert!(all_files.contains(&"_workflow".to_string()));
/// assert!(all_files.contains(&"workflow.bash".to_string()));
/// assert!(all_files.contains(&"workflow.fish".to_string()));
/// ```
pub fn get_all_completion_files(commands: &[&str]) -> Vec<String> {
    let shell_types = ["zsh", "bash", "fish", "powershell", "elvish"];
    let mut all_files = Vec::new();

    for shell_type in &shell_types {
        if let Ok(files) = get_completion_files_for_shell(shell_type, commands) {
            all_files.extend(files);
        }
    }

    all_files
}
