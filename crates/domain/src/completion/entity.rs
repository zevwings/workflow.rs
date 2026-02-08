//! Shell Completion 实体

use crate::path::{COMPLETIONS_DIR, COMPLETIONS_FILE, COMPLETION_CACHE_DIR, MAIN_DIR};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// 生成结果
// ============================================================================

/// Completion 生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionGenerateResult {
    /// Shell 类型
    pub shell: String,
    /// 脚本文件路径
    pub script_path: PathBuf,
    /// 配置文件路径
    pub config_file: Option<PathBuf>,
    /// 是否新添加了配置（false 表示配置已存在）
    pub config_added: bool,
    /// 重载命令提示
    pub reload_hint: String,
}

// ============================================================================
// 检查结果
// ============================================================================

/// 单个 Shell 的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCompletionStatus {
    /// Shell 类型
    pub shell: String,
    /// 是否已配置
    pub is_configured: bool,
    /// 脚本文件是否存在
    pub script_exists: bool,
    /// 配置文件路径
    pub config_file: Option<PathBuf>,
    /// 是否为当前 shell
    pub is_current: bool,
}

/// Completion 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCheckResult {
    /// 当前 Shell 类型
    pub current_shell: Option<String>,
    /// Completion 目录
    pub completion_dir: Option<PathBuf>,
    /// 各个 Shell 的状态
    pub shell_statuses: Vec<ShellCompletionStatus>,
}

// ============================================================================
// 移除结果
// ============================================================================

/// Completion 移除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRemoveResult {
    /// 移除配置的 shell 列表
    pub removed_configs: Vec<String>,
    /// 删除的脚本文件列表
    pub removed_files: Vec<PathBuf>,
    /// 删除的配置文件
    pub removed_config_file: Option<PathBuf>,
    /// 失败的操作（shell 或文件路径，错误信息）
    pub failures: Vec<(String, String)>,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 获取 completion 文件名
pub fn get_completion_filename(shell: &str) -> String {
    match shell.to_lowercase().as_str() {
        "zsh" => "_workflow".to_string(),
        "bash" => "workflow.bash".to_string(),
        "fish" => "workflow.fish".to_string(),
        "powershell" | "pwsh" => "_workflow.ps1".to_string(),
        "elvish" => "workflow.elv".to_string(),
        _ => "workflow".to_string(),
    }
}

/// 获取所有 shell 类型的 completion 文件名
pub fn get_all_completion_filenames() -> Vec<String> {
    vec![
        "_workflow".to_string(),     // zsh
        "workflow.bash".to_string(), // bash
        "workflow.fish".to_string(), // fish
        "_workflow.ps1".to_string(), // powershell
        "workflow.elv".to_string(),  // elvish
    ]
}

/// 获取 shell 的 source 路径
///
/// 注意：此函数返回 shell 脚本中使用的路径字符串。
pub fn get_shell_source_path(shell: &str) -> String {
    match shell.to_lowercase().as_str() {
        "zsh" | "bash" => format!("$HOME/{}/{}", MAIN_DIR, COMPLETIONS_FILE),
        "fish" => format!(
            "$HOME/{}/{}/{}",
            MAIN_DIR,
            COMPLETIONS_DIR,
            get_completion_filename("fish")
        ),
        "powershell" | "pwsh" => format!(
            "$HOME/{}/{}/{}",
            MAIN_DIR,
            COMPLETIONS_DIR,
            get_completion_filename("powershell")
        ),
        "elvish" => format!(
            "$HOME/{}/{}/{}",
            MAIN_DIR,
            COMPLETIONS_DIR,
            get_completion_filename("elvish")
        ),
        _ => format!("$HOME/{}/{}", MAIN_DIR, COMPLETIONS_FILE),
    }
}

pub fn get_completion_shell_dir() -> String {
    format!("$HOME/{}/{}", MAIN_DIR, COMPLETIONS_DIR)
}

pub fn get_completion_cache_shell_dir() -> String {
    format!("$HOME/{}/{}", MAIN_DIR, COMPLETION_CACHE_DIR)
}

pub fn get_completion_source_shell_path() -> String {
    format!("$HOME/{}/{}", MAIN_DIR, COMPLETIONS_FILE)
}

