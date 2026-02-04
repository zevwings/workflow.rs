//! 回滚相关类型定义

use std::path::PathBuf;

/// 恢复结果类型：成功列表和失败列表（文件名，错误信息）
pub type RestoreResult = (Vec<String>, Vec<(String, String)>);

/// 备份结果
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// 备份信息
    pub backup_info: BackupInfo,
    /// 备份的二进制文件数量
    pub binary_count: usize,
    /// 备份的补全脚本数量
    pub completion_count: usize,
}

/// 回滚结果
#[derive(Debug, Clone)]
pub struct RollbackResult {
    /// 恢复的二进制文件列表
    pub restored_binaries: Vec<String>,
    /// 恢复的补全脚本列表
    pub restored_completions: Vec<String>,
    /// 失败的二进制文件列表（文件名称和错误信息）
    pub failed_binaries: Vec<(String, String)>,
    /// 失败的补全脚本列表（文件名称和错误信息）
    pub failed_completions: Vec<(String, String)>,
    /// 是否成功重新加载 shell 配置
    pub shell_reload_success: Option<bool>,
    /// Shell 配置文件路径（如果检测到）
    pub shell_config_file: Option<PathBuf>,
}

/// 备份信息
///
/// 存储备份的文件路径和备份目录。
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 备份的二进制文件路径
    pub binary_backups: Vec<(String, PathBuf)>, // (binary_name, backup_path)
    /// 备份的补全脚本路径
    pub completion_backups: Vec<(String, PathBuf)>, // (completion_name, backup_path)
}
