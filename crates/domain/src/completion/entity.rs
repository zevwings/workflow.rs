//! Shell Completion 实体

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// 常量定义
// ============================================================================

/// Completion 配置文件名
pub const COMPLETIONS_FILE: &str = ".completions";

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
/// 实际路径应使用 `toolkit::completion_source_shell_path()` 和
/// `toolkit::completion_file_shell_path()` 获取。
pub fn get_shell_source_path(shell: &str) -> String {
    // 使用常量避免硬编码
    const WORKFLOW_DIR: &str = ".workflow";
    const COMPLETIONS_DIR: &str = "completions";
    const COMPLETIONS_CONFIG: &str = ".completions";

    match shell.to_lowercase().as_str() {
        "zsh" | "bash" => format!("$HOME/{}/{}", WORKFLOW_DIR, COMPLETIONS_CONFIG),
        "fish" => format!(
            "$HOME/{}/{}/{}",
            WORKFLOW_DIR,
            COMPLETIONS_DIR,
            get_completion_filename("fish")
        ),
        "powershell" | "pwsh" => format!(
            "$HOME/{}/{}/{}",
            WORKFLOW_DIR,
            COMPLETIONS_DIR,
            get_completion_filename("powershell")
        ),
        "elvish" => format!(
            "$HOME/{}/{}/{}",
            WORKFLOW_DIR,
            COMPLETIONS_DIR,
            get_completion_filename("elvish")
        ),
        _ => format!("$HOME/{}/{}", WORKFLOW_DIR, COMPLETIONS_CONFIG),
    }
}