pub fn get_completion_shell_path(filename: &str) -> String {
    format!("$HOME/{}/{}/{}", MAIN_DIR, COMPLETIONS_DIR, filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // get_completion_filename 测试
    // ========================================================================

    #[test]
    fn test_get_completion_filename_zsh() {
        assert_eq!(get_completion_filename("zsh"), "_workflow");
        assert_eq!(get_completion_filename("ZSH"), "_workflow");
        assert_eq!(get_completion_filename("Zsh"), "_workflow");
    }

    #[test]
    fn test_get_completion_filename_bash() {
        assert_eq!(get_completion_filename("bash"), "workflow.bash");
        assert_eq!(get_completion_filename("BASH"), "workflow.bash");
    }

    #[test]
    fn test_get_completion_filename_fish() {
        assert_eq!(get_completion_filename("fish"), "workflow.fish");
        assert_eq!(get_completion_filename("FISH"), "workflow.fish");
    }

    #[test]
    fn test_get_completion_filename_powershell() {
        assert_eq!(get_completion_filename("powershell"), "_workflow.ps1");
        assert_eq!(get_completion_filename("pwsh"), "_workflow.ps1");
        assert_eq!(get_completion_filename("POWERSHELL"), "_workflow.ps1");
        assert_eq!(get_completion_filename("PWSH"), "_workflow.ps1");
    }

    #[test]
    fn test_get_completion_filename_elvish() {
        assert_eq!(get_completion_filename("elvish"), "workflow.elv");
        assert_eq!(get_completion_filename("ELVISH"), "workflow.elv");
    }

    #[test]
    fn test_get_completion_filename_unknown() {
        assert_eq!(get_completion_filename("unknown"), "workflow");
        assert_eq!(get_completion_filename(""), "workflow");
        assert_eq!(get_completion_filename("csh"), "workflow");
    }

    // ========================================================================
    // get_all_completion_filenames 测试
    // ========================================================================

    #[test]
    fn test_get_all_completion_filenames() {
        let filenames = get_all_completion_filenames();
        assert_eq!(filenames.len(), 5);
        assert!(filenames.contains(&"_workflow".to_string()));
        assert!(filenames.contains(&"workflow.bash".to_string()));
        assert!(filenames.contains(&"workflow.fish".to_string()));
        assert!(filenames.contains(&"_workflow.ps1".to_string()));
        assert!(filenames.contains(&"workflow.elv".to_string()));
    }

    // ========================================================================
    // get_shell_source_path 测试
    // ========================================================================

    #[test]
    fn test_get_shell_source_path_zsh() {
        let path = get_shell_source_path("zsh");
        assert_eq!(path, "$HOME/.workflow/.completions");
    }

    #[test]
    fn test_get_shell_source_path_bash() {
        let path = get_shell_source_path("bash");
        assert_eq!(path, "$HOME/.workflow/.completions");
    }

    #[test]
    fn test_get_shell_source_path_fish() {
        let path = get_shell_source_path("fish");
        assert_eq!(path, "$HOME/.workflow/completions/workflow.fish");
    }

    #[test]
    fn test_get_shell_source_path_powershell() {
        let path = get_shell_source_path("powershell");
        assert_eq!(path, "$HOME/.workflow/completions/_workflow.ps1");

        let path = get_shell_source_path("pwsh");
        assert_eq!(path, "$HOME/.workflow/completions/_workflow.ps1");
    }

    #[test]
    fn test_get_shell_source_path_elvish() {
        let path = get_shell_source_path("elvish");
        assert_eq!(path, "$HOME/.workflow/completions/workflow.elv");
    }

    #[test]
    fn test_get_shell_source_path_unknown() {
        let path = get_shell_source_path("unknown");
        assert_eq!(path, "$HOME/.workflow/.completions");
    }

    #[test]
    fn test_get_shell_source_path_case_insensitive() {
        assert_eq!(get_shell_source_path("ZSH"), get_shell_source_path("zsh"));
        assert_eq!(get_shell_source_path("FISH"), get_shell_source_path("fish"));
        assert_eq!(
            get_shell_source_path("POWERSHELL"),
            get_shell_source_path("powershell")
        );
    }
}
