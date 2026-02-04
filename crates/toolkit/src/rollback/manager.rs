//! 回滚管理器
//!
//! 提供更新失败时的回滚机制，包括：
//! - 备份当前版本的二进制文件和补全脚本
//! - 在更新失败时恢复备份的文件
//! - 清理备份文件

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::process::Command;

use thiserror::Error;

use crate::shell::{config_file_path, detect_shell};
use crate::util::DirectoryWalker;
use crate::Paths;

use super::helpers::get_all_completion_files;
use super::reload::Reload;

/// 回滚错误类型
#[derive(Debug, Error)]
pub enum RollbackError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 路径错误
    #[error("Path error: {0}")]
    PathError(#[from] crate::PathError),

    /// 文件系统错误
    #[error("Filesystem error: {0}")]
    FsError(#[from] crate::util::FsError),

    /// 备份二进制文件失败
    #[error("Failed to backup binary file: {src} -> {dest}: {message}")]
    BackupBinaryFailed {
        src: String,
        dest: String,
        message: String,
    },

    /// 恢复二进制文件失败
    #[error("Failed to restore binary file: {src} -> {dest}: {message}")]
    RestoreBinaryFailed {
        src: String,
        dest: String,
        message: String,
    },

    /// 备份补全脚本失败
    #[error("Failed to backup completion script: {src} -> {dest}: {message}")]
    BackupCompletionFailed {
        src: String,
        dest: String,
        message: String,
    },

    /// 清理备份目录失败
    #[error("Failed to remove backup directory: {path}: {reason}")]
    CleanupFailed { path: String, reason: String },
}

// ==================== 类型别名 ====================

/// 恢复结果类型：成功列表和失败列表（文件名，错误信息）
type RestoreResult = (Vec<String>, Vec<(String, String)>);

// ==================== 返回结构体 ====================

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

/// 回滚管理器
///
/// 提供备份和恢复功能，用于更新失败时的回滚操作。
pub struct RollbackManager;

impl RollbackManager {
    /// 创建备份目录
    ///
    /// 在临时目录中创建一个唯一的备份目录。
    ///
    /// # 返回
    ///
    /// 返回备份目录路径。
    fn create_backup_dir() -> Result<PathBuf, RollbackError> {
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let backup_dir = temp_dir.join(format!("workflow-backup-{}", timestamp));

        DirectoryWalker::new(&backup_dir).ensure_exists()?;

        tracing::debug!("Created backup directory: {}", backup_dir.display());
        Ok(backup_dir)
    }

    /// 备份二进制文件
    ///
    /// 备份系统目录（通常是 /usr/local/bin）中的二进制文件到备份目录。
    ///
    /// # 参数
    ///
    /// * `backup_dir` - 备份目录
    /// * `binaries` - 要备份的二进制文件名称列表
    ///
    /// # 返回
    ///
    /// 返回备份的文件路径列表。
    fn backup_binaries(
        backup_dir: &Path,
        binaries: &[&str],
    ) -> Result<Vec<(String, PathBuf)>, RollbackError> {
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let mut backups = Vec::new();

        for binary in binaries {
            let binary_name = Paths::binary_name(binary);
            let source = install_path.join(&binary_name);

            // 如果文件不存在，跳过
            if !source.exists() {
                tracing::debug!(
                    "Binary file does not exist, skipping backup: {}",
                    source.display()
                );
                continue;
            }

            let backup_path = backup_dir.join(binary);

            // 复制文件（Unix 使用 sudo，Windows 直接复制）
            #[cfg(unix)]
            {
                let status = Command::new("sudo")
                    .arg("cp")
                    .arg(&source)
                    .arg(&backup_path)
                    .status()
                    .map_err(|e| RollbackError::BackupBinaryFailed {
                        src: source.display().to_string(),
                        dest: backup_path.display().to_string(),
                        message: e.to_string(),
                    })?;

                if !status.success() {
                    return Err(RollbackError::BackupBinaryFailed {
                        src: source.display().to_string(),
                        dest: backup_path.display().to_string(),
                        message: format!("exit code: {}", status.code().unwrap_or(-1)),
                    });
                }

                // 设置执行权限（仅 Unix）
                let chmod_status =
                    Command::new("chmod").arg("+x").arg(&backup_path).status().map_err(|e| {
                        RollbackError::BackupBinaryFailed {
                            src: source.display().to_string(),
                            dest: backup_path.display().to_string(),
                            message: format!("Failed to set executable permission: {}", e),
                        }
                    })?;

                if !chmod_status.success() {
                    return Err(RollbackError::BackupBinaryFailed {
                        src: source.display().to_string(),
                        dest: backup_path.display().to_string(),
                        message: format!(
                            "Failed to set executable permission (exit code: {})",
                            chmod_status.code().unwrap_or(-1)
                        ),
                    });
                }
            }
            #[cfg(windows)]
            {
                fs::copy(&source, &backup_path).map_err(|e| RollbackError::BackupBinaryFailed {
                    src: source.display().to_string(),
                    dest: backup_path.display().to_string(),
                    message: e.to_string(),
                })?;
            }

            tracing::debug!(
                "Backed up binary file: {} -> {}",
                source.display(),
                backup_path.display()
            );
            backups.push((binary.to_string(), backup_path));
        }

        Ok(backups)
    }

    /// 备份补全脚本
    ///
    /// 备份补全脚本目录中的文件到备份目录。
    ///
    /// # 参数
    ///
    /// * `backup_dir` - 备份目录
    /// * `completion_dir` - 补全脚本目录
    ///
    /// # 返回
    ///
    /// 返回备份的文件路径列表。
    fn backup_completions(
        backup_dir: &Path,
        completion_dir: &Path,
    ) -> Result<Vec<(String, PathBuf)>, RollbackError> {
        let mut backups = Vec::new();

        // 如果补全脚本目录不存在，返回空列表
        if !completion_dir.exists() {
            tracing::debug!(
                "Completion directory does not exist, skipping backup: {}",
                completion_dir.display()
            );
            return Ok(backups);
        }

        // 要备份的补全脚本文件（所有 shell 类型）
        let commands = Paths::command_names();
        let completion_files = get_all_completion_files(commands);

        for file_name in &completion_files {
            let source = completion_dir.join(file_name);

            // 如果文件不存在，跳过
            if !source.exists() {
                tracing::debug!(
                    "Completion script does not exist, skipping backup: {}",
                    source.display()
                );
                continue;
            }

            let backup_path = backup_dir.join(file_name);

            // 复制文件
            fs::copy(&source, &backup_path).map_err(|e| RollbackError::BackupCompletionFailed {
                src: source.display().to_string(),
                dest: backup_path.display().to_string(),
                message: e.to_string(),
            })?;

            tracing::debug!(
                "Backed up completion script: {} -> {}",
                source.display(),
                backup_path.display()
            );
            backups.push((file_name.to_string(), backup_path));
        }

        Ok(backups)
    }

    /// 创建备份
    ///
    /// 备份当前版本的二进制文件和补全脚本。
    ///
    /// # 返回
    ///
    /// 返回 `BackupResult` 结构体，包含备份信息和统计。
    pub fn create_backup() -> Result<BackupResult, RollbackError> {
        tracing::info!("Creating backup");

        // 创建备份目录
        let backup_dir = Self::create_backup_dir()?;

        // 备份二进制文件
        let binaries = Paths::command_names();
        let binary_backups = Self::backup_binaries(&backup_dir, binaries)?;

        // 备份补全脚本（不依赖 shell 检测）
        let completion_dir = Paths::completion_dir()?;
        let completion_backups = Self::backup_completions(&backup_dir, &completion_dir)?;

        // 先获取计数，再转移所有权，避免不必要的 clone
        let binary_count = binary_backups.len();
        let completion_count = completion_backups.len();

        let backup_info = BackupInfo {
            backup_dir,
            binary_backups,
            completion_backups,
        };

        tracing::info!(
            "Backed up {} binary file(s), {} completion script(s)",
            binary_count,
            completion_count
        );

        Ok(BackupResult {
            backup_info,
            binary_count,
            completion_count,
        })
    }

    /// 恢复二进制文件
    ///
    /// 从备份恢复二进制文件到系统目录（通常是 /usr/local/bin）。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    ///
    /// # 返回
    ///
    /// 返回恢复成功的文件列表和失败的文件列表。
    fn restore_binaries(backups: &[(String, PathBuf)]) -> RestoreResult {
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let mut restored = Vec::new();
        let mut failed = Vec::new();

        for (binary_name, backup_path) in backups {
            let target = install_path.join(binary_name);

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                let error_msg = format!("Backup file does not exist: {}", backup_path.display());
                tracing::warn!("{}", error_msg);
                failed.push((binary_name.clone(), error_msg));
                continue;
            }

            // 复制文件（Unix 使用 sudo，Windows 直接复制）
            let result = (|| -> Result<(), String> {
                #[cfg(unix)]
                {
                    let status = Command::new("sudo")
                        .arg("cp")
                        .arg(backup_path)
                        .arg(&target)
                        .status()
                        .map_err(|e| {
                            format!(
                                "Failed to restore binary file: {} -> {}: {}",
                                backup_path.display(),
                                target.display(),
                                e
                            )
                        })?;

                    if !status.success() {
                        return Err(format!(
                            "Failed to restore binary file: {} -> {} (exit code: {})",
                            backup_path.display(),
                            target.display(),
                            status.code().unwrap_or(-1)
                        ));
                    }

                    // 设置执行权限（仅 Unix）
                    Command::new("sudo")
                        .arg("chmod")
                        .arg("+x")
                        .arg(&target)
                        .status()
                        .map_err(|e| {
                            format!(
                                "Failed to set executable permission for restored binary file: {}: {}",
                                target.display(),
                                e
                            )
                        })?;
                }
                #[cfg(windows)]
                {
                    fs::copy(backup_path, &target).map_err(|e| {
                        format!(
                            "Failed to restore binary file: {} -> {}: {}",
                            backup_path.display(),
                            target.display(),
                            e
                        )
                    })?;
                }
                Ok(())
            })();

            match result {
                Ok(_) => {
                    tracing::info!("Restored binary file: {}", binary_name);
                    restored.push(binary_name.clone());
                }
                Err(error_msg) => {
                    tracing::warn!(
                        "Failed to restore binary file {}: {}",
                        binary_name,
                        error_msg
                    );
                    failed.push((binary_name.clone(), error_msg));
                }
            }
        }

        (restored, failed)
    }

    /// 恢复补全脚本
    ///
    /// 从备份恢复补全脚本到补全脚本目录。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    /// * `completion_dir` - 补全脚本目录
    ///
    /// # 返回
    ///
    /// 返回恢复成功的文件列表和失败的文件列表。
    fn restore_completions(backups: &[(String, PathBuf)], completion_dir: &Path) -> RestoreResult {
        // 确保补全脚本目录存在
        if let Err(e) = DirectoryWalker::new(completion_dir).ensure_exists() {
            let error_msg = format!("Failed to ensure completion directory exists: {}", e);
            return (
                Vec::new(),
                backups.iter().map(|(name, _)| (name.clone(), error_msg.clone())).collect(),
            );
        }

        let mut restored = Vec::new();
        let mut failed = Vec::new();

        for (file_name, backup_path) in backups {
            let target = completion_dir.join(file_name);

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                let error_msg = format!("Backup file does not exist: {}", backup_path.display());
                tracing::warn!("{}", error_msg);
                failed.push((file_name.clone(), error_msg));
                continue;
            }

            // 复制文件
            match fs::copy(backup_path, &target) {
                Ok(_) => {
                    tracing::info!("Restored completion script: {}", file_name);
                    restored.push(file_name.clone());
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to restore completion script: {} -> {}: {}",
                        backup_path.display(),
                        target.display(),
                        e
                    );
                    tracing::warn!("{}", error_msg);
                    failed.push((file_name.clone(), error_msg));
                }
            }
        }

        (restored, failed)
    }

    /// 执行回滚
    ///
    /// 从备份恢复所有文件。
    ///
    /// # 参数
    ///
    /// * `backup_info` - 备份信息
    ///
    /// # 返回
    ///
    /// 返回 `RollbackResult` 结构体，包含恢复的文件列表和状态信息。
    pub fn rollback(backup_info: &BackupInfo) -> Result<RollbackResult, RollbackError> {
        tracing::info!("Starting rollback operation");

        let mut restored_binaries = Vec::new();
        let mut restored_completions = Vec::new();
        let mut failed_binaries = Vec::new();
        let mut failed_completions = Vec::new();

        // 恢复二进制文件
        if !backup_info.binary_backups.is_empty() {
            tracing::info!("Restoring binary files");
            let (restored, failed) = Self::restore_binaries(&backup_info.binary_backups);
            restored_binaries = restored;
            failed_binaries = failed;
        }

        // 恢复补全脚本（不依赖 shell 检测）
        if !backup_info.completion_backups.is_empty() {
            tracing::info!("Restoring completion scripts");
            let completion_dir = match Paths::completion_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    tracing::warn!("Failed to get completion directory: {}", e);
                    // 将所有补全脚本标记为失败
                    for (file_name, _) in &backup_info.completion_backups {
                        failed_completions.push((file_name.clone(), format!("{}", e)));
                    }
                    return Ok(RollbackResult {
                        restored_binaries,
                        restored_completions,
                        failed_binaries,
                        failed_completions,
                        shell_reload_success: None,
                        shell_config_file: None,
                    });
                }
            };

            let (restored, failed) =
                Self::restore_completions(&backup_info.completion_backups, &completion_dir);
            restored_completions = restored;
            failed_completions = failed;
        }

        // 尝试重新加载当前 shell 的配置（需要检测 shell，但这是可选操作）
        let (shell_reload_success, shell_config_file) = match detect_shell() {
            Ok(shell) => {
                tracing::info!("Reloading shell configuration for {}", shell);
                let config_file = config_file_path(&shell);

                match Reload::shell(&shell) {
                    Ok(result) => {
                        if result.reloaded {
                            tracing::info!("Shell configuration reloaded successfully");
                        }
                        (Some(result.reloaded), config_file)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to reload shell configuration: {}", e);
                        (Some(false), config_file)
                    }
                }
            }
            Err(e) => {
                tracing::debug!("Failed to detect shell type, skipping reload: {}", e);
                (None, None)
            }
        };

        tracing::info!(
            "Rollback completed: {} binary(ies), {} completion(s) restored",
            restored_binaries.len(),
            restored_completions.len()
        );

        Ok(RollbackResult {
            restored_binaries,
            restored_completions,
            failed_binaries,
            failed_completions,
            shell_reload_success,
            shell_config_file,
        })
    }

    /// 清理备份
    ///
    /// 删除备份目录及其所有内容。
    ///
    /// # 参数
    ///
    /// * `backup_info` - 备份信息
    pub fn cleanup_backup(backup_info: &BackupInfo) -> Result<(), RollbackError> {
        if backup_info.backup_dir.exists() {
            tracing::debug!(
                "Cleaning up backup directory: {}",
                backup_info.backup_dir.display()
            );
            fs::remove_dir_all(&backup_info.backup_dir).map_err(|e| {
                RollbackError::CleanupFailed {
                    path: backup_info.backup_dir.display().to_string(),
                    reason: e.to_string(),
                }
            })?;
            tracing::debug!("Backup directory cleaned up");
        }
        Ok(())
    }
}
